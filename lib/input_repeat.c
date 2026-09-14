/*
 * KITAQFC phase 9 NES pad repeat / trigger helper.
 *
 * Intent:
 * - build on top of phase 8 pad polling
 * - expose per-button repeat bits with configurable first delay / repeat interval
 * - stay within the currently supported C subset
 */

extern unsigned char nes_pad1_cur;
extern unsigned char nes_pad1_pressed;
extern unsigned char nes_pad1_released;

unsigned char nes_pad_repeat_delay;
unsigned char nes_pad_repeat_interval;
unsigned char nes_pad1_repeat;
unsigned char nes_pad1_repeat_counter[8];

// Set initial/repeat countdowns and clear repeat output. Zero countdowns are
// allowed and cause a repeat on the next held-button step.
void nes_pad_repeat_config(unsigned char delay, unsigned char interval)
{
    unsigned char i;

    nes_pad_repeat_delay = delay;
    nes_pad_repeat_interval = interval;
    nes_pad1_repeat = 0;

    i = 0;
    while (i != 8)
    {
        nes_pad1_repeat_counter[i] = delay;
        i = i + 1;
    }
}

// Generate per-button repeat bits from the most recently polled pad state.
// Poll first, then call once per tick; a countdown reaching zero emits on the
// following step, so an interval N leaves N decrement steps between pulses.
void nes_pad_repeat_step(void)
{
    unsigned char bit_index;
    unsigned char mask;
    unsigned char counter;

    nes_pad1_repeat = 0;

    bit_index = 0;
    mask = 1;
    while (mask != 0)
    {
        if ((nes_pad1_cur & mask) == 0)
        {
            nes_pad1_repeat_counter[bit_index] = nes_pad_repeat_delay;
        }
        else
        {
            if ((nes_pad1_pressed & mask) != 0)
            {
                nes_pad1_repeat = nes_pad1_repeat | mask;
                nes_pad1_repeat_counter[bit_index] = nes_pad_repeat_delay;
            }
            else
            {
                counter = nes_pad1_repeat_counter[bit_index];
                if (counter == 0)
                {
                    nes_pad1_repeat = nes_pad1_repeat | mask;
                    nes_pad1_repeat_counter[bit_index] = nes_pad_repeat_interval;
                }
                else
                {
                    nes_pad1_repeat_counter[bit_index] = counter - 1;
                }
            }
        }

        bit_index = bit_index + 1;
        // Byte overflow terminates the scan after the eighth button; each button has its own countdown.
        mask = mask + mask;
    }
}

// Return the requested newly-pressed bits, not a normalized Boolean.
unsigned char nes_pad_trigger(unsigned char mask)
{
    return (unsigned char)(nes_pad1_pressed & mask);
}

// Return the requested repeat-pulse bits from the latest repeat step.
unsigned char nes_pad_repeat(unsigned char mask)
{
    return (unsigned char)(nes_pad1_repeat & mask);
}

// Return the requested newly-released bits from the latest pad poll.
unsigned char nes_pad_release(unsigned char mask)
{
    return (unsigned char)(nes_pad1_released & mask);
}

// Return the requested currently-held bits in the NES pad layout.
unsigned char nes_pad_held(unsigned char mask)
{
    return (unsigned char)(nes_pad1_cur & mask);
}
