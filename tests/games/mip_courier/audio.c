#include "audio.h"

__location(0x4000) u8 MIP_SQ1_VOL;
__location(0x4001) u8 MIP_SQ1_SWEEP;
__location(0x4002) u8 MIP_SQ1_LO;
__location(0x4003) u8 MIP_SQ1_HI;
__location(0x4004) u8 MIP_SQ2_VOL;
__location(0x4005) u8 MIP_SQ2_SWEEP;
__location(0x4006) u8 MIP_SQ2_LO;
__location(0x4007) u8 MIP_SQ2_HI;
__location(0x4008) u8 MIP_TRI_LINEAR;
__location(0x400A) u8 MIP_TRI_LO;
__location(0x400B) u8 MIP_TRI_HI;
__location(0x400C) u8 MIP_NOISE_VOL;
__location(0x400E) u8 MIP_NOISE_LO;
__location(0x400F) u8 MIP_NOISE_HI;
__location(0x4015) u8 MIP_APU_STATUS;
__location(0x4017) u8 MIP_APU_FRAME;

u8 mip_song;
u8 mip_music_clock;
u8 mip_music_step;
u8 mip_sfx_kind;
u8 mip_sfx_timer;

__prg_rom const u8 mip_note_lo[24] = {
    0x56,0x26,0xF9,0xCF,0xA6,0x80,0x5C,0x3A,
    0x1A,0xFC,0xDF,0xC4,0xAB,0x93,0x7C,0x67,
    0x52,0x3F,0x2D,0x1C,0x0C,0xFD,0xEE,0xE1
};

__prg_rom const u8 mip_note_hi[24] = {
    0x03,0x03,0x02,0x02,0x02,0x02,0x02,0x02,
    0x02,0x01,0x01,0x01,0x01,0x01,0x01,0x01,
    0x01,0x01,0x01,0x01,0x01,0x00,0x00,0x00
};

__prg_rom const u8 mip_title_melody[32] = {
    12,0xFF,16,19, 17,16,14,0xFF,
    12,14,16,21, 19,16,14,0xFF,
    17,0xFF,19,21, 23,21,19,16,
    14,16,17,19, 16,14,12,0xFF
};

__prg_rom const u8 mip_game_melody[32] = {
    12,16,19,16, 14,17,21,17,
    12,16,21,19, 17,16,14,11,
    14,17,21,23, 21,19,17,16,
    12,14,16,17, 19,17,14,11
};

__prg_rom const u8 mip_clear_melody[16] = {
    12,16,19,21, 16,19,23,21,
    17,21,23,0xFF, 19,23,21,0xFF
};

__prg_rom const u8 mip_bass_line[8] = {
    0,0,7,7, 5,5,7,9
};

void audio_write_pulse2(u8 note)
{
    if (note == 0xFF) {
        MIP_SQ2_VOL = 0x30;
        return;
    }
    MIP_SQ2_VOL = 0xBA;
    MIP_SQ2_SWEEP = 0x08;
    MIP_SQ2_LO = mip_note_lo[(__safe_index u8)note];
    MIP_SQ2_HI = (u8)(0x08 | mip_note_hi[(__safe_index u8)note]);
}

void audio_write_triangle(u8 note)
{
    MIP_TRI_LINEAR = 0x81;
    MIP_TRI_LO = mip_note_lo[(__safe_index u8)note];
    MIP_TRI_HI = (u8)(0x08 | mip_note_hi[(__safe_index u8)note]);
}

void audio_music_event(void)
{
    u8 note;
    u8 bass;
    u8 local_step;
    local_step = mip_music_step;
    if (mip_song == SONG_TITLE) {
        note = mip_title_melody[(__safe_index u8)local_step];
    } else if (mip_song == SONG_CLEAR) {
        note = mip_clear_melody[(__safe_index u8)(local_step & 15)];
    } else {
        note = mip_game_melody[(__safe_index u8)local_step];
    }
    audio_write_pulse2(note);

    if ((local_step & 3) == 0) {
        bass = mip_bass_line[(__safe_index u8)((local_step >> 2) & 7)];
        if (mip_song == SONG_CLEAR) {
            bass = (u8)(bass + 5);
        }
        audio_write_triangle(bass);
    }

    if ((local_step & 7) == 0) {
        MIP_NOISE_VOL = 0x17;
        MIP_NOISE_LO = 0x06;
        MIP_NOISE_HI = 0x08;
    } else if ((local_step & 3) == 2) {
        MIP_NOISE_VOL = 0x13;
        MIP_NOISE_LO = 0x02;
        MIP_NOISE_HI = 0x08;
    } else {
        MIP_NOISE_VOL = 0x10;
    }

    mip_music_step = (u8)((mip_music_step + 1) & 31);
}

