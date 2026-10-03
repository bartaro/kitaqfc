#include "lib/fc.h"
#include "lib/fds_overlay.h"

#pragma bank 2
u8 overlay_value(void) {
    return 30;
}

#pragma bank 3
u8 overlay_value_b3(void) {
    return 31;
}

#pragma bank 0
void main(void) {
    u8 a;
    u8 b;
    u8 count;

    count = __fds_overlay_function_count();
    (void)count;

    a = __fds_overlay_farcall(2, overlay_value);
    b = __fds_farcall(3, overlay_value_b3);

    if (!__fds_is_bank_resident(2)) {
        (void)__fds_require_bank(2);
    }

    (void)a;
    (void)b;
    while (1) { }
}
