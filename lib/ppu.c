__location(0x2006) unsigned char PPUADDR;
__location(0x2007) unsigned char PPUDATA;

// Write a high/low PPU address pair without resetting the shared latch first.
// The caller must prepare latch state and safe PPU timing. This signature
// differs from runtime.c's nes_ppu_seek; do not link both implementations.
void nes_ppu_seek(unsigned char hi, unsigned char lo)
{
    PPUADDR = hi;
    PPUADDR = lo;
}

// Set the address and write count bytes directly through PPUDATA. The current
// PPUCTRL increment mode determines how the destination advances.
// count is 0..255; zero still changes PPU address/latch state. No bank switch or payload copy is performed.
void nes_ppu_write_bytes(unsigned char hi, unsigned char lo, const unsigned char* src, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek(hi, lo);
    for (i = 0; i != count; ++i)
        PPUDATA = src[i];
}

// Set the address and issue count identical PPUDATA writes without waiting for VBlank.
void nes_ppu_fill(unsigned char hi, unsigned char lo, unsigned char value, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek(hi, lo);
    for (i = 0; i != count; ++i)
        PPUDATA = value;
}
