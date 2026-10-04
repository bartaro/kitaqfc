#ifndef MAPPER_H
#define MAPPER_H

#include "intrinsics.h"

u8 __bankof(u16 symbol_or_function);
void __bankswitch(u8 bank);
void __prg_bank_set(u8 bank);
u8 __farcall(u8 bank, u16 func);
void __far_memcpy(u8* dst, u8 bank, u16 src, u16 len);
u8 __farpeek8(u8 bank, u16 addr);
u16 __farpeek16(u8 bank, u16 addr);
u8 __mapper_id(void);
void __chr_bank_set(u8 bank);
void __chr_bank_set0(u8 bank);
void __chr_bank_set1(u8 bank);
void __mirroring_set(u8 mode);
void __irq_scanline_set(u8 line);
void __mapper_irq_set(u8 scanline);
void __mapper_irq_enable(void);
void __mapper_irq_disable(void);
void __mapper_irq_ack(void);
void __sprite0_wait_hit(void);
void __split_scroll_sprite0(u8 x, u8 y);

#endif
