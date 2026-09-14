/*
 * Optional declarations for the phase 11 collision helper.
 */

struct NesBox16 {
    unsigned short x;
    unsigned short y;
    unsigned char w;
    unsigned char h;
};

// Test two axis-aligned world boxes using exclusive right/bottom edges.
// Coordinate-plus-size sums must fit u16; touching edges return false.
// Both boxes must have positive dimensions; the overlap test has no separate empty-box guard.
unsigned char nes_box16_intersects(struct NesBox16* a, struct NesBox16* b);
// Test a half-open box: left/top edges are included and right/bottom edges
// excluded. Callers provide valid pointers and non-overflowing edge coordinates.
unsigned char nes_box16_contains_point(struct NesBox16* box, unsigned short px, unsigned short py);
