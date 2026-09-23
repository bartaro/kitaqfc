"""Check named FDS calls' argument evaluation and callback type validation.

Callbacks remain in common bank zero: these tests exercise the CPU call contract,
not BIOS overlay loading. A private zero firmware fixture is removed on exit.
"""
from pathlib import Path
import argparse,hashlib,json,os,subprocess,tempfile
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
ap.add_argument('--emulator',type=Path,required=True)
ap.add_argument('--report',type=Path)
args=ap.parse_args();compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
prefix='''#pragma fixed_bank 0
#include "nes_game.h"
__location(0x0600) u8 result[8];
u8 bank_value;
u8 choose_bank(void) { result[0]++; return 0; }
u8 target(void) { result[1]++; result[2]=result[0]; return 183; }
'''
records=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-fds-call-') as temp:
    root=Path(temp);firmware=root/'zero-firmware.bin';firmware.write_bytes(bytes(8192))
    environment={**os.environ,'KUROSAKI_DISKSYS_ROM':str(firmware)}
    for api in ['__fds_overlay_farcall','__fds_farcall','nes_fds_farcall','__farcall']:
        for optimize in [True,False]:
            for form,expression in [('literal','0'),('variable','bank_value'),('side-effect','choose_bank()')]:
                name=api+'-'+form+('-default' if optimize else '-O0');folder=root/name;folder.mkdir()
                source=prefix+'''void main(void) {
    __irq_disable(); result[0]=0;result[1]=0;result[2]=0;bank_value=0;
    result[3]='''+api+'('+expression+''',target);
    result[4]=165;while(1){}
}
'''
                src=folder/'case.c';src.write_text(source,encoding='ascii');rom=folder/'case.fds';statepath=folder/'state.json'
                cmd=[str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--mapper=fds','--fds-no-license-bypass','--no-cache','--no-disasm']+([] if optimize else ['-O0'])
                built=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90)
                expected=[int(form=='side-effect'),1,int(form=='side-effect'),183,165]
                row=dict(name=name,source_sha256=sha(src),build_exit=built.returncode,expected=expected,passed=False)
                if built.returncode==0:
                    run=subprocess.run([str(emulator),'run',str(rom),'--frames','8','--snapshot',str(statepath)],capture_output=True,cwd=folder,env=environment,timeout=90)
                    row.update(run_exit=run.returncode,rom_sha256=sha(rom))
                    if run.returncode==0:
                        state=json.loads(statepath.read_text());actual=state['bus']['ram'][0x600:0x605]
                        row.update(actual=actual,passed=actual==expected and not state['cpu']['stopped'])
                records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',row.get('actual'),flush=True)
                if built.returncode:print((built.stdout+built.stderr).decode(errors='replace')[-1500:],flush=True)
            for form,declaration,target,diagnostic in [
                ('parameter','u8 wrong(u8 value){return value;}','wrong','no parameters'),
                ('data-symbol','u16 pointer;','pointer','declared function name')]:
                name=api+'-'+form+('-default' if optimize else '-O0');folder=root/name;folder.mkdir()
                src=folder/'case.c';src.write_text(prefix+declaration+'\nvoid main(void){'+api+'(0,'+target+');while(1){}}',encoding='ascii')
                rom=folder/'case.fds';cmd=[str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--mapper=fds','--fds-no-license-bypass','--no-cache','--no-disasm']+([] if optimize else ['-O0'])
                p=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90)
                row=dict(name=name,source_sha256=sha(src),build_exit=p.returncode,expected_diagnostic=diagnostic,passed=p.returncode!=0 and diagnostic.encode() in p.stdout+p.stderr and not rom.exists())
                records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',flush=True)
report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),records=records)
if args.report:
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
