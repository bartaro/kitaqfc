typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Pair{u8 a;u16 b;};struct Pair p;void main(void){p.a=3;result=p.a++;result_hi=p.a;while(1){}}
