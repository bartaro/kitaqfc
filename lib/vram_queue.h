#ifndef VRAM_QUEUE_H
#define VRAM_QUEUE_H

#include "intrinsics.h"

/* Safe frame-time VRAM queue. These names are compiler intrinsic entry points. */
// Reset pending intrinsic-queue records; this is separate from runtime.c nes_vram_queue_clear.
void __vramq_clear(void);
// Append one address/value update; use the overflow latch to detect capacity failure.
void __vramq_put(u16 ppu_addr, u8 value);
// Retain the source address for later consumption; keep its storage and bank mapping valid.
void __vramq_copy(u16 ppu_addr, const u8* src, u8 len);
// Append a repeated-byte update; the source value is stored in the record.
void __vramq_fill(u16 ppu_addr, u8 value, u8 len);
// Publish the pending records to the configured consumer. Do not mutate them while consumption can run.
void __vramq_commit(void);
// Execute a ready queue directly; committing and safe PPU timing are caller responsibilities.
void __vramq_exec(void);
// Return encoded bytes currently occupied, not transfer payload bytes or command count.
u8 __vramq_len(void);
// Read the latched capacity failure; reading does not reset it.
u8 __vramq_overflow(void);
// Clear the overflow latch explicitly after handling the failed submission.
void __vramq_clear_overflow(void);
// Return total encoded command-buffer capacity; this is independent of PPU VRAM capacity.
u8 __vramq_capacity(void);

#endif
