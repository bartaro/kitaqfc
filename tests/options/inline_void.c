typedef unsigned char u8;typedef unsigned short u16;
__location(0x0700) u8 result;__location(0x0701) u8 result_hi;
void set(u8 n){result=n;}void main(void){set(42);while(1){}}
