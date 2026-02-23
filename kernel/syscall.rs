extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

mod console {
    // Temporary stub for console output.
    // In a real kernel, these would write to serial port or framebuffer.
    pub fn print(_s: &str) {}
    pub fn print_u64(_n: u64) {}
    pub fn print_hex(_n: u64) {}
}

mod task {
    // Temporary stub for task management.
    pub struct Task;

    impl Task {
        pub fn has_cap(&self, cap: crate::caps::Capability) -> bool {
            // For stub, assume all capabilities are granted if not explicitly checked for testing.
            match cap {
                crate::caps::Capability::NetworkAccess => true,
                crate::caps::Capability::LogWrite => true,
                crate::caps::Capability::TimeRead => true,
                _ => false,
            }
        }
    }

    pub fn get_current_task() -> Task {
        Task
    }

    pub fn block_current_on_channel(_id: u32) {}
}

mod ipc {
    extern crate alloc;

    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use spin::Mutex;

    pub type ChannelId = u32;

    pub struct Message {
        pub data: Vec<u8>,
    }

    pub struct KernelChannel {
        queue: VecDeque<Message>,
    }

    static CHANNELS: Mutex<[Option<KernelChannel>; 32]> = Mutex::new([
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
    ]);

    pub fn kernel_send(id: ChannelId, data: &[u8]) -> Result<(), &'static str> {
        let mut chans = CHANNELS.lock();
        if let Some(chan) = chans[id as usize].as_mut() {
            chan.queue.push_back(Message { data: data.to_vec() });
            // In a real kernel, this would also wake up blocked tasks.
            Ok(())
        } else {
            Err("Channel not found")
        }
    }

    pub fn kernel_recv(id: ChannelId) -> Option<alloc::vec::Vec<u8>> {
        let mut chans = CHANNELS.lock();
        chans[id as usize].as_mut()?.queue.pop_front().map(|m| m.data)
    }
}

mod timer {
    // Temporary stub for timer.
    pub static mut TICKS: u64 = 0;
}

mod caps {
    // Temporary stub for capabilities.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Capability {
        LogWrite,
        TimeRead,
        NetworkAccess,
        StorageAccess,
    }
}

mod arch {
    // Temporary stub for arch-specific functionality.
    pub mod irq {
        pub fn register_irq_handler(_irq: u8, _chan: u32) {}
        pub fn acknowledge_irq(_irq: u8) {}
    }

    pub mod dma {
        extern crate alloc;

        use alloc::collections::BTreeMap;
        use alloc::vec::Vec;
        use core::sync::atomic::{AtomicU64, Ordering};
        use spin::Mutex;

        static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
        static DMA_BUFFERS: Mutex<BTreeMap<u64, Vec<u8>>> = Mutex::new(BTreeMap::new());

        pub fn alloc_dma_buffer(size: usize) -> Option<u64> {
            let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
            let mut buffers = DMA_BUFFERS.lock();
            // Allocate a Vec as a placeholder for a DMA page.
            let buffer = Vec::with_capacity(size);
            buffers.insert(handle, buffer);
            Some(handle)
        }

        pub fn free_dma_buffer(handle: u64) {
            let mut buffers = DMA_BUFFERS.lock();
            buffers.remove(&handle);
        }

        pub fn get_dma_buffer_ptr(handle: u64) -> Option<*mut u8> {
            let mut buffers = DMA_BUFFERS.lock();
            buffers.get_mut(&handle).map(|buf| buf.as_mut_ptr())
        }

        pub fn get_dma_buffer_capacity(handle: u64) -> Option<usize> {
            let buffers = DMA_BUFFERS.lock();
            buffers.get(&handle).map(|buf| buf.capacity())
        }

        pub fn set_dma_buffer_len(handle: u64, len: usize) -> Result<(), &'static str> {
            let mut buffers = DMA_BUFFERS.lock();
            if let Some(buf) = buffers.get_mut(&handle) {
                if len <= buf.capacity() {
                    // SAFETY: Bound checked against capacity above.
                    unsafe { buf.set_len(len) };
                    Ok(())
                } else {
                    Err("Length exceeds capacity")
                }
            } else {
                Err("DMA handle not found")
            }
        }
    }
}

pub const E_ACC_DENIED: u64 = 0xFFFFFFFFFFFFFFFE;
pub const E_UNKNOWN_SYSCALL: u64 = 0xFFFFFFFFFFFFFFFF;
pub const E_ERROR: u64 = 1;
pub const SUCCESS: u64 = 0;

pub const SYS_LOG: u64 = 0;
pub const SYS_IPC_SEND: u64 = 1;
pub const SYS_IPC_RECV: u64 = 2;
pub const SYS_BLOCK_ON_CHAN: u64 = 3;
pub const SYS_TIME: u64 = 4;
pub const SYS_IRQ_REGISTER: u64 = 5;
pub const SYS_NET_RX_POLL: u64 = 6;
pub const SYS_NET_ALLOC_BUF: u64 = 7;
pub const SYS_NET_FREE_BUF: u64 = 8;
pub const SYS_NET_TX: u64 = 9;
pub const SYS_IRQ_ACK: u64 = 10;
pub const SYS_GET_DMA_BUF_PTR: u64 = 11;
pub const SYS_SET_DMA_BUF_LEN: u64 = 12;
pub const SYS_IPC_RECV_NONBLOCKING: u64 = 13;

