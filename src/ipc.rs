#![no_std]

extern crate alloc;

//! # Inter-Process Communication (IPC)
//!
//! Provides message passing between V-Node processes using kernel-managed channels.
//! Each channel maintains a queue of messages that can be sent/received by V-Nodes.
//! Messages are copied through kernel space for security isolation.

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;

use crate::kernel::config::IPC_CHANNEL_COUNT;
use crate::kernel::error::{KernelError, Result};

pub mod vnode;

/// IPC send trait used by V-Node channel wrappers.
pub trait IpcSend {
    /// Sends a raw payload into an IPC channel wrapper.
    fn send_raw(&mut self, bytes: &[u8]) -> core::result::Result<(), ()>;
}

/// IPC receive trait used by V-Node channel wrappers.
pub trait IpcRecv {
    /// Receives and deserializes one payload from an IPC channel wrapper.
    fn recv<T: serde::de::DeserializeOwned>(&mut self) -> Option<T>;
}

/// Message queue for a single IPC channel.
pub type MessageQueue = VecDeque<Vec<u8>>;

/// Maximum IPC message payload size (16 KiB).
const MAX_IPC_MESSAGE_SIZE: usize = 16 * 1024;

/// Global IPC channel storage protected by a spinlock.
static CHANNELS: Mutex<Option<Vec<MessageQueue>>> = Mutex::new(None);

fn with_channels<R>(f: impl FnOnce(&mut Vec<MessageQueue>) -> R) -> R {
    let mut guard = CHANNELS.lock();
    let channels = guard.get_or_insert_with(|| {
        let mut channels = Vec::with_capacity(IPC_CHANNEL_COUNT as usize);
        for _ in 0..IPC_CHANNEL_COUNT {
            channels.push(VecDeque::new());
        }
        channels
    });
    f(channels)
}

/// Sends a message through an IPC channel.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn kernel_send(channel_id: u32, data: &[u8]) -> Result<()> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    if data.len() > MAX_IPC_MESSAGE_SIZE {
        return Err(KernelError::InvalidArgument("Message too large"));
    }

    with_channels(|channels| channels[channel_id as usize].push_back(data.to_vec()));
    Ok(())
}

/// Receives a message from an IPC channel (non-blocking).
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn kernel_recv(channel_id: u32) -> Result<Option<Vec<u8>>> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    Ok(with_channels(|channels| channels[channel_id as usize].pop_front()))
}

/// Returns the number of queued messages on a channel.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn channel_message_count(channel_id: u32) -> Result<usize> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    Ok(with_channels(|channels| channels[channel_id as usize].len()))
}

/// Initializes a channel explicitly.
///
/// This is idempotent and primarily useful to validate a channel ID.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn init_channel(channel_id: u32) -> Result<()> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    with_channels(|_| ());
    Ok(())
}

/// Clears all queued messages from a channel.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn clear_channel(channel_id: u32) -> Result<()> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    with_channels(|channels| channels[channel_id as usize].clear());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_recv_basic() {
        clear_channel(1).unwrap();
        let data = b"test message";
        kernel_send(1, data).unwrap();

        let received = kernel_recv(1).unwrap();
        assert_eq!(received, Some(data.to_vec()));
    }

    #[test]
    fn test_invalid_channel() {
        let invalid_channel = IPC_CHANNEL_COUNT;
        let result = kernel_send(invalid_channel, b"data");
        assert_eq!(result, Err(KernelError::InvalidChannelId(invalid_channel)));
    }

    #[test]
    fn test_empty_channel() {
        clear_channel(5).unwrap();
        let received = kernel_recv(5).unwrap();
        assert_eq!(received, None);
    }

    #[test]
    fn test_channel_message_count() {
        let channel = 6;
        clear_channel(channel).unwrap();
        assert_eq!(channel_message_count(channel).unwrap(), 0);
        kernel_send(channel, b"one").unwrap();
        kernel_send(channel, b"two").unwrap();
        assert_eq!(channel_message_count(channel).unwrap(), 2);
        let _ = kernel_recv(channel).unwrap();
        assert_eq!(channel_message_count(channel).unwrap(), 1);
    }

    #[test]
    fn test_large_message_rejected() {
        clear_channel(7).unwrap();
        let too_large = alloc::vec![0_u8; MAX_IPC_MESSAGE_SIZE + 1];
        let result = kernel_send(7, &too_large);
        assert_eq!(result, Err(KernelError::InvalidArgument("Message too large")));
    }

    #[test]
    fn test_clear_channel() {
        clear_channel(8).unwrap();
        kernel_send(8, b"hello").unwrap();
        assert_eq!(channel_message_count(8).unwrap(), 1);
        clear_channel(8).unwrap();
        assert_eq!(channel_message_count(8).unwrap(), 0);
    }
}
