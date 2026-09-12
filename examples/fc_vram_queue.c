// NMIでVRAM更新
// Expected: 4 at (3,8)
#include "fc_common.h"


void main(void) {
    m_init();
    m_text(2,3,"VRAM QUEUE");
    __vramq_put(0x2103,52); __vramq_commit();
    while (1) { m_wait();  }
}
