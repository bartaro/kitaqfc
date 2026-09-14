/*
 * Optional declarations for the phase 9 NES pad repeat helper.
 *
 * Button bit layout:
 *   bit0=A bit1=B bit2=Select bit3=Start bit4=Up bit5=Down bit6=Left bit7=Right
 */

extern unsigned char nes_pad_repeat_delay;
extern unsigned char nes_pad_repeat_interval;
extern unsigned char nes_pad1_repeat;

// Set initial/repeat countdowns and clear repeat output. Zero countdowns are
// allowed and cause a repeat on the next held-button step.
// A fresh press emits immediately. Held repeats follow after delay+1 and then interval+1 steps.
// These are call counts, not milliseconds; use one fresh pad poll before each step.
void nes_pad_repeat_config(unsigned char delay, unsigned char interval);
// Generate per-button repeat bits from the most recently polled pad state.
// Poll first, then call once per tick; a countdown reaching zero emits on the
// following step, so an interval N leaves N decrement steps between pulses.
void nes_pad_repeat_step(void);
// Return the requested newly-pressed bits, not a normalized Boolean.
unsigned char nes_pad_trigger(unsigned char mask);
// Return the requested repeat-pulse bits from the latest repeat step.
unsigned char nes_pad_repeat(unsigned char mask);
// Return the requested newly-released bits from the latest pad poll.
unsigned char nes_pad_release(unsigned char mask);
// Return the requested currently-held bits in the NES pad layout.
unsigned char nes_pad_held(unsigned char mask);
