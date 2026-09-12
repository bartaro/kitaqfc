/*
 * Optional declarations for the phase 8 NES pad runtime.
 *
 * Button bit layout in nes_pad1_cur / pressed / released:
 *   bit0=A bit1=B bit2=Select bit3=Start bit4=Up bit5=Down bit6=Left bit7=Right
 */

extern unsigned char nes_pad1_cur;
extern unsigned char nes_pad1_prev;
extern unsigned char nes_pad1_pressed;
extern unsigned char nes_pad1_released;

void nes_pad_poll(void);
