#include "lib/fc.h"

u8 player_x;
u8 player_y;
u8 frame_counter;

u8 add1(u8 v) {
    return v + 1;
}

u8 pair_sum(u8 a, u8 b) {
    return a + b;
}

void tiny_move(void) {
    player_x = add1(player_x);
    player_y = pair_sum(player_y, 1);
}

void update_loop(void) {
    u8 i;
    for (i = 0; i < 16; i++) {
        frame_counter = frame_counter + 1;
    }
}

void main(void) {
    tiny_move();
    update_loop();
    __vramq_put(0x2000, frame_counter);
    __vramq_commit();
}
