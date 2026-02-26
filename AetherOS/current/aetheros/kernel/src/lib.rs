// kernel/src/lib.rs

#![no_std]
#![feature(abi_x86_interrupt)] // For x86_64 interrupt handling
#![feature(const_fn_trait_bound)] // For heap init

extern crate alloc;

use bootloader_api::info::MemoryRegions;
use x86_64::VirtAddr;

#[macro_use]
pub mod console; // Our new console module
pub mod timer;   // Our new timer module
pub mod caps;    // Our new capabilities module
pub mod task;    // Our new task management module
pub mod ipc;     // Our new IPC module
pub mod syscall; // Syscall dispatcher

// Architecture-specific modules
pub mod arch;

pub mod drivers; // New: Drivers module

pub mod memory;  // New: Memory management module
pub mod heap;    // Heap allocator

// Other kernel components (stubs for now, will be fleshed out later)
pub mod aetherfs;
pub mod elf;       // New: ELF module
pub mod vnode_loader;

// Constants for heap size and start (these would be dynamically determined in a real system)
pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// The main initialization function for the AetherOS kernel.
pub fn init(memory_regions: &'static MemoryRegions) {
    // Initialize architecture-specific components first
    arch::init();
    drivers::serial::init(); // Initialize serial driver first for early logging
    console::init(); // Initialize console (now depends on serial driver)
    memory::init(memory_regions); // Initialize memory management with bootloader info

    // Initialize kernel heap
    // SAFETY: The caller (bootloader) must ensure that HEAP_START and HEAP_SIZE
    // describe a valid, unused region of memory that is mapped correctly.
    // For this stub, we assume this is handled conceptually.
    unsafe { heap::init(VirtAddr::new(HEAP_START), HEAP_SIZE); }

    timer::init(); // Initialize timer
    task::init(); // Initialize task management
    ipc::init();  // Initialize IPC module
    elf::init(); // Initialize ELF loader

    kprintln!("[kernel] AetherOS kernel initialized.");
}

// The dummy console init function is moved to console.rs and handles actual serial init.
// We remove the redundant `impl console::Uart` block from here.
