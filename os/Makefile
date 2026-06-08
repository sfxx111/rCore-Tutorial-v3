.PHONY: all clean

TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL := target/$(TARGET)/$(MODE)/os

all: $(KERNEL)

$(KERNEL): src/link_app.S
	cargo build --release

src/link_app.S:
	$(MAKE) -C ../user build

clean:
	cargo clean
	rm -f src/link_app.S
