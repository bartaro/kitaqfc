#ifndef VRAM_QUEUE_H
#define VRAM_QUEUE_H

#include "intrinsics.h"

/* Safe frame-time VRAM queue. These names are compiler intrinsic entry points. */
void __vramq_clear(void);
void __vramq_put(u16 ppu_addr, u8 value);
void __vramq_copy(u16 ppu_addr, const u8* src, u8 len);
void __vramq_fill(u16 ppu_addr, u8 value, u8 len);
void __vramq_commit(void);
void __vramq_exec(void);
u8 __vramq_len(void);
u8 __vramq_overflow(void);
void __vramq_clear_overflow(void);
u8 __vramq_capacity(void);

#endif
