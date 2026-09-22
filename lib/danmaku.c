// Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT
// General-purpose fixed-point pool based on this project's original shooter
// kernel. Game-specific scores, charge absorption, damage and assets are absent.
#include "danmaku.h"

__location(0x6000) u8 dm_x[64];
__location(0x6040) u8 dm_y[64];
__location(0x6080) u8 dm_fraction_x[64];
__location(0x60c0) u8 dm_fraction_y[64];
__location(0x6100) u8 dm_vx_integer[64];
__location(0x6140) u8 dm_vx_fraction[64];
__location(0x6180) u8 dm_vy_integer[64];
__location(0x61c0) u8 dm_vy_fraction[64];
__location(0x6200) u8 dm_active[64];
__location(0x6240) u8 dm_grazed[64];
u8 dm_count;
u8 dm_peak;
u8 dm_hit;
u8 dm_graze;
u8 dm_player_x;
u8 dm_player_y;
u8 dm_invulnerable;
u16 dm_spawned;
u16 dm_rejected;
u8 dm_cursor;
u8 dm_draw_phase;
u8 dm_distance_x;
u8 dm_distance_y;
// Rounded sin(2*pi*i/32)*64, generated mathematically; no game data is used.
__prg_rom const s8 Danmaku_Sin[32]={
    0,12,24,36,45,53,59,63,64,63,59,53,45,36,24,12,
    0,-12,-24,-36,-45,-53,-59,-63,-64,-63,-59,-53,-45,-36,-24,-12
};

void danmaku_clear(void) {
    u8 i;
    for(i=0;i<64;i++)dm_active[i]=0;
    dm_count=0;dm_hit=0;dm_graze=0;
}
void danmaku_reset(void) {
    danmaku_clear();
    dm_peak=0;dm_spawned=0;dm_rejected=0;dm_cursor=0;dm_draw_phase=0;
}
u8 danmaku_spawn(u8 x,u8 y,s8 vx,s8 vy) {
    u8 i,n;
    if(x<8 || x>=248 || y<24 || y>=232){dm_rejected++;return 0;}
    i=dm_cursor;
    for(n=0;n<64;n++) {
        if(dm_active[i]==0) {
            dm_x[i]=x;dm_y[i]=y;dm_fraction_x[i]=0;dm_fraction_y[i]=0;
            // Expand signed Q4.4 to a two-byte two's-complement Q8.8 step.
            dm_vx_integer[i]=(u8)((s16)vx>>4);dm_vx_fraction[i]=(u8)((u8)vx<<4);
            dm_vy_integer[i]=(u8)((s16)vy>>4);dm_vy_fraction[i]=(u8)((u8)vy<<4);
            dm_grazed[i]=0;dm_active[i]=1;dm_cursor=(u8)((i+1)&63);
            dm_count++;if(dm_count>dm_peak)dm_peak=dm_count;
            dm_spawned++;return 1;
        }
        i=(u8)((i+1)&63);
    }
    dm_rejected++;return 0;
}
void danmaku_fan(u8 x,u8 y,u8 direction,u8 step,u8 count,u8 speed) {
    u8 i,angle;s16 vx,vy;
    if(count>64)count=64;if(speed>64)speed=64;
    angle=(u8)(direction&31);
    for(i=0;i<count;i++) {
        vx=(s16)Danmaku_Sin[(u8)((angle+8)&31)]*(s16)speed;
        vy=(s16)Danmaku_Sin[angle]*(s16)speed;
        danmaku_spawn(x,y,(s8)(vx/64),(s8)(vy/64));
        angle=(u8)((angle+step)&31);
    }
}
void danmaku_step(void) {
    // Byte arrays let the 6502 move every slot without multiplying a structure
    // index. Carry propagates fractional motion into the signed integer step.
    __asm {
        LDX #0
        STX dm_count
        STX dm_hit
        STX dm_graze
dm_tick_loop:
        LDA dm_active,X
        BNE dm_tick_live
        JMP dm_tick_next
dm_tick_live:
        CLC
        LDA dm_fraction_x,X
        ADC dm_vx_fraction,X
        STA dm_fraction_x,X
        LDA dm_x,X
        ADC dm_vx_integer,X
        STA dm_x,X
        CMP #8
        BCC dm_tick_kill
        CMP #248
        BCS dm_tick_kill
        CLC
        LDA dm_fraction_y,X
        ADC dm_vy_fraction,X
        STA dm_fraction_y,X
        LDA dm_y,X
        ADC dm_vy_integer,X
        STA dm_y,X
        CMP #24
        BCC dm_tick_kill
        CMP #232
        BCS dm_tick_kill
        // Unsigned absolute distances avoid false collisions across screen wrap.
        LDA dm_x,X
        SEC
        SBC dm_player_x
        BCS dm_tick_dx
        EOR #255
        CLC
        ADC #1
dm_tick_dx:
        STA dm_distance_x
        LDA dm_y,X
        SEC
        SBC dm_player_y
        BCS dm_tick_dy
        EOR #255
        CLC
        ADC #1
dm_tick_dy:
        STA dm_distance_y
        CMP #4
        BCS dm_tick_graze
        LDA dm_distance_x
        CMP #4
        BCS dm_tick_graze
        LDA dm_invulnerable
        BNE dm_tick_count
        LDA #1
        STA dm_hit
dm_tick_kill:
        LDA #0
        STA dm_active,X
        JMP dm_tick_next
dm_tick_graze:
        LDA dm_grazed,X
        BNE dm_tick_count
        LDA dm_distance_x
        CMP #12
        BCS dm_tick_count
        LDA dm_distance_y
        CMP #12
        BCS dm_tick_count
        LDA #1
        STA dm_grazed,X
        INC dm_graze
dm_tick_count:
        INC dm_count
dm_tick_next:
        INX
        CPX #64
        BEQ dm_tick_done
        JMP dm_tick_loop
dm_tick_done:
    }
}
u8 danmaku_draw(u8 first_oam,u8 slots,u8 tile,u8 attributes) {
    u8 i,n,drawn;
    if(first_oam>=64)return 0;
    if(slots>(u8)(64-first_oam))slots=(u8)(64-first_oam);
    dm_draw_phase=(u8)((dm_draw_phase+13)&63);i=dm_draw_phase;drawn=0;
    for(n=0;n<64;n++) {
        if(dm_active[i]!=0 && drawn<slots) {
            __sprite_set((u8)(first_oam+drawn),(u8)(dm_x[i]-4),(u8)(dm_y[i]-5),tile,attributes);
            drawn++;
        }
        i=(u8)((i+1)&63);
    }
    for(n=drawn;n<slots;n++)__sprite_hide((u8)(first_oam+n));
    return drawn;
}
