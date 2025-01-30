#define F_CPU 1000000UL

#include <avr/interrupt.h>
#include <avr/io.h>

int main(void) {
  DDRB |= (1 << DDB1);
  PORTB = 0xE;

  TCCR0B = (1 << CS02) | (1 << CS00);
  OCR0B = 0x3D08;
  TIFR0 = 1 << TOV0;
  TIMSK0 = 1 << TOIE0;

  sei();

  while (1) {
  }
}

ISR(TIMER0_OVF_vect) {
  PORTB ^= (1 << PORTB1);
  TCNT1 = 0;
}
