#ifndef FDS_H
#define FDS_H

#include "intrinsics.h"

u8 __fds_available(void);
u8 __fds_disk_ready(void);
u8 __fds_side(void);
u8 __fds_error(void);
void __fds_wait_ready(void);
void __fds_wait_insert(void);

#endif
