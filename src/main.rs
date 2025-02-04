#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::{
    atmega328pb::{Peripherals, TWI0},
    interrupt::{self, Mutex},
};
use core::cell::Cell;

const SSD1306_ADDR: u8 = 0x3C;
const SSD1306_COMMAND: u8 = 0x00;
const SSD1306_DATA_STREAM: u8 = 0x40;
const SSD1306_DISPLAY_OFF: u8 = 0xAE;
const SSD1306_DISPLAY_ON: u8 = 0xAF;
const SSD1306_SET_CONTRAST: u8 = 0x81;
const SSD1306_SET_COLUMN_ADDR: u8 = 0x21;
const SSD1306_SET_PAGE_ADDR: u8 = 0x22;
const SSD1306_SET_START_LINE: u8 = 0x40;

static TIMER_INT_COUNT: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static EXTERNAL_INT_COUNT: Mutex<Cell<u16>> = Mutex::new(Cell::new(0));

/// Sets a block of memory to a specific value.
///
/// # Arguments
///
/// * `dest` - Pointer to the memory block to fill.
/// * `c` - Value to set (only the lowest 8 bits are used.)
/// * `n` - Number of bytes to set.
///
/// # Returns
///
/// * The pointer to the memory block.
///
/// # Safety
///
/// This function is unsafe because it:
/// * Takes a raw pointer which must be valid for writes of `n` bytes.
/// * The memory range [dest, dest + n) must be contained within a single allocated object.
/// * Must not overflow an isize.
/// * The memory must not be accessed by other threads while this operation is in progress.
#[unsafe(no_mangle)]
#[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            *dest.add(i) = c as u8;
        }

        i += 1;
    }

    dest
}

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

struct I2C {
    twi: TWI0,
}

impl I2C {
    fn new(twi: TWI0) -> Self {
        // Configure for 100kHz with 16MHz clock.
        twi.twbr.write(|w| w.bits(72));
        twi.twsr.write(|w| w.twps().bits(0));
        unsafe {
            twi.twar.write(|w| w.bits(0));
        }

        twi.twcr.write(|w| w.twen().set_bit().twea().set_bit());

        Self { twi }
    }

    fn start(&self) -> bool {
        self.twi
            .twcr
            .write(|w| w.twen().set_bit().twsta().set_bit().twint().set_bit());
        while self.twi.twcr.read().twint().bit_is_clear() {}

        (self.twi.twsr.read().bits() & 0xF8) == 0x08
    }

    fn write_byte(&self, byte: u8) -> bool {
        self.twi.twdr.write(|w| w.bits(byte));
        self.twi
            .twcr
            .write(|w| w.twen().set_bit().twint().set_bit());
        while self.twi.twcr.read().twint().bit_is_clear() {}

        (self.twi.twsr.read().bits() & 0xF8) == 0x28
    }

    fn stop(&self) {
        self.twi
            .twcr
            .write(|w| w.twen().set_bit().twsto().set_bit().twint().set_bit());
    }
}

struct SSD1306<'a> {
    i2c: &'a mut I2C,
}

impl<'a> SSD1306<'a> {
    fn new(i2c: &'a mut I2C) -> Self {
        Self { i2c }
    }

    fn init(&self) -> bool {
        let init_sequence = [
            SSD1306_DISPLAY_OFF,
            0xA8,
            0x3F, // Set multiplex ratio.
            0xD3,
            0x00, // Set display offset.
            SSD1306_SET_START_LINE,
            0xA1, // Set segment re-map.
            0xC8, // Set COM output scan direction.
            0xDA,
            0x12, // Set COM pins hardware configuration.
            SSD1306_SET_CONTRAST,
            0xCF,
            0xA4, // Display all on resume.
            0xA6, // Set normal display.
            0xD5,
            0x80, // Set oscillator frequency.
            0x8D,
            0x14, // Enable charge pump regulator.
            SSD1306_DISPLAY_ON,
        ];

        if !self.i2c.start() {
            return false;
        }

        if !self.i2c.write_byte(SSD1306_ADDR << 1) {
            return false;
        }

        for &cmd in &init_sequence {
            if !self.i2c.write_byte(SSD1306_COMMAND) {
                return false;
            }

            if !self.i2c.write_byte(cmd) {
                return false;
            }
        }

        self.i2c.stop();

        true
    }

