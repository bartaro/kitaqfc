typedef unsigned char u8;typedef unsigned short u16;
__location(0x0700) u8 result;__location(0x0701) u8 result_hi;
u8 unused(void){return 9;}u8 inc(void){return 42;}void main(void){__asm { JSR inc
STA $0700
 } while(1){}}
