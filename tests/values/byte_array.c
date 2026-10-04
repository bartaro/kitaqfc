typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 data[4];void main(void){u8 i;for(i=0;i<4;i++){data[i]=i+10;}result=data[3];while(1){}}
