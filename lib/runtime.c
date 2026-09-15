/*
 * KITAQFC NES runtime helpers.
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

// Reserve CPU page 02 for OAM shadow and 0300..03BF for this runtime queue; avoid overlapping other allocations.
__location(0x0200) unsigned char nes_oam_shadow[256];
__location(0x0300) unsigned char nes_vram_queue_data[192];

unsigned char nes_nmi_counter;
unsigned char nes_vram_queue_used;
unsigned char nes_vram_queue_overflow;

// Discard pending VRAM commands and clear the overflow latch; no PPU writes occur.
void nes_vram_queue_clear(void)
{
    nes_vram_queue_used = 0;
    nes_vram_queue_overflow = 0;
}

// Copy a literal payload into the queue and publish its new used length last.
// Accept lengths 0..127; reject larger lengths and latch overflow without changing the queue.
// The caller must ensure queue production cannot race its NMI consumer.
unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len)
{
    unsigned char used;
    unsigned char need;
    unsigned char i;
    unsigned char dst_index;

    // Bit 7 selects a fill record; accepting it here would misdecode a literal
    // and lengths 253..255 would also wrap the byte-sized allocation cost.
    if ((len & 0x80) != 0)
    {
        nes_vram_queue_overflow = 1;
        return 0;
    }

    used = nes_vram_queue_used;
    // Each literal record costs three header bytes plus len payload bytes.
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

// Queue a four-byte fill record: address high/low, length with bit 7 set, value.
// Reject lengths above 127 and latch overflow on any capacity/format failure.
// A zero-length fill still occupies its four-byte record and performs no PPUDATA writes when consumed.
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

// Consume complete records in order during a safe PPU access period. Reading
// PPUSTATUS resets the shared address latch before each address pair. The caller
// is responsible for ensuring the transfer fits its NMI/VBlank time budget.
// Consume only well-formed records produced by these APIs, with no concurrent producer.
// The PPUCTRL increment mode remains active; use increment one for ordinary contiguous rows.
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

// Flush queued VRAM updates before advancing the 8-bit NMI completion counter.
void __nes_nmi(void)
{
    if (nes_vram_queue_used != 0)
    {
        nes_vram_queue_nmi_flush();
    }
    nes_nmi_counter = nes_nmi_counter + 1;
}

// Wait until the NMI handler advances its counter. NMI must already be enabled
// and able to execute; this polling loop has no timeout.
void nes_wait_nmi(void)
{
    unsigned char start;
    start = nes_nmi_counter;
    while (nes_nmi_counter == start)
    {
    }
}

// Wait for the current VBlank to end, then for the next one to begin. Each
// PPUSTATUS read has hardware side effects, including clearing the VBlank flag.
void nes_vblank_wait(void)
{
    while ((PPUSTATUS & 0x80) != 0)
    {
    }
    while ((PPUSTATUS & 0x80) == 0)
    {
    }
}

// Reset the PPU address latch with a status read, then write the high/low address bytes.
void nes_ppu_seek(unsigned short ppu_addr)
{
    unsigned char latch;
    latch = PPUSTATUS;
    PPUADDR = (unsigned char)(ppu_addr >> 8);
    PPUADDR = (unsigned char)ppu_addr;
    latch = latch;
}

// Write len source bytes directly through PPUDATA. Call only during an appropriate
// PPU access period; the address increment mode comes from the current PPUCTRL.
// Even a zero-length operation seeks the PPU address and changes the shared latch state.
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

// Write value repeatedly through PPUDATA, using the current PPU address increment
// mode. This routine does not wait for VBlank or disable rendering.
void nes_ppu_stream_fill(unsigned short ppu_addr, unsigned char value, unsigned short len)
{
    nes_ppu_seek(ppu_addr);
    while (len != 0)
    {
        PPUDATA = value;
        len = len - 1;
    }
}


// Reset the OAM destination and DMA one 256-byte CPU page. The hardware stalls
// the CPU during transfer; page is the source address high byte.
void nes_oam_dma(unsigned char page)
{
    OAMADDR = 0;
    OAMDMA = page;
}
