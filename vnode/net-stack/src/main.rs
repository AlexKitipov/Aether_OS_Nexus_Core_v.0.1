#![no_std]
#![no_main]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::socket::{tcp, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};

use crate::ipc::net_ipc::{NetPacketMsg, NetStackRequest, NetStackResponse};
use crate::ipc::vnode::VNodeChannel;
use crate::syscall::{syscall2, syscall3, SUCCESS, SYS_LOG, SYS_TIME};

mod aethernet_device;
use aethernet_device::AetherNetDevice;

fn log(msg: &str) {
    unsafe {
        let _ = syscall3(SYS_LOG, msg.as_ptr() as u64, msg.len() as u64, 0);
    }
}

fn get_current_time_ms() -> u64 {
    unsafe { syscall2(SYS_TIME, 0, 0) * 10 }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut own_chan = VNodeChannel::new(3);
    let mut bridge_data_chan = VNodeChannel::new(2);

    log("AetherNet Service V-Node starting up...");

    let mut device = AetherNetDevice::new(0, bridge_data_chan.id);

    let ethernet_addr = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);
    let config = Config::new(HardwareAddress::Ethernet(ethernet_addr));
    let mut iface = Interface::new(
        config,
        &mut device,
        Instant::from_millis(get_current_time_ms()),
    );

    iface.update_ip_addrs(|addrs| {
        let _ = addrs.push(IpCidr::new(IpAddress::v4(10, 0, 2, 15), 24));
    });

    let mut sockets = SocketSet::new(Vec::new());
    let mut next_socket_handle: u32 = 1;
    let mut socket_map = BTreeMap::new();

    loop {
        let timestamp = Instant::from_millis(get_current_time_ms());

        if let Ok(Some(net_msg_data)) = bridge_data_chan.recv_non_blocking() {
            if let Ok(net_packet_msg) = postcard::from_bytes::<NetPacketMsg>(&net_msg_data) {
                match net_packet_msg {
                    NetPacketMsg::RxPacket { dma_handle, len } => {
                        log("AetherNet: Received RxPacket from net-bridge.");
                        device.enqueue_rx_packet(dma_handle, len);
                    }
                    NetPacketMsg::TxPacketAck => {
                        log("AetherNet: Received TxPacketAck from net-bridge.");
                    }
                    _ => log("AetherNet: Received unexpected NetPacketMsg from net-bridge."),
                }
            }
        }

        let _ = iface.poll(timestamp, &mut device, &mut sockets);

        if let Ok(Some(req_data)) = own_chan.recv_non_blocking() {
            let response = if let Ok(request) = postcard::from_bytes::<NetStackRequest>(&req_data) {
                match request {
                    NetStackRequest::OpenSocket(sock_type, local_port) => {
                        let handle = next_socket_handle;
                        next_socket_handle += 1;

                        let smoltcp_handle = match sock_type {
                            0 => {
                                let rx = tcp::SocketBuffer::new(vec![0; 2048]);
                                let tx = tcp::SocketBuffer::new(vec![0; 2048]);
                                let mut s = tcp::Socket::new(rx, tx);
                                if local_port != 0 {
                                    let _ = s.listen(local_port);
                                }
                                sockets.add(s)
                            }
                            1 => {
                                let rx_meta = vec![udp::PacketMetadata::EMPTY; 8];
                                let rx_data = vec![0; 2048];
                                let tx_meta = vec![udp::PacketMetadata::EMPTY; 8];
                                let tx_data = vec![0; 2048];
                                let mut s = udp::Socket::new(
                                    udp::PacketBuffer::new(rx_meta, rx_data),
                                    udp::PacketBuffer::new(tx_meta, tx_data),
                                );
                                if local_port != 0 {
                                    let _ = s.bind(local_port);
                                }
                                sockets.add(s)
                            }
                            _ => {
                                own_chan
                                    .send(&NetStackResponse::Error(100))
                                    .unwrap_or_else(|_| log("AetherNet: Failed to send response."));
                                continue;
                            }
                        };

                        socket_map.insert(handle, smoltcp_handle);
                        NetStackResponse::SocketOpened(handle)
                    }
                    NetStackRequest::Send(handle, data) => match socket_map.get(&handle).copied() {
                        Some(h) => {
                            let socket = sockets.get_mut::<tcp::Socket>(h);
                            if socket.can_send() {
                                let _ = socket.send_slice(&data);
                                NetStackResponse::Success
                            } else {
                                NetStackResponse::Error(101)
                            }
                        }
                        None => NetStackResponse::Error(103),
                    },
                    NetStackRequest::SendTo(handle, remote_ip, remote_port, data) => {
                        match socket_map.get(&handle).copied() {
                            Some(h) => {
                                let socket = sockets.get_mut::<udp::Socket>(h);
                                let endpoint = smoltcp::wire::IpEndpoint::new(
                                    IpAddress::v4(
                                        remote_ip[0],
                                        remote_ip[1],
                                        remote_ip[2],
                                        remote_ip[3],
                                    ),
                                    remote_port,
                                );
                                let _ = socket.send_slice(&data, endpoint);
                                NetStackResponse::Success
                            }
                            None => NetStackResponse::Error(103),
                        }
                    }
                    NetStackRequest::Recv(handle) => match socket_map.get(&handle).copied() {
                        Some(h) => {
                            // Conservative fixed-size receive buffer for IPC replies.
                            let mut buf = [0u8; 2048];

                            if let Ok(size) = sockets.get_mut::<tcp::Socket>(h).recv_slice(&mut buf)
                            {
                                NetStackResponse::Data(buf[..size].to_vec())
                            } else if let Ok((size, _)) =
                                sockets.get_mut::<udp::Socket>(h).recv_slice(&mut buf)
                            {
                                NetStackResponse::Data(buf[..size].to_vec())
                            } else {
                                NetStackResponse::Data(Vec::new())
                            }
                        }
                        None => NetStackResponse::Error(103),
                    },
                    NetStackRequest::CloseSocket(handle) => {
                        if let Some(h) = socket_map.remove(&handle) {
                            sockets.remove(h);
                            NetStackResponse::Success
                        } else {
                            NetStackResponse::Error(103)
                        }
                    }
                }
            } else {
                NetStackResponse::Error(104)
            };

            own_chan
                .send(&response)
                .unwrap_or_else(|_| log("AetherNet: Failed to send response."));
        }
    }
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    if unsafe { syscall3(SYS_LOG, b"panic\0".as_ptr() as u64, 5, 0) } != SUCCESS {
        loop {}
    }
    loop {}
}
