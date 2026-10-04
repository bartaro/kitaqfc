typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Pair{u8 a;u16 b;};u16 add(struct Pair p,u8 n){p.a+=n;return p.b+p.a;}void main(void){struct Pair p;u16 n;p.a=3;p.b=0x1234;n=add(p,5);result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
