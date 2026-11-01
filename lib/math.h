#pragma once
#include "core.h"
// 128 + round(127*sin(2*pi*phase/256)); subtract 128 for signed amplitude.
extern __prg_rom u8 MATH_SIN[256];
