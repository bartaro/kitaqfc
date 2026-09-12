#include "oam_fair.h"
u8 oam_fair_phase;
u8 oam_fair_limit;
u8 oam_fair_tile;
u8 oam_fair_attr;
u8 oam_fair_drawn;
u8 oam_fair_scanned;
void OAM_FairDraw(void) {
    __asm {
        LDA oam_fair_phase
        CLC
        ADC #13
        AND #63
        STA oam_fair_phase
        TAX
        LDY oam_fair_used
        LDA #0
        STA oam_fair_drawn
        STA oam_fair_scanned
    kq_fair_loop:
        LDA oam_fair_active,X
        BEQ kq_fair_next
        LDA oam_fair_drawn
        CMP oam_fair_limit
        BCS kq_fair_done
        CPY #252
        BCS kq_fair_done
        LDA oam_fair_y,X
        SEC
        SBC #4
        STA oam_fair_shadow,Y
        LDA oam_fair_tile
        STA oam_fair_shadow+1,Y
        LDA oam_fair_attr
        STA oam_fair_shadow+2,Y
        LDA oam_fair_x,X
        SEC
        SBC #4
        STA oam_fair_shadow+3,Y
        INY
        INY
        INY
        INY
        INC oam_fair_drawn
    kq_fair_next:
        TXA
        CLC
        ADC #1
        AND #63
        TAX
        CPX oam_fair_phase
        BNE kq_fair_loop
    kq_fair_done:
        STY oam_fair_used
    }
}
