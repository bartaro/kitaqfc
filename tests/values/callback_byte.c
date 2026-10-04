typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
typedef u8(*Callback)(u8);u8 inc(u8 a){return a+1;}void main(void){Callback f=inc;result=f(41);while(1){}}
