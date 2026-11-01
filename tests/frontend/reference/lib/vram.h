#ifndef VRAM_H
#define VRAM_H

#include "core.h"
#include "intrinsics.h"

// Coordinates use 32-byte nametable rows. These wrappers do not clip to the 32x30 visible tile area.
#ifndef VRAM_NT_BASE
#define VRAM_NT_BASE ((u16)0x2000)
#endif

// Clear the intrinsic queue and its overflow latch.
void vram_init(void);
// Discard queued commands and explicitly reset the overflow latch.
void vram_clear_queue(void);
// Queue one tile at the default nametable base. Return the current overflow
// status, which may include a failure from an earlier operation.
u8 vram_queue_bg_tile(u8 x, u8 y, u8 tile);
// Alias the queued background-tile operation.
u8 vram_queue_tile(u8 x, u8 y, u8 tile);
// Queue one fill per row. This is not transactional: capacity exhaustion can
// leave earlier rows queued, and the loop continues attempting later rows.
u8 vram_queue_bg_rect(u8 x, u8 y, u8 w, u8 h, u8 tile);
// Queue each tightly packed source row as a pointer-based copy record. Keep
// source storage and its ROM bank accessible until queue execution finishes;
// partial queuing remains possible on overflow.
u8 vram_queue_bg_block(u16 base, u8 x, u8 y, u8 w, u8 h, const u8* src);
// Split a byte transfer into copy records of at most 127 bytes. The runtime
// retains source pointers rather than copying payloads into the queue. Return
// the overflow latch state after all records have been attempted.
u8 vram_queue_memcpy(u16 dst, const void* src, u16 len);
// Alias the byte-copy queue helper; len remains a byte count, not a tile count.
u8 vram_queue_tiles(u16 dst, const void* src, u16 len);
// Queue fills of at most 127 bytes each. Overflow can leave a partially queued
// transfer; no rollback is performed.
u8 vram_queue_memset(u16 dst, u8 value, u16 len);
// Invoke the queue executor directly. Its ready flag must already be set by
// commit, and the caller supplies appropriate PPU timing.
void vram_flush_now(void);
// Commit queued records for consumption and wait for the NMI path.
void vram_flush(void);
// Return the encoded queue length in bytes, not the number of commands.
u8 vram_get_queue_used(void);
// Return the remaining command-buffer capacity in bytes. A command requires
// space for its complete encoded record, including its address and metadata.
// Read before committing: NMI may consume a committed queue asynchronously.
u8 vram_get_queue_free(void);
// Return the queue's total capacity in bytes, independent of its current usage.
// This describes the command buffer, not free space in PPU VRAM.
u8 vram_get_queue_capacity(void);
// Read the overflow latch without clearing it.
u8 vram_get_overflowed(void);

#endif
