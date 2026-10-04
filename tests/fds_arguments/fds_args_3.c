typedef unsigned char u8;typedef unsigned short u16;
#pragma bank 2
u16 add(u8 a,u8 b){return a+b;}
#pragma bank 1
void main(void){u16 n=add(17,25);__asm { LDA n
 STA $0700
 }while(1){}}
