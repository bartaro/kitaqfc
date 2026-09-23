"""Unit-test compiled input masks using explicit ROM register-read substitution.

Only absolute LDA $4016/$4017 operands in the owned test PRG are redirected to
RAM fixture bytes. This validates arithmetic/branches, not emulated peripherals,
electrical connections, optical sensing or serial timing. Production ROMs are
never modified. Both original and substituted ROM hashes are recorded.
"""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
repo=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
ap.add_argument('--emulator',type=Path,required=True)
ap.add_argument('--report',type=Path)
opt=ap.parse_args();compiler=opt.compiler.resolve(strict=True);emulator=opt.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();records=[]
with tempfile.TemporaryDirectory(prefix='kitaqfc-input-mask-') as temp:
    for state1 in [0,8,16,24]:
        for variant,flags in [('default',[]),('O0',['-O0'])]:
            value1=state1|0xE7;value2=(24^state1)|0xE7
            name=str(state1)+'-'+variant;folder=Path(temp)/name;folder.mkdir()
            src=folder/'case.c';rom=folder/'original.nes';testrom=folder/'substituted.nes';snapshot=folder/'snapshot.json'
            src.write_text('''#include "intrinsics.h"
__location(0x600) u8 result[16];
__location(0x700) u8 port1;
__location(0x701) u8 port2;
void main(void){__irq_disable();__ppu_ctrl_set(0);port1='''+str(value1)+';port2='+str(value2)+''';
result[0]=__zapper_raw1();result[1]=__zapper_trigger1();result[2]=__zapper_light1();
result[3]=__zapper_raw2();result[4]=__zapper_trigger2();result[5]=__zapper_light2();
result[6]=__zapper_trigger();result[7]=__zapper_light();result[8]=__serial_rx_bit();
result[15]=165;while(1){}}
''',encoding='ascii')
            p=subprocess.run([str(compiler),str(src),'-I',str(repo/'lib'),'-o',str(rom),'--no-cache','--no-disasm']+flags,cwd=folder,capture_output=True,timeout=90)
            assert p.returncode==0,(p.stdout+p.stderr).decode(errors='replace')[-2000:]
            data=bytearray(rom.read_bytes());assert data[:4]==b'NES\x1a' and not data[6]&4
            end=16+data[4]*16384;patches=[]
            for address,target in [(0x4016,0x700),(0x4017,0x701)]:
                pattern=bytes([0xAD,address&255,address>>8]);start=16
                while True:
                    at=data.find(pattern,start,end)
                    if at<0:break
                    patches.append(dict(file_offset=at,read_address=address,fixture_address=target))
                    data[at+1:at+3]=target.to_bytes(2,'little');start=at+3
            assert len(patches)>=8
            testrom.write_bytes(data)
            p=subprocess.run([str(emulator),'run',str(testrom),'--frames','6','--snapshot',str(snapshot)],cwd=folder,capture_output=True,timeout=90);assert p.returncode==0
            ram=json.loads(snapshot.read_text())['bus']['ram'];actual=ram[0x600:0x609]+[ram[0x60F]]
            expected=[value1&24,int(bool(value1&16)),int(not value1&8),value2&24,int(bool(value2&16)),int(not value2&8),int(bool(value2&16)),int(not value2&8),int(bool(value2&16)),165]
            record=dict(name=name,inputs=[value1,value2],original_rom_sha256=sha(rom),substituted_rom_sha256=sha(testrom),patches=patches,actual=actual,expected=expected,passed=actual==expected)
            records.append(record);print(name,'PASS' if record['passed'] else 'FAIL',actual,flush=True)
report=dict(compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),script_sha256=sha(Path(__file__)),records=records)
if opt.report:
    opt.report.parent.mkdir(parents=True,exist_ok=True);opt.report.write_text(json.dumps(report,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['passed'] for r in records) else 1)
