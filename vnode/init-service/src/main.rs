#![no_std]
#![no_main]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::panic::PanicInfo;

use crate::ipc::init_ipc::{InitRequest, InitResponse};
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

// Placeholder for V-Node Configuration parsed from /etc/services
#[derive(Clone)]
struct VNodeConfig {
    entrypoint: String,
    capabilities: Vec<String>, // Simplified for now
    // Add more config fields as needed
}

// Placeholder for a running V-Node's state
struct RunningVNode {
    pid: u64,          // Conceptual Process ID/handle from kernel
    status_channel: u32, // IPC channel for monitoring status or sending signals
    config: VNodeConfig,
}

struct InitService {
    client_chan: VNodeChannel,
    aetherfs_chan: VNodeChannel,
    // Conceptual channel to kernel-vnode-manager
    // kernel_vnode_manager_chan: VNodeChannel,

    service_configs: BTreeMap<String, VNodeConfig>,
    running_vnodes: BTreeMap<String, RunningVNode>,
}

impl InitService {
    fn new(client_chan_id: u32, aetherfs_chan_id: u32) -> Self {
        let client_chan = VNodeChannel::new(client_chan_id);
        let aetherfs_chan = VNodeChannel::new(aetherfs_chan_id);

        log("Init Service: (Conceptual) Reading /etc/services...");
        // Simulate reading service configurations
        let mut service_configs = BTreeMap::new();
        service_configs.insert(
            "aethernet-service".to_string(),
            VNodeConfig {
                entrypoint: "bin/aethernet-service.vnode".to_string(),
                capabilities: alloc::vec!["NetworkAccess".to_string()],
            },
        );
        service_configs.insert(
            "socket-api".to_string(),
            VNodeConfig {
                entrypoint: "bin/socket-api.vnode".to_string(),
                capabilities: alloc::vec!["IPC_CONNECT:aethernet".to_string()],
            },
        );
        log(&alloc::format!(
            "Init Service: Loaded {} service configurations.",
            service_configs.len()
        ));

        Self {
            client_chan,
            aetherfs_chan,
            service_configs,
            running_vnodes: BTreeMap::new(),
        }
    }

    fn handle_request(&mut self, request: InitRequest) -> InitResponse {
        match request {
            InitRequest::ServiceStart { service_name } => {
                if self.running_vnodes.contains_key(&service_name) {
                    return InitResponse::Error(alloc::format!(
                        "Service {} is already running.",
                        service_name
                    ));
                }

                if let Some(config) = self.service_configs.get(&service_name) {
                    // Conceptual: Send IPC to kernel-vnode-manager to start the V-Node
                    // For now, simulate success and assign a dummy PID.
                    let pid = 1000 + self.running_vnodes.len() as u64; // Dummy PID
                    log(&alloc::format!(
                        "Init Service: (Conceptual) Starting service '{}' (PID: {}).",
                        service_name,
                        pid
                    ));

                    let new_vnode = RunningVNode {
                        pid,
                        status_channel: 0, // Placeholder
                        config: config.clone(),
                    };
                    self.running_vnodes.insert(service_name.clone(), new_vnode);
                    InitResponse::Success(alloc::format!(
                        "Service '{}' started with PID {}.",
                        service_name,
                        pid
                    ))
                } else {
                    InitResponse::Error(alloc::format!(
                        "Service '{}' not found in configuration.",
                        service_name
                    ))
                }
            }
            InitRequest::ServiceStatus { service_name } => {
                if let Some(vnode) = self.running_vnodes.get(&service_name) {
                    InitResponse::Status {
                        service_name: service_name.clone(),
                        is_running: true,
                        pid: Some(vnode.pid),
                    }
                } else {
                    InitResponse::Status {
                        service_name: service_name.clone(),
                        is_running: false,
                        pid: None,
                    }
                }
            }
            InitRequest::ServiceRestart { service_name } => {
                // Conceptual: Stop then start
                log(&alloc::format!(
                    "Init Service: (Conceptual) Restarting service '{}'.",
                    service_name
                ));
                if self.running_vnodes.remove(&service_name).is_some() {
                    self.handle_request(InitRequest::ServiceStart {
                        service_name: service_name.clone(),
                    })
                } else {
                    InitResponse::Error(alloc::format!(
                        "Service '{}' not running to restart.",
                        service_name
                    ))
                }
            }
            InitRequest::ServiceStop { service_name } => {
                if self.running_vnodes.remove(&service_name).is_some() {
                    // Conceptual: Send IPC to kernel-vnode-manager to stop the V-Node
                    log(&alloc::format!(
                        "Init Service: (Conceptual) Stopping service '{}'.",
                        service_name
                    ));
                    InitResponse::Success(alloc::format!("Service '{}' stopped.", service_name))
                } else {
                    InitResponse::Error(alloc::format!("Service '{}' not running.", service_name))
                }
            }
        }
    }

    fn run_loop(&mut self) -> ! {
        log("Init Service: Entering main event loop.");
        loop {
            // 1. Process incoming requests from client V-Nodes
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<InitRequest>(&req_data) {
                    log(&alloc::format!(
                        "Init Service: Received InitRequest: {:?}",
                        request
                    ));
                    let response = self.handle_request(request);
                    self.client_chan
                        .send(&response)
                        .unwrap_or_else(|_| log("Init Service: Failed to send response to client."));
                }
            }

            // Conceptual: Monitor running V-Nodes (e.g., check their status channels, or poll kernel-vnode-manager)
            // For now, this is a placeholder.

            // Yield to other V-Nodes to prevent busy-waiting
            unsafe {
                syscall3(SYS_TIME, 0, 0, 0);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Assuming channel ID 6 for init-service for client requests
    // Assuming channel ID 7 for aetherfs for config reads
    let mut init_service = InitService::new(6, 7);
    init_service.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("Init Service V-Node panicked!");
    loop {}
}
