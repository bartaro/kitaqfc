/*
 * Optional declarations for the phase 9 NES pad repeat helper.
 *
 * Button bit layout:
 *   bit0=A bit1=B bit2=Select bit3=Start bit4=Up bit5=Down bit6=Left bit7=Right
 */

extern unsigned char nes_pad_repeat_delay;
extern unsigned char nes_pad_repeat_interval;
extern unsigned char nes_pad1_repeat;

void nes_pad_repeat_config(unsigned char delay, unsigned char interval);
void nes_pad_repeat_step(void);
unsigned char nes_pad_trigger(unsigned char mask);
unsigned char nes_pad_repeat(unsigned char mask);
unsigned char nes_pad_release(unsigned char mask);
unsigned char nes_pad_held(unsigned char mask);
