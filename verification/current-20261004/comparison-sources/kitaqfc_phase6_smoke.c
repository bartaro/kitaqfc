struct Sprite4 {
    unsigned char y;
    unsigned char tile;
    unsigned char attr;
    unsigned char x;
};

__location(0x2006) unsigned char PPUADDR;
__location(0x2007) unsigned char PPUDATA;

unsigned char oam_shadow[256];

__prg_rom const unsigned char demo_palette[4] = {
    0x0F, 0x11, 0x21, 0x31,
};

__prg_rom const unsigned char demo_nametable_row[8] = {
    1, 2, 3, 4, 5, 6, 7, 8,
};

void ppu_write_bytes(unsigned char hi, unsigned char lo, const unsigned char* src, unsigned char count)
{
    unsigned char i;
    PPUADDR = hi;
    PPUADDR = lo;
    for (i = 0; i != count; ++i)
        PPUDATA = src[i];
}

unsigned char main()
{
    struct Sprite4 s;

    s.y = 32;
    s.tile = 1;
    s.attr = 0;
    s.x = 48;

    oam_shadow[0] = s.y;
    oam_shadow[1] = s.tile;
    oam_shadow[2] = s.attr;
    oam_shadow[3] = s.x;

    ppu_write_bytes(0x3F, 0x00, demo_palette, 4);
    ppu_write_bytes(0x20, 0x80, demo_nametable_row, 8);

    return oam_shadow[3];
}
