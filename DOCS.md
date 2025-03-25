[Rust Embedded bogen]: https://docs.rust-embedded.org/book/

# Dokumentation

Dette projekt dokumenterer hvad jeg har brugt det meste af min tid i embedded
på. I form af ting der bimler og bamler har jeg ikke det helt store da jeg har
fokuseret på det low level af det low level. Ting som build tools, logik og
teorien bag ved er spænende og derfor har jeg fokuseret på det. En anden grund
er at jeg har lavet lignende med breadboards tidligere. Samt protokol design og
implementation, så derfor lavede jeg kun lige det basale MQTT implementation da
jeg, f.eks. har lavet et message queue system fra bunden før.

## Indholdsfortegnelse

- [Starten](#starten)
- [Oversættelse](#oversættelse)
- [Flashing](#flashing)
- [Afslutning](#afslutning)
- [Debugging](#debugging)

## Starten

Jeg begyndte med at genopfriske mine færdigheder i C og Assembly. Da det minder
om at cykle, var det ikke en stor udfordring - det mest interessante var at
gennemgå AVR-instruktionssættet. Efter at have læst AVR-specifikationen fik jeg
idéen om at implementere kommunikation mellem devices, da jeg har lavet
lignende projekter tidligere. Derudover fandt jeg det spændende at omskrive
projektet til Rust, især fordi jeg allerede har erfaring med at udvikle et
operativsystem til `x86_64`-arkitekturen.

## Oversættelse

Jeg startede med at udvikle et simpelt embedded "Hello, World!"-eksempel, hvor
en LED blinker. Oversættelsen var forholdsvis ligetil. Jeg endte med at
implementere et HAL (Hardware Abstraction Layer), hvilket gjorde det nemt og
intuitivt at skrive logikken for at tænde og slukke LED'en.

## Flashing

Jeg har ikke meget erfaring med at flashe custom firmware på AtMega AVR boards,
hvilket blev en udfordring, da MPLAB ikke understøtter Rust. For at løse dette
lavede jeg et custom LLVM build target, tilpasset AtMega328PB, baseret på et
eksisterende build target. Byggeprocessen blev herefter så enkel som at køre:

```sh
cargo build --release
```

Denne kommando (`cargo`) er Rust's build tool
(på samme måde som `ninja` eller `CMake` bruges i C-verdenen), og producerede
firmware til AtMega'en. Næste skridt var at flashe firmware til boardet. Dette
opnås med følgende kommando:

```sh
# avrdude -p m328pb -c xplainedmini -P usb:03eb:2145 -b 115200 -U flash:w:target/avr-unknown-gnu-atmega328/release/h4_embedded_avr.elf:e -v
```

Her specificerer vi med `avrdude`:

- Board: `-p m328pb`
- Programmer: `-c xplainedmini` (angiver hvilken programmeringsmetode, f.eks. bootloader eller Debug Wire)
- USB-port: `-P usb:03eb:2145`
- Baudrate: `-b 115200`
- Firmware: `-U flash:w:target/avr-unknown-gnu-atmega328/release/h4_embedded_avr.elf:e`
- Verbose mode: `-v` (Giver detaljeret information om processen)

## Afslutning

Det over var et simpelt projekt, det jeg fik noget ud af var ikke koden jeg
skrev nødvendigvis siden processen af at lære hvordan man kunne flashe firmware
var meget lærerigt i sig selv. Jeg tror ofte at nye udviklere kan tage for
givet hvor meget der rent faktisk sker når de klikker på den grønne "Run" knap
i deres IDE, og jeg tror at kunne kompilere sine programmer manuelt er en ret
vigtig ting i.ft. at danne sig en dybere forståelse for deres felt. At få en
LED til at blinke når man klikker på en interface er ret sekundært i min optik
og er ikke noget jeg ville kalde specielt kompieceret.

Her til sidst begyndte jeg at kigge ind til at oversætte mit operativsystem fra
`x86_64` til `AVR`. En god del af de egenskaber mit OS havde (filsystem, tastatur, hukommelses-allokator, osv..)
kan godt pilles ud. En af de større problemer kom i form af at oversætte IDT'en
(Interrupt Descriptor Table) da de samme interrupts ikke nødvendigvis ligger
på de samme adresser i hukommelsen. Der er også problemet af at oversætte GDT'en (Global Descriptor Table)
da det ser ud til at det er en `x86`-arkitektur-specifik ting.

## Debugging

At debugge er ret simpelt, og jeg tænker at MPLAB og Microsoft Visual Studio
gør meget af det samme omme bagved. [Rust Embedded bogen] har noget super
dokumentation på debugging og alt muligt andet godt om embedded i Rust og
generelt. De snakker selvfølgelig også om opsætning i VSCode.
