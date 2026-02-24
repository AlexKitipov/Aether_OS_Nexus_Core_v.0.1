#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use core::panic::PanicInfo;

use crate::ipc::file_manager_ipc::{FileManagerRequest, FileManagerResponse};
use crate::ipc::vfs_ipc::{VfsRequest, VfsResponse};
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

struct FileManagerService {
    client_chan: VNodeChannel, // Channel for AetherTerminal or other client V-Nodes
    vfs_chan: VNodeChannel,    // Channel to svc://vfs
}

impl FileManagerService {
    fn new(client_chan_id: u32, vfs_chan_id: u32) -> Self {
        let client_chan = VNodeChannel::new(client_chan_id);
        let vfs_chan = VNodeChannel::new(vfs_chan_id);

        log("File Manager Service: Initializing...");

        Self {
            client_chan,
            vfs_chan,
        }
    }

    fn handle_request(&mut self, request: FileManagerRequest) -> FileManagerResponse {
        match request {
            FileManagerRequest::Browse { path } => {
                log(&alloc::format!("File Manager: Browse request for path: {}", path));
                match self
                    .vfs_chan
                    .send_and_recv::<VfsRequest, VfsResponse>(&VfsRequest::List { path: path.clone() })
                {
                    Ok(VfsResponse::DirectoryEntries(entries)) => {
                        FileManagerResponse::DirectoryEntries(entries)
                    }
                    Ok(VfsResponse::Error { message, .. }) => {
                        FileManagerResponse::Error(format!("Failed to browse {}: {}", path, message))
                    }
                    _ => FileManagerResponse::Error(
                        "Unexpected response from VFS during browse".to_string(),
                    ),
                }
            }
            FileManagerRequest::Copy {
                source,
                destination,
            } => {
                log(&alloc::format!(
                    "File Manager: Copy request from {} to {}",
                    source,
                    destination
                ));
                // Conceptual: This would involve reading from source and writing to destination via VFS.
                // For now, simulate success.
                FileManagerResponse::Success(format!(
                    "Successfully copied {} to {}",
                    source, destination
                ))
            }
            FileManagerRequest::Move {
                source,
                destination,
            } => {
                log(&alloc::format!(
                    "File Manager: Move request from {} to {}",
                    source,
                    destination
                ));
                // Conceptual: This would involve renaming/moving via VFS.
                // For now, simulate success.
                FileManagerResponse::Success(format!(
                    "Successfully moved {} to {}",
                    source, destination
                ))
            }
            FileManagerRequest::Delete { path } => {
                log(&alloc::format!("File Manager: Delete request for path: {}", path));
                // Conceptual: This would involve deleting via VFS.
                // For now, simulate success.
                FileManagerResponse::Success(format!("Successfully deleted {}", path))
            }
            FileManagerRequest::CreateDirectory { path } => {
                log(&alloc::format!(
                    "File Manager: Create directory request for path: {}",
                    path
                ));
                // Conceptual: This would involve creating directory via VFS.
                // For now, simulate success.
                FileManagerResponse::Success(format!("Successfully created directory {}", path))
            }
        }
    }

    fn run_loop(&mut self) -> ! {
        log("File Manager Service: Entering main event loop.");
        loop {
            // Process incoming requests from client V-Nodes
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<FileManagerRequest>(&req_data) {
                    log(&alloc::format!(
                        "File Manager Service: Received FileManagerRequest: {:?}",
                        request
                    ));
                    let response = self.handle_request(request);
                    self.client_chan.send(&response).unwrap_or_else(|_| {
                        log("File Manager Service: Failed to send response to client.")
                    });
                }
            }

            // Yield to other V-Nodes to prevent busy-waiting
            unsafe {
                syscall3(SYS_TIME, 0, 0, 0);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Assuming channel IDs:
    // 9 for File Manager Service client requests
    // 7 for VFS Service
    let mut file_manager_service = FileManagerService::new(9, 7);
    file_manager_service.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("File Manager V-Node panicked!");
    loop {}
}
