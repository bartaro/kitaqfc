use crate::{
    editing::{self, Edit, Selection},
    model::*,
    nsf,
    samples::Sample,
    services,
};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::path::Path;
fn err(e: impl ToString) -> PyErr {
    PyValueError::new_err(e.to_string())
}
#[pyclass(name = "Controller")]
pub struct Controller {
    score: Score,
    undo: Vec<Score>,
    redo: Vec<Score>,
}
impl Controller {
    fn edit(&mut self, f: impl FnOnce(&mut Score) -> Result<(), String>) -> PyResult<()> {
        let before = self.score.clone();
        editing::transaction(&mut self.score, f).map_err(err)?;
        if self.score != before {
            self.undo.push(before);
            self.redo.clear();
            if self.undo.len() > 64 {
                self.undo.remove(0);
            }
        }
        Ok(())
    }
}
#[pymethods]
impl Controller {
    #[new]
    fn new() -> Self {
        Self {
            score: Score::empty(),
            undo: vec![],
            redo: vec![],
        }
    }
    fn load_project(&mut self, path: &str) -> PyResult<()> {
        self.score = Score::load(Path::new(path)).map_err(err)?;
        self.undo.clear();
        self.redo.clear();
        Ok(())
    }
    fn save_project(&self, path: &str) -> PyResult<()> {
        self.score.save(Path::new(path)).map_err(err)
    }
    fn import_nsf(&mut self, path: &str) -> PyResult<()> {
        let imported = nsf::import(Path::new(path)).map_err(err)?;
        self.edit(|s| {
            *s = imported;
            Ok(())
        })
    }
    fn export_nsf(&self, py: Python<'_>, path: &str) -> PyResult<String> {
        let score = self.score.clone();
        let path = Path::new(path).to_owned();
        py.allow_threads(move || {
            nsf::export(&score, &path)
                .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
        })
        .map_err(err)
    }
    fn nsf_info(&self) -> PyResult<String> {
        nsf::info(&self.score)
            .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
            .map_err(err)
    }
    fn set_nsf_song(&mut self, song: u8) -> PyResult<()> {
        self.edit(|s| {
            s.nsf
                .as_mut()
                .ok_or("Import an NSF first")?
                .select_song(song.checked_sub(1).ok_or("Song numbers start at 1")?)?;
            Ok(())
        })
    }
    fn set_nsf_seconds(&mut self, seconds: u32) -> PyResult<()> {
        self.edit(|s| {
            s.nsf.as_mut().ok_or("Import an NSF first")?.seconds = seconds;
            s.nsf.as_mut().unwrap().duration_ms = None;
            Ok(())
        })
    }
    fn project_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.score).map_err(err)
    }
    fn load_json(&mut self, json: &str) -> PyResult<()> {
        let score = Score::from_json(json).map_err(err)?;
        self.edit(|s| {
            *s = score;
            Ok(())
        })
    }
    fn validate(&self) -> PyResult<()> {
        self.score.validate().map_err(err)
    }
    fn set_tempo(&mut self, bpm: f64) -> PyResult<()> {
        self.edit(|s| {
            s.bpm = bpm;
            Ok(())
        })
    }
    fn set_name(&mut self, name: String) -> PyResult<()> {
        self.edit(|s| {
            s.name = name;
            Ok(())
        })
    }
    fn set_expansion(&mut self, chip: &str) -> PyResult<()> {
        let expansion = Expansion::parse(chip).map_err(err)?;
        self.edit(|s| s.set_expansion(expansion))
    }
    fn set_loop(&mut self, enabled: bool) -> PyResult<()> {
        self.edit(|s| {
            s.loop_playback = enabled;
            Ok(())
        })
    }
    fn add_section(&mut self, name: String) -> PyResult<()> {
        self.edit(|s| {
            s.sections
                .push(Section::with_channels(&name, s.expansion.channels()));
            Ok(())
        })
    }
    fn duplicate_section(&mut self, section: usize) -> PyResult<()> {
        self.edit(|s| {
            let mut part = s
                .sections
                .get(section)
                .ok_or("Section out of range")?
                .clone();
            part.name += " COPY";
            s.sections.insert(section + 1, part);
            Ok(())
        })
    }
    fn delete_section(&mut self, section: usize) -> PyResult<()> {
        self.edit(|s| {
            if section >= s.sections.len() {
                return Err("Section out of range".into());
            }
            s.sections.remove(section);
            Ok(())
        })
    }
    fn set_section_name(&mut self, section: usize, name: String) -> PyResult<()> {
        self.edit(|s| {
            s.sections
                .get_mut(section)
                .ok_or("Section out of range")?
                .name = name;
            Ok(())
        })
    }
    fn set_bars(&mut self, section: usize, bars: u16) -> PyResult<()> {
        self.edit(|s| {
            s.sections
                .get_mut(section)
                .ok_or("Section out of range")?
                .rows = bars.checked_mul(16).ok_or("Too many bars")?;
            Ok(())
        })
    }
    fn clear_track(&mut self, section: usize, channel: usize) -> PyResult<()> {
        self.edit(|s| {
            s.sections
                .get_mut(section)
                .and_then(|x| x.tracks.get_mut(channel))
                .ok_or("Track out of range")?
                .notes
                .clear();
            Ok(())
        })
    }
    fn set_track_mix(
        &mut self,
        section: usize,
        channel: usize,
        mute: bool,
        solo: bool,
    ) -> PyResult<()> {
        self.edit(|s| {
            let t = s
                .sections
                .get_mut(section)
                .and_then(|x| x.tracks.get_mut(channel))
                .ok_or("Track out of range")?;
            t.mute = mute;
            t.solo = solo;
            Ok(())
        })
    }
    fn add_note(
        &mut self,
        section: usize,
        channel: usize,
        row: u16,
        length: u16,
        note: u8,
    ) -> PyResult<()> {
        self.edit(|s| {
            let mut added = Note::new(row, length, note);
            if s.expansion.kind(channel) == Voice::Pcm {
                added.sound = Some(Sound { instrument: 0, ..Sound::default() });
            }
            s.sections
                .get_mut(section)
                .and_then(|x| x.tracks.get_mut(channel))
                .ok_or("Track out of range")?
                .notes
                .push(added);
            Ok(())
        })
    }
    fn set_note(
        &mut self,
        section: usize,
        channel: usize,
        index: usize,
        row: u16,
        length: u16,
        note: u8,
    ) -> PyResult<()> {
        self.edit(|s| {
            let n = s
                .sections
                .get_mut(section)
                .and_then(|x| x.tracks.get_mut(channel))
                .and_then(|x| x.notes.get_mut(index))
                .ok_or("Note out of range")?;
            n.row = row;
            n.length = length;
            n.note = note;
            Ok(())
        })
    }
    fn set_sound(
        &mut self,
        section: usize,
        channel: usize,
        index: usize,
        json: &str,
    ) -> PyResult<()> {
        let sound: Sound = serde_json::from_str(json).map_err(err)?;
        self.edit(|s| {
            s.sections
                .get_mut(section)
                .and_then(|x| x.tracks.get_mut(channel))
                .and_then(|x| x.notes.get_mut(index))
                .ok_or("Note out of range")?
                .sound = Some(sound);
            Ok(())
        })
    }
    fn delete_note(&mut self, section: usize, channel: usize, index: usize) -> PyResult<()> {
        self.edit(|s| {
            editing::edit_notes(s, section, &[(channel, index)], Edit::Delete).map(|_| ())
        })
    }
    #[pyo3(signature=(section,selection,operation,value=0))]
    fn edit_notes(
        &mut self,
        section: usize,
        selection: Selection,
        operation: &str,
        value: i32,
    ) -> PyResult<Selection> {
        let edit = match operation {
            "move" => Edit::Move(value),
            "transpose" => Edit::Transpose(i16::try_from(value).map_err(err)?),
            "resize" => Edit::Resize(value),
            "quantize" => Edit::Quantize(u16::try_from(value).map_err(err)?),
            "duplicate" => Edit::Duplicate,
            "delete" => Edit::Delete,
            "volume" => Edit::Volume(i16::try_from(value).map_err(err)?),
            "tone" => Edit::Tone,
            "noise" => Edit::PsgNoise,
            "envelope" => Edit::Envelope,
            "envelope-shape" => Edit::EnvelopeShape(i16::try_from(value).map_err(err)?),
            "envelope-period" => Edit::EnvelopePeriod(value),
            "pan" => Edit::Pan(i16::try_from(value).map_err(err)?),
            _ => return Err(err("Unknown operation")),
        };
        let mut out = vec![];
        self.edit(|s| {
            out = editing::edit_notes(s, section, &selection, edit)?;
            Ok(())
        })?;
        Ok(out)
    }
    fn import_audio(&mut self, path: &str) -> PyResult<usize> {
        let sample = Sample::import(Path::new(path)).map_err(err)?;
        let index = self.score.samples.len();
        self.edit(|s| {
            s.samples.push(sample);
            Ok(())
        })?;
        Ok(index)
    }
    fn set_sample_settings(
        &mut self,
        index: usize,
        start: u32,
        end: u32,
        gain: f32,
        normalize: bool,
        reverse: bool,
        rate: u8,
    ) -> PyResult<()> {
        self.edit(|s| {
            let sample = s.samples.get_mut(index).ok_or("Sample out of range")?;
            sample.start = start;
            sample.end = end;
            sample.gain = gain;
            sample.normalize = normalize;
            sample.reverse = reverse;
            sample.rate = rate;
            // PCM instruments retain full processed audio; DPCM encoding is
            // validated when the sample is used on the DPCM track or exported.
            sample.validate()?;
            Ok(())
        })
    }
    fn export_dpcm(&self, index: usize, path: &str) -> PyResult<()> {
        let data = self
            .score
            .samples
            .get(index)
            .ok_or_else(|| err("Sample out of range"))?
            .encode()
            .map_err(err)?;
        std::fs::write(path, data).map_err(err)
    }
    fn set_fds_wave(&mut self, values: Vec<u8>) -> PyResult<()> {
        self.edit(|s| {
            s.fds_wave = values;
            Ok(())
        })
    }
    fn set_n163_wave(&mut self, index: usize, values: Vec<u8>) -> PyResult<()> {
        self.edit(|s| {
            *s.n163_waves
                .get_mut(index)
                .ok_or("Wave index out of range")? = values;
            Ok(())
        })
    }
    fn set_n163_channels(&mut self, channels: u8) -> PyResult<()> {
        self.edit(|s| {
            s.n163_channels = channels;
            Ok(())
        })
    }
    fn set_vrc7_parameter(&mut self, index: usize, value: u8) -> PyResult<()> {
        self.edit(|s| {
            crate::instruments::FM_PARAMETERS
                .get(index)
                .ok_or("FM parameter index out of range")?
                .set(&mut s.vrc7_patch, value)
        })
    }
    fn set_vrc7_patch(&mut self, values: Vec<u8>) -> PyResult<()> {
        let patch: [u8; 8] = values
            .try_into()
            .map_err(|_| err("Eight patch bytes required"))?;
        self.edit(|s| {
            s.vrc7_patch = patch;
            Ok(())
        })
    }
    fn set_epsm_patch(&mut self, index: usize, values: Vec<u8>) -> PyResult<()> {
        self.edit(|s| {
            *s.epsm_patches
                .get_mut(index)
                .ok_or("EPSM patch index must be 0..15")? = values;
            Ok(())
        })
    }
    fn set_epsm_parameter(&mut self, patch: usize, index: usize, value: u8) -> PyResult<()> {
        self.edit(|s| {
            crate::instruments::set_epsm_parameter(
                s.epsm_patches
                    .get_mut(patch)
                    .ok_or("EPSM patch index must be 0..15")?,
                index,
                value,
            )
        })
    }
    fn undo(&mut self) -> bool {
        if let Some(s) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.score, s));
            true
        } else {
            false
        }
    }
    fn redo(&mut self) -> bool {
        if let Some(s) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.score, s));
            true
        } else {
            false
        }
    }
    #[pyo3(signature=(directory,header="fcseq_score.h"))]
    fn export_kitaqfc(&self, py: Python<'_>, directory: &str, header: &str) -> PyResult<String> {
        if header != "fcseq_score.h" {
            return Err(err(
                "Generic export uses fcseq_score.h; include it from your game header",
            ));
        }
        let score = self.score.clone();
        let path = Path::new(directory).to_owned();
        py.allow_threads(move || {
            services::export(&score, &path)
                .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
        })
        .map_err(err)
    }
    #[pyo3(signature=(path,section=0))]
    fn render_wav(&self, py: Python<'_>, path: &str, section: usize) -> PyResult<String> {
        let score = self.score.clone();
        let path = Path::new(path).to_owned();
        py.allow_threads(move || {
            services::render_wav(&score, section, &path)
                .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
        })
        .map_err(err)
    }
}
#[pymodule]
fn fcseq_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Controller>()?;
    Ok(())
}
