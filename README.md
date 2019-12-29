# fart-joke
## A Kernel written in GHC Haskell and Zig

## Index

 - [Abstract](#Abstract)
   - [Tech stack](#Tech-stack)
   - [Goals](#Goals)
 - [Progress](#Progress)

## [Abstract](#Index)

> *So about the name?* It's a place holder, if this project gets weirdly
> popular I'll change it, but otherwise I reserve the bragging rights of
> saying I made a fart joke that ran my code.

Lets try writing a small kernel in Zig and Haskell.

### [Tech stack](#Index)

 - Zig
 - GHC Haskell

### Goals

 - *Should support AMD86 and work in 64bit mode w/ full paging*
 - *Should perform basic co-operative multitasking*
 - *Should be written in Haskell as much as possible.*
   - Assembly or Zig should only be used where required for functionality or performance reasons.

#### Optional

 - *Could, support userland and syscalls*
 - *Could, provide a "complete" libc where possible.*"
 - *Could, embed a webassembly runtime engine*

## [Progress](#Index)

 - [ ] Haskell RTS
 - [ ] The Kernel
