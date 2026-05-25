TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin

OBJCOPY := rust-objcopy --binary-architecture=riscv64

build:
	@cargo build --release

$(KERNEL_BIN): build
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

run: env $(KERNEL_BIN)
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios ../bootloader/opensbi.bin \
		-device loader,file=$(KERNEL_BIN),addr=0x80200000

env:
	@# 确保构建环境存在，这里可以留空

.PHONY: build run env
