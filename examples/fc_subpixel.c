// Accumulate Q5.3 eighth-pixel motion into an integer coordinate.
// Expected: 042; explicit game-side integration using header constants
#include "fc_common.h"
#include "physics2d.h"

// Integrate eight increments of 2/8 pixel. Carry whole pixels into X and
// retain the fractional remainder with the supplied mask, moving X from 40 to 42.
void main(void) {
    m_init();
    m_text(2,3,"SUBPIXEL");
    KQ2DPosition x; KQ2DFraction frac; KQ2DSpeed speed; u8 i;
    // This example demonstrates positive motion only; signed negative motion needs a matching carry/borrow path.
    x=40; frac=0; speed=2;
    for(i=0;i<8;i++) {
        frac=(u8)(frac+speed);
        x=(u8)(x+(frac>>KQ2D_SUBPIXEL_BITS));
        frac=(u8)(frac & KQ2D_SUBPIXEL_MASK);
    }
    m_number(x);
    while (1) { m_wait();  }
}
