#![no_std]
#![no_main]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use crate::ipc::net_ipc::{NetStackRequest, NetStackResponse};
use crate::ipc::socket_ipc::{SocketFd, SocketRequest, SocketResponse};
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

struct SocketInfo {
    net_socket_handle: u32,
    socket_type: i32,
    is_listening: bool,
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut client_chan = VNodeChannel::new(4);
    let mut net_chan = VNodeChannel::new(3);

    log("Socket API V-Node starting up...");

    let mut next_fd: SocketFd = 1;
    let mut sockets: BTreeMap<SocketFd, SocketInfo> = BTreeMap::new();
    let mut pending_accept_fci: BTreeMap<u32, SocketFd> = BTreeMap::new();

    loop {
        if let Ok(Some(req_data)) = client_chan.recv_non_blocking() {
            if let Ok(request) = postcard::from_bytes::<SocketRequest>(&req_data) {
                log(&alloc::format!(
                    "SocketAPI: Received request from client: {:?}",
                    request
                ));

                let response = match request {
                    SocketRequest::Socket {
                        domain: _,
                        ty,
                        protocol: _,
                    } => {
                        let net_sock_type = match ty {
                            1 => 0,
                            2 => 1,
                            _ => {
                                log("SocketAPI: Unsupported socket type.");
                                client_chan
                                    .send(&SocketResponse::Error(
                                        100,
                                        "Unsupported socket type".to_string(),
                                    ))
                                    .unwrap_or_else(|_| {
                                        log("SocketAPI: Failed to send unsupported-type response.")
                                    });
                                continue;
                            }
                        };

                        match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                            &NetStackRequest::OpenSocket(net_sock_type, 0),
                        ) {
                            Ok(NetStackResponse::SocketOpened(net_handle)) => {
                                let fd = next_fd;
                                next_fd += 1;
                                sockets.insert(
                                    fd,
                                    SocketInfo {
                                        net_socket_handle: net_handle,
                                        socket_type: ty,
                                        is_listening: false,
                                    },
                                );
                                SocketResponse::Success(fd as i32)
                            }
                            Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                code as i32,
                                "Failed to open socket in AetherNet".to_string(),
                            ),
                            _ => SocketResponse::Error(
                                -1,
                                "Unexpected response from AetherNet during Socket open"
                                    .to_string(),
                            ),
                        }
                    }
                    SocketRequest::Bind { fd, addr: _, port } => {
                        if let Some(socket_info) = sockets.get(&fd) {
                            match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                                &NetStackRequest::OpenSocket(socket_info.socket_type as u32, port),
                            ) {
                                Ok(NetStackResponse::SocketOpened(net_handle)) => {
                                    if let Some(sock) = sockets.get_mut(&fd) {
                                        sock.net_socket_handle = net_handle;
                                    }
                                    SocketResponse::Success(0)
                                }
                                Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                    code as i32,
                                    "Failed to bind socket in AetherNet".to_string(),
                                ),
                                _ => SocketResponse::Error(
                                    -1,
                                    "Unexpected response from AetherNet during Bind".to_string(),
                                ),
                            }
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                    SocketRequest::Listen { fd, backlog: _ } => {
                        if let Some(socket_info) = sockets.get_mut(&fd) {
                            socket_info.is_listening = true;
                            pending_accept_fci.insert(socket_info.net_socket_handle, fd);
                            SocketResponse::Success(0)
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                    SocketRequest::Accept { fd: _ } => {
                        log("SocketAPI: Accept is conceptual; requires AetherNet callback.");
                        SocketResponse::Error(11, "Operation would block (EWOULDBLOCK)".to_string())
                    }
                    SocketRequest::Connect { fd, addr, port } => {
                        if let Some(socket_info) = sockets.get_mut(&fd) {
                            match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                                &NetStackRequest::SendTo(
                                    socket_info.net_socket_handle,
                                    addr,
                                    port,
                                    Vec::new(),
                                ),
                            ) {
                                Ok(NetStackResponse::Success) => SocketResponse::Success(0),
                                Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                    code as i32,
                                    "Failed to connect via AetherNet".to_string(),
                                ),
                                _ => SocketResponse::Error(
                                    -1,
                                    "Unexpected response from AetherNet during Connect"
                                        .to_string(),
                                ),
                            }
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                    SocketRequest::Send { fd, data } => {
                        if let Some(socket_info) = sockets.get(&fd) {
                            match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                                &NetStackRequest::Send(socket_info.net_socket_handle, data),
                            ) {
                                Ok(NetStackResponse::Success) => SocketResponse::Success(0),
                                Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                    code as i32,
                                    "Failed to send via AetherNet".to_string(),
                                ),
                                _ => SocketResponse::Error(
                                    -1,
                                    "Unexpected response from AetherNet during Send".to_string(),
                                ),
                            }
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                    SocketRequest::Recv { fd, len: _ } => {
                        if let Some(socket_info) = sockets.get(&fd) {
                            match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                                &NetStackRequest::Recv(socket_info.net_socket_handle),
                            ) {
                                Ok(NetStackResponse::Data(data)) => SocketResponse::Data(data),
                                Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                    code as i32,
                                    "Failed to receive via AetherNet".to_string(),
                                ),
                                _ => SocketResponse::Error(
                                    -1,
                                    "Unexpected response from AetherNet during Recv".to_string(),
                                ),
                            }
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                    SocketRequest::Close { fd } => {
                        if let Some(socket_info) = sockets.remove(&fd) {
                            pending_accept_fci.remove(&socket_info.net_socket_handle);
                            match net_chan.send_and_recv::<NetStackRequest, NetStackResponse>(
                                &NetStackRequest::CloseSocket(socket_info.net_socket_handle),
                            ) {
                                Ok(NetStackResponse::Success) => SocketResponse::Success(0),
                                Ok(NetStackResponse::Error(code)) => SocketResponse::Error(
                                    code as i32,
                                    "Failed to close socket in AetherNet".to_string(),
                                ),
                                _ => SocketResponse::Error(
                                    -1,
                                    "Unexpected response from AetherNet during Close".to_string(),
                                ),
                            }
                        } else {
                            SocketResponse::Error(-1, "Bad file descriptor".to_string())
                        }
                    }
                };

                client_chan
                    .send(&response)
                    .unwrap_or_else(|_| log("SocketAPI: Failed to send response to client."));
            }
        }

        // TODO(v0.2): process unsolicited events from `net_chan` (accept/data callbacks).
        let _ = &pending_accept_fci;

        unsafe {
            syscall3(SYS_TIME, 0, 0, 0);
        }
    }
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("Socket API V-Node panicked!");
    loop {}
}
