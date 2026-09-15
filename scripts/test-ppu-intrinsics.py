"""Inspect control, latch, scrolling, direct transfer, nametable and palette intrinsics."""
from pathlib import Path
import argparse,hashlib,json,subprocess,tempfile
REPO=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler',type=Path,default=REPO/'kitaqfc.exe')
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--output-parent',type=Path)
args=p.parse_args()
if args.output_parent:args.output_parent.mkdir(parents=True,exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='fc-ppu-intrinsics-',dir=args.output_parent)).resolve()
compiler=args.compiler.resolve(strict=True);emulator=args.emulator.resolve(strict=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
prefix='#include "intrinsics.h"\n__location(0x2006) u8 address_port;\n__location(0x2007) u8 data_port;\n__location(0x2005) u8 scroll_port;\n__location(0x0700) u8 result[16];\n__location(0x0500) u8 source[255];\n'
init='u16 i;__ppu_off();__ppu_ctrl_set(0);for(i=0;i<16;i++)result[i]=0;for(i=0;i<255;i++)source[i]=(u8)(i+1);'
cases=[]
def add(name,body,ppu=None,vram=None,writes=0,result=None):
    cases.append((name,body,{'data_writes':writes,**(ppu or {})},vram or {},result or {}))
add('render-on','__ppu_ctrl_set(4);__ppu_mask_set(0xE1);__ppu_on();',{'ctrl':4,'mask':30})
add('render-off','__ppu_ctrl_set(4);__ppu_mask_set(0x1E);__ppu_off();',{'ctrl':4,'mask':0})
add('mask-exact','__ppu_ctrl_set(4);__ppu_mask_set(0xE1);',{'ctrl':4,'mask':225})
add('ctrl-exact','__ppu_ctrl_set(0x3F);',{'ctrl':63,'mask':0})
add('address-reset','address_port=0x21;__ppu_addr(0x2380);__ppu_data(7);',vram={0x2380:7},writes=1)
add('address-mask','__ppu_addr(0xA040);__ppu_data(5);',vram={0x2040:5},writes=1)
add('data-increment-1','__ppu_addr(0x2040);__ppu_data(7);__ppu_data(8);__ppu_data(9);',vram={0x2040:7,0x2041:8,0x2042:9},writes=3)
add('data-increment-32','__ppu_ctrl_set(4);__ppu_addr(0x2040);__ppu_data(7);__ppu_data(8);__ppu_data(9);',vram={0x2040:7,0x2060:8,0x2080:9},writes=3)
add('status-flag-and-latch','address_port=0x21;result[0]=__ppu_read_status();while((result[0]&128)==0)result[0]=__ppu_read_status();result[1]=__ppu_read_status();address_port=0x23;address_port=0x80;data_port=7;',vram={0x2380:7},writes=1,result={0:128})
add('latch-reset','address_port=0x21;__scroll_latch_reset();address_port=0x23;address_port=0x80;data_port=7;',vram={0x2380:7},writes=1)
add('scroll-pair','address_port=0x21;__scroll_set(19,27);',{'scroll_x':19,'scroll_y':27,'fine_x':3,'scroll_writes':2})
add('scroll-x-preserves-y','__scroll_set(19,27);scroll_port=100;scroll_port=101;__scroll_x_set(255);',{'scroll_x':255,'scroll_y':27,'fine_x':7,'scroll_writes':6})
add('scroll-y-preserves-x','__scroll_set(19,27);scroll_port=100;scroll_port=101;__scroll_y_set(255);',{'scroll_x':19,'scroll_y':255,'fine_x':3,'scroll_writes':6})
for name in ['write','fill']:
    arg='source' if name=='write' else '7'
    add(name+'-zero-seeks','address_port=0x21;__vram_'+name+'(0x2340,'+arg+',0);__ppu_data(9);',vram={0x2340:9},writes=1)
    add(name+'-255','__vram_'+name+'(0x2040,'+arg+',255);',vram={0x2040+i:i+1 if name=='write' else 7 for i in range(255)},writes=255)
    add(name+'-increment-32','__ppu_ctrl_set(4);__vram_'+name+'(0x2040,'+arg+',3);',vram={0x2040+i*32:i+1 if name=='write' else 7 for i in range(3)},writes=3)
add('put-corners','__nametable_put(0,0,7);__nametable_put(31,29,9);',vram={0x2000:7,0x23BF:9},writes=2)
add('put-nt-selector','__nametable_put_nt(7,31,29,9);',vram={0x2FBF:9},writes=1)
add('rect-bottom-right','__nametable_rect(30,28,2,2,7);',vram={0x2000+y*32+x:7 for y in [28,29] for x in [30,31]},writes=4)
add('rect-nt-selector','__nametable_rect_nt(7,1,1,2,2,9);',vram={0x2C00+y*32+x:9 for y in [1,2] for x in [1,2]},writes=4)
for name in ['__nametable_rect','__nametable_rect_nt']:
    head='3,' if name.endswith('_nt') else ''
    add(name+'-zero-width',name+'('+head+'2,3,0,3,7);',{'addr_writes':6})
    add(name+'-zero-height',name+'('+head+'2,3,3,0,7);',{'addr_writes':0})
add('attr-whole-byte','__attr_set(4,4,0x55);__attr_set(6,7,0xAA);',vram={0x23C9:170},writes=2)
add('attr-nt-whole-byte','__attr_set_nt(7,4,4,0x55);__attr_set_nt(7,6,7,0xAA);',vram={0x2FC9:170},writes=2)
add('palette-background','__palette_bg_load(source);',{'palette':list(range(1,17))+[0]*16},writes=16)
pal=[0]*32
for i in range(16):pal[i if i%4==0 else 16+i]=i+1
add('palette-sprites','__palette_sp_load(source);',{'palette':pal},writes=16)

def physical(addr,rom):
    addr&=0x3FFF
    if 0x3000<=addr<0x3F00:addr-=0x1000
    offset=addr-0x2000;table=offset//1024
    table=table&1 if rom[6]&1 else table>>1
    return 0x2000+table*1024+(offset%1024)

rows=[]
for name,body,expected_ppu,writes,expected_result in cases:
    folder=out/name;folder.mkdir();source=folder/'case.c';source.write_text(prefix+'void main(){'+init+body+'result[15]=165;while(1){}}',encoding='ascii')
    rom=folder/'case.nes';command=[str(compiler),str(source),'-I',str(REPO/'lib'),'--no-cache','--no-disasm','-o',str(rom)]
    run=subprocess.run(command,capture_output=True,timeout=60,cwd=folder);(folder/'build.txt').write_bytes(run.stdout+run.stderr)
    row=dict(name=name,source_sha256=sha(source),build_command=command,build_exit=run.returncode,passed=False)
    if run.returncode==0:
        snapshot=folder/'snapshot.json';runtime=folder/'runtime.json';command=[str(emulator),'run',str(rom),'--frames','60','--headless','--snapshot',str(snapshot),'--json',str(runtime)]
        run=subprocess.run(command,capture_output=True,timeout=60,cwd=folder);(folder/'runtime.txt').write_bytes(run.stdout+run.stderr);row['runtime_exit']=run.returncode
        if run.returncode==0:
            state=json.loads(snapshot.read_text(encoding='utf-8'));ppu=state['bus']['ppu'];expected=[0]*2048
            for addr,value in writes.items():expected[physical(addr,rom.read_bytes())-0x2000]=value
            want=[0]*16;want[15]=165
            for index,value in expected_result.items():want[index]=value
            actual={k:ppu[k] for k in expected_ppu};result=state['bus']['ram'][0x700:0x710];vram=ppu['vram'][0x2000:0x2800]
            row.update(ppu=actual,expected_ppu=expected_ppu,result=result,expected_result=want,vram=vram,expected_vram=expected,passed=actual==expected_ppu and result==want and vram==expected)
    rows.append(row);print(name,'PASS' if row['passed'] else 'FAIL',flush=True)
    (out/'report.json').write_text(json.dumps(dict(script_sha256=sha(Path(__file__)),compiler_sha256=sha(compiler),emulator_sha256=sha(emulator),header_sha256=sha(REPO/'lib/intrinsics.h'),cases=rows),indent=2),encoding='utf-8')
print('Report:',out/'report.json',flush=True)
raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
