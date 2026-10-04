#ifndef MIP_AUDIO_H
#define MIP_AUDIO_H

#include "game_defs.h"

#define SFX_CARGO  ((u8)1)
#define SFX_BOUNCE ((u8)2)
#define SFX_DAMAGE ((u8)3)
#define SFX_GATE   ((u8)4)
#define SFX_BOOST  ((u8)5)
#define SFX_CLEAR  ((u8)6)

#define SONG_TITLE ((u8)0)
#define SONG_GAME  ((u8)1)
#define SONG_CLEAR ((u8)2)

void audio_init(void);
void audio_start_song(u8 song);
void audio_tick(void);
void audio_play_sfx(u8 kind);

#endif
