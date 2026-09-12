#include "vram.h"

static u16 vram_xy_addr(u16 base, u8 x, u8 y)
{
    return (u16)(base + (u16)y * 32 + x);
}

void vram_init(void)
{
    __vramq_clear();
    __vramq_clear_overflow();
}

void vram_clear_queue(void)
{
    __vramq_clear();
    __vramq_clear_overflow();
}

u8 vram_queue_bg_tile(u8 x, u8 y, u8 tile)
{
    __vramq_put(vram_xy_addr(VRAM_NT_BASE, x, y), tile);
    return (u8)(__vramq_overflow() == 0);
}

u8 vram_queue_tile(u8 x, u8 y, u8 tile)
{
    return vram_queue_bg_tile(x, y, tile);
}

u8 vram_queue_bg_rect(u8 x, u8 y, u8 w, u8 h, u8 tile)
{
    u8 row;
    row = 0;
    while (row < h) {
        __vramq_fill(vram_xy_addr(VRAM_NT_BASE, x, (u8)(y + row)), tile, w);
        row = (u8)(row + 1);
    }
    return (u8)(__vramq_overflow() == 0);
}

u8 vram_queue_bg_block(u16 base, u8 x, u8 y, u8 w, u8 h, const u8* src)
{
    u8 row;
    row = 0;
    while (row < h) {
        __vramq_copy(vram_xy_addr(base, x, (u8)(y + row)), src, w);
        src = src + w;
        row = (u8)(row + 1);
    }
    return (u8)(__vramq_overflow() == 0);
}

u8 vram_queue_memcpy(u16 dst, const void* src, u16 len)
{
    const u8* p;
    u8 chunk;
    p = (const u8*)src;
    while (len != 0) {
        if (len > 127) chunk = 127;
        else chunk = (u8)len;
        __vramq_copy(dst, p, chunk);
        dst = (u16)(dst + chunk);
        p = p + chunk;
        len = (u16)(len - chunk);
    }
    return (u8)(__vramq_overflow() == 0);
}

u8 vram_queue_tiles(u16 dst, const void* src, u16 len)
{
    return vram_queue_memcpy(dst, src, len);
}

u8 vram_queue_memset(u16 dst, u8 value, u16 len)
{
    u8 chunk;
    while (len != 0) {
        if (len > 127) chunk = 127;
        else chunk = (u8)len;
        __vramq_fill(dst, value, chunk);
        dst = (u16)(dst + chunk);
        len = (u16)(len - chunk);
    }
    return (u8)(__vramq_overflow() == 0);
}

void vram_flush_now(void)
{
    __vramq_exec();
}

void vram_flush(void)
{
    __vramq_commit();
    __nmi_wait();
}

u8 vram_get_queue_used(void)
{
    return __vramq_len();
}

u8 vram_get_queue_free(void)
{
    return __vramq_capacity();
}

u8 vram_get_overflowed(void)
{
    return __vramq_overflow();
}
