// Copyright (c) 2026 DAISUKE OBA. MIT License.
#include "wire3d_model.h"
extern u8 w3dfc_pixels[1536]; extern u8 w3dfc_dirty[192];
__location(0x7100) u8 w3dm_occlusion[1536];
__location(0x7700) s16 w3dm_x[24];
__location(0x7730) s16 w3dm_y[24];
__location(0x7760) u8 w3dm_valid[24];
__location(0x7778) u8 w3dm_faces[16];
static u8 w3dm_occluding;
static s16 w3dm_cx;static s16 w3dm_cy;static s16 w3dm_cz;
static s8 w3dm_pitch;static s8 w3dm_yaw;static s8 w3dm_roll;
static __prg_rom s8 w3dm_sine[16]={0,24,45,59,64,59,45,24,0,-24,-45,-59,-64,-59,-45,-24};
static s16 w3dm_mul(s16 v,s16 coefficient,s16 divisor) {
 u16 a,b,d,q,r,result;u8 negative;
 negative=(u8)((v<0)!=(coefficient<0));a=v<0?(u16)(0-(u16)v):(u16)v;
 b=coefficient<0?(u16)(0-(u16)coefficient):(u16)coefficient;d=(u16)divisor;
 q=(u16)(a/d);r=(u16)(a%d);
 result=(u16)(q*b+r*(b/d)+(r*(b%d))/d);
 return negative?(s16)(0-result):(s16)result;
}
void Wire3D_RotatePoint(s16* x,s16* y,s16* z,u8 rx,u8 ry,u8 rz) {
 s16 a,b; s16 sn,cs;
 sn=w3dm_sine[ry&15];cs=w3dm_sine[(ry+4)&15];a=*x;b=*z;
 *x=(s16)(w3dm_mul(a,cs,64)+w3dm_mul(b,sn,64));*z=(s16)(w3dm_mul(b,cs,64)-w3dm_mul(a,sn,64));
 sn=w3dm_sine[rx&15];cs=w3dm_sine[(rx+4)&15];a=*y;b=*z;
 *y=(s16)(w3dm_mul(a,cs,64)-w3dm_mul(b,sn,64));*z=(s16)(w3dm_mul(a,sn,64)+w3dm_mul(b,cs,64));
 sn=w3dm_sine[rz&15];cs=w3dm_sine[(rz+4)&15];a=*x;b=*y;
 *x=(s16)(w3dm_mul(a,cs,64)-w3dm_mul(b,sn,64));*y=(s16)(w3dm_mul(a,sn,64)+w3dm_mul(b,cs,64));
}
void Wire3D_SetCamera(s16 x,s16 y,s16 z,s8 pitch,s8 yaw,s8 roll) {
 w3dm_cx=x;w3dm_cy=y;w3dm_cz=z;w3dm_pitch=pitch;w3dm_yaw=yaw;w3dm_roll=roll;
}
static void w3dm_camera(s16* x,s16* y,s16* z) {
 *x=(s16)(*x-w3dm_cx);*y=(s16)(*y-w3dm_cy);*z=(s16)(*z-w3dm_cz);
 Wire3D_RotatePoint(x,y,z,0,0,(u8)(0-w3dm_roll));
 Wire3D_RotatePoint(x,y,z,(u8)(0-w3dm_pitch),0,0);
 Wire3D_RotatePoint(x,y,z,0,(u8)(0-w3dm_yaw),0);
}
void Wire3D_Init(void) { Wire3DFC_Init();Wire3D_SetCamera(0,0,0,0,0,0);Wire3D_BeginFrame(); }
void Wire3D_BeginFrame(void) { Wire3DFC_BeginFrame();__memset(w3dm_occlusion,0,1536);w3dm_occluding=0; }
void Wire3D_EndFrame(void) { Wire3DFC_EndFrame(); }
u8 Wire3D_ProjectPoint(s16 x,s16 y,s16 z,s16* sx,s16* sy) {
 w3dm_camera(&x,&y,&z);return Wire3DFC_ProjectPoint(x,y,z,sx,sy);
}
static void w3dm_pixel(s16 x,s16 y,u8 operation) {
 u16 address;u16 mask_index;u8 bit;
 if(x<0||x>=WIRE3D_FC_WIDTH||y<0||y>=WIRE3D_FC_HEIGHT)return;
 bit=(u8)(128>>(x&7));mask_index=(u16)(y*(WIRE3D_FC_WIDTH/8)+(x>>3));
 if(operation==2) { w3dm_occlusion[mask_index]=(u8)(w3dm_occlusion[mask_index]|bit);return; }
 if(operation==1&&w3dm_occluding&&(w3dm_occlusion[mask_index]&bit)!=0)return;
 address=(u16)(((y>>3)*(WIRE3D_FC_WIDTH/8)+(x>>3))*8+(y&7));
 if(operation==0)w3dfc_pixels[address]=(u8)(w3dfc_pixels[address]&~bit);
 else w3dfc_pixels[address]=(u8)(w3dfc_pixels[address]|bit);
 w3dfc_dirty[address>>3]=1;
}
void Wire3D_DrawLine2D(s16 ax,s16 ay,s16 bx,s16 by) {
 s16 dx,dy,sx,sy,error,two;u16 budget;
 if(!w3dm_occluding) {Wire3DFC_DrawLine2D(ax,ay,bx,by);return;}
 if(ax<-512||ax>511||bx<-512||bx>511||ay<-512||ay>511||by<-512||by>511)return;
 dx=bx>ax?bx-ax:ax-bx;dy=by>ay?ay-by:by-ay;sx=ax<bx?1:-1;sy=ay<by?1:-1;error=dx+dy;budget=1024;
 while(budget!=0) {budget--;w3dm_pixel(ax,ay,1);if(ax==bx&&ay==by)return;two=(s16)(error*2);
 if(two>=dy){error=(s16)(error+dy);ax=(s16)(ax+sx);}if(two<=dx){error=(s16)(error+dx);ay=(s16)(ay+sy);}}
}
void Wire3D_DrawLine3D(s16 ax,s16 ay,s16 az,s16 bx,s16 by,s16 bz) {
 s16 x0,y0,x1,y1;if(!Wire3D_ProjectPoint(ax,ay,az,&x0,&y0))return;
 if(!Wire3D_ProjectPoint(bx,by,bz,&x1,&y1))return;Wire3D_DrawLine2D(x0,y0,x1,y1);
}
static s16 w3dm_edge(s16 ax,s16 ay,s16 bx,s16 by,s16 x,s16 y) {
 s16 dx,dy,px,py;dx=(s16)(bx-ax);dy=(s16)(by-ay);px=(s16)(x-ax);py=(s16)(y-ay);
 while(dx>63||dx<-63||dy>63||dy<-63||px>63||px<-63||py>63||py<-63) {dx=(s16)(dx/2);dy=(s16)(dy/2);px=(s16)(px/2);py=(s16)(py/2);}
 return (s16)(dx*py-dy*px);
}
static void w3dm_triangle(u8 a,u8 b,u8 c,u8 operation) {
 s16 x,y; s16 e0,e1,e2; s16 minx,maxx,miny,maxy;
 if(!w3dm_valid[a]||!w3dm_valid[b]||!w3dm_valid[c])return;
 minx=w3dm_x[a];maxx=minx;miny=w3dm_y[a];maxy=miny;
 if(w3dm_x[b]<minx)minx=w3dm_x[b];if(w3dm_x[c]<minx)minx=w3dm_x[c];
 if(w3dm_x[b]>maxx)maxx=w3dm_x[b];if(w3dm_x[c]>maxx)maxx=w3dm_x[c];
 if(w3dm_y[b]<miny)miny=w3dm_y[b];if(w3dm_y[c]<miny)miny=w3dm_y[c];
 if(w3dm_y[b]>maxy)maxy=w3dm_y[b];if(w3dm_y[c]>maxy)maxy=w3dm_y[c];
 if(minx<0)minx=0;if(miny<0)miny=0;if(maxx>=WIRE3D_FC_WIDTH)maxx=WIRE3D_FC_WIDTH-1;if(maxy>=WIRE3D_FC_HEIGHT)maxy=WIRE3D_FC_HEIGHT-1;
 for(y=miny;y<=maxy;y++)for(x=minx;x<=maxx;x++) {
 e0=w3dm_edge(w3dm_x[a],w3dm_y[a],w3dm_x[b],w3dm_y[b],x,y);
 e1=w3dm_edge(w3dm_x[b],w3dm_y[b],w3dm_x[c],w3dm_y[c],x,y);
 e2=w3dm_edge(w3dm_x[c],w3dm_y[c],w3dm_x[a],w3dm_y[a],x,y);
 if((e0>=0&&e1>=0&&e2>=0)||(e0<=0&&e1<=0&&e2<=0))w3dm_pixel(x,y,operation);
 }
}
void Wire3D_EraseSpan2D(u8 y,u8 x0,u8 x1) {u16 x;u8 t;if(x0>x1){t=x0;x0=x1;x1=t;}for(x=x0;x<=x1;x++)w3dm_pixel((s16)x,y,0);}
void Wire3D_EraseTriangle2D(u8 ax,u8 ay,u8 bx,u8 by,u8 cx,u8 cy) {
 w3dm_x[0]=ax;w3dm_y[0]=ay;w3dm_x[1]=bx;w3dm_y[1]=by;w3dm_x[2]=cx;w3dm_y[2]=cy;
 w3dm_valid[0]=1;w3dm_valid[1]=1;w3dm_valid[2]=1;
 if(w3dm_edge(ax,ay,bx,by,cx,cy)!=0)w3dm_triangle(0,1,2,0);
}
u16 Wire3D_SelectEdgeMask(const Wire3D_Model* m,u8 yaw) {
 u8 bin;if(m==0||m->edge_masks==0||m->edge_mask_count==0)return 65535;
 bin=(u8)(((yaw&15)*m->edge_mask_count)>>4);return m->edge_masks[bin];
}
static void w3dm_prepare(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz,s16 scale) {
 u8 i,a,b,c;s16 px,py,pz;
 if(scale<=0)scale=256;
 for(i=0;i<24;i++) { w3dm_valid[i]=0;if(i<m->vertex_count) {
 px=w3dm_mul(m->vertices[i].x,scale,256);py=w3dm_mul(m->vertices[i].y,scale,256);pz=w3dm_mul(m->vertices[i].z,scale,256);
 Wire3D_RotatePoint(&px,&py,&pz,(u8)rx,(u8)ry,(u8)rz);px=(s16)(px+x);py=(s16)(py+y);pz=(s16)(pz+z);
 w3dm_valid[i]=Wire3D_ProjectPoint(px,py,pz,&w3dm_x[i],&w3dm_y[i]);
 }}
 for(i=0;i<16;i++) { w3dm_faces[i]=0;if(m->faces!=0&&i<m->face_count) {
 a=m->faces[i].a;b=m->faces[i].b;c=m->faces[i].c;
 if(a<24&&b<24&&c<24&&w3dm_valid[a]&&w3dm_valid[b]&&w3dm_valid[c])
 w3dm_faces[i]=(w3dm_edge(w3dm_x[a],w3dm_y[a],w3dm_x[b],w3dm_y[b],w3dm_x[c],w3dm_y[c])>0);
 }}
}
static void w3dm_edges(const Wire3D_Model* m,u16 mask) {
 u8 i,a,b,visible,f0,f1;
 for(i=0;i<m->edge_count;i++) {
 a=m->edges[i].a;b=m->edges[i].b;if(a>=24||b>=24||!w3dm_valid[a]||!w3dm_valid[b])continue;
 if(i<16&&(mask&((u16)1<<i))==0)continue;visible=1;
 if((m->flags&1)&&m->edge_faces!=0) {f0=m->edge_faces[i].f0;f1=m->edge_faces[i].f1;
 visible=(f0==255||f1==255||(f0<16&&w3dm_faces[f0])||(f1<16&&w3dm_faces[f1]));}
 if(visible)Wire3D_DrawLine2D(w3dm_x[a],w3dm_y[a],w3dm_x[b],w3dm_y[b]);
 }
}
void Wire3D_DrawModelScaled(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz,s16 scale_q8) {
 if(m==0||m->vertices==0||m->edges==0)return;
 w3dm_prepare(m,x,y,z,rx,ry,rz,scale_q8);w3dm_edges(m,Wire3D_SelectEdgeMask(m,(u8)ry));
}
void Wire3D_DrawModel(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz) {Wire3D_DrawModelScaled(m,x,y,z,rx,ry,rz,256);}
void Wire3D_EraseModelFaces(const Wire3D_Model* m,s16 x,s16 y,s16 z,s8 rx,s8 ry,s8 rz,s16 scale_q8) {
 u8 i;if(m==0||m->vertices==0||m->faces==0)return;w3dm_prepare(m,x,y,z,rx,ry,rz,scale_q8);
 for(i=0;i<16&&i<m->face_count;i++)if(w3dm_faces[i])w3dm_triangle(m->faces[i].a,m->faces[i].b,m->faces[i].c,0);
}
void Wire3D_DrawScene(const Wire3D_Object* objects,u8 count) {
 u8 order[8];s16 depths[8];u8 i,j,t,index;u8 f;s16 x,y,z;const Wire3D_Object* o;const Wire3D_Model* m;
 if(objects==0)return;if(count>8)count=8;
 for(i=0;i<count;i++){order[i]=i;x=objects[i].x;y=objects[i].y;z=objects[i].z;w3dm_camera(&x,&y,&z);depths[i]=z;}
 for(i=0;i<count;i++)for(j=(u8)(i+1);j<count;j++)if(depths[order[j]]<depths[order[i]]){t=order[i];order[i]=order[j];order[j]=t;}
 __memset(w3dm_occlusion,0,1536);w3dm_occluding=1;
 for(i=0;i<count;i++){index=order[i];o=&objects[index];m=o->model;
 if(!o->visible||m==0||m->vertices==0||m->edges==0)continue;
 w3dm_prepare(m,o->x,o->y,o->z,o->rx,o->ry,o->rz,o->scale_q8);w3dm_edges(m,Wire3D_SelectEdgeMask(m,(u8)o->ry));
 if(m->faces!=0)for(f=0;f<16&&f<m->face_count;f++)if(w3dm_faces[f])w3dm_triangle(m->faces[f].a,m->faces[f].b,m->faces[f].c,2);
 }w3dm_occluding=0;
}
