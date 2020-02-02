export common := $(abspath ./common)

GRUB_MKRESCUE = grub-mkrescue

arch   ?= x86_64
export target ?= target-$(arch)

iso    := ./build/kernel-$(arch).iso

grub_cfg := $(common)/grub.cfg

export kernel_blob := $(abspath ./build/kernel-$(arch).bin)

.PHONY: all kernel

all: $(iso)

$(iso): kernel $(grub_cfg)
	mkdir -p ./build/isofiles/boot/grub
	cp $(kernel_blob) build/isofiles/boot/kernel.bin
	strip build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	$(GRUB_MKRESCUE) -o $(iso) build/isofiles
	rm -r build/isofiles


kernel:
	mkdir -p build
	make -C ./kernel
