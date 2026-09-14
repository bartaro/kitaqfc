/*
 * KITAQFC phase 10 NES metasprite helper.
 *
 * Intent:
 * - write a metasprite byte stream into nes_oam_shadow
 * - stay within the current C subset
 * - use unsigned offsets only for now
 */

extern unsigned char nes_oam_shadow[256];

// Read four-byte records [dx,dy,tile,attr] until dx=0xFF, writing them to
// shadow OAM. Coordinates and destination offsets wrap as bytes. The caller
// must supply a terminator and enough unused slots; there is no capacity guard.
// Return the next slot index derived from the final wrapped byte offset.
// FF is reserved in the X-offset position, so that byte cannot encode a visible part at offset -1.
// Writing through the final slot wraps the returned next index to zero; callers track remaining capacity separately.
unsigned char nes_metasprite_draw(unsigned char oam_index, unsigned char base_x, unsigned char base_y, unsigned char* metasprite)
{
    unsigned char dst;
    unsigned char dx;

    dst = oam_index + oam_index;
    dst = dst + dst;

    while (1)
    {
        dx = metasprite[0];
        if (dx == 0xFF)
        {
            break;
        }

        // These are raw OAM Y bytes, not a corrected screen-top coordinate; no allocation bookkeeping is updated.
        nes_oam_shadow[dst] = (unsigned char)(base_y + metasprite[1]);
        nes_oam_shadow[(unsigned char)(dst + 1)] = metasprite[2];
        nes_oam_shadow[(unsigned char)(dst + 2)] = metasprite[3];
        nes_oam_shadow[(unsigned char)(dst + 3)] = (unsigned char)(base_x + dx);

        dst = (unsigned char)(dst + 4);
        metasprite = metasprite + 4;
    }

    return (unsigned char)(dst >> 2);
}

// Hide consecutive shadow slots by setting Y to 0xF8. Keep the requested
// range within 64 slots to avoid wrapping and hiding earlier entries.
void nes_metasprite_hide_from(unsigned char oam_index, unsigned char sprite_count)
{
    unsigned char dst;

    dst = oam_index + oam_index;
    dst = dst + dst;

    while (sprite_count != 0)
    {
        nes_oam_shadow[dst] = 0xF8;
        dst = (unsigned char)(dst + 4);
        sprite_count = sprite_count - 1;
    }
}
