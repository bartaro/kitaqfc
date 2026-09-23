#ifndef FDS_OVERLAY_H
#define FDS_OVERLAY_H

#include "intrinsics.h"

// Load disk file id through BIOS, return its error byte and mark bank residency FF (unknown).
u8 __fds_load_overlay(u8 id);
// Load logical bank 1 from its separate boot file, or bank 2+ from overlay-start-id+(bank-2).
// Return zero on success, FF for invalid/unavailable mappings, or the BIOS error.
// A failed BIOS transfer marks residency FF (unknown); a matching valid guard can skip loading.
u8 __fds_load_bank(u8 bank);
// Return zero when the software residency record matches, otherwise call __fds_load_bank.
u8 __fds_require_bank(u8 bank);
// Return the software resident-bank byte, or FF after a failed bank transfer.
// This is not a checksum of overlay memory.
u8 __fds_current_bank(void);
// Compare a valid bank with the software resident-bank byte; FF (unknown) never matches.
u8 __fds_is_bank_resident(u8 bank);
// Return the linker-generated overlay-function count; zero if the table is disabled.
u8 __fds_overlay_function_count(void);
// Evaluate bank once and call the named zero-argument function using its linked bank.
// Restore the caller window before returning; restoration failure stops in common code.
u8 __fds_overlay_farcall(u8 bank, u16 func);
// Same named, zero-argument dispatch as __fds_overlay_farcall; evaluate bank once.
u8 __fds_farcall(u8 bank, u16 func);

#endif
