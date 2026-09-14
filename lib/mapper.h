#ifndef MAPPER_H
#define MAPPER_H

#include "intrinsics.h"

// Resolve a symbol or function placement to its byte bank number at compilation time.
u8 __bankof(u16 symbol_or_function);
// Select the mapper-specific PRG bank; the common bank and legal bank range depend on the target board.
void __bankswitch(u8 bank);
// Use the same mapper PRG-switch helper as __bankswitch; retain executable code in a valid mapping.
void __prg_bank_set(u8 bank);
// Invoke the compiler banked-call path; target placement and calling convention must match the selected mapper.
u8 __farcall(u8 bank, u16 func);
// Switch to the source bank, copy len bytes forward and restore the previous bank.
// Keep destination RAM and helper code accessible throughout; overlapping copies are not memmove.
void __far_memcpy(u8* dst, u8 bank, u16 src, u16 len);
// Temporarily select bank, read one byte at addr and restore the prior bank mapping.
u8 __farpeek8(u8 bank, u16 addr);
// Temporarily select bank and read a little-endian word, then restore the previous mapping.
u16 __farpeek16(u8 bank, u16 addr);
// Return the low byte of the mapper number selected for this build; no cartridge probe is performed.
u8 __mapper_id(void);
// Select a CNROM bank or MMC3 register 0 bank; unsupported mapper profiles produce a compile error.
void __chr_bank_set(u8 bank);
// Use the same CNROM/MMC3 register 0 selection as __chr_bank_set.
void __chr_bank_set0(u8 bank);
// Select MMC3 register 1, or the same full CNROM bank register; this is mapper-specific banking.
void __chr_bank_set1(u8 bank);
// Emit the selected mapper mirroring operation; available modes and writable control depend on the board.
void __mirroring_set(u8 mode);
// Write the MMC3 IRQ latch and reload registers. This does not enable IRQ delivery.
void __irq_scanline_set(u8 line);
// Alias the MMC3 IRQ latch/reload operation; other mapper profiles are rejected.
void __mapper_irq_set(u8 scanline);
// Write MMC3 E001. The CPU IRQ mask and a valid handler are separate caller responsibilities.
void __mapper_irq_enable(void);
// Write MMC3 E000 to disable and acknowledge mapper IRQs.
void __mapper_irq_disable(void);
// Use the same E000 write as disable; explicitly re-enable when another interrupt is needed.
void __mapper_irq_ack(void);
// Wait for the current hit flag to clear, then for a new hit. No timeout is provided.
void __sprite0_wait_hit(void);
// Wait for a new sprite-zero hit, reset the PPU latch and write X/Y scroll; requires a visible hit setup.
void __split_scroll_sprite0(u8 x, u8 y);

#endif
