// 条件分岐と繰り返し
// Expected: 042
#include "fc_common.h"


void main(void) {
    m_init();
    m_text(2,3,"CONTROL");
    u8 i; u8 sum; sum=0;
    for (i=0;i<8;i++) { if (i==3) continue; sum=(u8)(sum+i); }
    while (sum<32) { sum++; }
    if (sum==32) sum=42; else sum=0;
    m_number(sum);
    while (1) { m_wait();  }
}
