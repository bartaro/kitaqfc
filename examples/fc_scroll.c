// Scroll the NES background horizontally.
// Expected: title scrolls horizontally
#include "fc_common.h"

u8 phase;
// Advance an 8-bit horizontal scroll position once per frame, wrapping naturally
// after 255. Background motion demonstrates the changing scroll register.
void main(void) {
    m_init();
    m_text(2,3,"SCROLL");
    __scroll_set(0,0);
    while (1) { m_wait(); phase++; __scroll_set(phase,0); }
}
