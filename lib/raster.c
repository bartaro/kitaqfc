// Copyright (c) 2026 DAISUKE OBA. MIT License.
#include "raster.h"
u8 nes_raster_lines[8];u8 nes_raster_x[8];u8 nes_raster_y[8];
u8 nes_raster_low[8];u8 nes_raster_delta[8];u8 nes_raster_count;u8 nes_raster_index;
u8 nes_raster_error;u8 nes_raster_applied;
static __prg_rom u8 nes_raster_title[16]={252,252,253,253,254,254,255,255,0,0,1,1,2,2,3,3};
static __prg_rom u8 nes_raster_gate[64]={0,2,5,7,9,11,13,15,17,19,20,21,22,23,24,24,24,24,24,23,22,21,20,19,17,15,13,11,9,7,5,2,0,254,251,249,247,245,243,241,239,237,236,235,234,233,232,232,232,232,232,233,234,235,236,237,239,241,243,245,247,249,251,254};
void Raster_Clear(void) {__mapper_irq_disable();nes_raster_count=0;nes_raster_index=0;nes_raster_error=0;}
void Raster_Init(void) {*(u8*)0x4017=64;Raster_Clear();nes_raster_applied=0;}
void Raster_Disable(void) {Raster_Clear();}
u8 Raster_GetCount(void) {return nes_raster_count;}
u8 Raster_GetLastError(void) {return nes_raster_error;}
u8 Raster_Push(u8 line,u8 x,u8 y) {
 if(line>=240){nes_raster_error=KQ_RASTER_BAD_LY;return nes_raster_error;}
 if(nes_raster_count>=8){nes_raster_error=KQ_RASTER_FULL;return nes_raster_error;}
 if(nes_raster_count!=0&&line<=nes_raster_lines[nes_raster_count-1]){nes_raster_error=KQ_RASTER_UNSORTED;return nes_raster_error;}
 if(nes_raster_count==0&&line!=0){nes_raster_error=KQ_RASTER_BAD_RANGE;return nes_raster_error;}
 nes_raster_lines[nes_raster_count]=line;nes_raster_x[nes_raster_count]=x;nes_raster_y[nes_raster_count]=y;
 nes_raster_low[nes_raster_count]=(u8)(((y&56)<<2)|(x>>3));
 nes_raster_delta[nes_raster_count]=nes_raster_count==0?0:(u8)(line-nes_raster_lines[nes_raster_count-1]-1);
 nes_raster_count++;nes_raster_error=0;return 0;
}
u8 Raster_Commit(void) {
 __nmi_wait();__mapper_irq_disable();nes_raster_index=0;
 if(nes_raster_count==0)return nes_raster_error;
 __scroll_set(nes_raster_x[0],nes_raster_y[0]);
 if(nes_raster_count>1){nes_raster_index=1;__mapper_irq_set(nes_raster_delta[1]);__mapper_irq_enable();__irq_enable();}
 return nes_raster_error;
}
// Raw IRQ path touches no compiler scratch, parameters or C locals.
void __nes_irq(void) {
 __asm {
 PHA
 TXA
 PHA
 TYA
 PHA
 LDA #0
 STA $E000
 LDX nes_raster_index
 CPX nes_raster_count
 BCS nes_raster_irq_done
 LDA $2002
 LDA #0
 STA $2006
 LDA nes_raster_y,X
 STA $2005
 LDA nes_raster_x,X
 STA $2005
 LDA nes_raster_low,X
 STA $2006
 INC nes_raster_applied
 INX
 STX nes_raster_index
 CPX nes_raster_count
 BCS nes_raster_irq_done
 LDA nes_raster_delta,X
 STA $C000
 LDA #0
 STA $C001
 STA $E001
 nes_raster_irq_done:
 PLA
 TAY
 PLA
 TAX
 PLA
 }
}
u8 Raster_BuildParallax2(u8 line,u8 far_x,u8 near_x,u8 y) {Raster_Clear();if(line==0||line>=240)return KQ_RASTER_BAD_RANGE;Raster_Push(0,far_x,y);return Raster_Push(line,near_x,y);}
u8 Raster_BuildHudTop(u8 height,u8 x,u8 y) {Raster_Clear();if(height==0||height>=240)return KQ_RASTER_BAD_RANGE;Raster_Push(0,0,0);return Raster_Push(height,x,y);}
u8 Raster_BuildHudBottom(u8 line,u8 x,u8 y) {Raster_Clear();if(line==0||line>=240)return KQ_RASTER_BAD_RANGE;Raster_Push(0,x,y);return Raster_Push(line,0,0);}
u8 Raster_BuildParallax3(u8 line1,u8 line2,u8 x0,u8 x1,u8 x2,u8 y) {Raster_Clear();if(line1==0||line2>=240||line2<=line1)return KQ_RASTER_BAD_RANGE;Raster_Push(0,x0,y);Raster_Push(line1,x1,y);return Raster_Push(line2,x2,y);}
void Raster_LineXInit(RasterLineX* e,u8 profile,u8 base_x) {e->profile=profile;e->base_x=base_x;e->phase=0;e->phase_step=1;e->frame_divider=1;e->frame_counter=0;}
void Raster_LineXSetProfile(RasterLineX* e,u8 profile) {e->profile=profile;e->phase=0;e->frame_counter=0;}
void Raster_LineXSetSpeed(RasterLineX* e,u8 step,u8 divider) {e->phase_step=step;e->frame_divider=divider==0?1:divider;e->frame_counter=0;}
void Raster_LineXSetPhase(RasterLineX* e,u8 phase) {e->phase=phase;}
u8 Raster_LineXGetOffset(const RasterLineX* e,u8 line) {if(e->profile==1)return (u8)(e->base_x+nes_raster_title[(line+e->phase)&15]);if(e->profile==2)return (u8)(e->base_x+nes_raster_gate[(line+e->phase)&63]);return e->base_x;}
void Raster_LineXBuildBands(RasterLineX* e) {u8 line;Raster_Clear();for(line=0;line<240;line+=30)Raster_Push(line,Raster_LineXGetOffset(e,line),0);e->frame_counter++;if(e->frame_counter>=e->frame_divider){e->frame_counter=0;e->phase=(u8)(e->phase+e->phase_step);}}
