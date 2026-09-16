/*
 * KITAQFC phase15 attribute patch helpers.
 *
 * Operates on 16x16 quadrants expressed in tile coordinates.
 */

extern unsigned char nes_attr_shadow[64];
extern unsigned short nes_attr_base_from_nt(unsigned short nt_base);
extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
extern void nes_attr_shadow_set_quad(unsigned char tile_x, unsigned char tile_y, unsigned char pal_index);

// Convert a tile X coordinate to a two-tile quadrant X coordinate.
static unsigned char nes_attr_rect_left(unsigned char tile_x)
{
    return (unsigned char)(tile_x >> 1);
}

// Convert a tile Y coordinate to a two-tile quadrant Y coordinate.
static unsigned char nes_attr_rect_top(unsigned char tile_y)
{
    return (unsigned char)(tile_y >> 1);
}

// Copy 64 already-packed attribute bytes into shadow storage. This does not
// convert one palette ID per tile into NES attribute bitfields.
void nes_attr_shadow_build_from_palette_map(unsigned char* src64)
{
    unsigned char i;
    i = 0;
    while (i != 64)
    {
        nes_attr_shadow[i] = (unsigned char)(src64[i] & 0xFF);
        i = (unsigned char)(i + 1);
    }
}

// Apply a palette to every two-by-two tile quadrant touched by the rectangle.
// Empty or out-of-range rectangles leave the shadow unchanged.
void nes_attr_shadow_fill_rect(unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pal_index)
{
    unsigned char qx0;
    unsigned char qy0;
    unsigned char qx1;
    unsigned char qy1;
    unsigned char qx;
    unsigned char qy;

    if (width == 0 || height == 0) return;
    if (tile_x >= 32 || tile_y >= 30) return;
    if (width > (unsigned char)(32 - tile_x) || height > (unsigned char)(30 - tile_y)) return;

    qx0 = nes_attr_rect_left(tile_x);
    qy0 = nes_attr_rect_top(tile_y);
    qx1 = nes_attr_rect_left((unsigned char)(tile_x + width - 1));
    qy1 = nes_attr_rect_top((unsigned char)(tile_y + height - 1));

    qy = qy0;
    while (1)
    {
        qx = qx0;
        while (1)
        {
            nes_attr_shadow_set_quad((unsigned char)(qx << 1), (unsigned char)(qy << 1), pal_index);
            if (qx == qx1)
            {
                break;
            }
            qx = (unsigned char)(qx + 1);
        }
        if (qy == qy1)
        {
            break;
        }
        qy = (unsigned char)(qy + 1);
    }
}

// Queue every four-by-four-tile attribute byte touched by the rectangle.
// Empty rectangles succeed; out-of-range rectangles fail before queuing.
// A capacity failure can leave earlier rows queued.
unsigned char nes_attr_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height)
{
    unsigned char qx0;
    unsigned char qy0;
    unsigned char qx1;
    unsigned char qy1;
    unsigned char row;
    unsigned char col;
    // The runtime queue copies these local bytes before return; they are not retained pointers.
    // Caller coordinates must keep each computed row length within eight bytes.
    unsigned char line[8];
    unsigned short ppu_addr;
    unsigned char len;

    if (width == 0 || height == 0) return 1;
    if (tile_x >= 32 || tile_y >= 30) return 0;
    if (width > (unsigned char)(32 - tile_x) || height > (unsigned char)(30 - tile_y)) return 0;

    qx0 = (unsigned char)(tile_x >> 2);
    qy0 = (unsigned char)(tile_y >> 2);
    qx1 = (unsigned char)((tile_x + width - 1) >> 2);
    qy1 = (unsigned char)((tile_y + height - 1) >> 2);

    row = qy0;
    while (1)
    {
        len = (unsigned char)(qx1 - qx0 + 1);
        col = 0;
        while (col != len)
        {
            line[col] = nes_attr_shadow[(unsigned char)(row * 8 + qx0 + col)];
            col = (unsigned char)(col + 1);
        }
        ppu_addr = (unsigned short)(nes_attr_base_from_nt(nt_base) + (unsigned short)(row * 8 + qx0));
        if (nes_vram_queue_try_write(ppu_addr, line, len) == 0)
        {
            return 0;
        }
        if (row == qy1)
        {
            break;
        }
        row = (unsigned char)(row + 1);
    }
    return 1;
}
