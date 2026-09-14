/*
 * KITAQFC phase13 frame-budgeted scene streaming helper.
 *
 * Intent:
 * - stream large nametable / attribute / palette assets across multiple frames
 * - reuse the deferred VRAM queue already flushed from __nes_nmi()
 * - keep logic simple enough for the current C subset
 */

extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);

#include "scene.h"

static const SceneDef* kq_scene_table;
static u8 kq_scene_count;
static u8 kq_scene_current;
static u8 kq_scene_changed;

struct NesSceneStreamState nes_scene_stream_state;

// Attach the scene table, select ID zero and mark the state changed. The FC scene
// API currently tracks IDs only and does not invoke SceneDef callbacks.
void scene_init(const SceneDef* scenes, u8 count)
{
    kq_scene_table = scenes;
    kq_scene_count = count;
    kq_scene_current = 0;
    kq_scene_changed = 1;
}

// Replace the table, clamp an invalid current ID to zero and set the change flag.
void scene_set_table(const SceneDef* scenes, u8 count)
{
    kq_scene_table = scenes;
    kq_scene_count = count;
    if (kq_scene_current >= kq_scene_count) kq_scene_current = 0;
    kq_scene_changed = 1;
}

// Accept a different valid scene ID and latch the change flag; no callbacks run.
void scene_change(u8 scene_id)
{
    if (scene_id >= kq_scene_count) return;
    if (scene_id == kq_scene_current) return;
    kq_scene_current = scene_id;
    kq_scene_changed = 1;
}

// Clear the change flag. Game-specific update callbacks must be called by the application.
void scene_update(void)
{
    kq_scene_changed = 0;
}

// Compatibility placeholder: this function performs no rendering or callback dispatch.
void scene_draw(void)
{
}

// Read the stored scene ID; callers must ensure a nonempty table before indexing it.
u8 scene_get_current(void)
{
    return kq_scene_current;
}

// Read the change flag, which remains set until scene_update clears it.
u8 scene_was_changed(void)
{
    return kq_scene_changed;
}

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
