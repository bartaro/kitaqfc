"""Check OAM aliases, C helper variants and rotating-pool coordinate semantics."""
from pathlib import Path
import argparse,json,hashlib,subprocess
REPO=Path(__file__).resolve().parents[1];REPOS=REPO.parent
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--output-parent',type=Path)
args=p.parse_args()
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
import tempfile
out=Path(tempfile.mkdtemp(prefix='fc-oam-library-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
fixtures=[]
def add(name,includes,decl,body,shadow,result=None):fixtures.append((name,includes,decl,body,shadow,result or {}))
base=[0]*256
# Unused slots are hidden so reading OAM changes does not depend on startup content.
for i in range(0,256,4):base[i]=240
prefix='#include "intrinsics.h"\n__location(0x0700) u8 result[16];\n'
init='u16 i;__ppu_off();__ppu_ctrl_set(0);for(i=0;i<256;i++)*((u8*)(0x200+i))=0;__oam_clear();for(i=0;i<16;i++)result[i]=0;\n'
for name,body,changes in [
 ('alias-clear','nes_sprite_set(3,40,55,128,227);nes_oam_clear();',{13:128,14:227,15:40}),
 ('alias-set','nes_sprite_set(3,40,55,128,227);',{12:55,13:128,14:227,15:40}),
 ('alias-move','nes_sprite_set(3,16,17,128,227);nes_sprite_move(3,40,55);',{12:55,13:128,14:227,15:40}),
 ('alias-hide','nes_sprite_set(3,40,55,128,227);nes_sprite_hide(3);',{13:128,14:227,15:40})]:
 expected=base.copy()
 for i,v in changes.items():expected[i]=v
 add(name,'#include "nes_game.h"\n','',body+'nes_oam_dma();',expected)
stream='__location(0x05FE) u8 stream[9];\n'
prepare='stream[0]=248;stream[1]=249;stream[2]=128;stream[3]=227;stream[4]=8;stream[5]=9;stream[6]=129;stream[7]=0;stream[8]=255;'
for variant,includes in [('macro','#include "nes_game.h"\n'),('c','#include "runtime.c"\n#include "metasprite.c"\n')]:
 for mode in ['two','empty','wrap']:
  start=63 if mode=='wrap' else 4;expected=base.copy();values={0:start}
  if mode!='empty':
   for index,part in [(start,[252,128,227,252]),((start+1)%64,[12,129,0,12])]:expected[index*4:index*4+4]=part
   values[0]=(start+2)%64
  body=prepare+'result[0]=nes_metasprite_draw('+str(start)+',4,3,stream'+('+8' if mode=='empty' else '')+');'+('__oam_dma();' if variant=='macro' else 'nes_oam_dma(2);')
  add('meta-'+variant+'-'+mode,includes,stream,body,expected,values)
expected=base.copy();expected[0:8]=[240,128,0,32,248,129,64,40]
add('c-hide-range','#include "runtime.c"\n#include "metasprite.c"\n','', '__sprite_set(0,32,47,128,0);__sprite_set(1,40,47,129,64);nes_metasprite_hide_from(0,0);nes_metasprite_hide_from(1,1);__sprite_hide(0);nes_oam_dma(2);',expected)
# The configured pool positions are centers: an 8x8 sprite centered at (40,40)
# must have visible top-left (36,36), hence raw OAM Y=35.
fair_decl='#include "oam_fair_impl.h"\nu8 oam_fair_x[64];u8 oam_fair_y[64];u8 oam_fair_active[64];\n__location(0x0200) u8 oam_fair_shadow[256];u8 oam_fair_used;\n'
expected=base.copy();expected[:4]=[35,128,0,36]
body='for(i=0;i<64;i++)oam_fair_active[i]=0;oam_fair_phase=0;oam_fair_active[13]=1;oam_fair_x[13]=40;oam_fair_y[13]=40;oam_fair_limit=1;oam_fair_used=0;oam_fair_tile=128;oam_fair_attr=0;OAM_FairDraw();result[0]=oam_fair_phase;result[1]=oam_fair_drawn;result[2]=oam_fair_used;__oam_dma();'
add('fair-center','',''+fair_decl,body,expected,{0:13,1:1,2:4})
# Capacity and rotation checks keep the byte cursor aligned and preserve earlier entries.
all_active='for(i=0;i<64;i++){oam_fair_active[i]=1;oam_fair_x[i]=(u8)(i+4);oam_fair_y[i]=40;}oam_fair_phase=0;oam_fair_tile=128;oam_fair_attr=0;'
for name,used,limit,count in [('limit-zero',4,0,0),('append-two',4,2,2),('last-usable-slot',248,64,1),('full-cursor',252,64,0)]:
 expected=base.copy()
 for j in range(count):expected[used+4*j:used+4*j+4]=[35,128,0,13+j]
 b=all_active+'oam_fair_used='+str(used)+';oam_fair_limit='+str(limit)+';OAM_FairDraw();result[0]=oam_fair_phase;result[1]=oam_fair_drawn;result[2]=oam_fair_used;__oam_dma();'
 add('fair-'+name,'',fair_decl,b,expected,{0:13,1:count,2:used+4*count})
expected=base.copy();expected[:4]=[35,128,0,0]
cycle=all_active+'u8 round;u8 id;oam_fair_limit=1;for(i=0;i<64;i++)visits[i]=0;for(round=0;round<64;round++){oam_fair_used=0;OAM_FairDraw();id=oam_fair_shadow[3];visits[id]=visits[id]+1;}for(i=0;i<64;i++){if(visits[i]!=1)result[3]=result[3]+1;}result[0]=oam_fair_phase;result[1]=oam_fair_drawn;result[2]=oam_fair_used;__oam_dma();'
add('fair-cycle-64','',fair_decl+'__location(0x0600) u8 visits[64];',cycle,expected,{0:0,1:1,2:4,3:0})
records=[]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for name,includes,decl,body,expected,values in fixtures:
 folder=out/name;folder.mkdir(exist_ok=True);source=folder/'case.c';source.write_text(prefix+includes+decl+'void main(){'+init+body+'result[15]=165;while(1){}}\n',encoding='ascii');rom=folder/'case.nes'
 command=[str(compiler),str(source),'-I',str(REPOS/'kitaqfc/lib'),'--no-cache','--no-disasm','-o',str(rom)]
 r=subprocess.run(command,cwd=folder,capture_output=True,timeout=90);(folder/'build.txt').write_bytes(r.stdout+r.stderr)
 row=dict(name=name,build_command=command,build_exit=r.returncode,source_sha256=sha(source),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),passed=False)
 if r.returncode==0:
  snapshot=folder/'snapshot.json';runtime=folder/'runtime.json';r=subprocess.run([str(emulator),'run',str(rom),'--frames','60','--headless','--snapshot',str(snapshot),'--json',str(runtime)],cwd=folder,capture_output=True,timeout=60);(folder/'runtime.txt').write_bytes(r.stdout+r.stderr);row['run_exit']=r.returncode
  if r.returncode==0:
   state=json.loads(snapshot.read_text(encoding='utf-8'));ram=state['bus']['ram'];actual=ram[0x200:0x300];result=ram[0x700:0x710];want=[0]*16;want[15]=165
   for i,v in values.items():want[i]=v
   row.update(shadow=actual,expected_shadow=expected,result=result,expected_result=want,oam=state['bus']['ppu']['oam'],rom_sha256=sha(rom));row['passed']=actual==expected and result==want and row['oam']==expected
 records.append(row);(out/'report.json').write_text(json.dumps(dict(script_sha256=sha(Path(__file__)),library_sha256={n:sha(REPO/'lib'/n) for n in ['intrinsics.h','core.h','nes_game.h','runtime.h','runtime.c','metasprite.c','oam_fair.h','oam_fair_impl.h']},cases=records),indent=2),encoding='utf-8');print(name,'PASS' if row['passed'] else 'FAIL',row.get('result'),flush=True)
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
