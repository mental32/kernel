#!/usr/bin/python

import os
import sys

assert sys.version_info >= (3, 8)

import argparse
from subprocess import check_call
from pathlib import Path
from argparse import BooleanOptionalAction

parser = argparse.ArgumentParser()

parser.add_argument("--release", action=BooleanOptionalAction)

parser.add_argument("--build", action=BooleanOptionalAction)
parser.add_argument("--iso", action=BooleanOptionalAction)

parser.add_argument("--qemu", action=BooleanOptionalAction)
parser.add_argument("--qemu-nographic", action=BooleanOptionalAction)


args = parser.parse_args()


build_path = Path(".").joinpath("build")

common_path = Path(__file__).parent.joinpath("common")

arch = "x86_64"
mode = "debug" if not args.release else "release"

kernel_blob = build_path.joinpath(f"kernel-{arch}-{mode}.bin")
grub_cfg = common_path.joinpath("grub.cfg")
linker_script = common_path.joinpath("x86_64.linker.ld")
iso_path = build_path.joinpath(f"kernel-{arch}-{mode}.iso")

libkernel_path = Path(f"target/target-x86_64/{mode}/libkernel.a").absolute()

def sh(st: str):
    return check_call(st, shell=True)

if args.build or args.iso or args.qemu:
    # Clean the build path
    sh("rm -rf build")

    build_path.mkdir()

    # Build the kernel.
    release = "--release" if args.release else ""
    sh(f"cargo b {release} -vv --target common/target-x86_64.json")

    assert libkernel_path.exists()

    # Assemble all {arch} asm sources and collect all object paths.
    assembly_files = Path(__file__).parent.glob(f"./arch/src/{arch}/**/*.asm")
    object_files = []

    for asm in assembly_files:
        obj = build_path.joinpath(f"{asm.stem}.o").absolute()
        sh(f"nasm -felf64 {asm} -o {obj}")
        assert obj.exists()
        object_files.append(obj)

    object_files.append(libkernel_path)
    objs = " ".join(map(str, object_files))

    # Link it together with linker script.
    sh(f"ld -n --gc-sections -T {linker_script} -o {kernel_blob} {objs}")

if args.iso or args.qemu:
    # Create iso using grub-mkrescue.
    sh("mkdir -p build")
    sh("mkdir -p ./build/isofiles/boot/grub")
    sh(f"cp {kernel_blob} build/isofiles/boot/kernel.bin")
    sh("strip build/isofiles/boot/kernel.bin")
    sh(f"cp {grub_cfg} build/isofiles/boot/grub")
    sh(f"grub-mkrescue -o {iso_path} build/isofiles")
    sh("rm -r build/isofiles")

if args.qemu:
    # Run QEMU with our ISO.
    qemu_args = os.environ.get("QEMU_ARGS", None)

    if qemu_args is None:
        memory = os.environ.get("QEMU_MEM", "512M")
        smp = os.environ.get("QEMU_SMP", "4")

        qemu_args = " ".join([
            f"-m {memory}",
            f"-smp {smp}",
            "-nographic" if args.qemu_nographic else "-serial stdio",
            "-nic user,model=rtl8139",
            "-machine q35",
            "-no-reboot",
            "-no-shutdown",
        ])

    sh(f"qemu-system-x86_64 -drive format=raw,file={iso_path} {qemu_args}")
