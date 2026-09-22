// Three measured poses use the same body update as the GB sample.
#include "fc_common.h"
#include "chain.c"
__location(0x0600) u8 scene_result[26];
__prg_rom u8 joint_palette[16]={15,48,16,22,15,48,16,18,15,48,16,26,15,48,16,48};
void begin_scene() {
    m_init();__ppu_off();__oam_clear();
    __palette_sp_load(joint_palette);
    m_text(0,0,"JOINT FOLLOWING");m_wait();
    m_text(0,2,"STRAIGHT");m_text(0,7,"TURN: TICK 1");m_wait();
    m_text(0,11,"TURN: TICK 4");m_wait();
    m_text(0,17,"HEAD=FRONT TAIL=END");m_wait();
}
void dot(u8 id,u8 x,u8 y,u8 palette){__sprite_set(id,x,(u8)(y-1),128,palette);}
void finish_scene(){__nmi_wait();__oam_dma();__scroll_set(0,0);__ppu_mask_set(0x1E);}

ChainBody body;
u8 joint_x[8];u8 joint_y[8];u8 joint_heading[8];
// Coordinates are logical pixels. The diagram magnifies them two times.
void show_pose(u8 first, u8 offset_y) {
    u8 i;u8 palette;
    for(i=0;i<body.count;i++) {
        palette=1;
        if(i==0)palette=0;
        else if(i==7)palette=2;
        dot((u8)(first+i),(u8)(joint_x[i]*2),(u8)(joint_y[i]*2+offset_y),palette);
    }
}
void main() {
    u8 step;u8 i;
    begin_scene();
    // example:chain_body_init:start
    chain_body_init(&body,joint_x,joint_y,joint_heading,8,160,144);
    // Eight caller-owned slots, including the head, on a wrapped 160x144 field.
    // example:chain_body_init:end
    // example:chain_body_reset:start
    chain_body_reset(&body,7,48,16,0); // Seven joints, facing right.
    // example:chain_body_reset:end
    // example:chain_body_grow:start
    chain_body_grow(&body); // Copy the tail pose into the eighth slot.
    chain_body_step(&body,48,16,0);
    chain_body_step(&body,48,16,0);
    chain_body_step(&body,48,16,0);
    chain_body_step(&body,48,16,0); // Allow the duplicated tail to separate.
    // example:chain_body_grow:end
    show_pose(0,8);
    // example:chain_body_step:start
    chain_body_step(&body,48,20,4); // The head turns down; only joint 1 receives 4.
    show_pose(8,34);
    for(step=1;step<4;step++) {
        chain_body_step(&body,48,(u8)(20+step),4);
    }
    // After four ticks, the new heading has reached joints 1 through 4.
    // example:chain_body_step:end
    show_pose(16,72);
    for(i=0;i<8;i++) {
        scene_result[(u8)(i*3)]=joint_x[i];
        scene_result[(u8)(i*3+1)]=joint_y[i];
        scene_result[(u8)(i*3+2)]=joint_heading[i];
    }
    // example:chain_body_clear:start
    chain_body_clear(&body); // Forget the pose; the three arrays remain owned by us.
    // example:chain_body_clear:end
    scene_result[24]=body.count;
    scene_result[25]=165;
    finish_scene();
    while(1) {}
}
