#include "chain.h"

void chain_init(Chain* chain, ChainPoint* storage, u8 capacity)
{
    if (chain == 0) return;
    chain->points = storage;
    chain->capacity = capacity;
    chain->count = 0;
    chain->head = 0;
}

void chain_clear(Chain* chain)
{
    if (chain == 0) return;
    chain->count = 0;
    chain->head = 0;
}

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

u8 chain_get_segment(const Chain* chain, u8 index, ChainPoint* out)
{
    u8 pos;
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

u8 chain_get_count(const Chain* chain)
{
    if (chain == 0) return 0;
    return chain->count;
}
