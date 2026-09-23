// Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT
#include "audio_vblank.h"
#pragma fixed_bank 0

// Eight physical slots, one always empty, distinguish full from empty using
// byte offsets 0,5,...,35. Only foreground code writes head; NMI writes tail.
u8 nav_queue[40];
u8 nav_head;u8 nav_tail;u8 nav_active;u8 nav_delay;
u8 nav_enabled;u8 nav_starved;u8 nav_underruns;u8 nav_invalid;
u8 nav_timbre[4];
const u8* nav_source;const u8* nav_cursor;
u16 nav_count;u16 nav_left;u8 nav_loop;
// Pausing and the independent, bounded SFX stream share the NMI note writer.
// Foreground publishes nav_sfx_active last; the NMI owns cursor and delay.
u8 nav_paused;u8 nav_refresh;u8 nav_dirty;u8 nav_effective_mask;u8 nav_previous_mask;
u8 nav_notes[4];u8 nav_sfx_notes[4];u8 nav_sfx_queue[35];
u8 nav_sfx_active;u8 nav_sfx_mask;u8 nav_sfx_cursor;u8 nav_sfx_length;u8 nav_sfx_delay;
__prg_rom const u8 nav_bits[4]={1,2,4,8};
__prg_rom const u8 nav_end[5]={0,255,255,255,255};
__location(0x4015) u8 nav_status;
__location(0x4017) u8 nav_frame;
__location(0x4001) u8 nav_sweep1;
__location(0x4005) u8 nav_sweep2;
__location(0x4010) u8 nav_dmc_flags;
__location(0x4011) u8 nav_dmc_level;

// Publish inactive before changing queue state. An NMI that occurs afterwards
// exits without reading partially reset producer or consumer fields.
void nes_audio_vblank_stop(void){
    u8 i;
    nav_paused=1;nav_active=0;nav_sfx_active=0;nav_status=0;
    nav_delay=0;nav_enabled=0;nav_head=0;nav_tail=0;
    nav_source=0;nav_cursor=0;nav_count=0;nav_left=0;nav_starved=0;
    nav_previous_mask=0;nav_effective_mask=0;nav_refresh=0;
    for(i=0;i<4;i++){nav_notes[(__safe_index u8)i]=254;nav_sfx_notes[(__safe_index u8)i]=254;}
    nav_paused=0;
}
// Preserve queue, remaining note delay, feeder and SFX position. Resume
// requests a full note restore because $4015 clears hardware length counters.
void nes_audio_vblank_pause(u8 on){
    if(on!=0){nav_paused=1;nav_status=0;}
    else{nav_refresh=1;nav_paused=0;}
}
void nes_audio_vblank_stop_sfx(void){nav_sfx_active=0;}
// Up to seven complete records are copied before publishing the effect.
// Only selected channels override BGM; the BGM timeline keeps advancing.
u8 nes_audio_vblank_play_sfx(const u8* records,u8 count,u8 mask){
    u8 i;u8 ch;u8 value;u8 bytes;
    if(records==0||count==0||count>7||(mask&15)==0)return 0;
    bytes=(u8)(count+count+count+count+count);
    i=0;
    while(i<bytes){
        if(records[i]==0)return 0;
        ch=1;while(ch<5){
            value=records[i+ch];
            if(value<254){if(ch<4){if(value>71)return 0;}else{if(value>31)return 0;}}
            ch++;
        }
        i=i+5;
    }
    nav_sfx_active=0;
    for(i=0;i<bytes;i++)nav_sfx_queue[(__safe_index u8)i]=records[i];
    for(i=0;i<4;i++)nav_sfx_notes[(__safe_index u8)i]=254;
    nav_sfx_mask=mask&15;nav_sfx_cursor=0;nav_sfx_length=bytes;nav_sfx_delay=0;
    nav_sfx_active=1;return 1;
}
void nes_audio_vblank_init(void){
    nes_audio_vblank_stop();nav_underruns=0;nav_frame=0x40;
    nav_sweep1=8;nav_sweep2=8;nav_dmc_flags=0;nav_dmc_level=0;
    nav_timbre[0]=0xBC;nav_timbre[1]=0x7A;nav_timbre[2]=0xFF;nav_timbre[3]=0x38;
}

