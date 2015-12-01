use spin::Mutex;
use core::ptr;

use cpuio::{Port, UnsafePort};

//Cmd sent to begin PIC initialization
const CMD_INIT: u8 = 0x11;

//Cmd sent to acknowledge an interrupt
const CMD_END_OF_INTERRUPT: u8 = 0x20;

//The mode in which we want to run PIC
const MODE_8086: u8 = 0x01;

extern "C" {
    static interrupt_handlers: [*const u8; 256];
}

macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

struct Pic {
    offset: u8,
    command: UnsafePort<u8>,
    data: UnsafePort<u8>,
}

impl Pic {
    fn handles_interrupt(&self, int_id: u8) -> bool {
        self.offset <= int_id && int_id < self.offset + 8
    }

    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics:  [
                Pic {
                    offset: offset1,
                    command: UnsafePort::new(0x20),
                    data: UnsafePort::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: UnsafePort::new(0xA0),
                    data: UnsafePort::new(0xA1),
                },
            ]
        }
    }

    pub unsafe fn initialize(&mut self) {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || {wait_port.write(0) };

        let saved_mask1 = self.pics[0].data.read();
        let saved_mask2 = self.pics[1].data.read();

        //starts the initialization sequence (in cascade mode)
        self.pics[0].command.write(CMD_INIT);
        wait();
        self.pics[1].command.write(CMD_INIT);
        wait();

        //Master PIC vector offset
        self.pics[0].data.write(self.pics[0].offset);
        wait();
        //Slave PIC vector offset
        self.pics[1].data.write(self.pics[1].offset);
        wait();

        //tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
        self.pics[0].data.write(4);
        wait();
        //tell Slave PIC its cascade identity (0000 0010)
        self.pics[1].data.write(2);
        wait();

        self.pics[0].data.write(MODE_8086);
        wait();
        self.pics[1].data.write(MODE_8086);
        wait();
        
        println!("0b{:b}", saved_mask1);
        println!("0b{:b}", saved_mask2);
        
        self.pics[0].data.write(saved_mask1 | 0b00000001);//disable timer?
        self.pics[1].data.write(saved_mask2);
    }

    pub fn handles_interrupt(&self, int_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(int_id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, int_id: u8) {
        if self.pics[1].handles_interrupt(int_id) {
            println!("EOI");
            self.pics[1].end_of_interrupt();
        }

        self.pics[0].end_of_interrupt();
    }
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
    
    (*e).selector = 0x08;
    (*e).type_and_attr = 0b1000_1110;
    
    (*e).zero = 0;
    (*e).zero2 = 0;
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
            idt_set_gate(0x08, index, handler);
        }
    }
    
    for (idx,en) in IDT_ENTRIES.lock().iter().enumerate() {
        println!("{}", en.selector);
        if idx == 8 {
            break;
        }
    }
        
    unsafe {
        idt_flush(&IDT_PTR.lock());
        
        //int!(80);
        
        asm!("sti");
    }
}