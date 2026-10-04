typedef unsigned char u8; typedef signed char s8; typedef unsigned short u16; typedef signed short s16;
__location(0x0700) u8 result; __location(0x0701) u8 result_hi;
typedef struct Vec2 {
    u8 x;
    u8 y;
} Vec2;

typedef struct Rect {
    Vec2 pos;
    u8 w;
    u8 h;
} Rect;

typedef struct BigPacket {
    u8 bytes[18];
} BigPacket;

__location(0x0700) u8 results[16];

Vec2 make_vec2(u8 x, u8 y)
{
    Vec2 out;
    out.x = x;
    out.y = y;
    return out;
}

Vec2 bump_vec2(Vec2 v)
{
    v.x = (u8)(v.x + 1);
    v.y = (u8)(v.y + 2);
    return v;
}

Rect make_rect(u8 x, u8 y)
{
    Rect r;
    r.pos = make_vec2(x, y);
    r.w = 3;
    r.h = 4;
    return r;
}

BigPacket make_big()
{
    BigPacket b;
    b.bytes[0] = 4;
    b.bytes[17] = 8;
    return b;
}

u8 consume_vec2(Vec2 v)
{
    v.x = (u8)(v.x + 10);
    return (u8)(v.x + v.y);
}

void main()
{
    Vec2 a;
    Vec2 b;
    Rect r;
    BigPacket big;
    u16 selfplay_lo;

    a = make_vec2(5, 6);
    results[0] = (u8)(a.x + a.y);

    b = bump_vec2(a);
    results[1] = (u8)(b.x + b.y);
    results[2] = (u8)(a.x + a.y);

    r = make_rect(2, 3);
    results[3] = (u8)(r.pos.x + r.pos.y + r.w + r.h);

    big = make_big();
    results[4] = (u8)(big.bytes[0] + big.bytes[17]);
    results[5] = consume_vec2(make_vec2(7, 8));
    results[6] = make_vec2(9, 10).y;

    selfplay_lo = 0x1234;
    if (__farpeek8(0, (u16)results) != (u8)(selfplay_lo >> 8)) {
        goto complex_mismatch;
    }

    if (((u8)(__rng8() + 3) >= (u8)(selfplay_lo >> 8)) && (results[0] != 0)) {
        results[7] = 0x42;
    }

complex_done:
    results[8] = 0xA5;
    while (1) {
    }

complex_mismatch:
    results[9] = (u8)((selfplay_lo >> 8) + results[0]);
    goto complex_done;
}

