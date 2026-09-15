#include "oam_fair.h"
u8 oam_fair_phase;
u8 oam_fair_limit;
u8 oam_fair_tile;
u8 oam_fair_attr;
u8 oam_fair_drawn;
u8 oam_fair_scanned;
// Rotate the starting pool entry by 13 modulo 64, then append active sprites
// until the caller's draw limit or OAM byte cursor limit is reached. Coordinates
// describe 8x8 centers: subtract four for X and five for raw OAM Y. Include once
// and invoke it from the main loop with a four-byte-aligned used cursor.
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
    // Visit each pool entry at most once, preserving already-written OAM entries
    // and sharing the configured tile/attribute values across emitted sprites.
    kq_fair_loop:
        LDA oam_fair_active,X
        BEQ kq_fair_next
        LDA oam_fair_drawn
        CMP oam_fair_limit
        BCS kq_fair_done
        // Reserve the final four-byte slot so advancing this byte cursor cannot wrap to zero.
        CPY #252
        BCS kq_fair_done
        // Convert the 8x8 center to raw OAM coordinates; Y also needs the hardware minus-one bias.
        LDA oam_fair_y,X
        SEC
        SBC #5
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
    // Advance cyclically and stop after returning to this frame's starting entry.
    kq_fair_next:
        TXA
        CLC
        ADC #1
        AND #63
        TAX
        CPX oam_fair_phase
        BNE kq_fair_loop
    // Publish the updated byte cursor for later OAM writers and the VBlank DMA path.
    kq_fair_done:
        STY oam_fair_used
    }
}
