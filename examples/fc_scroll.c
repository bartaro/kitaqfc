// NESの背景スクロール
// Expected: title scrolls horizontally
#include "fc_common.h"

u8 phase;
void main(void) {
    m_init();
    m_text(2,3,"SCROLL");
    __scroll_set(0,0);
    while (1) { m_wait(); phase++; __scroll_set(phase,0); }
}
