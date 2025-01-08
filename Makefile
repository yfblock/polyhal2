ARCH ?= aarch64
TARGET ?= aarch64-unknown-none-softfloat
MODE ?= debug
QEMU_EXEC := qemu-system-$(ARCH) -machine virt -nographic -cpu cortex-a72 \
				-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors \
			 	-serial mon:stdio
all:

build:
	cargo build --target $(TARGET) --package test-boot --bin test_no_page_boot
	rust-objcopy target/$(TARGET)/$(MODE)/test_no_page_boot --strip-all -O binary target/$(TARGET)/$(MODE)/test_no_page_boot.bin
qemu: build
#	$(QEMU_EXEC) -kernel target/$(TARGET)/$(MODE)/test_no_page_boot
	$(QEMU_EXEC) -kernel target/$(TARGET)/$(MODE)/test_no_page_boot.bin
.PHONY:
