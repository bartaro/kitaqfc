"""Retain actual native Rust test results for the binary distribution receipt."""
from pathlib import Path
import json, re, subprocess, sys

root = Path(__file__).resolve().parents[1]
result = subprocess.run(
    ['cargo', 'test', '--locked', '--release', '--tests'],
    cwd=root, text=True, encoding='utf-8', errors='replace', capture_output=True,
)
output = result.stdout + result.stderr
folder = root / 'verification'
folder.mkdir(exist_ok=True)
(folder / 'rust-tests.log').write_text(output, encoding='utf-8')
sys.stdout.write(output)
rows = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;', output)
passed = sum(int(p) for _, p, _ in rows)
assert result.returncode == 0 and rows and passed > 0
assert all(status == 'ok' and int(failed) == 0 for status, _, failed in rows)
(folder / 'rust-tests.json').write_text(
    json.dumps({'passed': passed, 'failed': 0, 'suites': len(rows)}, indent=2),
    encoding='utf-8',
)
