target := target-x86_64

kernel_lib  := ./target/$(target)/debug/libkernel.a

assembly_source_files := $(wildcard boot/*.asm)
assembly_object_files := $(patsubst boot/%.asm, build/asm/%.o, $(assembly_source_files))

NASM = nasm

linker_script := $(common)/x86_64.linker.ld

LD      := ld
LD_OPTS := -n --gc-sections -T $(linker_script)


.PHONY: all

all: $(kernel_blob)

$(kernel_blob): $(kernel_lib) $(assembly_object_files)
	$(LD) $(LD_OPTS) -o $(kernel_blob) \
		$(assembly_object_files) $(kernel_lib)

$(kernel_lib): $(target).json
	RUST_TARGET_PATH="$(shell pwd)" cargo xbuild --target $(target)
	rm $(target).json

$(target).json:
	cp $(common)/$(target).json $(target).json

build/asm/%.o: boot/%.asm
	mkdir -p $(shell dirname $@)
	$(NASM) -felf64 $< -o $@
