/*
 * KITAQFC phase 10 NES runtime helpers.
 *
 * Intent:
 * - stay within the currently supported C subset
 * - provide PPU stream write, vblank/NMI wait, OAM DMA, and VRAM queue helpers
 * - provide a deferred VRAM update queue flushed from NMI
 * - never define public intrinsic fallbacks such as __memcpy/__memset
 * - work with NROM-256 + CHR-ROM assumptions
 */

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;
__location(0x2002) unsigned char PPUSTATUS;
__location(0x2003) unsigned char OAMADDR;
__location(0x2004) unsigned char OAMDATA;
__location(0x2005) unsigned char PPUSCROLL;
__location(0x2006) unsigned char PPUADDR;
__location(0x2007) unsigned char PPUDATA;
__location(0x4014) unsigned char OAMDMA;

__location(0x0200) unsigned char nes_oam_shadow[256];
__location(0x0300) unsigned char nes_vram_queue_data[192];

unsigned char nes_nmi_counter;
unsigned char nes_vram_queue_used;
unsigned char nes_vram_queue_overflow;

void nes_vram_queue_clear(void)
{
    nes_vram_queue_used = 0;
    nes_vram_queue_overflow = 0;
}

unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len)
{
    unsigned char used;
    unsigned char need;
    unsigned char i;
    unsigned char dst_index;

    used = nes_vram_queue_used;
    need = (unsigned char)(len + 3);

    if ((unsigned char)(used + need) < used)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }
    if ((unsigned char)(used + need) > 192)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }

    nes_vram_queue_data[used] = (unsigned char)(ppu_addr >> 8);
    nes_vram_queue_data[(unsigned char)(used + 1)] = (unsigned char)ppu_addr;
    nes_vram_queue_data[(unsigned char)(used + 2)] = len;

    i = 0;
    dst_index = (unsigned char)(used + 3);
    while (i != len)
    {
        nes_vram_queue_data[dst_index] = src[i];
        dst_index = dst_index + 1;
        i = i + 1;
    }

    nes_vram_queue_used = (unsigned char)(used + need);
    return 1;
}

unsigned char nes_vram_queue_try_fill(unsigned short ppu_addr, unsigned char value, unsigned char len)
{
    unsigned char used;
    unsigned char need;

    used = nes_vram_queue_used;
    need = 4;

    if ((unsigned char)(used + need) < used)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }
    if ((unsigned char)(used + need) > 192)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }
    if ((len & 0x80) != 0)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }

    nes_vram_queue_data[used] = (unsigned char)(ppu_addr >> 8);
    nes_vram_queue_data[(unsigned char)(used + 1)] = (unsigned char)ppu_addr;
    nes_vram_queue_data[(unsigned char)(used + 2)] = (unsigned char)(len | 0x80);
    nes_vram_queue_data[(unsigned char)(used + 3)] = value;
    nes_vram_queue_used = (unsigned char)(used + need);
    return 1;
}

void nes_vram_queue_nmi_flush(void)
{
    unsigned char i;
    unsigned char cmd;
    unsigned char len;
    unsigned char data_index;
    unsigned char latch;

    i = 0;
    while (i != nes_vram_queue_used)
    {
        latch = PPUSTATUS;
        PPUADDR = nes_vram_queue_data[i];
        PPUADDR = nes_vram_queue_data[(unsigned char)(i + 1)];
        cmd = nes_vram_queue_data[(unsigned char)(i + 2)];

        if ((cmd & 0x80) != 0)
        {
            len = (unsigned char)(cmd & 0x7F);
            data_index = nes_vram_queue_data[(unsigned char)(i + 3)];
            while (len != 0)
            {
                PPUDATA = data_index;
                len = len - 1;
            }
            i = (unsigned char)(i + 4);
        }
        else
        {
            len = cmd;
            data_index = (unsigned char)(i + 3);
            while (len != 0)
            {
                PPUDATA = nes_vram_queue_data[data_index];
                data_index = data_index + 1;
                len = len - 1;
            }
            i = data_index;
        }

        latch = latch;
    }

    nes_vram_queue_used = 0;
}

void __nes_nmi(void)
{
    if (nes_vram_queue_used != 0)
    {
        nes_vram_queue_nmi_flush();
    }
    nes_nmi_counter = nes_nmi_counter + 1;
}

void nes_wait_nmi(void)
{
    unsigned char start;
    start = nes_nmi_counter;
    while (nes_nmi_counter == start)
    {
    }
}

void nes_vblank_wait(void)
{
    while ((PPUSTATUS & 0x80) != 0)
    {
    }
    while ((PPUSTATUS & 0x80) == 0)
    {
    }
}

void nes_ppu_seek(unsigned short ppu_addr)
{
    unsigned char latch;
    latch = PPUSTATUS;
    PPUADDR = (unsigned char)(ppu_addr >> 8);
    PPUADDR = (unsigned char)ppu_addr;
    latch = latch;
}

void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len)
{
    nes_ppu_seek(ppu_addr);
    while (len != 0)
    {
        PPUDATA = *src;
        src = src + 1;
        len = len - 1;
    }
}

void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len)
{
    nes_ppu_seek(ppu_addr);
    while (len != 0)
    {
        PPUDATA = value;
        len = len - 1;
    }
}


void nes_oam_dma(unsigned char page)
{
    OAMADDR = 0;
    OAMDMA = page;
}
