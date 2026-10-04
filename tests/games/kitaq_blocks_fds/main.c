#include "kbfc.h"

void __nes_reset()
{
    __asm
    {
        SEI
        CLD
        LDX #$40
        STX APU_FRAME_CTR
        LDX #$FF
        TXS
        LDX #0
        STX PPU_CTRL
        STX PPU_MASK
        STX APU_DMC
        BIT PPU_STATUS
    vblank_wait_1:
        BIT PPU_STATUS
        BPL vblank_wait_1
        STX APU_STATUS
        LDA #0
    clear_ram:
        STA $0000,X
        STA $0100,X
        STA $0300,X
        STA $0400,X
        STA $0500,X
        STA $0600,X
        STA $0700,X
        INX
        BNE clear_ram
    }

    ResetSprites();

    __asm
    {
    vblank_wait_2:
        BIT PPU_STATUS
        BPL vblank_wait_2
        LDA #>VRAM_PALETTES
        STA PPU_ADDRESS
        LDA #<VRAM_PALETTES
        STA PPU_ADDRESS
        LDA #$0F
        LDY #$20
    clear_palettes:
        STA PPU_DATA
        DEY
        BNE clear_palettes
        LDX #0
        STX OAM_ADDRESS
        LDA #$02
        STA OAM_DMA
        LDA PPU_STATUS
    }

    ClearBlitBuffer();
    InMainThread = TRUE;

    __asm
    {
        LDA #$C0
        STA $0100
        LDA #$80
        STA $0101
        LDA #$35
        STA $0102
        LDA #$AC
        STA $0103
        LDA #0
        STA $4022
    }

    PPU_CTRL = 0x80;
    main();
}

void main()
{
    Audio_Initialize();
    SeedRandom(0x42);
    SetScroll(0, 0);
    PpuCtrl = 0x00;
    PpuMask = 0x00;
    SettingStartLevel = 0;
    SettingRandomizer = RANDOMIZER_BAG;
    SettingNext = TRUE;
    SettingHold = TRUE;
    SettingMusic = TRUE;
    SettingSound = TRUE;
    EnablePPU();
    Title_Start();

    while (TRUE)
    {
        BeginFrame();
        if (CurrentMode == MODE_TITLE) Title_Update();
        else if (CurrentMode == MODE_GAME) Game_Update();
        else if (CurrentMode == MODE_PAUSE) Pause_Update();
        else if (CurrentMode == MODE_GAMEOVER) GameOver_Update();
        CommitBlitBuffer();
    }
}
