#ifndef SPRITE_H
#define SPRITE_H

#include "core.h"
#include "intrinsics.h"

#ifndef SPRITE_MAX
#define SPRITE_MAX ((u8)64)
#endif

#define SPRITE_FLAG_PAL1     ((u8)0x01)
#define SPRITE_FLAG_PAL2     ((u8)0x02)
#define SPRITE_FLAG_PAL3     ((u8)0x03)
#define SPRITE_FLAG_PRIORITY ((u8)0x20)
#define SPRITE_FLAG_XFLIP    ((u8)0x40)
#define SPRITE_FLAG_YFLIP    ((u8)0x80)

typedef struct MetaSpritePart {
    s8 dx;
    s8 dy;
    u8 tile;
    u8 flags;
} MetaSpritePart;

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

void sprite_init(void);
u8 sprite_alloc(void);
void sprite_free(u8 id);
void sprite_set_pos(u8 id, u8 x, u8 y);
void sprite_set_tile(u8 id, u8 tile);
void sprite_set_flags(u8 id, u8 flags);
void sprite_hide(u8 id);
void sprite_flush_oam_now(void);
void sprite_flush_oam(void);
u8 sprite_count_used(void);
u8 sprite_warn_scanline_overflow(void);
u8 sprite_max_scanline_count(void);
u8 metasprite_draw(u8 first_id, u8 x, u8 y, const MetaSpritePart* parts, u8 count);
u8 anim_update(SpriteAnim* anim);

#endif