void audio_sfx_tick(void)
{
    u8 phase;
    u8 note;
    if (mip_sfx_timer == 0) {
        MIP_SQ1_VOL = 0x30;
        return;
    }
    mip_sfx_timer = (u8)(mip_sfx_timer - 1);
    phase = mip_sfx_timer;

    if (mip_sfx_kind == SFX_CARGO) {
        note = (u8)(16 + ((12 - phase) >> 2));
        MIP_SQ1_VOL = (u8)(0xB0 | (u8)(8 + (phase >> 2)));
        MIP_SQ1_LO = mip_note_lo[(__safe_index u8)note];
        MIP_SQ1_HI = (u8)(0x08 | mip_note_hi[(__safe_index u8)note]);
    } else if (mip_sfx_kind == SFX_BOUNCE) {
        MIP_SQ1_VOL = (u8)(0x90 | (phase & 0x0F));
        MIP_SQ1_LO = (u8)(0x60 + (phase << 3));
        MIP_SQ1_HI = 0x08;
    } else if (mip_sfx_kind == SFX_DAMAGE) {
        note = (u8)(8 + (phase >> 2));
        MIP_SQ1_VOL = (u8)(0x50 | (phase & 0x0F));
        MIP_SQ1_LO = mip_note_lo[(__safe_index u8)note];
        MIP_SQ1_HI = (u8)(0x08 | mip_note_hi[(__safe_index u8)note]);
        MIP_NOISE_VOL = (u8)(0x10 | (phase & 0x0F));
        MIP_NOISE_LO = 0x0E;
        MIP_NOISE_HI = 0x08;
    } else if (mip_sfx_kind == SFX_GATE || mip_sfx_kind == SFX_CLEAR) {
        note = (u8)(12 + ((24 - phase) >> 2));
        if (note > 23) note = 23;
        MIP_SQ1_VOL = 0xBF;
        MIP_SQ1_LO = mip_note_lo[(__safe_index u8)note];
        MIP_SQ1_HI = (u8)(0x08 | mip_note_hi[(__safe_index u8)note]);
    } else {
        MIP_SQ1_VOL = (u8)(0xD0 | (phase & 0x0F));
        MIP_SQ1_LO = (u8)(0x20 + (phase << 2));
        MIP_SQ1_HI = 0x08;
    }
}

void audio_init(void)
{
    MIP_APU_FRAME = 0x40;
    MIP_APU_STATUS = 0x0F;
    MIP_SQ1_SWEEP = 0x08;
    MIP_SQ2_SWEEP = 0x08;
    MIP_SQ1_VOL = 0x30;
    MIP_SQ2_VOL = 0x30;
    MIP_TRI_LINEAR = 0x80;
    MIP_NOISE_VOL = 0x10;
    mip_song = SONG_TITLE;
    mip_music_clock = 0;
    mip_music_step = 0;
    mip_sfx_kind = 0;
    mip_sfx_timer = 0;
}

void audio_start_song(u8 song)
{
    mip_song = song;
    mip_music_clock = 0;
    mip_music_step = 0;
    audio_music_event();
}

void audio_tick(void)
{
    mip_music_clock = (u8)(mip_music_clock + 1);
    if (mip_music_clock >= 7) {
        mip_music_clock = 0;
        audio_music_event();
    }
    audio_sfx_tick();
}

void audio_play_sfx(u8 kind)
{
    mip_sfx_kind = kind;
    if (kind == SFX_GATE || kind == SFX_CLEAR) {
        mip_sfx_timer = 24;
    } else if (kind == SFX_DAMAGE) {
        mip_sfx_timer = 16;
    } else {
        mip_sfx_timer = 12;
    }
}
