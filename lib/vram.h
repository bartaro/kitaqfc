#ifndef VRAM_H
#define VRAM_H

#include "core.h"
#include "intrinsics.h"

#ifndef VRAM_NT_BASE
#define VRAM_NT_BASE ((u16)0x2000)
#endif

void vram_init(void);
void vram_clear_queue(void);
u8 vram_queue_bg_tile(u8 x, u8 y, u8 tile);
u8 vram_queue_tile(u8 x, u8 y, u8 tile);
u8 vram_queue_bg_rect(u8 x, u8 y, u8 w, u8 h, u8 tile);
u8 vram_queue_bg_block(u16 base, u8 x, u8 y, u8 w, u8 h, const u8* src);
u8 vram_queue_memcpy(u16 dst, const void* src, u16 len);
u8 vram_queue_tiles(u16 dst, const void* src, u16 len);
u8 vram_queue_memset(u16 dst, u8 value, u16 len);
void vram_flush_now(void);
void vram_flush(void);
u8 vram_get_queue_used(void);
u8 vram_get_queue_free(void);
u8 vram_get_overflowed(void);

#endif
