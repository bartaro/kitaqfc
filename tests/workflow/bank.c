#include "values.h"
#pragma bank 2
u8 other(u8 n){return n+1;}
#pragma bank 1
void main(void){other(7);while(1){}}
