#include "kbfc.h"

#pragma fixed_bank 0

define u8 MUSIC_THEME_STEPS = 64;
define u8 MUSIC_STEP_COUNT = 64;
define u8 MUSIC_STEP_FRAMES = 12;
define u8 MUSIC_GAMEOVER_STEPS = 16;
define u8 MUSIC_GAMEOVER_STEP_FRAMES = 12;
define u8 MUSIC_NO_NOTE = 0xFF;
define u8 MUSIC_FDS_VOLUME = 0x1F;
define u8 MUSIC_BASS_VOLUME = 0xBF;

u8 MusicId;
u8 MusicFrame;
u8 MusicStep;
u8 MusicBassNote;

#pragma fixed_bank -1
__prg_rom u8 MusicFdsWave[] = {
    32, 38, 44, 50, 55, 59, 62, 63,
    63, 61, 57, 52, 46, 40, 34, 28,
    22, 16, 11, 7, 4, 2, 1, 0,
    1, 3, 6, 10, 15, 21, 27, 32,
    37, 43, 49, 54, 58, 61, 63, 63,
    62, 59, 55, 49, 43, 37, 31, 25,
    19, 14, 9, 5, 2, 1, 0, 0,
    1, 3, 7, 12, 18, 24, 30, 32
};

__prg_rom u8 MusicNoteFds[] = {
    5, 5, 5, 6, 6, 6, 7, 7,
    8, 8, 9, 9, 10, 10, 11, 11,
    12, 13, 14, 14, 15, 16, 17, 18,
    19, 20, 22, 23, 24, 26, 27, 29,
    30, 32
};

__prg_rom u8 MusicApuPeriodLo[] = {
    86, 38, 249, 206, 166, 128, 92, 58,
    26, 251, 223, 196, 171, 147, 124, 103,
    82, 63, 45, 28, 12, 253, 239, 225,
    213, 201, 189, 179, 169, 159, 150, 142,
    134, 126
};

__prg_rom u8 MusicApuPeriodHi[] = {
    3, 3, 2, 2, 2, 2, 2, 2,
    2, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0
};

/* Simple two-voice broken-chord theme in a Bach-like C major / A minor shape. */
__prg_rom u8 MusicTheme1Lead[] = {
    0x10, 0x14, 0x17, 0x1C,
    0x14, 0x17, 0x1C, 0x20,
    0x0F, 0x13, 0x17, 0x1B,
    0x13, 0x17, 0x1B, 0x1F,
    0x10, 0x13, 0x19, 0x1C,
    0x13, 0x19, 0x1C, 0x20,
    0x0E, 0x12, 0x17, 0x1A,
    0x12, 0x17, 0x1A, 0x1E,
    0x0C, 0x10, 0x15, 0x19,
    0x10, 0x15, 0x19, 0x1C,
    0x0D, 0x11, 0x15, 0x1A,
    0x11, 0x15, 0x1A, 0x1D,
    0x0F, 0x12, 0x17, 0x1B,
    0x12, 0x17, 0x1B, 0x1F,
    0x10, 0x14, 0x17, 0x1C,
    0x17, 0x14, 0x12, 0x10
};

__prg_rom u8 MusicTheme1Bass[] = {
    0x04, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x04, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x0B, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x0B, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x09, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x09, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x07, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x07, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x05, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x05, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x06, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x06, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x0B, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x0B, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x04, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE,
    0x04, MUSIC_NO_NOTE, MUSIC_NO_NOTE, MUSIC_NO_NOTE
};

#pragma fixed_bank 0
u8 Music_GetThemeStep()
{
    return MusicStep & 0x3F;
}

void Music_PlayLead(u8 note)
{
    if (note >= 34) return;

    __fds_freq_set(MusicNoteFds[note]);
    __fds_volume_set(MUSIC_FDS_VOLUME);
}

void Music_PlayBass(u8 note)
{
    u8 triNote;

    if (note >= 34) return;

    MusicBassNote = note;
    APU_0 = MUSIC_BASS_VOLUME;
    APU_1 = 0x08;
    APU_2 = MusicApuPeriodLo[note];
    APU_3 = MusicApuPeriodHi[note] | 0x08;

    triNote = note;
    if (triNote >= 12) triNote -= 12;
    APU_8 = 0xBF;
    APU_A = MusicApuPeriodLo[triNote];
    APU_B = MusicApuPeriodHi[triNote] | 0x08;
}

void Music_Start(u8 id)
{
    MusicId = id;
    MusicFrame = 0;
    MusicStep = 0;
    MusicBassNote = MUSIC_NO_NOTE;

    __fds_sound_enable();
    __fds_wave_load(MusicFdsWave);
    __fds_env_set(0x97, 0x80, 0x00);

    if (id == BGM_GAMEOVER) Music_PlayLead(0x18);
    else Music_PlayLead(MusicTheme1Lead[0]);
}

void Music_Stop()
{
    MusicId = BGM_NONE;
    MusicFrame = 0;
    MusicStep = 0;
    MusicBassNote = MUSIC_NO_NOTE;

    __fds_freq_set(0);
    __fds_volume_set(0);
    APU_0 = 0x30;
    APU_1 = 0x08;
    APU_2 = 0;
    APU_3 = 0;
    APU_8 = 0;
    APU_A = 0;
    APU_B = 0;
}

void Music_Update()
{
    u8 lead;
    u8 bass;
    u8 themeStep;

    if (AudioBgm == BGM_NONE) return;
    if (!SettingMusic)
    {
        Audio_StopBgm();
        return;
    }

    if (MusicId == BGM_NONE) return;

    if (MusicFrame == 0)
    {
        if (MusicId == BGM_GAMEOVER)
        {
            lead = 0x1C - MusicStep;
            bass = MUSIC_NO_NOTE;
            if ((MusicStep & 0x03) == 0) bass = 0x04;
        }
        else
        {
            themeStep = Music_GetThemeStep();
            lead = MusicTheme1Lead[themeStep];
            bass = MusicTheme1Bass[themeStep];
        }

        Music_PlayLead(lead);
        if (bass != MUSIC_NO_NOTE) Music_PlayBass(bass);
    }

    MusicFrame += 1;
    if (MusicId == BGM_GAMEOVER)
    {
        if (MusicFrame >= MUSIC_GAMEOVER_STEP_FRAMES)
        {
            MusicFrame = 0;
            MusicStep += 1;
            if (MusicStep >= MUSIC_GAMEOVER_STEPS)
            {
                AudioBgm = BGM_NONE;
                Music_Stop();
                return;
            }
        }
    }
    else if (MusicFrame >= MUSIC_STEP_FRAMES)
    {
        MusicFrame = 0;
        MusicStep += 1;
        if (MusicStep >= MUSIC_STEP_COUNT) MusicStep = 0;
    }
}

#pragma fixed_bank -1
