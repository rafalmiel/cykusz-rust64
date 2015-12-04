/// WIP.  Some bits were sanity-checked against
/// https://github.com/ryanra/RustOS/blob/master/src/arch/x86/idt.rs
///
/// See section 6.10 of
/// http://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-manual-325462.pdf
///
/// See http://jvns.ca/blog/2013/12/04/day-37-how-a-keyboard-works/ for
/// some general advice on setting up interrupts and an entertaining saga
/// of frustration.

mod idt;

use spin::Mutex;
use super::pic::*;
use self::idt::*;

pub use arch::dtables::*;

//=========================================================================
//  Interface to interrupt_handlers.asm

/// Maximum possible number of interrupts; we can shrink this later if we
/// want.


/// Our global IDT.
static IDT: Mutex<idt::Idt> = Mutex::new(idt::Idt {
    table: [idt::missing_handler(); idt::IDT_ENTRY_COUNT]
});

#[allow(dead_code)]
extern {

    /// A primitive interrupt-reporting function.
    fn report_interrupt();
}

/// Various data available on our stack when handling an interrupt.
///
/// Only `pub` because `rust_interrupt_handler` is.
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
    _pad_1: u32,
    error_code: u32,
    _pad_2: u32,
}

/// Enable Interrupts.
pub unsafe fn enable()  {
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable()  {
    asm!("cli");
}

macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}


//=========================================================================
//  Handling interrupts

/// Interface to our PIC (programmable interrupt controller) chips.  We
/// want to map hardware interrupts to 0x20 (for PIC1) or 0x28 (for PIC2).
static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

/// Print our information about a CPU exception, and loop.
fn cpu_exception_handler(_ctx: &InterruptContext) {

    // Print general information provided by x86::irq.
//     println!("{}, error 0x{:x}",
//              EXCEPTIONS[ctx.int_id as usize],
//              ctx.error_code);

    println!("Caught cpu exception");

    // Provide detailed information about our error code if we know how to
    // parse it.
//     match ctx.int_id {
//         14 => {
//             let err = x86::irq::PageFaultError::from_bits(ctx.error_code);
//             println!("{:?}", err);
//         }
//         _ => {}
//     }

    loop {}
}

/// Called from our assembly-language interrupt handlers to dispatch an
/// interrupt.
#[no_mangle]
pub unsafe extern "C" fn rust_interrupt_handler(ctx: &InterruptContext) {
    match ctx.int_id {
        0x00...0x0F => cpu_exception_handler(ctx),
        0x20 => { println!("TIMER!"); }
        0x21 => {
            /*if let Some(input) = keyboard::read_char() {
                if input == '\r' {
                    println!("");
                } else {
                    print!("{}", input);
                }
            }*/
        }
        0x80 => println!("Not actually Linux, sorry."),
        _ => {
            println!("UNKNOWN INTERRUPT #{}", ctx.int_id);
            loop {}
        }
    }

    PICS.lock().notify_end_of_interrupt(ctx.int_id as u8);
}

//=========================================================================
//  Initialization

/// Use the `int` instruction to manually trigger an interrupt without
/// actually using `sti` to enable interrupts.  This is highly recommended by
/// http://jvns.ca/blog/2013/12/04/day-37-how-a-keyboard-works/
#[allow(dead_code)]
pub unsafe fn test_interrupt() {
    println!("Triggering interrupt.");
    int!(0x80);
    println!("Interrupt returned!");
}

/// Platform-independent initialization.
pub unsafe fn initialize() {
    PICS.lock().initialize();
    IDT.lock().initialize();

    // Enable this to trigger a sample interrupt.
    test_interrupt();

    // Turn on real interrupts.
    enable();
}
 
