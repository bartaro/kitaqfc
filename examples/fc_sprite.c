// NESのOAM
// Expected: A sprite on screen
#include "fc_common.h"


void main(void) {
    m_init();
    m_text(2,3,"SPRITE");
    __palette_sp_load(manual_pal); __sprite_set(0,70,80,65,0);
    __ppu_mask_set(0x1E);
    while (1) { m_wait(); __oam_dma(); }
}
