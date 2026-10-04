typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 mul(u16 a,u16 b){return a*b;}void main(void){u16 n=mul(mul(13,7),3);result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
