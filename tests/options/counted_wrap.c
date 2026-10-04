typedef unsigned char u8;typedef unsigned short u16;
__location(0x0700) u8 result;__location(0x0701) u8 result_hi;
void main(void){u8 i;u16 n=0;for(i=0;i<256;i++){n++;}result=(u8)n;result_hi=(u8)(n>>8);while(1){}}
