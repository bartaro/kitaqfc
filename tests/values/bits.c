typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 a;void main(void){a=0x55;result=(a^0x11)|0x80;while(1){}}
