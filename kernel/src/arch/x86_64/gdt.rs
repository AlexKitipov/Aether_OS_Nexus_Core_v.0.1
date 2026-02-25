#![allow(dead_code)] // Allow dead code for now as not all functions might be used immediately

use crate::kprintln;
use x86_64::instructions::segmentation::{CS, Segment};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};

/// Define our Global Descriptor Table.
/// The GDT contains entries for kernel code and data segments.
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

/// Define our segment selectors.
/// These are used to load the segment registers after the GDT is loaded.
static mut KERNEL_CODE_SELECTOR: Option<SegmentSelector> = None;
static mut KERNEL_DATA_SELECTOR: Option<SegmentSelector> = None;

/// Initializes the GDT and loads it into the CPU.
/// Also reloads segment registers with the new selectors.
pub fn init() {
    // SAFETY: We are writing to static mut variables during early boot single-threaded init.
    unsafe {
        kprintln!("[kernel] gdt: Initializing GDT...");

        // Add kernel code and data segments to the GDT.
        KERNEL_CODE_SELECTOR = Some(GDT.add_entry(Descriptor::kernel_code_segment()));
        KERNEL_DATA_SELECTOR = Some(GDT.add_entry(Descriptor::kernel_data_segment()));

        // Load the GDT into the CPU.
        GDT.load();
        kprintln!("[kernel] gdt: GDT loaded.");

        // Reload segment registers.
        let code_selector = KERNEL_CODE_SELECTOR.expect("kernel code selector must be initialized");
        let data_selector = KERNEL_DATA_SELECTOR.expect("kernel data selector must be initialized");

        // Reloading CS requires a far jump under the hood and is handled by this API.
        CS::set_reg(code_selector);
        kprintln!("[kernel] gdt: CS reloaded with selector {:#?}.", code_selector);

        // Reload other segment registers (DS, ES, FS, GS, SS).
        x86_64::instructions::segmentation::DS::set_reg(data_selector);
        x86_64::instructions::segmentation::ES::set_reg(data_selector);
        x86_64::instructions::segmentation::FS::set_reg(data_selector);
        x86_64::instructions::segmentation::GS::set_reg(data_selector);
        x86_64::instructions::segmentation::SS::set_reg(data_selector);

        kprintln!("[kernel] gdt: Segment registers reloaded.");
    }
}
