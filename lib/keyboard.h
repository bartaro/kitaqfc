#ifndef KEYBOARD_H
#define KEYBOARD_H
#include "intrinsics.h"
u8 __fkb_detect(void);
void __fkb_scan(u8* dst18);
u8 __fkb_read_row_col(u8 row, u8 col);
#endif
