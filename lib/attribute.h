/*
 * KITAQFC phase15 attribute patch helpers.
 */

// Copy 64 already-packed attribute bytes into shadow storage. This does not
// convert one palette ID per tile into NES attribute bitfields.
// The source must expose all 64 packed bytes and stay readable for the duration of this copy.
void nes_attr_shadow_build_from_palette_map(unsigned char* src64);
// Apply a palette to every two-by-two tile quadrant touched by the rectangle.
// Width/height must be nonzero and the rectangle must stay within the nametable;
// there is no clipping before the byte-sized endpoint arithmetic.
void nes_attr_shadow_fill_rect(unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pal_index);
// Queue shadow-byte rows using the half-tile coordinates computed below.
// These indices are used directly as byte indices, unlike the four-tile byte
// indexing in nes_attr_shadow_set_quad; this is not a general tile-to-attribute
// rectangle conversion. Keep each row within the eight-byte local buffer and
// all indices within the 64-byte shadow. Failure may leave earlier rows queued.
unsigned char nes_attr_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height);
