#ifndef DEBUG_H
#define DEBUG_H

#include "core.h"

// Choose 1..255 consistently across units; the valid-entry count is a byte.
#ifndef DEBUG_TRACE_MAX
#define DEBUG_TRACE_MAX 32
#endif

// Names are retained pointers, not copied strings; keep their storage and ROM mapping readable.
typedef __packed struct DebugTraceEntry {
    const u8* name;
    u16 value;
    u16 frame;
} DebugTraceEntry;

extern DebugTraceEntry kq_debug_trace_log[DEBUG_TRACE_MAX];
extern u8 kq_debug_trace_count;
extern u16 kq_debug_frame;
extern u16 kq_debug_last_assert;

// Build the RAM-resident ASSERT label and reset log length, frame tag and last
// assertion code. Old log storage bytes are not erased.
void debug_init(void);
// Choose the frame tag stored in subsequent debug entries.
void debug_set_frame(u16 frame);
// Append a named marker with the reserved marker value 0xFFFF.
void debug_mark_frame(const u8* label);
// Widen a byte value into the trace entry's 16-bit value field.
void debug_trace_u8(const u8* name, u8 value);
// Append a 16-bit value with the current frame tag.
void debug_trace_u16(const u8* name, u16 value);
// Store the assertion code even if the log is full, then attempt an ASSERT
// entry. The helper records failure but does not stop execution.
void debug_assert_fail(u16 code);
// Read the number of valid entries in the fixed trace buffer.
u8 debug_get_trace_count(void);
// Borrow the internal trace array; bound inspection by debug_get_trace_count.
const DebugTraceEntry* debug_get_trace_log(void);
// Read the latest assertion code, initially zero after debug_init.
u16 debug_get_last_assert(void);

#define KITAQGB_TRACE_U8(name, value) debug_trace_u8((const u8*)(name), (u8)(value))
#define KITAQGB_TRACE_U16(name, value) debug_trace_u16((const u8*)(name), (u16)(value))
#define KITAQGB_TRACE(name, value) debug_trace_u16((const u8*)(name), (u16)(value))
#define KITAQGB_MARK_FRAME(label) debug_mark_frame((const u8*)(label))
#define KITAQGB_ASSERT(cond) do { if (!(cond)) debug_assert_fail((u16)1); } while (0)
#define KITAQGB_ASSERT_CODE(cond, code) do { if (!(cond)) debug_assert_fail((u16)(code)); } while (0)

#endif
