"""Check ordinary and explicit far ROM access across MMC3 banks.

Original test fixtures, copyright (c) 2026 DAISUKE OBA. MIT License.
Requires the KITAQFC compiler and KUROSAKI command-line emulator.
"""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler',type=Path,default=Path(__file__).resolve().parents[1]/'kitaqfc.exe')
    parser.add_argument('--emulator',type=Path,required=True)
    parser.add_argument('--output-parent',type=Path)
    args=parser.parse_args()
    compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
    parent=args.output_parent.resolve() if args.output_parent else None
    if parent:parent.mkdir(parents=True,exist_ok=True)
    out=Path(tempfile.mkdtemp(prefix='fc-near-rom-',dir=parent))
    fixtures=json.loads(Path(__file__).with_name('near-rom-fixtures.json').read_text(encoding='utf-8'))
    rows=[];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    for fixture in fixtures:
        for variant,flags in [('default',[]),('unoptimized',['-O0']),('no-inline',['--no-small-inline']),('fastcall',['--fastcall-v2'])]:
            folder=out/fixture['name']/variant;folder.mkdir(parents=True)
            source=folder/'case.c';rom=folder/'case.nes';snapshot=folder/'state.json'
            source.write_text(fixture['source'],encoding='utf-8')
            build=subprocess.run([str(compiler),str(source),'--mapper=mmc3','--no-cache','--no-disasm','-o',str(rom),*flags],cwd=folder,capture_output=True,timeout=120)
            (folder/'build.log').write_bytes(build.stdout+build.stderr)
            actual=[];run_exit=None
            if build.returncode==0:
                run=subprocess.run([str(emulator),'run',str(rom),'--frames','30','--headless','--snapshot',str(snapshot)],cwd=folder,capture_output=True,timeout=120)
                run_exit=run.returncode;(folder/'run.log').write_bytes(run.stdout+run.stderr)
                if run.returncode==0:
                    ram=json.loads(snapshot.read_text(encoding='utf-8'))['bus']['ram']
                    actual=[ram[0x600+2*i]+256*ram[0x601+2*i] for i in range(len(fixture['expected']))]
            row=dict(name=fixture['name'],variant=variant,build_exit=build.returncode,run_exit=run_exit,actual=actual,expected=fixture['expected'],passed=build.returncode==0 and run_exit==0 and actual==fixture['expected'],source_sha256=sha(source))
            if rom.exists():row['rom_sha256']=sha(rom)
            rows.append(row);print(fixture['name'],variant,'PASS' if row['passed'] else 'FAIL',flush=True)
    report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),script_sha256=sha(Path(__file__)),fixtures_sha256=sha(Path(__file__).with_name('near-rom-fixtures.json')),cases=rows,passed=all(r['passed'] for r in rows))
    (out/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('Report:',out/'report.json')
    return 0 if report['passed'] else 1

if __name__=='__main__':raise SystemExit(main())
