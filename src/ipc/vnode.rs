#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use crate::ipc::{IpcRecv, IpcSend};
use crate::syscall::{
    syscall3, E_ERROR, SUCCESS, SYS_IPC_RECV, SYS_IPC_RECV_NONBLOCKING, SYS_IPC_SEND,
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

            match len {
                l if l > SUCCESS => {
                    return Ok(self.buffer[..l as usize].to_vec());
                }
                SUCCESS => {
                    // Kernel has blocked and re-scheduled us; retry once resumed.
                }
                E_ERROR => {
                    return Err(());
                }
                _ => {
                    return Err(());
                }
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

        match len {
            l if l > SUCCESS => Ok(Some(self.buffer[..l as usize].to_vec())),
            SUCCESS => Ok(None),
            E_ERROR => Err(()),
            _ => Err(()),
        }
    }

    pub fn send_and_recv<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &mut self,
        request: &Req,
    ) -> Result<Resp, ()> {
        let serialized_request = postcard::to_allocvec(request).map_err(|_| ())?;
        self.send_raw(&serialized_request)?;

        let response = self.recv_blocking()?;
        postcard::from_bytes(&response).map_err(|_| ())
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
            if res == SUCCESS {
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
