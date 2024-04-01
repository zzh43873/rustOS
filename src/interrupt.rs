use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use crate::{println, print};
use crate::gdt;
use spin;
use pic8259::ChainedPics;

pub const PIC_1_OFFSET : u8 = 32;
pub const PIC_2_OFFSET : u8 = PIC_1_OFFSET + 8;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum InterruptIndx {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndx {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS : spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe {ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndx::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndx::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

use x86_64::structures::idt::PageFaultErrorCode;
extern "x86-interrupt" fn page_fault_interrupt_handler(
    stack_frame: InterruptStackFrame, _error_code : PageFaultErrorCode)
{
    use crate::hlt_loop;
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern  "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code : u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern  "x86-interrupt" fn timer_interrupt_handler(
    stack_frame: InterruptStackFrame)
{
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndx::Timer.as_u8());
    }
}

extern  "x86-interrupt" fn keyboard_interrupt_handler(
    stack_frame: InterruptStackFrame)
{
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndx::Keyboard.as_u8());
    }
}

#[cfg(test)]
use crate::{serial_print, serial_println};
#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception... ");
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}