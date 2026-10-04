/*
 * KITAQFC phase 8 sample:
 * - NMI-based frame loop
 * - controller 1 polling
 * - single sprite movement via OAM shadow + OAM DMA
 */

struct Sprite4 {
    unsigned char y;
    unsigned char tile;
    unsigned char attr;
    unsigned char x;
};

extern unsigned char nes_oam_shadow[256];
extern unsigned char nes_pad1_cur;
extern unsigned char nes_pad1_pressed;
extern void nes_wait_nmi(void);
extern void nes_vblank_wait(void);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern void __memset(unsigned char* dst, unsigned char value, unsigned short len);
extern void nes_oam_dma(unsigned char page);
extern void nes_pad_poll(void);

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

__prg_rom const unsigned char demo_palette[8] = {
    0x0F, 0x20, 0x16, 0x30,
    0x0F, 0x00, 0x10, 0x30,
};

void sprite_commit(unsigned char x, unsigned char y)
{
    nes_oam_shadow[0] = y;
    nes_oam_shadow[1] = 1;
    nes_oam_shadow[2] = 0;
    nes_oam_shadow[3] = x;
}

void ppu_bootstrap(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;

    nes_vblank_wait();
    nes_ppu_stream_write(0x3F00, demo_palette, sizeof(demo_palette));

    __memset(nes_oam_shadow, 0xFF, sizeof(nes_oam_shadow));
    sprite_commit(120, 96);
    nes_oam_dma(0x02);

    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
}

unsigned char main(void)
{
    unsigned char x;
    unsigned char y;

    x = 120;
    y = 96;

    ppu_bootstrap();

    while (1)
    {
        nes_wait_nmi();
        nes_pad_poll();

        if ((nes_pad1_cur & 0x80) != 0)
        {
            x = x + 1;
        }
        if ((nes_pad1_cur & 0x40) != 0)
        {
            x = x - 1;
        }
        if ((nes_pad1_cur & 0x20) != 0)
        {
            y = y + 1;
        }
        if ((nes_pad1_cur & 0x10) != 0)
        {
            y = y - 1;
        }
        if ((nes_pad1_pressed & 0x08) != 0)
        {
            x = 120;
            y = 96;
        }

        sprite_commit(x, y);
        nes_oam_dma(0x02);
    }

    return 0;
}
