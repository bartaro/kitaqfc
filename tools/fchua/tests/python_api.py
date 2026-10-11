"""Run after installing the optional extension: python tests/python_api.py."""
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

import fchua


def png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


class PythonApiTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.editor = fchua.Controller()
        self.editor.new_blank()

    def test_headless_module(self):
        self.assertFalse(fchua.GUI_AVAILABLE)
        self.assertFalse(hasattr(fchua, "run_editor"))
        self.assertEqual(fchua.__version__, "0.1.0")
        self.assertEqual(len(self.editor.chr_bytes()), 8192)

    def test_transaction_and_project_roundtrip(self):
        self.editor.apply_commands('[{"op":"pixel","tile":1,"x":2,"y":3,"color":2}]')
        before = self.editor.chr_bytes()
        with self.assertRaises(ValueError):
            self.editor.apply_commands('[{"op":"fill","tile":1,"color":1},{"op":"pixel","tile":3,"x":8,"y":0,"color":1}]')
        self.assertEqual(self.editor.chr_bytes(), before)
        path = self.root / "scene.fch"
        self.editor.save_project(str(path))
        self.assertEqual(json.loads(path.read_text())["format"], "fchua")
        restored = fchua.Controller()
        restored.load_project(str(path))
        self.assertEqual(restored.chr_bytes(), before)
        self.assertEqual(restored.get_tile_pixel(1, 2, 3), 2)

    def test_png_and_all_frame_exports(self):
        pixels = (b"\0" + bytes([236, 238, 236, 255]) * 8) * 8
        png = b"\x89PNG\r\n\x1a\n"
        png += png_chunk(b"IHDR", struct.pack(">IIBBBBB", 8, 8, 8, 6, 0, 0, 0))
        png += png_chunk(b"IDAT", zlib.compress(pixels)) + png_chunk(b"IEND", b"")
        path = self.root / "art.png"
        path.write_bytes(png)
        report = json.loads(self.editor.import_png(str(path), True))
        self.assertIsInstance(report, dict)
        preview = self.root / "preview.png"
        self.editor.preview_png(str(preview), "map", 2)
        self.assertEqual(struct.unpack(">II", preview.read_bytes()[16:24]), (512, 480))
        self.editor.apply_commands('[{"op":"sprite","frame":1,"index":0,"x":-8,"y":0,"tile":1,"attr":64}]')
        self.editor.export_raw_assets(str(self.root / "assets"), "scene")
        for ext, size in [("chr", 8192), ("pal", 32), ("nam", 960), ("atr", 64)]:
            self.assertEqual((self.root / f"assets/scene.{ext}").stat().st_size, size)
        self.assertIn("scene_metasprite_1", (self.root / "assets/scene_assets.c").read_text())


if __name__ == "__main__":
    unittest.main()
