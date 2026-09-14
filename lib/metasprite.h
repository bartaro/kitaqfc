/*
 * Optional declarations for the phase 10 NES metasprite helper.
 *
 * Metasprite data layout:
 *   [dx, dy, tile, attr] repeated, terminated by dx=0xFF
 */

// Read four-byte records [dx,dy,tile,attr] until dx=0xFF, writing them to
// shadow OAM. Coordinates and destination offsets wrap as bytes. The caller
// must supply a terminator and enough unused slots; there is no capacity guard.
// Return the next slot index derived from the final wrapped byte offset.
unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite);
// Compiler intrinsic counterpart; this declaration does not route through the C function above.
unsigned char __metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, const unsigned char* metasprite);
// Hide consecutive shadow slots by setting Y to 0xF8. Keep the requested
// range within 64 slots to avoid wrapping and hiding earlier entries.
void nes_metasprite_hide_from(unsigned char oam_index, unsigned char sprite_count);
