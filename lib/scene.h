#ifndef SCENE_H
#define SCENE_H

#include "core.h"

typedef void (*SceneFunc)(void);

typedef __packed struct SceneDef {
    SceneFunc enter;
    SceneFunc update;
    SceneFunc draw;
    SceneFunc exit;
} SceneDef;

struct NesSceneStreamState {
    unsigned char* src;
    unsigned short ppu_addr;
    unsigned short remaining;
    unsigned char chunk;
    unsigned char active;
};

extern struct NesSceneStreamState nes_scene_stream_state;

void scene_init(const SceneDef* scenes, u8 count);
void scene_set_table(const SceneDef* scenes, u8 count);
void scene_change(u8 scene_id);
void scene_update(void);
void scene_draw(void);
u8 scene_get_current(void);
u8 scene_was_changed(void);

void nes_scene_stream_begin(unsigned short ppu_addr, unsigned char* src, unsigned short len, unsigned char chunk);
void nes_scene_stream_begin_nametable(unsigned short nt_base, unsigned char* src960);
void nes_scene_stream_begin_attr(unsigned short nt_base, unsigned char* src64);
void nes_scene_stream_begin_palette(unsigned char* src32);
unsigned char nes_scene_stream_step(void);

#endif
