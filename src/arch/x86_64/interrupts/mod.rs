pub mod pic;

use spin::Mutex;
use core::ptr;
use self::pic::ChainedPics;

macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

extern "C" {
    static gdt64_code_offset: u16;
    static interrupt_handlers: [*const u8; 256];
}

static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });
    
#[repr(C, packed)]
struct Idtr {
    limit: u16,
    offset: u64,
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

static IDT_ENTRIES: Mutex<[IdtDescriptor; 256]> =
    Mutex::new([IdtDescriptor{
        offset_low: 0, 
        selector: 0, 
        zero: 0, 
        type_and_attr: 0, 
        offset_high: 0, 
        zero2: 0
    }; 256]);
    
static IDT_PTR: Mutex<Idtr> = Mutex::new(Idtr{limit: 0, offset: 0});

fn idt_set_gate(gdt_code_selector: u16, num: usize, handler: *const u8)
{
    let e: &mut IdtDescriptor = &mut IDT_ENTRIES.lock()[num];
    
    (*e).offset_low = ((handler as u64) & 0xFFFF) as u16;
    (*e).offset_high = (handler as u64) >> 16;
    
    (*e).selector = gdt_code_selector;
    (*e).type_and_attr = 0b1000_1110;
}

#[repr(C, packed)]
pub struct InterruptContext {
    rsi: u64,
    rdi: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    int_id: u64,
    error_code: u64,
}

#[no_mangle]
pub extern fn isr_handler(ctx: &InterruptContext)
{
    println!("Isr handler! {} 0x{:b}", ctx.int_id, ctx.error_code);
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(ctx.int_id as u8);
    }
}

unsafe fn idt_flush(idt: &Idtr)
{
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
    
pub fn init()
{    
    unsafe {
        PICS.lock().initialize();
    }
    
    IDT_PTR.lock().limit = (16 * 256) as u16;
    IDT_PTR.lock().offset = &IDT_ENTRIES.lock()[0] as *const _ as u64;
    
    println!("isr addr: 0x{:x}", IDT_PTR.lock().offset);
            
    for (index, &handler) in interrupt_handlers.iter().enumerate() {
        if handler != ptr::null() {
            idt_set_gate(gdt64_code_offset, index, handler);
        }
    }
        
    unsafe {
        idt_flush(&IDT_PTR.lock());
        
        int!(80);
        
        asm!("sti");
    }
}