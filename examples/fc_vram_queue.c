// Submit a VRAM update for the NMI handler.
// Expected: 4 at (3,8)
#include "fc_common.h"


// Queue tile 52 at nametable address 0x2103, corresponding to (3,8), and
// commit the queue so the NMI update path displays the digit 4.
void main(void) {
    m_init();
    m_text(2,3,"VRAM QUEUE");
    __vramq_put(0x2103,52); __vramq_commit();
    while (1) { m_wait();  }
}
