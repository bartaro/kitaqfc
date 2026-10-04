typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 g; void main(void){u16 x=0x1234; g=x+0x102; result=(u8)g; result_hi=(u8)(g>>8); while(1){}}
