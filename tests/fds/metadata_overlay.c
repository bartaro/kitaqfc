#pragma bank 2
void overlay(void) { __asm { LDA #$5A
 STA $0701
 } }
#pragma bank 1
void main(void) { overlay(); while (1) { } }
