/*
 * Optional declarations for the phase 11 tilemap helper.
 */

// The map borrows row-major tile bytes; keep width*height bytes readable in the active CPU mapping.
// solid_from is a tile-ID threshold, not an independent collision bitmap.
struct NesTilemap {
    unsigned char* tiles;
    unsigned char width;
    unsigned char height;
    unsigned char solid_from;
};

// Compute a row-major byte offset by adding the row stride ty times. This
// internal arithmetic helper does not validate coordinates.
unsigned short nes_tilemap_index(struct NesTilemap* map, unsigned char tx, unsigned char ty);
// Return a tile byte, or 0xFF when coordinates exceed map dimensions. The
// map/storage pointers must be valid; 0xFF may also be an actual stored tile.
unsigned char nes_tilemap_get(struct NesTilemap* map, unsigned char tx, unsigned char ty);
// Treat out-of-bounds/0xFF tiles as solid, then compare other tile IDs with
// the map's solid_from threshold.
unsigned char nes_tilemap_point_solid(struct NesTilemap* map, unsigned char tx, unsigned char ty);
// Convert pixels to eight-pixel tile X and narrow to a byte. Keep world X
// below 2048 when wraparound is not intended.
unsigned char nes_world_to_tile8_x(unsigned short world_x);
// Convert pixels to eight-pixel tile Y with the same byte-coordinate limit.
unsigned char nes_world_to_tile8_y(unsigned short world_y);
// Convert a world point to tile coordinates and apply the tile solidity rule.
unsigned char nes_tilemap_world_point_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y);
// Test only the four corners of a nonempty world box. Interior/edge tiles
// between the corners are not scanned, so large boxes can miss obstacles.
// Width/height must be nonzero and endpoint arithmetic must not overflow.
unsigned char nes_tilemap_world_box_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y, unsigned char width, unsigned char height);
