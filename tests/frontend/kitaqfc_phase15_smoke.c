/* phase15 smoke: generated pipeline assets + attr patch helper */

extern __prg_rom const unsigned char demo_png_palette32[32];
extern __prg_rom const unsigned char demo_png_title_nt[960];
extern __prg_rom const unsigned char demo_png_title_at[64];
extern __prg_rom const unsigned char demo_png_player_ms_0[17];
extern __prg_rom const unsigned char demo_png_player_ms_1[17];

extern void nes_vblank_wait(void);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64);
extern unsigned char nes_attr_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height);
extern void nes_attr_shadow_fill_rect(unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pal_index);
extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);

void main(void)
{
    nes_vblank_wait();
    nes_ppu_stream_write(0x3F00, demo_png_palette32, 32);
    nes_nametable_apply_now(0x2000, demo_png_title_nt, demo_png_title_at);

    nes_attr_shadow_fill_rect(8, 8, 8, 4, 3);
    nes_attr_queue_rect(0x2000, 8, 8, 8, 4);
    nes_vram_queue_try_write(0x23C0, demo_png_title_at, 8);

    while (1)
    {
        nes_vblank_wait();
    }
}
