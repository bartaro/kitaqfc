"""Compile and execute signed-comparison/loop regressions using KUROSAKI.

Python 3 example:
  python scripts/test-signed-comparisons.py --compiler kitaqfc.exe --emulator ../kurosaki/kurosaki.exe
Sources, ROMs, snapshots and logs are retained in a fresh output directory.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


def sha256(path):
    """Identify the exact executable or generated file used by a test run."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def execute_rom(compiler, emulator, output, name, source_text, optimize_loop):
    """Build a fresh ROM, execute a bounded frame count, and return its CPU RAM."""
    directory = output/name
    directory.mkdir()
    source = directory/'case.c'
    source.write_text(source_text, encoding='ascii')
    rom = directory/'case.nes'
    command = [str(compiler), str(source), '--mapper=nrom', '--no-cache', '--no-disasm', '-O1',
               '--loop-lowering' if optimize_loop else '--no-loop-lowering', '-o', str(rom)]
    build = subprocess.run(command, cwd=directory, capture_output=True, text=True,
                           encoding='utf-8', errors='replace', timeout=30)
    (directory/'compile.log').write_text(build.stdout+build.stderr, encoding='utf-8')
    if build.returncode != 0 or not rom.exists():
        raise RuntimeError('Compilation failed; inspect '+str(directory/'compile.log'))
    snapshot = directory/'snapshot.json'
    run_command = [str(emulator), 'run', str(rom), '--frames', '12', '--max-instructions', '200000',
                   '--headless', '--snapshot', str(snapshot)]
    run = subprocess.run(run_command, cwd=directory, capture_output=True, text=True,
                         encoding='utf-8', errors='replace', timeout=30)
    (directory/'run.log').write_text(run.stdout+run.stderr, encoding='utf-8')
    if run.returncode != 0 or not snapshot.exists():
        raise RuntimeError('Emulation failed; inspect '+str(directory/'run.log'))
    state = json.loads(snapshot.read_text(encoding='utf-8'))
    ram = state['bus']['ram']
    if len(ram) != 2048:
        raise RuntimeError('Unexpected CPU RAM snapshot size')
    return ram, dict(name=name, command=command, run_command=run_command,
                     source_sha256=sha256(source), rom_sha256=sha256(rom), snapshot_sha256=sha256(snapshot))


def comparison_program():
    """Generate comparison inputs and independent expected truth values."""
    operations = [('<', lambda a,b:a<b), ('<=', lambda a,b:a<=b),
                  ('>', lambda a,b:a>b), ('>=', lambda a,b:a>=b),
                  ('==', lambda a,b:a==b), ('!=', lambda a,b:a!=b)]
    pairs = [('s8','s8',-128,127), ('s8','s8',-1,-1), ('s8','u8',-1,128),
             ('u8','s8',255,-1), ('s16','s16',-32768,32767), ('s16','s16',-1,0),
             ('u16','u16',65535,32768), ('u8','u16',255,256)]
    lines = ['__location(0x0700) u8 answers[102];', '__location(0x0770) u8 done;',
             '__location(0x0771) u8 calls;',
             's8 negative(void) { calls++; return (s8)255; }',
             'void main(void) { s8 a_s8; s8 b_s8; u8 a_u8; u8 b_u8;',
             's16 a_s16; s16 b_s16; u16 a_u16; u16 b_u16; done=0; calls=0;']
    expected = []
    for left_type, right_type, left, right in pairs:
        left_name, right_name = 'a_'+left_type, 'b_'+right_type
        lines.append(left_name+'='+str(left)+'; '+right_name+'='+str(right)+';')
        for operation, compare in operations:
            expression = left_name+operation+right_name
            # Exercise both conditional control flow and materialized Boolean values.
            for form in ('branch', 'value'):
                index = len(expected)
                if form == 'branch':
                    lines.append('if ('+expression+') answers['+str(index)+']=1; else answers['+str(index)+']=0;')
                else:
                    lines.append('answers['+str(index)+']=(u8)('+expression+');')
                expected.append(dict(index=index, left_type=left_type, right_type=right_type,
                                     left=left, right=right, operation=operation, form=form,
                                     expected=int(compare(left,right))))
    # A side-effecting signed-return call must be evaluated exactly once per comparison.
    for operation, compare in operations:
        index = len(expected)
        lines.append('answers['+str(index)+']=(u8)(negative()'+operation+'0);')
        expected.append(dict(index=index, operation=operation, form='signed-call', expected=int(compare(-1,0))))
    lines += ['done=0xA5; while (1) { } }']
    return '\n'.join(lines)+'\n', expected


def main():
    """Run the loop controls and comparison matrix and preserve a JSON summary."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=Path(__file__).resolve().parent.parent/'kitaqfc.exe')
    parser.add_argument('--emulator', type=Path, required=True)
    parser.add_argument('--output-parent', type=Path)
    args = parser.parse_args()
    compiler, emulator = args.compiler.resolve(strict=True), args.emulator.resolve(strict=True)
    parent = args.output_parent.resolve() if args.output_parent else None
    if parent:
        parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='fc-signed-', dir=parent))
    loops = [('u8-wrap-zero','u8',0,256,300), ('u8-wrap-one','u8',1,256,300),
             ('u8-bounded','u8',0,255,255), ('s8-wrap-128','s8',0,128,300),
             ('s8-bounded','s8',120,127,7), ('s8-wrap-200','s8',120,200,300)]
    results = []
    for name, kind, start, limit, expected in loops:
        for optimized in (False, True):
            text = ('__location(0x0700) u16 result;\n__location(0x0702) u8 done;\n'
                    'void main(void) { u16 visits; '+kind+' i; done=0; visits=0;\n'
                    'for(i='+str(start)+';i<'+str(limit)+';i++){visits++;if(visits==300)break;}\n'
                    'result=visits;done=0xA5;while(1){} }\n')
            ram, record = execute_rom(compiler, emulator, output,
                                      name+('-optimized' if optimized else '-generic'), text, optimized)
            actual = ram[0x700] | (ram[0x701] << 8)
            record.update(expected=expected, actual=actual, done=ram[0x702],
                          passed=actual == expected and ram[0x702] == 0xA5)
            results.append(record)
    text, comparisons = comparison_program()
    ram, record = execute_rom(compiler, emulator, output, 'comparison-matrix', text, True)
    for comparison in comparisons:
        comparison['actual'] = ram[0x700+comparison['index']]
        comparison['passed'] = comparison['actual'] == comparison['expected']
    record.update(comparisons=comparisons, done=ram[0x770], calls=ram[0x771],
                  passed=all(c['passed'] for c in comparisons) and ram[0x770] == 0xA5 and ram[0x771] == 6)
    results.append(record)
    report = dict(compiler=str(compiler), compiler_sha256=sha256(compiler),
                  emulator=str(emulator), emulator_sha256=sha256(emulator), cases=results,
                  scope='Compiled FC ROM execution in KUROSAKI; CPU RAM results, completion markers and call count. No hardware test.')
    (output/'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    failed = [r['name'] for r in results if not r['passed']]
    print(json.dumps(dict(rom_cases=len(results), failed=failed, comparison_checks=len(comparisons),
                         failed_comparisons=[c for c in comparisons if not c['passed']],
                         report=str(output/'report.json'))))
    return 1 if failed else 0


if __name__ == '__main__':
    raise SystemExit(main())
