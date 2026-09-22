"""Verify direct PPU streams, address-latch variants and runtime wait completion."""
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
out=Path(tempfile.mkdtemp(prefix='fc-runtime-ppu-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
prefix='#include "intrinsics.h"\n#include "runtime.h"\n#include "runtime.c"\n__location(0x0700) u8 result[16];\n__location(0x0500) u8 source[256];\n'
init='u16 i;__ppu_off();__ppu_ctrl_set(0);nes_vram_queue_clear();for(i=0;i<16;i++)result[i]=0;for(i=0;i<256;i++)source[i]=(u8)(i+1);'
cases=[]
def add(name,body,result,vram=None,writes=0,variant='runtime'):
 cases.append((name,body,result,vram or {},writes,variant))
add('seek-resets-latch','PPUADDR=0x21;nes_ppu_seek(0x2380);PPUDATA=7;',{}, {0x2380:7},1)
add('write-256','nes_ppu_stream_write(0x2040,source,256);source[0]=99;',{}, {0x2040+i:(i+1)&255 for i in range(256)},256)
add('fill-256','nes_ppu_stream_fill(0x2040,7,256);',{}, {0x2040+i:7 for i in range(256)},256)
add('write-zero-seeks','PPUADDR=0x21;nes_ppu_stream_write(0x2340,source,0);PPUDATA=8;',{}, {0x2340:8},1)
add('fill-zero-seeks','PPUADDR=0x21;nes_ppu_stream_fill(0x2340,9,0);PPUDATA=8;',{}, {0x2340:8},1)
add('write-increment-32','__ppu_ctrl_set(4);nes_ppu_stream_write(0x2040,source,4);',{}, {0x2040+i*32:i+1 for i in range(4)},4)
add('fill-increment-32','__ppu_ctrl_set(4);nes_ppu_stream_fill(0x2140,7,3);',{}, {0x2140+i*32:7 for i in range(3)},3)
add('wait-nmi','nes_nmi_counter=0;__ppu_ctrl_set(0x80);nes_wait_nmi();__ppu_ctrl_set(0);result[0]=nes_nmi_counter;', {0:1})
add('wait-nmi-wrap','nes_nmi_counter=255;__ppu_ctrl_set(0x80);nes_wait_nmi();__ppu_ctrl_set(0);result[0]=nes_nmi_counter;', {})
add('wait-vblank-twice','nes_vblank_wait();result[0]=1;result[1]=(u8)(PPUSTATUS&0x80);nes_vblank_wait();result[2]=1;result[3]=(u8)(PPUSTATUS&0x80);result[4]=nes_nmi_counter;', {0:1,2:1})
add('pair-seek-prepared','__ppu_read_status();nes_ppu_seek_bytes(0x23,0x80);PPUDATA=7;',{}, {0x2380:7},1,'pair')
add('pair-seek-dirty-latch','__ppu_read_status();PPUADDR=0x21;nes_ppu_seek_bytes(0x23,0x80);PPUDATA=7;',{}, {0x2380:7},1,'pair')
add('pair-write-255','__ppu_read_status();nes_ppu_write_bytes(0x20,0x40,source,255);',{}, {0x2040+i:i+1 for i in range(255)},255,'pair')
add('pair-fill-255','__ppu_read_status();nes_ppu_fill(0x20,0x40,7,255);',{}, {0x2040+i:7 for i in range(255)},255,'pair')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=[]
for name,body,expected,pixels,writes,variant in cases:
 folder=out/name;folder.mkdir(exist_ok=True);source=folder/'case.c';source.write_text(prefix+('#include "ppu.c"\n' if variant=='pair' else '')+'void main(){'+init+body+'result[15]=165;while(1){}}',encoding='ascii')
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
 rows.append(row);(out/'report.json').write_text(json.dumps(dict(script_sha256=sha(Path(__file__)),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),library_sha256={n:sha(REPOS/'kitaqfc/lib'/n) for n in ['runtime.c','runtime.h','ppu.c','intrinsics.h']},cases=rows),indent=2),encoding='utf-8')
 print(name,'PASS' if row['passed'] else 'FAIL',row.get('result'),flush=True)
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
