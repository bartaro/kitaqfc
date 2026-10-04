/*
 * KITAQFC phase 8 NES controller helper.
 *
 * Intent:
 * - stay within the currently supported C subset
 * - read controller 1 from $4016
 * - expose held / pressed / released states
 */

__location(0x4016) unsigned char JOY1;
__location(0x4017) unsigned char JOY2;

unsigned char nes_pad1_cur;
unsigned char nes_pad1_prev;
unsigned char nes_pad1_pressed;
unsigned char nes_pad1_released;

void nes_pad_poll(void)
{
    unsigned char state;
    unsigned char mask;
    unsigned char sample;

    nes_pad1_prev = nes_pad1_cur;

    JOY1 = 1;
    JOY1 = 0;

    state = 0;
    mask = 1;
    while (mask != 0)
    {
        sample = JOY1;
        if ((sample & 1) != 0)
        {
            state = state | mask;
        }
        mask = mask + mask;
    }

    nes_pad1_cur = state;
    nes_pad1_pressed = (unsigned char)(state & (unsigned char)(state ^ nes_pad1_prev));
    nes_pad1_released = (unsigned char)(nes_pad1_prev & (unsigned char)(state ^ nes_pad1_prev));

    sample = JOY2;
    sample = sample;
}
