"""Collect unmodified Windows dependency licenses for CLI/Python or GUI."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--gui", action="store_true")
args = parser.parse_args()
features = "gui" if args.gui else "python"
license_dir = "gui-licenses" if args.gui else "licenses"
inventory = "GUI_DEPENDENCIES.json" if args.gui else "DEPENDENCIES.json"
metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
    "--features", features, "--filter-platform", "x86_64-pc-windows-msvc"], cwd=root))
packages = {p["id"]: p for p in metadata["packages"]}
nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
pending = [metadata["resolve"]["root"]]
seen = set()
while pending:
    ident = pending.pop()
    if ident in seen:
        continue
    seen.add(ident)
    pending.extend(d["pkg"] for d in nodes[ident]["deps"]
                   if any(k["kind"] != "dev" for k in d["dep_kinds"]))
records = []
missing = []
for ident in sorted(seen):
    package = packages[ident]
    if not package.get("source"):
        continue
    source = Path(package["manifest_path"]).parent
    crate_source = source
    files = [p for p in source.rglob("*") if p.is_file() and
             p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE", "AUTHORS"))]
    if not files:
        source = root / "license-overrides" / f"{package['name']}-{package['version']}"
        files = [p for p in source.rglob("*") if p.is_file()]
    if not files:
        missing.append({k: package.get(k) for k in ["name", "version", "license", "repository", "manifest_path"]})
        continue
    notices = []
    for path in files:
        target = root / license_dir / f"{package['name']}-{package['version']}" / path.relative_to(source)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
        notices.append(target.relative_to(root).as_posix())
    if package["name"] == "khronos_api":
        excerpts = []
        for folder in ["api/xml", "api_egl/api", "api_angle/scripts"]:
            for path in sorted((crate_source / folder).glob("*.xml")):
                document = ET.parse(path)
                for comment in document.findall("./comment"):
                    value = "".join(comment.itertext())
                    if "copyright" in value.lower():
                        excerpts.append(path.relative_to(crate_source).as_posix() + "\n" + value)
        if not excerpts:
            raise RuntimeError("Missing Khronos XML copyright notices")
        target = root / license_dir / f"{package['name']}-{package['version']}" / "NOTICE-XML.txt"
        target.write_text("\n\n".join(excerpts), encoding="utf-8")
        notices.append(target.relative_to(root).as_posix())
    records.append({"name": package["name"], "version": package["version"],
                    "license": package["license"], "source": package["source"], "notices": notices})
if missing:
    raise RuntimeError("Missing licenses: " + json.dumps(missing, indent=2))
sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
docs = sysroot / "share/doc/rust"
for path in [docs / "COPYRIGHT-library.html", *(docs / "licenses").rglob("*")]:
    if path.is_file():
        target = root / license_dir / "rust-standard-library" / path.relative_to(docs)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
(root / inventory).write_text(json.dumps({
    "scope": f"Windows {features} normal/build dependency closure, including code generators",
    "packages": records,
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"packages": len(records), "missing": 0}))
