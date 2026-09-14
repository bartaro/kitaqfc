#ifndef SYSTEM_H
#define SYSTEM_H

#include "core.h"
#include "intrinsics.h"

// Retained for source compatibility; this FC implementation stores but never invokes this callback.
typedef void (*SystemCallback)(void);

extern u16 kq_system_frame;

// Reset the software frame counter and callback storage, then enable NMI.
void system_init(void);
// Wait for NMI and count the completed wait. Unlike the GB implementation, this
// FC wrapper does not invoke the callback stored by system_set_vblank_callback.
void system_wait_vblank(void);
// Store the callback for API compatibility. No function in this implementation
// invokes it; applications must call their per-frame work explicitly.
void system_set_vblank_callback(SystemCallback callback);
// Return the wrapping 16-bit count of completed waits through this wrapper.
u16 system_get_frame(void);
// Return the low byte of the software frame counter.
u8 system_get_frame8(void);
// Enable maskable CPU IRQs through the intrinsic; NMI is controlled separately.
void system_enable_interrupts(void);
// Disable maskable CPU IRQs; this does not disable the PPU's NMI source.
void system_disable_interrupts(void);

#define fc_init system_init
#define fc_wait_vblank system_wait_vblank
#define fc_set_vblank_callback system_set_vblank_callback
#define fc_frame_count system_get_frame
#define gb_init system_init
#define gb_wait_vblank system_wait_vblank
#define gb_set_vblank_callback system_set_vblank_callback
#define gb_frame_count system_get_frame

#endif
