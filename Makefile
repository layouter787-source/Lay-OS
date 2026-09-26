BUILD := build
KERNEL_ELF := $(BUILD)/kernel.elf
KERNEL_BIN := $(BUILD)/kernel.bin
IMAGE := $(BUILD)/lay-os.img

all: $(IMAGE)

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/boot.bin: boot/boot.asm | $(BUILD)
	nasm -f bin $< -o $@

$(BUILD)/kernel_entry.o: kernel/entry.asm | $(BUILD)
	nasm -f elf64 $< -o $@

$(BUILD)/liblay_kernel.a: kernel/main.rs Cargo.toml | $(BUILD)
	cargo build --release

$(KERNEL_ELF): $(BUILD)/kernel_entry.o $(BUILD)/liblay_kernel.a kernel/linker.ld
	ld -nostdlib -z max-page-size=0x1000 -T kernel/linker.ld -o $@ $(BUILD)/kernel_entry.o $(BUILD)/liblay_kernel.a

$(KERNEL_BIN): $(KERNEL_ELF)
	objcopy -O binary $< $@

$(IMAGE): $(BUILD)/boot.bin $(KERNEL_BIN)
	cat $(BUILD)/boot.bin $(KERNEL_BIN) > $@
	truncate -s %512 $@

clean:
	rm -rf $(BUILD)

.PHONY: all clean
