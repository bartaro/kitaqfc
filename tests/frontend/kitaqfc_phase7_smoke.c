struct Sprite4 {
    unsigned char y;
    unsigned char tile;
    unsigned char attr;
    unsigned char x;
};

extern unsigned char nes_oam_shadow[256];
extern void nes_vblank_wait(void);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern void __memset(unsigned char* dst, unsigned char value, unsigned short len);
extern void nes_oam_dma(unsigned char page);

__prg_rom const unsigned char demo_palette[4] = {
    0x0F, 0x11, 0x21, 0x31,
};

__prg_rom const unsigned char demo_row[8] = {
    1, 2, 3, 4, 5, 6, 7, 8,
};

unsigned char main()
{
    struct Sprite4 s;
    unsigned short sprite_size;
    unsigned short sprite_x_off;

    sprite_size = sizeof(struct Sprite4);
    sprite_x_off = offsetof(struct Sprite4, x);

    s.y = 32;
    s.tile = 1;
    s.attr = 0;
    s.x = 48;

    __memset(nes_oam_shadow, 0, sizeof(nes_oam_shadow));
    nes_oam_shadow[0] = s.y;
    nes_oam_shadow[1] = s.tile;
    nes_oam_shadow[2] = s.attr;
    nes_oam_shadow[3] = s.x;

    nes_vblank_wait();
    nes_ppu_stream_write(0x3F00, demo_palette, sizeof(demo_palette));
    nes_ppu_stream_write(0x2080, demo_row, sizeof(demo_row));
    nes_oam_dma(0x02);

    return (unsigned char)(sprite_size + sprite_x_off);
}
