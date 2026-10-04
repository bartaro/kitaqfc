#pragma bank 1
void helper(void){}void __nes_nmi(void){__asm { JSR helper
 }}void main(void){while(1){}}
