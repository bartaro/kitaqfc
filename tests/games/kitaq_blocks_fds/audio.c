#include "kbfc.h"

__location(0x4000) u8 APU_0;
__location(0x4001) u8 APU_1;
__location(0x4002) u8 APU_2;
__location(0x4003) u8 APU_3;
__location(0x4004) u8 APU_4;
__location(0x4005) u8 APU_5;
__location(0x4006) u8 APU_6;
__location(0x4007) u8 APU_7;
__location(0x4008) u8 APU_8;
__location(0x400A) u8 APU_A;
__location(0x400B) u8 APU_B;
__location(0x400C) u8 APU_C;
__location(0x400E) u8 APU_E;
__location(0x400F) u8 APU_F;

u8 AudioBgm;
u8 AudioTick;
u8 SfxId;
u8 SfxTimer;

void Audio_Initialize()
{
    APU_STATUS = 0x0F;
    APU_FRAME_CTR = 0x40;
    AudioBgm = BGM_NONE;
    AudioTick = 0;
    SfxId = 0;
    SfxTimer = 0;
    Audio_StopBgm();
}

void Audio_PlayBgm(u8 id)
{
    Audio_StopBgm();

    if (id == BGM_NONE) return;
    if (!SettingMusic) return;

    AudioBgm = id;
    AudioTick = 0;
    Music_Start(id);
}

void Audio_StopBgm()
{
    AudioBgm = BGM_NONE;
    Music_Stop();
}

void Sfx_Play(u8 id)
{
    if (!SettingSound) return;
    if (id != SFX_LOCK && id != SFX_HOLD && id != SFX_CLEAR) return;

    SfxId = id;
    if (id == SFX_CLEAR) SfxTimer = 12;
    else SfxTimer = 6;
}

void Audio_Update()
{
    AudioTick += 1;
    Music_Update();

    if (SfxTimer)
    {
        SfxTimer -= 1;

        if (SfxId == SFX_CLEAR)
        {
            APU_C = 0x38;
            APU_E = 0x04 + SfxTimer;
            APU_F = 0x08;
        }
        else
        {
            APU_4 = 0x38;
            APU_5 = 0x08;
            if (SfxId == SFX_HOLD) APU_6 = 0x20;
            else APU_6 = 0x58;
            APU_7 = 0x04;
        }

        if (SfxTimer == 0)
        {
            APU_4 = 0x30;
            APU_C = 0x30;
        }
    }
}
