#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::kernel::config::IPC_CHANNEL_COUNT;
use crate::kernel::error::KernelError;

pub mod vnode;


/// IPC send trait used by V-Node channel wrappers.
pub trait IpcSend {
    fn send_raw(&mut self, bytes: &[u8]) -> Result<(), ()>;
}

/// IPC receive trait used by V-Node channel wrappers.
pub trait IpcRecv {
    fn recv<T: serde::de::DeserializeOwned>(&mut self) -> Option<T>;
}

static mut CHANNELS: Option<Vec<VecDeque<Vec<u8>>>> = None;

fn channels_mut() -> &'static mut Vec<VecDeque<Vec<u8>>> {
    // SAFETY: This kernel prototype runs single-threaded in tests/simulation.
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

/// Sends a payload to a kernel IPC channel queue.
pub fn kernel_send(channel_id: u32, data: &[u8]) -> Result<(), KernelError> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return Err(KernelError::InvalidChannelId(channel_id));
    }

    channels_mut()[channel_id as usize].push_back(data.to_vec());
    Ok(())
}

/// Receives one payload from a kernel IPC channel queue.
pub fn kernel_recv(channel_id: u32) -> Option<Vec<u8>> {
    if channel_id >= IPC_CHANNEL_COUNT {
        return None;
    }

    channels_mut()[channel_id as usize].pop_front()
}
