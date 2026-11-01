#pragma once
#include "wire3d_model.h"

/* Rigid, baked articulation. Authoring units are shared with Wire3D.
   Parent IDs must precede children. All data/state pointers are non-null.
   Call from the main loop only. Allocate the large states in cartridge RAM. */
#define W3DR_PARTS 6
#define W3DR_VERTICES 8
#define W3DR_NONE 255
#define W3DR_ATTACHED 0
#define W3DR_FREE 1
#define W3DR_DOCKING 2
#define W3DR_WORLD_LIMIT 4096

typedef struct {
    Wire3D_Vec3 origin[6];
    Wire3D_Vec3 vertex[48];
} Wire3DRigPose;

typedef struct {
    Wire3DRigPose pose;
    Wire3D_Vec3 world[6];
    u8 parent[6];
    u8 mode[6];
    u8 count;
} Wire3DRig;

/* Init disables the rig on invalid count/topology; does not read a ROM pose.
   parents must be in accessible RAM or currently mapped ROM.
   Load a valid baked pose before Step or BindModels. */
u8 Wire3DRig_Init(Wire3DRig* rig, const u8* parents, u8 count);
/* Copy from currently mapped memory. Source is a 324-byte pose.
   The exported loader calls this while its own pose bank is mapped. */
void Wire3DRig_CopyPose(Wire3DRig* rig, const Wire3DRigPose* source);
/* Main simulation tick. dock_step is 0..255 units PER AXIS per tick.
   Zero pauses docking. Children follow updated parents, never stale positions. */
void Wire3DRig_Step(Wire3DRig* rig, u8 dock_step);
/* Detach preserves the current world pose and keeps attached descendants.
   Dock rejoins only the CAD-defined parent, with zero socket gap. */
u8 Wire3DRig_Detach(Wire3DRig* rig, u8 part);
u8 Wire3DRig_Dock(Wire3DRig* rig, u8 part);
/* Only roots/free/docking components may move independently. Reject overflow. */
u8 Wire3DRig_Move(Wire3DRig* rig, u8 part, s16 x, s16 y, s16 z);
/* Models use the copied, already rotated vertices. Build objects once, then
   update their x/y/z from world[] after Step. Call DrawScene once for all parts. */
void Wire3DRig_BindModels(Wire3DRig* rig, Wire3D_Model* models);
/* Optional render-only whole-rig rotation about source->world[0], Y/X/Z order.
   Angles wrap at 16 (22.5 degrees/step). Source is unchanged; view must differ.
   Both states must be in accessible RAM. Relative points must be within +/-8191.
   Output is flattened draw data, NOT the simulation/joint state. BindModels to
   view for drawing, and keep all Step/Detach/Dock operations on source.
   Returns zero without changing view for aliasing or invalid source count. */
u8 Wire3DRig_BuildView(const Wire3DRig* source, Wire3DRig* view, u8 rx, u8 ry, u8 rz);
/* NES scene renderer: camera coordinates and unit scale.
   Valid CAD topology only; same viewport/depth bounds as the NES projector.
   Reads visible from objects; positions come from rig->world.
   Clears the scene occlusion mask and leaves occlusion off / monochrome line palette.
   Use one rig per BeginFrame/EndFrame scene; other objects are not Z-sorted here. */
void Wire3DRig_Draw(Wire3DRig* rig, Wire3D_Model* models, Wire3D_Object* objects);
