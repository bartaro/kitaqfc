#include "sprite.h"

u8 kq_sprite_active[64];
u8 kq_sprite_x[64];
u8 kq_sprite_y[64];
u8 kq_sprite_used;
u8 kq_sprite_height;

static u8 sprite_valid(u8 id)
{
    return (u8)(id < SPRITE_MAX);
}

void sprite_init(void)
{
    u8 i;
    kq_sprite_used = 0;
    kq_sprite_height = 8;
    i = 0;
    while (i < SPRITE_MAX) {
        kq_sprite_active[i] = 0;
        kq_sprite_x[i] = 0;
        kq_sprite_y[i] = 0;
        __sprite_hide(i);
        i = (u8)(i + 1);
    }
}

u8 sprite_alloc(void)
{
    u8 i;
    i = 0;
    while (i < SPRITE_MAX) {
        if (kq_sprite_active[i] == 0) {
            kq_sprite_active[i] = 1;
            kq_sprite_used = (u8)(kq_sprite_used + 1);
            sprite_hide(i);
            return i;
        }
        i = (u8)(i + 1);
    }
    return 0xFF;
}

void sprite_free(u8 id)
{
    if (sprite_valid(id) == 0) return;
    if (kq_sprite_active[id] == 0) return;
    kq_sprite_active[id] = 0;
    if (kq_sprite_used != 0) kq_sprite_used = (u8)(kq_sprite_used - 1);
    sprite_hide(id);
}

void sprite_set_pos(u8 id, u8 x, u8 y)
{
    if (sprite_valid(id) == 0) return;
    kq_sprite_x[id] = x;
    kq_sprite_y[id] = y;
    __sprite_move(id, x, y);
}

void sprite_set_tile(u8 id, u8 tile)
{
    if (sprite_valid(id) == 0) return;
    __sprite_tile(id, tile);
}

void sprite_set_flags(u8 id, u8 flags)
{
    if (sprite_valid(id) == 0) return;
    __sprite_attr(id, flags);
}

void sprite_hide(u8 id)
{
    if (sprite_valid(id) == 0) return;
    kq_sprite_x[id] = 0;
    kq_sprite_y[id] = 0xF0;
    __sprite_hide(id);
}

void sprite_flush_oam_now(void)
{
    __oam_dma();
}

void sprite_flush_oam(void)
{
    __nmi_wait();
    __oam_dma();
}

u8 sprite_count_used(void)
{
    return kq_sprite_used;
}

u8 sprite_max_scanline_count(void)
{
    u8 line;
    u8 best;
    line = 0;
    best = 0;
    while (line < 240) {
        u8 i;
        u8 count;
        i = 0;
        count = 0;
        while (i < SPRITE_MAX) {
            if (kq_sprite_active[i] != 0) {
                if (line >= kq_sprite_y[i] && line < (u8)(kq_sprite_y[i] + kq_sprite_height)) {
                    count = (u8)(count + 1);
                }
            }
            i = (u8)(i + 1);
        }
        if (count > best) best = count;
        line = (u8)(line + 1);
    }
    return best;
}

u8 sprite_warn_scanline_overflow(void)
{
    return (u8)(sprite_max_scanline_count() > 8);
}

u8 metasprite_draw(u8 first_id, u8 x, u8 y, const MetaSpritePart* parts, u8 count)
{
    u8 i;
    i = 0;
    while (i < count) {
        u8 id;
        id = (u8)(first_id + i);
        if (id >= SPRITE_MAX) return i;
        kq_sprite_active[id] = 1;
        __sprite_set(id, (u8)(x + parts[i].dx), (u8)(y + parts[i].dy), parts[i].tile, parts[i].flags);
        kq_sprite_x[id] = (u8)(x + parts[i].dx);
        kq_sprite_y[id] = (u8)(y + parts[i].dy);
        i = (u8)(i + 1);
    }
    if ((u8)(first_id + count) > kq_sprite_used) kq_sprite_used = (u8)(first_id + count);
    return count;
}

u8 anim_update(SpriteAnim* anim)
{
    if (anim == 0) return 0;
    if (anim->frame_count == 0) return anim->first_tile;
    if (anim->ticks_per_frame == 0) return (u8)(anim->first_tile + anim->frame);

    anim->ticks = (u8)(anim->ticks + 1);
    if (anim->ticks >= anim->ticks_per_frame) {
        anim->ticks = 0;
        anim->frame = (u8)(anim->frame + 1);
        if (anim->frame >= anim->frame_count) anim->frame = 0;
    }
    return (u8)(anim->first_tile + anim->frame);
}
