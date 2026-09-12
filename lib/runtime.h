/*
 * Core NES runtime declarations.
 *
 * Note: KITAQFC's preprocessor/include story is still evolving.
 * This header is primarily reference material for repo users.
 */

struct NesSprite4 {
    unsigned char y;
    unsigned char tile;
    unsigned char attr;
    unsigned char x;
};

extern unsigned char nes_nmi_counter;
extern unsigned char nes_oam_shadow[256];
extern unsigned char nes_vram_queue_used;
extern unsigned char nes_vram_queue_overflow;

void nes_wait_nmi(void);
void nes_vblank_wait(void);
void nes_ppu_seek(unsigned short ppu_addr);
void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len);
void nes_oam_dma(unsigned char page);

void nes_vram_queue_clear(void);
unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
unsigned char nes_vram_queue_try_fill(unsigned short ppu_addr, unsigned char value, unsigned char len);
void nes_vram_queue_nmi_flush(void);
