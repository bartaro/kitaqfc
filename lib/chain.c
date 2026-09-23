#include "chain.h"

// Attach caller-owned ring storage and start with no recorded points.
// The storage must remain valid for the lifetime of the chain.
void chain_init(Chain* chain, ChainPoint* storage, u8 capacity)
{
    if (chain == 0) return;
    // A nonzero capacity requires writable storage for that many complete ChainPoint records.
    chain->points = storage;
    chain->capacity = capacity;
    chain->count = 0;
    chain->head = 0;
}

// Forget the recorded points without clearing or freeing the backing storage.
void chain_clear(Chain* chain)
{
    if (chain == 0) return;
    chain->count = 0;
    chain->head = 0;
}

// Move the ring head backward and store the newest point. Once full, each push
// replaces the oldest point; zero capacity is a no-op.
void chain_push_head(Chain* chain, s16 x, s16 y)
{
    if (chain == 0) return;
    if (chain->capacity == 0) return;

    if (chain->head == 0) chain->head = (u8)(chain->capacity - 1);
    else chain->head = (u8)(chain->head - 1);

    chain->points[(__safe_index u8)chain->head].x = x;
    chain->points[(__safe_index u8)chain->head].y = y;

    if (chain->count < chain->capacity) chain->count = (u8)(chain->count + 1);
}

// Read a point by age, with index zero denoting the newest point. Return zero
// for an invalid request and clear a non-null output first. Keep head + index
// within the 8-bit range: addition is narrowed before the capacity wrap.
u8 chain_get_segment(const Chain* chain, u8 index, ChainPoint* out)
{
    u8 pos;
    // Do not alias out with the backing ring: it is cleared before the requested point is read.
    if (out != 0) {
        out->x = 0;
        out->y = 0;
    }

    if (out == 0) return 0;
    if (chain == 0) return 0;
    if (chain->capacity == 0) return 0;
    if (index >= chain->count) return 0;

    pos = (u8)(chain->head + index);
    while (pos >= chain->capacity) {
        pos = (u8)(pos - chain->capacity);
    }
    out->x = chain->points[(__safe_index u8)pos].x;
    out->y = chain->points[(__safe_index u8)pos].y;
    return 1;
}

// Return the number of stored points, or zero for a null chain.
u8 chain_get_count(const Chain* chain)
{
    if (chain == 0) return 0;
    return chain->count;
}

// Use the same integer socket geometry as the GB articulated-snake implementation.
// Branches avoid a banked lookup table and floating-point trigonometry.
static s16 chain_body_offset_x(u8 sector)
{
    if (sector == 0 || sector == 1 || sector == 15) return 4;
    if (sector == 2 || sector == 14) return 3;
    if (sector == 3 || sector == 13) return 1;
    if (sector == 4 || sector == 12) return 0;
    if (sector == 5 || sector == 11) return (s16)(0 - 1);
    if (sector == 6 || sector == 10) return (s16)(0 - 3);
    return (s16)(0 - 4);
}

static s16 chain_body_offset_y(u8 sector)
{
    // Rotate the X component by a quarter turn in the clockwise sector system.
    return chain_body_offset_x((u8)((sector + 12) & 15));
}

static u8 chain_body_wrap(s16 value, u16 size)
{
    // Size is validated at initialization, so neither loop can stall at zero.
    // Inspect the sign bit once, then compare nonnegative magnitudes. This
    // gives a compact 6502 branch path while preserving signed wrap behavior.
    while (((u16)value & 0x8000) != 0) value = (s16)(value + (s16)size);
    while ((u16)value >= size) value = (s16)(value - (s16)size);
    return (u8)value;
}

// Return the shortest signed displacement for normalized coordinates.
// Preconditions: size is 1..256 and both coordinates are less than size.
s16 chain_wrap_delta(u8 target, u8 current, u16 size)
{
    s16 delta;
    s16 half;
    delta = (s16)((s16)target - (s16)current);
    half = (s16)(size >> 1);
    // At exactly half a field, retain the sign of the direct displacement.
    if (((u16)delta & 0x8000) != 0) {
        if ((u16)(0 - delta) > (u16)half) delta = (s16)(delta + (s16)size);
    } else if ((u16)delta > (u16)half) delta = (s16)(delta - (s16)size);
    return delta;
}

static s16 chain_body_correction(s16 delta)
{
    if (((u16)delta & 0x8000) != 0) {
        if ((u16)(0 - delta) > 4) return (s16)(0 - 2);
        return (s16)(0 - 1);
    }
    if ((u16)delta > 4) return 2;
    if (delta != 0) return 1;
    return 0;
}

