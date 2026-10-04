/*
 * phase12 smoke:
 * - generated assets via tools/kitaqfc_asset_pack.py
 * - palette apply
 * - immediate nametable/attr apply
 * - compiled with --nes-chr=assets/generated/demo_chr8k.chr
 */

extern __prg_rom const unsigned char demo_palette32[32];
extern __prg_rom const unsigned char demo_title_nt[960];
extern __prg_rom const unsigned char demo_title_at[64];

extern void nes_vblank_wait(void);
extern void nes_palette_apply_now(unsigned char* src32);
extern void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64);

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

void main(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;
    nes_vblank_wait();
    nes_palette_apply_now(demo_palette32);
    nes_nametable_apply_now(0x2000, demo_title_nt, demo_title_at);
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
    while (1)
    {
    }
}
