"""Exercise actual template/snippet output through KITAQFC and KUROSAKI.

Uses only the Python standard library. Keep the report directory as evidence;
the test never deletes an existing directory. Physical hardware is not tested.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import wave


def digest(path):
    """Fingerprint one produced artifact for independent report verification."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    """Generate original CLI artifacts, execute them, and report each acceptance check."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--emulator', required=True, type=Path)
    parser.add_argument('--output-parent', type=Path, default=Path('.'))
    parser.add_argument('--powershell', default='powershell')
    options = parser.parse_args()
    compiler = options.compiler.resolve()
    emulator = options.emulator.resolve()
    options.output_parent.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix='fc-generated-', dir=options.output_parent.resolve()))
    cases = []

    def run(command, folder, name, timeout=90):
        """Run an explicit argument list without a shell and retain both output streams."""
        process = subprocess.run([str(x) for x in command], cwd=folder,
                                 capture_output=True, timeout=timeout)
        (folder / (name + '.stdout.log')).write_bytes(process.stdout)
        (folder / (name + '.stderr.log')).write_bytes(process.stderr)
        return process

    def check(name, passed, **details):
        """Record failed checks as well as successful ones; never silently skip failures."""
        cases.append(dict(name=name, passed=bool(passed), **details))

    def build(name, text, mapper='nrom'):
        """Compile the emitted fragment with only the fixture code needed to observe it."""
        folder = root / name
        folder.mkdir()
        (folder / 'case.c').write_text(text, encoding='utf-8')
        process = run([compiler, 'case.c', '-o', 'case.nes', '--fast-build',
                       '--mapper=' + mapper], folder, 'compile')
        check(name + '-build', process.returncode == 0, exit_code=process.returncode)
        return folder if process.returncode == 0 else None

    def execute(folder, frames=6, pad=0, label='runtime', png=False):
        """Capture real ROM execution, including PPU and CPU-memory state."""
        command = [emulator, 'run', 'case.nes', '--frames', frames,
                   '--max-instructions', max(1000000, frames * 40000), '--headless',
                   '--pad1', pad, '--snapshot', label + '.json']
        if png:
            command += ['--png', label + '.png']
        process = run(command, folder, label)
        check(folder.name + '-' + label + '-run', process.returncode == 0,
              exit_code=process.returncode)
        return json.loads((folder / (label + '.json')).read_text(encoding='utf-8')) if process.returncode == 0 else None

    # Use the untouched generated source for the visible quickstart checks.
    template = root / 'template'
    template.mkdir()
    generation = run([compiler, 'template', 'case.c'], template, 'generate')
    check('template-generation', generation.returncode == 0)
    if generation.returncode == 0:
        source = (template / 'case.c').read_text(encoding='utf-8-sig')
        check('template-fc-registers', not any(x in source for x in ['0xFF40', '0xFF44', '0xFF47']))
        compiled = run([compiler, 'case.c', '-o', 'case.nes', '--fast-build'], template, 'compile')
        check('template-build', compiled.returncode == 0)
        if compiled.returncode == 0:
            for label, frames, pad, color in [('blue', 10, 0, 0x01), ('green', 45, 0, 0x1A),
                                             ('blue-again', 80, 0, 0x01), ('held-a', 10, 1, 0x16)]:
                snapshot = execute(template, frames, pad, label, png=True)
                if snapshot:
                    ppu = snapshot['bus']['ppu']
                    pixels = set(ppu['frame_buffer'])
                    check('template-' + label + '-pixels', ppu['palette'][0] == color and pixels == {color},
                          palette=ppu['palette'][0], pixel_values=sorted(pixels), expected=color)

    # Retrieve every catalog entry using the public interface, not a duplicated fixture body.
    catalog = root / 'catalog'
    catalog.mkdir()
    snippets = {}
    for snippet in ['input.poll_joypad', 'render.wait_vblank', 'render.clear_oam4',
                    'sound.beep_ch2', 'sram.enable_disable', 'sram.save_byte']:
        process = run([compiler, 'snippet', 'get', snippet], catalog, snippet)
        check('get-' + snippet, process.returncode == 0)
        snippets[snippet] = process.stdout.decode('utf-8-sig') if process.returncode == 0 else ''
    result_decl = '__location(0x0700) u8 result[8];\n'
    stop = 'result[7]=0xA5; while(1) { }'
    folder = build('controller', result_decl + snippets['input.poll_joypad'] +
                   '\nvoid main(void) { result[0]=PollJoypad(); ' + stop + ' }')
    if folder:
        for pad in [0, 1, 2, 0xA5, 0xFF]:
            snapshot = execute(folder, pad=pad, label='pad-' + str(pad))
            if snapshot:
                actual = snapshot['bus']['ram'][0x700:0x708]
                check('controller-bits-' + str(pad), actual[0] == pad and actual[7] == 0xA5, actual=actual)

    folder = build('vblank', result_decl + snippets['render.wait_vblank'] + '''
void main(void) {
    __ppu_ctrl_set(0); __ppu_off(); __ppu_read_status(); result[0]=0;
    while (1) { WaitVBlank(); result[0]=result[0]+1; result[7]=0xA5; }
}
''')
    if folder:
        snapshots = [execute(folder, frames=f, label='frame-' + str(f)) for f in [5, 9]]
        if all(snapshots):
            values = [s['bus']['ram'][0x700] for s in snapshots]
            check('vblank-frame-cadence', (values[1] - values[0]) % 256 == 4 and
                  all(s['bus']['ram'][0x707] == 0xA5 for s in snapshots), counts=values)

    setup = '\n'.join('__sprite_set(%d,%d,%d,%d,2);' % (i, 10+i, 20+i, 30+i) for i in range(6))
    folder = build('sprites', result_decl + snippets['render.clear_oam4'] +
                   '\nvoid main(void) { __ppu_ctrl_set(0); __ppu_off(); __oam_clear();\n' + setup +
                   '\nClearOam4(); ' + stop + ' }')
    if folder:
        snapshot = execute(folder)
        if snapshot:
            oam = snapshot['bus']['ppu']['oam'][:24]
            expected = [v for i in range(6) for v in [0xF0 if i < 4 else 20+i, 30+i, 2, 10+i]]
            check('hide-four-preserve-neighbors', oam == expected and snapshot['bus']['ram'][0x707] == 0xA5,
                  actual=oam, expected=expected)

    folder = build('pulse2', result_decl + snippets['sound.beep_ch2'] +
                   '\nvoid main(void) { SfxBeepCh2(); ' + stop + ' }')
    if folder:
        process = run([emulator, 'audio-export', 'case.nes', '--frames', 60,
                       '--wav', 'pulse.wav', '--json', 'audio.json'], folder, 'audio')
        check('pulse-audio-export', process.returncode == 0)
        if process.returncode == 0:
            with wave.open(str(folder / 'pulse.wav'), 'rb') as wav:
                assert wav.getsampwidth() == 2
                rate = wav.getframerate()
                raw = wav.readframes(wav.getnframes())
            samples = struct.unpack('<' + 'h' * (len(raw) // 2), raw)
            first = samples[:rate // 5]
            tail = samples[-rate // 5:]
            rms = lambda data: math.sqrt(sum(x*x for x in data) / max(1, len(data)))
            check('pulse-audible-then-decays', rms(first) > 100 and rms(tail) < rms(first) / 20,
                  sample_rate=rate, initial_rms=rms(first), final_rms=rms(tail))

    # MMC3 control is board-specific. The current emulator latches its protection
    # register but does not enforce it, so verify the latch and RAM round trip separately.
    sram = snippets['sram.enable_disable'] + '\n' + snippets['sram.save_byte']
    for state, value in [('Enable', 0x80), ('Disable', 0)]:
        folder = build('sram-' + state.lower(), result_decl + sram +
                       '\nvoid main(void) { SRAM_' + state + '(); ' + stop + ' }', 'mmc3')
        if folder:
            snapshot = execute(folder)
            if snapshot:
                check('mmc3-control-' + state.lower(), snapshot['mapper_private'][1] == value and
                      snapshot['bus']['ram'][0x707] == 0xA5, register=snapshot['mapper_private'][1])
    folder = build('sram-save', result_decl + sram + '''
void main(void) {
    result[1]=0x6D;
    SRAM_SaveByte(0x6000,0x55); SRAM_SaveByte(0x7FFF,0xA6);
    SRAM_SaveByte(0x0701,0x99); SRAM_SaveByte(0x5FFF,0x33); SRAM_SaveByte(0x8000,0x77);
    SRAM_Enable(); result[0]=*((u8*)0x6000); result[2]=*((u8*)0x7FFF); SRAM_Disable();
    result[7]=0xA5; while(1) { }
}
''', 'mmc3')
    if folder:
        snapshot = execute(folder)
        if snapshot:
            actual = snapshot['bus']['ram'][0x700:0x708]
            check('sram-window-and-guard', actual[:3] == [0x55, 0x6D, 0xA6] and actual[7] == 0xA5 and
                  snapshot['mapper_private'][1] == 0, actual=actual)
    bundle = run([compiler, 'snippet', 'emit', 'all'], catalog, 'all')
    check('all-snippets-emitted', bundle.returncode == 0)
    if bundle.returncode == 0:
        build('all-snippets', bundle.stdout.decode('utf-8-sig') + '\nvoid main(void) { while(1) { } }', 'mmc3')

    # Protect each output before generation, including sidecar-only collisions.
    for suffix in ['.c', '_build.ps1', '_README.md']:
        folder = root / ('collision-' + suffix.replace('.', '').replace('_', ''))
        folder.mkdir()
        protected = folder / ('demo' + suffix)
        protected.write_text('KEEP THIS FILE', encoding='utf-8')
        process = run([compiler, 'template', 'demo.c'], folder, 'generate')
        check('protect-' + suffix, process.returncode != 0 and protected.read_text(encoding='utf-8') == 'KEEP THIS FILE'
              and (suffix == '.c' or not (folder / 'demo.c').exists()))
    folder = root / 'unsupported-cgb'
    folder.mkdir()
    process = run([compiler, 'template', '--cgb'], folder, 'generate')
    check('reject-fc-cgb-option', process.returncode != 0 and not (folder / 'quickstart.c').exists())

    # Run the actual generated PowerShell script from another directory with a
    # filename containing spaces, a quote and a dollar sign; these must remain literal.
    folder = root / 'script-paths'
    folder.mkdir()
    name = "tiny sample's $value"
    process = run([compiler, 'template', name + '.c'], folder, 'generate')
    check('special-filename-generation', process.returncode == 0)
    if process.returncode == 0:
        script = folder / (name + '_build.ps1')
        for mode in ['dev', 'release']:
            command = [options.powershell, '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', script,
                       '-Compiler', compiler]
            if mode == 'release':
                command += ['-Release']
            process = run(command, root, 'script-' + mode)
            check('script-' + mode + '-from-other-directory', process.returncode == 0 and (folder / (name + '.nes')).exists(),
                  exit_code=process.returncode)
        (folder / (name + '.c')).write_text('void main( {\n', encoding='utf-8')
        process = run([options.powershell, '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', script,
                       '-Compiler', compiler], root, 'script-failed-build')
        check('script-propagates-failure', process.returncode != 0, exit_code=process.returncode)

    # Hash retained artifacts only after all commands finish. The report does not
    # claim real-board SRAM protection, battery retention, or physical audio timing.
    artifacts = {str(p.relative_to(root)): digest(p) for p in root.rglob('*') if p.is_file()}
    report = dict(compiler=str(compiler), compiler_sha256=digest(compiler), emulator=str(emulator),
                  emulator_sha256=digest(emulator), cases=cases, artifacts=artifacts,
                  limitations='KUROSAKI execution only. MMC3 protection latch and RAM writes checked; access enforcement and battery persistence not proven.')
    (root / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    failures = [c['name'] for c in cases if not c['passed']]
    print(json.dumps(dict(report=str(root / 'report.json'), checks=len(cases), failures=failures)))
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
