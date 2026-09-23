#ifndef FDS_FILE_H
#define FDS_FILE_H

#include "intrinsics.h"

// Native FDS32: load a non-boot file at its disk-header address, then memmove
// PRG bytes to dst. Null dst keeps the native destination (also for PPU files).
// Both CPU spans must fit $0200-$07FF or $6000-$DFFF and be owned by the caller.
// Return 0, a BIOS error, $40 for a missing transfer, or $FF for invalid input.
// Call from common code with disk-I/O interrupt/display prerequisites satisfied.
u8 __fds_load_file(u8 id, u8* dst);
// Overwrite the final non-boot PRG slot on its side, preserving its ID/name/load
// address. len must equal its packaged size; src must be a valid CPU RAM span.
// The resolved physical ordinal selects the slot, independently of file ID.
// Return 0, a BIOS error, or $FF before any write for an invalid slot/span/size.
u8 __fds_save_file(u8 id, const u8* src, u16 len);
// Query the compiled --fds-meta/--fds-manifest table, not the inserted disk; return 0 or 1.
u8 __fds_file_exists(u8 id);
// Return the compiled metadata size, or zero if id is absent. No disk directory is read.
u16 __fds_file_size(u8 id);

#endif
