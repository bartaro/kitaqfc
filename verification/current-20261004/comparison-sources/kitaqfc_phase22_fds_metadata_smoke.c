#include "lib/intrinsics.h"

u8 ok;
u16 size0;
u16 size1;

void main(void) {
    if (__fds_available()) {
        ok = __fds_file_exists(1);
        size0 = __fds_file_size(1);
        size1 = __fds_file_size(2);
    }
    for (;;) {
        __nmi_wait();
    }
}
