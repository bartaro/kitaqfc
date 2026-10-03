void main(void) { __asm { LDA #$80
 STA $0600
 ASL A
 BNE ready
 JMP ($0080)
ready:
 RTS
} }
