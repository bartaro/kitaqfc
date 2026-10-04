typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Pair{u8 a;u16 b;};const struct Pair p[1]={{3,0x1234}};void main(void){result=p[0].a;result_hi=(u8)(p[0].b>>8);while(1){}}
