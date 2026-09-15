"""Check FC OAM field semantics, byte-stream boundaries and DMA page selection."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
BASE=Path(__file__).resolve().parents[1];REPOS=BASE.parent
p=argparse.ArgumentParser();p.add_argument('--output-parent',type=Path,required=True);p.add_argument('--compiler',type=Path,default=REPOS/'kitaqfc/kitaqfc.exe');p.add_argument('--emulator',type=Path,default=REPOS/'kurosaki/kurosaki.exe');args=p.parse_args()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
args.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-oam-',dir=args.output_parent)).resolve()
PREFIX='''#include "intrinsics.h"
__location(0x0400) u8 alternate[256];
__location(0x05FE) u8 stream[13];
__location(0x0700) u8 result[16];
u8 choose_page(){result[1]=result[1]+1;*((u8*)0x2003)=12;return 4;}
u8 arg(u8 n){result[2]=result[2]+1;return n;}
void main(){
 u16 i;__ppu_off();__ppu_ctrl_set(0);
 for(i=0;i<256;i++){*((u8*)(0x0200+i))=(u8)(i+17);alternate[i]=(u8)(i+79);}
 for(i=0;i<16;i++)result[i]=0;
 stream[0]=248;stream[1]=249;stream[2]=128;stream[3]=227;
 stream[4]=8;stream[5]=9;stream[6]=129;stream[7]=0;stream[8]=255;
'''
base=[(i+17)&255 for i in range(256)];alternate=[(i+79)&255 for i in range(256)]
cases=[]
def add(name,body,changes=None,result=None,dma=None,addr=None):
 shadow=base.copy()
 if changes:
  for n,v in changes.items():shadow[n]=v
 cases.append((name,body,shadow,result or {},dma,addr))
add('clear','__oam_clear();',{i:240 for i in range(0,256,4)})
add('set','__sprite_set(3,40,55,128,227);',{12:55,13:128,14:227,15:40})
add('move','__sprite_move(3,40,55);',{12:55,15:40})
add('tile','__sprite_tile(3,128);',{13:128})
add('attr','__sprite_attr(3,227);',{14:227})
add('hide','__sprite_hide(3);',{12:240})
add('set-arguments-once','__sprite_set(arg(3),arg(40),arg(55),arg(128),arg(227));',{12:55,13:128,14:227,15:40},{2:5})
add('index-wrap','__sprite_set(67,40,55,128,227);',{12:55,13:128,14:227,15:40})
add('meta-page-cross','result[0]=__metasprite_draw(4,4,3,stream);',{16:252,17:128,18:227,19:252,20:12,21:129,22:0,23:12},{0:6})
add('meta-empty','result[0]=__metasprite_draw(4,4,3,stream+8);',{}, {0:4})
add('meta-last-slot','result[0]=__metasprite_draw(63,4,3,stream);',{252:252,253:128,254:227,255:252,0:12,1:129,2:0,3:12},{0:1})
add('dma-default','*((u8*)0x2003)=12;__oam_dma();',dma=base,addr=0)
add('dma-selected','*((u8*)0x2003)=12;__oam_dma_page(4);',dma=alternate,addr=0)
add('dma-argument-side-effect','__oam_dma_page(choose_page());',result={1:1},dma=alternate,addr=0)
# This raw-register case isolates the emulator's OAMADDR addressing from the compiler intrinsic.
rotated=[0]*256
for i,v in enumerate(alternate):rotated[(i+12)&255]=v
add('dma-raw-offset','*((u8*)0x2003)=12;*((u8*)0x4014)=4;',dma=rotated,addr=12)
records=[]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for name,body,expected,values,dma,addr in cases:
 folder=out/name;folder.mkdir(exist_ok=True);src=folder/'case.c';src.write_text(PREFIX+body+'\nresult[15]=165;while(1){}\n}\n',encoding='ascii');rom=folder/'case.nes'
 command=[str(compiler),str(src),'-I',str(REPOS/'kitaqfc/lib'),'--no-cache','--no-disasm','-o',str(rom)]
 r=subprocess.run(command,cwd=folder,capture_output=True,timeout=90);(folder/'build.txt').write_bytes(r.stdout+r.stderr)
 row=dict(name=name,compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),source_sha256=sha(src),build_exit=r.returncode,passed=False)
 if r.returncode==0:
  snapshot=folder/'snapshot.json';runtime=folder/'runtime.json'
  command=[str(emulator),'run',str(rom),'--frames','60','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
  r=subprocess.run(command,cwd=folder,capture_output=True,timeout=60);(folder/'runtime.txt').write_bytes(r.stdout+r.stderr);row['run_exit']=r.returncode
  if r.returncode==0:
   d=json.loads(snapshot.read_text(encoding='utf-8'));ram=d['bus']['ram'];actual=ram[0x200:0x300];result=ram[0x700:0x710];expected_result=[0]*16;expected_result[15]=165
   for i,v in values.items():expected_result[i]=v
   row.update(shadow=actual,expected_shadow=expected,result=result,expected_result=expected_result,oam=d['bus']['ppu']['oam'],oam_addr=d['bus']['ppu']['oam_addr'])
   row['passed']=actual==expected and result==expected_result and (dma is None or row['oam']==dma) and (addr is None or row['oam_addr']==addr)
   row.update(rom_sha256=sha(rom),expected_oam=dma,expected_oam_addr=addr)
 records.append(row);(out/'report.json').write_text(json.dumps(records,indent=2),encoding='utf-8');print(name,'PASS' if row['passed'] else 'FAIL',row.get('result'),row.get('oam_addr'),flush=True)
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
