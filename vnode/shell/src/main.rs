#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};

use crate::ipc::vnode::VNodeChannel;
use crate::syscall::{syscall3, SYS_LOG, SUCCESS, SYS_TIME};
use crate::ipc::shell_ipc::{ShellRequest, ShellResponse};
use crate::ipc::vfs_ipc::{VfsRequest, VfsResponse, Fd, VfsMetadata};
use crate::ipc::init_ipc::{InitRequest, InitResponse};
use crate::ipc::dns_ipc::{DnsRequest, DnsResponse};

// Temporary log function for V-Nodes
fn log(msg: &str) {
    unsafe {
        let res = syscall3(
            SYS_LOG,
            msg.as_ptr() as u64,
            msg.len() as u64,
            0 // arg3 is unused for SYS_LOG
        );
        if res != SUCCESS { /* Handle log error, maybe panic or fall back */ }
    }
}

// Placeholder for shell state
struct ShellService {
    client_chan: VNodeChannel, // Channel for AetherTerminal or other client V-Nodes
    vfs_chan: VNodeChannel, // Channel to svc://vfs
    init_chan: VNodeChannel, // Channel to svc://init-service
    dns_chan: VNodeChannel, // Channel to svc://dns-resolver

    current_dir: String,
    command_history: Vec<String>,
    // Add more state as needed, e.g., environmental variables
}

impl ShellService {
    fn new(client_chan_id: u32, vfs_chan_id: u32, init_chan_id: u32, dns_chan_id: u32) -> Self {
        let client_chan = VNodeChannel::new(client_chan_id);
        let vfs_chan = VNodeChannel::new(vfs_chan_id);
        let init_chan = VNodeChannel::new(init_chan_id);
        let dns_chan = VNodeChannel::new(dns_chan_id);

        log("Shell Service: Initializing...");

        Self {
            client_chan,
            vfs_chan,
            init_chan,
            dns_chan,
            current_dir: String::from("/"), // Default to root
            command_history: Vec::new(),
        }
    }

    fn handle_request(&mut self, request: ShellRequest) -> ShellResponse {
        match request {
            ShellRequest::ExecuteCommand { command, args } => {
                self.command_history.push(format!("{} {}", command, args.join(" ")));
                log(&alloc::format!("Shell: Executing command: {} with args: {:?}", command, args));

                // Conceptual: Implement built-in commands or forward to init-service
                match command.as_str() {
                    "cd" => {
                        if let Some(path) = args.get(0) {
                            return self.handle_change_directory(path.to_string());
                        } else {
                            return ShellResponse::Error("cd: missing argument".to_string());
                        }
                    },
                    "ls" => {
                        // Conceptual: IPC to VFS to list directory
                        match self.vfs_chan.send_and_recv::<VfsRequest, VfsResponse>(&VfsRequest::List { path: self.current_dir.clone() }) {
                            Ok(VfsResponse::DirectoryEntries(entries)) => {
                                let mut output = String::new();
                                for (name, _) in entries {
                                    output.push_str(&name);
                                    output.push_str("\n");
                                }
                                ShellResponse::CommandOutput { stdout: output, stderr: String::new(), exit_code: 0 }
                            },
                            Ok(VfsResponse::Error { message, .. }) => ShellResponse::Error(format!("ls: {}", message)),
                            _ => ShellResponse::Error("ls: Unexpected response from VFS".to_string()),
                        }
                    },
                    "ping" => {
                        if let Some(hostname) = args.get(0) {
                            match self.dns_chan.send_and_recv::<DnsRequest, DnsResponse>(&DnsRequest::ResolveHostname { hostname: hostname.clone() }) {
                                Ok(DnsResponse::ResolvedHostname { ip_address, .. }) => {
                                    ShellResponse::CommandOutput { stdout: format!("Pinging {} ({}.{}.{}.{})", hostname, ip_address[0], ip_address[1], ip_address[2], ip_address[3]), stderr: String::new(), exit_code: 0 }
                                },
                                Ok(DnsResponse::NotFound { query }) => ShellResponse::Error(format!("ping: Host '{}' not found.", query)),
                                Ok(DnsResponse::Error { message }) => ShellResponse::Error(format!("ping: DNS error: {}", message)),
                                _ => ShellResponse::Error("ping: Unexpected response from DNS Resolver".to_string()),
                            }
                        } else {
                            ShellResponse::Error("ping: missing hostname".to_string())
                        }
                    },
                    "start" => {
                        if let Some(service_name) = args.get(0) {
                            match self.init_chan.send_and_recv::<InitRequest, InitResponse>(&InitRequest::ServiceStart { service_name: service_name.clone() }) {
                                Ok(InitResponse::Success(msg)) => ShellResponse::Success(msg),
                                Ok(InitResponse::Error(msg)) => ShellResponse::Error(format!("start: {}", msg)),
                                _ => ShellResponse::Error("start: Unexpected response from Init Service".to_string()),
                            }
                        } else {
                            ShellResponse::Error("start: missing service name".to_string())
                        }
                    }
                    // Add more built-in commands or forward to init-service for app execution
                    _ => ShellResponse::CommandOutput { stdout: format!("Command '{}' not found.\n", command), stderr: String::new(), exit_code: 127 },
                }
            },
            ShellRequest::ChangeDirectory { path } => {
                self.handle_change_directory(path)
            },
            ShellRequest::GetCurrentDirectory => {
                ShellResponse::CurrentDirectory(self.current_dir.clone())
            },
        }
    }

    fn handle_change_directory(&mut self, path: String) -> ShellResponse {
        // Conceptual: Validate path with VFS or simplify
        // For now, allow any path for simplicity
        // In a real system, would check if path is a directory and exists
        if path == ".." {
            // Go up one level
            if let Some(last_slash) = self.current_dir.rfind('/') {
                if last_slash == 0 && self.current_dir.len() > 1 {
                    self.current_dir = String::from("/");
                } else if last_slash > 0 {
                    self.current_dir.truncate(last_slash);
                }
            }
        } else if path.starts_with('/') {
            self.current_dir = path;
        } else {
            // Relative path
            if !self.current_dir.ends_with('/') {
                self.current_dir.push('/');
            }
            self.current_dir.push_str(&path);
        }
        ShellResponse::Success(format!("Changed directory to {}", self.current_dir))
    }

    fn run_loop(&mut self) -> ! {
        log("Shell Service: Entering main event loop.");
        loop {
            // Process incoming requests from client V-Nodes
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<ShellRequest>(&req_data) {
                    log(&alloc::format!("Shell Service: Received ShellRequest: {:?}", request));
                    let response = self.handle_request(request);
                    self.client_chan.send(&response).unwrap_or_else(|_| log("Shell Service: Failed to send response to client."));
                }
            }

            // Yield to other V-Nodes to prevent busy-waiting
            unsafe { syscall3(SYS_TIME, 0, 0, 0); }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Assuming channel IDs:
    // 8 for Shell Service client requests (e.g., AetherTerminal)
    // 7 for VFS Service
    // 6 for Init Service
    // 5 for DNS Resolver
    let mut shell_service = ShellService::new(8, 7, 6, 5);
    shell_service.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("Shell V-Node panicked!");
    loop {}
}
