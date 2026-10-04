typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
#pragma bank 2
u16 add(u16 a,u8 b){return a+b;}
#pragma bank 1
void main(void){u16 n=add(0x1234,5);result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
