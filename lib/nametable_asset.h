/*
 * Optional declarations for phase12 nametable / attribute asset helpers.
 */

extern unsigned char nes_attr_shadow[64];

unsigned short nes_nt_base_from_index(unsigned char index);
unsigned short nes_attr_base_from_nt(unsigned short nt_base);
void nes_attr_shadow_clear(unsigned char value);
void nes_attr_shadow_copy(unsigned char* src64);
void nes_attr_shadow_set_quad(unsigned char tile_x, unsigned char tile_y, unsigned char pal_index);
void nes_attr_apply_now(unsigned short nt_base);
unsigned char nes_attr_queue_all(unsigned short nt_base);
void nes_nt_stream_row(unsigned short nt_base, unsigned char row, unsigned char* src32);
unsigned char nes_nt_queue_row(unsigned short nt_base, unsigned char row, unsigned char* src32);
void nes_nt_stream_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src);
unsigned char nes_nt_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src);
unsigned char nes_nt_queue_fill_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char value);
void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64);
