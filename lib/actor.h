/*
 * Optional declarations for the phase 11 actor helper.
 */

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

void nes_actor_set_world(struct NesActor* actor, unsigned short x, unsigned short y);
void nes_actor_update_screen(struct NesActor* actor, unsigned short camera_x, unsigned short camera_y);
unsigned char nes_actor_draw_metasprite(struct NesActor* actor, unsigned char oam_index, unsigned char* metasprite);
unsigned char nes_actor_collide(struct NesActor* a, struct NesActor* b);
