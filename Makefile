ARCH ?= aarch64
MODE ?= debug
QEMU_EXEC := qemu-system-$(ARCH) -nographic -m 1G
QEMU_LOG := off

ifeq ($(ARCH), aarch64) 
	TARGET := aarch64-unknown-none-softfloat
	QEMU_EXEC += -machine virt -cpu cortex-a72
else ifeq ($(ARCH), loongarch64)
	TARGET := loongarch64-unknown-none-softfloat
	QEMU_EXEC += -machine virt
endif

ELF := target/$(TARGET)/$(MODE)/test_no_page_boot
BIN := $(ELF).bin

ifneq ($(filter $(ARCH), aarch64),)
	QEMU_EXEC += -kernel $(BIN)
else ifneq ($(filter $(ARCH), loongarch64),)
	QEMU_EXEC += -kernel $(ELF)
endif

ifeq ($(QEMU_LOG), on)
QEMU_EXEC += -D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors \
			 	-serial mon:stdio
endif

all:

build:
	cargo build --target $(TARGET) --package test-boot --bin test_no_page_boot
	rust-objcopy $(ELF) --strip-all -O binary $(BIN)
qemu: build
#	$(QEMU_EXEC) -kernel target/$(TARGET)/$(MODE)/test_no_page_boot
	$(QEMU_EXEC) 
clean:
	rm -rf target/
.PHONY: all build qemu
