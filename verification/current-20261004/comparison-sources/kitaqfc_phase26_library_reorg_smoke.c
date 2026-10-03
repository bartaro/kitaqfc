#include "lib/fc.h"
#include "lib/math_fast.h"
#include "lib/math_fixed.h"
#include "lib/fds.h"
#include "lib/fds_file.h"
#include "lib/fds_sound.h"
#include "lib/zapper.h"
#include "lib/keyboard.h"
#include "lib/rob.h"
#include "lib/mic.h"
#include "lib/midi.h"

unsigned char phase26_probe(unsigned char x, unsigned char y)
{
    unsigned char d;
    d = __manhattan(x, y, 8, 8);
    if (__xy_in_rect(x, y, 0, 0, 32, 30))
    {
        __vramq_put(0x2000, d);
    }
    return d;
}
