#ifndef FDS_FILE_H
#define FDS_FILE_H

#include "intrinsics.h"

// Call BIOS LoadFiles for id and return its error byte (zero on success).
// The disk header determines the load address; the current helper does not use dst.
u8 __fds_load_file(u8 id, u8* dst);
// Call BIOS WriteFile with a RAM source and byte length, returning its error byte.
// The generated PRG descriptor uses id as file number and the fixed name KQFCFILE.
u8 __fds_save_file(u8 id, const u8* src, u16 len);
// Query the compiled --fds-meta/--fds-manifest table, not the inserted disk; return 0 or 1.
u8 __fds_file_exists(u8 id);
// Return the compiled metadata size, or zero if id is absent. No disk directory is read.
u16 __fds_file_size(u8 id);

#endif
