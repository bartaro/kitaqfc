__location(0x2006) unsigned char PPUADDR;
__location(0x2007) unsigned char PPUDATA;

void nes_ppu_seek(unsigned char hi, unsigned char lo)
{
    PPUADDR = hi;
    PPUADDR = lo;
}

void nes_ppu_write_bytes(unsigned char hi, unsigned char lo, const unsigned char* src, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek(hi, lo);
    for (i = 0; i != count; ++i)
        PPUDATA = src[i];
}

void nes_ppu_fill(unsigned char hi, unsigned char lo, unsigned char value, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek(hi, lo);
    for (i = 0; i != count; ++i)
        PPUDATA = value;
}
