typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
__location(0x0B00) u16 reserved;u8 g;void main(void){g=0xA5;result=g;while(1){}}
