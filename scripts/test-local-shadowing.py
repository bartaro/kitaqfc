"""Verify that function storage shadows global RAM and ROM without changing them."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
REPO=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
parser.add_argument('--emulator',type=Path,required=True)
parser.add_argument('--output-parent',type=Path)
args=parser.parse_args();compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-local-shadowing-',dir=args.output_parent)).resolve()
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
fixtures=[
 ('parameter-ram','u8 value; u8 apply(u8 value){value=(u8)(value+2);return value;}',
  'value=0xA7;result[0]=apply(5);result[1]=value;',[7,0xA7]),
 ('parameter-rom','__prg_rom u8 data[2]={0xA7,0xB8}; __prg_rom u8 other[2]={0x39,0x4A}; u8 apply(const u8* data){return data[1];}',
  'result[0]=apply(other);result[1]=data[1];',[0x4A,0xB8]),
 ('local-ram','u8 value; u8 apply(){u8 value;value=0x39;return value;}',
  'value=0xA7;result[0]=apply();result[1]=value;',[0x39,0xA7]),
 ('local-rom','__prg_rom u8 data[2]={0xA7,0xB8}; u8 apply(){u8 data;data=0x39;return data;}',
  'result[0]=apply();result[1]=data[0];',[0x39,0xA7]),
 ('writable-pointer','__prg_rom u8 data[2]={0xA7,0xB8};u8 buffer[2];void apply(u8* data){data[0]=0x61;}',
  'buffer[0]=0;apply(buffer);result[0]=buffer[0];result[1]=data[0];',[0x61,0xA7]),
 ('local-array','u8 values[2];u8 apply(){u8 values[2];values[0]=0x39;values[1]=0x4A;return values[1];}',
  'values[1]=0xA7;result[0]=apply();result[1]=values[1];',[0x4A,0xA7]),
]
records=[]
for name,declarations,body,expected in fixtures:
    folder=out/name;folder.mkdir()
    source=folder/'case.c';source.write_text('__location(0x0700) u8 result[16];\n'+declarations+'\nvoid main(){'+body+'result[15]=0xA5;while(1){}}\n',encoding='ascii')
    rom=folder/'case.nes'
    command=[str(compiler),str(source),'--no-cache','--no-disasm','-o',str(rom)]
    run=subprocess.run(command,cwd=folder,capture_output=True,timeout=60)
    (folder/'build.log').write_bytes(run.stdout+run.stderr)
    row={'name':name,'build_command':command,'build_exit':run.returncode,'source_sha256':sha(source),'expected':expected,'passed':False}
    if run.returncode==0:
        runtime=folder/'runtime.json';snapshot=folder/'snapshot.json'
        command=[str(emulator),'run',str(rom),'--frames','12','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
        run=subprocess.run(command,cwd=folder,capture_output=True,timeout=60)
        (folder/'run.log').write_bytes(run.stdout+run.stderr)
        row.update(run_exit=run.returncode,run_command=command,rom_sha256=sha(rom))
        if run.returncode==0:
            data=json.loads(snapshot.read_text(encoding='utf-8'))['bus']['ram'][0x700:0x710]
            frames=json.loads(runtime.read_text(encoding='utf-8'))['frames']
            row.update(actual=data,frames=frames,passed=data[:2]==expected and data[15]==0xA5 and frames==12)
    records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',row.get('actual'),flush=True)
    (out/'report.json').write_text(json.dumps({'compiler_sha256':sha(compiler),'emulator_sha256':sha(emulator),'cases':records},indent=2),encoding='utf-8')
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
