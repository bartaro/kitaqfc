use fcseq_native::{model::Score, services};
use rodio::Source;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, RwLock,
    },
    time::Duration,
};

const BLOCK: usize = 64;
const FADE: usize = 128;

#[derive(Clone)]
struct Buffer {
    samples: Arc<RwLock<Arc<[i16]>>>,
    silent: Arc<AtomicBool>,
}
impl Buffer {
    fn replace(&self, samples: Arc<[i16]>) -> Result<(), String> {
        let mut current = self.samples.write().unwrap();
        if samples.len() != current.len() {
            return Err("Track playback length changed".into());
        }
        *current = samples;
        self.silent.store(false, Ordering::Release);
        Ok(())
    }
}

/// A fixed-format source whose mix can change without replacing its audio sink,
/// resetting its sample cursor, or restarting the loop.
pub(super) struct MonitorSource {
    buffer: Buffer,
    samples: Arc<[i16]>,
    previous: Option<Arc<[i16]>>,
    fade: usize,
    position: usize,
    looped: bool,
}
impl Iterator for MonitorSource {
    type Item = i16;
    fn next(&mut self) -> Option<i16> {
        if self.samples.is_empty() {
            return None;
        }
        if self.position == self.samples.len() {
            if !self.looped {
                return None;
            }
            self.position = 0;
        }
        if self.position % BLOCK == 0 {
            let samples = self.buffer.samples.read().unwrap().clone();
            if !Arc::ptr_eq(&samples, &self.samples) {
                self.previous = Some(std::mem::replace(&mut self.samples, samples));
                self.fade = 0;
            }
        }
        let mut value = self.samples[self.position] as i32;
        if let Some(previous) = &self.previous {
            self.fade += 1;
            value = (previous[self.position] as i32 * (FADE - self.fade) as i32
                + value * self.fade as i32)
                / FADE as i32;
            if self.fade == FADE {
                self.previous = None;
            }
        }
        self.position += 1;
        // Silence still consumes time. Unmuting must not resume an old note.
        Some(if self.buffer.silent.load(Ordering::Acquire) {
            0
        } else {
            value as i16
        })
    }
}
impl Source for MonitorSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        44100
    }
    fn total_duration(&self) -> Option<Duration> {
        (!self.looped).then(|| Duration::from_secs_f64(self.samples.len() as f64 / 44100.0))
    }
}

