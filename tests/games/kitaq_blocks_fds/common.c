#include "kbfc.h"

__location(0x2000) u8 PPU_CTRL;
__location(0x2001) u8 PPU_MASK;
__location(0x2002) u8 PPU_STATUS;
__location(0x2005) u8 PPU_SCROLL;
__location(0x2006) u8 PPU_ADDRESS;
__location(0x2007) u8 PPU_DATA;
__location(0x2003) u8 OAM_ADDRESS;
__location(0x4014) u8 OAM_DMA;
__location(0x4010) u8 APU_DMC;
__location(0x4015) u8 APU_STATUS;
__location(0x4017) u8 APU_FRAME_CTR;
__location(0x4016) u8 INPUT_0;
__location(0x4017) u8 INPUT_1;

__hram u8 Pad0;
__hram u8 Pad0Prev;
__hram u8 InMainThread;
__hram u8 NmiFrameCounter;
__hram u8 AudioFrameCounter;
__hram u8 NmiPaletteIndex;

u8 PpuCtrl;
u8 PpuMask;
u8 PpuMaskApplied;
u8 PpuEnablePending;
u8 PpuScrollX;
u8 PpuScrollY;
u8 PpuPalettes[0x20];
u8 PpuPalettesNew;
u8 NameAttributes[64];
u16 RandomState;
u8 DecimalTens;
u8 DecimalOnes;
u8 CurrentMode;
u8 PreviousMode;

void ResetSprites()
{
    __oam_clear();
}

void HideAllSprites()
{
    __oam_clear();
}

void BeginFrame()
{
    InMainThread = FALSE;
    while (!InMainThread) { }
    UpdateInput();
    CatchUpAudioIfNeeded();
}

void FlushBlitFrame()
{
    InMainThread = FALSE;
    while (!InMainThread) { }
    CatchUpAudioIfNeeded();
}

void CatchUpAudioIfNeeded()
{
    while (AudioFrameCounter != NmiFrameCounter)
    {
        Audio_Update();
        AudioFrameCounter += 1;
    }
}

void UpdateInput()
{
    u8 last;

    Pad0Prev = Pad0;
    ReadInput();
    while (TRUE)
    {
        last = Pad0;
        ReadInput();
        if (Pad0 == last) break;
    }
}

void ReadInput()
{
    Pad0 = ConvertKitaqfcPadBits(__pad_read1_safe());
}

u8 ConvertKitaqfcPadBits(u8 raw)
{
    u8 mapped = 0;

    if (raw & 0x01) mapped = mapped | BUTTON_A;
    if (raw & 0x02) mapped = mapped | BUTTON_B;
    if (raw & 0x04) mapped = mapped | BUTTON_SELECT;
    if (raw & 0x08) mapped = mapped | BUTTON_START;
    if (raw & 0x10) mapped = mapped | BUTTON_UP;
    if (raw & 0x20) mapped = mapped | BUTTON_DOWN;
    if (raw & 0x40) mapped = mapped | BUTTON_LEFT;
    if (raw & 0x80) mapped = mapped | BUTTON_RIGHT;

    return mapped;
}

bool Button(u8 button)
{
    return (Pad0 & button) != 0;
}

bool ButtonDown(u8 button)
{
    return ((Pad0Prev & button) == 0) && ((Pad0 & button) != 0);
}

void SeedRandom(u8 seed)
{
    if (seed == 0) seed = 0x80;
    RandomState = seed;
    RandomState = RandomState << 8;
    RandomState = RandomState | seed;
}

u8 GetRandomByte()
{
    __asm
    {
        LDY #8
        LDA RandomState+0
    random_byte_loop:
        ASL
        ROL RandomState+1
        BCC random_byte_skip
        EOR #$2D
    random_byte_skip:
        DEY
        BNE random_byte_loop
        STA RandomState+0
        CMP #0
    }
}

void DisablePPU()
{
    PpuEnablePending = FALSE;
    PpuMask = 0x00;
    BeginFrame();
}

void EnablePPU()
{
    PpuMask = 0x1E;
    PpuEnablePending = TRUE;
    BeginFrame();
}

void SetScroll(u8 x, u8 y)
{
    PpuScrollX = x;
    PpuScrollY = y;
}

void SetPalettes(u8 *bg, u8 *sp)
{
    for (u8 i = 0; i < 16; ++i)
    {
        PpuPalettes[i] = *bg;
        bg++;
    }
    for (u8 j = 0; j < 16; ++j)
    {
        PpuPalettes[j + 16] = *sp;
        sp++;
    }
    PpuPalettesNew = 1;
}

