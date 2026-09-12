// ビット操作・メモリ転送
// Expected: 010
#include "fc_common.h"


void main(void) {
    m_init();
    m_text(2,3,"BITS MEMORY");
    u8 data[4]; u8 copy[4];
    __memset(data,0,4); __bit_set(data,3); __bit_toggle(data,1);
    __memcpy(copy,data,4); m_number(copy[0]);
    while (1) { m_wait();  }
}
