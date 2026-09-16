/*
 * KITAQFC phase15 attribute patch helpers.
 */

// Copy 64 already-packed attribute bytes into shadow storage. This does not
// convert one palette ID per tile into NES attribute bitfields.
// The source must expose all 64 packed bytes and stay readable for the duration of this copy.
void nes_attr_shadow_build_from_palette_map(unsigned char* src64);
// Apply a palette to every two-by-two tile quadrant touched by the rectangle.
// Empty or out-of-range rectangles leave the shadow unchanged.
void nes_attr_shadow_fill_rect(unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pal_index);
// Queue the attribute bytes covering a tile rectangle within 32x30 tiles.
// Empty rectangles succeed; invalid bounds fail without queuing.
// Capacity failure may leave earlier rows queued.
unsigned char nes_attr_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height);
