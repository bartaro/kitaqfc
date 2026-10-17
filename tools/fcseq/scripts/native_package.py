"""Build a portable native package and exercise its actual CLI/Python binaries."""
from pathlib import Path
import argparse, hashlib, json, math, os, platform, shutil, struct, subprocess, sys, tempfile, wave

ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--platform',required=True);p.add_argument('--output',default='dist');a=p.parse_args()
dest=(ROOT/a.output/a.platform).resolve();dest.mkdir(parents=True,exist_ok=True)
proof=dest/'verification';proof.mkdir(exist_ok=True)
for name in ['rust-tests.log','rust-tests.json']:
    shutil.copy2(ROOT/'verification'/name,proof/name)
rust_tests=json.loads((proof/'rust-tests.json').read_text())
assert rust_tests['passed']>0 and rust_tests['failed']==0
suffix='.exe' if os.name=='nt' else ''
for name in ['fcseq','fcseq-cli']:shutil.copy2(ROOT/'target/release'/(name+suffix),dest/(name+suffix))
library='fcseq_native.dll' if os.name=='nt' else ('libfcseq_native.dylib' if sys.platform=='darwin' else 'libfcseq_native.so')
extension='fcseq_native.pyd' if os.name=='nt' else 'fcseq_native.so'
shutil.copy2(ROOT/'target/release'/library,dest/extension)
for name in ['LICENSE','THIRD_PARTY_NOTICES.md','LICENSE_AUDIT.json','VERIFICATION.md','fcseq.py','fcseq_native.pyi']:
    shutil.copy2(ROOT/name,dest/name)
for name in ['licenses','third_party_sources','assets']:
    shutil.copytree(ROOT/name,dest/name,dirs_exist_ok=True)
for source in ROOT.glob('README*.md'):shutil.copy2(source,dest/source.name)
for source in (ROOT/'docs').glob('*.md'):
    (dest/'docs').mkdir(exist_ok=True);shutil.copy2(source,dest/'docs'/source.name)
sys.path.insert(0,str(dest))
import fcseq_native
cli=dest/('fcseq-cli'+suffix)
def run(*args):
    result=subprocess.run([str(cli),*map(str,args)],cwd=dest,text=True,encoding='utf-8',capture_output=True,timeout=90)
    assert result.returncode==0,result.stderr
    return result.stdout
def audio(path):
    with wave.open(str(path)) as w:
        data=w.readframes(w.getnframes());samples=struct.unpack('<'+'h'*(len(data)//2),data)
        assert w.getframerate()==44100 and len(samples)>100
        peak=max(samples)-min(samples);assert peak>0,(path,peak)
        return {'frames':len(samples),'peak_to_peak':peak,'sha256':hashlib.sha256(data).hexdigest()}
rows=[]
with tempfile.TemporaryDirectory() as tmp:
    tmp=Path(tmp);source=tmp/'sample.wav'
    with wave.open(str(source),'wb') as w:
        w.setparams((1,2,22050,0,'NONE','not compressed'))
        w.writeframes(b''.join(struct.pack('<h',int(16000*math.sin(i*2*math.pi*440/22050))) for i in range(2205)))
    for chip,count in [('none',5),('vrc6',8),('vrc7',11),('fds',6),('n163',13)]:
        c=fcseq_native.Controller();c.set_expansion(chip);c.set_bars(0,1);c.import_audio(str(source))
        for channel in range(count):
            c.add_note(0,channel,0,4,1 if channel==3 else (15 if channel==4 else 69))
        project=tmp/(chip+'.fcq');c.save_project(str(project))
        assert json.loads(run('validate',project))['valid']
        wav=tmp/(chip+'.wav');run('render-wav',project,wav,0)
        native=audio(wav)
        nsf=tmp/(chip+'.nsf');run('export-nsf',project,nsf)
        assert nsf.with_suffix('.nsfe').exists()
        run('nsf-render',nsf,wav,1,1);exported=audio(wav)
        imported=tmp/'import.fcq';run('import-nsf',nsf,imported,1,1)
        restored=tmp/'roundtrip.nsf';run('export-nsf',imported,restored)
        assert nsf.read_bytes()==restored.read_bytes()
        rows.append({'chip':chip,'voices':count,'native':native,'nsf':exported,'roundtrip':True})
    c=fcseq_native.Controller();c.set_expansion('n163');c.set_n163_channels(8);c.set_n163_wave(0,[i//2 for i in range(32)])
    c.set_expansion('vrc7');c.set_vrc7_parameter(0,3)
env=dict(os.environ,PYTHONPATH=str(dest),PYTHONUTF8='1')
# The existing tests insert their parent folder: run a copied test suite inside the package.
shutil.copytree(ROOT/'tests',dest/'tests',dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__'))
tested=subprocess.run([sys.executable,'-m','unittest','discover','-s','tests','-v'],cwd=dest,env=env,text=True,encoding='utf-8',capture_output=True,timeout=180)
(proof/'python-tests.log').write_text(tested.stdout+tested.stderr,encoding='utf-8');assert tested.returncode==0,tested.stderr
env['FCSEQ_SMOKE_DIR']=str(proof/'ui')
smoke=subprocess.run([str(dest/('fcseq'+suffix)),'--ui-smoke'],cwd=dest,env=env,capture_output=True,timeout=90)
assert smoke.returncode==0,smoke.stderr.decode(errors='replace')
assert (proof/'ui/ui_smoke.json').exists()
commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
record={'passed':True,'source_commit':commit,'platform':a.platform,'system':platform.platform(),'python':sys.version,'rust_tests':rust_tests,'audio_tests':rows,
        'binaries':{name:hashlib.sha256((dest/name).read_bytes()).hexdigest() for name in ['fcseq'+suffix,'fcseq-cli'+suffix,extension]},
        'native_audio_output_device_tested':False,'physical_hardware_tested':False}
(proof/'native.json').write_text(json.dumps(record,indent=2),encoding='utf-8')
print(json.dumps(record,indent=2))
