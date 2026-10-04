typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 a;void main(void){a=1;result=a?0xA5:0xEE;while(1){}}
