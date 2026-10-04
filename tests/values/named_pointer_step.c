typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Block{u8 data[256];};void main(void){struct Block*p=(struct Block*)0x0400;p++;result=(u8)(u16)p;result_hi=(u8)((u16)p>>8);while(1){}}
