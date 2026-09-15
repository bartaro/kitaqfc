extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
#include "scene.h"

static const SceneDef* kq_scene_table;
static u8 kq_scene_count;
static u8 kq_scene_current;
static u8 kq_scene_changed;
static u8 kq_scene_has_current;

// Attach the caller-owned scene table and clear current-scene state without invoking callbacks.
void scene_init(const SceneDef* scenes, u8 count)
{
    kq_scene_table = scenes;
    kq_scene_count = count;
    kq_scene_current = 0;
    kq_scene_changed = 0;
    kq_scene_has_current = 0;
}

// Replace the table by resetting scene state; this does not call the old scene's exit handler.
void scene_set_table(const SceneDef* scenes, u8 count)
{
    scene_init(scenes, count);
}

// Ignore invalid IDs; otherwise exit the current scene and enter the requested one.
// Selecting the current ID still performs exit/enter. Callbacks run synchronously.
void scene_change(u8 scene_id)
{
    SceneFunc fn;

    if (scene_id >= kq_scene_count) return;

    // Exit/enter handlers must not recursively change scenes or replace this shared table.
    if (kq_scene_has_current != 0) {
        fn = kq_scene_table[(__safe_index u8)kq_scene_current].exit;
        if (fn != 0) fn();
    }

    kq_scene_current = scene_id;
    kq_scene_has_current = 1;
    kq_scene_changed = 1;

    fn = kq_scene_table[(__safe_index u8)kq_scene_current].enter;
    if (fn != 0) fn();
}

// Clear the change flag before calling the active scene's update handler, so a
// transition made during that handler is visible afterward.
void scene_update()
{
    SceneFunc fn;

    if (kq_scene_has_current == 0) return;
    kq_scene_changed = 0;
    fn = kq_scene_table[(__safe_index u8)kq_scene_current].update;
    if (fn != 0) fn();
}

// Call the active scene's draw handler if one exists; no current scene is a no-op.
void scene_draw()
{
    SceneFunc fn;

    if (kq_scene_has_current == 0) return;
    fn = kq_scene_table[(__safe_index u8)kq_scene_current].draw;
    if (fn != 0) fn();
}

// Return the stored scene ID; zero is also returned before the first transition.
u8 scene_get_current()
{
    return kq_scene_current;
}

// Read the transition flag, which is cleared at the start of an active scene update.
u8 scene_was_changed()
{
    return kq_scene_changed;
}

struct NesSceneStreamState nes_scene_stream_state;

// Replace the single active transfer and retain its source pointer. Keep the
// source storage and any required ROM bank readable until all bytes are queued.
void nes_scene_stream_begin(unsigned short ppu_addr, unsigned char* src, unsigned short len, unsigned char chunk)
{
    nes_scene_stream_state.ppu_addr = ppu_addr;
    nes_scene_stream_state.src = src;
    nes_scene_stream_state.remaining = len;
    nes_scene_stream_state.chunk = chunk;
    nes_scene_stream_state.active = 1;
}

// Prepare the 960-byte tile portion of a nametable using 32-byte queue records.
void nes_scene_stream_begin_nametable(unsigned short nt_base, unsigned char* src960)
{
    nes_scene_stream_begin(nt_base, src960, 960, 32);
}

// Prepare the 64-byte attribute table at nametable base + 0x03C0.
void nes_scene_stream_begin_attr(unsigned short nt_base, unsigned char* src64)
{
    nes_scene_stream_begin((unsigned short)(nt_base + 0x03C0), src64, 64, 16);
}

// Prepare a 32-byte palette transfer to PPU address 0x3F00.
void nes_scene_stream_begin_palette(unsigned char* src32)
{
    nes_scene_stream_begin(0x3F00, src32, 32, 16);
}

// Queue as many chunks as available capacity allows, capped at 127 bytes each.
// Return one when queue exhaustion requires a later retry, zero when all data has
// been queued. Zero does not mean the NMI consumer has already written the PPU.
unsigned char nes_scene_stream_step(void)
{
    unsigned char chunk;

    while (nes_scene_stream_state.active != 0)
    {
        if (nes_scene_stream_state.remaining == 0)
        {
            nes_scene_stream_state.active = 0;
            return 0;
        }

        // A step may enqueue several chunks; chunk limits record size, not the total bytes or time per frame.
        chunk = nes_scene_stream_state.chunk;
        if (chunk == 0)
        {
            chunk = 32;
        }
        if ((unsigned short)chunk > nes_scene_stream_state.remaining)
        {
            chunk = (unsigned char)nes_scene_stream_state.remaining;
        }
        if (chunk > 127)
        {
            chunk = 127;
        }

        // A failed enqueue leaves this chunk pending so a later step can retry after the queue drains.
        if (nes_vram_queue_try_write(nes_scene_stream_state.ppu_addr, nes_scene_stream_state.src, chunk) == 0)
        {
            return 1;
        }

        nes_scene_stream_state.ppu_addr = (unsigned short)(nes_scene_stream_state.ppu_addr + chunk);
        nes_scene_stream_state.src = nes_scene_stream_state.src + chunk;
        nes_scene_stream_state.remaining = (unsigned short)(nes_scene_stream_state.remaining - chunk);
    }

    return 0;
}
