//! NSF playback delegates CPU/APU/DPCM/N163 execution to KUROSAKI.
use crate::nsf::Imported;
use kurosaki_core::{
    nsf::{NsfFile, NsfPlayer},
    TraceConfig,
};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn render(nsf: &Imported, cancel: Option<&AtomicBool>) -> Result<Vec<i16>, String> {
    let cancelled = || {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            Err("Cancelled".to_string())
        } else {
            Ok(())
        }
    };
    cancelled()?;
    nsf.validate()?;
    let file = NsfFile::from_bytes(&nsf.data).map_err(|e| e.to_string())?;
    let mut player = NsfPlayer::new(&file, Some(nsf.song + 1), 44100, TraceConfig::none())
        .map_err(|e| e.to_string())?;
    let ms = nsf.duration_ms.unwrap_or(nsf.seconds * 1000);
    let count = ((ms as u64 * 44100 + 500) / 1000) as usize;
    if cancel.is_none() {
        return player.render_samples(count).map_err(|e| e.to_string());
    }
    let mut pcm = Vec::with_capacity(count);
    while pcm.len() < count {
        cancelled()?;
        pcm.extend(
            player
                .render_samples((count - pcm.len()).min(4410))
                .map_err(|e| e.to_string())?,
        );
    }
    cancelled()?;
    Ok(pcm)
}
