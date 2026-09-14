"""Exercise FC bit operations across byte and 256-byte page boundaries.

Pass --compiler and --emulator to select a KITAQFC compiler and KUROSAKI CLI.
Evidence is retained under --output-parent, or the system temporary directory.
These original fixtures verify CPU RAM behavior, not physical hardware.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import tempfile


def digest(path):
    """Bind evidence to its exact input and output bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, directory, log):
    """Bound each external process and preserve diagnostics without locale decoding."""
    try:
        process=subprocess.run(command,cwd=directory,capture_output=True,timeout=40)
    except subprocess.TimeoutExpired as error:
        log.write_bytes((error.stdout or b'')+(error.stderr or b''))
        raise
    log.write_bytes(process.stdout+process.stderr)
    return process.returncode


def fixture(bit, dynamic):
    """Observe both the intended byte and the incorrect low-page alias after each update."""
    offset=bit//8
    alias=offset&255
    mask=1<<(bit&7)
    index='read_index()' if dynamic else str(bit)
    initial_alias=f'data[{alias}]=0x80;' if alias!=offset else ''
    source='''// Original bit-index regression fixture with a deliberately unaligned base.
__location(0x0301) u8 data[768];
__location(0x0700) u8 result[8];
u16 requested;
u16 read_index() { return requested; }
u8 __bit_test(u8* base, u16 bit);
void __bit_set(u8* base, u16 bit);
void __bit_clear(u8* base, u16 bit);
void __bit_toggle(u8* base, u16 bit);
void main() {
result[7]=0;
''' + f'''requested={bit}; data[{offset}]=0; {initial_alias}
__bit_set(data,{index});
result[0]=data[{offset}]; result[1]=data[{alias}];
result[2]=__bit_test(data,{index});
__bit_clear(data,{index});
result[3]=data[{offset}]; result[4]=data[{alias}];
__bit_toggle(data,{index});
result[5]=data[{offset}]; result[6]=data[{alias}];
result[7]=0xA5; while(1) {{ }}
}}
'''
    expected=[mask,128 if alias!=offset else mask,mask,0,128 if alias!=offset else 0,
        mask,128 if alias!=offset else mask,165]
    return source,expected


def main():
    """Test constant inline/helper dispatch and runtime helper dispatch independently."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler',required=True,type=Path)
    parser.add_argument('--emulator',required=True,type=Path)
    parser.add_argument('--output-parent',type=Path)
    args=parser.parse_args()
    compiler=args.compiler.resolve(strict=True)
    emulator=args.emulator.resolve(strict=True)
    if args.output_parent: args.output_parent.mkdir(parents=True,exist_ok=True)
    output=Path(tempfile.mkdtemp(prefix='fc-bit-indices-',dir=args.output_parent)).resolve()
    report=dict(compiler=str(compiler),compiler_sha256=digest(compiler),emulator=str(emulator),
        emulator_sha256=digest(emulator),cases=[],scope='Original ROMs and KUROSAKI CPU RAM snapshots; no physical hardware.')
    for bit in (0,7,8,2047,2048,2055,4095,4096,6143):
        for dynamic in (False,True):
            name=f'bit-{bit}-'+('runtime' if dynamic else 'constant')
            directory=output/name
            directory.mkdir()
            source=directory/'case.c'
            text,expected=fixture(bit,dynamic)
            source.write_text(text,encoding='ascii')
            rom=directory/'case.nes'
            command=[str(compiler),str(source),'--mapper=nrom','--no-cache','--no-disasm','-O1','-o',str(rom)]
            rc=run(command,directory,directory/'compile.log')
            case=dict(name=name,bit=bit,dynamic=dynamic,expected=expected,passed=False,compile_exit=rc,
                command=command,source_sha256=digest(source))
            if rc==0:
                snapshot=directory/'snapshot.json'
                run_command=[str(emulator),'run',str(rom),'--frames','8','--max-instructions','200000',
                    '--headless','--snapshot',str(snapshot)]
                run_exit=run(run_command,directory,directory/'run.log')
                case['run_exit']=run_exit
                if run_exit==0:
                    state=json.loads(snapshot.read_text(encoding='utf-8'))
                    ram=state['bus']['ram']
                    if len(ram)!=2048: raise RuntimeError('Invalid CPU RAM snapshot: '+name)
                    actual=ram[0x700:0x708]
                    case.update(actual=actual,passed=actual==expected,rom_sha256=digest(rom),
                        snapshot_sha256=digest(snapshot),run_command=run_command)
            report['cases'].append(case)
            (output/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    failures=[case['name'] for case in report['cases'] if not case['passed']]
    print(json.dumps(dict(report=str(output/'report.json'),cases=len(report['cases']),failures=failures)))
    if failures: raise SystemExit(1)


if __name__=='__main__':
    main()
