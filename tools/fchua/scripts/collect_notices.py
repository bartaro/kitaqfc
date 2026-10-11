"""Collect unmodified dependency licenses for redistributable CLI binaries."""
import json
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
    "--features", "python", "--filter-platform", "x86_64-pc-windows-msvc"], cwd=root))
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
for ident in sorted(seen):
    package = packages[ident]
    if not package.get("source"):
        continue
    source = Path(package["manifest_path"]).parent
    files = [p for p in source.rglob("*") if p.is_file() and
             p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE", "AUTHORS"))]
    if not files:
        raise RuntimeError(f"Missing license: {package['name']}")
    notices = []
    for path in files:
        target = root / "licenses" / f"{package['name']}-{package['version']}" / path.relative_to(source)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
        notices.append(target.relative_to(root).as_posix())
    records.append({"name": package["name"], "version": package["version"],
                    "license": package["license"], "source": package["source"], "notices": notices})
sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
docs = sysroot / "share/doc/rust"
for path in [docs / "COPYRIGHT-library.html", *(docs / "licenses").rglob("*")]:
    if path.is_file():
        target = root / "licenses/rust-standard-library" / path.relative_to(docs)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
(root / "DEPENDENCIES.json").write_text(json.dumps({"scope": "Windows CLI and optional Python normal/build dependency closure, including code generators", "packages": records}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"packages": len(records), "missing": 0}))
