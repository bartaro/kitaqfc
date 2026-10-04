typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
u16 a;u16 b;void main(void){a=257;b=256;if(a>b){result=0x5A;}else{result=0xEE;}while(1){}}
