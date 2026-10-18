use fcseq_native::{
    editing::{edit_notes, Edit},
    model::*,
    nsf,
    samples::Sample,
    services,
};
use std::path::Path;
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |i: usize| {
        args.get(i)
            .map(String::as_str)
            .ok_or_else(|| "Missing argument; run fcseq-cli help".to_string())
    };
    let result = match args.first().map(String::as_str).unwrap_or("help") {
        "new" => {
            let mut s = Score::empty();
            if let Some(chip) = args.get(2) {
                s.set_expansion(Expansion::parse(chip)?)?;
            }
            s.save(Path::new(get(1)?))?;
            serde_json::json!({"created":get(1)?})
        }
        "validate" => {
            let s = Score::load(Path::new(get(1)?))?;
            if s.nsf.is_some() {
                nsf::info(&s)?
            } else {
                serde_json::json!({"valid":true,"expansion":s.expansion,"channels":s.expansion.channels(),"sections":s.sections.len(),"frames_per_row":s.frames_per_row()})
            }
        }
        "export-nsf" | "export-nsfe" => {
            nsf::export(&Score::load(Path::new(get(1)?))?, Path::new(get(2)?))?
        }
        "import-nsf" | "nsf-render" | "play-nsf" | "nsf-info" => {
            let mut s = Score::load(Path::new(get(1)?))?;
            let play = args[0] == "play-nsf";
            let offset = if play { 2 } else { 3 };
            let asset = s.nsf.as_mut().ok_or("Input does not contain NSF music")?;
            if args[0] != "nsf-info" {
                if let Some(song) = args.get(offset) {
                    let song: u8 = song.parse().map_err(|_| "Song must be 1..255")?;
                    asset.select_song(song.checked_sub(1).ok_or("Song numbers start at 1")?)?;
                }
                if let Some(seconds) = args.get(offset + 1) {
                    asset.seconds = seconds.parse().map_err(|_| "Seconds must be 1..600")?;
                    asset.duration_ms = None;
                }
            }
            s.validate()?;
            match args[0].as_str() {
                "import-nsf" => {
                    s.save(Path::new(get(2)?))?;
                    nsf::info(&s)?
                }
                "nsf-render" => services::render_wav(&s, 0, Path::new(get(2)?))?,
                "play-nsf" => {
                    let audio = services::render(&s, 0, None)?;
                    let (_device, handle) =
                        rodio::OutputStream::try_default().map_err(|e| e.to_string())?;
                    let sink = rodio::Sink::try_new(&handle).map_err(|e| e.to_string())?;
                    sink.append(rodio::buffer::SamplesBuffer::new(1, 44100, audio));
                    sink.sleep_until_end();
                    nsf::info(&s)?
                }
                _ => nsf::info(&s)?,
            }
        }
        "render-wav" => {
            let s = Score::load(Path::new(get(1)?))?;
            let part = args
                .get(3)
                .map(|s| s.parse::<usize>())
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(0);
            services::render_wav(&s, part, Path::new(get(2)?))?
        }
        "export-kitaqfc" => {
            let s = Score::load(Path::new(get(1)?))?;
            services::export(&s, Path::new(get(2)?))?
        }
        "import-audio" => {
            let mut s = Score::load(Path::new(get(1)?))?;
            s.samples.push(Sample::import(Path::new(get(2)?))?);
            s.save(Path::new(get(3)?))?;
            serde_json::json!({"sample":s.samples.len()-1,"project":get(3)?})
        }
        "export-dpcm" => {
            let s = Score::load(Path::new(get(1)?))?;
            let index: usize = get(2)?.parse::<usize>().map_err(|e| e.to_string())?;
            let data = s
                .samples
                .get(index)
                .ok_or("Sample out of range")?
                .encode()?;
            std::fs::write(get(3)?, &data).map_err(|e| e.to_string())?;
            serde_json::json!({"bytes":data.len()})
        }
        "edit" => {
            let mut s = Score::load(Path::new(get(1)?))?;
            let part = get(3)?.parse::<usize>().map_err(|e| e.to_string())?;
            let ch = get(4)?.parse::<usize>().map_err(|e| e.to_string())?;
            let ids = get(5)?
                .split(',')
                .map(|i| {
                    i.parse::<usize>()
                        .map(|i| (ch, i))
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = args
                .get(7)
                .map(|v| v.parse::<i32>())
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(0);
            let edit = match get(6)? {
                "move" => Edit::Move(value),
                "transpose" => {
                    Edit::Transpose(value.try_into().map_err(|_| "Invalid semitone offset")?)
                }
                "resize" => Edit::Resize(value),
                "quantize" => Edit::Quantize(value.try_into().map_err(|_| "Invalid grid")?),
                "delete" => Edit::Delete,
                "duplicate" => Edit::Duplicate,
                _ => return Err("Unknown edit operation".into()),
            };
            let selected = edit_notes(&mut s, part, &ids, edit)?;
            s.save(Path::new(get(2)?))?;
            serde_json::json!({"selection":selected,"output":get(2)?})
        }
        "help" | "--help" | "-h" => {
            println!("FCSEQ {}\nnew OUTPUT.fcq [none|vrc6|vrc7|fds|n163]\nvalidate PROJECT\nrender-wav PROJECT OUTPUT.wav [SECTION]\nexport-kitaqfc PROJECT DIRECTORY\nexport-nsf PROJECT OUTPUT.nsf
export-nsfe PROJECT OUTPUT.nsfe\nimport-nsf INPUT.nsf OUTPUT.fcq [SONG] [SECONDS]\nnsf-info INPUT\nnsf-render INPUT OUTPUT.wav [SONG] [SECONDS]\nplay-nsf INPUT [SONG] [SECONDS]\nimport-audio PROJECT AUDIO OUTPUT.fcq\nexport-dpcm PROJECT SAMPLE OUTPUT.dmc\nedit PROJECT OUTPUT SECTION CHANNEL INDICES OPERATION [VALUE]\n  Operations: move, transpose, resize, quantize, duplicate, delete\nNSF song numbers start at 1; other indexes start at 0. Audio: WAV, FLAC, Ogg Vorbis, MP3.", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => return Err("Unknown command; run fcseq-cli help".into()),
    };
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("FCSEQ: {e}");
        std::process::exit(1);
    }
}
