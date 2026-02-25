#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use crate::ipc::{IpcRecv, IpcSend};
use crate::syscall::{
    syscall2, syscall3, SYS_BLOCK_ON_CHAN, SYS_IPC_RECV, SYS_IPC_RECV_NONBLOCKING, SYS_IPC_SEND,
};

pub struct VNodeChannel {
    pub id: u32,
    buffer: [u8; 4096],
}

impl VNodeChannel {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            buffer: [0; 4096],
        }
    }

    pub fn recv_blocking(&mut self) -> Result<Vec<u8>, ()> {
        loop {
            let len = unsafe {
                syscall3(
                    SYS_IPC_RECV,
                    self.id as u64,
                    self.buffer.as_mut_ptr() as u64,
                    self.buffer.len() as u64,
                )
            };
            if len > 0 {
                return Ok(self.buffer[..len as usize].to_vec());
            } else if len == 0 {
                unsafe {
                    syscall2(SYS_BLOCK_ON_CHAN, self.id as u64, 0);
                }
            } else {
                return Err(());
            }
        }
    }

    pub fn recv_non_blocking(&mut self) -> Result<Option<Vec<u8>>, ()> {
        let len = unsafe {
            syscall3(
                SYS_IPC_RECV_NONBLOCKING,
                self.id as u64,
                self.buffer.as_mut_ptr() as u64,
                self.buffer.len() as u64,
            )
        };
        if len > 0 {
            Ok(Some(self.buffer[..len as usize].to_vec()))
        } else if len == 0 {
            Ok(None)
        } else {
            Err(())
        }
    }


    pub fn send_and_recv<Req: serde::Serialize, Res: serde::de::DeserializeOwned>(
        &mut self,
        request: &Req,
    ) -> Result<Res, ()> {
        let encoded = postcard::to_allocvec(request).map_err(|_| ())?;
        self.send_raw(&encoded)?;

        loop {
            match self.recv_non_blocking()? {
                Some(data) => return postcard::from_bytes(&data).map_err(|_| ()),
                None => {
                    unsafe {
                        syscall2(SYS_BLOCK_ON_CHAN, self.id as u64, 0);
                    }
                }
            }
        }
    }
}

impl IpcSend for VNodeChannel {
    fn send_raw(&mut self, bytes: &[u8]) -> Result<(), ()> {
        unsafe {
            let res = syscall3(
                SYS_IPC_SEND,
                self.id as u64,
                bytes.as_ptr() as u64,
                bytes.len() as u64,
            );
            if res == crate::syscall::SUCCESS {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

impl IpcRecv for VNodeChannel {
    fn recv<T: serde::de::DeserializeOwned>(&mut self) -> Option<T> {
        match self.recv_non_blocking() {
            Ok(Some(data)) => postcard::from_bytes(&data).ok(),
            _ => None,
        }
    }
}
