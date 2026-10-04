#ifndef MIP_GAME_DEFS_H
#define MIP_GAME_DEFS_H

#include "../../../lib/physics2d.h"

#define PAD_A      ((u8)0x01)
#define PAD_B      ((u8)0x02)
#define PAD_SELECT ((u8)0x04)
#define PAD_START  ((u8)0x08)
#define PAD_UP     ((u8)0x10)
#define PAD_DOWN   ((u8)0x20)
#define PAD_LEFT   ((u8)0x40)
#define PAD_RIGHT  ((u8)0x80)

#define GAME_TITLE  ((u8)0)
#define GAME_PLAY   ((u8)1)
#define GAME_RESULT ((u8)2)

#define TILE_SPACE         ((u8)0)
#define TILE_FLOOR         ((u8)1)
#define TILE_WALL          ((u8)2)
#define TILE_WALL_ALT      ((u8)3)
#define TILE_CARGO         ((u8)4)
#define TILE_GATE_CLOSED   ((u8)5)
#define TILE_GATE_OPEN     ((u8)6)
#define TILE_BUMPER        ((u8)7)
#define TILE_CROSS         ((u8)8)
#define TILE_CORE          ((u8)9)
#define TILE_SPARK         ((u8)10)
#define TILE_HEART         ((u8)11)
#define TILE_RING          ((u8)12)
#define TILE_BAR           ((u8)13)
#define TILE_SMALL_SPARK   ((u8)14)
#define TILE_CAPSULE       ((u8)15)
#define TILE_0             ((u8)16)
#define TILE_A             ((u8)26)
#define TILE_COLON         ((u8)52)
#define TILE_DASH          ((u8)53)
#define TILE_SLASH         ((u8)54)
#define TILE_EXCL          ((u8)55)
#define TILE_DOT           ((u8)56)

#define PLAYER_SPRITE_IDLE   ((u8)0)
#define PLAYER_SPRITE_THRUST ((u8)4)
#define PLAYER_SPRITE_BOOST  ((u8)8)
#define DRONE_SPRITE         ((u8)16)
#define SPARK_SPRITE         ((u8)20)
#define HIT_SPRITE           ((u8)21)

#define CARGO_COUNT ((u8)8)
#define DRONE_COUNT ((u8)2)
#define WALL_COUNT  ((u8)4)
#define BUMPER_COUNT ((u8)3)

#define PHYS_EVENT_NONE   ((u8)0)
#define PHYS_EVENT_BOOST  ((u8)1)
#define PHYS_EVENT_BOUNCE ((u8)2)

typedef struct MipGameState {
    KQ2DPosition player_x;
    KQ2DPosition player_y;
    KQ2DFraction player_sub_x;
    KQ2DFraction player_sub_y;
    KQ2DSpeed player_speed_x;
    KQ2DSpeed player_speed_y;
    KQ2DDirection player_dir_x;
    KQ2DDirection player_dir_y;
    KQ2DPosition drone0_x;
    KQ2DPosition drone0_y;
    KQ2DFraction drone0_sub_x;
    KQ2DFraction drone0_sub_y;
    KQ2DSpeed drone0_speed_x;
    KQ2DSpeed drone0_speed_y;
    KQ2DDirection drone0_dir_x;
    KQ2DDirection drone0_dir_y;
    KQ2DPosition drone1_x;
    KQ2DPosition drone1_y;
    KQ2DFraction drone1_sub_x;
    KQ2DFraction drone1_sub_y;
    KQ2DSpeed drone1_speed_x;
    KQ2DSpeed drone1_speed_y;
    KQ2DDirection drone1_dir_x;
    KQ2DDirection drone1_dir_y;
    u16 score;
    u16 frame;
    u8 mode;
    u8 stage;
    u8 cargo_active_mask;
    u8 cargo_remaining;
    u8 health;
    u8 boosts;
    u8 time_seconds;
    u8 second_frames;
    u8 combo;
    u8 combo_timer;
    u8 boost_timer;
    u8 invul_timer;
    u8 bumper_cooldown;
    u8 drone0_stun;
    u8 drone1_stun;
    u8 gate_open;
    u8 last_dir;
    u8 result_win;
    u8 result_delay;
} MipGameState;

extern MipGameState mip_game;

#endif
