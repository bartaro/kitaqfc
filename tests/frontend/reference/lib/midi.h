#ifndef MIDI_H
#define MIDI_H
#include "intrinsics.h"
void __serial_tx_bit(u8 bit);
u8 __serial_rx_bit(void);
void __midi_out_byte(u8 byte);
u8 __midi_in_byte(void);
void __midi_note_on(u8 ch, u8 note, u8 velocity);
void __midi_note_off(u8 ch, u8 note, u8 velocity);
void __midi_control_change(u8 ch, u8 cc, u8 value);
void __midi_program_change(u8 ch, u8 program);
void __midi_clock(void);
void __midi_start(void);
void __midi_continue(void);
void __midi_stop(void);
#endif
