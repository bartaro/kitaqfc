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

// Attach the scene table, select ID zero and mark the state changed. The FC scene
// API currently tracks IDs only and does not invoke SceneDef callbacks.
void scene_init(const SceneDef* scenes, u8 count);
// Replace the table, clamp an invalid current ID to zero and set the change flag.
void scene_set_table(const SceneDef* scenes, u8 count);
// Accept a different valid scene ID and latch the change flag; no callbacks run.
void scene_change(u8 scene_id);
// Clear the change flag. Game-specific update callbacks must be called by the application.
void scene_update(void);
// Compatibility placeholder: this function performs no rendering or callback dispatch.
void scene_draw(void);
// Read the stored scene ID; callers must ensure a nonempty table before indexing it.
u8 scene_get_current(void);
// Read the change flag, which remains set until scene_update clears it.
u8 scene_was_changed(void);

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
