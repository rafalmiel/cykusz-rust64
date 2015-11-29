use core::ptr;
use core::mem::size_of;
use arch::interrupts::DescriptorTablePointer;

pub const IDT_ENTRY_COUNT: usize = 256;

#[allow(dead_code)]
extern {
    /// The offset of the main code segment in our GDT.  Exported by our
    /// assembly code.
    static gdt64_code_offset: u16;

    /// Interrupt handlers which call back to rust_interrupt_handler.
    static interrupt_handlers: [*const u8; IDT_ENTRY_COUNT];
}

pub struct Idt {
    pub table: [IdtEntry; IDT_ENTRY_COUNT],
}

impl Idt {
    /// Initialize interrupt handling.
    pub unsafe fn initialize(&mut self) {
        self.add_handlers();
        self.load();
    }

    /// Fill in our IDT with our handlers.
    fn add_handlers(&mut self) {
        for (index, &handler) in interrupt_handlers.iter().enumerate() {
            if handler != ptr::null() {
                self.table[index] = IdtEntry::new(gdt64_code_offset, handler);
            }
        }
    }

    /// Load this table as our interrupt table.
    unsafe fn load(&self) {
        let pointer = DescriptorTablePointer {
            base: &self.table[0] as *const IdtEntry as u64,
            limit: (size_of::<IdtEntry>() * IDT_ENTRY_COUNT) as u16,
        };
        lidt(&pointer);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR.
    pub base_lo: u16,
    /// Segment selector.
    pub sel: u16,
    /// This must always be zero.
    pub res0: u8,
    /// Flags.
    pub flags: u8,
    /// The upper 48 bits of ISR (the last 16 bits must be zero).
    pub base_hi: u64,
    /// Must be zero.
    pub res1: u16
}

//-------------------------------------------------------------------------
//  Being merged upstream
//
//  This code will go away when https://github.com/gz/rust-x86/pull/4
//  is merged.

/// Create a IdtEntry marked as "absent".  Not tested with real
/// interrupts yet.  This contains only simple values, so we can call
/// it at compile time to initialize data structures.
pub const fn missing_handler() -> IdtEntry {
    IdtEntry {
        base_lo: 0,
        sel: 0,
        res0: 0,
        flags: 0,
        base_hi: 0,
        res1: 0,
    }
}

trait IdtEntryExt {
    fn new(gdt_code_selector: u16, handler: *const u8) -> IdtEntry;
}

impl IdtEntryExt for IdtEntry {

    /// Create a new IdtEntry pointing at `handler`.
    fn new(gdt_code_selector: u16, handler: *const u8) -> IdtEntry {
        IdtEntry {
            base_lo: ((handler as u64) & 0xFFFF) as u16,
            sel: gdt_code_selector,
            res0: 0,
            flags: 0b100_01110,
            base_hi: (handler as u64) >> 16,
            res1: 0,
        }
    }
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
