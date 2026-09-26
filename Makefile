BUILD := build
KERNEL_ELF := $(BUILD)/kernel.elf
KERNEL_BIN := $(BUILD)/kernel.bin
IMAGE := $(BUILD)/lay-os.img

RUSTFLAGS := -C opt-level=2 -C panic=abort -C no-redzone
IMAGE_SIZE := 524800

all: $(IMAGE)

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/boot.bin: boot/boot.asm | $(BUILD)
	nasm -f bin $< -o $@

$(BUILD)/kernel_entry.o: kernel/entry.asm | $(BUILD)
	nasm -f elf64 $< -o $@

$(BUILD)/interrupt_stubs.o: kernel/interrupt_stubs.asm | $(BUILD)
	nasm -f elf64 $< -o $@

$(BUILD)/liblay_kernel.a: $(wildcard kernel/*.rs) Cargo.toml
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release

$(KERNEL_ELF): $(BUILD)/kernel_entry.o $(BUILD)/interrupt_stubs.o $(BUILD)/liblay_kernel.a kernel/linker.ld
	ld -nostdlib -z max-page-size=0x1000 -T kernel/linker.ld -o $@ $(BUILD)/kernel_entry.o $(BUILD)/interrupt_stubs.o $(BUILD)/liblay_kernel.a

$(KERNEL_BIN): $(KERNEL_ELF)
	objcopy -O binary $< $@

$(IMAGE): $(BUILD)/boot.bin $(KERNEL_BIN)
	@size=$$(stat -c %s $(KERNEL_BIN)); test $$size -le 524288 || { echo "kernel.bin exceeds 1024-sector boot window"; exit 1; }
	cat $(BUILD)/boot.bin $(KERNEL_BIN) > $@
	truncate -s $(IMAGE_SIZE) $@

clean:
	rm -rf $(BUILD)

.PHONY: all clean
