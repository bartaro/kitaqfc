/*
 * KITAQFC phase15 attribute patch helpers.
 */

void nes_attr_shadow_build_from_palette_map(unsigned char* src64);
void nes_attr_shadow_fill_rect(unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pal_index);
unsigned char nes_attr_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height);
