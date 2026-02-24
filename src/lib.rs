#![no_std]
extern crate alloc;

pub mod cid;
pub mod manifest;
pub mod trust;
pub mod arp_dht;
pub use swarm_engine::nexus_net_transport; // Re-export NexusNetTransport module for direct access
pub mod swarm_engine;
pub mod ipc;
pub mod syscall;
pub mod kernel;
pub mod vnode;
pub mod socket_ipc;
pub mod dns_ipc;
pub mod init_ipc;
pub mod vfs_ipc;
pub mod shell_ipc;
pub mod file_manager_ipc;
pub mod mail_ipc;
pub mod model_runtime_ipc; // New: Publicly expose the model_runtime_ipc module
