typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
extern u8 __kq_vramq_buf;void main(void){result=(u8)sizeof(__kq_vramq_buf);while(1){}}
