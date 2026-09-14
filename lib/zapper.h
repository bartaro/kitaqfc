#ifndef ZAPPER_H
#define ZAPPER_H
#include "intrinsics.h"
// Return raw 4016 & 18, retaining trigger and active-low light bits.
u8 __zapper_raw1(void);
// Return raw 4017 & 18 for the second port.
u8 __zapper_raw2(void);
// Return port-1 trigger bit 4 normalized to 0 or 1.
u8 __zapper_trigger1(void);
// Return port-2 trigger bit 4 normalized to 0 or 1.
u8 __zapper_trigger2(void);
// Invert port-1 bit 3 and normalize: one means light detected.
u8 __zapper_light1(void);
// Invert port-2 bit 3 and normalize: one means light detected.
u8 __zapper_light2(void);
// Default to the port-2 normalized trigger reader.
u8 __zapper_trigger(void);
// Default to the port-2 normalized light reader.
u8 __zapper_light(void);
#endif
