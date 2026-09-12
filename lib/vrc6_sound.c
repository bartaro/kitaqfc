#include "vrc6_sound.h"

__location(0x9000) u8 VRC6_P1_CTRL;
__location(0x9001) u8 VRC6_P1_LO;
__location(0x9002) u8 VRC6_P1_HI;
__location(0xA000) u8 VRC6_P2_CTRL;
__location(0xA001) u8 VRC6_P2_LO;
__location(0xA002) u8 VRC6_P2_HI;
__location(0xB000) u8 VRC6_SAW_RATE;
__location(0xB001) u8 VRC6_SAW_LO;
__location(0xB002) u8 VRC6_SAW_HI;

void nes_vrc6_silence_all(void)
{
    VRC6_P1_HI = 0x00;
    VRC6_P2_HI = 0x00;
    VRC6_SAW_HI = 0x00;
    VRC6_P1_CTRL = 0x00;
    VRC6_P2_CTRL = 0x00;
    VRC6_SAW_RATE = 0x00;
}

void nes_vrc6_pulse1_set(u8 control, u16 period)
{
    VRC6_P1_CTRL = control;
    VRC6_P1_LO = (u8)period;
    VRC6_P1_HI = (u8)(0x80 | ((period >> 8) & 0x0F));
}

void nes_vrc6_pulse2_set(u8 control, u16 period)
{
    VRC6_P2_CTRL = control;
    VRC6_P2_LO = (u8)period;
    VRC6_P2_HI = (u8)(0x80 | ((period >> 8) & 0x0F));
}

void nes_vrc6_pulse1_off(void)
{
    VRC6_P1_HI = 0x00;
}

void nes_vrc6_pulse2_off(void)
{
    VRC6_P2_HI = 0x00;
}

void nes_vrc6_saw_set(u8 rate, u16 period)
{
    VRC6_SAW_RATE = (u8)(rate & 0x3F);
    VRC6_SAW_LO = (u8)period;
    VRC6_SAW_HI = (u8)(0x80 | ((period >> 8) & 0x0F));
}

void nes_vrc6_saw_off(void)
{
    VRC6_SAW_HI = 0x00;
}
