#ifndef INPUT_H
#define INPUT_H

#include "core.h"
#include "intrinsics.h"

// These are shared logical masks, not raw controller-register bit positions.
// input_update translates controller 1 before these masks are applied.
#define BTN_RIGHT  ((u8)0x01)
#define BTN_LEFT   ((u8)0x02)
#define BTN_UP     ((u8)0x04)
#define BTN_DOWN   ((u8)0x08)
#define BTN_A      ((u8)0x10)
#define BTN_B      ((u8)0x20)
#define BTN_SELECT ((u8)0x40)
#define BTN_START  ((u8)0x80)

// A new press pulses immediately; the next pulse occurs DELAY + 1 input updates later.
#ifndef INPUT_REPEAT_DELAY
#define INPUT_REPEAT_DELAY ((u8)18)
#endif

// Later pulses occur RATE + 1 updates apart; zero repeats on each held update.
#ifndef INPUT_REPEAT_RATE
#define INPUT_REPEAT_RATE ((u8)5)
#endif

extern u8 kq_input_prev_keys;
extern u8 kq_input_keys;
extern u8 kq_input_press_keys;
extern u8 kq_input_release_keys;
extern u8 kq_input_repeat_keys;

// Clear held/edge/repeat masks and reset each independent repeat countdown.
void input_init(void);
// Read controller 1 through the safe intrinsic, translate its bit order and
// derive press/release edges. Repeat delays count calls to this function, so
// call it once per intended input tick.
void input_update(void);
// Return true when any masked logical button is held.
u8 input_down(u8 mask);
// Return true when any masked button became pressed in the latest input update.
u8 input_pressed(u8 mask);
// Return true when any masked button became released in the latest update.
u8 input_released(u8 mask);
// Return true for a masked initial press or scheduled repeat pulse.
u8 input_repeat(u8 mask);
// Read the complete translated held-button mask.
u8 input_current(void);
// Read the translated held-button mask from before the latest update.
u8 input_previous(void);

#define btn_down input_down
#define btn_pressed input_pressed
#define btn_released input_released
#define btn_repeat input_repeat

#endif
