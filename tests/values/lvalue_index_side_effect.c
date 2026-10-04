typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 data[2];void main(void){u8 i=0;data[0]=3;data[1]=7;result=data[i++]++;result_hi=i+data[0];while(1){}}
