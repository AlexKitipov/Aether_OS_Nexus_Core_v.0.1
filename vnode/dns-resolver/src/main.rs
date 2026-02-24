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
use crate::ipc::socket_ipc::{SocketRequest, SocketResponse, SocketFd};
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

// Placeholder for DNS cache entry
struct DnsCacheEntry {
    ip_address: [u8; 4],
    expires_at_ms: u64,
}

// Main struct for the DNS Resolver V-Node logic
struct DnsResolver {
    client_chan: VNodeChannel,
    socket_chan: VNodeChannel,
    aetherfs_chan: VNodeChannel,
    dns_cache: BTreeMap<String, DnsCacheEntry>,
    dns_servers: Vec<[u8; 4]>,
    dns_socket_fd: SocketFd,
}

impl DnsResolver {
    fn new(client_chan_id: u32, socket_chan_id: u32, aetherfs_chan_id: u32) -> Self {
        let mut client_chan = VNodeChannel::new(client_chan_id);
        let mut socket_chan = VNodeChannel::new(socket_chan_id);
        let mut aetherfs_chan = VNodeChannel::new(aetherfs_chan_id);

        log("DNS Resolver: (Conceptual) Reading /etc/network/resolv.conf...");
        // For now, hardcode a dummy DNS server.
        let mut dns_servers = Vec::new();
        dns_servers.push([8, 8, 8, 8]); // Google DNS as a dummy
        log(&alloc::format!("DNS Resolver: Using DNS server: {}.{}.{}.{}", dns_servers[0][0], dns_servers[0][1], dns_servers[0][2], dns_servers[0][3]));

        // Conceptual: Open a UDP socket with socket-api for DNS queries
        let dns_socket_fd: SocketFd = match socket_chan.send_and_recv::<SocketRequest, SocketResponse>(&SocketRequest::Socket { domain: 2, ty: 2, protocol: 0 }) {
            Ok(SocketResponse::Success(fd)) => {
                log(&alloc::format!("DNS Resolver: Opened UDP socket with fd: {}", fd));
                fd as SocketFd
            },
            _ => { log("DNS Resolver: Failed to open UDP socket with socket-api. Fatal error."); loop {} }
        };

        Self {
            client_chan,
            socket_chan,
            aetherfs_chan,
            dns_cache: BTreeMap::new(),
            dns_servers,
            dns_socket_fd,
        }
    }

    // This function encapsulates the network lookup logic for a hostname
    fn perform_network_lookup(&mut self, hostname: &String, current_time_ms: u64) -> DnsResponse {
        log(&alloc::format!("DNS Resolver: Performing network lookup for {}", hostname));

        // Conceptual: Build DNS query packet (e.g., using a simple DNS query library or manual construction)
        // For now, let's simulate a successful lookup for "example.com" and a failure for others.
        let resolved_ip: Option<[u8; 4]> = if hostname == "example.com" {
            Some([192, 0, 2, 1]) // Dummy IP for example.com
        } else {
            // Simulate sending a DNS query packet over UDP via socket-api
            // and receiving a response.
            // For now, just return a dummy failure
            None
        };

        if let Some(ip_addr) = resolved_ip {
            // Cache the result
            let expires_at_ms = current_time_ms + 60_000; // Cache for 60 seconds
            self.dns_cache.insert(hostname.clone(), DnsCacheEntry { ip_address: ip_addr, expires_at_ms });
            log(&alloc::format!("DNS Resolver: Resolved {} to {}.{}.{}.{} (cached)", hostname, ip_addr[0], ip_addr[1], ip_addr[2], ip_addr[3]));
            DnsResponse::ResolvedHostname { hostname: hostname.clone(), ip_address: ip_addr }
        } else {
            log(&alloc::format!("DNS Resolver: Failed to resolve {}", hostname));
            DnsResponse::NotFound { query: hostname.clone() }
        }
    }

    fn run_loop(&mut self) -> ! {
        loop {
            let current_time_ms = unsafe { syscall3(SYS_TIME, 0, 0, 0) * 10 }; // Assuming 1 tick = 10 ms

            // 1. Process incoming DNS queries from client V-Nodes
            if let Ok(Some(req_data)) = self.client_chan.recv_non_blocking() {
                if let Ok(request) = postcard::from_bytes::<DnsRequest>(&req_data) {
                    log(&alloc::format!("DNS Resolver: Received DnsRequest: {:?}", request));

                    let response = match request {
                        DnsRequest::ResolveHostname { hostname } => {
                            // Check cache first
                            if let Some(entry) = self.dns_cache.get(&hostname) {
                                if current_time_ms < entry.expires_at_ms {
                                    log(&alloc::format!("DNS Resolver: Cache hit for {}: {}.{}.{}.{}", hostname, entry.ip_address[0], entry.ip_address[1], entry.ip_address[2], entry.ip_address[3]));
                                    DnsResponse::ResolvedHostname { hostname: hostname.clone(), ip_address: entry.ip_address }
                                } else {
                                    log(&alloc::format!("DNS Resolver: Cache expired for {}", hostname));
                                    self.dns_cache.remove(&hostname);
                                    // Fall through to network lookup
                                    self.perform_network_lookup(&hostname, current_time_ms)
                                }
                            } else {
                                log(&alloc::format!("DNS Resolver: Cache miss for {}, performing network lookup.", hostname));
                                self.perform_network_lookup(&hostname, current_time_ms)
                            }
                        },
                    };
                    self.client_chan.send(&response).unwrap_or_else(|_| log("DNS Resolver: Failed to send response to client."));
                }
            }

            // Yield to other V-Nodes to prevent busy-waiting
            unsafe { syscall3(SYS_TIME, 0, 0, 0); }
        }
    }
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut dns_resolver = DnsResolver::new(5, 4, 6); // Client, Socket, AetherFS channel IDs
    dns_resolver.run_loop();
}

#[panic_handler]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    log("DNS Resolver V-Node panicked!");
    loop {}
}