// Validate before claiming a slot. A bad record leaves the queue unchanged.
u8 nes_audio_vblank_enqueue(const u8* record){
    u8 next;u8 i;u8 value;
    nav_invalid=0;
    if(record==0){nav_invalid=1;return 0;}
    next=nav_head+5;if(next==40)next=0;
    if(next==nav_tail)return 0;
    if(record[0]!=0){
        for(i=1;i<5;i++){
            value=record[i];
            if(value<254){
                if(i<4){if(value>71){nav_invalid=1;return 0;}}
                else{if(value>31){nav_invalid=1;return 0;}}
            }
        }
    }
    for(i=0;i<5;i++)nav_queue[(__safe_index u8)(nav_head+i)]=record[i];
    nav_head=next;return 1;
}
// Tail may advance between reads; head is stable in the foreground producer.
// The result is a momentary occupancy, not a reservation of future slots.
u8 nes_audio_vblank_queued(void){
    u8 head;u8 tail;head=nav_head;tail=nav_tail;
    if(head<tail)head=head+40;
    return (head-tail)/5;
}
u8 nes_audio_vblank_free(void){return 7-nes_audio_vblank_queued();}
void nes_audio_vblank_start(void){nav_active=1;}
u8 nes_audio_vblank_is_playing(void){return nav_active;}
u8 nes_audio_vblank_underruns(void){return nav_underruns;}
u8 nes_audio_vblank_set_timbre(u8 channel,u8 control){
    if(channel>=4)return 0;
    if(channel<2)nav_timbre[(__safe_index u8)channel]=(control&0xCF)|0x30;
    else if(channel==2){if(control==0)nav_timbre[2]=0x80;else nav_timbre[2]=0xFF;}
    else {if((control&128)!=0)nav_timbre[3]=control&15;else nav_timbre[3]=(control&15)|0x30;}
    return 1;
}
u8 nes_audio_vblank_set_music(const u8* records,u16 count,u8 loop){
    nes_audio_vblank_stop();
    if(records==0||count==0||count>13107)return 0;
    nav_source=records;nav_cursor=records;nav_count=count;nav_left=count;nav_loop=loop;
    return 1;
}
// Work is bounded even for a looping song. Copying happens outside NMI so no
// bank switch, variable-length parse or shared compiler arithmetic runs there.
u8 nes_audio_vblank_refill(void){
    u8 copied;copied=0;
    while(copied<7&&nav_source!=0){
        if(nav_left==0){
            if(nav_loop!=0){nav_cursor=nav_source;nav_left=nav_count;}
            else {if(nes_audio_vblank_enqueue(nav_end)!=0){copied++;nav_source=0;}return copied;}
        }
        if(nes_audio_vblank_enqueue(nav_cursor)==0){
            // The producer-only flag distinguishes malformed data from full.
            // Re-reading occupancy here would race NMI freeing a full queue.
            if(nav_invalid!=0){nes_audio_vblank_stop();return 0;}
            return copied;
        }
        copied++;nav_cursor=nav_cursor+5;nav_left--;
    }
    return copied;
}
u8 nes_audio_vblank_play_music(const u8* records,u16 count,u8 loop){
    if(nes_audio_vblank_set_music(records,count,loop)==0)return 0;
    if(nes_audio_vblank_refill()==0)return 0;
    nes_audio_vblank_start();return 1;
}

// Rounded NTSC timer values: CPU/(16*f)-1 for pulse, CPU/(32*f)-1
// for triangle; f=440*2^((MIDI_note-69)/12), MIDI notes 36..107.
__prg_rom const u8 nav_pulse_lo[72]={173,77,243,157,76,0,184,116,52,248,191,137,86,38,249,206,166,128,92,58,26,251,223,196,171,147,124,103,82,63,45,28,12,253,239,225,213,201,189,179,169,159,150,142,134,126,119,112,106,100,94,89,84,79,75,70,66,63,59,56,52,49,47,44,41,39,37,35,33,31,29,27};
__prg_rom const u8 nav_pulse_hi[72]={6,6,5,5,5,5,4,4,4,3,3,3,3,3,2,2,2,2,2,2,2,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0};
__prg_rom const u8 nav_triangle_lo[72]={86,38,249,206,166,128,92,58,26,251,223,196,171,147,124,103,82,63,45,28,12,253,239,225,213,201,189,179,169,159,150,142,134,126,119,112,106,100,94,89,84,79,75,70,66,63,59,56,52,49,47,44,41,39,37,35,33,31,29,27,26,24,23,21,20,19,18,17,16,15,14,13};
__prg_rom const u8 nav_triangle_hi[72]={3,3,2,2,2,2,2,2,2,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0};

