#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustOS::println;
use bootloader::{bootinfo, entry_point, BootInfo};

entry_point!(kernel_main);
pub fn kernel_main(boot_info : &'static BootInfo) -> ! {
    println!("hello world{}", "!");
    
    rustOS::init();

    use rustOS::memory::translate_addr;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }
    
    #[cfg(test)]
    test_main();
    
    println!("it did not crash");
    rustOS::hlt_loop();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(_info : &PanicInfo) -> ! {
    println!("{}", _info);
    rustOS::hlt_loop();
}


#[panic_handler]
#[cfg(test)]
fn panic(_info : &PanicInfo) -> ! {
    rustOS::test_panic_handler(_info)
}