//! Atomic editing shared by the plita editor, CLI and Python.
use crate::model::*;
pub type Selection = Vec<(usize, usize)>;
#[derive(Clone, Copy, Debug)]
pub enum Edit {
    Move(i32),
    Transpose(i16),
    Resize(i32),
    Quantize(u16),
    Delete,
    Duplicate,
    Volume(i16),
    Duty(i16),
    Decay(i16),
    SawRate(i16),
    Instrument(i16),
    NoisePeriod(i16),
    NoiseMode,
}
pub fn transaction(
    score: &mut Score,
    f: impl FnOnce(&mut Score) -> Result<(), String>,
) -> Result<(), String> {
    let mut candidate = score.clone();
    f(&mut candidate)?;
    candidate.validate()?;
    *score = candidate;
    Ok(())
}
pub fn edit_notes(
    score: &mut Score,
    part: usize,
    selection: &[(usize, usize)],
    edit: Edit,
) -> Result<Selection, String> {
    if selection.is_empty() {
        return Err("Select notes first".into());
    }
    let ids: std::collections::BTreeSet<_> = selection.iter().copied().collect();
    let section = score.sections.get(part).ok_or("Section out of range")?;
    for &(ch, i) in &ids {
        section
            .tracks
            .get(ch)
            .and_then(|t| t.notes.get(i))
            .ok_or("Note out of range")?;
    }
    let mut result: Selection = ids.iter().copied().collect();
    transaction(score, |s| {
        let exp = s.expansion;
        let sample_max = s.samples.len().saturating_sub(1) as i16;
        let wave_max = s.n163_waves.len().saturating_sub(1) as i16;
        let sec = &mut s.sections[part];
        if matches!(edit, Edit::Duplicate) {
            let start = ids
                .iter()
                .map(|&(c, i)| sec.tracks[c].notes[i].row)
                .min()
                .unwrap();
            let end = ids
                .iter()
                .map(|&(c, i)| sec.tracks[c].notes[i].end())
                .max()
                .unwrap();
            result.clear();
            for &(c, i) in &ids {
                let mut n = sec.tracks[c].notes[i].clone();
                n.row = n.row.checked_add(end - start).ok_or("Position overflow")?;
                let index = sec.tracks[c].notes.len();
                sec.tracks[c].notes.push(n);
                result.push((c, index));
            }
        } else if matches!(edit, Edit::Delete) {
            for &(c, i) in ids.iter().rev() {
                sec.tracks[c].notes.remove(i);
            }
            result.clear();
        } else {
            for &(c, i) in &ids {
                let n = &mut sec.tracks[c].notes[i];
                match edit {
                    Edit::Move(d) => {
                        n.row = u16::try_from(n.row as i32 + d)
                            .map_err(|_| "Position outside section")?
                    }
                    Edit::Transpose(d) => {
                        n.note =
                            u8::try_from(n.note as i16 + d).map_err(|_| "Pitch outside range")?
                    }
                    Edit::Resize(d) => {
                        n.length =
                            u16::try_from(n.length as i32 + d).map_err(|_| "Invalid length")?
                    }
                    Edit::Quantize(grid) => {
                        if grid == 0 {
                            return Err("Grid must be positive".into());
                        }
                        n.row = (((n.row as u32 + grid as u32 / 2) / grid as u32) * grid as u32)
                            .try_into()
                            .map_err(|_| "Position overflow")?;
                    }
                    _ => {
                        let mut sound = n.sound(c);
                        match edit {
                            Edit::Volume(d) => {
                                sound.volume = (sound.volume as i16 + d).clamp(0, 15) as u8
                            }
                            Edit::Duty(d) => {
                                sound.duty = (sound.duty as i16 + d)
                                    .clamp(0, if exp.kind(c) == Voice::VrcPulse { 7 } else { 3 })
                                    as u8
                            }
                            Edit::Decay(d) => {
                                sound.decay = (sound.decay as i16 + d).clamp(0, 255) as u8
                            }
                            Edit::SawRate(d) => {
                                sound.saw_rate = Some(
                                    (sound.saw_rate.unwrap_or(sound.volume * 2) as i16 + d)
                                        .clamp(0, 63) as u8,
                                )
                            }
                            Edit::Instrument(d) => {
                                sound.instrument = (sound.instrument as i16 + d).clamp(
                                    0,
                                    if c == 4 {
                                        sample_max
                                    } else if exp.kind(c) == Voice::N163 {
                                        wave_max
                                    } else {
                                        15
                                    },
                                ) as u8
                            }
                            Edit::NoisePeriod(d) => {
                                sound.noise_period = Some(
                                    (sound
                                        .noise_period
                                        .unwrap_or([13, 8, 2, 3][(n.note.clamp(1, 4) - 1) as usize])
                                        as i16
                                        + d)
                                        .clamp(0, 15) as u8,
                                )
                            }
                            Edit::NoiseMode => sound.noise_short = !sound.noise_short,
                            _ => {}
                        }
                        n.sound = Some(sound);
                    }
                }
            }
        }
        Ok(())
    })?;
    Ok(result)
}
#[derive(Clone, Default)]
pub struct Clipboard {
    pub notes: Vec<(usize, Note)>,
    pub expansion: Expansion,
}
impl Clipboard {
    pub fn copy(score: &Score, part: usize, selection: &[(usize, usize)]) -> Self {
        let mut notes: Vec<_> = selection
            .iter()
            .filter_map(|&(c, i)| {
                score
                    .sections
                    .get(part)?
                    .tracks
                    .get(c)?
                    .notes
                    .get(i)
                    .map(|n| (c, n.clone()))
            })
            .collect();
        let start = notes.iter().map(|(_, n)| n.row).min().unwrap_or(0);
        for (_, n) in &mut notes {
            n.row -= start;
        }
        Self {
            notes,
            expansion: score.expansion,
        }
    }
    pub fn paste(&self, score: &mut Score, part: usize, row: u16) -> Result<Selection, String> {
        if self.notes.is_empty() {
            return Err("Clipboard is empty".into());
        }
        if self.expansion != score.expansion && self.notes.iter().any(|(c, _)| *c >= 5) {
            return Err("Clipboard expansion does not match this project".into());
        }
        let mut result = vec![];
        transaction(score, |s| {
            let sec = s.sections.get_mut(part).ok_or("Section out of range")?;
            for (c, n) in &self.notes {
                let mut n = n.clone();
                n.row = n.row.checked_add(row).ok_or("Position overflow")?;
                let t = sec.tracks.get_mut(*c).ok_or("Channel out of range")?;
                result.push((*c, t.notes.len()));
                t.notes.push(n);
            }
            Ok(())
        })?;
        Ok(result)
    }
}
