#include "lib/intrinsics.h"

unsigned char main(void)
{
    unsigned char d;
    unsigned short idx;
    unsigned char buf[32];

    d = __manhattan(1, 2, 9, 7);
    idx = __map_index(3, 4, 32);
    if (__xy_in_rect(5, 6, 0, 0, 10, 10)) {
        __memset_small(buf, d, 16);
        __copy16(buf + 16, buf);
    }
    __rng_seed(idx);
    return (unsigned char)(d + __rng8());
}
