arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-unknown-none-gnu
rust_os := target/$(target)/debug/libpark_os.a

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
    build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso gdb initial-setup

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -hda ./disk/disk.iso -boot order=d -s -k en-gb

debug: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s -S

debug_int: $(iso)
	@qemu-system-x86_64 -d int -no-reboot -cdrom build/os-x86_64.iso

gdb:
	@tools/rust-gdb -tui "build/kernel-x86_64.bin" -ex "target remote :1234"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

cargo:
	@cargo rustc --target $(target) -- -Z no-landing-pads -C no-redzone -C target-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@

libcore:
	git submodule update
	cp $(target).json nightly-libcore/
	cd nightly-libcore && cargo build --release --features disable_float --target $(target)
	mkdir -p ~/.multirust/toolchains/nightly/lib/rustlib/$(target)/lib
	cp nightly-libcore/target/$(target)/release/libcore.rlib ~/.multirust/toolchains/nightly/lib/rustlib/$(target)/lib
