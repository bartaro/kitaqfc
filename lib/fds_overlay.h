#ifndef FDS_OVERLAY_H
#define FDS_OVERLAY_H

#include "intrinsics.h"

// Load the disk file id through BIOS and return its error byte; this alone does not update bank residency.
u8 __fds_load_overlay(u8 id);
// Map bank >=2 to overlay-start-id+(bank-2), load it and update residency on success.
// Return FF for bank 0 or 1; an enabled residency guard may skip an already-resident load.
u8 __fds_load_bank(u8 bank);
// Return zero when the software residency record matches, otherwise call __fds_load_bank.
u8 __fds_require_bank(u8 bank);
// Return the software resident-bank byte; it is not a checksum of overlay memory.
u8 __fds_current_bank(void);
// Compare bank with the software resident-bank byte and return 0 or 1.
u8 __fds_is_bank_resident(u8 bank);
// Read the emitted overlay-function table count; the current helper emits an initial zero entry.
u8 __fds_overlay_function_count(void);
// Use a direct call with a function name as the second argument; arbitrary numeric pointers are rejected.
u8 __fds_overlay_farcall(u8 bank, u16 func);
// Alternate direct-call spelling for FDS overlay calls; supply a function symbol, not a runtime pointer.
u8 __fds_farcall(u8 bank, u16 func);

#endif
