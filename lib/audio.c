#include "audio.h"

__location(0x4000) u8 SQ1_VOL;
__location(0x4001) u8 SQ1_SWEEP;
__location(0x4002) u8 SQ1_LO;
__location(0x4003) u8 SQ1_HI;
__location(0x4004) u8 SQ2_VOL;
__location(0x4005) u8 SQ2_SWEEP;
__location(0x4006) u8 SQ2_LO;
__location(0x4007) u8 SQ2_HI;
__location(0x4008) u8 TRI_LINEAR;
__location(0x400A) u8 TRI_LO;
__location(0x400B) u8 TRI_HI;
__location(0x400C) u8 NOISE_VOL;
__location(0x400E) u8 NOISE_LO;
__location(0x400F) u8 NOISE_HI;
__location(0x4010) u8 DMC_FREQ;
__location(0x4011) u8 DMC_RAW;
__location(0x4012) u8 DMC_START;
__location(0x4013) u8 DMC_LEN;
__location(0x4015) u8 APU_STATUS;
__location(0x4017) u8 APU_FRAME;

void nes_apu_init(void)
{
    APU_FRAME = 0x40;
    APU_STATUS = 0x0F;
    SQ1_VOL = 0x10;
    SQ1_SWEEP = 0x08;
    SQ2_VOL = 0x10;
    SQ2_SWEEP = 0x08;
    TRI_LINEAR = 0x80;
    NOISE_VOL = 0x10;
    DMC_FREQ = 0x00;
    DMC_RAW = 0x00;
}

void nes_apu_channel_enable(u8 mask)
{
    APU_STATUS = (u8)(mask & 0x1F);
}

void nes_apu_silence_all(void)
{
    APU_STATUS = 0x00;
    SQ1_VOL = 0x10;
    SQ2_VOL = 0x10;
    TRI_LINEAR = 0x80;
    NOISE_VOL = 0x10;
    DMC_FREQ = 0x00;
    DMC_RAW = 0x00;
}

void nes_sfx_square1(u8 duty_volume, u16 period, u8 length_index)
{
    APU_STATUS = 0x01;
    SQ1_SWEEP = 0x08;
    SQ1_VOL = duty_volume;
    SQ1_LO = (u8)period;
    SQ1_HI = (u8)((u8)(length_index << 3) | (u8)((period >> 8) & 0x07));
}

void nes_sfx_square2(u8 duty_volume, u16 period, u8 length_index)
{
    APU_STATUS = 0x02;
    SQ2_SWEEP = 0x08;
    SQ2_VOL = duty_volume;
    SQ2_LO = (u8)period;
    SQ2_HI = (u8)((u8)(length_index << 3) | (u8)((period >> 8) & 0x07));
}

void nes_sfx_triangle(u8 linear, u16 period, u8 length_index)
{
    APU_STATUS = 0x04;
    TRI_LINEAR = linear;
    TRI_LO = (u8)period;
    TRI_HI = (u8)((u8)(length_index << 3) | (u8)((period >> 8) & 0x07));
}

void nes_sfx_noise(u8 volume, u8 period_mode, u8 length_index)
{
    APU_STATUS = 0x08;
    NOISE_VOL = volume;
    NOISE_LO = period_mode;
    NOISE_HI = (u8)(length_index << 3);
}

void nes_sfx_tick_blip(void)
{
    nes_sfx_square1(0x9A, 0x03F0, 4);
}

void nes_sfx_move_blip(void)
{
    nes_sfx_square1(0x5A, 0x0280, 2);
}

void nes_dmc_config(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len)
{
    DMC_FREQ = flags_rate;
    DMC_RAW = (u8)(output_level & 0x7F);
    DMC_START = (u8)((sample_addr - 0xC000) >> 6);
    DMC_LEN = (u8)((sample_len - 1) >> 4);
}

void nes_dmc_start(void)
{
    APU_STATUS = 0x1F;
}

void nes_dmc_stop(void)
{
    APU_STATUS = 0x0F;
}

void nes_dmc_play(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len)
{
    nes_dmc_config(flags_rate, output_level, sample_addr, sample_len);
    nes_dmc_start();
}
