# Common code in 32-bit, prepare states to enter 64-bit.
.section .text
.code32
.global _start
_start:
    mov     edi, eax        # arg1: magic: 0x2BADB002
    mov     esi, ebx        # arg2: multiboot info

    lgdt    [.Ltmp_gdt_desc - {offset}]             # load the temporary GDT
    # set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    call    build_pt

    # set PAE, PGE bit in CR4
    mov     eax, cr4
    or      eax, {cr4}
    mov     cr4, eax

    # load the temporary page table
    lea     eax, [boot_page - {offset}]
    mov     cr3, eax

    # set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    rdmsr
    or      eax, {efer}
    wrmsr

    # set protected mode, write protect, paging bit in CR0
    mov     eax, cr0
    or      eax, {cr0}
    mov     cr0, eax

    ljmp    0x10, offset bsp_entry64 - {offset}    # 0x10 is code64 segment

# Build Page Table
# size of boot_page is 3 * 0x1000
# 0x0000 - 0x1000: page_table_root   4 level
# 0x1000 - 0x2000: page_table_pdpt   3 level
# 0x2000 - 0x3000: page_table_pt     2 level (2MB Page)   
build_pt:
    # Build boot_page
    lea     eax, [boot_page - {offset}]

    mov     ebx, eax
    add     ebx, 0x1003             # flags: 0x3   PRESENT | WRITABLE
    mov     [eax], ebx              # 0x0000_0000_0000_0000 ~ 0x0000_0000_ffff_ffff
    mov     [eax + 8*511], ebx      # 0xffff_ff80_0000_0000 ~ 0xffff_ff80_ffff_ffff

build_pdpt:
    add     eax, 0x1000
    mov     ebx, eax
    add     ebx, 0x1003             # flags: 0x3   PRESENT | WRITABLE
    mov     [eax], ebx              # 0x0000_0000_0000_0000 ~ 0x0000_0000_3fff_ffff

    # build a loop to fill 2MB Page
    # Mapping 0x0000_0000 - 0x3fff_ffff
    mov     ecx, 512
    add     eax, 0x1000
    mov     ebx, 0x83               # flags: 0x83  PRESENT | WRITABLE | HUGE_PAGE
build_pd:
    mov     [eax], ebx
    add     ebx, 0x200000
    add     eax, 0x8
    loop    build_pd
    ret

.code64
bsp_entry64:
    # clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set RSP to boot stack
    movabs  rsp, offset bstack_top

    # call rust_entry(magic, mbi)
    movabs  rax, offset {entry}
    call    rax
    jmp     .Lhlt

.Lhlt:
    hlt
    jmp     .Lhlt

.section .rodata
.balign 8
.Ltmp_gdt_desc:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   # limit
    .long   .Ltmp_gdt - {offset}            # base

.section .data
.balign 16
.Ltmp_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00af9b000000ffff    # 0x10: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x18: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
.Ltmp_gdt_end:
