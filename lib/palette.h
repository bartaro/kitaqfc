/*
 * Optional declarations for the phase 11 NES palette helper.
 */

extern unsigned char nes_palette_shadow[32];

// Copy exactly 32 palette bytes to the fixed shadow buffer without writing the PPU.
void nes_palette_copy(unsigned char* src32);
// Replace shadow palette data and upload all 32 bytes immediately. The caller
// provides a safe PPU access period; this helper does not wait for VBlank.
void nes_palette_apply_now(unsigned char* src32);
// Upload the current palette shadow directly, with timing controlled by the caller.
void nes_palette_apply_shadow_now(void);
// Update the shadow and copy all colors into the runtime queue. A queue
// failure leaves the new shadow contents in place.
// Return one if the runtime queue accepted the copied payload, zero if it was full.
// Admission does not mean the NMI consumer has already updated the PPU.
unsigned char nes_palette_queue_all(unsigned char* src32);
// Update and queue four background palette bytes. pal_index must be 0..3;
// there is no range check before writing the shadow.
unsigned char nes_palette_queue_bg4(unsigned char pal_index, unsigned char* src4);
// Update and queue four sprite palette bytes at the 0x3F10 region. The
// caller supplies pal_index 0..3; NES palette mirroring still applies.
unsigned char nes_palette_queue_sprite4(unsigned char pal_index, unsigned char* src4);
