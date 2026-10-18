use super::*;
use fcseq_native::instruments::epsm_parameter;
impl Studio {
    pub(super) fn epsm_buttons(&self) -> Vec<Button> {
        let m = self.modal_rect();
        let mut b = vec![];
        for (a, x, label) in [
            (Action::EpsmPatch(-1), 28.0, "<"),
            (Action::EpsmPatch(1), 224.0, ">"),
            (Action::EpsmOperator(-1), 442.0, "<"),
            (Action::EpsmOperator(1), 638.0, ">"),
        ] {
            b.push(Button::new(a, m.x + x, m.y + 108.0, 32.0, label, None));
        }
        for i in 0..11 {
            let index = self.epsm_operator * 11 + i;
            let y = m.y + 154.0 + i as f32 * 34.0;
            b.push(Button::new(
                Action::EpsmParam(index, -1),
                m.x + 28.0,
                y,
                28.0,
                "-",
                None,
            ));
            b.push(Button::new(
                Action::EpsmParam(index, 1),
                m.x + 394.0,
                y,
                28.0,
                "+",
                None,
            ));
        }
        for i in 0..4 {
            let index = 44 + i;
            let y = m.y + 154.0 + i as f32 * 44.0;
            b.push(Button::new(
                Action::EpsmParam(index, -1),
                m.x + 442.0,
                y,
                28.0,
                "-",
                None,
            ));
            b.push(Button::new(
                Action::EpsmParam(index, 1),
                m.x + 808.0,
                y,
                28.0,
                "+",
                None,
            ));
        }
        for (id, y) in [(4, 354.0), (0, 398.0), (12, 442.0)] {
            b.push(Button::new(
                Action::Timbre(id, -1),
                m.x + 442.0,
                m.y + y,
                28.0,
                "-",
                None,
            ));
            b.push(Button::new(
                Action::Timbre(id, 1),
                m.x + 808.0,
                m.y + y,
                28.0,
                "+",
                None,
            ));
        }
        b
    }
    pub(super) fn draw_epsm(&self, d: &mut DrawList) {
        let m = self.modal_rect();
        self.text(
            d,
            m.x + 74.0,
            m.y + 118.0,
            &format!("PATCH {}", self.epsm_slot),
            0.85,
            ACCENT,
        );
        self.text(
            d,
            m.x + 488.0,
            m.y + 118.0,
            &format!("OPERATOR {}", self.epsm_operator + 1),
            0.85,
            ACCENT,
        );
        for (i, index) in (self.epsm_operator * 11..self.epsm_operator * 11 + 11).enumerate() {
            let (name, byte, shift, max) = epsm_parameter(index).unwrap();
            let value = (self.score.epsm_patches[self.epsm_slot][byte] >> shift) & max;
            self.text(
                d,
                m.x + 68.0,
                m.y + 164.0 + i as f32 * 34.0,
                &format!("{name}  {value}"),
                0.73,
                TEXT,
            );
        }
        for i in 0..4 {
            let (name, byte, shift, max) = epsm_parameter(44 + i).unwrap();
            let value = (self.score.epsm_patches[self.epsm_slot][byte] >> shift) & max;
            self.text(
                d,
                m.x + 482.0,
                m.y + 164.0 + i as f32 * 44.0,
                &format!("{name}  {value}"),
                0.73,
                TEXT,
            );
        }
        let sound = self.instrument_sound();
        for (text, y) in [
            (format!("NOTE PATCH  {}", sound.instrument), 364.0),
            (format!("NOTE VOLUME  {}", sound.volume), 408.0),
            (
                format!("PAN {}: 0 OFF / 1 R / 2 L / 3 BOTH", sound.pan),
                452.0,
            ),
        ] {
            self.text(d, m.x + 482.0, m.y + y, &text, 0.65, TEXT);
        }
        self.text(
            d,
            m.x + 442.0,
            m.y + 500.0,
            "Patch edits affect every note using that patch.",
            0.67,
            MUTED,
        );
        self.text(
            d,
            m.x + 442.0,
            m.y + 528.0,
            "FM operators use sine waves; no editable wave RAM.",
            0.65,
            MUTED,
        );
    }
}
