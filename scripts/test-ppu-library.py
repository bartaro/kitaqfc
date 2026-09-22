"""Execute PPU library calls, restoration, nametable bounds and runtime coexistence."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
REPO=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--output-parent',type=Path)
a=p.parse_args()
if a.output_parent:a.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-ppu-implemented-',dir=a.output_parent)).resolve()
compiler=a.compiler.resolve();emulator=a.emulator.resolve();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
colors=[15,1,2,3,15,5,6,7,15,9,10,11,15,13,14,16,15,17,18,19,15,21,22,23,15,25,26,27,15,29,30,32]
palette=colors[:16]+[0 if i%4==0 else v for i,v in enumerate(colors[16:])]
prefix='#include "runtime.h"\n#include "runtime.c"\n#include "ppu.c"\n__location(0x0600) u8 result[8];\n__prg_rom u8 colors[32]={'+','.join(map(str,colors))+'};\n'
cases=[
 ('screen-off','__ppu_ctrl_set(4);__ppu_mask_set(255);nes_ppu_screen_off();result[1]=__ppu_mask_get();',dict(ctrl=4,mask=231,data_writes=0),[0,231,0,0,0,0,0,165]),
 ('screen-on','nes_ppu_screen_on(0x80,0x0A);result[1]=__ppu_ctrl_get();result[2]=__ppu_mask_get();',dict(ctrl=128,mask=10,data_writes=0),[0,128,10,0,0,0,0,165]),
 ('palette-increment-32','__ppu_ctrl_set(4);nes_ppu_load_palette(colors);result[1]=__ppu_ctrl_get();',dict(ctrl=4,palette=palette,data_writes=32),[0,4,0,0,0,0,0,165]),
 ('null-palette','__ppu_ctrl_set(4);nes_ppu_load_palette(0);result[1]=__ppu_ctrl_get();',dict(ctrl=4,data_writes=0),[0,4,0,0,0,0,0,165]),
 ('invalid-base','__ppu_ctrl_set(4);nes_ppu_clear_nt(0x2001,7,255);nes_ppu_clear_nt(0x3000,7,255);result[1]=__ppu_ctrl_get();',dict(ctrl=4,data_writes=0),[0,4,0,0,0,0,0,165]),
 ('runtime-coexistence','PPUADDR=0x21;nes_ppu_seek_bytes(0x23,0x80);PPUDATA=7;nes_ppu_seek(0x2380);result[6]=PPUDATA;result[1]=PPUDATA;nes_ppu_fill(0x23,0x81,9,1);nes_ppu_write_bytes(0x23,0x82,colors,1);nes_ppu_seek(0x2381);result[6]=PPUDATA;result[2]=PPUDATA;result[3]=PPUDATA;',dict(ctrl=0,data_writes=3),[0,7,9,15,0,0,0,165]),
]
for nt in range(4):
    base=0x2000+nt*0x400
    body=f'__ppu_ctrl_set(4);nes_ppu_clear_nt({base},7,228);result[1]=__ppu_ctrl_get();__ppu_ctrl_set(0);nes_ppu_seek({base});result[6]=PPUDATA;for(i=0;i<960;i++)if(PPUDATA!=7)result[0]=1;for(i=0;i<64;i++)if(PPUDATA!=228)result[0]=2;'
    cases.append(('nametable-'+str(nt),body,dict(ctrl=0,data_writes=1024),[0,4,0,0,0,0,0,165]))
rows=[]
for variant,flags in [('default',[]),('unoptimized',['-O0']),('no-inline',['--no-small-inline']),('fastcall',['--fastcall-v2'])]:
 for name,body,expected_ppu,expected in cases:
    folder=out/variant/name;folder.mkdir(parents=True)
    source=folder/'case.c';rom=folder/'case.nes';snapshot=folder/'snapshot.json'
    source.write_text(prefix+'void main(){u16 i;u8 dummy;__ppu_mask_set(0);__ppu_ctrl_set(0);for(i=0;i<8;i++)result[i]=0;'+body+'result[7]=165;while(1){}}',encoding='ascii')
    build=subprocess.run([str(compiler),str(source),'-I',str(REPO/'lib'),'--no-cache','--no-disasm','-o',str(rom)]+flags,cwd=folder,capture_output=True,timeout=90)
    (folder/'build.txt').write_bytes(build.stdout+build.stderr)
    if build.returncode:raise RuntimeError((build.stdout+build.stderr).decode(errors='replace')[-1800:])
    run=subprocess.run([str(emulator),'run',str(rom),'--frames','60','--headless','--snapshot',str(snapshot)],cwd=folder,capture_output=True,timeout=90)
    assert run.returncode==0
    state=json.loads(snapshot.read_text(encoding='utf-8'));ppu=state['bus']['ppu']
    actual=state['bus']['ram'][0x600:0x608];actual_ppu={k:ppu[k] for k in expected_ppu}
    passed=actual==expected and actual_ppu==expected_ppu and ppu['data_writes_while_rendering']==0
    rows.append(dict(name=name,variant=variant,passed=passed,actual=actual,expected=expected,ppu=actual_ppu,expected_ppu=expected_ppu,unsafe_ppu_writes=ppu['data_writes_while_rendering'],source_sha256=sha(source),rom_sha256=sha(rom)))
    snapshot.unlink()
    print(name,variant,'PASS' if passed else 'FAIL',actual,flush=True)
    (out/'report.json').write_text(json.dumps(dict(cases=rows,passed=all(r['passed'] for r in rows),script_sha256=sha(Path(__file__)),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),library_sha256={name:sha(REPO/'lib'/name) for name in ['ppu.h','ppu.c','intrinsics.h','runtime.c','runtime.h']}),indent=2),encoding='utf-8')
print('Report:',out/'report.json')
raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
