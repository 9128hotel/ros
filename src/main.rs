#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![reexport_test_harness_main = "test_main"] // create a test_main function for testing
#![feature(abi_x86_interrupt)] // error fixer for interrupts.rs

///////////////////////////////////////////////////////////////////////////////
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#[cfg(test)] // custom tester system
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} test(s)", tests.len()); // output to serial
    println!("Running {} test(s)", tests.len()); // and to the screen
    for test in tests {
        test();
    }
    println!("\nTesting complete");
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) { // closes the QEMU testing enviroment
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

///////////////////////////////////////////////////////////////////////////////

mod serial; // serial comunication
mod vga; // VGA writing to screen
pub mod interrupts; // breakpoint and double fault handler
pub mod gdt; // stack overflow handler

///////////////////////////////////////////////////////////////////////////////

pub fn hlt_loop() -> ! { // allows the CPU to enter a sleep state when not in use
    loop {
        x86_64::instructions::hlt();
    }
}

///////////////////////////////////////////////////////////////////////////////

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! { // custom entry point
    interrupts::init_idt(); // load IDT
    gdt::init(); // load GDT

    #[cfg(test)]
    test_main();

    println!("It did not crash!!");
    println!("Hello world{}", "!");
    hlt_loop();
}
// panic system
use core::panic::PanicInfo;

// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}
