/*
 * KITAQFC phase 11 collision helper.
 */

struct NesBox16 {
    unsigned short x;
    unsigned short y;
    unsigned char w;
    unsigned char h;
};

unsigned char nes_box16_intersects(struct NesBox16* a, struct NesBox16* b)
{
    unsigned short a_right;
    unsigned short b_right;
    unsigned short a_bottom;
    unsigned short b_bottom;

    a_right = (unsigned short)(a->x + a->w);
    b_right = (unsigned short)(b->x + b->w);
    a_bottom = (unsigned short)(a->y + a->h);
    b_bottom = (unsigned short)(b->y + b->h);

    if (a_right <= b->x)
    {
        return 0;
    }
    if (b_right <= a->x)
    {
        return 0;
    }
    if (a_bottom <= b->y)
    {
        return 0;
    }
    if (b_bottom <= a->y)
    {
        return 0;
    }

    return 1;
}

unsigned char nes_box16_contains_point(struct NesBox16* box, unsigned short px, unsigned short py)
{
    unsigned short right;
    unsigned short bottom;

    right = (unsigned short)(box->x + box->w);
    bottom = (unsigned short)(box->y + box->h);

    if (px < box->x)
    {
        return 0;
    }
    if (py < box->y)
    {
        return 0;
    }
    if (px >= right)
    {
        return 0;
    }
    if (py >= bottom)
    {
        return 0;
    }

    return 1;
}
