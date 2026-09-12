/*
 * Optional declarations for the phase 11 collision helper.
 */

struct NesBox16 {
    unsigned short x;
    unsigned short y;
    unsigned char w;
    unsigned char h;
};

unsigned char nes_box16_intersects(struct NesBox16* a, struct NesBox16* b);
unsigned char nes_box16_contains_point(struct NesBox16* box, unsigned short px, unsigned short py);
