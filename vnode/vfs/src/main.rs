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

// Temporary log function for V-Nodes
fn log(msg: &str) {
    unsafe {
        let res = syscall3(
            SYS_LOG,
            msg.as_ptr() as u64,
            msg.len() as u64,
            0, // arg3 is unused for SYS_LOG
        );
        if res != SUCCESS {
            /* Handle log error, maybe panic or fall back */
        }
    }
}

// Placeholder for an open file handle in the VFS
struct OpenFile {
    path: String,
    flags: u32,
    cursor: u64,
    // Conceptual: backend-specific handle (e.g., AetherFS handle, Ramdisk handle)
}

struct VfsService {
    client_chan: VNodeChannel,
    aetherfs_chan: VNodeChannel, // Channel to AetherFS backend
    // ramdisk_chan: VNodeChannel, // Conceptual: Channel to RAM disk backend
    // disk_driver_chan: VNodeChannel, // Conceptual: Channel to block device backend

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
                // Conceptual: Send IPC to AetherFS or other backend to open/create file
                // For now, simulate success and create a dummy OpenFile entry.
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
            VfsRequest::Read { fd, len, offset } => {
                if let Some(file) = self.open_files.get_mut(&fd) {
                    log(&alloc::format!(
                        "VFS: Read request for fd: {}, len: {}, offset: {}",
                        fd,
                        len,
                        offset
                    ));
                    // Conceptual: Send IPC to backend (e.g., AetherFS) to read data
                    // For now, return dummy data
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
                    // Conceptual: Send IPC to backend (e.g., AetherFS) to write data
                    // For now, simulate success
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
                // Conceptual: Send IPC to backend to list directory contents
                // For now, return dummy entries
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
                // Conceptual: Send IPC to backend to get metadata
                // For now, return dummy metadata
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
                    // Conceptual: Send IPC to backend to close file handle
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
            // Process incoming requests from client V-Nodes
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<VfsRequest>(&req_data) {
                    log(&alloc::format!(
                        "VFS Service: Received VfsRequest: {:?}",
                        request
                    ));
                    let response = self.handle_request(request);
                    self.client_chan
                        .send(&response)
                        .unwrap_or_else(|_| log("VFS Service: Failed to send response to client."));
                }
            }

            // Placeholder interaction with the backend channel to mark it active in this scaffold.
            let _ = &self.aetherfs_chan;

            // Yield to other V-Nodes to prevent busy-waiting
            unsafe {
                syscall3(SYS_TIME, 0, 0, 0);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Assuming channel ID 7 for VFS Service for client requests
    // Assuming channel ID 6 for AetherFS backend
    let mut vfs_service = VfsService::new(7, 6);
    vfs_service.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("VFS V-Node panicked!");
    loop {}
}
