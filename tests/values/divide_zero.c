typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 a;u16 b;void main(void){a=301;b=0;result=(u8)(a/b);result_hi=(u8)(a%b);while(1){}}
