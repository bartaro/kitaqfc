#ifndef SPRITE_H
#define SPRITE_H

#include "core.h"
#include "intrinsics.h"

// Keep this value within 1..64; tracking arrays and hardware OAM contain exactly 64 slots.
#ifndef SPRITE_MAX
#define SPRITE_MAX ((u8)64)
#endif

#define SPRITE_FLAG_PAL1     ((u8)0x01)
#define SPRITE_FLAG_PAL2     ((u8)0x02)
#define SPRITE_FLAG_PAL3     ((u8)0x03)
#define SPRITE_FLAG_PRIORITY ((u8)0x20)
#define SPRITE_FLAG_XFLIP    ((u8)0x40)
#define SPRITE_FLAG_YFLIP    ((u8)0x80)

// Signed per-part offsets are added modulo 256. This struct API has an explicit count,
// unlike the FF-terminated byte-stream API in metasprite.h.
typedef struct MetaSpritePart {
    s8 dx;
    s8 dy;
    u8 tile;
    u8 flags;
} MetaSpritePart;

// Initialize frame/ticks to valid values. Returned tile IDs advance by one regardless of hardware sprite size.
typedef struct SpriteAnim {
    u8 first_tile;
    u8 frame_count;
    u8 frame;
    u8 ticks;
    u8 ticks_per_frame;
} SpriteAnim;

extern u8 kq_sprite_active[64];
extern u8 kq_sprite_used;
extern u8 kq_sprite_height;

// Reset allocation/position state and hide each OAM slot through the intrinsic.
// The software height defaults to eight; hardware sprite size is configured elsewhere.
void sprite_init(void);
// Claim and hide the first free slot, or return 0xFF if all slots are active.
u8 sprite_alloc(void);
// Release an active valid slot and hide it; invalid/repeated frees are ignored.
void sprite_free(u8 id);
// Update software coordinates and the intrinsic OAM position for a valid slot.
void sprite_set_pos(u8 id, u8 x, u8 y);
// Replace the tile byte of a valid OAM slot without changing allocation state.
void sprite_set_tile(u8 id, u8 tile);
// Replace the hardware attribute byte of a valid OAM slot.
void sprite_set_flags(u8 id, u8 flags);
// Mark the software position off screen and hide its OAM entry without freeing it.
void sprite_hide(u8 id);
// Transfer intrinsic shadow OAM immediately; this wrapper does not wait for VBlank.
void sprite_flush_oam_now(void);
// Wait for NMI before requesting OAM DMA.
void sprite_flush_oam(void);
// Read the maintained allocation counter. Metasprite placement also treats it
// as a high-water mark, so mixed allocation styles may not yield a true active count.
u8 sprite_count_used(void);
// Flag software overlap estimates above eight sprites on one scanline.
u8 sprite_warn_scanline_overflow(void);
// Estimate peak overlap across 240 lines from software Y/height values.
// This does not model pixel transparency, OAM priority or every hardware overflow quirk.
u8 sprite_max_scanline_count(void);
// Activate consecutive slots and populate their OAM/position data. Return a
// partial part count if the range reaches SPRITE_MAX; callers must prevent
// byte-ID wraparound and collisions with other slot allocations.
u8 metasprite_draw(u8 first_id, u8 x, u8 y, const MetaSpritePart* parts, u8 count);
// Advance animation ticks and wrap the frame index, returning the selected tile.
// Zero ticks-per-frame freezes advancement; null state returns zero.
u8 anim_update(SpriteAnim* anim);

#endif
