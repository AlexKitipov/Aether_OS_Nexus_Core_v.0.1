#![no_std]

extern crate alloc;

//! # Inter-Process Communication (IPC)
//!
//! Provides message passing between V-Node processes using kernel-managed channels.
//! Each channel maintains a queue of messages that can be sent/received by V-Nodes.
//! Messages are copied through kernel space for security isolation.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

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

/// Global IPC channel storage.
static mut CHANNELS: Option<Vec<MessageQueue>> = None;

fn channels_mut() -> &'static mut Vec<MessageQueue> {
    // SAFETY: This kernel prototype currently runs single-threaded in tests/simulation.
    unsafe {
        if CHANNELS.is_none() {
            let mut channels = Vec::with_capacity(IPC_CHANNEL_COUNT as usize);
            for _ in 0..IPC_CHANNEL_COUNT {
                channels.push(VecDeque::new());
            }
            CHANNELS = Some(channels);
        }

        CHANNELS.as_mut().expect("CHANNELS must be initialized")
    }
}

/// Sends a message through an IPC channel.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn kernel_send(channel_id: u32, data: &[u8]) -> Result<()> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    channels_mut()[channel_id as usize].push_back(data.to_vec());
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

    Ok(channels_mut()[channel_id as usize].pop_front())
}

/// Returns the number of queued messages on a channel.
///
/// # Errors
/// Returns [`KernelError::InvalidChannelId`] if `channel_id >= IPC_CHANNEL_COUNT`.
pub fn channel_message_count(channel_id: u32) -> Result<usize> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    Ok(channels_mut()[channel_id as usize].len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_recv_basic() {
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
        let received = kernel_recv(5).unwrap();
        assert_eq!(received, None);
    }

    #[test]
    fn test_channel_message_count() {
        let channel = 6;
        assert_eq!(channel_message_count(channel).unwrap(), 0);
        kernel_send(channel, b"one").unwrap();
        kernel_send(channel, b"two").unwrap();
        assert_eq!(channel_message_count(channel).unwrap(), 2);
        let _ = kernel_recv(channel).unwrap();
        assert_eq!(channel_message_count(channel).unwrap(), 1);
    }
}
