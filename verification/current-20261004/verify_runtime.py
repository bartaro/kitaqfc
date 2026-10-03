"""Replay the two bounded FC library execution checks using explicit tools."""
from pathlib import Path
import argparse,json,subprocess,tempfile
p=argparse.ArgumentParser(__doc__)
p.add_argument('--compiler',type=Path,required=True)
p.add_argument('--emulator',type=Path,required=True)
p.add_argument('--library',type=Path,required=True)
args=p.parse_args();args.compiler=args.compiler.resolve();args.emulator=args.emulator.resolve();args.library=args.library.resolve()
sources=Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='kitaqfc-verify-') as temporary:
    root=Path(temporary)
    for name,linked,mapper,expected in [
        ('library_dispatch',['entity.c'],'mmc3',[0,1,1,4,2,4,1,0,255,13,165]),
        ('library_basics',[],'nrom',[254,36,7,5,255,254,165])]:
        rom=root/(name+'.nes');snapshot=root/(name+'.snapshot.json')
        compile_args=[args.compiler,sources/(name+'.c')]+[args.library/n for n in linked]+['-I',args.library,'--mapper='+mapper,'--no-cache','--no-disasm','-o',rom]
        subprocess.run(list(map(str,compile_args)),cwd=root,check=True)
        subprocess.run(list(map(str,[args.emulator,'snapshot-save','--frames','4','--out',snapshot,rom])),cwd=root,check=True)
        state=json.loads(snapshot.read_text(encoding='utf-8'))
        actual=state['bus']['ram'][0x700:0x700+len(expected)]
        assert actual==expected,(name,actual,expected)
        assert not state['cpu']['stopped'],name
        print(json.dumps({'case':name,'passed':True,'expected':expected,'actual':actual}))
