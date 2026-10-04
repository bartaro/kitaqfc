typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
s8 a;s8 b;void main(void){a=-3;b=2;if(a<b){result=0x5A;}else{result=0xEE;}while(1){}}
