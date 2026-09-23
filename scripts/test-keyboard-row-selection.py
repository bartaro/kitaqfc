"""Check executed keyboard port writes against an independent matrix counter.

KUROSAKI does not emulate the keyboard. This test verifies selected row/column,
settling delays, scan extent and cleanup from actual CPU bus traces, not keys.
Hardware protocol: https://www.nesdev.org/wiki/Family_basic_keyboard
"""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
ap.add_argument('--emulator',type=Path,required=True)
ap.add_argument('--report',type=Path)
ap.add_argument('--smoke',action='store_true')
opt=ap.parse_args();compiler=opt.compiler.resolve(strict=True);emulator=opt.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='''#include "intrinsics.h"
__location(0x0600) u8 result[64];
__location(0x0700) u8 phase;
void main(void){u8 row;u8 col;u8 i;__irq_disable();__ppu_ctrl_set(0);
result[3]=85;result[22]=170;for(i=4;i<22;i++)result[i]=204;
phase=1;__fkb_scan(result+4);
phase=2;i=32;for(row=0;row<10;row++){for(col=0;col<2;col++){
result[i]=__fkb_read_row_col(row,col);i++;}}
phase=3;result[1]=__fkb_detect();phase=4;result[0]=165;while(1){}}
'''
records=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-keyboard-') as temp:
    for name,flags in [('default',[]),('O0',['-O0']),('zp',['--zp-alloc']),('O0-zp',['-O0','--zp-alloc'])]:
        if opt.smoke and name!='default':continue
        folder=Path(temp)/name;folder.mkdir();src=folder/'case.c';rom=folder/'case.nes';trace=folder/'trace.jsonl';state=folder/'state.json'
        src.write_text(source,encoding='ascii')
        p=subprocess.run([str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--no-cache','--no-disasm']+flags,cwd=folder,capture_output=True,timeout=90)
        assert p.returncode==0,(p.stdout+p.stderr).decode(errors='replace')[-2000:]
        p=subprocess.run([str(emulator),'run',str(rom),'--frames','6','--snapshot',str(state)],cwd=folder,capture_output=True,timeout=90);assert p.returncode==0
        ram=json.loads(state.read_text())['bus']['ram']
        p=subprocess.run([str(emulator),'trace',str(rom),'--frames','6','--mem-read','--mem-write','--out',str(trace)],cwd=folder,capture_output=True,timeout=90);assert p.returncode==0
        phase=0;value=0;row=0;column=0;last_write=0;last_reset=None;resets=[];delays=[];selections={1:[],2:[],3:[]};final_outputs=[]
        with trace.open(encoding='utf-8') as stream:
            for line in stream:
                e=json.loads(line);kind=e['kind'];addr=e.get('addr');cycle=e.get('cpu_cycle',0)
                if kind=='mem.write' and addr==0x700:
                    if phase in selections:final_outputs.append(value)
                    phase=e['value']
                if phase not in selections:continue
                if kind=='mem.write' and addr==0x4016:
                    v=e['value']
                    if v&1:row=0;last_reset=cycle
                    elif value&2 and not v&2:row=(row+1)%10
                    if last_reset is not None and v==4:resets.append(cycle-last_reset);last_reset=None
                    value=v;column=(v>>1)&1;last_write=cycle
                if kind=='mem.read' and addr==0x4017:
                    selections[phase].append([row,column,bool(value&4)])
                    if value&4:delays.append(cycle-last_write)
        expected_scan=[[r,c,True] for r in range(9) for c in range(2)]
        expected_rows=[[r,c,True] for r in range(10) for c in range(2)]
        # The current no-keyboard model returns zero, so detection ends after its first read.
        expected_detect=[[9,0,True]]
        good=selections=={1:expected_scan,2:expected_rows,3:expected_detect} and final_outputs==[0,0,0]
        good=good and min(resets)>=16 and min(delays)>=50 and ram[0x600]==165 and ram[0x603]==85 and ram[0x616]==170 and ram[0x601]==0 and ram[0x604:0x616]==[0]*18
        record=dict(name=name,rom_sha256=sha(rom),selected=selections,expected=dict(scan=expected_scan,rows=expected_rows,detect=expected_detect),
                    final_outputs=final_outputs,reset_delays=resets,read_delays=delays,scan_bytes=ram[0x604:0x616],ram=[ram[0x600],ram[0x601],ram[0x603],ram[0x616]],passed=good)
        records.append(record);print(name,'PASS' if good else 'FAIL','row/col reads',selections[2],'detect',selections[3],flush=True)
report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),script_sha256=sha(Path(__file__)),records=records)
if opt.report:
    opt.report.parent.mkdir(parents=True,exist_ok=True);opt.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
