# fart-joke

## A hobby kernel written in Rust.

The root workspace is divided into multiple crates:

* kernel (architecture independent code)
* arch (architecture specific code)
* sched (scheduling i.e. futures executor)
* mem (memory management utilities)

## Building/Running

A script is provided `/x.py` that is used to build the kernel, produce an ISO
and, optionally, ivoke QEMU with the kernel ISO.

Appropriately named options are available:

* `python x.py --build`
* `python x.py --iso`
* `python x.py --qemu`

Also a `--release` flag is available that can be added with any of the above (note that I dont test release builds so stuff probably breaks there.)
