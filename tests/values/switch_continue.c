typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
void main(void){u8 i;u8 n=0;for(i=0;i<4;i++){switch(i){case 1:continue;case 2:n+=5;break;default:n+=i;}}result=n;while(1){}}
