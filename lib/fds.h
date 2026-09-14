#ifndef FDS_H
#define FDS_H

#include "intrinsics.h"

// Return the compile-time FDS mapper selection as 0 or 1; this is not a hardware probe.
u8 __fds_available(void);
// Return the inverted disk-absent bit from 4032 as 0 or 1; this checks presence, not transfer completion.
u8 __fds_disk_ready(void);
// Return raw status mask 4032 & 04, either 0 or 4. This is not a numbered disk-side identifier.
u8 __fds_side(void);
// Read the entire 4030 status register; this is not a stored BIOS error code.
u8 __fds_error(void);
// Poll disk presence until the absent bit clears. There is no timeout or cancellation path.
void __fds_wait_ready(void);
// Alias the same disk-presence wait as __fds_wait_ready; the call can block indefinitely.
void __fds_wait_insert(void);

#endif
