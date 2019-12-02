version ?= debug
ARCH := x86_64
TARGET_SPEC := ../x86_64.json
kernel := build/$(ARCH)/libkernel.bin
rust_os := target/$(ARCH)/$(version)/libkernel.a

linker_script := src/arch/$(ARCH)/linker.ld
linker_flags := -T $(linker_script)
linker_flags += -Map build/$(ARCH)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

assembly_source_files := $(wildcard src/arch/$(ARCH)/*.S)
assembly_object_files := $(patsubst src/arch/$(ARCH)/%.S, \
	build/$(ARCH)/%.o, $(assembly_source_files))

.PHONY: all clean run cargo kernel

all: $(kernel)

clean:
	@rm -r build
	@rm -r target

build: cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@ld $(linker_flags) -o $(kernel).elf64 $(assembly_object_files) $(rust_os)
	@objcopy $(kernel).elf64 -F elf32-i386 $(kernel)

cargo:
ifeq ($(version),release)
	@cargo xbuild --release --target $(TARGET_SPEC)
else
	@cargo xbuild --target $(TARGET_SPEC)
endif

# compile assembly files
build/$(ARCH)/%.o: src/arch/$(ARCH)/%.S
	@mkdir -p $(shell dirname $@)
	@$(AS) -o $@ $<
