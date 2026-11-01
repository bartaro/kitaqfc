#include "wire3d_rig.h"

const u8* w3dr_copy_source;
u8* w3dr_copy_target;

/* 36 bytes of anchors + 288 bytes of rotated local vertices. No per-vertex
   C calls, multiplies, trigonometry or bank switches in the pose hot path. */
void w3dr_copy_pose_asm() { __memcpy(w3dr_copy_target,w3dr_copy_source,324); }

void Wire3DRig_CopyPose(Wire3DRig* rig, const Wire3DRigPose* source)
{
    w3dr_copy_source = (const u8*)source;
    w3dr_copy_target = (u8*)&rig->pose;
    w3dr_copy_pose_asm();
}


u8 Wire3DRig_Init(Wire3DRig* rig, const u8* parents, u8 count)
{
    u8 i;
    rig->count = 0;
    if (count == 0 || count > W3DR_PARTS) return 0;
    i = 0;
    while (i < count) {
        if (parents[i] != W3DR_NONE && parents[i] >= i) return 0;
        i++;
    }
    i = 0;
    while (i < count) {
        rig->parent[i] = parents[i];
        rig->mode[i] = parents[i] == W3DR_NONE ? W3DR_FREE : W3DR_ATTACHED;
        rig->world[i].x = 0;
        rig->world[i].y = 0;
        rig->world[i].z = 0;
        i++;
    }
    rig->count = count;
    return 1;
}

static s16 w3dr_approach(s16 value, s16 target, u8 step)
{
    s16 delta;
    delta = target - value;
    if (delta > (s16)step) return value + (s16)step;
    if (delta < (s16)(0 - (s16)step)) return value - (s16)step;
    return target;
}

void Wire3DRig_Step(Wire3DRig* rig, u8 dock_step)
{
    u8 i;
    u8 p;
    s16 x;
    s16 y;
    s16 z;
    i = 0;
    while (i < rig->count) {
        p = rig->parent[i];
        if (p != W3DR_NONE && rig->mode[i] != W3DR_FREE) {
            x = rig->world[p].x + rig->pose.origin[i].x - rig->pose.origin[p].x;
            y = rig->world[p].y + rig->pose.origin[i].y - rig->pose.origin[p].y;
            z = rig->world[p].z + rig->pose.origin[i].z - rig->pose.origin[p].z;
            if (rig->mode[i] == W3DR_DOCKING) {
                rig->world[i].x = w3dr_approach(rig->world[i].x, x, dock_step);
                rig->world[i].y = w3dr_approach(rig->world[i].y, y, dock_step);
                rig->world[i].z = w3dr_approach(rig->world[i].z, z, dock_step);
                if (rig->world[i].x == x && rig->world[i].y == y && rig->world[i].z == z)
                    rig->mode[i] = W3DR_ATTACHED;
            } else {
                rig->world[i].x = x;
                rig->world[i].y = y;
                rig->world[i].z = z;
            }
        }
        i++;
    }
}

u8 Wire3DRig_Detach(Wire3DRig* rig, u8 part)
{
    if (part >= rig->count || rig->parent[part] == W3DR_NONE) return 0;
    rig->mode[part] = W3DR_FREE;
    return 1;
}

u8 Wire3DRig_Dock(Wire3DRig* rig, u8 part)
{
    if (part >= rig->count || rig->parent[part] == W3DR_NONE) return 0;
    if (rig->mode[part] != W3DR_ATTACHED) rig->mode[part] = W3DR_DOCKING;
    return 1;
}

u8 Wire3DRig_Move(Wire3DRig* rig, u8 part, s16 x, s16 y, s16 z)
{
    if (part >= rig->count || rig->mode[part] == W3DR_ATTACHED) return 0;
    if (x < -W3DR_WORLD_LIMIT || x > W3DR_WORLD_LIMIT) return 0;
    if (y < -W3DR_WORLD_LIMIT || y > W3DR_WORLD_LIMIT) return 0;
    if (z < -W3DR_WORLD_LIMIT || z > W3DR_WORLD_LIMIT) return 0;
    rig->world[part].x = x;
    rig->world[part].y = y;
    rig->world[part].z = z;
    return 1;
}

void Wire3DRig_BindModels(Wire3DRig* rig, Wire3D_Model* models)
{
    u8 i;
    i = 0;
    while (i < rig->count) {
        models[i].vertices = &rig->pose.vertex[i * W3DR_VERTICES];
        i++;
    }
}
