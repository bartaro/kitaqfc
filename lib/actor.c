/*
 * KITAQFC phase 11 actor helper.
 */

extern unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite);

struct NesActor {
    unsigned short world_x;
    unsigned short world_y;
    unsigned char width;
    unsigned char height;
    unsigned char screen_x;
    unsigned char screen_y;
    unsigned char attr;
    unsigned char flags;
};

// Set the actor's 16-bit world position without updating cached screen coordinates.
void nes_actor_set_world(struct NesActor* actor, unsigned short x, unsigned short y)
{
    actor->world_x = x;
    actor->world_y = y;
}

// Subtract the camera and retain only the low coordinate bytes. This performs
// wrapping projection, not clipping; callers must hide actors outside the viewport.
void nes_actor_update_screen(struct NesActor* actor, unsigned short camera_x, unsigned short camera_y)
{
    unsigned short dx;
    unsigned short dy;

    dx = (unsigned short)(actor->world_x - camera_x);
    dy = (unsigned short)(actor->world_y - camera_y);

    actor->screen_x = (unsigned char)dx;
    actor->screen_y = (unsigned char)dy;
}

// Draw the byte-stream metasprite at cached screen coordinates and return its
// next OAM index. Actor attr/flags are not applied by this wrapper.
unsigned char nes_actor_draw_metasprite(struct NesActor* actor, unsigned char oam_index, unsigned char* metasprite)
{
    return nes_metasprite_draw(oam_index, actor->screen_x, actor->screen_y, metasprite);
}

// Test overlap in world coordinates with exclusive right/bottom edges. Keep
// position-plus-size sums within u16; touching edges are not collisions.
// Require positive widths and heights; zero extents are not rejected before the edge tests.
unsigned char nes_actor_collide(struct NesActor* a, struct NesActor* b)
{
    unsigned short a_right;
    unsigned short b_right;
    unsigned short a_bottom;
    unsigned short b_bottom;

    a_right = (unsigned short)(a->world_x + a->width);
    b_right = (unsigned short)(b->world_x + b->width);
    a_bottom = (unsigned short)(a->world_y + a->height);
    b_bottom = (unsigned short)(b->world_y + b->height);

    if (a_right <= b->world_x)
    {
        return 0;
    }
    if (b_right <= a->world_x)
    {
        return 0;
    }
    if (a_bottom <= b->world_y)
    {
        return 0;
    }
    if (b_bottom <= a->world_y)
    {
        return 0;
    }

    return 1;
}
