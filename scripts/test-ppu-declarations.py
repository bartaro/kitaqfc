"""Check absent PPU function bodies and the actual state of implemented alternatives."""
from pathlib import Path
import argparse, hashlib, json, subprocess, tempfile

REPO=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--output-parent',type=Path)
args=p.parse_args()
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-ppu-declarations-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def build(name,code):
    folder=out/name;folder.mkdir();source=folder/'case.c';source.write_text(code,encoding='ascii')
    rom=folder/'case.nes'
    command=[str(compiler),str(source),'-I',str(REPO/'lib'),'--no-cache','--no-disasm','-o',str(rom)]
    run=subprocess.run(command,capture_output=True,timeout=60,cwd=folder)
    (folder/'build.txt').write_bytes(run.stdout+run.stderr)
    row=dict(name=name,source_sha256=sha(source),build_command=command,build_exit=run.returncode,passed=False)
    return folder,rom,row,(run.stdout+run.stderr).decode('utf-8',errors='replace')

declarations=[]
for name,call in [('nes_ppu_screen_off','nes_ppu_screen_off();'),('nes_ppu_screen_on','nes_ppu_screen_on(0x80,0x0A);'),('nes_ppu_load_palette','nes_ppu_load_palette(palette);'),('nes_ppu_clear_nt','nes_ppu_clear_nt(0x2000,0,0);')]:
    folder,rom,row,diagnostic=build(name,'#include "ppu.h"\nunsigned char palette[32];\nvoid main(){'+call+'while(1){}}\n')
    expected='unresolved symbol: '+name
    row.update(expected_diagnostic=expected,diagnostic_present=expected in diagnostic,rom_created=rom.exists())
    row['passed']=row['build_exit']!=0 and row['diagnostic_present'] and not row['rom_created']
    declarations.append(row);print(name,'expected rejection' if row['passed'] else 'FAIL',flush=True)

bg=[15,1,2,3,15,5,6,7,15,9,10,11,15,13,14,16]
sp=[15,17,18,19,15,21,22,23,15,25,26,27,15,29,30,32]
# Palette addresses 3F10/14/18/1C alias the corresponding background entries.
palette=bg+[0 if i%4==0 else value for i,value in enumerate(sp)]
prefix='#include "intrinsics.h"\n__prg_rom u8 colors[32]={'+','.join(map(str,bg+sp))+'};\n'
cases=[
    ('screen-off','__ppu_ctrl_set(0x84);__ppu_mask_set(0x1E);__ppu_off();',{'ctrl':132,'mask':0}),
    ('screen-configured','__scroll_set(0,0);__ppu_ctrl_set(0x80);__ppu_mask_set(0x0A);',{'ctrl':128,'mask':10}),
    ('palette-32','__ppu_off();__ppu_ctrl_set(0);__palette_bg_load(colors);__palette_sp_load(colors+16);',{'palette':palette,'data_writes':32}),
    ('nametable-clear','__ppu_off();__ppu_ctrl_set(0);__nametable_rect_nt(0,0,0,32,30,7);__vram_fill(0x23C0,255,64);__nametable_rect_nt(0,0,0,32,30,0);__vram_fill(0x23C0,0,64);',{'nametable_0':[0]*1024,'data_writes':2048})
]
alternatives=[]
for name,body,expected in cases:
    folder,rom,row,diagnostic=build(name,prefix+'void main(){'+body+'while(1){}}\n')
    if row['build_exit']==0:
        snapshot=folder/'snapshot.json';runtime=folder/'runtime.json'
        command=[str(emulator),'run',str(rom),'--frames','60','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
        run=subprocess.run(command,capture_output=True,timeout=60,cwd=folder)
        (folder/'runtime.txt').write_bytes(run.stdout+run.stderr);row['runtime_exit']=run.returncode
        if run.returncode==0:
            ppu=json.loads(snapshot.read_text(encoding='utf-8'))['bus']['ppu']
            actual={key:ppu['vram'][0x2000:0x2400] if key=='nametable_0' else ppu[key] for key in expected}
            row.update(actual=actual,expected=expected,passed=actual==expected)
    alternatives.append(row);print(name,'PASS' if row['passed'] else 'FAIL',flush=True)
report=dict(script_sha256=sha(Path(__file__)),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),library_sha256={name:sha(REPO/'lib'/name) for name in ['ppu.h','intrinsics.h']},declarations=declarations,alternatives=alternatives)
(out/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in declarations+alternatives) else 1)
