#ifndef FDS_OVERLAY_H
#define FDS_OVERLAY_H

#include "intrinsics.h"

u8 __fds_load_overlay(u8 id);
u8 __fds_load_bank(u8 bank);
u8 __fds_require_bank(u8 bank);
u8 __fds_current_bank(void);
u8 __fds_is_bank_resident(u8 bank);
u8 __fds_overlay_function_count(void);
u8 __fds_overlay_farcall(u8 bank, u16 func);
u8 __fds_farcall(u8 bank, u16 func);

#endif
