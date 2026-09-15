"""Inspect IRQ status, PPU NMI control and the default handler's wrapping counter."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile

REPO=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
parser.add_argument('--emulator',type=Path,required=True)
parser.add_argument('--output-parent',type=Path)
args=parser.parse_args()
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-interrupt-intrinsics-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
prefix='#include "intrinsics.h"\n__location(0x0700) u8 saved;\n__location(0x0701) u8 saved2;\n__location(0x0702) u8 after;\n__location(0x0703) u8 before;\n__location(0x0704) u8 done;\n__location(0x2000) u8 raw_ctrl;\n'
init='u16 i;__ppu_off();__ppu_ctrl_set(0);__irq_disable();saved=0;saved2=0;after=0;before=0;done=17;'
cases=[]

def add(name,body,expected=None,ctrl=0,custom=False,blocked=False,frames=60,vram=None):
    want={'saved':0,'saved2':0,'after':0,'before':0,'done':17 if blocked else 165}
    want.update(expected or {})
    cases.append(dict(name=name,body=body,expected=want,ctrl=ctrl,custom=custom,frames=frames,vram=vram))

add('nmi-enable-preserves-shadow','__ppu_ctrl_set(0x2B);__nmi_enable();',ctrl=0xAB)
add('nmi-disable-preserves-shadow','__ppu_ctrl_set(0xAB);__nmi_disable();',ctrl=0x2B)
add('nmi-enable-uses-shadow','__ppu_ctrl_set(0x2B);raw_ctrl=4;__nmi_enable();',ctrl=0xAB)
add('nmi-disable-uses-shadow','__ppu_ctrl_set(0xAB);raw_ctrl=0x84;__nmi_disable();',ctrl=0x2B)
add('irq-disable-masks','__irq_enable();__irq_disable();saved=__irq_save();saved=saved&4;',{'saved':4})
add('irq-enable-unmasks','__irq_disable();__irq_enable();saved=__irq_save();saved=saved&4;')
add('irq-save-captures-enabled-and-masks','__irq_enable();saved=__irq_save();saved2=__irq_save();saved=saved&4;saved2=saved2&4;',{'saved2':4})
add('irq-save-captures-disabled','__irq_disable();saved=__irq_save();saved2=__irq_save();saved=saved&4;saved2=saved2&4;',{'saved':4,'saved2':4})
add('irq-restore-enabled','__irq_enable();saved=__irq_save();__irq_restore(saved);saved2=__irq_save();saved=saved&4;saved2=saved2&4;')
add('irq-restore-disabled','__irq_disable();saved=__irq_save();__irq_enable();__irq_restore(saved);saved2=__irq_save();saved=saved&4;saved2=saved2&4;',{'saved':4,'saved2':4})
add('irq-restore-status-bits','__irq_restore(0xC9);saved=__irq_save();',{'saved':0xF9})
add('irq-restore-cleared-status','__irq_restore(0);saved=__irq_save();',{'saved':0x30})
add('nmi-ready-no-boolean','saved=__nmi_ready();saved2=__nmi_ready();')
add('nmi-wait-with-irq-masked','__nmi_enable();__nmi_wait();before=__nmi_ready();__nmi_wait();__nmi_wait();__nmi_wait();__nmi_disable();after=__nmi_ready();after=after-before;before=0;saved=__irq_save();saved=saved&4;',{'saved':4,'after':3})
add('nmi-ready-wrap-256','__nmi_enable();__nmi_wait();before=__nmi_ready();for(i=0;i<256;i++)__nmi_wait();__nmi_disable();after=__nmi_ready();after=after-before;before=0;',frames=300)
add('nmi-disabled-stays-still','__nmi_enable();__nmi_wait();__nmi_disable();before=__nmi_ready();for(i=0;i<60000;i++){saved=__nmi_ready();}after=saved-before;before=0;saved=0;',frames=300)
add('nmi-wait-blocked-disabled','__nmi_wait();',blocked=True)
add('nmi-counter-with-empty-handler','__nmi_enable();for(i=0;i<60000;i++){saved=__nmi_ready();}__nmi_disable();',custom=True,frames=300)
add('nmi-wait-blocked-empty-handler','__nmi_enable();__nmi_wait();',custom=True,blocked=True,ctrl=128)
add('nmi-wait-retains-uncommitted','__vramq_put(0x2000,7);__nmi_enable();__nmi_wait();__nmi_disable();saved=__vramq_len();',{'saved':4},vram=(0x2000,0))
add('nmi-wait-flushes-before-return','__vramq_put(0x2000,7);__vramq_commit();__nmi_enable();__nmi_wait();__nmi_disable();saved=__vramq_len();',vram=(0x2000,7))

rows=[]
for case in cases:
    folder=out/case['name'];folder.mkdir();source=folder/'case.c'
    text=prefix+('void __nes_nmi(void){}\n' if case['custom'] else '')+'void main(){'+init+case['body']+'done=165;while(1){}}'
    source.write_text(text,encoding='ascii');rom=folder/'case.nes'
    command=[str(compiler),str(source),'-I',str(REPO/'lib'),'--no-cache','--no-disasm','-o',str(rom)]
    process=subprocess.run(command,capture_output=True,timeout=60,cwd=folder)
    (folder/'build.txt').write_bytes(process.stdout+process.stderr)
    row={'name':case['name'],'source_sha256':sha(source),'build_exit':process.returncode,'passed':False}
    if process.returncode==0:
        snapshot=folder/'snapshot.json'
        process=subprocess.run([str(emulator),'run',str(rom),'--frames',str(case['frames']),'--headless','--snapshot',str(snapshot)],capture_output=True,timeout=60,cwd=folder)
        (folder/'runtime.txt').write_bytes(process.stdout+process.stderr);row['runtime_exit']=process.returncode
        if process.returncode==0:
            state=json.loads(snapshot.read_text(encoding='utf-8'));ppu=state['bus']['ppu']
            actual=dict(zip(['saved','saved2','after','before','done'],state['bus']['ram'][0x700:0x705]))
            actual['ctrl']=ppu['ctrl'];expected={**case['expected'],'ctrl':case['ctrl']}
            if case['vram']:
                addr,value=case['vram'];actual['ppu_byte']=ppu['vram'][addr];expected['ppu_byte']=value
            row.update(actual=actual,expected=expected,passed=actual==expected)
    rows.append(row);print(case['name'],'PASS' if row['passed'] else 'FAIL',flush=True)
    if not row['passed']:print(json.dumps(row),flush=True)
    (out/'report.json').write_text(json.dumps({'script_sha256':sha(Path(__file__)),'compiler_sha256':sha(compiler),'emulator_sha256':sha(emulator),'header_sha256':sha(REPO/'lib/intrinsics.h'),'cases':rows},indent=2),encoding='utf-8')
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(row['passed'] for row in rows) else 1)
