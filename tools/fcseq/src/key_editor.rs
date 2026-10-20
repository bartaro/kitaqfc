use super::*;
pub(super) const KEYBOARD_WIDTH: f32 = 88.0;
impl Studio {
    pub(super) fn clamp_key_range(&mut self) {
        let (lo, hi) = self.score.expansion.kind(self.track).bounds();
        self.key_rows = self.key_rows.min(hi - lo + 1).max(1);
        self.key_low = self.key_low.clamp(lo, hi + 1 - self.key_rows);
    }
    pub(super) fn open_key_track(&mut self, ch: usize) {
        self.key_editor = true;
        self.track = ch;
        self.first_track = ch;
        self.selection.retain(|(track, _)| *track == ch);
        self.key_rows = 24;
        let center = self
            .selection
            .first()
            .map(|(_, i)| self.score.sections[self.part].tracks[ch].notes[*i].note)
            .or_else(|| {
                self.score.sections[self.part].tracks[ch]
                    .notes
                    .first()
                    .map(|n| n.note)
            })
            .unwrap_or(60);
        self.key_low = center.saturating_sub(12);
        self.clamp_key_range();
        self.status=format!("{} — drag notes to move, drag their right edge to resize; wheel scrolls pitch; E returns to all tracks",self.score.expansion.channel_name(ch));
    }
    pub(super) fn key_height(&self) -> f32 {
        self.grid().height / self.key_rows as f32
    }
    pub(super) fn key_pitch(&self, y: f32) -> u8 {
        (self.key_low as f32 + ((self.grid().bottom() - y) / self.key_height()).floor()).clamp(
            self.key_low as f32,
            (self.key_low + self.key_rows - 1) as f32,
        ) as u8
    }
    pub(super) fn key_note_rect(&self, n: &Note) -> Rect {
        let g = self.grid();
        let h = self.key_height();
        Rect::new(
            g.x + (n.row as f32 - self.page as f32) * self.step() + 1.0,
            g.bottom() - (n.note as f32 - self.key_low as f32 + 1.0) * h + 1.0,
            (n.length as f32 * self.step() - 2.0).max(5.0),
            (h - 2.0).max(3.0),
        )
    }
    pub(super) fn keyboard_rect(&self) -> Rect {
        let g = self.grid();
        Rect::new(16.0, g.y, KEYBOARD_WIDTH, g.height)
    }
    pub(super) fn draw_keyboard(&self, d: &mut DrawList) {
        let g = self.grid();
        let h = self.key_height();
        let keys = self.keyboard_rect();
        let black_width = keys.width * 0.66;
        self.text(d, keys.x + 28.0, 152.0, "KEYS", 0.72, ACCENT);
        for pitch in self.key_low..self.key_low + self.key_rows {
            let y = g.bottom() - (pitch - self.key_low + 1) as f32 * h;
            let black = matches!(pitch % 12, 1 | 3 | 6 | 8 | 10);
            let hit =
                self.keyboard_rect().contains(self.hover) && self.key_pitch(self.hover.y) == pitch;
            d.fill_rect(
                Rect::new(g.x, y, g.width, h),
                if black {
                    Color::rgb8(17, 25, 36)
                } else {
                    Color::rgb8(25, 36, 48)
                },
            );
            d.fill_rect(
                Rect::new(g.x, y + h - 1.0, g.width, 1.0),
                if pitch % 12 == 0 {
                    Color::rgb8(68, 86, 105)
                } else {
                    LINE.with_alpha(0.35)
                },
            );
            d.fill_rect(
                Rect::new(keys.x, y, keys.width, h - 1.0),
                Color::rgb8(214, 223, 228),
            );
            if black {
                d.fill_rect(Rect::new(keys.x, y, black_width, h - 1.0), Color::rgb8(35, 42, 51));
            }
            if hit {
                d.fill_rect(
                    Rect::new(keys.x, y, if black { black_width } else { keys.width }, h - 1.0),
                    ACCENT.with_alpha(0.65),
                );
            }
            if h >= 14.0 {
                let text = self.score.expansion.pitch_name(pitch, self.track);
                let available = if black { black_width } else { keys.width } - 12.0;
                let scale = 0.58 * (available / measure_text_width(&text, 0.58).max(1.0)).min(1.0);
                let x = if black || text.chars().count() > 4 {
                    keys.x + 6.0
                } else {
                    keys.right() - measure_text_width(&text, scale) - 6.0
                };
                d.push_clip(self.keyboard_rect());
                self.text(
                    d,
                    x,
                    y + (h - 10.0) / 2.0,
                    &text,
                    scale,
                    if black { TEXT } else { BG },
                );
                d.pop_clip();
            }
        }
        self.text(
            d,
            g.x,
            self.grid().bottom() + 36.0,
            &self.score.expansion.channel_name(self.track),
            0.65,
            ACCENT,
        );
    }
    pub(super) fn audition_key(&mut self, pitch: u8) {
        let mut score = self.score.clone();
        let ch = self.track;
        let mut sound = self
            .selection
            .first()
            .map(|&(c, i)| score.sections[self.part].tracks[c].notes[i].sound(c))
            .unwrap_or_default();
        if matches!(score.expansion.kind(ch), Voice::Dpcm | Voice::Pcm) {
            sound.instrument = self.sample as u8;
        }
        if score.expansion.kind(ch) == Voice::N163 {
            sound.instrument = self.wave_slot as u8;
        }
        score.sections = vec![Section::with_channels(
            "Keyboard",
            score.expansion.channels(),
        )];
        score.sections[0].rows = 16;
        let mut note = Note::new(0, 4, pitch);
        note.sound = Some(sound);
        score.sections[0].tracks[ch].notes.push(note);
        self.launch_render(score, 0, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drag(a: &mut Studio, p: Point, q: Point) {
        a.handle_event(UiEvent::PointerDown {
            position: p,
            button: PointerButton::Primary,
        });
        a.handle_event(UiEvent::PointerMove { position: q });
        a.handle_event(UiEvent::PointerUp {
            position: q,
            button: PointerButton::Primary,
        });
    }
    #[test]
    fn single_track_keys_drag_resize_undo_and_track_switch() {
        let mut a = Studio::new(Size::new(1120.0, 780.0), None);
        a.score.sections[0].tracks[0]
            .notes
            .push(Note::new(4, 4, 60));
        a.open_key_track(0);
        a.snap = 1;
        let r = a.note_rect(0, &a.score.sections[0].tracks[0].notes[0]);
        let p = Point::new(r.x + 3.0, r.y + r.height / 2.0);
        let q = Point::new(p.x + a.step() * 2.0, p.y - a.key_height() * 3.0);
        drag(&mut a, p, q);
        let n = &a.score.sections[0].tracks[0].notes[0];
        assert_eq!((n.row, n.note, n.length), (6, 63, 4));
        let r = a.note_rect(0, n);
        let p = Point::new(r.right() - 1.0, r.y + 3.0);
        let q = Point::new(p.x + a.step() * 3.0, p.y);
        drag(&mut a, p, q);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].length, 7);
        a.undo(false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].length, 4);
        a.undo(false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].note, 60);
        a.apply(Action::KeyTrack(1));
        assert_eq!(a.track, 1);
        assert!(a.selection.is_empty());
        assert!(a.keyboard_rect().right() <= a.grid().x);
        // The space released by the shorter keyboard is a usable note grid.
        let p = Point::new(140.0, a.grid().y + a.key_height() * 4.5);
        assert!(a.grid().contains(p));
        assert!(!a.keyboard_rect().contains(p));
        let pitch = a.key_pitch(p.y);
        a.handle_event(UiEvent::PointerDoubleClick {
            position: p,
            button: PointerButton::Primary,
        });
        assert_eq!(a.score.sections[0].tracks[1].notes.len(), 1);
        assert_eq!(a.score.sections[0].tracks[1].notes[0].note, pitch);
        for b in a.buttons().into_iter().filter(|b| matches!(b.action, Action::KeyZoom(_) | Action::KeyTrack(_))) {
            assert!(b.rect.x >= a.keyboard_rect().x);
            assert!(b.rect.right() <= a.keyboard_rect().right());
        }
        for b in a.buttons().into_iter().filter(|b| matches!(b.action, Action::Mute(_) | Action::Solo(_))) {
            assert!(b.rect.bottom() <= a.grid().y);
        }
        a.draw_list();
        a.apply(Action::KeyEditor);
        assert!(!a.key_editor);
    }
    #[test]
    fn new_chip_controls_and_keyboard_stay_reachable() {
        for (chip, ch) in [
            (Expansion::Mmc5, 5),
            (Expansion::Sunsoft5b, 5),
            (Expansion::Epsm, 8),
            (Expansion::Epsm, 19),
            (Expansion::Vt02, 8),
        ] {
            let mut a = Studio::new(Size::new(1120.0, 780.0), None);
            a.score.set_expansion(chip).unwrap();
            let pitch = if ch == 8 && chip == Expansion::Vt02 {
                2
            } else {
                60
            };
            a.score.sections[0].tracks[ch]
                .notes
                .push(Note::new(0, 4, pitch));
            a.open_key_track(ch);
            a.selection = vec![(ch, 0)];
            a.apply(Action::Wave);
            for b in a.modal_buttons() {
                assert!(
                    b.rect.x >= 0.0
                        && b.rect.y >= 0.0
                        && b.rect.right() <= 1120.0
                        && b.rect.bottom() <= 780.0,
                    "{chip:?} {:?}",
                    b.rect
                );
            }
            if chip == Expansion::Epsm && ch == 8 {
                let old = a.score.clone();
                a.apply(Action::EpsmParam(0, 1));
                assert_ne!(a.score, old);
                a.undo(false);
                assert_eq!(a.score, old);
            }
            a.draw_list();
            a.score.validate().unwrap();
        }
    }
}
