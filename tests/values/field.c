typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Pair{u8 a;u16 b;};struct Pair p;void main(void){p.a=3;p.b=0x1234;result=p.a;result_hi=(u8)(p.b>>8);while(1){}}
