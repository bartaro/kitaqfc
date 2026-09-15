"""Verify fixed RAM survives automatic runtime/global/local allocation."""
from pathlib import Path
import argparse, hashlib, json, subprocess, tempfile

REPO = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler', type=Path, default=REPO/'kitaqfc.exe')
p.add_argument('--emulator', type=Path, required=True)
p.add_argument('--output-parent', type=Path)
args = p.parse_args()
compiler = args.compiler.resolve(strict=True)
emulator = args.emulator.resolve(strict=True)
if args.output_parent: args.output_parent.mkdir(parents=True, exist_ok=True)
out = Path(tempfile.mkdtemp(prefix='fc-fixed-ram-', dir=args.output_parent)).resolve()
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

# Keep result storage outside the automatic global pool. Fixed-to-fixed aliases
# remain intentional; the metadata test only rejects automatic storage overlap.
prefix = '#include "intrinsics.h"\n__location(0x0200) u8 result[16];\n'
worker = 'u8 work[12];u8 transform(u8 x){u8 tmp[5];tmp[4]=(u8)(x+7);work[11]=tmp[4];return work[11];}\n'
body = 'void main(){u16 i;__ppu_off();__ppu_ctrl_set(0);for(i=0;i<256;i++)fixed_data[i]=(u8)(i^0xA5);result[0]=transform(12);result[1]=fixed_data[0];result[2]=fixed_data[255];result[15]=0x5A;while(1){}}\n'
fixed = '__location(0x0300) u8 fixed_data[256];\n'
cases = [
    ('fixed-first', prefix+fixed+worker+body, [], True, [19,165,90]),
    ('fixed-last', prefix+worker+fixed+body, [], True, [19,165,90]),
    ('mirror', prefix+fixed.replace('0x0300','0x0B00')+worker+body, [], True, [19,165,90]),
    ('two-pages', prefix+fixed+'__location(0x0400) u8 second[256];\n'+worker+body, [], True, [19,165,90]),
    ('fragmented-reverse', prefix+fixed+'__location(0x0560) u8 gap_b[32];\n__location(0x0440) u8 gap_a[16];\n'+worker+body, [], True, [19,165,90]),
    ('zp-reserved', prefix+fixed+'__location(0x0010) u8 zp_fixed[80];\n'+worker+body, ['--whole-program-zp'], True, [19,165,90]),
    ('aggregate-size', prefix+'typedef struct {u16 words[128];} Page;\n__location(0x0400) Page other;\n'+fixed+worker+body, [], True, [19,165,90]),
    ('out-of-ram', prefix+'__location(0x0300) u8 fixed_data[1280];\n'+worker+body, [], False, None),
]

def physical_ranges(address, size):
    """Map CPU RAM mirrors to physical spans without folding PPU/I/O registers."""
    end = min(address+size, 0x2000)
    while address < end:
        start = address & 0x7FF
        length = min(end-address, 0x800-start)
        yield start, start+length
        address += length

records = []
for name, text, options, valid, expected in cases:
    folder = out/name; folder.mkdir()
    source = folder/'case.c'; source.write_text(text, encoding='ascii')
    rom = folder/'case.nes'; metadata = folder/'metadata.json'
    command = [str(compiler),str(source),'-I',str(REPO/'lib'),'--no-cache','--no-disasm',
               '--kurosaki-metadata='+str(metadata),'-o',str(rom)]+options
    run = subprocess.run(command, cwd=folder, capture_output=True, timeout=60)
    log = run.stdout+run.stderr; (folder/'build.log').write_bytes(log)
    row = dict(name=name, build_command=command, build_exit=run.returncode,
               source_sha256=sha(source), passed=False)
    if not valid:
        row['passed'] = run.returncode != 0 and b'ran out of internal RAM' in log
    elif run.returncode == 0:
        allocations = json.loads(metadata.read_text(encoding='utf-8'))['ram_access_contract']['allocations']
        fixed_ranges = [span for a in allocations if a['kind']=='global_fixed'
                        for span in physical_ranges(a['address'],a['size'])]
        overlaps = [a for a in allocations if a['kind']!='global_fixed'
                    and any(a['address']<end and start<a['address']+a['size'] for start,end in fixed_ranges)]
        snapshot=folder/'snapshot.json'; runtime=folder/'runtime.json'
        command=[str(emulator),'run',str(rom),'--frames','30','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
        run=subprocess.run(command,cwd=folder,capture_output=True,timeout=60)
        (folder/'run.log').write_bytes(run.stdout+run.stderr)
        row.update(overlaps=overlaps,run_command=command,run_exit=run.returncode,rom_sha256=sha(rom))
        if run.returncode==0:
            ram=json.loads(snapshot.read_text(encoding='utf-8'))['bus']['ram']
            data=ram[0x200:0x210];frames=json.loads(runtime.read_text(encoding='utf-8'))['frames']
            intact=ram[0x300:0x400]==[i^0xA5 for i in range(256)]
            row.update(actual=data,expected=expected,fixed_page_intact=intact,frames=frames,
                       passed=not overlaps and data[:3]==expected and data[15]==0x5A and intact and frames==30)
    records.append(row)
    print(name,'PASS' if row['passed'] else 'FAIL',flush=True)
    (out/'report.json').write_text(json.dumps(dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),cases=records),indent=2),encoding='utf-8')
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
