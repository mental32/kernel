# fart-joke
## A Kernel written in Rust

## Index

 - [Abstract](#Abstract)
   - [Tech stack](#Tech-stack)
   - [Goals](#Goals)
 - [Progress](#Progress)

## [Abstract](#Index)

> *So about the name?* It's a place holder, if this project gets weirdly
> popular I'll change it, but otherwise I reserve the bragging rights of
> saying I made a fart joke that ran my code.

Lets try writing a small kernel in C, Rust and Haskell.

### [Tech stack](#Abstract)

 - Make
 - Rust
   - cargo-xargo
 - x86 Assembly
   - NASM

## [Progress](#Index)

 - [x] Build system
     - [x] Bootstrapping

 - [ ] Drivers
    - [x] VGA Text mode driver
    - [x] Keyboard driver (UK-GB layout)
    - [x] Pic8259
    - [x] Pit825x
    - [ ] VBE driver
    - [ ] APIC
    - [ ] ACPI
    - [ ] RTC
    - [ ] WASM

## [Related projects](#Index)

 - [Hos](https://github.com/tathougies/hos)
 - [Pluto](https://github.com/SamTebbs33/pluto)
 - [HaLVM](https://github.com/GaloisInc/HaLVM)
