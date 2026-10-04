typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
s8 a;void main(void){a=-8;result=(u8)(a>>2);while(1){}}
