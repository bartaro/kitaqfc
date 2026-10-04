#include "vrc7_sound.h"

__location(0x9010) u8 VRC7_ADDR;
__location(0x9030) u8 VRC7_DATA;

void nes_vrc7_write(u8 reg, u8 value)
{
    VRC7_ADDR = reg;
    VRC7_DATA = value;
}

void nes_vrc7_set_user_patch(const u8* patch8)
{
    const u8* p;
    u8 i;
    p = patch8;
    i = 0;
    while (i < 8) {
        nes_vrc7_write(i, *p);
        p = p + 1;
        i = (u8)(i + 1);
    }
}

void nes_vrc7_channel_set(u8 channel, u8 instrument, u8 volume, u16 fnum, u8 block, u8 key_flags)
{
    u8 reg;
    u8 hi;
    if (channel >= 6) return;
    reg = channel;
    hi = (u8)(((fnum >> 8) & 0x01) | ((block & 0x07) << 1) | (key_flags & 0x30));
    nes_vrc7_write((u8)(0x10 + reg), (u8)fnum);
    nes_vrc7_write((u8)(0x20 + reg), hi);
    nes_vrc7_write((u8)(0x30 + reg), (u8)(((instrument & 0x0F) << 4) | (volume & 0x0F)));
}

void nes_vrc7_key_off(u8 channel)
{
    if (channel >= 6) return;
    nes_vrc7_write((u8)(0x20 + channel), 0x00);
}

void nes_vrc7_silence_all(void)
{
    u8 i;
    i = 0;
    while (i < 6) {
        nes_vrc7_key_off(i);
        nes_vrc7_write((u8)(0x30 + i), 0x0F);
        i = (u8)(i + 1);
    }
}
