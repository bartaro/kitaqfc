typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
const u16 data[3]={0x1234,0x5678,0x90AB};void main(void){u8 i=1;result=(u8)data[i];result_hi=(u8)(data[i]>>8);while(1){}}
