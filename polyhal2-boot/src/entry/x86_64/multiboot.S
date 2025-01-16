# Bootstrapping from 32-bit with the Multiboot specification.
# See https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

.section .multiboot
.balign 4
.type multiboot_header, @object
multiboot_header:
    .int    {mb_hdr_magic}                      # magic: 0x1BADB002
    .int    {mb_hdr_flags}                      # flags
    .int    -({mb_hdr_magic} + {mb_hdr_flags})  # checksum
    .int    multiboot_header - {offset}         # header_addr
    .int    _skernel - {offset}                 # load_addr
    .int    _sbss - {offset}                    # load_end
    .int    end - {offset}                      # bss_end_addr
    .int    _start - {offset}                   # entry_addr
    .int    {graphic_mode}
    .int    0
    .int    0
    .int    32
