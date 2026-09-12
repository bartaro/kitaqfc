#ifndef SYSTEM_H
#define SYSTEM_H

#include "core.h"
#include "intrinsics.h"

typedef void (*SystemCallback)(void);

extern u16 kq_system_frame;

void system_init(void);
void system_wait_vblank(void);
void system_set_vblank_callback(SystemCallback callback);
u16 system_get_frame(void);
u8 system_get_frame8(void);
void system_enable_interrupts(void);
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
