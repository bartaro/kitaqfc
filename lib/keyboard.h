#ifndef KEYBOARD_H
#define KEYBOARD_H
#include "intrinsics.h"
// Probe selected-row and disabled-matrix bit patterns and return 0 or 1; this is a heuristic detection.
u8 __fkb_detect(void);
// Write 18 raw masked samples (4017 & 1E), two per row, to caller storage; reset 4016 afterward.
void __fkb_scan(u8* dst18);
// Select a matrix row and the low bit of col, then return raw 4017 & 1E.
// Use rows 0..8 for keys; row 9 is used by detection. No row-range validation is emitted.
u8 __fkb_read_row_col(u8 row, u8 col);
#endif
