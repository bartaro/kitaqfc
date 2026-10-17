"""Require successful native window/GPU initialization and automatic shutdown."""
from pathlib import Path
import sys, subprocess, os, json, hashlib
folder=Path(sys.argv[1]).resolve()
exe=folder/('fcseq.exe' if os.name=='nt' else 'fcseq')
output=folder/'verification/ui'
env=dict(os.environ,FCSEQ_SMOKE_DIR=str(output))
result=subprocess.run([str(exe),'--startup-smoke'],cwd=folder,env=env,timeout=90,capture_output=True)
(folder/'verification/gui-startup.log').write_bytes(result.stdout+result.stderr)
assert result.returncode==0, result.stderr.decode(errors='replace')
proof=output/'native_event_loop.json'
assert proof.exists(), 'Native window did not produce its startup receipt'
data=json.loads(proof.read_text());assert data['empty_startup'] and data['ticks']==16
print(json.dumps(data))
