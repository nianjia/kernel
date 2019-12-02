version ?= debug
ARCH := x86_64

linker_script := linker.ld

linker_flags := -T $(linker_script)
linker_flags += -Map build/$(ARCH)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

librinit ?= target/$(ARCH)/$(version)/$(name).a

assembly_source_files := $(wildcard src/arch/$(ARCH)/*.S)
assembly_object_files := $(patsubst src/arch/$(ARCH)/%.S, \
	build/$(arch)/%.o, $(assembly_source_files))

rinit := build/$(ARCH)/$(name).bin

.PHONY: clean cargo build

# compile assembly files
build/$(arch)/%.o: src/arch/$(ARCH)/%.S
	@mkdir -p $(shell dirname $@)
	@as -o $@ $<

build: cargo $(librinit) $(assembly_object_files) $(linker_script)
	@mkdir -p build/$(ARCH)
	@ld $(linker_flags) -o $(rinit) $(assembly_object_files) $(librinit)

clean:
	@rm -rf build
	@rm -rf target
