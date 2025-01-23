#![no_std]
#![no_main]

mod hal;

use core::{hint::black_box, panic::PanicInfo};

const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

fn sleep() {
    for i in 0..100_000 {
        black_box(i);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        core::ptr::write_volatile(DDRB, core::ptr::read_volatile(DDRB) | 0b0000_1111);
        core::ptr::write_volatile(PORTB, 0b0000_1010);
    }

    loop {
        sleep();
    }
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
