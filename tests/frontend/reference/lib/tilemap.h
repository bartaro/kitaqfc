/*
 * Optional declarations for the phase 11 tilemap helper.
 */

struct NesTilemap {
    unsigned char* tiles;
    unsigned char width;
    unsigned char height;
    unsigned char solid_from;
};

unsigned short nes_tilemap_index(struct NesTilemap* map, unsigned char tx, unsigned char ty);
unsigned char nes_tilemap_get(struct NesTilemap* map, unsigned char tx, unsigned char ty);
unsigned char nes_tilemap_point_solid(struct NesTilemap* map, unsigned char tx, unsigned char ty);
unsigned char nes_world_to_tile8_x(unsigned short world_x);
unsigned char nes_world_to_tile8_y(unsigned short world_y);
unsigned char nes_tilemap_world_point_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y);
unsigned char nes_tilemap_world_box_solid(struct NesTilemap* map, unsigned short world_x, unsigned short world_y, unsigned char width, unsigned char height);
