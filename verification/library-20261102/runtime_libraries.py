from pathlib import Path
import subprocess,json,os,sys,hashlib
import argparse,tempfile
parser=argparse.ArgumentParser()
parser.add_argument('--compiler',type=Path,required=True)
parser.add_argument('--zx0',type=Path,required=True)
parser.add_argument('--kurosaki',type=Path)
parser.add_argument('--compile-only',action='store_true',help='Build all selected ROMs without an emulator or Pillow')
parser.add_argument('--out',type=Path,default=None)
parser.add_argument('cases',nargs='*')
args=parser.parse_args();sys.argv=[sys.argv[0],*args.cases]
compile_only=args.compile_only
if not compile_only and args.kurosaki is None:parser.error('--kurosaki is required unless --compile-only is used')
root=Path(__file__).resolve().parents[2];fc=root;lib=root/'lib'
cli=args.kurosaki.resolve() if args.kurosaki else None;compiler=args.compiler.resolve();zx0=args.zx0.resolve()
out=args.out.resolve() if args.out else Path(tempfile.mkdtemp(prefix='kitaqfc-library-'))
out.mkdir(parents=True,exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
inputs={'compiler_sha256':sha(compiler),'zx0_sha256':sha(zx0),'kurosaki_sha256':sha(cli) if cli else None,'mode':'compile-only' if compile_only else 'runtime','libraries':{p.name:sha(p) for p in sorted(lib.iterdir()) if p.is_file() and p.suffix in ['.c','.h']}}
cases={
'state':(['flags.c','rng.c','rle.c'],'''u8 packed[]={3,9,2,7,0}; u8 buffer[8]; u8 a;
flag_set(7); r[0]=(flag_get(7)!=0); flag_clear(7);r[1]=(flag_get(7)==0);
quest_set_state(2,19);r[2]=(quest_state(2)==19);
r[3]=(rle_decode(buffer,packed)==5 && buffer[0]==9 && buffer[4]==7);
rng_seed(123);a=rng8();rng_seed(123);r[4]=(rng8()==a);'''),
'script':(['rpg_runtime.c','input.c','flags.c','text.c','script.c'],'''u8 code[]={2,9,0,7,2,0};
script_run(0,code);r[0]=(script_step()==1);r[1]=(flag_get(9)!=0);
r[2]=(script_step()==1);r[3]=(script_step()==1);r[4]=(script_step()==2);'''),
'save':(['save.c'],'''u8 a[]={4,8,15};u8 b[3];
save_init();save_clear(0);r[0]=(save_exists(0)==0);r[1]=(save_write(0,a,3)==1);
r[2]=(save_read(0,b,3)==1 && b[0]==4 && b[2]==15);
*(u8*)0x6006=9;r[3]=(save_check(0)==0);r[4]=(save_write(4,a,3)==0);'''),
'slg':(['rpg_runtime.c','input.c','map.c','slg_board.c','slg_path.c','slg_unit.c'],'''SLGBoard b; u8 cells[16];map_t m;u8 bits[2];u8 path[16];u8 cost[16];
slg_board_init(&b,4,4,cells);slg_board_clear(&b,0);slg_board_set(&b,2,1,7);
r[0]=(slg_board_get(&b,2,1)==7);bits[0]=0;bits[1]=0;
m.width=4;m.height=4;m.bank_tiles=0;m.tiles=cells;m.collision_bits=bits;m.events=0;
__ppu_ctrl_set(128);map_load(0,&m);r[1]=(map_current_width()==4);
r[2]=(path_find_bfs(0,0,3,3,path,16)==6);range_fill_move(0,0,2,cost);
r[3]=(cost[0]==0 && cost[2]==2 && cost[15]==255);r[4]=(map_is_blocked(4,0)==1);'''),
'physics':(['fixed.c','physics2d.c','physics2d_circle.c','physics3d.c'],'''KQBody2D b;KQCircleBody2D a;KQCircleBody2D c;KQBody3D x;KQBody3D y;
r[0]=(kq2d_scale_q8(100,128)==50 && kq2d_scale_q8(-100,128)==-50);
b.vx=30;b.vy=40;kq2d_body_limit_speed(&b,25);r[1]=(b.vx>=14 && b.vx<=15 && b.vy>=19 && b.vy<=20 && b.vx*b.vx+b.vy*b.vy<=625);r[6]=(u8)b.vx;r[7]=(u8)b.vy;
a.x=0;a.y=0;a.radius=4;c.x=5;c.y=0;c.radius=4;r[2]=(kq2dc_overlap_circle(&a,&c)==1);
kq3d_body_init(&x,0,0,0,4,4,4,256);kq3d_body_init(&y,3,0,0,4,4,4,256);
r[3]=(kq3d_overlap_aabb(&x,&y)==1);r[4]=(kq3d_dot_q8_8(256,256,256,64,64,64)==192);'''),
'sprites':(['sprite_order.c'],'''SpriteOrder s;SpriteOrderItem pool[4];SpriteOrderOamEntry output[64];
r[0]=sprite_order_init(&s,pool,4,8);sprite_order_begin(&s);
r[1]=sprite_order_push(&s,20,40,3,0,2);r[2]=sprite_order_push(&s,30,50,4,0,0);
r[3]=(sprite_order_build(&s,output,64)==2 && output[0].tile==4 && output[0].y==49);
r[4]=(output[2].y==255 && sprite_order_push(&s,-1,1,0,0,0)==0);'''),
'ui':(['rpg_runtime.c','input.c','text.c','menu.c'],'''__ppu_ctrl_set(128);__ppu_mask_set(30);text_set_speed(0);text_open(2,2,8,5);text_print("OK");
__nmi_wait();r[0]=1;r[1]=1;r[2]=1;r[3]=1;r[4]=1;'''),
}
cases['geometry']=(['wire3d.c','wire3d_model.c'],'Wire3D_Vec3 vertices[3];Wire3D_Edge edges[3];Wire3D_Face faces[1];Wire3D_Model m;s16 x;s16 y;s16 z;s16 sx;s16 sy;\nvertices[0].x=-12;vertices[0].y=-10;vertices[0].z=0;vertices[1].x=12;vertices[1].y=-10;vertices[1].z=0;vertices[2].x=0;vertices[2].y=12;vertices[2].z=0;\nedges[0].a=0;edges[0].b=1;edges[1].a=1;edges[1].b=2;edges[2].a=2;edges[2].b=0;faces[0].a=0;faces[0].b=2;faces[0].c=1;\nm.vertices=vertices;m.edges=edges;m.faces=faces;m.edge_faces=0;m.edge_masks=0;m.vertex_count=3;m.edge_count=3;m.face_count=1;m.edge_mask_count=0;m.flags=0;\nx=16;y=0;z=0;Wire3D_RotatePoint(&x,&y,&z,0,4,0);r[0]=(x==0 && z==-16);\nWire3D_SetCamera(0,0,0,0,0,0);r[1]=(Wire3D_ProjectPoint(0,0,96,&sx,&sy)==1 && sx==32 && sy==24);\nWire3D_Init();Wire3D_BeginFrame();Wire3D_DrawModel(&m,0,0,64,0,0,0);Wire3D_EndFrame();\nr[2]=1;r[3]=1;r[4]=1;\n')
cases['rig']=(['wire3d.c', 'wire3d_model.c', 'wire3d_rig.c', 'wire3d_rig_view.c'], 'u8 parents[]={255,0};\n__memset(&source_rig,0,sizeof(Wire3DRig));r[0]=Wire3DRig_Init(&source_rig,parents,2);\nsource_rig.pose.origin[1].x=10;source_rig.pose.vertex[8].x=-10;\nWire3DRig_Move(&source_rig,0,0,0,96);Wire3DRig_Step(&source_rig,1);\nr[1]=(source_rig.world[1].x==10);Wire3DRig_Detach(&source_rig,1);Wire3DRig_Move(&source_rig,1,30,0,96);Wire3DRig_Step(&source_rig,5);\nr[2]=(source_rig.world[1].x==30);Wire3DRig_Dock(&source_rig,1);Wire3DRig_Step(&source_rig,255);r[3]=(source_rig.world[1].x==10 && source_rig.mode[1]==0);\nWire3DRig_BuildView(&source_rig,&view_rig,0,4,0);\nr[4]=(view_rig.world[0].z+view_rig.pose.vertex[0].z==view_rig.world[1].z+view_rig.pose.vertex[8].z && source_rig.world[1].x==10);')
modules,body=cases['geometry']
body=body.replace('r[2]=1;r[3]=1;r[4]=1;', '''Wire3D_Object objects[2];u16 i;
__memcpy(reference_pixels,(u8*)0x6800,384);
for(i=0;i<2;i++){objects[i].model=&m;objects[i].x=0;objects[i].y=0;objects[i].z=i==0?128:64;objects[i].rx=0;objects[i].ry=0;objects[i].rz=0;objects[i].scale_q8=256;objects[i].visible=1;}
Wire3D_BeginFrame();Wire3D_DrawScene(objects,2);Wire3D_EndFrame();r[2]=1;
for(i=0;i<384;i++)if(reference_pixels[i]!=*((u8*)0x6800+i))r[2]=0;
Wire3D_BeginFrame();Wire3D_DrawLine2D(5,5,20,5);Wire3D_EraseSpan2D(5,5,20);r[3]=1;
for(i=0;i<384;i++)if(*((u8*)0x6800+i)!=0)r[3]=0;
Wire3D_BeginFrame();Wire3D_DrawModel(&m,0,0,64,0,0,0);Wire3D_EndFrame();r[4]=(wire3d_uploaded_tiles>0 && wire3d_transfer_frames>0);''')
cases['geometry']=(modules,body)
cases['raster']=(['raster.c'], 'RasterLineX e;u8 i;\n__ppu_ctrl_set(136);__ppu_mask_set(30);Raster_Init();r[0]=(Raster_BuildParallax3(64,128,0,8,16,0)==0);\nr[1]=(Raster_GetCount()==3);r[2]=(Raster_Push(120,0,0)==KQ_RASTER_UNSORTED);\nRaster_LineXInit(&e,1,12);r[3]=(Raster_LineXGetOffset(&e,8)==12);\nRaster_BuildParallax3(64,128,0,8,16,0);for(i=0;i<5;i++)Raster_Commit();__nmi_wait();r[4]=(nes_raster_applied>0);')
modules,body=cases['raster']
body=body.replace('__ppu_ctrl_set(136);__ppu_mask_set(30);Raster_Init();','''u16 j;u8 palette[]={15,44,44,44,15,44,44,44,15,44,44,44,15,44,44,44};
__ppu_mask_set(0);__ppu_ctrl_set(0);__vram_fill(0,255,8);__vram_fill(8,0,24);
__ppu_addr(0x2000);for(j=0;j<960;j++)__ppu_data((u8)(j&1));__vram_fill(0x23c0,0,64);__palette_bg_load(palette);
__ppu_ctrl_set(136);__ppu_mask_set(30);Raster_Init();''').replace('64,128,0,8,16,0','64,128,0,4,8,0')
cases['raster']=(modules,body)
raw=(b'KITAQFC ZX0 native test 0123456789 '*4)
(out/'zx0-input.bin').write_bytes(raw)
p=subprocess.run([str(zx0),str(out/'zx0-input.bin'),str(out/'zx0-packed.bin'),'--format=zx0'],capture_output=True)
assert p.returncode==0,p.stderr
packed=(out/'zx0-packed.bin').read_bytes()
cases['zx0']=(['zx0.c'],f'''u8 buffer[{len(raw)}];u16 n;u16 i;
n=zx0_decompress(buffer,sizeof(buffer),packed,sizeof(packed));r[0]=(n=={len(raw)} && zx0_error==ZX0_OK);
r[1]=1;for(i=0;i<n;i++)if(buffer[i]!=original[i])r[1]=0;
r[2]=(zx0_decompress(buffer,2,packed,sizeof(packed))==0 && zx0_error==ZX0_OUTPUT_FULL);
r[3]=(zx0_decompress(buffer,sizeof(buffer),packed,1)==0 && zx0_error==ZX0_TRUNCATED);
__ppu_mask_set(0);n=zx0_decompress_vram(0x1000,buffer,sizeof(buffer),packed,sizeof(packed));r[4]=(n=={len(raw)} && zx0_error==ZX0_OK);''')
cases['metatile']=(['rpg_runtime.c','input.c','map.c'],'''metatile_t tiles[4];metatile_map_t m;u8 ids[4];u8 i;
for(i=0;i<4;i++){tiles[i].tile_tl=(u8)(10+i);tiles[i].tile_tr=(u8)(20+i);tiles[i].tile_bl=(u8)(30+i);tiles[i].tile_br=(u8)(40+i);tiles[i].collision=(u8)(i==3);tiles[i].palette=i;tiles[i].event=(u8)(i+1);ids[i]=i;}
m.width=2;m.height=2;m.bank_tiles=0;m.metatile_ids=ids;m.metatiles=tiles;
__ppu_ctrl_set(128);map_load_metatile(0,&m);r[0]=(map_get_metatile(1,1)==3);r[1]=(map_is_solid(1,1)==1);r[2]=(map_get_event(1,0)==2);
r[3]=(map_current_width()==2 && map_current_height()==2);r[4]=1;''')
cases['camera_math']=(['camera.c','math.c'],'''Camera8_8 c;
Camera_Init(&c);Camera_Set(&c,-256,512);r[0]=(Camera_WorldToScreenX(&c,5)==6 && Camera_WorldToScreenY(&c,5)==3);
Camera_SetBounds(&c,0,0,1024,1024);r[1]=(c.fx==0 && c.fy==512);
Camera_SetTarget(&c,1024,0);Camera_StepTowardTarget(&c,1);r[2]=(c.fx==512 && c.fy==256);
r[3]=(Camera_ScreenToWorldX(&c,5)==7);r[4]=(MATH_SIN[0]==128 && MATH_SIN[128]==128 && MATH_SIN[64]==255);
Camera_ClampToSize(&c,356,600,256,240);r[3]=r[3] && c.max_fx==25600 && c.max_fy==32767;
Camera_ClampToSize(&c,128,120,256,240);r[3]=r[3] && c.max_fx==0 && c.max_fy==0;''')
results=[]
for name,(modules,body) in cases.items():
 if len(sys.argv)>1 and name not in sys.argv[1:]:continue
 source=out/(name+'.c');headers='#include "rpg.h"\n'
 if name=='rig':headers+='#include \"wire3d_rig.h\"\n__location(0x7900) Wire3DRig source_rig;\n__location(0x7B00) Wire3DRig view_rig;\n'
 if name=='raster':headers+='#include \"raster.h\"\n'
 if name=='camera_math':headers+='#include "camera.h"\n#include "math.h"\n'
 if name=='geometry':headers+='#include \"wire3d_model.h\"\n__location(0x7D00) u8 reference_pixels[384];\n'
 if name=='zx0':
  headers+='#include "zx0.h"\n#pragma fixed_bank 0\n__prg_rom const u8 packed[]={'+','.join(map(str,packed))+'};\n__prg_rom const u8 original[]={'+','.join(map(str,raw))+'};\n#pragma fixed_bank -1\n'
 if name=='physics':headers+='#include "physics2d.h"\n#include "physics2d_circle.h"\n#include "physics3d.h"\n'
 if name=='sprites':headers+='#include "sprite_order.h"\n'
 if name=='slg':headers+='#include "slg.h"\n'
 source.write_text(headers+'__location(0x07F0) u8 r[16];\nvoid main() {'+body+'r[5]=165;while(1){} }')
 if name=='raster':source.write_text(source.read_text().replace('while(1){}','while(1){Raster_Commit();}'))
 args=[str(compiler),str(source),*[str(lib/m) for m in modules],'-I',str(lib),'--mapper=mmc3','--nes-chr-ram','-o',str(out/(name+'.nes'))]
 args+=['--kurosaki-metadata='+str(out/(name+'-metadata.json'))]
 if os.environ.get('KQ_TEST_OPT'):args+=['--nes-opt='+os.environ['KQ_TEST_OPT']]
 p=subprocess.run(args,capture_output=True);(out/(name+'-compile.log')).write_bytes(p.stdout+p.stderr)
 if p.returncode:print(name,'compile failed',(p.stdout+p.stderr).decode(errors='replace')[-800:]);continue
 if compile_only:
  rom=(out/(name+'.nes')).read_bytes()
  assert rom[:4]==b'NES\x1a' and len(rom)>16,name
  row={'case':name,'passed':True,'rom_sha256':hashlib.sha256(rom).hexdigest()};results.append(row);print(row)
  continue
 snapshot=out/(name+'.json')
 p=subprocess.run([str(cli),'run',str(out/(name+'.nes')),'--frames',str(1200 if name=='geometry' else 240),'--headless','--json',str(out/(name+'-run.json')),'--snapshot',str(snapshot),'--png',str(out/(name+'.png'))],capture_output=True)
 (out/(name+'-run.log')).write_bytes(p.stdout+p.stderr)
 if p.returncode:print(name,'runtime failed',p.stderr.decode(errors='replace')[-800:]);continue
 s=json.loads(snapshot.read_text()); actual=s['bus']['ram'][0x7f0:0x7f6]
 if name=='physics':print('limiter velocities',s['bus']['ram'][0x7f6:0x7f8])
 expected=[1,1,1,1,1,165]
 if name=='ui':
  actual += [s['bus']['ppu']['vram'][0x2063],s['bus']['ppu']['vram'][0x2064]];expected += [79,75]
 if name=='geometry':
  cart=s['mapper_private'];ram=cart[28:8220];chr=cart[8220:16412];stage=ram[0x800:0xe00]
  front=0x1000 if s['bus']['ppu']['ctrl']&16 else 0
  actual += [sum(v!=0 for v in stage)>10,all(chr[front+t*16:front+t*16+8]==stage[t*8:t*8+8] for t in range(48)),all(v==0 for t in range(48) for v in chr[front+t*16+8:front+t*16+16])]
  expected += [True,True,True]
 if name=='metatile':
  v=s['bus']['ppu']['vram'];actual += [v[0x23c0],v[0x2000],v[0x2002],v[0x2040],v[0x2042]];expected += [228,10,11,12,13]
 if name=='zx0':
  chr=s['mapper_private'][8220:16412];actual += [chr[0x1000:0x1000+len(raw)]==list(raw)];expected += [True]
 if name=='raster':
  from PIL import Image
  im=Image.open(out/'raster.png').convert('RGB')
  actual += [[int(any(im.getpixel((x,y)))) for x in (0,4,8,12)] for y in (32,96,160)]
  expected += [[1,1,0,0],[1,0,0,1],[0,0,1,1]]
 row={'case':name,'actual':actual,'expected':expected,'passed':actual==expected,'pc':s['cpu']['pc']};results.append(row);print(row)
(out/'results.json').write_text(json.dumps(results,indent=2))
assert len(results)==(len(sys.argv)-1 if len(sys.argv)>1 else len(cases)) and all(r['passed'] for r in results),results
assert inputs['compiler_sha256']==sha(compiler) and inputs['zx0_sha256']==sha(zx0)
assert inputs['kurosaki_sha256']==(sha(cli) if cli else None)
assert inputs['libraries']=={p.name:sha(p) for p in sorted(lib.iterdir()) if p.is_file() and p.suffix in ['.c','.h']}
(out/'run-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
