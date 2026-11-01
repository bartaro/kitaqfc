// Copyright (c) 2026 DAISUKE OBA. MIT License.
#include "wire3d_rig.h"
u8 Wire3DRig_BuildView(const Wire3DRig* source,Wire3DRig* view,u8 rx,u8 ry,u8 rz) {
 u8 i,j,index;s16 x,y,z; s16 px,py,pz;
 if(source==0||view==0||source==view||source->count==0||source->count>6)return 0;
 __memcpy(view,source,sizeof(Wire3DRig));px=source->world[0].x;py=source->world[0].y;pz=source->world[0].z;
 for(i=0;i<source->count;i++) {
 x=(s16)(source->world[i].x-px);y=(s16)(source->world[i].y-py);z=(s16)(source->world[i].z-pz);
 Wire3D_RotatePoint(&x,&y,&z,rx,ry,rz);view->world[i].x=(s16)(x+px);view->world[i].y=(s16)(y+py);view->world[i].z=(s16)(z+pz);
 for(j=0;j<8;j++) {index=(u8)(i*8+j);
 x=(s16)(source->world[i].x+source->pose.vertex[index].x-px);
 y=(s16)(source->world[i].y+source->pose.vertex[index].y-py);
 z=(s16)(source->world[i].z+source->pose.vertex[index].z-pz);
 Wire3D_RotatePoint(&x,&y,&z,rx,ry,rz);
 view->pose.vertex[index].x=(s16)(x+px-view->world[i].x);
 view->pose.vertex[index].y=(s16)(y+py-view->world[i].y);
 view->pose.vertex[index].z=(s16)(z+pz-view->world[i].z);
 }}return 1;
}
void Wire3DRig_Draw(Wire3DRig* rig,Wire3D_Model* models,Wire3D_Object* objects) {
 u8 i;if(rig==0||models==0||objects==0||rig->count>6)return;
 Wire3DRig_BindModels(rig,models);
 for(i=0;i<rig->count;i++) {
 objects[i].model=&models[i];objects[i].x=rig->world[i].x;objects[i].y=rig->world[i].y;objects[i].z=rig->world[i].z;
 objects[i].rx=0;objects[i].ry=0;objects[i].rz=0;objects[i].scale_q8=256;
 }Wire3D_DrawScene(objects,rig->count);
}
