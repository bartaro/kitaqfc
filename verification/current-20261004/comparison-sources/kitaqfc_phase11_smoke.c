/*
 * KITAQFC phase 11 sample:
 * - palette helper
 * - actor helper
 * - tilemap / collision helper
 * - metasprite + camera loop
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

struct NesTilemap {
    unsigned char* tiles;
    unsigned char width;
    unsigned char height;
    unsigned char solid_from;
};

extern unsigned char nes_oam_shadow[256];
extern void __memset(unsigned char* dst, unsigned char value, unsigned short len);
extern void nes_wait_nmi(void);
extern void nes_vblank_wait(void);
extern void nes_oam_dma(unsigned char page);
extern void nes_pad_poll(void);
extern void nes_pad_repeat_config(unsigned char delay, unsigned char interval);
extern void nes_pad_repeat_step(void);
extern unsigned char nes_pad_repeat(unsigned char mask);
extern void nes_scroll_set_base_ctrl(unsigned char ctrl);
extern void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y);
extern void nes_scroll_apply(void);
extern void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len);
extern void nes_palette_apply_now(unsigned char* src32);
extern void nes_actor_set_world(struct NesActor* actor, unsigned short x, unsigned short y);
extern void nes_actor_update_screen(struct NesActor* actor, unsigned short camera_x, unsigned short camera_y);
extern unsigned char nes_actor_draw_metasprite(struct NesActor* actor, unsigned char oam_index, unsigned char* metasprite);
extern unsigned char nes_tilemap_world_box_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y, unsigned char width, unsigned char height);

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

__prg_rom const unsigned char demo_palette[32] = {
    0x0F, 0x12, 0x22, 0x32,
    0x0F, 0x16, 0x26, 0x36,
    0x0F, 0x19, 0x29, 0x39,
    0x0F, 0x0C, 0x1C, 0x2C,
    0x0F, 0x00, 0x10, 0x20,
    0x0F, 0x06, 0x16, 0x26,
    0x0F, 0x09, 0x19, 0x29,
    0x0F, 0x03, 0x13, 0x23,
};

__prg_rom const unsigned char player_ms[17] = {
    0, 0, 0x20, 0,
    8, 0, 0x21, 0,
    0, 8, 0x22, 0,
    8, 8, 0x23, 0,
    0xFF
};

__prg_rom const unsigned char demo_collision[64] = {
    0x80,0x80,0x80,0x80,0x80,0x80,0x80,0x80,
    0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x80,
    0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x80,
    0x80,0x00,0x80,0x80,0x80,0x00,0x00,0x80,
    0x80,0x00,0x00,0x00,0x80,0x00,0x00,0x80,
    0x80,0x00,0x00,0x00,0x80,0x00,0x00,0x80,
    0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x80,
    0x80,0x80,0x80,0x80,0x80,0x80,0x80,0x80
};

struct NesActor player;
struct NesTilemap world_map;

void ppu_bootstrap(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;
    nes_vblank_wait();
    nes_palette_apply_now(demo_palette);
    nes_ppu_stream_fill(0x2000, 0x01, 0x03C0);
    __memset(nes_oam_shadow, 0xFF, sizeof(nes_oam_shadow));
    nes_scroll_set_base_ctrl(0x80);
    nes_scroll_apply();
    PPUMASK = 0x1E;
}

void try_move_right(void)
{
    unsigned short nx;
    nx = (unsigned short)(player.world_x + 2);
    if (nes_tilemap_world_box_solid(&world_map, nx, player.world_y, player.width, player.height) == 0)
    {
        nes_actor_set_world(&player, nx, player.world_y);
    }
}

void try_move_left(void)
{
    unsigned short nx;
    nx = (unsigned short)(player.world_x - 2);
    if (nes_tilemap_world_box_solid(&world_map, nx, player.world_y, player.width, player.height) == 0)
    {
        nes_actor_set_world(&player, nx, player.world_y);
    }
}

void try_move_down(void)
{
    unsigned short ny;
    ny = (unsigned short)(player.world_y + 2);
    if (nes_tilemap_world_box_solid(&world_map, player.world_x, ny, player.width, player.height) == 0)
    {
        nes_actor_set_world(&player, player.world_x, ny);
    }
}

void try_move_up(void)
{
    unsigned short ny;
    ny = (unsigned short)(player.world_y - 2);
    if (nes_tilemap_world_box_solid(&world_map, player.world_x, ny, player.width, player.height) == 0)
    {
        nes_actor_set_world(&player, player.world_x, ny);
    }
}

unsigned char main(void)
{
    unsigned char screen_cx;
    unsigned char screen_cy;
    unsigned short camera_x;
    unsigned short camera_y;

    world_map.tiles = demo_collision;
    world_map.width = 8;
    world_map.height = 8;
    world_map.solid_from = 0x80;

    player.width = 16;
    player.height = 16;
    player.attr = 0;
    player.flags = 0;
    nes_actor_set_world(&player, 24, 24);

    screen_cx = 120;
    screen_cy = 96;

    ppu_bootstrap();
    nes_pad_repeat_config(12, 3);

    while (1)
    {
        nes_wait_nmi();
        nes_pad_poll();
        nes_pad_repeat_step();

        if (nes_pad_repeat(0x80) != 0)
        {
            try_move_right();
        }
        if (nes_pad_repeat(0x40) != 0)
        {
            try_move_left();
        }
        if (nes_pad_repeat(0x20) != 0)
        {
            try_move_down();
        }
        if (nes_pad_repeat(0x10) != 0)
        {
            try_move_up();
        }

        nes_camera_follow_center(player.world_x, player.world_y, screen_cx, screen_cy);
        nes_scroll_apply();

        camera_x = (unsigned short)(player.world_x - screen_cx);
        camera_y = (unsigned short)(player.world_y - screen_cy);
        nes_actor_update_screen(&player, camera_x, camera_y);
        nes_actor_draw_metasprite(&player, 0, player_ms);
        nes_oam_dma(0x02);
    }

    return 0;
}
