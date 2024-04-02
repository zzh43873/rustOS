#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(heap_allocator_test_main);

fn heap_allocator_test_main(boot_info: &'static BootInfo) -> ! {
    use rustOS::allocator;
    use rustOS::memory::{self, BootInfoFrameAllocator};
    use x86_64::{VirtAddr, structures::paging::Page};

    rustOS::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe { 
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
     };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {}
}


use rustOS::{serial_print, serial_println};
use alloc::boxed::Box;

#[test_case]
fn simple_allocation() {
    serial_print!("simple allocation ... ");
    let heap_value = Box::new(23);
    assert_eq!(*heap_value, 23);
    serial_println!("[ok]");
}

use alloc::vec::Vec;
#[test_case]
fn large_vec() {
    serial_print!("large vec ... ");
    let n = 1000;
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }

    assert_eq!(v.iter().sum::<u64>(), n * (n - 1) / 2);
    serial_println!("[ok]");
}

#[test_case]
fn many_boxes() {
    serial_print!("many boxes ... ");
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    serial_println!("[ok]");
}