typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u8 a;u8 b;void main(void){a=7;b=3;if(a>b && b!=0){result=0xA5;}else{result=0xEE;}while(1){}}
