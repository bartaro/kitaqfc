#include "lib/intrinsics.h"

u8 buf_a[64];
u8 buf_b[64];
u8 flags[4];

u8 main(void)
{
    u8 d;
    u16 idx;
    u8 ok;
    u8 r;

    __rng_seed(0x1234);
    r = __rng8();

    d = __manhattan(10, 20, 3, 18);
    ok = __xy_in_rect(8, 9, 4, 5, 16, 16);
    idx = __map_index(3, 7, 32);

    __bit_set(flags, 9);
    if (__bit_test(flags, 9)) {
        __copy16(buf_a, buf_b);
        __memset_small(buf_a, d, 16);
    }

    /* Far memory helpers are compiled to bank-save/switch/read/restore helpers
       when a banking mapper is selected. */
    r ^= __farpeek8(1, 0x8000);
    idx ^= __farpeek16(1, 0x8001);
    __far_memcpy(buf_a, 1, 0x8000, 8);

    return (u8)(r ^ d ^ ok ^ (u8)idx);
}
