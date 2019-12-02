ARCH := x86_64

kernel := kernel/build/$(ARCH)/libkernel.bin
rinit := rinit/build/$(ARCH)/librinit.bin

.PHONY: all clean run run-release rinit rinit-release kernel kernel-release doc-kernel doc-kernel-deploy

kernel:
	@make -C kernel build

kernel-release:
	@make -C kernel version=release build

rinit:
	@make -C rinit build

rinit-release:
	@make -C rinit version=release build

run: kernel rinit
	@qemu-system-$(ARCH) -kernel $(kernel) -initrd $(rinit) -serial stdio --no-reboot

run-release: kernel-release rinit-release
	@qemu-system-$(ARCH) -kernel $(kernel) -initrd $(rinit) -serial stdio --no-reboot

debug: kernel rinit
	@qemu-system-$(ARCH) -d int -no-reboot -s -S -kernel $(kernel) -initrd $(rinit) -serial stdio

noreboot: kernel rinit
	@qemu-system-$(ARCH) -d int -no-reboot -kernel $(kernel) -initrd $(rinit) -serial stdio

noreboot-release: kernel-release rinit-release
	@qemu-system-$(ARCH) -d int -no-reboot -kernel $(kernel) -initrd $(rinit) -serial stdio

test: kernel-release
	@make -C tests/userspace version=release kernel=$(shell realpath $(kernel)) test=allocator test

gdb:
	@gdb $(kernel) -ex "target remote :1234"

clean:
	@make -C kernel clean
	@make -C rinit clean
	@make -C tests/userspace kernel=none test=none clean

doc-kernel:
	@rm -rf kernel/target/doc
	@cargo rustdoc --manifest-path kernel/Cargo.toml -- \
		--no-defaults \
		--passes strip-hidden \
		--passes collapse-docs \
		--passes unindent-comments \
		--passes strip-priv-imports

doc-kernel-deploy: doc-kernel
	@rsync -vraP --delete-after kernel/target/doc/ deploy@that.world:~/~docs/rux
