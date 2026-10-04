#ifndef MIP_RENDER_H
#define MIP_RENDER_H

#include "game_defs.h"

void render_title_screen(void);
void render_game_screen(void);
void render_result_screen(void);
void render_title_sprites(void);
void render_game_sprites(void);
void render_queue_score(void);
void render_queue_time(void);
void render_queue_health(void);
void render_queue_boosts(void);
void render_queue_stage(void);
void render_queue_collected_cargo(void);
void render_queue_gate(u8 open);
void render_queue_next_stage(void);

#endif
