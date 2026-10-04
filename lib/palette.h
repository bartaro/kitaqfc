/*
 * Optional declarations for the phase 11 NES palette helper.
 */

extern unsigned char nes_palette_shadow[32];

void nes_palette_copy(unsigned char* src32);
void nes_palette_apply_now(unsigned char* src32);
void nes_palette_apply_shadow_now(void);
unsigned char nes_palette_queue_all(unsigned char* src32);
unsigned char nes_palette_queue_bg4(unsigned char pal_index, unsigned char* src4);
unsigned char nes_palette_queue_sprite4(unsigned char pal_index, unsigned char* src4);
