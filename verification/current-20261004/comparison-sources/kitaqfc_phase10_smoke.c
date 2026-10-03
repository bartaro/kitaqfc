/*
 * KITAQFC phase 10 sample:
 * - pad repeat + trigger
 * - square-1 APU helper
 * - metasprite helper
 * - deferred VRAM update queue flushed from NMI
 */

extern unsigned char nes_oam_shadow[256];
extern void nes_wait_nmi(void);
extern void nes_vblank_wait(void);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len);
extern void __memset(unsigned char* dst, unsigned char value, unsigned short len);
extern void nes_oam_dma(unsigned char page);
extern void nes_vram_queue_clear(void);
extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
extern unsigned char nes_vram_queue_try_fill(unsigned short ppu_addr, unsigned char value, unsigned char len);
extern void nes_pad_poll(void);
extern void nes_pad_repeat_config(unsigned char delay, unsigned char interval);
extern void nes_pad_repeat_step(void);
extern unsigned char nes_pad_repeat(unsigned char mask);
extern unsigned char nes_pad_trigger(unsigned char mask);
extern void nes_scroll_set_base_ctrl(unsigned char ctrl);
extern void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y);
extern void nes_scroll_apply(void);
extern void nes_apu_init(void);
extern void nes_sfx_tick_blip(void);
extern void nes_sfx_move_blip(void);
extern unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite);
extern void nes_metasprite_hide_from(unsigned char oam_index, unsigned char sprite_count);

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

__prg_rom const unsigned char demo_palette[16] = {
    0x0F, 0x12, 0x22, 0x32,
    0x0F, 0x16, 0x26, 0x36,
    0x0F, 0x19, 0x29, 0x39,
    0x0F, 0x0C, 0x1C, 0x2C,
};

__prg_rom const unsigned char marker_a[4] = { 0x05, 0x06, 0x07, 0x08 };
__prg_rom const unsigned char marker_b[4] = { 0x09, 0x0A, 0x0B, 0x0C };

__prg_rom const unsigned char player_meta[17] = {
    0, 0, 0x10, 0,
    8, 0, 0x11, 0,
    0, 8, 0x12, 0,
    8, 8, 0x13, 0,
    0xFF
};

void draw_demo_bg(void)
{
    nes_ppu_stream_fill(0x2000, 0x01, 0x03C0);
    nes_ppu_stream_fill(0x2400, 0x02, 0x03C0);
    nes_ppu_stream_fill(0x2800, 0x03, 0x03C0);
    nes_ppu_stream_fill(0x2C00, 0x04, 0x03C0);
}

void ppu_bootstrap(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;

    nes_vblank_wait();
    nes_ppu_stream_write(0x3F00, demo_palette, sizeof(demo_palette));
    draw_demo_bg();

    __memset(nes_oam_shadow, 0xFF, sizeof(nes_oam_shadow));
    nes_metasprite_draw(0, 120, 96, player_meta);
    nes_oam_dma(0x02);

    nes_scroll_set_base_ctrl(0x80);
    nes_scroll_apply();
    nes_vram_queue_clear();
    PPUMASK = 0x1E;
}

unsigned char main(void)
{
    unsigned short world_x;
    unsigned short world_y;
    unsigned char screen_x;
    unsigned char screen_y;
    unsigned short stamp_addr;

    world_x = 120;
    world_y = 96;
    screen_x = 120;
    screen_y = 96;
    stamp_addr = 0x21C0;

    ppu_bootstrap();
    nes_apu_init();
    nes_pad_repeat_config(12, 3);

    while (1)
    {
        nes_wait_nmi();
        nes_pad_poll();
        nes_pad_repeat_step();

        if (nes_pad_repeat(0x80) != 0)
        {
            world_x = world_x + 2;
        }
        if (nes_pad_repeat(0x40) != 0)
        {
            world_x = world_x - 2;
        }
        if (nes_pad_repeat(0x20) != 0)
        {
            world_y = world_y + 2;
        }
        if (nes_pad_repeat(0x10) != 0)
        {
            world_y = world_y - 2;
        }

        if ((nes_pad_repeat(0xF0)) != 0)
        {
            nes_sfx_move_blip();
        }

        if (nes_pad_trigger(0x01) != 0)
        {
            nes_vram_queue_try_write(stamp_addr, marker_a, 4);
            stamp_addr = stamp_addr + 4;
            nes_sfx_tick_blip();
        }
        if (nes_pad_trigger(0x02) != 0)
        {
            nes_vram_queue_try_fill(0x23C0, 0xAA, 8);
            nes_sfx_tick_blip();
        }
        if (nes_pad_trigger(0x08) != 0)
        {
            world_x = 120;
            world_y = 96;
            stamp_addr = 0x21C0;
            nes_vram_queue_try_write(0x21C0, marker_b, 4);
            nes_sfx_tick_blip();
        }

        nes_camera_follow_center(world_x, world_y, screen_x, screen_y);
        nes_scroll_apply();
        nes_metasprite_draw(0, screen_x, screen_y, player_meta);
        nes_metasprite_hide_from(4, 60);
        nes_oam_dma(0x02);
    }

    return 0;
}
