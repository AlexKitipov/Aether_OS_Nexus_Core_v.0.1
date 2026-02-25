#![no_std]

pub use crate::kernel::syscall::{
    E_ACC_DENIED, E_ERROR, E_UNKNOWN_SYSCALL, SUCCESS, SYS_BLOCK_ON_CHAN, SYS_GET_DMA_BUF_PTR,
    SYS_IPC_RECV, SYS_IPC_RECV_NONBLOCKING, SYS_IPC_SEND, SYS_IRQ_ACK, SYS_IRQ_REGISTER,
    SYS_LOG, SYS_NET_ALLOC_BUF, SYS_NET_FREE_BUF, SYS_NET_RX_POLL, SYS_NET_TX,
    SYS_SET_DMA_BUF_LEN, SYS_TIME,
};

/// Invokes a 2-argument syscall through the in-kernel dispatcher.
///
/// # Safety
/// Caller must provide ABI-safe arguments.
pub unsafe fn syscall2(n: u64, a1: u64, a2: u64) -> u64 {
    crate::kernel::syscall::syscall_dispatch(n, a1, a2, 0)
}

/// Invokes a 3-argument syscall through the in-kernel dispatcher.
///
/// # Safety
/// Caller must provide ABI-safe arguments.
pub unsafe fn syscall3(n: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    crate::kernel::syscall::syscall_dispatch(n, a1, a2, a3)
}
