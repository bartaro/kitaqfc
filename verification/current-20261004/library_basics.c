#include "fc.h"
__location(0x0700) u8 proof[16];
void main(void) {
    u16 a;
    s16 b;
    a=257;
    b=-3;
    proof[0]=(u8)(a+b);
    proof[1]=(u8)sizeof(KQBody3D);
    proof[2]=NES_AUDIO_QUEUE_CAPACITY;
    proof[3]=NES_AUDIO_RECORD_BYTES;
    proof[4]=NES_AUDIO_HOLD;
    proof[5]=NES_AUDIO_STOP;
    proof[6]=0xA5;
    while (1) { }
}
