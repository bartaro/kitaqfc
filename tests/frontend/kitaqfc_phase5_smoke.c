__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;
__location(0x2002) unsigned char PPUSTATUS;

unsigned char read_status_direct(void)
{
    return PPUSTATUS;
}

unsigned char read_status_ptr(void)
{
    unsigned char* p;
    p = (unsigned char*)0x2000;
    return p[2];
}

void write_ctrl_mask(unsigned char ctrl, unsigned char mask)
{
    unsigned char* p;
    p = (unsigned char*)0x2000;
    p[0] = ctrl;
    p[1] = mask;
}

unsigned short read_word(unsigned char* p)
{
    unsigned short w;
    w = *(unsigned short*)p;
    return w;
}

void write_word(unsigned char* p, unsigned short w)
{
    *(unsigned short*)p = w;
}

unsigned char read_index(unsigned char* p, unsigned char i)
{
    return p[i];
}

void write_index(unsigned char* p, unsigned char i, unsigned char v)
{
    p[i] = v;
}

unsigned short main(void)
{
    unsigned short w;
    write_ctrl_mask(0x80, 0x1E);
    if ((read_status_direct() & 0x80) != 0)
        PPUCTRL = 0x00;
    w = read_word((unsigned char*)0x0000);
    write_word((unsigned char*)0x0002, w);
    return w;
}
