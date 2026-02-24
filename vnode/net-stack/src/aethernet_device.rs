//! AetherNet device shim used by the net-stack V-Node.
//!
//! This file is intentionally small in this repository snapshot because the
//! full device implementation depends on the OS runtime and DMA bridge types.

pub struct AetherNetDevice {
    _driver_id: u32,
    _bridge_channel_id: u32,
}

impl AetherNetDevice {
    pub fn new(driver_id: u32, bridge_channel_id: u32) -> Self {
        Self {
            _driver_id: driver_id,
            _bridge_channel_id: bridge_channel_id,
        }
    }

    pub fn enqueue_rx_packet(&mut self, _dma_handle: u64, _len: usize) {
        // Actual implementation is platform-specific and lives in runtime repo.
    }
}
