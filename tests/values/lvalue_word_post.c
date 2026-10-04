typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 data[2];void main(void){u16 n;data[1]=255;n=data[1]++;result=(u8)n;result_hi=(u8)(data[1]>>8);while(1){}}
