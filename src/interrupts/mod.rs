use spin::Mutex;
use core::mem::size_of;

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

extern "C" {
    fn idt_flush(addr: u64);
    fn raise_exc();
    fn setup_interrupts();
}

#[no_mangle]
pub extern fn isr_handler()
{
    println!("Isr handler!");
    
    loop{}
}

#[no_mangle]
pub extern fn irq_handler()
{
    println!("IRQ handler!");
    
    loop{}
}
    
pub fn init()
{
    
    unsafe {
        //setup_interrupts();
        PICS.lock().initialize();
        raise_exc();
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