"""Check boot-file thresholds and restoration with original FDS fixture code.

No external BIOS is used; passing does not establish physical-disk behavior.
"""
from pathlib import Path
import argparse,hashlib,json,os,subprocess,tempfile
from fds_abi_fixture import disk_files,loadfiles_firmware
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
ap.add_argument('--emulator',type=Path,required=True)
ap.add_argument('--report',type=Path)
args=ap.parse_args();compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
split=[dict(id=7,load='0x6000',size=16384,type=0,boot=True),
       dict(id=8,load='0xA000',size=16384,type=0,boot=True),
       dict(id=9,load=0,size=8192,type=1,boot=True)]
source='''#include "nes_game.h"
__location(0x600) u8 result[4];
__location(0x700) u8 fixture_log[256];
#pragma fixed_bank 2
u8 callback(void){return 42;}
#pragma fixed_bank 1
void main(void){__irq_disable();result[0]=__fds_farcall(2,callback);
result[1]=__fds_current_bank();result[3]=165;while(1){}}
'''
records=[]
specs=[
    ('default-trigger',None,[],None,True,3,[32,0]),
    ('custom-trigger',split,[],None,True,10,[32,7]),
    ('custom-no-trigger',split,['--fds-no-license-bypass'],None,True,9,[32,7]),
    ('guard-disabled',None,['--fds-no-overlay-guard','--fds-no-license-bypass'],None,True,2,[32,0]),
    ('unsplit', [dict(id=0,load='0x6000',size=32768,type=0,boot=True)],['--fds-no-license-bypass'],'KQFC2516',True,None,None),
    ('reserved-default-id',None,['--fds-overlay-start-id=2','--fds-no-license-bypass'],'KQFC2508',True,None,None),
    ('trigger-collision',None,['--fds-overlay-start-id=3'],'KQFC2517',True,None,None),
    ('nonboot-below-boot',split+[dict(id=6,load='0x7000',size=1,type=0,boot=False)],['--fds-no-license-bypass'],'KQFC2518',True,None,None),
    ('trigger-overflow',[dict(id=254,load='0x6000',size=32768,type=0,boot=True)],[],'KQFC2517',False,None,None)]
with tempfile.TemporaryDirectory(prefix='kitaqfc-fds-boot-') as temp:
    for name,manifest,flags,diagnostic,overlay,boot_id,expected_ids in specs:
        for optimize in [True,False]:
            mode=name+('-default' if optimize else '-O0');folder=Path(temp)/mode;folder.mkdir()
            src=folder/'case.c';src.write_text(source if overlay else 'void main(void){while(1){}}',encoding='ascii')
            rom=folder/'case.fds';snapshot=folder/'state.json';bios=folder/'fixture.bin'
            cmd=[str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--mapper=fds','--fds-overlay-trim','--no-cache','--no-disasm']+flags+([] if optimize else ['-O0'])
            if manifest:
                meta=folder/'manifest.json';meta.write_text(json.dumps(dict(files=manifest)),encoding='ascii');cmd+=['--fds-meta='+str(meta)]
            p=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90)
            row=dict(name=mode,build_exit=p.returncode,passed=False)
            if diagnostic:
                row.update(expected_diagnostic=diagnostic,passed=p.returncode!=0 and diagnostic.encode() in p.stdout+p.stderr and not rom.exists())
            elif p.returncode==0:
                files=disk_files(rom);header=rom.read_bytes()[16:72]
                bios.write_bytes(loadfiles_firmware(files))
                p=subprocess.run([str(emulator),'run',str(rom),'--frames','60','--snapshot',str(snapshot)],cwd=folder,capture_output=True,env={**os.environ,'KUROSAKI_DISKSYS_ROM':str(bios)},timeout=90)
                if p.returncode==0:
                    state=json.loads(snapshot.read_text());ram=state['bus']['ram'];ids=ram[0x780:0x780+ram[0x700]]
                    row.update(actual=ram[0x600:0x604],ids=ids,boot_id=header[25],passed=ram[0x600:0x604]==[42,1,0,165] and ids==expected_ids and header[25]==boot_id and not next(f for f in files if f['id']==32)['boot'])
            records.append(row);print(mode,'PASS' if row['passed'] else 'FAIL',row,flush=True)
            if not row['passed']:print((p.stdout+p.stderr).decode(errors='replace')[-2400:],flush=True)
report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),records=records)
if args.report:
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
