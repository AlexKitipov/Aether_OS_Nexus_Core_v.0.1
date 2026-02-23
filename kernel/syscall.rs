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

fn try_recv_into_buffer(channel_id: u32, out_ptr: *mut u8, out_cap: usize) -> Result<Option<u64>, ()> {
    if let Some(data) = crate::ipc::kernel_recv(channel_id) {
        if data.len() > out_cap {
            return Err(());
        }

        // SAFETY: The caller owns `out_ptr` and guarantees writable memory of size `out_cap`.
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), out_ptr, data.len());
        }

        Ok(Some(data.len() as u64))
    } else {
        Ok(None)
    }
}

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

            // SAFETY: `ptr` and `len` come from userspace ABI and are expected to be valid.
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
            // SAFETY: ABI input buffer from userspace.
            let buf = unsafe { core::slice::from_raw_parts(a2 as *const u8, a3 as usize) };
            if crate::ipc::kernel_send(a1 as u32, buf).is_ok() {
                SUCCESS
            } else {
                E_ERROR
            }
        }
        SYS_IPC_RECV => {
            let chan_id = a1 as u32;
            let out_ptr = a2 as *mut u8;
            let out_cap = a3 as usize;

            match try_recv_into_buffer(chan_id, out_ptr, out_cap) {
                Ok(Some(len)) => len,
                Ok(None) => {
                    crate::task::block_current_on_channel(chan_id);
                    SUCCESS
                }
                Err(()) => E_ERROR,
            }
        }
        SYS_IPC_RECV_NONBLOCKING => {
            let chan_id = a1 as u32;
            let out_ptr = a2 as *mut u8;
            let out_cap = a3 as usize;

            match try_recv_into_buffer(chan_id, out_ptr, out_cap) {
                Ok(Some(len)) => len,
                Ok(None) => SUCCESS,
                Err(()) => E_ERROR,
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

            // SAFETY: Kernel timer tick value is maintained by timer IRQ context.
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

            if packet_len > out_cap {
                return E_ERROR;
            }

            if let Some(buf_ptr) = crate::arch::dma::get_dma_buffer_ptr(dma_handle) {
                // SAFETY: DMA ptr is managed by kernel and packet_len <= out_cap.
                unsafe {
                    core::ptr::copy_nonoverlapping(simulated_packet.as_ptr(), buf_ptr, packet_len);
                }

                if crate::arch::dma::set_dma_buffer_len(dma_handle, packet_len).is_ok() {
                    packet_len as u64
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
