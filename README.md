# fart-joke
## A Kernel written in C, GHC Haskell and Rust

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
 - C
   - Clang
 - x86 Assembly
   - NASM
 - GHC Haskell

### [Goals](#Abstract)

 - *Should support AMD86 and work in 64bit mode w/ full paging*
 - *Should perform basic co-operative multitasking*
 - *Should be written in Haskell as much as possible.*
   - C, Assembly or Rust should only be used where required for functionality or performance reasons.

#### [Optional](#Goals)

 - *Could, support userland and syscalls*
 - *Could, provide a "complete" libc where possible.*"
 - *Could, embed a webassembly runtime engine*

## [Progress](#Index)

 - [ ] Build system
     - [ ] Bootstrapping
 - [ ] IO/Memory memes
    - [ ] Compilmentary VGA Driver
    - [ ] Compilmentary Keyboard Driver (UK-GB layout)
 - [ ] Haskell RTS
     - [ ] Multitasking
 - [ ] The Kernel (All of the above.)

## [Related projects](#Index)

 - [Hos](https://github.com/tathougies/hos)
 - [Pluto](https://github.com/SamTebbs33/pluto)
 - [HaLVM](https://github.com/GaloisInc/HaLVM)
