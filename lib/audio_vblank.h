#ifndef NES_AUDIO_VBLANK_H
#define NES_AUDIO_VBLANK_H
#include "core.h"

// Original MIT-licensed NMI sequencer. Channel order follows the physical APU:
// CH1 pulse 1, CH2 pulse 2, CH3 triangle, CH4 noise (indices 0, 1, 2, 3).
// Every record is five bytes: delay, CH1, CH2, CH3, CH4.
// Delay 1..255 holds the new state for that many NMI ticks. Delay 0 ends playback.
// Tonal notes 0..71 are C2..B7, tuned for NTSC. Noise 0..15 selects a long-mode
// period; 16..31 selects short mode. HOLD preserves a channel; STOP silences it.
#define NES_AUDIO_HOLD 255
#define NES_AUDIO_STOP 254
#define NES_AUDIO_RECORD_BYTES 5
#define NES_AUDIO_QUEUE_CAPACITY 7

// Reset the queue, feeder and diagnostic counters; silence and initialize the
// APU. Owns the four basic channels and disables DMC and APU frame IRQs.
void nes_audio_vblank_init(void);
// Copy one complete validated record to RAM and publish it atomically last.
// Return 1 on success, 0 for null/invalid/full. Call from one foreground producer.
u8 nes_audio_vblank_enqueue(const u8* record);
// Return queued records (0..7), excluding the record whose delay is in progress.
u8 nes_audio_vblank_queued(void);
// Return available record slots (0..7), not bytes.
u8 nes_audio_vblank_free(void);
// Enable consumption at the next NMI. Does not enable PPU NMI itself.
void nes_audio_vblank_start(void);
// Disable consumption, silence, discard queued records and clear the feeder.
void nes_audio_vblank_stop(void);
// Stop and attach a readable five-byte-record stream; return 0 for null, zero
// count or a count above 13107. Nonzero loop repeats the stream. Does not start.
u8 nes_audio_vblank_set_music(const u8* records,u16 count,u8 loop);
// Copy at most seven records from the attached stream. Return copies accepted.
// A one-shot feeder appends a delay-zero terminator after the last timed record.
// Keep the stream's ROM bank mapped during this foreground call only.
u8 nes_audio_vblank_refill(void);
// Set the stream, prefill the queue and start. Return 0 for invalid setup/data.
u8 nes_audio_vblank_play_music(const u8* records,u16 count,u8 loop);
// Return 1 while the consumer is enabled, including a recoverable queue underrun.
u8 nes_audio_vblank_is_playing(void);
// Saturating count of starvation episodes, cleared by init; END is not starvation.
u8 nes_audio_vblank_underruns(void);
// Configure the next note: pulse bits 7..6 select duty and 3..0 select volume;
// noise uses volume 3..0; triangle uses zero for silent, nonzero for full level.
// Return 0 for channel >=4. Hardware has no triangle volume control.
u8 nes_audio_vblank_set_timbre(u8 channel,u8 control);
// Compiler NMI hook: linked automatically once per NMI. Do not call manually.
// Saves A/X/Y, uses no compiler scratch bytes and reads only RAM/fixed ROM.
void __nes_audio_vblank_tick(void);
#endif
