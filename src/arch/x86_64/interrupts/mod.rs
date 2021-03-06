mod pic;
mod idt;

use spin::Mutex;

static PICS: Mutex<pic::ChainedPics> = Mutex::new(unsafe { pic::ChainedPics::new(0x20, 0x28) });

static IDT: Mutex<idt::Idt> = Mutex::new(idt::Idt::new());

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
    int_id: u32,
    _pad1: u32,
    error_code: u32,
    _pad2: u32,
}

#[no_mangle]
pub extern "C" fn isr_handler(ctx: &InterruptContext) {
    match ctx.int_id {
        80 => println!("INTERRUPTS WORKING {} 0x{:x}", ctx.int_id, ctx.error_code),
        33 => println!("Keyboard interrupt detected"),
        14 => {
            println!("PAGE FAULT");
            loop{};
        },
        _ => {
            println!("OTHER FAULT {}", ctx.int_id);
            loop{};
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(ctx.int_id as u8);
    }
}

pub fn init() {
    unsafe {
        PICS.lock().init();
        IDT.lock().init();

        idt::test();

        idt::enable();
    }
}
