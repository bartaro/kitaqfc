#include "vram.h"

// Convert tile coordinates to an address using the NES 32-byte nametable row stride.
static u16 vram_xy_addr(u16 base, u8 x, u8 y)
{
    return (u16)(base + (u16)y * 32 + x);
}

// Clear the intrinsic queue and its overflow latch.
void vram_init(void)
{
    __vramq_clear();
    __vramq_clear_overflow();
}

// Discard queued commands and explicitly reset the overflow latch.
void vram_clear_queue(void)
{
    __vramq_clear();
    __vramq_clear_overflow();
}

// Queue one tile at the default nametable base. Return the current overflow
// status, which may include a failure from an earlier operation.
u8 vram_queue_bg_tile(u8 x, u8 y, u8 tile)
{
    __vramq_put(vram_xy_addr(VRAM_NT_BASE, x, y), tile);
    return (u8)(__vramq_overflow() == 0);
}

// Alias the queued background-tile operation.
u8 vram_queue_tile(u8 x, u8 y, u8 tile)
{
    return vram_queue_bg_tile(x, y, tile);
}

// Queue one fill per row. This is not transactional: capacity exhaustion can
// leave earlier rows queued, and the loop continues attempting later rows.
// Use y+h <=256 to avoid byte row wrap. Check queue capacity before a multirow submission if partial updates are unacceptable.
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

// Queue each tightly packed source row as a pointer-based copy record. Keep
// source storage and its ROM bank accessible until queue execution finishes;
// partial queuing remains possible on overflow.
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

// Split a byte transfer into copy records of at most 127 bytes. The runtime
// retains source pointers rather than copying payloads into the queue. Return
// the overflow latch state after all records have been attempted.
// These are intrinsic pointer-copy records, distinct from runtime.c literal records that copy payload bytes immediately.
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

// Alias the byte-copy queue helper; len remains a byte count, not a tile count.
u8 vram_queue_tiles(u16 dst, const void* src, u16 len)
{
    return vram_queue_memcpy(dst, src, len);
}

// Queue fills of at most 127 bytes each. Overflow can leave a partially queued
// transfer; no rollback is performed.
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

// Invoke the queue executor directly. Its ready flag must already be set by
// commit, and the caller supplies appropriate PPU timing.
void vram_flush_now(void)
{
    __vramq_exec();
}

// Commit queued records for consumption and wait for the NMI path.
void vram_flush(void)
{
    __vramq_commit();
    __nmi_wait();
}

// Return the encoded queue length in bytes, not the number of commands.
u8 vram_get_queue_used(void)
{
    return __vramq_len();
}

// Return the remaining command-buffer capacity in bytes. A command requires
// space for its complete encoded record, including its address and metadata.
// Read before committing: NMI may consume a committed queue asynchronously.
// The value describes encoded command bytes, whereas the GB API with this name returns command slots.
u8 vram_get_queue_free(void)
{
    return (u8)(__vramq_capacity() - __vramq_len());
}

// Return the queue's total capacity in bytes, independent of its current usage.
// This describes the command buffer, not free space in PPU VRAM.
u8 vram_get_queue_capacity(void)
{
    return __vramq_capacity();
}

// Read the overflow latch without clearing it.
u8 vram_get_overflowed(void)
{
    return __vramq_overflow();
}
