// Play a pulse-channel effect on the built-in NES APU.
// Expected: 042 and a pulse tone
#include "fc_common.h"
#include "audio.h"

// Initialize the APU, trigger pulse channel 1 with the chosen timer/length
// values, and leave the screen displaying 42 while the hardware plays the tone.
void main(void) {
    m_init();
    m_text(2,3,"SOUND");
    // Control BF selects constant volume and halts the length counter, so this tone sustains.
    // Call nes_apu_silence_all when the game should stop it; 20 is a length-table index, not frames.
    nes_apu_init(); nes_sfx_square1(0xBF,400,20);
    m_number(42);
    while (1) { m_wait();  }
}
