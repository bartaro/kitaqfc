#ifndef MATH_LUT_H
#define MATH_LUT_H

// Uniform 256-phase unsigned sine table in ROM. Range 1..255, centered at 128.
// Link math_lut.c once; indices 0..255 are valid and byte phase wrap is supported.
extern unsigned char MATH_SIN[256];

#endif
