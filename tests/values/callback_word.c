typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
typedef u16(*Callback)(u16,u8);u16 add(u16 a,u8 b){return a+b;}u8 inc(u8 a){return a+1;}void main(void){Callback f=&add;u16 n=f(0x1234,inc(4));result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
