ARCH ?= aarch64
MODE ?= debug
QEMU_EXEC := qemu-system-$(ARCH) -nographic -m 2G
QEMU_LOG := off

ifeq ($(ARCH), aarch64) 
	TARGET := aarch64-unknown-none-softfloat
	QEMU_EXEC += -machine virt -cpu cortex-a72
else ifeq ($(ARCH), loongarch64)
	TARGET := loongarch64-unknown-none-softfloat
	QEMU_EXEC += -machine virt
else ifeq ($(ARCH), x86_64)
  	TARGET := x86_64-unknown-none
	QEMU_EXEC += -machine q35 -cpu IvyBridge-v2
else ifeq ($(ARCH), riscv64)
	TARGET := riscv64gc-unknown-none-elf
	QEMU_EXEC += -machine virt
endif

ELF := target/$(TARGET)/$(MODE)/test-boot
BIN := $(ELF).bin

ifneq ($(filter $(ARCH), aarch64 riscv64),)
	QEMU_EXEC += -kernel $(BIN)
else ifneq ($(filter $(ARCH), loongarch64 x86_64),)
	QEMU_EXEC += -kernel $(ELF)
endif

ifeq ($(QEMU_LOG), on)
QEMU_EXEC += -D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors \
			 	-serial mon:stdio
endif

all:

build:
	cargo build --target $(TARGET) --package test-boot
	rust-objcopy $(ELF) --strip-all -O binary $(BIN)
qemu: build
	$(QEMU_EXEC) 
clean:
	rm -rf target/
check:
	cargo fmt --all -- --check
	cargo clippy --target loongarch64-unknown-none-softfloat --all-features -- -A clippy::new_without_default
	cargo clippy --target aarch64-unknown-none-softfloat --all-features -- -A clippy::new_without_default
	cargo clippy --target riscv64gc-unknown-none-elf --all-features -- -A clippy::new_without_default
	cargo clippy --target x86_64-unknown-none --all-features -- -A clippy::new_without_default
	cargo build --target loongarch64-unknown-none-softfloat --all-features
	cargo build --target aarch64-unknown-none-softfloat --all-features
	cargo build --target riscv64gc-unknown-none-elf --all-features
	cargo build --target x86_64-unknown-none --all-features

	RUSTDOCFLAGS="-Zunstable-options --enable-index-page -D rustdoc::broken_intra_doc_links -D missing-docs" \
	cargo doc --no-deps --all-features

.PHONY: all build qemu check