void ClearNametable()
{
    for (u8 page = 0; page < 4; ++page)
    {
        u16 addr = VRAM_NAMETABLE0 + page * 0x100;
        __vram_fill(addr, TILE_EMPTY, 128);
        __vram_fill(addr + 128, TILE_EMPTY, 128);
    }

    __vram_fill(VRAM_NAMETABLE0 + 0x03C0, 0, 64);
    for (u8 i = 0; i < 64; ++i)
    {
        NameAttributes[i] = 0;
    }
}

u8 CharToTile(u8 c)
{
    if (c == ' ') return TILE_EMPTY;
    if (c == ':') return TILE_COLON;
    if (c == '-') return TILE_DASH;
    if (c == '*') return TILE_STAR;
    if (c == '.') return TILE_DOT;
    if (c >= '0' && c <= '9') return TILE_DIGIT + c - '0';
    if (c >= 'A' && c <= 'Z') return TILE_LETTER + c - 'A';
    return TILE_EMPTY;
}

void AttrSetNow(u8 x, u8 y, u8 attr)
{
    u8 index;
    u8 shift = 0;
    u8 mask;
    u8 value;

    if (x >= 32) return;
    if (y >= 30) return;

    index = (y >> 2) << 3;
    index += x >> 2;
    if (x & 2) shift += 2;
    if (y & 2) shift += 4;

    attr = attr & 3;
    mask = 3 << shift;
    value = NameAttributes[index];
    value = (value & (mask ^ 0xFF)) | (attr << shift);
    NameAttributes[index] = value;

    __ppu_addr(VRAM_NAMETABLE0 + 0x03C0 + index);
    __ppu_data(value);
}

void AttrSetBlit(u8 x, u8 y, u8 attr)
{
    u8 index;
    u8 shift = 0;
    u8 mask;
    u8 value;

    if (x >= 32) return;
    if (y >= 30) return;

    index = (y >> 2) << 3;
    index += x >> 2;
    if (x & 2) shift += 2;
    if (y & 2) shift += 4;

    attr = attr & 3;
    mask = 3 << shift;
    value = NameAttributes[index];
    value = (value & (mask ^ 0xFF)) | (attr << shift);
    NameAttributes[index] = value;

    __vramq_put(VRAM_NAMETABLE0 + 0x03C0 + index, value);
}

#pragma fixed_bank 0
u16 NameAddr(u8 x, u8 y)
{
    u16 addr = VRAM_NAMETABLE0;
    addr = addr + y * 32;
    addr = addr + x;
    return addr;
}

bool IsVisibleNameCell(u8 x, u8 y)
{
    if (x >= 32) return FALSE;
    if (y >= 30) return FALSE;
    return TRUE;
}
#pragma fixed_bank -1

void PutTileNow(u8 x, u8 y, u8 tile)
{
    __nametable_put(x, y, tile);
}

void PutStringNow(u8 x, u8 y, char *s)
{
    u16 addr = NameAddr(x, y);
    __ppu_addr(addr);
    while (*s)
    {
        __ppu_data(CharToTile(*s));
        s++;
    }
}

void ClearBlitBuffer()
{
    __vramq_clear();
    __vramq_clear_overflow();
}

void CommitBlitBuffer()
{
    __vramq_commit();
}

void FlushBlitChunk()
{
    CommitBlitBuffer();
    FlushBlitFrame();
    ClearBlitBuffer();
}

void SplitDecimal10(u8 value)
{
    DecimalTens = 0;
    while (value >= 10)
    {
        value -= 10;
        DecimalTens += 1;
    }
    DecimalOnes = value;
}

void PutNumber2Now(u8 x, u8 y, u8 value)
{
    u16 addr = NameAddr(x, y);

    SplitDecimal10(value);
    __ppu_addr(addr);
    __ppu_data(TILE_DIGIT + DecimalTens);
    __ppu_data(TILE_DIGIT + DecimalOnes);
}

#pragma fixed_bank 0
void PutScoreNow(u8 x, u8 y)
{
    u16 addr = NameAddr(x, y);

    __ppu_addr(addr);
    SplitDecimal10(ScoreBCD[3]);
    __ppu_data(TILE_DIGIT + DecimalTens);
    __ppu_data(TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[2]);
    __ppu_data(TILE_DIGIT + DecimalTens);
    __ppu_data(TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[1]);
    __ppu_data(TILE_DIGIT + DecimalTens);
    __ppu_data(TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[0]);
    __ppu_data(TILE_DIGIT + DecimalOnes);
}
#pragma fixed_bank -1

void PutTileBlit(u8 x, u8 y, u8 tile)
{
    if (!IsVisibleNameCell(x, y)) return;

    __vramq_put(NameAddr(x, y), tile);
}

void PutStringBlit(u8 x, u8 y, char *s)
{
    u8 wrote = FALSE;
    u16 addr = NameAddr(x, y);

    if (!IsVisibleNameCell(x, y)) return;

    while (*s)
    {
        if (x >= 32) break;

        __vramq_put(addr, CharToTile(*s));
        addr += 1;
        x += 1;
        s++;
        wrote = TRUE;
    }

    if (!wrote) return;
}

