use spin::Mutex;

use cpuio::{Port, UnsafePort};

//Cmd sent to begin PIC initialization
const CMD_INIT: u8 = 0x11;

//Cmd sent to acknowledge an interrupt
const CMD_END_OF_INTERRUPT: u8 = 0x20;

//The mode in which we want to run PIC
const MODE_8086: u8 = 0x01;

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
        self.pics[0].data.write(CMD_INIT);
        wait();
        self.pics[1].data.write(CMD_INIT);
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

        self.pics[0].data.write(saved_mask1);
        self.pics[1].data.write(saved_mask2);
    }

    pub fn handles_interrupt(&self, int_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(int_id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, int_id: u8) {
        if self.pics[1].handles_interrupt(int_id) {
            self.pics[1].end_of_interrupt();
        }

        self.pics[0].end_of_interrupt();
    }
}

static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });
    
#[repr(packed)]
struct Idtr {
    limit: u16,
    offset: u64,
}

#[repr(packed)]
#[derive(Copy, Clone)]
struct IdtDescriptor {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_and_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero2: u32,
}

static IDT_ENTRIES: Mutex<[IdtDescriptor; 256]> =
    Mutex::new([IdtDescriptor{
        offset_low: 0, 
        selector: 0, 
        zero: 0, 
        type_and_attr: 0, 
        offset_mid: 0, 
        offset_high: 0, 
        zero2: 0
    }; 256]);
    
static IDT_PTR: Mutex<Idtr> = Mutex::new(Idtr{limit: 0, offset: 0});

fn idt_set_gate(num: u8, base: u64, sel: u16, flags: u8)
{
    let e: &mut IdtDescriptor = &mut IDT_ENTRIES.lock()[num as usize];
    
    (*e).offset_low = (base & 0xFFFF) as u16;
    (*e).offset_mid = ((base >> 16) & 0xFFFF) as u16;
    (*e).offset_high = ((base >> 32) & 0xFFFFFFFF) as u32;
    
    (*e).selector = sel;
    (*e).type_and_attr = flags;
}

extern "C" {
    fn idt_flush(addr: u64);
}

#[no_mangle]
pub extern fn isr_handler()
{
    println!("Isr handler!");
}

#[no_mangle]
pub extern fn irq_handler()
{
    println!("IRQ handler!");
}
    
pub fn init()
{
    IDT_PTR.lock().limit = (::core::mem::size_of::<IdtDescriptor>() * 256 - 1) as u16;
    IDT_PTR.lock().offset = &IDT_ENTRIES.lock() as *const _ as u64;
    
    idt_set_gate(0, isr0 as u64, 0x08, 0x8E);
    idt_set_gate(1, isr1 as u64, 0x08, 0x8E);
    idt_set_gate(2, isr2 as u64, 0x08, 0x8E);
    idt_set_gate(3, isr3 as u64, 0x08, 0x8E);
    idt_set_gate(4, isr4 as u64, 0x08, 0x8E);
    idt_set_gate(5, isr5 as u64, 0x08, 0x8E);
    idt_set_gate(6, isr6 as u64, 0x08, 0x8E);
    idt_set_gate(7, isr7 as u64, 0x08, 0x8E);
    idt_set_gate(8, isr8 as u64, 0x08, 0x8E);
    idt_set_gate(9, isr9 as u64, 0x08, 0x8E);
    idt_set_gate(10, isr10 as u64, 0x08, 0x8E);
    idt_set_gate(11, isr11 as u64, 0x08, 0x8E);
    idt_set_gate(12, isr12 as u64, 0x08, 0x8E);
    idt_set_gate(13, isr13 as u64, 0x08, 0x8E);
    idt_set_gate(14, isr14 as u64, 0x08, 0x8E);
    idt_set_gate(15, isr15 as u64, 0x08, 0x8E);
    idt_set_gate(16, isr16 as u64, 0x08, 0x8E);
    idt_set_gate(17, isr17 as u64, 0x08, 0x8E);
    idt_set_gate(18, isr18 as u64, 0x08, 0x8E);
    idt_set_gate(19, isr19 as u64, 0x08, 0x8E);
    idt_set_gate(20, isr20 as u64, 0x08, 0x8E);
    idt_set_gate(21, isr21 as u64, 0x08, 0x8E);
    idt_set_gate(22, isr22 as u64, 0x08, 0x8E);
    idt_set_gate(23, isr23 as u64, 0x08, 0x8E);
    idt_set_gate(24, isr24 as u64, 0x08, 0x8E);
    idt_set_gate(25, isr25 as u64, 0x08, 0x8E);
    idt_set_gate(26, isr26 as u64, 0x08, 0x8E);
    idt_set_gate(27, isr27 as u64, 0x08, 0x8E);
    idt_set_gate(28, isr28 as u64, 0x08, 0x8E);
    idt_set_gate(29, isr29 as u64, 0x08, 0x8E);
    idt_set_gate(30, isr30 as u64, 0x08, 0x8E);
    idt_set_gate(31, isr31 as u64, 0x08, 0x8E);
    
    idt_set_gate(32, irq0 as u64, 0x08, 0x8E);
    idt_set_gate(33, irq1 as u64, 0x08, 0x8E);
    idt_set_gate(34, irq2 as u64, 0x08, 0x8E);
    idt_set_gate(35, irq3 as u64, 0x08, 0x8E);
    idt_set_gate(36, irq4 as u64, 0x08, 0x8E);
    idt_set_gate(37, irq5 as u64, 0x08, 0x8E);
    idt_set_gate(38, irq6 as u64, 0x08, 0x8E);
    idt_set_gate(39, irq7 as u64, 0x08, 0x8E);
    idt_set_gate(40, irq8 as u64, 0x08, 0x8E);
    idt_set_gate(41, irq9 as u64, 0x08, 0x8E);
    idt_set_gate(42, irq10 as u64, 0x08, 0x8E);
    idt_set_gate(43, irq11 as u64, 0x08, 0x8E);
    idt_set_gate(44, irq12 as u64, 0x08, 0x8E);
    idt_set_gate(45, irq13 as u64, 0x08, 0x8E);
    idt_set_gate(46, irq14 as u64, 0x08, 0x8E);
    idt_set_gate(47, irq15 as u64, 0x08, 0x8E);
    
    unsafe {
        idt_flush(&IDT_PTR.lock() as *const _ as u64);
        PICS.lock().initialize()
    }
}

extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();

    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();
}