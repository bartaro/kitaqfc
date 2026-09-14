"""Check readonly array padding and overflow through the actual compiler CLI.

Run with Python 3: python scripts/test-readonly-arrays.py --compiler kitaqfc.exe
Each run retains generated sources, ROMs, logs and report.json in a new directory.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    # Resolve paths before switching each compiler subprocess to its own directory.
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path,
                        default=Path(__file__).resolve().parent.parent/'kitaqfc.exe')
    parser.add_argument('--output-parent', type=Path,
                        help='Parent for a fresh result directory; defaults to the system temp directory.')
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    parent = args.output_parent.resolve() if args.output_parent else None
    if parent:
        parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='fc-readonly-', dir=parent))

    # The same payload bytes distinguish emitted array data from instruction bytes.
    # Word cases also prove little-endian encoding and element-to-byte extent scaling.
    cases = [
        ('byte-literal', 'u8', '8', '', 8, False),
        ('byte-expression', 'u8', '4+4', '', 8, False),
        ('byte-named', 'u8', 'COUNT', 'const u8 COUNT = 8;\n', 8, False),
        ('byte-inferred', 'u8', '', '', 4, False),
        ('byte-exact', 'u8', '2+2', '', 4, False),
        ('word-literal', 'u16', '4', '', 8, False),
        ('word-expression', 'u16', '2+2', '', 8, False),
        ('word-named', 'u16', 'COUNT', 'const u8 COUNT = 4;\n', 8, False),
        ('word-inferred', 'u16', '', '', 4, False),
        ('overflow-literal', 'u8', '3', '', 3, True),
        ('overflow-expression', 'u8', '1+2', '', 3, True),
        ('overflow-named', 'u8', 'COUNT', 'const u8 COUNT = 3;\n', 3, True),
    ]
    marker = bytes([0xD3, 0x6B, 0xA7, 0x5E])
    results = []
    for name, element_type, dimension, preamble, expected_bytes, expect_error in cases:
        case_dir = output/name
        case_dir.mkdir()
        source = case_dir/'case.c'
        values = '0xD3, 0x6B, 0xA7, 0x5E' if element_type == 'u8' else '0x6BD3, 0x5EA7'
        source.write_text(preamble+'const '+element_type+' probe_data['+dimension+'] = { '+values+' };\n'
                          +element_type+' sink;\nvoid main(void) { sink = probe_data[0]; while (1) { } }\n',
                          encoding='ascii')
        rom = case_dir/'case.nes'
        command = [str(compiler), str(source), '--mapper=nrom', '--no-disasm', '--no-cache', '-o', str(rom)]

        # A bounded compiler failure is recorded, including timeouts; no stale ROM
        # can satisfy a case because every output directory is newly created.
        timed_out = False
        try:
            run = subprocess.run(command, cwd=case_dir, capture_output=True, text=True,
                                 encoding='utf-8', errors='replace', timeout=30)
            exit_code = run.returncode
            log = run.stdout+run.stderr
        except subprocess.TimeoutExpired as error:
            timed_out = True
            exit_code = None
            def decode(value):
                return value.decode('utf-8', errors='replace') if isinstance(value, bytes) else value or ''
            log = decode(error.stdout)+decode(error.stderr)+'\nCompiler timed out.\n'
        (case_dir/'compile.log').write_text(log, encoding='utf-8')

        # Check the actual ROM payload and its following fill byte. This catches
        # truncation, missing zero padding and over-allocation in the valid cases.
        data = rom.read_bytes() if rom.exists() else b''
        positions = [i for i in range(len(data)) if data.startswith(marker, i)]
        if expect_error:
            passed = not timed_out and exit_code != 0 and not rom.exists() and 'initializer' in log.lower()
        else:
            padded = marker+bytes(expected_bytes-len(marker))
            passed = (exit_code == 0 and data.startswith(b'NES\x1a') and len(positions) == 1 and
                      data[positions[0]:positions[0]+expected_bytes+1] == padded+b'\xff')
        results.append(dict(name=name, passed=passed, command=command, exit_code=exit_code,
                            timed_out=timed_out, expected_error=expect_error,
                            expected_bytes=expected_bytes, marker_offsets=positions,
                            source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                            rom_sha256=hashlib.sha256(data).hexdigest() if data else None))

    # Preserve executable identity with the results; this is compiler/ROM-layout
    # verification, not a claim of emulator or physical-hardware execution.
    report = dict(compiler=str(compiler), compiler_sha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),
                  cases=results, scope='FC compiler CLI, readonly ROM encoding and overflow diagnostics.')
    (output/'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    failed = [r['name'] for r in results if not r['passed']]
    print(json.dumps(dict(passed=len(results)-len(failed), failed=failed, report=str(output/'report.json'))))
    return 1 if failed else 0


if __name__ == '__main__':
    raise SystemExit(main())
