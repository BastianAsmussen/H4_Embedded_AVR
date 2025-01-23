#![no_std]
#![no_main]

use core::panic::PanicInfo;

const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        core::ptr::write_volatile(DDRB, core::ptr::read_volatile(DDRB) | 0b0000_1111);
        core::ptr::write_volatile(PORTB, 0b0000_1010);
    }
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
