typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
union Word{u8 lo;u16 word;};union Word p;union Word q;void main(void){p.word=0x1234;q=p;result=q.lo;result_hi=(u8)(q.word>>8);while(1){}}
