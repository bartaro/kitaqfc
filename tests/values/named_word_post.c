typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
void main(void){u16 a=255;u16 n=a++;result=(u8)n;result_hi=(u8)(a>>8);while(1){}}
