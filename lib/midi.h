#ifndef MIDI_H
#define MIDI_H
#include "intrinsics.h"
// Write bit & 1 to 4016, replacing the port output value; coordinate with other port users.
void __serial_tx_bit(u8 bit);
// Read 4017 bit 4 and normalize it to 0 or 1; an external serial adapter supplies the signal.
u8 __serial_rx_bit(void);
// Send start-low, eight least-significant-first bits and stop-high using fixed NOP delays.
// Instruction and interrupt overhead is additional; the emitted loop does not establish a verified MIDI baud rate.
void __midi_out_byte(u8 byte);
// Block for a low start signal and sample eight bits. No timeout, framing check or receive buffer is provided.
u8 __midi_in_byte(void);
// Send 90|(ch&0F), note and velocity. Supply 7-bit data bytes; they are not masked by this helper.
void __midi_note_on(u8 ch, u8 note, u8 velocity);
// Send 80|(ch&0F), note and velocity through the same synchronous serial path.
void __midi_note_off(u8 ch, u8 note, u8 velocity);
// Send B0|(ch&0F), controller and value; both data arguments must be valid 7-bit values.
void __midi_control_change(u8 ch, u8 cc, u8 value);
// Send C0|(ch&0F) followed by the caller-supplied 7-bit program value.
void __midi_program_change(u8 ch, u8 program);
// Transmit the single realtime byte F8 synchronously; this does not schedule future ticks.
void __midi_clock(void);
// Transmit realtime Start (FA) through the serial adapter path.
void __midi_start(void);
// Transmit realtime Continue (FB) through the serial adapter path.
void __midi_continue(void);
// Transmit realtime Stop (FC) through the serial adapter path.
void __midi_stop(void);
#endif
