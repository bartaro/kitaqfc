/* phase14 smoke: generated PNG assets の使用イメージ */

extern __prg_rom const unsigned char demo_png_palette32[32];
extern __prg_rom const unsigned char demo_png_title_nt[960];
extern __prg_rom const unsigned char demo_png_title_at[64];

extern void nes_vblank_wait(void);
extern void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64);
extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);

void main(void)
{
    nes_vblank_wait();
    nes_ppu_stream_write(0x3F00, demo_png_palette32, 32);
    nes_nametable_apply_now(0x2000, demo_png_title_nt, demo_png_title_at);
    while (1)
    {
        nes_vblank_wait();
    }
}
