/*
 * KITAQFC phase 11 tilemap helper.
 */

struct NesTilemap {
    unsigned char* tiles;
    unsigned char width;
    unsigned char height;
    unsigned char solid_from;
};

// Compute a row-major byte offset by adding the row stride ty times. This
// internal arithmetic helper does not validate coordinates.
unsigned short nes_tilemap_index(struct NesTilemap* map, unsigned char tx, unsigned char ty)
{
    unsigned short index;
    unsigned char y;

    index = tx;
    y = 0;
    while (y != ty)
    {
        index = (unsigned short)(index + map->width);
        y = y + 1;
    }

    return index;
}

// Return a tile byte, or 0xFF when coordinates exceed map dimensions. The
// map/storage pointers must be valid; 0xFF may also be an actual stored tile.
unsigned char nes_tilemap_get(struct NesTilemap* map, unsigned char tx, unsigned char ty)
{
    if (tx >= map->width)
    {
        return 0xFF;
    }
    if (ty >= map->height)
    {
        return 0xFF;
    }
    return map->tiles[nes_tilemap_index(map, tx, ty)];
}

// Treat out-of-bounds/0xFF tiles as solid, then compare other tile IDs with
// the map's solid_from threshold.
unsigned char nes_tilemap_point_solid(struct NesTilemap* map, unsigned char tx, unsigned char ty)
{
    unsigned char tile;

    tile = nes_tilemap_get(map, tx, ty);
    if (tile == 0xFF)
    {
        return 1;
    }
    if (tile >= map->solid_from)
    {
        return 1;
    }
    return 0;
}

// Convert pixels to eight-pixel tile X and narrow to a byte. Keep world X
// below 2048 when wraparound is not intended.
unsigned char nes_world_to_tile8_x(unsigned short world_x)
{
    return (unsigned char)(world_x >> 3);
}

// Convert pixels to eight-pixel tile Y with the same byte-coordinate limit.
unsigned char nes_world_to_tile8_y(unsigned short world_y)
{
    return (unsigned char)(world_y >> 3);
}

// Convert a world point to tile coordinates and apply the tile solidity rule.
unsigned char nes_tilemap_world_point_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y)
{
    return nes_tilemap_point_solid(map, nes_world_to_tile8_x(world_x), nes_world_to_tile8_y(world_y));
}

// Test only the four corners of a nonempty world box. Interior/edge tiles
// between the corners are not scanned, so large boxes can miss obstacles.
// Width/height must be nonzero and endpoint arithmetic must not overflow.
// Keep both far edges below 2048 as well as within u16; conversion to byte tile coordinates otherwise wraps.
unsigned char nes_tilemap_world_box_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y, unsigned char width, unsigned char height)
{
    unsigned short x2;
    unsigned short y2;

    x2 = (unsigned short)(world_x + width - 1);
    y2 = (unsigned short)(world_y + height - 1);

    if (nes_tilemap_world_point_solid(map, world_x, world_y) != 0)
    {
        return 1;
    }
    if (nes_tilemap_world_point_solid(map, x2, world_y) != 0)
    {
        return 1;
    }
    if (nes_tilemap_world_point_solid(map, world_x, y2) != 0)
    {
        return 1;
    }
    if (nes_tilemap_world_point_solid(map, x2, y2) != 0)
    {
        return 1;
    }

    return 0;
}
