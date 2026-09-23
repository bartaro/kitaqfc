"""Execute original overlay code through an independent LoadFiles ABI fixture.

Verify return to boot bank 1, nested overlays, arguments, failed transfers and
failure to restore. These tests do not establish BIOS or physical-disk behavior.
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
prefix='''#include "nes_game.h"
__location(0x0600) u8 result[16];
__location(0x0700) u8 fixture_log[256];
'''
target='''#pragma fixed_bank 2
u8 target(void){result[2]++;return 183;}
'''
specs=[]
for caller in [0,1]:
    for api in ['__fds_overlay_farcall','__fds_farcall','nes_fds_farcall','__farcall','ordinary']:
        call='target()' if api=='ordinary' else api+'(2,target)'
        specs.append(dict(name=f'caller{caller}-{api}',caller=caller,target=target,call=call,expected=[183,1,1],ids=[32,0]))
    specs+= [
        dict(name=f'caller{caller}-fastcall-bytes',caller=caller,target='#pragma fixed_bank 2\nu8 target(u8 a,u8 b){result[2]++;return a+b;}\n',call='target(7,11)',expected=[18,1,1],ids=[32,0]),
        dict(name=f'caller{caller}-word-arg',caller=caller,target='#pragma fixed_bank 2\nu16 target(u16 a){result[2]++;return a+1;}\n',call='target(0x12FF)',expected=[0,1,1],high=0x13,ids=[32,0]),
        dict(name=f'caller{caller}-stacked-args',caller=caller,target='#pragma fixed_bank 2\nu16 target(u8 a,u16 b){result[2]++;return a+b;}\n',call='target(7,0x12FF)',expected=[6,1,1],high=0x13,ids=[32,0]),
        dict(name=f'caller{caller}-nested',caller=caller,target='''#pragma fixed_bank 3
u8 inner(void){result[3]++;return 21;}
#pragma fixed_bank 2
u8 target(void){u8 value;result[2]++;value=inner();return value+34;}
''',call='target()',expected=[55,1,1],inner=1,ids=[32,33,32,0]),
        dict(name=f'caller{caller}-boot-bank-callee',caller=caller,target='''#pragma fixed_bank 1
u8 inner(void){result[3]++;return 21;}
#pragma fixed_bank 2
u8 target(void){u8 value;result[2]++;value=inner();return value+34;}
''',call='target()',expected=[55,1,1],inner=1,ids=[32,0,32,0]),
        dict(name=f'caller{caller}-load-error',caller=caller,target=target,call='__fds_farcall(2,target)',expected=[39,1,0],failures={1:39},ids=[32,0]),
        dict(name=f'caller{caller}-partial-error',caller=caller,target=target,call='__fds_farcall(2,target)',expected=[39,1,0],failures={1:39},partial=True,ids=[32,0]),
        dict(name=f'caller{caller}-restore-error',caller=caller,target=target,call='__fds_farcall(2,target)',trap=True,failures={2:39},ids=[32,0]),
        dict(name=f'caller{caller}-recovery-error',caller=caller,target=target,call='__fds_farcall(2,target)',trap=True,failures={1:39,2:39},ids=[32,0])
    ]
records=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-fds-overlay-') as temp:
    root=Path(temp)
    for spec in specs:
        for optimize in [True,False]:
            name=spec['name']+('-default' if optimize else '-O0');folder=root/name;folder.mkdir()
            src=folder/'case.c';rom=folder/'case.fds';bios=folder/'fixture.bin';snapshot=folder/'state.json'
            body=prefix+spec['target']+'#pragma fixed_bank '+str(spec['caller'])+'\nvoid main(void){u16 value;__irq_disable();result[2]=0;result[3]=0;\n'
            body+='value='+spec['call']+';result[0]=(u8)value;result[4]=(u8)(value>>8);result[1]=__fds_current_bank();result[15]=165;while(1){}}\n'
            src.write_text(body,encoding='ascii')
            cmd=[str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--mapper=fds','--fds-no-license-bypass','--fds-overlay-trim','--no-cache','--no-disasm']+([] if optimize else ['-O0'])
            p=subprocess.run(cmd,cwd=folder,capture_output=True,timeout=90)
            row=dict(name=name,source_sha256=sha(src),build_exit=p.returncode,passed=False)
            if p.returncode==0:
                files=disk_files(rom);boot=[f for f in files if f['id']==0];assert len(boot)==1 and boot[0]['address']==0x6000 and len(boot[0]['data'])==16384
                assert not next(f for f in files if f['id']==32)['boot']
                bios.write_bytes(loadfiles_firmware(files,spec.get('failures'),failure_after_copy=spec.get('partial',False)))
                p=subprocess.run([str(emulator),'run',str(rom),'--frames','60','--snapshot',str(snapshot)],cwd=folder,capture_output=True,env={**os.environ,'KUROSAKI_DISKSYS_ROM':str(bios)},timeout=90)
                row.update(run_exit=p.returncode,rom_sha256=sha(rom),fixture_sha256=sha(bios))
                if p.returncode==0:
                    state=json.loads(snapshot.read_text());ram=state['bus']['ram'];actual=ram[0x600:0x610];ids=ram[0x780:0x780+ram[0x700]]
                    good=ids==spec['ids'] and ram[0x701]==0 and not state['cpu']['stopped']
                    if spec.get('trap'):
                        pc=state['cpu']['pc'];common=next(f for f in files if f['address']==0xA000)['data'];offset=pc-0xA000
                        good=good and actual[15]==0 and state['cpu']['a']==39 and 0<=offset<len(common)-2 and common[offset:offset+3]==bytes([0x4C,pc&255,pc>>8])
                    else:good=good and actual[:3]==spec['expected'] and actual[3]==spec.get('inner',0) and actual[4]==spec.get('high',0) and actual[15]==165
                    row.update(actual=actual,ids=ids,expected_ids=spec['ids'],cpu=state['cpu'],passed=good)
            records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',row.get('actual'),row.get('ids'),flush=True)
            if p.returncode:print((p.stdout+p.stderr).decode(errors='replace')[-2000:],flush=True)
report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),fixture_generator_sha256=sha(Path(__file__).with_name('fds_abi_fixture.py')),records=records)
if args.report:
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
