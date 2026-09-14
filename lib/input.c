#include "input.h"

u8 kq_input_prev_keys;
u8 kq_input_keys;
u8 kq_input_press_keys;
u8 kq_input_release_keys;
u8 kq_input_repeat_keys;
// One countdown per logical button; reads of input state do not poll hardware or advance timers.
static u8 input_repeat_timer[8];

// Map one of eight logical button indices to its mask bit.
static u8 input_mask_for_index(u8 index)
{
    return (u8)(1 << index);
}

// Remap the NES serial-read bit order to the shared KITAQ BTN_* layout.
static u8 input_translate_pad(u8 raw)
{
    u8 out;
    out = 0;
    if ((raw & 0x80) != 0) out = (u8)(out | BTN_RIGHT);
    if ((raw & 0x40) != 0) out = (u8)(out | BTN_LEFT);
    if ((raw & 0x10) != 0) out = (u8)(out | BTN_UP);
    if ((raw & 0x20) != 0) out = (u8)(out | BTN_DOWN);
    if ((raw & 0x01) != 0) out = (u8)(out | BTN_A);
    if ((raw & 0x02) != 0) out = (u8)(out | BTN_B);
    if ((raw & 0x04) != 0) out = (u8)(out | BTN_SELECT);
    if ((raw & 0x08) != 0) out = (u8)(out | BTN_START);
    return out;
}

// Clear held/edge/repeat masks and reset each independent repeat countdown.
void input_init(void)
{
    u8 i;
    kq_input_prev_keys = 0;
    kq_input_keys = 0;
    kq_input_press_keys = 0;
    kq_input_release_keys = 0;
    kq_input_repeat_keys = 0;
    i = 0;
    while (i < 8) {
        input_repeat_timer[i] = INPUT_REPEAT_DELAY;
        i = (u8)(i + 1);
    }
}

// Read controller 1 through the safe intrinsic, translate its bit order and
// derive press/release edges. Repeat delays count calls to this function, so
// call it once per intended input tick.
void input_update(void)
{
    u8 i;
    u8 raw;

    kq_input_prev_keys = kq_input_keys;
    raw = __pad_read1_safe();
    kq_input_keys = input_translate_pad(raw);
    kq_input_press_keys = (u8)(kq_input_keys & (u8)(kq_input_keys ^ kq_input_prev_keys));
    kq_input_release_keys = (u8)(kq_input_prev_keys & (u8)(kq_input_keys ^ kq_input_prev_keys));
    kq_input_repeat_keys = kq_input_press_keys;

    i = 0;
    while (i < 8) {
        u8 mask;
        mask = input_mask_for_index(i);
        if ((kq_input_keys & mask) != 0) {
            if ((kq_input_press_keys & mask) != 0) {
                input_repeat_timer[i] = INPUT_REPEAT_DELAY;
            } else if (input_repeat_timer[i] != 0) {
                input_repeat_timer[i] = (u8)(input_repeat_timer[i] - 1);
            } else {
                kq_input_repeat_keys = (u8)(kq_input_repeat_keys | mask);
                input_repeat_timer[i] = INPUT_REPEAT_RATE;
            }
        } else {
            input_repeat_timer[i] = INPUT_REPEAT_DELAY;
        }
        i = (u8)(i + 1);
    }
}

// Return true when any masked logical button is held.
u8 input_down(u8 mask) { return (u8)((kq_input_keys & mask) != 0); }
// Return true when any masked button became pressed in the latest input update.
u8 input_pressed(u8 mask) { return (u8)((kq_input_press_keys & mask) != 0); }
// Return true when any masked button became released in the latest update.
u8 input_released(u8 mask) { return (u8)((kq_input_release_keys & mask) != 0); }
// Return true for a masked initial press or scheduled repeat pulse.
u8 input_repeat(u8 mask) { return (u8)((kq_input_repeat_keys & mask) != 0); }
// Read the complete translated held-button mask.
u8 input_current(void) { return kq_input_keys; }
// Read the translated held-button mask from before the latest update.
u8 input_previous(void) { return kq_input_prev_keys; }
