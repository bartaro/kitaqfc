#include "entity.h"

// Fixed-capacity storage keeps entity addresses stable and avoids heap allocation.
// IDs are reusable slot indices; 0xFF denotes allocation failure or no sprite.
static Entity entity_pool[ENTITY_MAX];

// Reset every slot before the game starts using the pool.
void entity_init(void)
{
    entity_clear_all();
}

// Release every entity and restore deterministic defaults, including the no-sprite sentinel.
void entity_clear_all(void)
{
    u8 i;
    i = 0;
    while (i < ENTITY_MAX) {
        entity_pool[(__safe_index u8)i].active = 0;
        entity_pool[(__safe_index u8)i].type = 0;
        entity_pool[(__safe_index u8)i].x = 0;
        entity_pool[(__safe_index u8)i].y = 0;
        entity_pool[(__safe_index u8)i].vx = 0;
        entity_pool[(__safe_index u8)i].vy = 0;
        entity_pool[(__safe_index u8)i].state = 0;
        entity_pool[(__safe_index u8)i].timer = 0;
        entity_pool[(__safe_index u8)i].sprite = 0xFF;
        i = (u8)(i + 1);
    }
}

// Claim the first inactive slot, initialize its state, and return its index.
// Return 0xFF when the pool is full; callers must check before using the ID.
u8 entity_create(u8 type, s16 x, s16 y)
{
    u8 i;
    i = 0;
    while (i < ENTITY_MAX) {
        if (entity_pool[(__safe_index u8)i].active == 0) {
            entity_pool[(__safe_index u8)i].active = 1;
            entity_pool[(__safe_index u8)i].type = type;
            entity_pool[(__safe_index u8)i].x = x;
            entity_pool[(__safe_index u8)i].y = y;
            entity_pool[(__safe_index u8)i].vx = 0;
            entity_pool[(__safe_index u8)i].vy = 0;
            entity_pool[(__safe_index u8)i].state = 0;
            entity_pool[(__safe_index u8)i].timer = 0;
            entity_pool[(__safe_index u8)i].sprite = 0xFF;
            return i;
        }
        i = (u8)(i + 1);
    }
    return 0xFF;
}

// Release a valid slot without clearing its payload. Reallocation resets the payload;
// a saved pointer or ID must not be treated as a persistent entity identity.
void entity_destroy(u8 id)
{
    if (id >= ENTITY_MAX) return;
    entity_pool[(__safe_index u8)id].active = 0;
}

// Return a pointer only for a currently active slot. Invalid or inactive IDs return null.
// The pointer aliases pool storage and may refer to a different entity after slot reuse.
Entity* entity_get(u8 id)
{
    if (id >= ENTITY_MAX) return 0;
    if (entity_pool[(__safe_index u8)id].active == 0) return 0;
    return &entity_pool[(__safe_index u8)id];
}

// Visit active slots in ascending ID order and pass each ID to the callback.
// Check activity when visiting the slot, so callback changes affect subsequent visits.
void entity_update_all(EntityFn fn)
{
    u8 i;
    if (fn == 0) return;
    i = 0;
    while (i < ENTITY_MAX) {
        if (entity_pool[(__safe_index u8)i].active != 0) fn(i);
        i = (u8)(i + 1);
    }
}

// Use the same active-slot traversal for the game's drawing callback.
// The callback submits drawing data; the entity pool does not allocate sprites.
void entity_draw_all(EntityFn fn)
{
    entity_update_all(fn);
}

// Count occupied slots without changing entity state.
u8 entity_count_active(void)
{
    u8 i;
    u8 count;
    i = 0;
    count = 0;
    while (i < ENTITY_MAX) {
        if (entity_pool[(__safe_index u8)i].active != 0) count = (u8)(count + 1);
        i = (u8)(i + 1);
    }
    return count;
}
