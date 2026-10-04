#include "lib/fc.h"
#include "lib/fds_overlay.h"

#pragma bank 2
u8 overlay_value(void) {
    return 29;
}

#pragma bank 0
void main(void) {
    u8 ok;
    ok = __fds_overlay_farcall(2, overlay_value);
    (void)ok;
    if (__fds_file_exists(32)) {
        (void)__fds_file_size(32);
    }
    (void)__fds_current_bank();
    while (1) { }
}