    fn write_number(&self, num: u32) -> bool {
        let mut buffer = [0u8; 10];
        let mut idx = 0;

        if num == 0 {
            buffer[idx] = b'0';
            idx += 1;
        } else {
            let mut n = num;
            while n > 0 && idx < buffer.len() {
                buffer[idx] = (n % 10) as u8 + b'0';
                n /= 10;
                idx += 1;
            }

            buffer[..idx].reverse();
        }

        self.write_bytes(&buffer[..idx])
    }

    fn write_bytes(&self, bytes: &[u8]) -> bool {
        if !self.i2c.start() {
            return false;
        }

        if !self.i2c.write_byte(SSD1306_ADDR << 1) {
            return false;
        }

        if !self.i2c.write_byte(SSD1306_DATA_STREAM) {
            return false;
        }

        for &byte in bytes {
            if !self.i2c.write_byte(byte) {
                return false;
            }
        }

        self.i2c.stop();

        true
    }

    fn set_position(&self, x: u8, y: u8) -> bool {
        if !self.i2c.start() {
            return false;
        }

        if !self.i2c.write_byte(SSD1306_ADDR << 1) {
            return false;
        }

        // Set column address.
        if !self.i2c.write_byte(SSD1306_COMMAND)
            || !self.i2c.write_byte(SSD1306_SET_COLUMN_ADDR)
            || !self.i2c.write_byte(x)
            || !self.i2c.write_byte(127)
        {
            return false;
        }

        // Set page address.
        if !self.i2c.write_byte(SSD1306_COMMAND)
            || !self.i2c.write_byte(SSD1306_SET_PAGE_ADDR)
            || !self.i2c.write_byte(y)
            || !self.i2c.write_byte(7)
        {
            return false;
        }

        self.i2c.stop();

        true
    }
}

struct Adc {
    adc: avr_device::atmega328pb::ADC,
}

impl Adc {
    fn new(adc: avr_device::atmega328pb::ADC) -> Self {
        adc.admux.write(|w| unsafe {
            w.refs()
                .avcc() // Use AVCC as reference.
                .mux()
                .bits(0) // Select ADC0.
        });

        // Enable ADC and set prescaler to 64.
        adc.adcsra.write(|w| unsafe {
            w.aden()
                .set_bit() // Enable ADC.
                .adps()
                .bits(0b110) // Prescaler of 64.
        });

        Self { adc }
    }

    fn read_adc(&self) -> u16 {
        self.adc.adcsra.modify(|_, w| w.adsc().set_bit());
        while self.adc.adcsra.read().adsc().bit_is_set() {}

        self.adc.adc.read().bits()
    }
}

struct Pwm {
    tc4: avr_device::atmega328pb::TC4,
}

impl Pwm {
    fn new(tc4: avr_device::atmega328pb::TC4, portd: &avr_device::atmega328pb::PORTD) -> Self {
        // Set PD6 (OC4A) as output.
        portd.ddrd.modify(|_, w| w.pd6().set_bit());

        // Configure Timer 4 for Fast PWM mode.
        tc4.tccr4a.write(|w| {
            w.com4a()
                .bits(0b10) // Non-inverting mode.
                .wgm4()
                .bits(0b01) // Fast PWM mode.
        });

        // Set prescaler to 8.
        tc4.tccr4b.write(
            |w| w.cs4().bits(0b010), // Prescaler = 8
        );

        // Initialize PWM duty cycle to 0.
        tc4.ocr4a.write(|w| w.bits(0));

        Self { tc4 }
    }

