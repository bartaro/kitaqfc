typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
s8 a;s16 b;void main(void){s16 n;a=-7;b=43;n=a*b;result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
