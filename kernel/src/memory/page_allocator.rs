use crate::kprintln;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::Size4KiB;

/// Basic page allocator bootstrap module.
///
/// This is currently a placeholder that validates allocator wiring
/// during early kernel initialization.
pub struct PageAllocator;

impl PageAllocator {
    /// Initialize the page allocator subsystem.
    pub fn init(frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
        let _ = frame_allocator;
        kprintln!("[kernel] page_allocator: initialized placeholder");
    }
}
