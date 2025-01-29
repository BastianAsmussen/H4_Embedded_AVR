#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::cell::Cell;

use avr_device::interrupt::{self, Mutex};

static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Firmware has panicked so no ISRs should continue running.
    avr_device::interrupt::disable();

    let dp = unsafe { avr_device::atmega328pb::Peripherals::steal() };
    loop {
        avr_device::asm::delay_cycles(1_000_000);
        dp.PORTB.portb.write(|w| w.pb3().set_bit());

        avr_device::asm::delay_cycles(1_000_000);
        dp.PORTB.portb.write(|w| w.pb3().clear_bit());
    }
}

#[avr_device::interrupt(atmega328pb)]
fn TIMER0_OVF() {
    // This interrupt should raise every (1024*255)/16MHz s ≈ 0.01s
    // We then count 61 times to approximate 1s.

    static mut OVF_COUNTER: u16 = 0;
    const ROLLOVER: u16 = 61;

    *OVF_COUNTER = OVF_COUNTER.wrapping_add(1);
    if *OVF_COUNTER > ROLLOVER {
        *OVF_COUNTER = 0;

        interrupt::free(|cs| {
            LED_STATE.borrow(cs).set(!LED_STATE.borrow(cs).get());
        });
    }
}

#[avr_device::interrupt(atmega328pb)]
fn USART0_RX() {}

#[avr_device::entry]
fn main() -> ! {
    let dp = avr_device::atmega328pb::Peripherals::take().unwrap();

    // Divide by 1024 -> 16MHz/1024 = 15.6kHz.
    dp.TC0.tccr0b.write(|w| w.cs0().prescale_1024());
    // Enable overflow interrupts.
    dp.TC0.timsk0.write(|w| w.toie0().set_bit());

    dp.PORTB.ddrb.modify(|_, w| w.pb1().set_bit());
    dp.PORTB.ddrb.modify(|_, w| w.pb3().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }

    let mut counter: u8 = 0;
    let mut previous_state = true;
    loop {
        let mut led_state = true;
        interrupt::free(|cs| {
            led_state = LED_STATE.borrow(cs).get();
        });

        if previous_state != led_state {
            counter += 1;
        }

        assert!(counter <= 9);

        previous_state = led_state;

        dp.PORTB.portb.modify(|_, w| w.pb1().bit(led_state));
    }
}
