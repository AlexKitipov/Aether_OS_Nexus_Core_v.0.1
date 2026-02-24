#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use crate::ipc::vnode::VNodeChannel;
use crate::syscall::{syscall3, E_ERROR, SUCCESS, SYS_IRQ_REGISTER, SYS_LOG, SYS_NET_RX_POLL};

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
            // TODO: handle log error (panic or serial fallback)
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // For this V-Node, let's assume its channel ID is passed by the kernel
    // or is a well-known ID for the net-bridge service.
    // For simplicity, we'll hardcode it to 2 for now, assuming 1 is for registry.
    let mut chan = VNodeChannel::new(2);

    log("Net-Bridge V-Node starting up...");

    // Register IRQ 11 (common for VirtIO-Net) for this V-Node's channel
    unsafe {
        let res = syscall3(
            SYS_IRQ_REGISTER,
            11,             // IRQ number for VirtIO-Net
            chan.id as u64, // Channel ID to route IRQ events
            0,              // arg3 is unused
        );
        if res == SUCCESS {
            log("Net-Bridge: Registered IRQ 11 successfully.");
        } else {
            log("Net-Bridge: Failed to register IRQ 11.");
            // In a real scenario, handle error: maybe exit or retry
        }
    }

    // Buffer for receiving network packets
    let mut rx_buffer = [0u8; 1536]; // Max Ethernet frame size

    loop {
        // Wait for IRQ events from the kernel (on its own channel)
        // This blocks until an IRQ event is routed to this V-Node's channel.
        match chan.recv_blocking() {
            Ok(_msg_data) => {
                // In a real scenario, msg_data would contain details about the IRQ event.
                log("Net-Bridge: Received IRQ event. Polling for packets...");

                // Poll for incoming network packets
                unsafe {
                    // SYS_NET_RX_POLL expects a handle to a pre-allocated buffer
                    let len = syscall3(
                        SYS_NET_RX_POLL,
                        0, // Interface ID (from cap, assumed 0 for now)
                        rx_buffer.as_mut_ptr() as u64,
                        rx_buffer.len() as u64, // Max buffer length
                    );

                    if len > SUCCESS {
                        log("Net-Bridge: Received packet!");
                        // In a real scenario, process packet (e.g., send to svc://aethernet).
                    } else if len == SUCCESS {
                        log("Net-Bridge: Poll returned no packets (spurious or already handled IRQ).");
                    } else if len == E_ERROR {
                        log("Net-Bridge: SYS_NET_RX_POLL returned an error.");
                    } else {
                        // Other error codes from syscall_dispatch
                        log("Net-Bridge: SYS_NET_RX_POLL returned unknown error code.");
                    }
                }
            }
            Err(_) => {
                log("Net-Bridge: Error receiving IPC message on its channel.");
                // Potentially fatal error, depending on error handling strategy.
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log("Net-Bridge V-Node panicked!");
    loop {}
}
