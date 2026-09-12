// 内蔵APUの効果音
// Expected: 042 and a pulse tone
#include "fc_common.h"
#include "audio.h"

void main(void) {
    m_init();
    m_text(2,3,"SOUND");
    nes_apu_init(); nes_sfx_square1(0xBF,400,20);
    m_number(42);
    while (1) { m_wait();  }
}
