/*
 * Optional declarations for the phase 11 actor helper.
 */

// Callers initialize size and metadata and pass non-null actors.
// attr/flags are retained fields; this helper does not interpret or apply them.
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
void nes_actor_set_world(struct NesActor* actor, unsigned short x, unsigned short y);
// Subtract the camera and retain only the low coordinate bytes. This performs
// wrapping projection, not clipping; callers must hide actors outside the viewport.
void nes_actor_update_screen(struct NesActor* actor, unsigned short camera_x, unsigned short camera_y);
// Draw the byte-stream metasprite at cached screen coordinates and return its
// next OAM index. Actor attr/flags are not applied by this wrapper.
unsigned char nes_actor_draw_metasprite(struct NesActor* actor, unsigned char oam_index, unsigned char* metasprite);
// Test overlap in world coordinates with exclusive right/bottom edges. Keep
// position-plus-size sums within u16; touching edges are not collisions.
// Require positive widths and heights; zero extents are not rejected before the edge tests.
unsigned char nes_actor_collide(struct NesActor* a, struct NesActor* b);
