arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := src/arch/$(arch)/asm/linker.ld
grub_cfg := src/arch/$(arch)/asm/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/asm/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/asm/%.asm, \
		build/arch/$(arch)/asm/%.o, $(assembly_source_files))

target ?= $(arch)-unknown-none-gnu
rust_os := target/$(target)/debug/libcykusz_rust64.a

.PHONY: all clean run iso

all: $(kernel)

clean:
	rm -r build
	cargo clean

run: $(iso)
	qemu-system-x86_64 -drive format=raw,file=$(iso) -no-reboot -m 1024

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -d /usr/lib/grub/i386-pc/ -o $(iso) build/isofiles 2> /dev/null
	rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	ld -n --gc-sections  -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

build:
	./update_core_nightly.sh

cargo: build
	RUSTFLAGS="-L ./build"  cargo rustc --target $(target) --verbose -- -Z no-landing-pads

# compile assembly files
build/arch/$(arch)/asm/%.o: src/arch/$(arch)/asm/%.asm
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@
