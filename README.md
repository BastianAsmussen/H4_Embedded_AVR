# H4 Embedded AVR

This is a school project for learning embedded programming.

## Compilation

```sh
export DEVICE="<YOUR_USB_DEVICE>"

cargo build --release
sudo avrdude -p m328pb -c xplainedmini -P $DEVICE -b 115200 -U flash:w:target/avr-unknown-gnu-atmega328/release/h4_embedded_avr.elf:e -v
```
