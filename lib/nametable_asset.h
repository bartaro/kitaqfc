/*
 * Optional declarations for phase12 nametable / attribute asset helpers.
 */

extern unsigned char nes_attr_shadow[64];

// Mask the index to the four logical nametables and return its PPU base.
// Physical storage still depends on the cartridge's mirroring configuration.
unsigned short nes_nt_base_from_index(unsigned char index);
// Locate the 64-byte attribute table at offset 0x03C0 from a nametable base.
unsigned short nes_attr_base_from_nt(unsigned short nt_base);
// Fill all 64 shadow bytes with an already-packed attribute value.
void nes_attr_shadow_clear(unsigned char value);
// Copy an entire packed attribute table into shadow RAM without writing the PPU.
void nes_attr_shadow_copy(unsigned char* src64);
// Replace only the chosen quadrant's palette bits, preserving its neighbors.
// Palette IDs are masked to two bits; tile coordinates must fit the shadow table.
void nes_attr_shadow_set_quad(unsigned char tile_x, unsigned char tile_y, unsigned char pal_index);
// Upload all attribute shadow bytes immediately; the caller provides safe PPU timing.
void nes_attr_apply_now(unsigned short nt_base);
// Copy all shadow attribute bytes into the runtime queue and return its admission result.
unsigned char nes_attr_queue_all(unsigned short nt_base);
// Write one complete 32-tile row directly. The caller validates the row and PPU timing.
void nes_nt_stream_row(unsigned short nt_base, unsigned char row, unsigned char* src32);
// Copy one complete nametable row into the runtime queue.
unsigned char nes_nt_queue_row(unsigned short nt_base, unsigned char row, unsigned char* src32);
// Write each rectangle row immediately, advancing source by pitch rather than
// width. Caller-provided coordinates, storage and timing are not clipped or checked.
void nes_nt_stream_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src);
// Copy rows into the runtime queue until complete or full. Failure leaves
// earlier rows queued; it does not roll back a partially admitted rectangle.
// Widths must respect the queue literal-record limit.
// Pitch is source bytes per row. Each copied literal record costs width+3 queue bytes;
// check total admission needs when partial updates are unacceptable.
unsigned char nes_nt_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src);
// Queue one fill record per row, stopping on failure while retaining previously
// queued rows. The runtime fill-length and rectangle bounds requirements apply.
unsigned char nes_nt_queue_fill_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char value);
// Upload 960 tile bytes followed by 64 attribute bytes. This bulk operation
// does not disable rendering or wait for VBlank on its own.
void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64);
