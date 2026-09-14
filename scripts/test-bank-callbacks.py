"""Measure FC bank-library behavior and reject malformed no-argument far calls."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
repo=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler',type=Path,default=repo/'kitaqfc.exe')
parser.add_argument('--emulator',type=Path,required=True)
parser.add_argument('--output-parent',type=Path)
args=parser.parse_args();compiler=args.compiler.resolve(strict=True)
emulator=args.emulator.resolve(strict=True)
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
output=Path(tempfile.mkdtemp(prefix='fc-bank-callbacks-',dir=args.output_parent)).resolve()
prefix='''#pragma fixed_bank 0
#include "bank.c"
__location(0x0700) u8 result[16];
__prg_rom u8 common[2]={0x49,0xE8};
u8 chosen_bank;
#pragma fixed_bank 1
__prg_rom u8 home1[2]={0xA1,0x11};
#pragma fixed_bank 2
__prg_rom u8 home2[2]={0xB2,0x22};
#pragma fixed_bank 3
__prg_rom u8 payload[4]={0xD3,0x7A,0x5C,0x96};
void callback() { result[0]=result[0]+1; result[1]=payload[0]; }
u16 word_callback() { result[0]=result[0]+1; return 0xB739; }
#pragma fixed_bank 0
'''
def case(name,declaration,body,expected):
    text=prefix+declaration+'\n#pragma fixed_bank 0\nvoid main() {\n'
    text+='bank_switch(2);result[0]=0;result[1]=0;\n'+body
    text+='\nresult[12]=home2[0];result[13]=bank_get_current();result[15]=0xA5;while(1){}\n}\n'
    return name,text,dict(expected)|{12:0xB2,13:2,15:0xA5}
cases=[
    case('data', '', '''u16 word;u8 bytes[4];BankPtr pointer;
result[0]=far_data_read8(3,payload);word=far_data_read16(3,payload+1);
result[1]=(u8)word;result[2]=(u8)(word>>8);
far_data_read(3,payload,bytes,4);result[3]=bytes[3];
farptr_make(&pointer,3,payload+1);farptr_make(0,1,home1);
result[4]=pointer.bank;result[5]=farptr_read8(pointer);word=farptr_read16(pointer);
result[6]=(u8)word;result[7]=(u8)(word>>8);farptr_read(pointer,bytes,3);result[8]=bytes[2];
bytes[0]=0xE7;far_data_read(3,payload,bytes,0);result[9]=bytes[0];
''',{0:0xD3,1:0x7A,2:0x5C,3:0x96,4:3,5:0x7A,6:0x7A,7:0x5C,8:0x96,9:0xE7}),
    case('common-read','','u16 word;result[0]=__farpeek8(0,(u16)common);word=__farpeek16(0,(u16)common);result[1]=(u8)word;result[2]=(u8)(word>>8);',{0:0x49,1:0x49,2:0xE8}),
    case('bankof','','result[0]=__bankof(common);result[1]=__bankof(home1);result[2]=__bankof(home2);result[3]=__bankof(payload);result[4]=__bankof(callback);',{0:0,1:1,2:2,3:3,4:3}),
    case('switch-aliases','','__bankswitch(1);result[0]=home1[0];result[1]=bank_get_current();__prg_bank_set(2);result[2]=home2[0];',{0:0xA1,1:2,2:0xB2}),
    case('farcall','','__farcall(3,callback);',{0:1,1:0xD3}),
    case('farcall-macro','','far_call(3,callback);',{0:1,1:0xD3}),
    case('runtime-bank','','chosen_bank=3;__farcall(chosen_bank,callback);',{0:1,1:0xD3}),
    case('word-return','','u16 value;value=__farcall(3,word_callback);result[1]=(u8)value;result[2]=(u8)(value>>8);',{0:1,1:0x39,2:0xB7}),
    case('side-effect','u8 get_bank() { result[2]=result[2]+1; return 3; }','result[2]=0;__farcall(get_bank(),callback);',{0:1,1:0xD3,2:1}),
    case('nested','''#pragma fixed_bank 1
void inner() { result[0]=result[0]+1;result[1]=home1[0]; }
#pragma fixed_bank 3
void outer() { far_call(1,inner);result[2]=payload[0]; }
''','far_call(3,outer);',{0:1,1:0xA1,2:0xD3}),
    case('local-guard','void invoke() { u8 guard;guard=0x6D;far_call(3,callback);result[2]=guard; }','invoke();',{0:1,1:0xD3,2:0x6D}),
    case('banked-caller','#pragma fixed_bank 2\nvoid invoke() { far_call(3,callback);result[2]=home2[0]; }','invoke();',{0:1,1:0xD3,2:0xB2}),
    case('repeat','','u8 i;for(i=0;i<100;i++) { far_call(3,callback); }',{0:100,1:0xD3}),
]
negative=[
    ('switch-zero','void main(){__bankswitch(0);while(1){}}','KQFC2603'),
    ('switch-upper','void main(){__bankswitch(31);while(1){}}','KQFC2603'),
    ('alias-zero','void main(){__prg_bank_set(0);while(1){}}','KQFC2603'),
    ('callback-parameter','void target(u8 value){result[0]=value;}\nvoid main(){__farcall(1,target);while(1){}}','no parameters'),
    ('callback-variable','u16 pointer;\nvoid main(){__farcall(1,pointer);while(1){}}','function name'),
]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
records=[]
for name,source,expected in cases+[(n,prefix+s,d) for n,s,d in negative]:
    folder=output/name;folder.mkdir(exist_ok=True)
    src=folder/'case.c';src.write_text(source,encoding='ascii');rom=folder/'case.nes';rom.unlink(missing_ok=True)
    command=[str(compiler),str(src),'-I',str(repo/'lib'),'--mapper=mmc1','--board=surom512','--no-cache','--no-disasm','-o',str(rom)]
    process=subprocess.run(command,cwd=folder,capture_output=True,timeout=60)
    diagnostics=process.stdout+process.stderr;(folder/'build.log').write_bytes(diagnostics)
    row=dict(name=name,build_exit=process.returncode,expected=expected,command=command,source_sha256=sha(src),passed=False)
    if isinstance(expected,str):
        row['passed']=process.returncode!=0 and expected.encode() in diagnostics and not rom.exists()
    elif process.returncode==0:
        snapshot=folder/'snapshot.json';snapshot.unlink(missing_ok=True)
        runtime=folder/'runtime.json';runtime.unlink(missing_ok=True)
        command=[str(emulator),'run',str(rom),'--frames','12','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
        process=subprocess.run(command,cwd=folder,capture_output=True,timeout=60)
        (folder/'run.log').write_bytes(process.stdout+process.stderr)
        row.update(run_exit=process.returncode,run_command=command,rom_sha256=sha(rom))
        if process.returncode==0:
            state=json.loads(snapshot.read_text(encoding='utf-8'));ram=state['bus']['ram'];assert len(ram)==2048
            actual=ram[0x700:0x710];frames=json.loads(runtime.read_text(encoding='utf-8'))['frames']
            row.update(actual=actual,frames=frames,passed=frames==12 and all(actual[k]==v for k,v in expected.items()))
    records.append(row);print(name,'PASS' if row['passed'] else 'FAIL',row.get('actual'),flush=True)
    (output/'report.json').write_text(json.dumps({'compiler_sha256':sha(compiler),'emulator_sha256':sha(emulator),
        'library_sha256':{name:sha(repo/'lib'/name) for name in ['bank.c','bank.h','intrinsics.h','core.h']},
        'mapper':'mmc1','board':'surom512','cases':records},indent=2),encoding='utf-8')
print('Report:',output/'report.json')
if not all(r['passed'] for r in records):raise SystemExit(1)
if any(not r['passed'] for r in records[:len(cases)]):raise SystemExit(2)