// Called once per NMI. No compiler scratch bytes, bank changes or foreground
// parser calls occur here. BGM and SFX feed the same four hardware writers.
void __nes_audio_vblank_tick(void){
    __asm {
        PHA
        TXA
        PHA
        TYA
        PHA
        LDA nav_paused
        BEQ nav_tick_begin
        JMP nav_tick_exit
nav_tick_begin:
        LDA #0
        STA nav_dirty
        LDA nav_refresh
        BEQ nav_tick_music
        LDA #0
        STA nav_refresh
        LDA #15
        STA nav_dirty
// Advance BGM even when an effect temporarily owns its output channel.
nav_tick_music:
        LDA nav_active
        BNE nav_tick_active
        JMP nav_tick_sfx
nav_tick_active:
        LDA nav_delay
        BEQ nav_tick_next
        DEC nav_delay
        BEQ nav_tick_next
        JMP nav_tick_sfx
nav_tick_next:
        LDX nav_tail
        CPX nav_head
        BNE nav_tick_record
        LDA nav_starved
        BNE nav_tick_empty_done
        INC nav_starved
        LDA nav_underruns
        CMP #255
        BEQ nav_tick_empty_done
        INC nav_underruns
nav_tick_empty_done:
        JSR nav_silence_music
        JMP nav_tick_sfx
nav_tick_record:
        LDA #0
        STA nav_starved
        LDA nav_queue,X
        BNE nav_tick_timed
        STA nav_active
        JSR nav_silence_music
        JMP nav_tick_release
nav_tick_timed:
        STA nav_delay
        LDY #0
// Remember the new BGM notes but do not retrigger channels owned by SFX.
nav_tick_copy:
        INX
        LDA nav_queue,X
        CMP #255
        BEQ nav_tick_copy_next
        STA nav_notes,Y
        LDA nav_bits,Y
        AND nav_previous_mask
        BNE nav_tick_copy_next
        LDA nav_bits,Y
        ORA nav_dirty
        STA nav_dirty
nav_tick_copy_next:
        INY
        CPY #4
        BNE nav_tick_copy
nav_tick_release:
        LDA nav_tail
        CLC
        ADC #5
        CMP #40
        BNE nav_tick_publish
        LDA #0
nav_tick_publish:
        STA nav_tail
// The SFX has its own delay; completion releases its mask automatically.
nav_tick_sfx:
        LDA nav_sfx_active
        BEQ nav_tick_select
        LDA nav_sfx_delay
        BEQ nav_sfx_next
        DEC nav_sfx_delay
        BNE nav_tick_select
nav_sfx_next:
        LDX nav_sfx_cursor
        CPX nav_sfx_length
        BCC nav_sfx_record
        LDA #0
        STA nav_sfx_active
        JMP nav_tick_select
nav_sfx_record:
        LDA nav_sfx_queue,X
        STA nav_sfx_delay
        LDY #0
nav_sfx_copy:
        INX
        LDA nav_sfx_queue,X
        CMP #255
        BEQ nav_sfx_copy_next
        STA nav_sfx_notes,Y
        LDA nav_bits,Y
        AND nav_sfx_mask
        ORA nav_dirty
        STA nav_dirty
nav_sfx_copy_next:
        INY
        CPY #4
        BNE nav_sfx_copy
        INX
        STX nav_sfx_cursor
nav_tick_select:
        LDA #0
        STA nav_effective_mask
        LDA nav_sfx_active
        BEQ nav_tick_restore
        LDA nav_sfx_mask
        STA nav_effective_mask
// A changed ownership bit forces the newly selected note onto the APU.
nav_tick_restore:
        LDA nav_effective_mask
        EOR nav_previous_mask
        ORA nav_dirty
        STA nav_dirty
        LDA nav_effective_mask
        STA nav_previous_mask
        LDA nav_dirty
        AND #1
        BNE nav_apply_0
        JMP nav_done_0
nav_apply_0:
        LDA nav_effective_mask
        AND #1
        BEQ nav_bg_0
        LDA nav_sfx_notes+0
        JMP nav_value_0
nav_bg_0:
        LDA nav_notes+0
nav_value_0:
        CMP #254
        BNE nav_note_0
        LDA nav_enabled
        AND #254
        STA nav_enabled
        STA $4015
        JMP nav_done_0
nav_note_0:
        TAY
        LDA nav_enabled
        ORA #1
        STA nav_enabled
        STA $4015
        LDA nav_timbre+0
        STA $4000
        LDA nav_pulse_lo,Y
        STA $4002
        LDA nav_pulse_hi,Y
        STA $4003
nav_done_0:
        LDA nav_dirty
        AND #2
        BNE nav_apply_1
        JMP nav_done_1
nav_apply_1:
        LDA nav_effective_mask
        AND #2
        BEQ nav_bg_1
        LDA nav_sfx_notes+1
        JMP nav_value_1
nav_bg_1:
        LDA nav_notes+1
nav_value_1:
        CMP #254
        BNE nav_note_1
        LDA nav_enabled
        AND #253
        STA nav_enabled
        STA $4015
        JMP nav_done_1
nav_note_1:
        TAY
        LDA nav_enabled
        ORA #2
        STA nav_enabled
        STA $4015
        LDA nav_timbre+1
        STA $4004
        LDA nav_pulse_lo,Y
        STA $4006
        LDA nav_pulse_hi,Y
        STA $4007
nav_done_1:
        LDA nav_dirty
        AND #4
        BNE nav_apply_2
        JMP nav_done_2
nav_apply_2:
        LDA nav_effective_mask
        AND #4
        BEQ nav_bg_2
        LDA nav_sfx_notes+2
        JMP nav_value_2
nav_bg_2:
        LDA nav_notes+2
nav_value_2:
        CMP #254
        BNE nav_note_2
        LDA nav_enabled
        AND #251
        STA nav_enabled
        STA $4015
        JMP nav_done_2
nav_note_2:
        TAY
        LDA nav_enabled
        ORA #4
        STA nav_enabled
        STA $4015
        LDA nav_timbre+2
        STA $4008
        LDA nav_triangle_lo,Y
        STA $400A
        LDA nav_triangle_hi,Y
        STA $400B
nav_done_2:
        LDA nav_dirty
        AND #8
        BNE nav_apply_3
        JMP nav_done_3
nav_apply_3:
        LDA nav_effective_mask
        AND #8
        BEQ nav_bg_3
        LDA nav_sfx_notes+3
        JMP nav_value_3
nav_bg_3:
        LDA nav_notes+3
nav_value_3:
        CMP #254
        BNE nav_note_3
        LDA nav_enabled
        AND #247
        STA nav_enabled
        STA $4015
        JMP nav_done_3
nav_note_3:
        TAY
        LDA nav_enabled
        ORA #8
        STA nav_enabled
        STA $4015
        LDA nav_timbre+3
        STA $400C
        TYA
        AND #16
        BEQ nav_noise_long
        TYA
        AND #15
        ORA #128
        JMP nav_noise_period
nav_noise_long:
        TYA
nav_noise_period:
        STA $400E
        // A one-shot envelope must outlive its slowest 16-step decay.
        // Length index 1 gives 254 half-frame ticks; index 0 cuts it at 10.
        LDA nav_timbre+3
        AND #16
        BNE nav_noise_constant_length
        LDA #8
        JMP nav_noise_trigger
nav_noise_constant_length:
        LDA #0
nav_noise_trigger:
        STA $400F
nav_done_3:
nav_tick_exit:
        PLA
        TAY
        PLA
        TAX
        PLA
        RTS
// Internal subroutine: stop BGM channels without disturbing an active effect.
// End/underrun forgets the BGM notes without cutting an independent SFX.
nav_silence_music:
        LDA #254
        STA nav_notes+0
        STA nav_notes+1
        STA nav_notes+2
        STA nav_notes+3
        LDA nav_previous_mask
        EOR #15
        ORA nav_dirty
        STA nav_dirty
        RTS
    }
}
