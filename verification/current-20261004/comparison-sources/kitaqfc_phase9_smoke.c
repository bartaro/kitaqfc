/*
 * KITAQFC phase 9 sample:
 * - controller read + repeat helpers
 * - simple 16-bit world position
 * - camera follow + scroll apply in NMI-paced loop
 * - sprite stays centered while camera tracks world movement
 */

extern unsigned char nes_oam_shadow[256];
extern unsigned char nes_pad1_cur;
extern unsigned char nes_pad1_pressed;
extern void nes_wait_nmi(void);
extern void nes_vblank_wait(void);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len);
extern void __memset(unsigned char* dst, unsigned char value, unsigned short len);
extern void nes_oam_dma(unsigned char page);
extern void nes_pad_poll(void);
extern void nes_pad_repeat_config(unsigned char delay, unsigned char interval);
extern void nes_pad_repeat_step(void);
extern unsigned char nes_pad_repeat(unsigned char mask);
extern unsigned char nes_pad_trigger(unsigned char mask);
extern void nes_scroll_set_base_ctrl(unsigned char ctrl);
extern void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y);
extern void nes_scroll_apply(void);

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

__prg_rom const unsigned char demo_palette[16] = {
    0x0F, 0x11, 0x21, 0x31,
    0x0F, 0x16, 0x26, 0x36,
    0x0F, 0x19, 0x29, 0x39,
    0x0F, 0x0C, 0x1C, 0x2C,
};

void sprite_commit(unsigned char screen_x, unsigned char screen_y)
{
    nes_oam_shadow[0] = screen_y;
    nes_oam_shadow[1] = 1;
    nes_oam_shadow[2] = 0;
    nes_oam_shadow[3] = screen_x;
}

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
    sprite_commit(120, 96);
    nes_oam_dma(0x02);

    nes_scroll_set_base_ctrl(0x80);
    nes_scroll_apply();

    PPUMASK = 0x1E;
}

unsigned char main(void)
{
    unsigned short world_x;
    unsigned short world_y;
    unsigned char screen_x;
    unsigned char screen_y;

    world_x = 120;
    world_y = 96;
    screen_x = 120;
    screen_y = 96;

    ppu_bootstrap();
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
        if (nes_pad_trigger(0x08) != 0)
        {
            world_x = 120;
            world_y = 96;
        }

        nes_camera_follow_center(world_x, world_y, screen_x, screen_y);
        nes_scroll_apply();
        sprite_commit(screen_x, screen_y);
        nes_oam_dma(0x02);
    }

    return 0;
}
