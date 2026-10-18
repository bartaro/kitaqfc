use super::*;

/// Monotone cubic interpolation: smooth sine curves without overshooting the
/// quantized sample endpoints. Only the view is interpolated, never saved data.
fn curve(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let delta = c - b;
    let slope = |x: f32, y: f32| {
        if x * y <= 0.0 {
            0.0
        } else {
            2.0 * x * y / (x + y)
        }
    };
    let m0 = slope(b - a, delta);
    let m1 = slope(delta, d - c);
    (2.0 * t * t * t - 3.0 * t * t + 1.0) * b
        + (t * t * t - 2.0 * t * t + t) * m0
        + (-2.0 * t * t * t + 3.0 * t * t) * c
        + (t * t * t - t * t) * m1
}
fn line(d: &mut DrawList, a: Point, b: Point, width: f32, color: Color) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = dx.hypot(dy);
    if len < 0.001 {
        return;
    }
    let nx = -dy / len * width * 0.5;
    let ny = dx / len * width * 0.5;
    let p = [
        Point::new(a.x + nx, a.y + ny),
        Point::new(b.x + nx, b.y + ny),
        Point::new(b.x - nx, b.y - ny),
        Point::new(a.x - nx, a.y - ny),
    ];
    d.fill_triangle_list(
        [0, 1, 2, 0, 2, 3]
            .map(|i| ColoredVertex {
                position: p[i],
                color,
            })
            .to_vec(),
    );
}
impl Studio {
    pub(super) fn draw_wave_graph(&self, d: &mut DrawList, wave: &[u8], max: f32) {
        let r = self.wave_rect();
        d.fill_rect(r, BG);
        if wave.is_empty() {
            return;
        }
        d.push_clip(r);
        for i in 0..=4 {
            let y = r.y + i as f32 * r.height / 4.0;
            d.fill_rect(
                Rect::new(r.x, y, r.width, 1.0),
                if i == 2 { LINE } else { LINE.with_alpha(0.5) },
            );
        }
        for i in 0..=8 {
            d.fill_rect(
                Rect::new(r.x + i as f32 * r.width / 8.0, r.y, 1.0, r.height),
                LINE.with_alpha(0.4),
            );
        }
        let point = |index: f32, value: f32| {
            Point::new(
                r.x + index * r.width / wave.len() as f32,
                r.bottom() - value / max * (r.height - 6.0) - 3.0,
            )
        };
        let get = |i: usize| f32::from(wave[i % wave.len()]);
        let mut prev = point(0.0, get(0));
        for i in 0..wave.len() {
            let steps = if self.wave_curve {
                (r.width / wave.len() as f32 / 3.0).ceil().max(4.0) as usize
            } else {
                1
            };
            for j in 1..=steps {
                let t = j as f32 / steps as f32;
                let v = if self.wave_curve {
                    curve(get(i + wave.len() - 1), get(i), get(i + 1), get(i + 2), t)
                } else {
                    get(i) * (1.0 - t) + get(i + 1) * t
                };
                let p = point(i as f32 + t, v);
                line(d, prev, p, 4.0, ACCENT.with_alpha(0.18));
                line(d, prev, p, 1.8, ACCENT);
                prev = p;
            }
        }
        if self.wave_editable() {
            for (i, v) in wave.iter().enumerate() {
                let p = point(i as f32, f32::from(*v));
                d.fill_rounded_rect(Rect::new(p.x - 2.5, p.y - 2.5, 5.0, 5.0), 2.5, TEXT);
            }
        }
        if r.contains(self.hover) {
            let i = (((self.hover.x - r.x) / r.width * wave.len() as f32).round() as usize)
                .min(wave.len() - 1);
            let p = point(i as f32, get(i));
            d.fill_rect(Rect::new(p.x, r.y, 1.0, r.height), TEXT.with_alpha(0.4));
            d.fill_rounded_rect(Rect::new(p.x - 4.0, p.y - 4.0, 8.0, 8.0), 4.0, ACCENT);
        }
        d.pop_clip();
        self.text(d, r.x + 6.0, r.y + 5.0, &format!("{max:.0}"), 0.65, MUTED);
        self.text(d, r.x + 6.0, r.bottom() - 15.0, "0", 0.65, MUTED);
        if r.contains(self.hover) {
            let i = (((self.hover.x - r.x) / r.width * wave.len() as f32).round() as usize)
                .min(wave.len() - 1);
            self.text(
                d,
                r.x + 60.0,
                r.y + 5.0,
                &format!("POINT {}  VALUE {}", i, wave[i]),
                0.7,
                ACCENT,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smooth_curve_hits_samples_without_overshoot() {
        for [a, b, c, d] in [
            [0., 8., 15., 8.],
            [15., 0., 15., 0.],
            [0., 0., 15., 15.],
            [15., 15., 0., 0.],
        ] {
            assert_eq!(curve(a, b, c, d, 0.), b);
            assert_eq!(curve(a, b, c, d, 1.), c);
            for i in 0..101 {
                let v = curve(a, b, c, d, i as f32 / 100.);
                assert!(v >= b.min(c) - 0.001 && v <= b.max(c) + 0.001);
            }
        }
    }
}