struct Pending {
    mask: u32,
    receiver: Receiver<Result<Vec<i16>, String>>,
    cancel: Arc<AtomicBool>,
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

pub(super) struct Monitor {
    pub score: Score,
    pub part: usize,
    pub source_part: usize,
    pub looped: bool,
    buffer: Option<Buffer>,
    cache: BTreeMap<u32, Arc<[i16]>>,
    pending: Option<Pending>,
}
impl Monitor {
    pub fn new(score: Score, part: usize, source_part: usize, looped: bool) -> Self {
        Self {
            score,
            part,
            source_part,
            looped,
            buffer: None,
            cache: BTreeMap::new(),
            pending: None,
        }
    }
    fn mask(&self) -> u32 {
        let section = &self.score.sections[self.part];
        (0..section.tracks.len()).fold(0, |bits, ch| bits | ((section.audible(ch) as u32) << ch))
    }
    pub fn sync_flags(&mut self, score: &Score) {
        for (target, source) in self.score.sections[self.part]
            .tracks
            .iter_mut()
            .zip(&score.sections[self.source_part].tracks)
        {
            target.mute = source.mute;
            target.solo = source.solo;
        }
    }
    pub fn is_ready(&self) -> bool {
        self.buffer.is_some()
    }
    pub fn duration_seconds(&self) -> Option<f64> {
        self.buffer
            .as_ref()
            .map(|buffer| buffer.samples.read().unwrap().len() as f64 / 44100.0)
    }
    pub fn attach(&mut self, samples: Vec<i16>) -> MonitorSource {
        let samples: Arc<[i16]> = samples.into();
        self.cache.insert(self.mask(), samples.clone());
        let buffer = Buffer {
            samples: Arc::new(RwLock::new(samples.clone())),
            silent: Arc::new(AtomicBool::new(self.mask() == 0)),
        };
        self.buffer = Some(buffer.clone());
        MonitorSource {
            buffer,
            samples,
            previous: None,
            fade: 0,
            position: 0,
            looped: self.looped,
        }
    }
    /// Cached mixes are immediate; other mixes render independently of the
    /// playing source. Dropping the old receiver rejects stale completion.
    pub fn update(&mut self) -> bool {
        self.pending = None;
        let mask = self.mask();
        let Some(buffer) = &self.buffer else {
            return false;
        };
        if mask == 0 {
            buffer.silent.store(true, Ordering::Release);
            return false;
        }
        if let Some(samples) = self.cache.get(&mask) {
            buffer.replace(samples.clone()).unwrap();
            return false;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let worker = cancel.clone();
        let (tx, receiver) = mpsc::channel();
        let score = self.score.clone();
        let part = self.part;
        std::thread::spawn(move || {
            let _ = tx.send(services::render(&score, part, Some(&worker)));
        });
        self.pending = Some(Pending {
            mask,
            receiver,
            cancel,
        });
        true
    }
    pub fn poll(&mut self) -> Option<Result<(), String>> {
        let pending = self.pending.as_ref()?;
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err("Track playback worker stopped".into()),
        };
        let mask = self.pending.take().unwrap().mask;
        Some(result.and_then(|samples| {
            let samples: Arc<[i16]> = samples.into();
            self.buffer.as_ref().unwrap().replace(samples.clone())?;
            self.cache.insert(mask, samples);
            if self.cache.len() > 8 {
                self.cache.pop_first();
            }
            Ok(())
        }))
    }
}

pub(super) fn same_music(a: &Score, b: &Score) -> bool {
    let mut compare = a.clone();
    for (left, right) in compare.sections.iter_mut().zip(&b.sections) {
        for (left, right) in left.tracks.iter_mut().zip(&right.tracks) {
            left.mute = right.mute;
            left.solo = right.solo;
        }
    }
    compare == *b
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcseq_native::model::{Expansion, Note};
    fn complete(monitor: &mut Monitor) {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = monitor.poll() {
                result.unwrap();
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Monitor render timed out"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    #[test]
    fn live_mix_swap_silence_and_loop_keep_the_sample_cursor() {
        let mut monitor = Monitor::new(Score::empty(), 0, 0, true);
        let mut source = monitor.attach(vec![1000; 512]);
        for _ in 0..64 {
            assert_eq!(source.next(), Some(1000));
        }
        let buffer = monitor.buffer.as_ref().unwrap();
        buffer.replace(Arc::from(vec![-1000; 512])).unwrap();
        let fade: Vec<_> = source.by_ref().take(128).collect();
        assert!(fade.windows(2).all(|s| s[0] >= s[1]));
        assert_eq!(fade.last(), Some(&-1000));
        assert_eq!(source.position, 192);
        buffer.silent.store(true, Ordering::Release);
        assert!(source.by_ref().take(400).all(|s| s == 0));
        assert_eq!(source.position, 80);
        buffer.silent.store(false, Ordering::Release);
        assert_eq!(source.next(), Some(-1000));
        assert_eq!(source.position, 81);
    }
    #[test]
    fn monitor_mix_matches_renderer_for_every_chip_and_rejects_stale_updates() {
        for chip in [
            Expansion::None,
            Expansion::Vrc6,
            Expansion::Vrc7,
            Expansion::Fds,
            Expansion::N163,
            Expansion::Mmc5,
            Expansion::Sunsoft5b,
            Expansion::Epsm,
            Expansion::Vt02,
        ] {
            let mut score = Score::empty();
            score.set_expansion(chip).unwrap();
            score.sections[0].rows = 16;
            score.sections[0].tracks[0].notes.push(Note::new(0, 16, 69));
            let ch = if chip == Expansion::None { 1 } else { 5 };
            score.sections[0].tracks[ch]
                .notes
                .push(Note::new(0, 16, 60));
            let original = services::render(&score, 0, None).unwrap();
            let mut monitor = Monitor::new(score.clone(), 0, 0, false);
            let mut source = monitor.attach(original.clone());
            source.by_ref().take(256).for_each(drop);
            score.sections[0].tracks[ch].solo = true;
            monitor.sync_flags(&score);
            assert!(monitor.update());
            complete(&mut monitor);
            assert_eq!(
                &**monitor.buffer.as_ref().unwrap().samples.read().unwrap(),
                &*services::render(&score, 0, None).unwrap()
            );
            assert_eq!(source.position, 256);
            score.sections[0].tracks[0].solo = true;
            monitor.sync_flags(&score);
            assert!(monitor.update());
            complete(&mut monitor);
            assert_eq!(
                &**monitor.buffer.as_ref().unwrap().samples.read().unwrap(),
                &*original
            );
            score.sections[0].tracks[0].solo = false;
            monitor.sync_flags(&score);
            assert!(!monitor.update());
            // Start another mix, then restore a cached mix before it finishes.
            score.sections[0].tracks[ch].solo = false;
            score.sections[0].tracks[ch].mute = true;
            monitor.sync_flags(&score);
            assert!(monitor.update());
            let cancel = monitor.pending.as_ref().unwrap().cancel.clone();
            score.sections[0].tracks[ch].mute = false;
            monitor.sync_flags(&score);
            assert!(!monitor.update());
            assert!(cancel.load(Ordering::Relaxed));
            assert!(monitor.poll().is_none());
            assert_eq!(
                &**monitor.buffer.as_ref().unwrap().samples.read().unwrap(),
                &*original
            );
            // Mute wins over solo, including a soloed but muted voice.
            score.sections[0].tracks[ch].solo = true;
            score.sections[0].tracks[ch].mute = true;
            monitor.sync_flags(&score);
            assert!(!monitor.update());
            assert_eq!(source.next(), Some(0));
        }
    }
    #[test]
    fn sample_tracks_use_the_same_live_mute_path() {
        use fcseq_native::{model::Sound, samples::Sample};
        for (chip, ch, pitch) in [
            (Expansion::None, 4, 12),
            (Expansion::Mmc5, 7, 69),
            (Expansion::Vt02, 9, 69),
        ] {
            let mut score = Score::empty();
            score.set_expansion(chip).unwrap();
            score.sections[0].rows = 16;
            score.samples.push(Sample {
                name: "Monitor fixture".into(),
                pcm: (0..4410)
                    .map(|i| ((i as f64 * 0.1).sin() * 12000.0) as i16)
                    .collect(),
                sample_rate: 44100,
                start: 0,
                end: 4410,
                gain: 1.0,
                normalize: false,
                reverse: false,
                rate: 12,
                initial_level: 64,
            });
            score.sections[0].tracks[0].notes.push(Note::new(0, 16, 60));
            let mut note = Note::new(0, 16, pitch);
            note.sound = Some(Sound {
                instrument: 0,
                ..Sound::default()
            });
            score.sections[0].tracks[ch].notes.push(note);
            let original = services::render(&score, 0, None).unwrap();
            let mut monitor = Monitor::new(score.clone(), 0, 0, false);
            let mut source = monitor.attach(original.clone());
            source.by_ref().take(256).for_each(drop);
            score.sections[0].tracks[ch].mute = true;
            monitor.sync_flags(&score);
            assert!(monitor.update());
            complete(&mut monitor);
            let expected = services::render(&score, 0, None).unwrap();
            assert_ne!(original, expected);
            assert_eq!(
                &**monitor.buffer.as_ref().unwrap().samples.read().unwrap(),
                &*expected
            );
            assert_eq!(source.position, 256);
        }
    }
}
