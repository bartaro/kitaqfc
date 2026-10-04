#include "intrinsics.h"
void callback(void){}
void main(void) { __farcall(1,callback); while(1){} }
