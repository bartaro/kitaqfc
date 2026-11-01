// Copyright (c) 2026 DAISUKE OBA. MIT License.
#pragma once
#include "wire3d.h"
#define WIRE3D_MODEL_VERTEX_LIMIT 24
#define WIRE3D_MODEL_FACE_LIMIT 16
#define WIRE3D_SCENE_OBJECT_LIMIT 8
#define WIRE3D_FACE_NONE 255
#define WIRE3D_MODEL_HIDDEN_LINES 1
typedef struct { s16 x; s16 y; s16 z; } Wire3D_Vec3;
typedef struct { u8 a; u8 b; } Wire3D_Edge;
typedef struct { u8 a; u8 b; u8 c; } Wire3D_Face;
typedef struct { u8 f0; u8 f1; } Wire3D_EdgeFaces;
typedef struct {
 const Wire3D_Vec3* vertices; const Wire3D_Edge* edges;
 const Wire3D_Face* faces; const Wire3D_EdgeFaces* edge_faces;
 const u16* edge_masks; u8 vertex_count; u8 edge_count; u8 face_count;
 u8 edge_mask_count; u8 flags;
} Wire3D_Model;
typedef struct {
 const Wire3D_Model* model; s16 x; s16 y; s16 z;
 s8 rx; s8 ry; s8 rz; s16 scale_q8; u8 visible;
} Wire3D_Object;
// NES monochrome backend, including camera, face culling and near-first
// scene occlusion. Link wire3d.c + wire3d_model.c. Owns PRG RAM $6800-$77FF;
// requires CHR RAM. Native viewport is 64x48, 96x64 or 128x96, not CGB VRAM.
// All pointers remain in the current mapping. 16 angle steps per turn.
void Wire3D_Init(void);
void Wire3D_BeginFrame(void);
void Wire3D_EndFrame(void);
void Wire3D_SetCamera(s16 x,s16 y,s16 z,s8 pitch,s8 yaw,s8 roll);
void Wire3D_RotatePoint(s16* x,s16* y,s16* z,u8 rx,u8 ry,u8 rz);
u8 Wire3D_ProjectPoint(s16 x,s16 y,s16 z,s16* sx,s16* sy);
void Wire3D_DrawLine2D(s16 ax,s16 ay,s16 bx,s16 by);
void Wire3D_DrawLine3D(s16 ax,s16 ay,s16 az,s16 bx,s16 by,s16 bz);
void Wire3D_DrawModel(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz);
void Wire3D_DrawModelScaled(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz,s16 scale_q8);
void Wire3D_DrawScene(const Wire3D_Object* objects,u8 count);
void Wire3D_EraseSpan2D(u8 y,u8 x0,u8 x1);
void Wire3D_EraseTriangle2D(u8 ax,u8 ay,u8 bx,u8 by,u8 cx,u8 cy);
void Wire3D_EraseModelFaces(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz,s16 scale_q8);
u16 Wire3D_SelectEdgeMask(const Wire3D_Model* model,u8 yaw);
