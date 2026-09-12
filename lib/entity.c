#include "entity.h"

static Entity entity_pool[ENTITY_MAX];

void entity_init(void)
{
    entity_clear_all();
}

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

void entity_destroy(u8 id)
{
    if (id >= ENTITY_MAX) return;
    entity_pool[(__safe_index u8)id].active = 0;
}

Entity* entity_get(u8 id)
{
    if (id >= ENTITY_MAX) return 0;
    if (entity_pool[(__safe_index u8)id].active == 0) return 0;
    return &entity_pool[(__safe_index u8)id];
}

void entity_update_all(EntityFn fn)
{
    if (fn == 0) return;
}

void entity_draw_all(EntityFn fn)
{
    if (fn == 0) return;
}

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
