extern kmain

global _start

section .rodata

gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.ptr:
    dw $ - gdt64 - 1
    dq gdt64

section .bss

align 4096

PML4: resb 4096
PDPT: resb 4096
PDT:  resb 4096

stack_bottom:
    resb (4096 * 4) * 4 ; 32KiB Stack
stack_top:

section .text

bits 32

_start:
    ; We should be loaded by a multiboot2 compliant bootloader, info:
    ; https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Machine-state

    cmp eax, 0x36d76289
    jne .error

    ; Keep the multiboot info ptr safe in edi
    mov edi, ebx

    ; Setup the stack
    mov esp, stack_top

    call _start.cpuid
    call _start.long_mode
    call _start.paging

    lgdt [gdt64.ptr]

    jmp gdt64.code:_long_start

.cpuid:
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ; in the FLAGS register. If we can flip it, CPUID is available.

    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, (1 << 21)

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .error
    ret

.long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .error

    ; use extended info to test if long mode is available
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .error
    ret

.paging:
.paging.map:
    ; map first P4 entry to P3 table
    mov eax, PDPT
    or eax, 0b11 ; present + writable
    mov [PML4], eax

    mov eax, PML4
    or eax, 0b11

    mov [PML4 + (8 * 511)], eax
    mov [PDPT + (8 * 511)], eax
    mov [PDT + (8 * 511)], eax

    ; map first P3 entry to P2 table
    mov eax, PDT
    or eax, 0b11 ; present + writable
    mov [PDPT], eax

    mov ecx, 0

.paging.map.inner:

    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page

    or eax, 0b10000011 ; present + writable + huge
    mov [PDT + ecx * 8], eax

    inc ecx
    cmp ecx, 511
    jne .paging.map.inner

.paging.enable:

    ; Load PML4 address into CR3
    mov eax, PML4
    mov cr3, eax

    ; Set PAE bit in CR4
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ; Set LM enable bit in the EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; enable paging in the CR0 register
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    ret

.error:
    hlt
    jmp .error

bits 64

extern kmain

_long_start:
    ; Empty all segment registers
    ; Most of them are ignored but the three that aren't shouldn't contain garbage
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Call into Rust
    call kmain

.spin:
    hlt
    jmp .spin
