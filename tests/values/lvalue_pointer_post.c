typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
struct Ops{u16*p;};u16 data[2];struct Ops ops;void main(void){u16*p;data[0]=3;data[1]=7;ops.p=data;p=ops.p++;result=(u8)*p;result_hi=(u8)*ops.p;while(1){}}
