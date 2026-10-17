import copy,json,tempfile,unittest,sys,wave,struct,math
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import fcseq as f
class ScoreTests(unittest.TestCase):
 def setUp(self):
  c=f.controller();c.add_note(0,0,0,4,69);self.doc=json.loads(c.project_json())
 def test_roundtrip(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'song.fcseq';f.save(self.doc,p);self.assertEqual(f.load(p),self.doc)
 def test_overlap_and_overrun_rejected(self):
  tr=self.doc['sections'][0]['tracks'][0];tr['notes'].append(copy.deepcopy(tr['notes'][0]))
  with self.assertRaisesRegex(ValueError,'Overlapping'):f.validate(self.doc)
  tr['notes'].pop();tr['notes'][0]['row']=63
  with self.assertRaisesRegex(ValueError,'outside'):f.validate(self.doc)
 def test_export_is_generic_and_preserves_register_stream(self):
  self.doc['bpm']=120
  with tempfile.TemporaryDirectory() as t:
   r=f.export_kitaqfc(self.doc,t);self.assertEqual(r['channels'],5);self.assertEqual(r['format'],'fcseq-export-2')
   self.assertTrue((Path(t)/'fcseq_player.c').exists());self.assertTrue((Path(t)/'section_0.bin').exists())
   self.assertNotIn('azure',(Path(t)/'score.c').read_text().lower())
 def test_low_pulse_and_nonfinite_tempo(self):
  self.doc['sections'][0]['tracks'][0]['notes'][0]['note']=24
  with self.assertRaises(ValueError):f.validate(self.doc)
  self.doc['bpm']=float('nan')
  with self.assertRaises(ValueError):f.validate(self.doc)
if __name__=='__main__':unittest.main()
