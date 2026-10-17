"""Create a new original phrase; no game project or ROM is required."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
import fcseq_native
out=ROOT/'out/python_example';out.mkdir(parents=True,exist_ok=True)
c=fcseq_native.Controller();c.set_name('New phrase');c.set_expansion('vrc6');c.set_tempo(120)
for i,pitch in enumerate([60,64,67,72,71,67,64,62]):
 c.add_note(0,5,i*4,3,pitch)
c.set_sound(0,5,0,'{"volume":10,"duty":4,"decay":16}')
c.save_project(str(out/'editable.fcq'));c.export_kitaqfc(str(out/'kitaqfc'))
if '--render' in sys.argv: print(c.render_wav(str(out/'phrase.wav')))
print(out/'editable.fcq')
