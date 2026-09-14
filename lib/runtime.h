/*
 * Core NES runtime declarations.
 *
 * Note: KITAQFC's preprocessor/include story is still evolving.
 * This header is primarily reference material for repo users.
 */

// The four-byte record follows hardware OAM byte order: Y, tile, attributes, X.
struct NesSprite4 {
    unsigned char y;
    unsigned char tile;
    unsigned char attr;
    unsigned char x;
};

extern unsigned char nes_nmi_counter;
extern unsigned char nes_oam_shadow[256];
// This counter measures the separate 192-byte copied-payload runtime queue, not the compiler intrinsic queue.
extern unsigned char nes_vram_queue_used;
extern unsigned char nes_vram_queue_overflow;

// Wait until the NMI handler advances its counter. NMI must already be enabled
// and able to execute; this polling loop has no timeout.
void nes_wait_nmi(void);
// Wait for the current VBlank to end, then for the next one to begin. Each
// PPUSTATUS read has hardware side effects, including clearing the VBlank flag.
void nes_vblank_wait(void);
// Reset the PPU address latch with a status read, then write the high/low address bytes.
void nes_ppu_seek(unsigned short ppu_addr);
// Write len source bytes directly through PPUDATA. Call only during an appropriate
// PPU access period; the address increment mode comes from the current PPUCTRL.
void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
// Write value repeatedly through PPUDATA, using the current PPU address increment
// mode. This routine does not wait for VBlank or disable rendering.
void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len);
// Reset the OAM destination and DMA one 256-byte CPU page. The hardware stalls
// the CPU during transfer; page is the source address high byte.
void nes_oam_dma(unsigned char page);

// Discard pending VRAM commands and clear the overflow latch; no PPU writes occur.
void nes_vram_queue_clear(void);
// Copy a literal payload into the queue and publish its new used length last.
// The command format reserves bit 7 of len for fills: callers must pass len <= 127
// and ensure queue production cannot race its NMI consumer.
unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
// Queue a four-byte fill record: address high/low, length with bit 7 set, value.
// Reject lengths above 127 and latch overflow on any capacity/format failure.
unsigned char nes_vram_queue_try_fill(unsigned short ppu_addr, unsigned char value, unsigned char len);
// Consume complete records in order during a safe PPU access period. Reading
// PPUSTATUS resets the shared address latch before each address pair. The caller
// is responsible for ensuring the transfer fits its NMI/VBlank time budget.
void nes_vram_queue_nmi_flush(void);
