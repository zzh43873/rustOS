#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustOS::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello world{}", "!");
    
    #[cfg(test)]
    test_main();
    
    loop {

    }
}

#[panic_handler]
#[cfg(not(test))]
fn panic(_info : &PanicInfo) -> ! {
    println!("{}", _info);
    loop{}
}


#[panic_handler]
#[cfg(test)]
fn panic(_info : &PanicInfo) -> ! {
    rustOS::test_panic_handler(_info)
}