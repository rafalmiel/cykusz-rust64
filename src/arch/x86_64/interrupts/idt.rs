use core::ptr;
use core::mem::size_of;

extern "C" {
    static gdt64_code_offset: u16;
    static interrupt_handlers: [*const u8; 256];
}

macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtDescriptor {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_and_attr: u8,
    offset_high: u64,
    zero2: u16,
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    offset: u64,
}

impl Idtr {
    fn setup_table(&mut self, table: &[IdtDescriptor; 256]) {
        self.limit = (size_of::<IdtDescriptor>() * 256) as u16;
        self.offset = table as *const _ as u64;
    }
}

pub struct Idt {
    table: [IdtDescriptor; 256],
    pointer: Idtr,
}

impl Idt {
    pub const fn new() -> Idt {
        Idt {
            table: [IdtDescriptor {
                offset_low: 0,
                selector: 0,
                zero: 0,
                type_and_attr: 0,
                offset_high: 0,
                zero2: 0,
            }; 256],
            pointer: Idtr {
                limit: 0,
                offset: 0,
            },
        }
    }

    pub fn init(&mut self) {
        self.pointer.setup_table(&self.table);

        self.setup_gates();

        unsafe {
            flush(&self.pointer);
        }
    }

    fn setup_gates(&mut self) {
        for (index, &handler) in interrupt_handlers.iter().enumerate() {
            if handler != ptr::null() {
                self.set_gate(gdt64_code_offset, 0b1000_1110, index, handler);
            }
        }
    }

    fn set_gate(&mut self, gdt_code_selector: u16, flags: u8, num: usize, handler: *const u8) {
        let e: &mut IdtDescriptor = &mut self.table[num];

        e.offset_low = ((handler as u64) & 0xFFFF) as u16;
        e.offset_high = (handler as u64) >> 16;

        e.selector = gdt_code_selector;
        e.type_and_attr = flags;
    }
}

unsafe fn flush(idt: &Idtr) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

pub unsafe fn test() {
    int!(80);
}

pub unsafe fn enable() {
    asm!("sti");
}

pub unsafe fn disable() {
    asm!("cli");
}
