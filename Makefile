project	:= bluesnow
arch	?= x86
NASM	:= nasm -f elf
LD		:= ld -m elf_i386 -n
QEMU	:= qemu-system-x86_64 -enable-kvm -monitor telnet:127.0.0.1:1234,server,nowait

kernel	:= build/kernel-$(arch).bin
iso		:= build/os-$(arch).iso
DIRISO	:= build/isofiles

target	?= $(arch)-$(project)
rust_os	:= target/$(target)/release/lib$(project).a
SHELL	:= /bin/bash

linker_script	:= src/arch/$(arch)/linker.ld
grub.cfg		:= src/arch/$(arch)/grub.cfg
asm_source		:= $(wildcard src/arch/$(arch)/*.asm)
asm_object		:= $(patsubst src/arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(asm_source))

all: $(kernel)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm Makefile
	@mkdir -p $(shell dirname $@)
	@$(NASM) $< -o $@

$(kernel): $(rust_os) $(asm_object) $(linker_script) Makefile
	@$(LD) -o $(kernel) -T $(linker_script) $(asm_object) $(rust_os)

$(iso): $(kernel) $(grub.cfg) Makefile
	@mkdir -p $(DIRISO)/boot/grub
	@cp $(grub.cfg) $(DIRISO)/boot/grub
	@cp $(kernel) $(DIRISO)/boot/kernel.bin
	@grub-mkrescue -o $(iso) $(DIRISO) 2>/dev/null

run: $(iso) Makefile
	@tmux info >&- || { echo -e "\033[38;5;16m ~~ NOT IN A VALID TMUX SESSION ~~\033[0m" ; exit 1; }
	@tmux split-window "tmux resize-pane -y 20; sleep 0.5; telnet 127.0.0.1 1234"
	@$(QEMU) -curses -cdrom $(iso)

clean:
	@cargo clean
	@rm -r build

$(rust_os): $(target).json Makefile
	@xargo build --release --target $(target)

kernel: $(rust_os)
iso: $(iso)

.PHONY: run clean kernel iso
