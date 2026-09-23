"""Check native FDS overlay placement without executing external firmware.

Use --compiler to test a candidate. Temporary outputs are removed on exit.
This regression checks packaging and board constraints, not BIOS disk loading.
"""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
ap.add_argument('--report',type=Path)
args=ap.parse_args();compiler=args.compiler.resolve(strict=True)
source='''#include "intrinsics.h"
__location(0x0600) u8 results[4];
#pragma bank 2
u8 first(void) { return 21; }
u8 second(void) { return 34; }
#pragma bank 0
void main(void) {
    results[0]=0;
    if (results[0]) { results[1]=first(); results[2]=second(); }
    while(1) {}
}
'''
records=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-fds-placement-') as temp:
    root=Path(temp)
    for mode,flags,diagnostic in [
        ('native',['--mapper=fds'],None),
        ('native-explicit',['--mapper=fds','--fds-layout=fds32'],None),
        ('legacy',['--mapper=fds','--fds-layout=legacy'],'does not support banked PRG'),
        ('no-export',['--mapper=fds','--fds-no-auto-overlay'],'KQFC2505'),
        ('nrom',['--mapper=nrom'],'does not support banked PRG'),
        ('cnrom',['--mapper=cnrom'],'does not support banked PRG'),
        ('uxrom',['--mapper=uxrom'],None),
        ('mmc3',['--mapper=mmc3'],None)]:
        for optimize in [True,False]:
            name=mode+('-default' if optimize else '-O0');folder=root/name;folder.mkdir()
            src=folder/'case.c';src.write_text(source,encoding='ascii')
            rom=folder/('case.fds' if flags[0]=='--mapper=fds' else 'case.nes')
            cmd=[str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--no-cache','--no-disasm']+flags+([] if optimize else ['-O0'])
            # No approval-screen boot stub is needed for this packaging-only fixture.
            if flags[0]=='--mapper=fds':cmd.append('--fds-no-license-bypass')
            p=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90)
            row=dict(name=name,exit_code=p.returncode,expected_diagnostic=diagnostic,passed=False)
            if diagnostic:
                row['passed']=p.returncode!=0 and diagnostic.encode() in p.stdout+p.stderr and not rom.exists()
            else:
                row['passed']=p.returncode==0 and rom.is_file()
                if row['passed']:
                    data=rom.read_bytes();row['rom_sha256']=hashlib.sha256(data).hexdigest()
                    if mode.startswith('native'):
                        # Parse the FDS block structure independently of the compiler.
                        assert data[:4]==b'FDS\x1a';side=data[16:65516];offset=58;files=[]
                        for _ in range(side[57]):
                            assert side[offset]==3
                            head=side[offset:offset+16];size=int.from_bytes(head[13:15],'little')
                            assert side[offset+16]==4
                            files.append(dict(id=head[2],address=int.from_bytes(head[11:13],'little'),size=size))
                            offset+=17+size
                        overlay=[f for f in files if f['id']==32]
                        row['overlay']=overlay
                        row['passed']=len(overlay)==1 and overlay[0]['address']==0x6000 and 0<overlay[0]['size']<=16384
            records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',flush=True)
            if not row['passed']:print((p.stdout+p.stderr).decode(errors='replace')[-2500:],flush=True)
report=dict(compiler_sha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),records=records)
if args.report:
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