void PutNumber2Blit(u8 x, u8 y, u8 value)
{
    u16 addr = NameAddr(x, y);

    if (!IsVisibleNameCell(x, y)) return;
    if (x >= 31) return;

    SplitDecimal10(value);
    __vramq_put(addr, TILE_DIGIT + DecimalTens);
    __vramq_put(addr + 1, TILE_DIGIT + DecimalOnes);
}

#pragma fixed_bank 0
void PutScoreBlit(u8 x, u8 y)
{
    u16 addr = NameAddr(x, y);

    if (!IsVisibleNameCell(x, y)) return;
    if (x > 25) return;

    SplitDecimal10(ScoreBCD[3]);
    __vramq_put(addr, TILE_DIGIT + DecimalTens);
    __vramq_put(addr + 1, TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[2]);
    __vramq_put(addr + 2, TILE_DIGIT + DecimalTens);
    __vramq_put(addr + 3, TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[1]);
    __vramq_put(addr + 4, TILE_DIGIT + DecimalTens);
    __vramq_put(addr + 5, TILE_DIGIT + DecimalOnes);
    SplitDecimal10(ScoreBCD[0]);
    __vramq_put(addr + 6, TILE_DIGIT + DecimalOnes);
}
#pragma fixed_bank -1

void FillBlit(u8 x, u8 y, u8 count, u8 tile)
{
    if (!IsVisibleNameCell(x, y)) return;
    if (count == 0) return;
    if (count > 32 - x) count = 32 - x;

    __vramq_fill(NameAddr(x, y), tile, count);
}

void DrawBoxNow(u8 x, u8 y, u8 w, u8 h)
{
    for (u8 xx = 0; xx < w; ++xx)
    {
        PutTileNow(x + xx, y, TILE_FRAME);
        PutTileNow(x + xx, y + h - 1, TILE_FRAME);
    }
    for (u8 yy = 0; yy < h; ++yy)
    {
        PutTileNow(x, y + yy, TILE_FRAME);
        PutTileNow(x + w - 1, y + yy, TILE_FRAME);
    }
}

void DrawBoxBlit(u8 x, u8 y, u8 w, u8 h)
{
    if (w < 2) return;
    if (h < 2) return;

    FillBlit(x, y, w, TILE_FRAME);
    FlushBlitChunk();

    FillBlit(x, y + h - 1, w, TILE_FRAME);
    FlushBlitChunk();

    for (u8 yy = 1; yy < h - 1; ++yy)
    {
        PutTileBlit(x, y + yy, TILE_FRAME);
        FillBlit(x + 1, y + yy, w - 2, TILE_EMPTY);
        PutTileBlit(x + w - 1, y + yy, TILE_FRAME);
        FlushBlitChunk();
    }
}

#pragma fixed_bank 0
void ProcessBlitBuffer()
{
    __vramq_exec();
}

#pragma fixed_bank 0
void __nes_nmi()
{
    __asm
    {
        PHA
        TXA
        PHA
        TYA
        PHA
        LDA 0
        PHA
        LDA 1
        PHA
    }

    NmiFrameCounter += 1;

    if (!InMainThread)
    {
        if (PpuEnablePending)
        {
            PPU_MASK = 0;
            PpuMaskApplied = 0;
        }
        else if (PpuMask && PpuMask != PpuMaskApplied)
        {
            PPU_MASK = PpuMask;
            PpuMaskApplied = PpuMask;
        }

        __oam_dma_page(0x02);

        if (PpuPalettesNew)
        {
            PPU_ADDRESS = 0x3F;
            PPU_ADDRESS = 0x00;
            for (NmiPaletteIndex = 0; NmiPaletteIndex < 0x20; NmiPaletteIndex += 1)
            {
                PPU_DATA = PpuPalettes[NmiPaletteIndex];
            }
            PpuPalettesNew = 0;
        }

        ProcessBlitBuffer();
        PPU_SCROLL = PpuScrollX;
        PPU_SCROLL = PpuScrollY;
        PPU_CTRL = (PpuCtrl & 0x18) | 0x80;
        if (PpuEnablePending)
        {
            PPU_MASK = 0;
            PpuMaskApplied = 0;
            PpuEnablePending = FALSE;
        }
        else if (PpuMask != PpuMaskApplied)
        {
            PPU_MASK = PpuMask;
            PpuMaskApplied = PpuMask;
        }
        Audio_Update();
        AudioFrameCounter += 1;
        InMainThread = TRUE;
    }

    __asm
    {
        PLA
        STA 1
        PLA
        STA 0
        PLA
        TAY
        PLA
        TAX
        PLA
        RTI
    }
}

void __nes_irq()
{
    __asm
    {
        RTI
    }
}
#pragma fixed_bank -1
