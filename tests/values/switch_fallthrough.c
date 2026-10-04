typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 a;void main(void){a=1;switch(a){case 1:result=3;fallthrough;case 2:result+=4;break;default:result=0xEE;}while(1){}}
