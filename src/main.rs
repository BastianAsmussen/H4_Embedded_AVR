#![no_std]
#![no_main]

use ruduino::Pin;
use ruduino::cores::current::port;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    port::B5::set_output();
    port::B5::set_high();

    loop {
        // port::B5::set_high();
        // ruduino::delay::delay_ms(1000);
        // port::B5::set_low();
        // ruduino::delay::delay_ms(1000);
    }
}