#[no_mangle]
pub extern "C" fn syscall_dispatch(n: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let current_task = crate::task::get_current_task();

    match n {
        SYS_LOG => {
            if !current_task.has_cap(crate::caps::Capability::LogWrite) {
                return E_ACC_DENIED;
            }
            let ptr = a1 as *const u8;
            let len = a2 as usize;
            // SAFETY: Caller provides pointer/len pair for syscall ABI.
            let msg = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(s) = core::str::from_utf8(msg) {
                crate::console::print("[V-Node Log] ");
                crate::console::print(s);
                crate::console::print("\n");
                SUCCESS
            } else {
                E_ERROR
            }
        }
        SYS_IPC_SEND => {
            // SAFETY: Caller provides pointer/len pair for syscall ABI.
            let buf = unsafe { core::slice::from_raw_parts(a2 as *const u8, a3 as usize) };
            if crate::ipc::kernel_send(a1 as u32, buf).is_ok() {
                SUCCESS
            } else {
                E_ERROR
            }
        }
        SYS_IPC_RECV | SYS_IPC_RECV_NONBLOCKING => {
            let out_ptr = a2 as *mut u8;
            let out_cap = a3 as usize;
            if let Some(data) = crate::ipc::kernel_recv(a1 as u32) {
                if data.len() <= out_cap {
                    // SAFETY: out_ptr points to writable buffer of at least out_cap.
                    unsafe {
                        core::ptr::copy_nonoverlapping(data.as_ptr(), out_ptr, data.len())
                    };
                    data.len() as u64
                } else {
                    E_ERROR
                }
            } else {
                SUCCESS
            }
        }
        SYS_BLOCK_ON_CHAN => {
            crate::task::block_current_on_channel(a1 as u32);
            SUCCESS
        }
        SYS_TIME => {
            if !current_task.has_cap(crate::caps::Capability::TimeRead) {
                return E_ACC_DENIED;
            }
            // SAFETY: Timer is a global monotonic tick in this stub.
            unsafe { crate::timer::TICKS }
        }
        SYS_IRQ_REGISTER => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            crate::arch::irq::register_irq_handler(a1 as u8, a2 as u32);
            crate::console::print("[Kernel] Registering IRQ ");
            crate::console::print_u64(a1);
            crate::console::print(" for channel ");
            crate::console::print_u64(a2);
            crate::console::print("\n");
            SUCCESS
        }
        SYS_NET_RX_POLL => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            let dma_handle = a2;
            let out_cap = a3 as usize;

            let simulated_packet: [u8; 42] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08,
                0x06, 0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x01, 0xC0, 0xA8, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0,
                0xA8, 0x01, 0x02,
            ];
            let packet_len = simulated_packet.len();

            if packet_len <= out_cap {
                if let Some(buf_ptr) = crate::arch::dma::get_dma_buffer_ptr(dma_handle) {
                    // SAFETY: Destination pointer comes from managed DMA map and has enough capacity.
                    unsafe {
                        core::ptr::copy_nonoverlapping(simulated_packet.as_ptr(), buf_ptr, packet_len)
                    };
                    if crate::arch::dma::set_dma_buffer_len(dma_handle, packet_len).is_ok() {
                        packet_len as u64
                    } else {
                        E_ERROR
                    }
                } else {
                    E_ERROR
                }
            } else {
                E_ERROR
            }
        }
        SYS_NET_ALLOC_BUF => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            if let Some(handle) = crate::arch::dma::alloc_dma_buffer(a1 as usize) {
                handle
            } else {
                E_ERROR
            }
        }
        SYS_NET_FREE_BUF => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            crate::arch::dma::free_dma_buffer(a1);
            SUCCESS
        }
        SYS_NET_TX => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            crate::console::print("[Kernel] Queuing packet for TX, handle: ");
            crate::console::print_hex(a2);
            crate::console::print(", len: ");
            crate::console::print_u64(a3);
            crate::console::print("\n");
            SUCCESS
        }
        SYS_IRQ_ACK => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            crate::arch::irq::acknowledge_irq(a1 as u8);
            SUCCESS
        }
        SYS_GET_DMA_BUF_PTR => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            if let Some(ptr) = crate::arch::dma::get_dma_buffer_ptr(a1) {
                ptr as u64
            } else {
                E_ERROR
            }
        }
        SYS_SET_DMA_BUF_LEN => {
            if !current_task.has_cap(crate::caps::Capability::NetworkAccess) {
                return E_ACC_DENIED;
            }
            if crate::arch::dma::set_dma_buffer_len(a1, a2 as usize).is_ok() {
                SUCCESS
            } else {
                E_ERROR
            }
        }
        _ => E_UNKNOWN_SYSCALL,
    }
}
