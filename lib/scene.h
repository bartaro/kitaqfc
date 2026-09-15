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

// One shared in-flight transfer. remaining counts source bytes not yet copied into the queue.
// An inactive stream may still have commands awaiting the NMI consumer.
struct NesSceneStreamState {
    unsigned char* src;
    unsigned short ppu_addr;
    unsigned short remaining;
    unsigned char chunk;
    unsigned char active;
};

extern struct NesSceneStreamState nes_scene_stream_state;

// Attach the caller-owned scene table and clear current-scene state without invoking callbacks.
void scene_init(const SceneDef* scenes, u8 count);
// Replace the table by resetting scene state; this does not call the old scene's exit handler.
void scene_set_table(const SceneDef* scenes, u8 count);
// Ignore invalid IDs; otherwise exit the current scene and enter the requested one.
// Selecting the current ID still performs exit/enter. Callbacks run synchronously.
void scene_change(u8 scene_id);
// Clear the change flag before calling the active scene's update handler, so a
// transition made during that handler is visible afterward.
void scene_update();
// Call the active scene's draw handler if one exists; no current scene is a no-op.
void scene_draw();
// Return the stored scene ID; zero is also returned before the first transition.
u8 scene_get_current();
// Read the transition flag, which is cleared at the start of an active scene update.
u8 scene_was_changed();


// Replace the single active transfer and retain its source pointer. Keep the
// source storage and any required ROM bank readable until all bytes are queued.
void nes_scene_stream_begin(unsigned short ppu_addr, unsigned char* src, unsigned short len, unsigned char chunk);
// Prepare the 960-byte tile portion of a nametable using 32-byte queue records.
void nes_scene_stream_begin_nametable(unsigned short nt_base, unsigned char* src960);
// Prepare the 64-byte attribute table at nametable base + 0x03C0.
void nes_scene_stream_begin_attr(unsigned short nt_base, unsigned char* src64);
// Prepare a 32-byte palette transfer to PPU address 0x3F00.
void nes_scene_stream_begin_palette(unsigned char* src32);
// Queue as many chunks as available capacity allows, capped at 127 bytes each.
// Return one when queue exhaustion requires a later retry, zero when all data has
// been queued. Zero does not mean the NMI consumer has already written the PPU.
unsigned char nes_scene_stream_step(void);

#endif
