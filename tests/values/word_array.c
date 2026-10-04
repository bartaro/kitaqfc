typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 data[4];void main(void){u8 i=2;data[i]=0x1234;result=(u8)data[i];result_hi=(u8)(data[i]>>8);while(1){}}
