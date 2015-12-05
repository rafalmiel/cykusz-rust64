use spin::Mutex;
use core::ptr;

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
pub struct IdtDescriptor {
    pub offset_low: u16,
    pub selector: u16,
    pub zero: u8,
    pub type_and_attr: u8,
    pub offset_high: u64,
    pub zero2: u16,
}
    
#[repr(C, packed)]
pub struct Idtr {
    pub limit: u16,
    pub offset: u64,
}

pub struct Idt {
    table: [IdtDescriptor; 256],
    pointer: Idtr,
}

impl Idt {
    pub const fn new() -> Idt {
        Idt {
            table: [IdtDescriptor{
            offset_low: 0, 
            selector: 0, 
            zero: 0, 
            type_and_attr: 0, 
            offset_high: 0, 
            zero2: 0
        }; 256],
            pointer: Idtr{limit: 0, offset: 0},
        }
    }

    pub fn initialize(&mut self) {
        self.pointer.limit = (16 * 256) as u16;
        self.pointer.offset = &self.table[0] as *const _ as u64;
    
        for (index, &handler) in interrupt_handlers.iter().enumerate() {
            if handler != ptr::null() {
                self.set_gate(gdt64_code_offset, 0b1000_1110, index, handler);
            }
        }
        
        unsafe {
            idt_flush(&self.pointer);
        }
    }

    fn set_gate(&mut self, gdt_code_selector: u16, flags: u8, num: usize, handler: *const u8)
    {
        let e: &mut IdtDescriptor = &mut self.table[num];
    
        (*e).offset_low = ((handler as u64) & 0xFFFF) as u16;
        (*e).offset_high = (handler as u64) >> 16;
        
        (*e).selector = gdt_code_selector;
        (*e).type_and_attr = flags;
    }
}

pub unsafe fn idt_flush(idt: &Idtr)
{
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

pub unsafe fn test()
{
    asm!("int $0" :: "N"(80));
}

pub unsafe fn enable()
{
    asm!("sti");
}

#[allow(dead_code)]
pub unsafe fn disable()
{
    asm!("cli");
}
