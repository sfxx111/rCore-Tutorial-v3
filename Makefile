.PHONY: all clean run

all: os

os:
	$(MAKE) -C os

clean:
	$(MAKE) -C os clean
	$(MAKE) -C user clean

run:
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios none \
	-serial mon:stdio \
	-monitor none \
	-kernel target/riscv64gc-unknown-none-elf/release/os
