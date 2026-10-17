from pathlib import Path
import json,sys,tempfile,unittest,wave
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import fcseq_native

class Instruments(unittest.TestCase):
    def test_n163_wave_history_fcq_nsf_and_wav(self):
        c=fcseq_native.Controller();c.set_expansion('n163');c.set_bars(0,1)
        c.set_n163_wave(0,[15 if i<16 else 0 for i in range(32)])
        c.add_note(0,12,0,4,69);c.set_sound(0,12,0,'{"instrument":0,"volume":12}')
        before=c.project_json()
        for op in [lambda:c.set_n163_channels(7),lambda:c.set_n163_wave(0,[16]*32),lambda:c.set_n163_wave(0,[0]*36)]:
            with self.assertRaises(ValueError):op()
            self.assertEqual(c.project_json(),before)
        c.set_n163_wave(0,list(range(16))*2);self.assertTrue(c.undo());self.assertEqual(c.project_json(),before);self.assertTrue(c.redo())
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp);c.save_project(str(p/'song.fcq'));d=fcseq_native.Controller();d.load_project(str(p/'song.fcq'));self.assertEqual(c.project_json(),d.project_json())
            c.export_nsf(str(p/'song.nsf'));self.assertEqual((p/'song.nsf').read_bytes()[123],16)
            d.import_nsf(str(p/'song.nsf'));d.set_nsf_seconds(1);d.render_wav(str(p/'song.wav'))
            with wave.open(str(p/'song.wav')) as w:self.assertEqual(w.getnframes(),44100);self.assertGreater(max(w.readframes(44100)),0)
    def test_fm_parameter_updates_only_requested_bits(self):
        c=fcseq_native.Controller();c.set_vrc7_patch([0]*8);c.set_vrc7_parameter(12,7)
        self.assertEqual(json.loads(c.project_json())['vrc7_patch'],[0,7,0,0,0,0,0,0])
        before=c.project_json()
        with self.assertRaises(ValueError):c.set_vrc7_parameter(12,16)
        self.assertEqual(c.project_json(),before)

if __name__=='__main__':unittest.main()
