/*
 * Optional declarations for the phase 10 NES metasprite helper.
 *
 * Metasprite data layout:
 *   [dx, dy, tile, attr] repeated, terminated by dx=0xFF
 */

unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite);
unsigned char __metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, const unsigned char* metasprite);
void nes_metasprite_hide_from(unsigned char oam_index, unsigned char sprite_count);
