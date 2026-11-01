#ifndef FDS_FILE_H
#define FDS_FILE_H

#include "intrinsics.h"

u8 __fds_load_file(u8 id, u8* dst);
u8 __fds_save_file(u8 id, const u8* src, u16 len);
u8 __fds_file_exists(u8 id);
u16 __fds_file_size(u8 id);

#endif
