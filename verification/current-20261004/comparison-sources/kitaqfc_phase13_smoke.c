/*
 * phase13 smoke:
 * - frame-budgeted nametable streaming
 * - palette + full-screen stream over multiple frames
 * - compiled with --nes-chr=assets/generated/demo_chr8k.chr
 */

extern __prg_rom const unsigned char demo_palette32[32];
extern __prg_rom const unsigned char demo_title_nt[960];
extern __prg_rom const unsigned char demo_title_at[64];
extern __prg_rom const unsigned char demo_player_ms[17];

extern void nes_vblank_wait(void);
extern void nes_wait_nmi(void);
extern void nes_pad_poll(void);
extern void nes_pad_repeat_step(void);
extern void nes_palette_apply_now(unsigned char* src32);
extern void nes_scene_stream_begin_nametable(unsigned short nt_base, unsigned char* src960);
extern void nes_scene_stream_begin_attr(unsigned short nt_base, unsigned char* src64);
extern void nes_scene_stream_begin_palette(unsigned char* src32);
extern unsigned char nes_scene_stream_step(void);
extern unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite);
extern void nes_oam_dma(unsigned char page);
extern unsigned char nes_oam_shadow[256];

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;

unsigned char scene_phase;
unsigned char spr_x;
unsigned char spr_y;

void scene_reset(void)
{
    scene_phase = 0;
    spr_x = 120;
    spr_y = 100;
    nes_scene_stream_begin_palette(demo_palette32);
}

void main(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;
    nes_vblank_wait();
    nes_palette_apply_now(demo_palette32);
    scene_reset();
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;

    while (1)
    {
        nes_wait_nmi();
        nes_pad_poll();
        nes_pad_repeat_step();

        if (nes_scene_stream_step() == 0)
        {
            if (scene_phase == 0)
            {
                scene_phase = 1;
                nes_scene_stream_begin_nametable(0x2000, demo_title_nt);
            }
            else if (scene_phase == 1)
            {
                scene_phase = 2;
                nes_scene_stream_begin_attr(0x2000, demo_title_at);
            }
        }

        nes_metasprite_draw(0, spr_x, spr_y, demo_player_ms);
        nes_oam_dma(0x02);
    }
}
