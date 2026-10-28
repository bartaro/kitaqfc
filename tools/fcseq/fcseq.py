"""FCSEQ: general-purpose Famicom sequencer. GUI, CLI and Python share the native engine."""
from __future__ import annotations
import argparse, json, subprocess, sys
from pathlib import Path
import fcseq_native
ROOT = Path(__file__).resolve().parent
EXE_SUFFIX = '.exe' if sys.platform == 'win32' else ''
FPS = 60.098813897

def controller(doc=None):
    c = fcseq_native.Controller()
    if doc is not None:
        c.load_json(json.dumps(doc, ensure_ascii=False, allow_nan=False))
    return c

def load(path):
    c = controller(); c.load_project(str(path)); return json.loads(c.project_json())

def save(doc, path):
    controller(doc).save_project(str(path))

def validate(doc):
    c = controller(doc); c.validate(); doc = json.loads(c.project_json())
    if doc.get('nsf'):
        return json.loads(c.nsf_info())
    pcm_channel={'mmc5':7,'vt02':9}.get(doc['expansion'])
    tick_rate=4000.0 if pcm_channel is not None and any(s['tracks'][pcm_channel]['notes'] for s in doc['sections']) else FPS
    fpr = int(tick_rate * 15 / doc['bpm'] + .5)
    duration = sum(int(s['rows'] * tick_rate * 15 / doc['bpm'] + .5) / tick_rate for s in doc['sections'])
    total_rows = sum(s['rows'] for s in doc['sections'])
    return dict(sections=len(doc['sections']), channels=len(doc['sections'][0]['tracks']),
                expansion=doc['expansion'], notes=sum(len(t['notes']) for s in doc['sections'] for t in s['tracks']),
                frames_per_row=fpr, ticks_per_row=fpr, average_ticks_per_row=tick_rate * 15 / doc['bpm'],
                requested_bpm=doc['bpm'], actual_bpm=total_rows * 15 / duration, tick_rate_hz=tick_rate,
                timing='accumulated-row-deadlines', duration_seconds=duration)

def export_kitaqfc(doc, directory, header='fcseq_score.h'):
    return json.loads(controller(doc).export_kitaqfc(str(directory), header))

def render_wav(doc, path, section=0):
    return json.loads(controller(doc).render_wav(str(path), section))

def export_nsf(doc, path):
    return json.loads(controller(doc).export_nsf(str(path)))

def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='command')
    g = sub.add_parser('gui'); g.add_argument('project', nargs='?')
    n = sub.add_parser('new'); n.add_argument('output'); n.add_argument('--chip', choices=['none','vrc6','vrc7','fds','n163','mmc5','sunsoft5b','epsm','vt02'], default='none')
    v = sub.add_parser('validate'); v.add_argument('project')
    e = sub.add_parser('export-kitaqfc'); e.add_argument('project'); e.add_argument('output')
    x = sub.add_parser('export-nsf'); x.add_argument('project'); x.add_argument('output')
    x = sub.add_parser('export-nsf2'); x.add_argument('project'); x.add_argument('output')
    x = sub.add_parser('export-nsfe'); x.add_argument('project'); x.add_argument('output')
    x = sub.add_parser('import-nsf'); x.add_argument('project'); x.add_argument('output')
    for name in ('nsf-info','nsf-render','play-nsf'):
        x = sub.add_parser(name); x.add_argument('project')
        if name=='nsf-render': x.add_argument('output')
        x.add_argument('--song',type=int); x.add_argument('--seconds',type=int)
    w = sub.add_parser('render-wav'); w.add_argument('project'); w.add_argument('output'); w.add_argument('--section',type=int,default=0)
    a = sub.add_parser('import-audio'); a.add_argument('project'); a.add_argument('audio'); a.add_argument('output')
    d = sub.add_parser('export-dpcm'); d.add_argument('project'); d.add_argument('sample',type=int); d.add_argument('output')
    args = p.parse_args()
    if args.command in (None,'gui'):
        cmd=[str(ROOT/('fcseq'+EXE_SUFFIX))]
        if getattr(args,'project',None): cmd.append(args.project)
        subprocess.run(cmd,check=True); return
    c = controller()
    if args.command=='new':
        c.set_expansion(args.chip); c.save_project(args.output); result={'created':args.output}
    else:
        c.load_project(args.project)
        if getattr(args,'song',None) is not None: c.set_nsf_song(args.song)
        if getattr(args,'seconds',None) is not None: c.set_nsf_seconds(args.seconds)
        if args.command=='validate': result=validate(json.loads(c.project_json()))
        elif args.command in ('export-nsf','export-nsfe','export-nsf2'):
            path=Path(args.output)
            if args.command!='export-nsf': path=path.with_suffix('.'+args.command.removeprefix('export-'))
            result=json.loads(c.export_nsf(str(path)))
        elif args.command=='import-nsf':
            result=json.loads(c.nsf_info()); c.save_project(args.output)
        elif args.command=='nsf-info': result=json.loads(c.nsf_info())
        elif args.command=='nsf-render': result=json.loads(c.render_wav(args.output))
        elif args.command=='play-nsf':
            info=json.loads(c.nsf_info())
            command=[str(ROOT/('fcseq-cli'+EXE_SUFFIX)),'play-nsf',str(args.project),str(info['selected_song'])]
            if info.get('duration_ms') is None: command.append(str(info['seconds']))
            subprocess.run(command,check=True)
            return
        elif args.command=='export-kitaqfc': result=json.loads(c.export_kitaqfc(args.output))
        elif args.command=='render-wav': result=json.loads(c.render_wav(args.output,args.section))
        elif args.command=='import-audio':
            result={'sample':c.import_audio(args.audio)}; c.save_project(args.output)
        else: c.export_dpcm(args.sample,args.output); result={'output':args.output}
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    try: main()
    except (ValueError,RuntimeError,OSError) as exc:
        raise SystemExit(f'FCSEQ: {exc}')
