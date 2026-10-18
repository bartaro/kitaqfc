import json, tempfile, unittest, math, struct, wave
from pathlib import Path
import fcseq_native

class ExpansionTests(unittest.TestCase):
    def test_pcm_note_uses_first_imported_sample(self):
        with tempfile.TemporaryDirectory() as directory:
            source=Path(directory)/'sine.wav'
            with wave.open(str(source),'wb') as f:
                f.setparams((1,2,22050,0,'NONE','not compressed'))
                f.writeframes(b''.join(struct.pack('<h',int(12000*math.sin(i*2*math.pi*440/22050))) for i in range(2205)))
            for chip,ch in [('mmc5',7),('vt02',9)]:
                c=fcseq_native.Controller();c.set_expansion(chip);c.set_bars(0,1);c.import_audio(str(source))
                c.add_note(0,ch,0,4,60)
                self.assertEqual(json.loads(c.project_json())['sections'][0]['tracks'][ch]['notes'][0]['sound']['instrument'],0)
                out=Path(directory)/(chip+'.wav');c.render_wav(str(out))
                with wave.open(str(out)) as f:
                    samples=struct.unpack('<'+'h'*f.getnframes(),f.readframes(f.getnframes()))
                self.assertGreater(max(samples)-min(samples),1000)

    def test_new_chips_nsf2_roundtrip_and_parameter_history(self):
        with tempfile.TemporaryDirectory() as directory:
            for chip, count in [('mmc5',8),('sunsoft5b',8),('epsm',20),('vt02',10)]:
                c=fcseq_native.Controller();c.set_expansion(chip)
                self.assertEqual(len(json.loads(c.project_json())['sections'][0]['tracks']),count)
                c.add_note(0,5,0,4,69)
                if chip in ('sunsoft5b','epsm'):
                    c.set_sound(0,5,0,json.dumps({'tone':True,'noise':True,'noise_period':6,'envelope':True,'envelope_shape':10,'envelope_period':500}))
                if chip=='epsm':
                    before=c.project_json();c.set_epsm_parameter(0,44,4)
                    self.assertNotEqual(c.project_json(),before);c.undo();self.assertEqual(c.project_json(),before);c.redo()
                path=Path(directory)/(chip+'.nsf2');result=json.loads(c.export_nsf(str(path)))
                self.assertEqual(result['format'],'nsf-2');self.assertEqual(path.read_bytes()[5],2)
                d=fcseq_native.Controller();d.import_nsf(str(path));info=json.loads(d.nsf_info());self.assertEqual(info['header']['version'],2)
                project=Path(directory)/(chip+'.fcq');c.save_project(str(project));d.load_project(str(project));self.assertEqual(c.project_json(),d.project_json())

if __name__=='__main__':unittest.main()
