"""Verify the source tree and packaged GUI: python tests/distribution.py."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ("en", "ja", "ko", "zh-CN", "zh-TW", "es", "pt", "fr", "de")
ARCHIVE = ROOT / "dist/FCHUA-GUI-0.1.0-windows-x86_64.zip"


class DistributionTests(unittest.TestCase):
    def test_equivalent_manuals_and_branding(self):
        for language in LANGUAGES:
            text = (ROOT / f"README.{language}.md").read_text(encoding="utf-8")
            for required in [ARCHIVE.name, "FCHUA.exe", "assets/fonts", "plita",
                             "--features gui", "--features gui,python", "GUI_AVAILABLE=False",
                             "FCHUA_LANGUAGE", "FCHUA_CONFIG_DIR", "Noto Sans JP", "日本語 / English"]:
                if language == "en" and required == "GUI_AVAILABLE=False":
                    required = "GUI_AVAILABLE is False"
                self.assertIn(required, text, language)
            self.assertNotRegex(text, r"(?<![A-Za-z0-9_])PLITA(?![A-Za-z0-9_])")
            for link in re.findall(r"\]\(([^)]+)\)", text):
                if "://" not in link:
                    self.assertTrue((ROOT / link).is_file(), link)
        self.assertEqual((ROOT / "README.md").read_bytes(), (ROOT / "README.en.md").read_bytes())

    def test_curated_foundation_and_notices(self):
        crates = ROOT / "vendor/plita/crates"
        self.assertEqual({p.name for p in crates.iterdir()},
                         {"plita", "plita_app", "plita_window", "plita_ui", "plita_render"})
        data = json.loads((ROOT / "GUI_DEPENDENCIES.json").read_text(encoding="utf-8"))
        self.assertGreater(len(data["packages"]), 0)
        for package in data["packages"]:
            self.assertTrue(package["notices"], package["name"])
            for name in package["notices"]:
                self.assertGreater((ROOT / name).stat().st_size, 0, name)
        sources = json.loads((ROOT / "license-overrides/SOURCES.json").read_text(encoding="utf-8-sig"))
        for source in sources:
            self.assertEqual(hashlib.sha256((ROOT / source["path"]).read_bytes()).hexdigest(), source["sha256"])

    def test_gui_archive_and_internal_hashes(self):
        with zipfile.ZipFile(ARCHIVE) as archive:
            self.assertIsNone(archive.testzip())
            names = set(archive.namelist())
            for name in names:
                path = PurePosixPath(name)
                self.assertFalse(path.is_absolute() or ".." in path.parts)
            self.assertTrue(archive.read("FCHUA.exe").startswith(b"MZ"))
            for language in LANGUAGES:
                self.assertEqual(archive.read(f"README.{language}.md"),
                                 (ROOT / f"README.{language}.md").read_bytes())
            for font in ["Roboto[wdth,wght].ttf", "RobotoMono-Regular.ttf", "NotoSansJP-Regular.otf",
                         "NotoSansJP-Regular-OFL.txt", "NotoSansJP-Regular-PROVENANCE.json"]:
                self.assertEqual(archive.read("assets/fonts/" + font),
                                 (ROOT / "assets/fonts" / font).read_bytes())
            self.assertIn("vendor/plita/crates/plita_ui/assets/lucide/LICENSE", names)
            inventory = json.loads(archive.read("GUI_DEPENDENCIES.json"))
            for package in inventory["packages"]:
                for notice in package["notices"]:
                    self.assertIn(notice, names)
            hashed = set()
            for line in archive.read("SHA256SUMS").decode().splitlines():
                checksum, name = line.split("  ", 1)
                self.assertEqual(hashlib.sha256(archive.read(name)).hexdigest(), checksum, name)
                hashed.add(name)
            self.assertEqual(hashed, names - {"SHA256SUMS"})

    def test_japanese_font_provenance(self):
        fonts = ROOT / "assets/fonts"
        provenance = json.loads((fonts / "NotoSansJP-Regular-PROVENANCE.json").read_text(encoding="utf-8"))
        for name, entry in provenance["files"].items():
            data = (fonts / name).read_bytes()
            self.assertEqual(len(data), entry["bytes"])
            self.assertEqual(hashlib.sha256(data).hexdigest(), entry["sha256"])

    def test_published_artifact_hashes(self):
        for line in (ROOT / "SHA256SUMS").read_text().splitlines():
            checksum, name = line.split("  ", 1)
            self.assertEqual(hashlib.sha256((ROOT / name).read_bytes()).hexdigest(), checksum, name)


if __name__ == "__main__":
    unittest.main()
