#include "debug.h"

// debug_init must initialize this shared zero-terminated label before assertion entries are inspected.
static u8 debug_assert_label[7];

// Shared trace storage has no interrupt locking; serialize writers and readers as required by the game.
DebugTraceEntry kq_debug_trace_log[DEBUG_TRACE_MAX];
u8 kq_debug_trace_count;
u16 kq_debug_frame;
u16 kq_debug_last_assert;

// Build the RAM-resident ASSERT label and reset log length, frame tag and last
// assertion code. Old log storage bytes are not erased.
void debug_init(void)
{
    debug_assert_label[0] = 'A';
    debug_assert_label[1] = 'S';
    debug_assert_label[2] = 'S';
    debug_assert_label[3] = 'E';
    debug_assert_label[4] = 'R';
    debug_assert_label[5] = 'T';
    debug_assert_label[6] = 0;
    kq_debug_trace_count = 0;
    kq_debug_frame = 0;
    kq_debug_last_assert = 0;
}

// Choose the frame tag stored in subsequent debug entries.
void debug_set_frame(u16 frame)
{
    kq_debug_frame = frame;
}

// Append a named value/frame entry while capacity remains, otherwise drop it.
// The name pointer is retained, so its string must outlive inspection of the log.
static void debug_push(const u8* name, u16 value)
{
    DebugTraceEntry* entry;
    if (kq_debug_trace_count >= DEBUG_TRACE_MAX) return;
    entry = &kq_debug_trace_log[(__safe_index u8)kq_debug_trace_count];
    entry->name = name;
    entry->value = value;
    entry->frame = kq_debug_frame;
    kq_debug_trace_count = (u8)(kq_debug_trace_count + 1);
}

// Append a named marker with the reserved marker value 0xFFFF.
void debug_mark_frame(const u8* label)
{
    debug_push(label, 0xFFFF);
}

// Widen a byte value into the trace entry's 16-bit value field.
void debug_trace_u8(const u8* name, u8 value)
{
    debug_push(name, (u16)value);
}

// Append a 16-bit value with the current frame tag.
void debug_trace_u16(const u8* name, u16 value)
{
    debug_push(name, value);
}

// Store the assertion code even if the log is full, then attempt an ASSERT
// entry. The helper records failure but does not stop execution.
void debug_assert_fail(u16 code)
{
    kq_debug_last_assert = code;
    debug_push(debug_assert_label, code);
}

// Read the number of valid entries in the fixed trace buffer.
u8 debug_get_trace_count(void)
{
    return kq_debug_trace_count;
}

// Borrow the internal trace array; bound inspection by debug_get_trace_count.
const DebugTraceEntry* debug_get_trace_log(void)
{
    return kq_debug_trace_log;
}

// Read the latest assertion code, initially zero after debug_init.
u16 debug_get_last_assert(void)
{
    return kq_debug_last_assert;
}
