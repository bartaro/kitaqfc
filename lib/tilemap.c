/*
 * KITAQFC phase 11 tilemap helper.
 */

struct NesTilemap {
    unsigned char* tiles;
    unsigned char width;
    unsigned char height;
    unsigned char solid_from;
};

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

unsigned char nes_world_to_tile8_x(unsigned short world_x)
{
    return (unsigned char)(world_x >> 3);
}

unsigned char nes_world_to_tile8_y(unsigned short world_y)
{
    return (unsigned char)(world_y >> 3);
}

unsigned char nes_tilemap_world_point_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y)
{
    return nes_tilemap_point_solid(map, nes_world_to_tile8_x(world_x), nes_world_to_tile8_y(world_y));
}

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
