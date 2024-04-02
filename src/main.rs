#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustOS::{allocator, memory, println};
use bootloader::{bootinfo, entry_point, BootInfo};
use x86_64::structures::paging::page;

extern crate alloc;

entry_point!(kernel_main);
pub fn kernel_main(boot_info : &'static BootInfo) -> ! {
    println!("hello world{}", "!");
    
    rustOS::init();

    use rustOS::memory::BootInfoFrameAllocator;
    use x86_64::{VirtAddr, structures::paging::Page};
    use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe { 
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
     };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    #[cfg(test)]
    test_main();
    
    println!("it did not crash");
    rustOS::hlt_loop();
}

// #[panic_handler]
// #[cfg(not(test))]
// fn panic(_info : &PanicInfo) -> ! {
//     println!("{}", _info);
//     rustOS::hlt_loop()
// }


#[panic_handler]
#[cfg(test)]
fn panic(_info : &PanicInfo) -> ! {
    rustOS::test_panic_handler(_info)
}