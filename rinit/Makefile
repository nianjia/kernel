name := librinit
TARGET_SPEC := ../x86_64.json

include ../userspace.mk

cargo:
ifeq ($(version),release)
	@cargo xbuild --release --target $(TARGET_SPEC) --verbose
else
	@cargo xbuild --target $(TARGET_SPEC) --verbose
endif
