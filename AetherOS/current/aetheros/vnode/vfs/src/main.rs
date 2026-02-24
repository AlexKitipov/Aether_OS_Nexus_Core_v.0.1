#![no_std]
#![no_main]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::panic::PanicInfo;

use crate::ipc::vfs_ipc::{Fd, VfsMetadata, VfsRequest, VfsResponse};
use crate::ipc::vnode::VNodeChannel;
use crate::syscall::{syscall3, SUCCESS, SYS_LOG, SYS_TIME};

fn log(msg: &str) {
    unsafe {
        let res = syscall3(SYS_LOG, msg.as_ptr() as u64, msg.len() as u64, 0);
        if res != SUCCESS {
            // Logging is best-effort in this early V-Node implementation.
        }
    }
}

struct OpenFile {
    path: String,
    flags: u32,
    cursor: u64,
}

struct VfsService {
    client_chan: VNodeChannel,
    aetherfs_chan: VNodeChannel,
    next_fd: Fd,
    open_files: BTreeMap<Fd, OpenFile>,
}

impl VfsService {
    fn new(client_chan_id: u32, aetherfs_chan_id: u32) -> Self {
        let client_chan = VNodeChannel::new(client_chan_id);
        let aetherfs_chan = VNodeChannel::new(aetherfs_chan_id);

        log("VFS Service: Initializing...");

        Self {
            client_chan,
            aetherfs_chan,
            next_fd: 1,
            open_files: BTreeMap::new(),
        }
    }

    fn handle_request(&mut self, request: VfsRequest) -> VfsResponse {
        match request {
            VfsRequest::Open { path, flags } => {
                log(&alloc::format!(
                    "VFS: Open request for path: {} with flags: {}",
                    path,
                    flags
                ));

                let fd = self.next_fd;
                self.next_fd += 1;
                self.open_files.insert(
                    fd,
                    OpenFile {
                        path: path.clone(),
                        flags,
                        cursor: 0,
                    },
                );
                log(&alloc::format!("VFS: Opened {} as fd {}", path, fd));
                VfsResponse::Success(fd as i32)
            }
            VfsRequest::Read { fd, len: _, offset } => {
                if let Some(file) = self.open_files.get_mut(&fd) {
                    log(&alloc::format!(
                        "VFS: Read request for fd: {}, offset: {}",
                        fd,
                        offset
                    ));

                    let dummy_data = alloc::format!(
                        "dummy_data_from_file_{}_at_offset_{}",
                        file.path,
                        offset
                    )
                    .as_bytes()
                    .to_vec();

                    file.cursor = offset + dummy_data.len() as u64;
                    VfsResponse::Data(dummy_data)
                } else {
                    log(&alloc::format!("VFS: Read failed, bad fd: {}", fd));
                    VfsResponse::Error {
                        code: 9,
                        message: "Bad file descriptor".to_string(),
                    }
                }
            }
            VfsRequest::Write { fd, data, offset } => {
                if let Some(file) = self.open_files.get_mut(&fd) {
                    log(&alloc::format!(
                        "VFS: Write request for fd: {}, len: {}, offset: {}",
                        fd,
                        data.len(),
                        offset
                    ));

                    file.cursor = offset + data.len() as u64;
                    VfsResponse::Success(data.len() as i32)
                } else {
                    log(&alloc::format!("VFS: Write failed, bad fd: {}", fd));
                    VfsResponse::Error {
                        code: 9,
                        message: "Bad file descriptor".to_string(),
                    }
                }
            }
            VfsRequest::List { path } => {
                log(&alloc::format!("VFS: List request for path: {}", path));

                let mut entries = BTreeMap::new();
                entries.insert(
                    "file1.txt".to_string(),
                    VfsMetadata {
                        is_dir: false,
                        size: 100,
                        created: 0,
                        modified: 0,
                        permissions: 0o644,
                    },
                );
                entries.insert(
                    "subdir/".to_string(),
                    VfsMetadata {
                        is_dir: true,
                        size: 0,
                        created: 0,
                        modified: 0,
                        permissions: 0o755,
                    },
                );
                VfsResponse::DirectoryEntries(entries)
            }
            VfsRequest::Stat { path } => {
                log(&alloc::format!("VFS: Stat request for path: {}", path));
                VfsResponse::Metadata(VfsMetadata {
                    is_dir: false,
                    size: 512,
                    created: 0,
                    modified: 0,
                    permissions: 0o644,
                })
            }
            VfsRequest::Close { fd } => {
                if self.open_files.remove(&fd).is_some() {
                    log(&alloc::format!("VFS: Closed fd: {}", fd));
                    VfsResponse::Success(0)
                } else {
                    log(&alloc::format!("VFS: Close failed, bad fd: {}", fd));
                    VfsResponse::Error {
                        code: 9,
                        message: "Bad file descriptor".to_string(),
                    }
                }
            }
        }
    }

    fn run_loop(&mut self) -> ! {
        log("VFS Service: Entering main event loop.");
        loop {
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<VfsRequest>(&req_data) {
                    log(&alloc::format!("VFS Service: Received VfsRequest: {:?}", request));
                    let response = self.handle_request(request);
                    self.client_chan
                        .send(&response)
                        .unwrap_or_else(|_| log("VFS Service: Failed to send response to client."));
                }
            }

            // TODO(v0.2): route requests to backend via `self.aetherfs_chan`.
            let _ = &self.aetherfs_chan;

            unsafe {
                syscall3(SYS_TIME, 0, 0, 0);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vfs_service = VfsService::new(7, 6);
    vfs_service.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("VFS V-Node panicked!");
    loop {}
}
