//! Kernel-wide error types used by syscall handlers and IPC primitives.

/// Strongly typed kernel errors surfaced by internal APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// The provided IPC channel id is outside the configured channel range.
    InvalidChannelId(u32),
    /// The output buffer is too small for the received message.
    BufferTooSmall { required: usize, provided: usize },
    /// The current task lacks required capability.
    PermissionDenied,
    /// Memory allocation failed.
    OutOfMemory,
    /// Invalid file descriptor.
    InvalidFd,
}

impl KernelError {
    /// Converts an internal error to userspace syscall return code.
    pub fn to_syscall_code(self) -> u64 {
        match self {
            Self::PermissionDenied => crate::kernel::syscall::E_ACC_DENIED,
            _ => crate::kernel::syscall::E_ERROR,
        }
    }
}
