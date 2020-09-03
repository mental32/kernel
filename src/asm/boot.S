# Disable generation of compressed instructions.
.option norvc

# Define a .text.init section. The .text.init is put at the
# starting address so that the entry _start is put at the RISC-V
# address 0x8000_0000.
.section .text.init

# Execution starts here.
.global _start
_start:

	# Disable linker instruction relaxation for the `la` instruction below.
	# This disallows the assembler from assuming that `gp` is already initialized.
	# This causes the value stored in `gp` to be calculated from `pc`.
	# The job of the global pointer is to give the linker the ability to address
	# memory relative to GP instead of as an absolute address.
.option push
.option norelax
	la		gp, _global_pointer
.option pop
	# SATP should be zero, but let's make sure. Each HART has its own
	# SATP register.
	csrw	satp, zero
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f

	# Set all bytes in the BSS section to zero.
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	# The stack grows from bottom to top, so we put the stack pointer
	# to the very end of the stack range.
	la		sp, _stack_end
	# Setting `mstatus` register:
	# 0b01 << 11: Machine's previous protection mode is 2 (MPP=2).
	li		t0, 0b11 << 11 | (1 << 13)
	csrw	mstatus, t0
	# Do not allow interrupts while running kinit
	csrw	mie, zero
	# Machine's exception program counter (MEPC) is set to `kinit`.
	la		t1, kinit
	csrw	mepc, t1
	# Set the return address to get us into supervisor mode
	la		ra, 2f
	# We use mret here so that the mstatus register is properly updated.
	mret
2:
    wfi
    j 2b
