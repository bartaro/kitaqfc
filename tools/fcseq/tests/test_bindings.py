from pathlib import Path
import sys,json,tempfile,unittest,wave,struct,math
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import fcseq_native
class BindingTests(unittest.TestCase):
 def setUp(self):self.c=fcseq_native.Controller()
 def test_atomic_history_and_redo(self):
  self.c.add_note(0,0,0,4,69);before=self.c.project_json()
  with self.assertRaises(ValueError):self.c.add_note(0,0,2,1,71)
  self.assertEqual(before,self.c.project_json());self.c.set_tempo(120);self.assertTrue(self.c.undo());self.assertEqual(before,self.c.project_json());self.assertTrue(self.c.redo())
 def test_expansion_edit_and_save(self):
  self.c.set_expansion('vrc7');self.c.add_note(0,10,0,4,69);self.c.set_sound(0,10,0,'{"instrument":8,"volume":11}')
  self.c.edit_notes(0,[(10,0)],'transpose',12)
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'score.fcseq';self.c.save_project(str(p));other=fcseq_native.Controller();other.load_project(str(p));self.assertEqual(self.c.project_json(),other.project_json())
   report=json.loads(self.c.render_wav(str(Path(t)/'fm.wav')));self.assertEqual(report['source'],'KUROSAKI core');self.assertGreater(report['samples'],0)
 def test_import_process_export_and_play_dpcm(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'input.wav'
   with wave.open(str(p),'wb') as w:
    w.setparams((2,2,44100,0,'NONE','not compressed'));w.writeframes(b''.join(struct.pack('<hh',int(math.sin(i*2*math.pi*440/44100)*16000),int(math.sin(i*2*math.pi*220/44100)*8000)) for i in range(4410)))
   i=self.c.import_audio(str(p));self.assertEqual(i,0)
   self.c.set_sample_settings(0,0,3000,.5,True,True,12);self.c.export_dpcm(0,str(Path(t)/'sample.dmc'));data=(Path(t)/'sample.dmc').read_bytes();self.assertEqual(len(data)%16,1)
   self.c.add_note(0,4,0,4,12);self.c.render_wav(str(Path(t)/'dpcm.wav'))
   old=self.c.project_json()
   with self.assertRaises(ValueError):self.c.set_sample_settings(0,4000,2000,1,True,False,15)
   self.assertEqual(old,self.c.project_json())
 def test_lengths_sections_and_custom_instruments(self):
  self.c.set_bars(0,256);self.c.add_note(0,0,4095,1,60)
  old=self.c.project_json()
  with self.assertRaises(ValueError):self.c.set_bars(0,1)
  self.assertEqual(old,self.c.project_json());self.c.duplicate_section(0);self.c.set_section_name(1,'B');self.c.delete_section(1)
  self.c.set_fds_wave([i for i in range(64)]);self.c.set_vrc7_patch([3,33,5,6,232,129,66,39])
if __name__=='__main__':unittest.main()