    fn set_duty_cycle(&self, value: u8) {
        // Convert u8 to u16 since OCR4A is 16-bit.
        self.tc4.ocr4a.write(|w| w.bits(u16::from(value)));
    }
}

#[avr_device::interrupt(atmega328pb)]
fn INT0() {
    interrupt::free(|cs| {
        let count = EXTERNAL_INT_COUNT.borrow(cs).get();

        EXTERNAL_INT_COUNT.borrow(cs).set(count + 1);
    });
}

#[avr_device::interrupt(atmega328pb)]
fn TIMER0_COMPA() {
    interrupt::free(|cs| {
        let count = TIMER_INT_COUNT.borrow(cs).get();

        TIMER_INT_COUNT.borrow(cs).set(count + 1);
    });
}

#[avr_device::entry]
fn main() -> ! {
    let dp = Peripherals::take().expect("Failed to get peripherals!");

    let mut i2c = I2C::new(dp.TWI0);
    let display = SSD1306::new(&mut i2c);
    let adc = Adc::new(dp.ADC);
    let pwm = Pwm::new(dp.TC4, &dp.PORTD);

    // Initialize display.
    display.init();

    // Configure Timer0 for 1 second intervals.
    dp.TC0.tccr0a.write(|w| w.wgm0().ctc());
    dp.TC0.tccr0b.write(|w| w.cs0().prescale_1024());
    dp.TC0.ocr0a.write(|w| w.bits(155));
    dp.TC0.timsk0.write(|w| w.ocie0a().set_bit());

    // Configure External Interrupt (INT0).
    dp.PORTD.ddrd.modify(|_, w| w.pd2().clear_bit());
    dp.PORTD.portd.modify(|_, w| w.pd2().set_bit());
    dp.EXINT.eicra.modify(|_, w| w.isc0().bits(0b10)); // Falling edge.
    dp.EXINT.eimsk.modify(|_, w| w.int0().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }

    let mut last_timer_count = 0;
    let mut last_ext_count = 0;
    let mut last_adc_value = 0;

    loop {
        let adc_value = adc.read_adc();

        let adc_diff = if adc_value > last_adc_value {
            adc_value - last_adc_value
        } else {
            last_adc_value - adc_value
        };

        // Only update if value changed significantly (to avoid display flicker).
        if adc_diff > 2 {
            // Map ADC value (0 - 1023) to PWM value (0 - 255).
            #[expect(clippy::cast_possible_truncation)]
            let pwm_value = (u32::from(adc_value) * 255 / 1_023) as u8;
            pwm.set_duty_cycle(pwm_value);

            // Display ADC value.
            display.set_position(0, 2);
            display.write_bytes(b"ADC: ");
            display.write_number(u32::from(adc_value));

            // Convert to voltage and display.
            let total_millivolts = (u32::from(adc_value) * 5_000) / 1_023;

            let whole_volts = total_millivolts / 1_000;
            let fractional_millivolts = total_millivolts % 1_000;

            display.set_position(0, 3);
            display.write_bytes(b"V: ");
            display.write_number(whole_volts);
            display.write_bytes(b".");

            // Ensure fractional part shows 3 digits.
            if fractional_millivolts < 100 {
                display.write_bytes(b"0");
            }

            if fractional_millivolts < 10 {
                display.write_bytes(b"0");
            }

            display.write_number(fractional_millivolts);

            last_adc_value = adc_value;
        }

        let (timer_count, ext_count) = interrupt::free(|cs| {
            (
                TIMER_INT_COUNT.borrow(cs).get(),
                EXTERNAL_INT_COUNT.borrow(cs).get(),
            )
        });

        if last_timer_count != timer_count || last_ext_count != ext_count {
            // Update timer count display.
            display.set_position(0, 0);
            display.write_bytes(b"Timer: ");
            display.write_number(timer_count);

            // Update external interrupt count display.
            display.set_position(0, 1);
            display.write_bytes(b"Ext Int: ");
            display.write_number(u32::from(ext_count));

            last_timer_count = timer_count;
            last_ext_count = ext_count;
        }
    }
}
