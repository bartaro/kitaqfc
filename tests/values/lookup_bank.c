typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
#pragma bank 2
u16 scale(u8 a){return a*257;}
#pragma bank 1
void main(void){u16 n=scale(200);result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
