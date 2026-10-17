"""Package a built Windows GUI with portable fonts, manuals and licenses."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("executable", type=Path)
args = parser.parse_args()
if not args.executable.is_file():
    parser.error("GUI executable does not exist")
files = {"FCHUA.exe": args.executable}
for pattern in ["README*.md", "VERIFICATION.md", "LICENSE", "THIRD_PARTY_NOTICES.md",
                "GUI_DEPENDENCIES.json", "assets/fonts/*", "gui-licenses/**/*"]:
    for path in root.glob(pattern):
        if path.is_file():
            files[path.relative_to(root).as_posix()] = path
files["vendor/plita/LICENSE"] = root / "vendor/plita/LICENSE"
files["vendor/README.md"] = root / "vendor/README.md"
files["license-overrides/SOURCES.json"] = root / "license-overrides/SOURCES.json"
for path in (root / "vendor/plita/licenses").rglob("*"):
    if path.is_file():
        files[path.relative_to(root).as_posix()] = path
for path in (root / "vendor/plita/crates/plita_ui/assets/lucide").rglob("*"):
    if path.is_file() and path.name.upper().startswith(("LICENSE", "NOTICE")):
        files[path.relative_to(root).as_posix()] = path
required = ["assets/fonts/Roboto[wdth,wght].ttf", "assets/fonts/Roboto-OFL.txt",
            "assets/fonts/RobotoMono-Regular.ttf", "assets/fonts/RobotoMono-OFL.txt",
            "GUI_DEPENDENCIES.json", "README.ja.md"]
if any(name not in files for name in required):
    raise RuntimeError("Missing fonts, notices or manuals")
inventory = json.loads(files["GUI_DEPENDENCIES.json"].read_text(encoding="utf-8"))
for package in inventory["packages"]:
    for notice in package["notices"]:
        if notice not in files:
            raise RuntimeError(f"Missing dependency notice: {notice}")
destination = root / "dist/FCHUA-GUI-0.1.0-windows-x86_64.zip"
destination.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    checksums = []
    for name, path in sorted(files.items()):
        data = path.read_bytes()
        info = zipfile.ZipInfo(name, (2026, 10, 18, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, data)
        checksums.append(f"{hashlib.sha256(data).hexdigest()}  {name}")
    archive.writestr("SHA256SUMS", "\n".join(checksums) + "\n")
print(json.dumps({"archive": destination.name, "files": len(files),
                  "bytes": destination.stat().st_size,
                  "sha256": hashlib.sha256(destination.read_bytes()).hexdigest()}))
