#ifndef MIP_HARDWARE_H
#define MIP_HARDWARE_H

#include "game_defs.h"

extern u8 mip_pad_current;
extern u8 mip_pad_previous;
extern u8 mip_pad_pressed;
extern u16 mip_settile_total;
extern u8 mip_settile_frame;
extern u8 mip_settile_peak;
extern u8 mip_settile_overflow;

void hw_init(void);
void hw_wait_frame(void);
void hw_wait_vblank_direct(void);
void hw_poll_pad(void);
void hw_screen_off(void);
void hw_screen_on(void);
void hw_ppu_write(u16 address, const u8* data, u16 length);
void hw_ppu_fill(u16 address, u8 value, u16 length);
void hw_put_tile_direct(u8 x, u8 y, u8 tile);
void hw_clear_screen_direct(u8 tile);
void hw_clear_updates(void);
void hw_begin_frame_updates(void);
void hw_finish_frame_updates(void);
void settile(u8 x, u8 y, u8 tile);
void hw_oam_set(u8 id, u8 x, u8 y, u8 tile, u8 attr);
void hw_oam_hide_from(u8 first_id);
void hw_oam_hide_all(void);

#endif
