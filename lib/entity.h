#ifndef ENTITY_H
#define ENTITY_H

#include "core.h"

// Choose capacity 1..255 consistently across units; counters are bytes and FF is the failure sentinel.
#ifndef ENTITY_MAX
#define ENTITY_MAX ((u8)16)
#endif

// Coordinates, velocities, state and timer have game-defined units.
// The sprite byte is bookkeeping only; releasing a slot does not hide or free its sprite.
typedef __packed struct Entity {
    u8 active;
    u8 type;
    s16 x;
    s16 y;
    s16 vx;
    s16 vy;
    u8 state;
    u8 timer;
    u8 sprite;
} Entity;

// An entity callback receives a slot ID, not a persistent object handle.
// Use a callback in common PRG bank 0, or keep its bank mapped throughout dispatch.
typedef void (*EntityFn)(u8 id);

// Reset every slot before the game starts using the pool.
void entity_init(void);
// Returns a reusable slot ID, or 0xFF if no slot is free.
u8 entity_create(u8 type, s16 x, s16 y);
// Release a valid slot without clearing its payload. Reallocation resets the payload;
// a saved pointer or ID must not be treated as a persistent entity identity.
void entity_destroy(u8 id);
// Returns null for an invalid or inactive ID; the returned pointer aliases reusable pool storage.
Entity* entity_get(u8 id);
// Call fn(id) for each active slot in ascending order. A null callback does nothing.
// Changes to later slots take effect during the same traversal. Do not reenter this traversal.
void entity_update_all(EntityFn fn);
// Call the drawing callback for each active slot, using the same traversal rules as update.
// The callback submits drawing data; the pool does not allocate sprites.
void entity_draw_all(EntityFn fn);
// Count occupied slots without changing entity state.
u8 entity_count_active(void);
// Release every entity and restore deterministic defaults, including the no-sprite sentinel.
void entity_clear_all(void);

#endif
