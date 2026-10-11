//! Strict, transactional editing commands shared by CLI and Python.
use crate::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Pixel {
        tile: usize,
        x: usize,
        y: usize,
        color: u8,
    },
    Fill {
        tile: usize,
        color: u8,
    },
    Checker {
        tile: usize,
        a: u8,
        b: u8,
    },
    Line {
        tile: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: u8,
    },
    Rect {
        tile: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: u8,
    },
    Mirror {
        tile: usize,
        horizontal: bool,
        vertical: bool,
    },
    Rotate {
        tile: usize,
    },
    Copy {
        source: usize,
        target: usize,
    },
    Palette {
        index: usize,
        value: u8,
    },
    MapRect {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        tile: u8,
        palette: u8,
    },
    Tables {
        background: u8,
        sprites: u8,
    },
    Sprite {
        frame: usize,
        index: usize,
        x: i8,
        y: i8,
        tile: u8,
        attr: u8,
    },
    RemoveSprite {
        frame: usize,
        index: usize,
    },
    CloneFrame {
        frame: usize,
    },
    ShiftMap {
        dx: i32,
        dy: i32,
    },
    Dedupe,
}

pub fn apply(doc: &mut KitaqfcGraphicsDocument, commands: &[Command]) -> Result<(), String> {
    if commands.len() > 100_000 {
        return Err("batch exceeds 100000 operations".into());
    }
    let mut next = doc.clone();
    for (i, command) in commands.iter().enumerate() {
        if !execute(&mut next, command) {
            return Err(format!(
                "operation {i}: invalid range or hardware constraint: {command:?}"
            ));
        }
    }
    *doc = next;
    Ok(())
}

fn execute(d: &mut KitaqfcGraphicsDocument, c: &Command) -> bool {
    match *c {
        Command::Pixel { tile, x, y, color } => color < 4 && d.set_tile_pixel(tile, x, y, color),
        Command::Fill { tile, color } => color < 4 && d.fill_tile(tile, color),
        Command::Checker { tile, a, b } => a < 4 && b < 4 && d.checker_tile(tile, a, b),
        Command::Line {
            tile,
            x0,
            y0,
            x1,
            y1,
            color,
        } => {
            [x0, y0, x1, y1].iter().all(|n| *n < 8)
                && color < 4
                && d.draw_line_tile(tile, x0 as i32, y0 as i32, x1 as i32, y1 as i32, color)
        }
        Command::Rect {
            tile,
            x0,
            y0,
            x1,
            y1,
            color,
        } => {
            [x0, y0, x1, y1].iter().all(|n| *n < 8)
                && color < 4
                && d.fill_tile_rect(tile, x0, y0, x1, y1, color)
        }
        Command::Mirror {
            tile,
            horizontal,
            vertical,
        } => d.mirror_tile(tile, horizontal, vertical),
        Command::Rotate { tile } => d.rotate_tile_cw(tile),
        Command::Copy { source, target } => d.copy_tile(source, target),
        Command::Palette { index, value } => value < 64 && d.set_palette_entry(index, value),
        Command::MapRect {
            x,
            y,
            width,
            height,
            tile,
            palette,
        } => {
            width <= 32
                && height <= 30
                && x < 32
                && y < 30
                && width <= 32 - x
                && height <= 30 - y
                && palette < 4
                && d.fill_nametable_rect(x, y, width, height, tile, palette)
        }
        Command::Tables {
            background,
            sprites,
        } => d.set_pattern_tables(background, sprites),
        Command::Sprite {
            frame,
            index,
            x,
            y,
            tile,
            attr,
        } => d.set_metasprite_quad(
            frame,
            index,
            MetaSpriteQuad::new(x as u8, y as u8, tile, attr),
        ),
        Command::RemoveSprite { frame, index } => {
            if let Some(f) = d.metasprites.get_mut(frame) {
                if index < f.quads.len() {
                    f.quads.remove(index);
                    return true;
                }
            }
            false
        }
        Command::CloneFrame { frame } => {
            if frame >= d.metasprites.len() || d.metasprites.len() >= 256 {
                return false;
            }
            d.metasprites.push(d.metasprites[frame].clone());
            d.current_frame = d.metasprites.len() - 1;
            true
        }
        Command::ShiftMap { dx, dy } => {
            // Keep 16x16 attribute blocks aligned when wrapping.
            if dx % 2 != 0 || dy % 2 != 0 || dx.abs_diff(0) > 32 || dy.abs_diff(0) > 30 {
                return false;
            }
            let before = d.clone();
            for y in 0..30 {
                for x in 0..32 {
                    let sx = (x as i32 - dx).rem_euclid(32) as usize;
                    let sy = (y as i32 - dy).rem_euclid(30) as usize;
                    d.nametable[y * 32 + x] = before.nametable[sy * 32 + sx];
                    d.palette_map[y * 32 + x] = before.palette_map[sy * 32 + sx];
                }
            }
            d.rebuild_attributes();
            true
        }
        Command::Dedupe => {
            d.dedupe_tiles();
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_batch_is_atomic_and_unknown_fields_rejected() {
        let mut d = KitaqfcGraphicsDocument::new_blank();
        let before = d.clone();
        assert!(apply(
            &mut d,
            &[
                Command::Fill { tile: 1, color: 2 },
                Command::Pixel {
                    tile: 1,
                    x: 8,
                    y: 0,
                    color: 1
                }
            ]
        )
        .is_err());
        assert_eq!(d, before);
        assert!(
            serde_json::from_str::<Command>(r#"{"op":"fill","tile":0,"color":1,"typo":3}"#)
                .is_err()
        );
    }
    #[test]
    fn palette_change_covers_the_hardware_quadrant() {
        let mut d = KitaqfcGraphicsDocument::new_blank();
        d.set_nametable_tile(1, 1, 2, 3);
        assert_eq!(
            [
                d.palette_map[0],
                d.palette_map[1],
                d.palette_map[32],
                d.palette_map[33]
            ],
            [3; 4]
        );
        assert_eq!(d.attributes[0] & 3, 3);
    }
    #[test]
    fn scanline_warning_and_reserved_sprite_byte() {
        let mut d = KitaqfcGraphicsDocument::new_blank();
        for i in 0..9 {
            assert!(d.set_metasprite_quad(0, i, MetaSpriteQuad::new((i * 8) as u8, 0, 1, 0)));
        }
        assert!(d
            .analysis_report()
            .warnings
            .iter()
            .any(|w| w.contains("scanline")));
        assert!(!d.set_metasprite_quad(0, 9, MetaSpriteQuad::new(255, 0, 0, 0)));
    }
}
