#include "values.h"
u8 rec(u8 n){if(n)return rec(n-1);return n;} void main(void){rec(2);while(1){}}
