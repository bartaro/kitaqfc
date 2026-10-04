#include "assets.h"

/*
 * Gameplay tables are deliberately kept in MMC3's always-visible fixed bank.
 * Render and physics live in separate switchable banks, so ordinary pointers
 * may safely read these compact shared tables without a far-data copy.
 */
#pragma fixed_bank 0

__prg_rom const u8 mip_palette[32] = {
    0x0F,0x01,0x11,0x21,
    0x0F,0x06,0x16,0x27,
    0x0F,0x08,0x18,0x28,
    0x0F,0x09,0x19,0x29,
    0x0F,0x07,0x17,0x27,
    0x0F,0x06,0x16,0x27,
    0x0F,0x04,0x14,0x24,
    0x0F,0x09,0x19,0x29
};

/* tile x, tile y, width, height */
__prg_rom const u8 mip_wall_rects[16] = {
    9,7,1,7,
    14,10,8,1,
    22,15,1,8,
    6,20,9,1
};

__prg_rom const u8 mip_bumpers[6] = {
    12,7,
    18,17,
    26,9
};

__prg_rom const u8 mip_cargo_x[24] = {
    4,15,25,6,17,27,4,17,
    7,12,18,27,3,18,11,25,
    4,12,20,27,4,18,26,13
};

__prg_rom const u8 mip_cargo_y[24] = {
    7,6,7,15,14,18,24,22,
    8,12,7,12,18,18,24,24,
    10,6,8,6,22,16,20,24
};

__prg_rom const u8 mip_text_title[] = {
    'M','I','P',' ','R','O','C','K','E','T',' ','C','O','U','R','I','E','R',0
};
__prg_rom const u8 mip_text_subtitle[] = {
    'M','O','M','E','N','T','U','M',' ','D','E','L','I','V','E','R','Y',0
};
__prg_rom const u8 mip_text_start[] = {
    'P','R','E','S','S',' ','S','T','A','R','T',0
};
__prg_rom const u8 mip_text_controls1[] = {
    'D','P','A','D',' ','T','H','R','U','S','T',' ',' ','A',' ','B','O','O','S','T',0
};
__prg_rom const u8 mip_text_controls2[] = {
    'B',' ','A','I','R',' ','B','R','A','K','E',0
};
__prg_rom const u8 mip_text_goal1[] = {
    'G','E','T',' ','8',' ','C','A','R','G','O',' ','C','O','R','E','S',0
};
__prg_rom const u8 mip_text_goal2[] = {
    'T','H','E','N',' ','D','O','C','K',' ','A','T',' ','T','H','E',' ','R','I','N','G',0
};
__prg_rom const u8 mip_text_hud_score[] = {'S','C','O','R','E',0};
__prg_rom const u8 mip_text_hud_time[] = {'T','I','M','E',0};
__prg_rom const u8 mip_text_hud_round[] = {'R',0};
__prg_rom const u8 mip_text_clear[] = {
    'A','L','L',' ','C','A','R','G','O',' ','D','E','L','I','V','E','R','E','D','!',0
};
__prg_rom const u8 mip_text_fail[] = {
    'D','E','L','I','V','E','R','Y',' ','L','O','S','T','!',0
};
__prg_rom const u8 mip_text_result_score[] = {'F','I','N','A','L',' ','S','C','O','R','E',0};
__prg_rom const u8 mip_text_again[] = {'S','T','A','R','T',' ','T','O',' ','R','E','T','R','Y',0};

#pragma fixed_bank -1
