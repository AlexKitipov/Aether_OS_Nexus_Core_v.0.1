#![allow(dead_code)] // Allow dead code for now as not all functions might be used immediately

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;

use crate::{kprintln, task};

/// A unique identifier for an IPC channel.
pub type ChannelId = u32;

/// A message sent over an IPC channel.
pub struct Message {
    pub sender_task_id: u64, // The ID of the task that sent this message
    pub data: Vec<u8>,
}

/// Represents a kernel-managed IPC channel or mailbox.
pub struct Mailbox {
    queue: VecDeque<Message>,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            queue: VecDeque::new(),
        }
    }
}

/// Global array of IPC channels. Max 32 channels for simplicity.
/// In a real system, this would be a dynamic structure like a BTreeMap.
const MAX_CHANNELS: usize = 32;
static MAILBOXES: Mutex<[Option<Mailbox>; MAX_CHANNELS]> =
    Mutex::new([const { None }; MAX_CHANNELS]);

/// Sends a message over the specified IPC channel (mailbox).
///
/// Returns `Ok(())` on success, `Err` with an error message on failure.
pub fn send(channel_id: ChannelId, sender_task_id: u64, data: &[u8]) -> Result<(), &'static str> {
    if channel_id as usize >= MAX_CHANNELS {
        kprintln!(
            "[kernel] mailbox: Send failed, channel ID {} out of bounds.",
            channel_id
        );
        return Err("Channel ID out of bounds");
    }

    let mut mailboxes = MAILBOXES.lock();
    let mailbox_entry = &mut mailboxes[channel_id as usize];

    // Ensure the mailbox exists, create if not (dynamic mailbox creation)
    if mailbox_entry.is_none() {
        *mailbox_entry = Some(Mailbox::new());
        kprintln!(
            "[kernel] mailbox: Dynamically created mailbox {}.",
            channel_id
        );
    }

    if let Some(mailbox) = mailbox_entry.as_mut() {
        mailbox.queue.push_back(Message {
            sender_task_id,
            data: data.to_vec(),
        });
        kprintln!(
            "[kernel] mailbox: Message sent to mailbox {} by task {}.",
            channel_id,
            sender_task_id
        );
        // If a task is blocked on this mailbox, unblock it.
        task::unblock_task_on_channel(channel_id);
        Ok(())
    } else {
        // This case should ideally not be reached if mailbox is created above
        kprintln!(
            "[kernel] mailbox: Send failed, mailbox {} not found after creation attempt.",
            channel_id
        );
        Err("Mailbox not found (internal error)")
    }
}

/// Receives a message from the specified IPC channel (mailbox).
///
/// Returns `Some(Message)` if a message is available, `None` otherwise.
pub fn recv(channel_id: ChannelId) -> Option<Message> {
    if channel_id as usize >= MAX_CHANNELS {
        kprintln!(
            "[kernel] mailbox: Recv failed, channel ID {} out of bounds.",
            channel_id
        );
        return None;
    }

    let mut mailboxes = MAILBOXES.lock();
    if let Some(mailbox) = mailboxes[channel_id as usize].as_mut() {
        let msg = mailbox.queue.pop_front();
        if msg.is_some() {
            kprintln!(
                "[kernel] mailbox: Message received from mailbox {}.",
                channel_id
            );
        }
        msg
    } else {
        kprintln!(
            "[kernel] mailbox: Recv failed, mailbox {} not found.",
            channel_id
        );
        None
    }
}

/// Checks if a mailbox has messages without removing them.
pub fn peek(channel_id: ChannelId) -> bool {
    if channel_id as usize >= MAX_CHANNELS {
        return false;
    }

    let mailboxes = MAILBOXES.lock();
    if let Some(mailbox) = mailboxes[channel_id as usize].as_ref() {
        !mailbox.queue.is_empty()
    } else {
        false
    }
}
