"""Run original FDS file-I/O ABI regressions in KUROSAKI. No external BIOS used."""
# Copyright (c) 2026 DAISUKE OBA
from pathlib import Path
import argparse,hashlib,json,os,subprocess,tempfile
from fds_abi_fixture import disk_files
from fds_fileio_fixture import fileio_firmware
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe');ap.add_argument('--emulator',type=Path,required=True);ap.add_argument('--report',type=Path);ap.add_argument('--smoke',action='store_true');args=ap.parse_args()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base=[dict(id=0,load='0x6000',size=16384,type=0,boot=True,number=0),dict(id=1,load='0xA000',size=16384,type=0,boot=True,number=1),dict(id=2,load=0,size=8192,type=1,boot=True,number=2)]
specs=[('relocate',0x8500,513,0,0,1),('forward-overlap',0x7ff1,513,0,0,1),('backward-overlap',0x8011,513,0,0,1),('native-null',0,513,0,0,1),('same-address',0x8000,257,0,0,1),('zero-size',0x8500,0,0,0,1),('load-error',0x8500,17,0x27,0,0),('missing-count',0x8500,17,0,0,0),('save-error',0x8500,17,0,3,1),('unknown-id',0x8500,17,0,0,1),('boot-id',0x8500,17,0,0,1),('id-255',0x8500,17,0,0,1),('not-last',0x8500,17,0,0,1),('bad-size',0x8500,17,0,0,1),('bad-destination',0x2000,17,0,0,1),('wrap-destination',0xfff0,17,0,0,1),('bad-source',0x8500,17,0,0,1),('table-page',0x8500,17,0,0,1),('side-1',0x8500,17,0,0,1),('side-2',0x8500,17,0,0,1)]
if args.smoke:specs=specs[:1]
rows=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-fileio-') as temp:
 for name,dst,size,load_error,save_error,count in specs:
  for opt in [True,False]:
   case=name+('-default' if opt else '-O0');folder=Path(temp)/case;folder.mkdir();rom=folder/'case.fds';snapshot=folder/'state.json';bios=folder/'fixture.bin'
   payload=bytes((i*37+11)&255 for i in range(size));(folder/'payload.bin').write_bytes(payload)
   files=[dict(f) for f in base];slot=dict(id=73,name='SAVEDEMO',load='0x8000',source='payload.bin',type=0,boot=False,number=200,side=int(name[-1]) if name.startswith('side-') else 0)
   if name=='table-page':files += [dict(id=i,load='0x8200',source='payload.bin',type=0,boot=False,number=i) for i in range(10,19)]
   files.append(slot)
   if name=='not-last':files.append(dict(id=74,load='0x8300',source='payload.bin',type=0,boot=False,number=201))
   (folder/'manifest.json').write_text(json.dumps(dict(files=files)))
   id=250 if name=='unknown-id' else 255 if name=='id-255' else 0 if name=='boot-id' else 73
   length=size+1 if name=='bad-size' else size;srcaddr=0x4010 if name=='bad-source' else 0x8500
   # Copy loaded bytes into a reserved CPU buffer and exercise save followed by reload.
   source=f'''#include "nes_game.h"
__location(0x600) u8 result[8];
__location(0x700) u8 fixture_log[256];
#pragma fixed_bank 0
void main(void){{u16 i;u8* buf=(u8*)0x8500;u8* loaded=(u8*){dst or 0x8000};
__irq_disable();for(i=0;i<{max(size,1)};i=i+1){{buf[i]=204;}}
result[0]=__fds_load_file({id},(u8*){dst});
for(i=0;i<{size};i=i+1){{if(loaded[i]!=(u8)(i*37+11)){{result[6]=1;}}}}
result[2]=loaded[0];result[3]=loaded[{max(size-1,0)}];
for(i=0;i<{size};i=i+1){{buf[i]=(u8)(i*13+7);}}
result[1]=__fds_save_file({id},(u8*){srcaddr},{length});
result[4]=__fds_load_file({id},(u8*)0x8600);loaded=(u8*)0x8600;
for(i=0;i<{size};i=i+1){{if(loaded[i]!=(u8)(i*13+7)){{result[7]=1;}}}}
result[5]=165;while(1){{}}}}
'''
   (folder/'case.c').write_text(source)
   cmd=[str(compiler),str(folder/'case.c'),'-I',str(repo/'lib'),'-o',str(rom),'--mapper=fds','--fds-no-license-bypass','--fds-meta='+str(folder/'manifest.json'),'--no-cache','--no-disasm']+([] if opt else ['-O0'])
   p=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90);row=dict(name=case,passed=False,build_exit=p.returncode)
   if p.returncode==0:
    resolved=next(f for f in disk_files(rom) if f['id']==73);bios.write_bytes(fileio_firmware(resolved,load_error,save_error,count))
    p=subprocess.run([str(emulator),'run',str(rom),'--frames','90','--snapshot',str(snapshot)],cwd=folder,capture_output=True,env={**os.environ,'KUROSAKI_DISKSYS_ROM':str(bios)},timeout=90)
    if p.returncode==0:
     state=json.loads(snapshot.read_text());ram=state['bus']['ram'];actual=ram[0x600:0x608]
     invalid_id=name in ('unknown-id','boot-id','id-255');invalid_dst=name in ('bad-destination','wrap-destination');invalid_save=invalid_id or name in ('not-last','bad-size','bad-source')
     load_status=255 if invalid_id or invalid_dst else load_error or (0 if count==1 else 0x40)
     save_status=255 if invalid_save else save_error
     expected_disk=[255]*10;expected_disk[6]=resolved['side']&1;expected_disk[7]=resolved['side']//2
     descriptor=bytes([73])+b'SAVEDEMO'+bytes([0,0x80,size&255,size>>8,0,0,0x85,0])
     passed=actual[0]==load_status and actual[1]==save_status and actual[4]==(255 if invalid_id else load_error or (0 if count==1 else 0x40)) and actual[5]==165
     if not load_status and size:passed &= actual[2]==payload[0] and actual[3]==payload[-1] and actual[6]==0
     if not load_error and count==1 and not invalid_save and not save_error:passed &= actual[7]==0
     passed &= ram[0x700]==(0 if invalid_id else 1 if invalid_dst else 2) and ram[0x704]==(0 if invalid_save else 1)
     if not invalid_save:passed &= ram[0x701]==resolved['ordinal'] and bytes(ram[0x720:0x731])==descriptor
     if not invalid_id:passed &= ram[0x710:0x71a]==expected_disk
     row.update(actual=actual,load_calls=ram[0x700],save_calls=ram[0x704],ordinal=ram[0x701],expected_ordinal=resolved['ordinal'],descriptor=ram[0x720:0x731],passed=bool(passed))
   rows.append(row);print(case,'PASS' if row['passed'] else 'FAIL',json.dumps(row),flush=True)
   if not row['passed']:print((p.stdout+p.stderr).decode(errors='replace')[-2000:])
report=dict(scope='Original CPU ABI fixture; physical disk and real BIOS unverified',compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),test_sha256=sha(Path(__file__)),fixture_sha256=sha(Path(__file__).with_name('fds_fileio_fixture.py')),records=rows)
if args.report:args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2))
raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
