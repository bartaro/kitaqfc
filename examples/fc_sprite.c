// Display a sprite using NES OAM.
// Expected: A sprite on screen
#include "fc_common.h"


// Load the sprite palette, assign font tile 65 to OAM slot zero, enable
// background/sprite display, and transfer OAM after each frame wait.
void main(void) {
    m_init();
    m_text(2,3,"SPRITE");
    __palette_sp_load(manual_pal); __sprite_set(0,70,80,65,0);
    __ppu_mask_set(0x1E);
    while (1) { m_wait(); __oam_dma(); }
}
