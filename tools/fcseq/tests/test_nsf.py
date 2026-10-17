from pathlib import Path
import sys, json, tempfile, unittest, wave
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import fcseq_native

class NsfTests(unittest.TestCase):
    def test_section_duration_metadata_and_nsfe_playback(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            c = fcseq_native.Controller()
            c.set_bars(0, 1)
            c.add_note(0, 0, 0, 4, 69)
            result = json.loads(c.export_nsf(str(root / 'music.nsf')))
            self.assertEqual(result['tracks'][0]['time_ms'], 1597)
            self.assertEqual(result['tracks'][0]['fade_ms'], 0)
            companion = root / 'music.nsfe'
            self.assertTrue(companion.read_bytes().startswith(b'NSFE'))
            imported = fcseq_native.Controller()
            imported.import_nsf(str(companion))
            self.assertEqual(json.loads(imported.nsf_info())['duration_ms'], 1597)
            imported.render_wav(str(root / 'exact.wav'))
            with wave.open(str(root / 'exact.wav')) as audio:
                self.assertEqual(audio.getnframes(), 70428)
            imported.set_nsf_seconds(1)
            imported.render_wav(str(root / 'override.wav'))
            with wave.open(str(root / 'override.wav')) as audio:
                self.assertEqual(audio.getnframes(), 44100)

    def test_export_import_fcq_embedded_playback_and_original_preservation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            c = fcseq_native.Controller()
            c.set_expansion('vrc7')
            c.set_bars(0, 1)
            c.add_note(0, 5, 0, 4, 69)
            c.add_section('Second')
            c.set_bars(1, 1)
            c.add_note(1, 0, 0, 4, 72)
            original = root / 'music.nsf'
            result = json.loads(c.export_nsf(str(original)))
            self.assertEqual(result['songs'], 2)
            imported = fcseq_native.Controller()
            imported.import_nsf(str(original))
            imported.set_nsf_song(2)
            imported.set_nsf_seconds(1)
            project = root / 'imported.fcq'
            imported.save_project(str(project))
            source_bytes = original.read_bytes()
            original.unlink()
            reopened = fcseq_native.Controller()
            reopened.load_project(str(project))
            self.assertEqual(json.loads(reopened.nsf_info())['selected_song'], 2)
            reopened.export_nsf(str(original))
            self.assertEqual(original.read_bytes(), source_bytes)
            reopened.render_wav(str(root / 'music.wav'))
            with wave.open(str(root / 'music.wav')) as audio:
                self.assertEqual(audio.getnframes(), 44100)
                self.assertGreater(max(audio.readframes(44100)), 0)
            before = reopened.project_json()
            for setter, value in [(reopened.set_nsf_song, 3), (reopened.set_nsf_song, 0),
                                  (reopened.set_nsf_seconds, 0), (reopened.set_nsf_seconds, 601)]:
                with self.assertRaises(ValueError): setter(value)
                self.assertEqual(reopened.project_json(), before)

    def test_renamed_legacy_project_opens_as_fcq(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            c = fcseq_native.Controller()
            c.add_note(0, 0, 0, 4, 69)
            path = root / 'old.fcseq'
            c.save_project(str(path))
            renamed = path.with_suffix('.fcq')
            path.rename(renamed)
            reopened = fcseq_native.Controller()
            reopened.load_project(str(renamed))
            self.assertEqual(c.project_json(), reopened.project_json())

if __name__ == '__main__': unittest.main()
