"""Measure copied-payload runtime queue boundaries separately from the intrinsic queue."""
from pathlib import Path
import argparse,hashlib,json,subprocess
REPO=Path(__file__).resolve().parents[1];REPOS=REPO.parent
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--output-parent',type=Path)
args=p.parse_args()
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
import tempfile
out=Path(tempfile.mkdtemp(prefix='fc-runtime-queue-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
prefix='#include "intrinsics.h"\n#include "runtime.h"\n#include "runtime.c"\n__location(0x0700) u8 result[16];\n__location(0x0500) u8 source[255];\n'
init='u16 i;__ppu_off();__ppu_ctrl_set(0);nes_vram_queue_clear();for(i=0;i<16;i++)result[i]=0;for(i=0;i<255;i++)source[i]=(u8)(i+1);'
cases=[]
def add(name,body,result,vram=None,writes=None):cases.append((name,body,result,vram or {},writes))
add('clear','result[0]=nes_vram_queue_try_fill(0x2040,3,4);result[1]=nes_vram_queue_try_fill(0x2040,3,128);nes_vram_queue_clear();result[2]=nes_vram_queue_used;result[3]=nes_vram_queue_overflow;',{0:1},writes=0)
add('write-copy-now','result[0]=nes_vram_queue_try_write(0x2040,source,127);result[1]=nes_vram_queue_used;source[0]=99;nes_vram_queue_nmi_flush();result[2]=nes_vram_queue_used;', {0:1,1:130}, {0x2040+i:i+1 for i in range(127)},127)
add('fill-captured','u8 value;value=7;result[0]=nes_vram_queue_try_fill(0x2140,value,127);result[1]=nes_vram_queue_used;value=9;nes_vram_queue_nmi_flush();',{0:1,1:4},{0x2140+i:7 for i in range(127)},127)
add('zero-records','result[0]=nes_vram_queue_try_write(0x2040,source,0);result[1]=nes_vram_queue_used;result[2]=nes_vram_queue_try_fill(0x2140,7,0);result[3]=nes_vram_queue_used;nes_vram_queue_nmi_flush();result[4]=nes_vram_queue_used;',{0:1,1:3,2:1,3:7},writes=0)
add('exact-fit','result[0]=nes_vram_queue_try_write(0x2040,source,127);result[1]=nes_vram_queue_try_write(0x2140,source,59);result[2]=nes_vram_queue_used;result[3]=nes_vram_queue_try_write(0x2200,source,0);result[4]=nes_vram_queue_used;result[5]=nes_vram_queue_overflow;nes_vram_queue_nmi_flush();result[6]=nes_vram_queue_used;result[7]=nes_vram_queue_overflow;',{0:1,1:1,2:192,4:192,5:1,7:1},{**{0x2040+i:i+1 for i in range(127)},**{0x2140+i:i+1 for i in range(59)}},186)
add('latched-error','result[0]=nes_vram_queue_try_fill(0x2040,3,128);result[1]=nes_vram_queue_try_fill(0x2140,7,1);result[2]=nes_vram_queue_used;result[3]=nes_vram_queue_overflow;nes_vram_queue_nmi_flush();result[4]=nes_vram_queue_overflow;',{1:1,2:4,3:1,4:1},{0x2140:7},1)
for mode in ['write','fill']:
 for length in [128,255]:
  function='nes_vram_queue_try_'+mode;argument='source' if mode=='write' else '7'
  add(mode+'-reject-'+str(length),'result[0]='+function+'(0x2040,'+argument+','+str(length)+');result[1]=nes_vram_queue_used;result[2]=nes_vram_queue_overflow;',{2:1},writes=0)
add('independent-queues','__vramq_put(0x2200,9);result[0]=nes_vram_queue_try_fill(0x2140,7,1);nes_vram_queue_nmi_flush();result[1]=__vramq_len();result[2]=nes_vram_queue_used;nes_vram_queue_clear();result[3]=__vramq_len();',{0:1,1:4,3:4},{0x2140:7},1)
add('automatic-nmi','result[0]=nes_vram_queue_try_fill(0x2140,7,1);__ppu_ctrl_set(0x80);nes_wait_nmi();__ppu_ctrl_set(0);result[1]=nes_vram_queue_used;result[2]=nes_nmi_counter;',{0:1,2:1},{0x2140:7},1)
add('increment-32','__ppu_ctrl_set(4);result[0]=nes_vram_queue_try_write(0x2040,source,3);nes_vram_queue_nmi_flush();',{0:1},{0x2040:1,0x2060:2,0x2080:3},3)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=[]
for name,body,expected,pixels,writes in cases:
 folder=out/name;folder.mkdir(exist_ok=True);source=folder/'case.c';source.write_text(prefix+'void main(){'+init+body+'result[15]=165;while(1){}}',encoding='ascii')
 rom=folder/'case.nes';cmd=[str(compiler.resolve()),str(source.resolve()),'-I',str((REPOS/'kitaqfc/lib').resolve()),'--no-cache','--no-disasm','-o',str(rom.resolve())]
 run=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=60);(folder/'build.txt').write_bytes(run.stdout+run.stderr)
 row=dict(name=name,build_command=cmd,build_exit=run.returncode,source_sha256=sha(source),passed=False)
 if run.returncode==0:
  state=folder/'snapshot.json';runtime=folder/'runtime.json'
  cmd=[str(emulator.resolve()),'run',str(rom.resolve()),'--frames','60','--headless','--snapshot',str(state.resolve()),'--json',str(runtime.resolve())]
  run=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=60);(folder/'runtime.txt').write_bytes(run.stdout+run.stderr);row['run_exit']=run.returncode
  if run.returncode==0:
   data=json.loads(state.read_text(encoding='utf-8'));ram=data['bus']['ram'];ppu=data['bus']['ppu'];want=[0]*16;want[15]=165
   for i,v in expected.items():want[i]=v
   actual=ram[0x700:0x710];memory=[pixels.get(i,0) for i in range(0x2000,0x2400)]
   row.update(result=actual,expected_result=want,queue=ram[0x300:0x3C0],vram=ppu['vram'][0x2000:0x2400],expected_vram=memory,data_writes=ppu['data_writes'],expected_writes=writes)
   row['passed']=actual==want and row['vram']==memory and (writes is None or writes==ppu['data_writes'])
 rows.append(row);(out/'report.json').write_text(json.dumps(dict(script_sha256=sha(Path(__file__)),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),library_sha256={n:sha(REPOS/'kitaqfc/lib'/n) for n in ['runtime.c','runtime.h','intrinsics.h']},cases=rows),indent=2),encoding='utf-8')
 print(name,'PASS' if row['passed'] else 'FAIL',row.get('result'),flush=True)
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
