#include "lib/fc.h"

const u8 player_meta[] = {
    0, 0, 1, 0,
    8, 0, 2, 0,
    0xFF
};

#pragma bank 0
void __nes_nmi(void)
{
    __vramq_exec();
    __oam_dma();
}

#pragma bank 1
void main(void)
{
    u8 next;

    __ppu_off();
    __vramq_clear();
    nes_vram_put(0x2000, 0x24);
    nes_vram_commit();

    __oam_clear();
    next = __metasprite_draw(0, 40, 80, player_meta);
    __sprite_hide(next);

    __mapper_irq_set(96);
    __mapper_irq_enable();
    __split_scroll_sprite0(0, 72);
}
