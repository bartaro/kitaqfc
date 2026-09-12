// 最初の画面表示
// Expected: HELLO WORLD and 042
#include "fc_common.h"


void main(void) {
    m_init();
    m_text(2,3,"HELLO");
    m_text(2,5,"HELLO WORLD");
    m_number(42);
    while (1) { m_wait();  }
}
