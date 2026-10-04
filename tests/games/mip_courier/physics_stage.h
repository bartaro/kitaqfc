#ifndef MIP_PHYSICS_STAGE_H
#define MIP_PHYSICS_STAGE_H

#include "game_defs.h"

void physics_stage_init(void);
void physics_stage_reset_positions(void);
u8 physics_stage_update_player(void);
void physics_stage_update_drones(void);
extern u8 mip_physics_collision_index;
extern u8 mip_physics_collision_result;
void physics_stage_check_cargo(void);
void physics_stage_check_gate(void);
void physics_stage_check_drone(void);
void physics_stage_knock_selected_drone(void);

#endif
