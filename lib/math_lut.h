#ifndef MATH_LUT_H
#define MATH_LUT_H

// Nominal legacy extent; math_lut.c actually supplies 254 initializer bytes.
// This declaration alone does not supply missing samples at indices 254 and 255.
extern unsigned char MATH_SIN[256];

#endif
