typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Pair{u8 a;u16 b;};struct Pair make(u8 a,u16 b){struct Pair p;p.a=a;p.b=b;return p;}void main(void){struct Pair p;p=make(3,0x1234);result=p.a;result_hi=(u8)(p.b>>8);while(1){}}