u8 chain_body_init(ChainBody* body, u8* x, u8* y, u8* heading, u8 capacity, u16 width, u16 height)
{
    if (body == 0) return 0;
    body->x = x;
    body->y = y;
    body->heading = heading;
    body->count = 0;
    body->capacity = 0;
    body->width = width;
    body->height = height;
    if (x == 0 || y == 0 || heading == 0 || capacity == 0) return 0;
    if (width == 0 || width > 256 || height == 0 || height > 256) return 0;
    body->capacity = capacity;
    return 1;
}

u8 chain_body_reset(ChainBody* body, u8 count, u8 head_x, u8 head_y, u8 heading)
{
    // Cache caller storage once; following math and old-heading propagation
    // use these local values. Avoid decoding packed struct pointers per joint.
    u8* coord_x;u8* coord_y;u8* directions;u16 width;u16 height;
    u8 i;
    u8 x;
    u8 y;
    s16 ox;
    s16 oy;
    if (body == 0) return 0;
    if (count == 0 || count > body->capacity) return 0;
    coord_x=body->x;coord_y=body->y;directions=body->heading;
    width=body->width;height=body->height;
    heading = (u8)(heading & 15);
    x = chain_body_wrap((s16)head_x, width);
    y = chain_body_wrap((s16)head_y, height);
    ox = chain_body_offset_x(heading);
    oy = chain_body_offset_y(heading);
    i = 0;
    while (i < count) {
        coord_x[(__safe_index u8)i] = x;
        coord_y[(__safe_index u8)i] = y;
        directions[(__safe_index u8)i] = heading;
        x = chain_body_wrap((s16)((s16)x - ox), width);
        y = chain_body_wrap((s16)((s16)y - oy), height);
        i = (u8)(i + 1);
    }
    body->count = count;
    return 1;
}

void chain_body_step(ChainBody* body, u8 head_x, u8 head_y, u8 heading)
{
    // Cache caller storage once; following math and old-heading propagation
    // use these local values. Avoid decoding packed struct pointers per joint.
    u8* coord_x;u8* coord_y;u8* directions;u16 width;u16 height;
    u8 i;
    u8 previous;
    u8 pull;
    u8 old;
    u8 x;
    u8 y;
    u8 tx;
    u8 ty;
    s16 sx;
    s16 sy;
    if (body == 0) return;
    if (body->count == 0) return;
    coord_x=body->x;coord_y=body->y;directions=body->heading;
    width=body->width;height=body->height;
    heading = (u8)(heading & 15);
    coord_x[0] = chain_body_wrap((s16)head_x, width);
    coord_y[0] = chain_body_wrap((s16)head_y, height);
    directions[0] = heading;
    pull = heading;
    i = 1;
    while (i < body->count) {
        previous = (u8)(i - 1);
        old = directions[(__safe_index u8)i];
        x = coord_x[(__safe_index u8)i];
        y = coord_y[(__safe_index u8)i];
        tx = chain_body_wrap((s16)((s16)coord_x[(__safe_index u8)previous] - chain_body_offset_x(pull)), width);
        ty = chain_body_wrap((s16)((s16)coord_y[(__safe_index u8)previous] - chain_body_offset_y(pull)), height);
        sx = chain_body_correction(chain_wrap_delta(tx, x, width));
        sy = chain_body_correction(chain_wrap_delta(ty, y, height));
        directions[(__safe_index u8)i] = pull;
        coord_x[(__safe_index u8)i] = chain_body_wrap((s16)((s16)x + sx), width);
        coord_y[(__safe_index u8)i] = chain_body_wrap((s16)((s16)y + sy), height);
        // Save before overwriting: a bend advances by one joint per update.
        pull = old;
        i = (u8)(i + 1);
    }
}

u8 chain_body_grow(ChainBody* body)
{
    u8 tail;
    u8 next;
    if (body == 0) return 0;
    next = body->count;
    if (next == 0 || next >= body->capacity) return 0;
    tail = (u8)(next - 1);
    body->x[(__safe_index u8)next] = body->x[(__safe_index u8)tail];
    body->y[(__safe_index u8)next] = body->y[(__safe_index u8)tail];
    body->heading[(__safe_index u8)next] = body->heading[(__safe_index u8)tail];
    body->count = (u8)(next + 1);
    return 1;
}

void chain_body_clear(ChainBody* body)
{
    if (body == 0) return;
    body->count = 0;
}
