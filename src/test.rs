use crate::println;

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "multi-line test";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
    println!("multi line test...\t [ok]")
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...\t");
    assert_eq!(1, 1);
    println!(" [ok]")
}

#[test_case]
fn test_println_many() {
    for _ in 0..3 {
        println!("println...\t [ok]");
    }
}

/* 
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

#[test_case]
fn test_double_fault() {
    // invoke a double fault
    unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    };
}
*/