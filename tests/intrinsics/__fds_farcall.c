#include "intrinsics.h"
void callback(void){}
void main(void) { __fds_farcall(1,callback); while(1){} }
