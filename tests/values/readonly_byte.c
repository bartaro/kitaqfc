typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
const u8 data[4]={2,4,6,8};void main(void){u8 i=2;result=data[i];while(1){}}
