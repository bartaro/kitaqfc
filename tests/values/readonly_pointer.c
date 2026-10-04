typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
const u8 data[4]={2,4,6,8};const u8*const pointers[2]={data,&data[2]};void main(void){result=*pointers[1];while(1){}}
