//! NES-aware PNG conversion: one four-color palette per 16x16 attribute block.
use crate::*;
use image::{ImageBuffer, Rgba};

#[derive(Debug, Serialize)]
pub struct ImportReport {
    pub width: u32,
    pub height: u32,
    pub unique_tiles: usize,
    pub mean_squared_error: f64,
    pub auto_palette: bool,
}

fn distance(a: [u8; 3], b: [u8; 3]) -> u32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (i32::from(*a) - i32::from(b)).pow(2) as u32)
        .sum()
}
fn nearest(rgb: [u8; 3], palette: &[u8]) -> (usize, u32) {
    palette
        .iter()
        .enumerate()
        .map(|(i, c)| (i, distance(rgb, NES_PALETTE_RGB[*c as usize])))
        .min_by_key(|p| p.1)
        .unwrap()
}

pub fn import_png(
    doc: &KitaqfcGraphicsDocument,
    path: &Path,
    auto_palette: bool,
) -> Result<(KitaqfcGraphicsDocument, ImportReport), String> {
    let mut reader = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(256);
    limits.max_image_height = Some(240);
    reader.limits(limits);
    let img = reader.decode().map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || w > 256 || h > 240 || w % 8 != 0 || h % 8 != 0 {
        return Err("PNG dimensions must be multiples of 8 within 8x8..256x240".into());
    }
    let bg = NES_PALETTE_RGB[doc.palette[0] as usize];
    let pixels: Vec<[u8; 3]> = img
        .pixels()
        .map(|p| {
            std::array::from_fn(|i| {
                ((u32::from(p[i]) * u32::from(p[3])
                    + u32::from(bg[i]) * (255 - u32::from(p[3]))
                    + 127)
                    / 255) as u8
            })
        })
        .collect();
    let mut d = doc.clone();
    let mut blocks = Vec::new();
    for by in (0..h as usize).step_by(16) {
        for bx in (0..w as usize).step_by(16) {
            let mut block = Vec::new();
            for y in by..(by + 16).min(h as usize) {
                for x in bx..(bx + 16).min(w as usize) {
                    block.push(pixels[y * w as usize + x]);
                }
            }
            blocks.push((bx, by, block));
        }
    }
    if auto_palette {
        let safe: Vec<u8> = (0..64).filter(|c| c % 16 < 13 || *c == 15).collect();
        let mut histogram = [0usize; 64];
        for p in &pixels {
            histogram[safe[nearest(*p, &safe).0] as usize] += 1;
        }
        let universal = (0..64).max_by_key(|i| histogram[*i]).unwrap() as u8;
        let mut candidates = Vec::<[u8; 4]>::new();
        for (_, _, block) in &blocks {
            let mut counts = [0usize; 64];
            for p in block {
                counts[safe[nearest(*p, &safe).0] as usize] += 1;
            }
            counts[universal as usize] = 0;
            let mut sorted = safe.clone();
            sorted.sort_by_key(|c| std::cmp::Reverse(counts[*c as usize]));
            let candidate = [universal, sorted[0], sorted[1], sorted[2]];
            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
        // Greedy error reduction includes uncommon highlights, not just frequency.
        let errors: Vec<Vec<u64>> = candidates
            .iter()
            .map(|pal| {
                blocks
                    .iter()
                    .map(|(_, _, b)| b.iter().map(|p| u64::from(nearest(*p, pal).1)).sum())
                    .collect()
            })
            .collect();
        let mut best = vec![u64::MAX; blocks.len()];
        let mut selected = Vec::new();
        for _ in 0..4 {
            let pick = (0..candidates.len())
                .min_by_key(|i| {
                    errors[*i]
                        .iter()
                        .zip(&best)
                        .map(|(a, b)| (*a).min(*b))
                        .sum::<u64>()
                })
                .unwrap();
            for (b, e) in best.iter_mut().zip(&errors[pick]) {
                *b = (*b).min(*e);
            }
            selected.push(candidates[pick]);
        }
        for (g, pal) in selected.iter().enumerate() {
            d.palette[g * 4..g * 4 + 4].copy_from_slice(pal);
        }
        // $3F10/$14/$18/$1C mirror the corresponding background entries.
        for i in (0..16).step_by(4) {
            d.palette[i + 16] = d.palette[i];
        }
    }
    let bank = d.background_pattern_table as usize * 4096;
    d.chr[bank..bank + 4096].fill(0);
    d.nametable.fill(0);
    d.palette_map.fill(0);
    let mut indexed = vec![0u8; (w * h) as usize];
    let mut total_error = 0u64;
    for (bx, by, block) in &blocks {
        let palettes: Vec<[u8; 4]> = (0..4)
            .map(|g| {
                [
                    d.palette[0],
                    d.palette[g * 4 + 1],
                    d.palette[g * 4 + 2],
                    d.palette[g * 4 + 3],
                ]
            })
            .collect();
        let group = (0..4)
            .min_by_key(|g| {
                block
                    .iter()
                    .map(|p| u64::from(nearest(*p, &palettes[*g]).1))
                    .sum::<u64>()
            })
            .unwrap();
        for y in *by..(*by + 16).min(h as usize) {
            for x in *bx..(*bx + 16).min(w as usize) {
                let (color, error) = nearest(pixels[y * w as usize + x], &palettes[group]);
                indexed[y * w as usize + x] = color as u8;
                total_error += u64::from(error);
                d.palette_map[(y / 8) * 32 + x / 8] = group as u8;
            }
        }
    }
    let mut unique = HashMap::<[u8; 16], u8>::new();
    // Tile zero stays blank so the unused part of smaller images stays blank.
    unique.insert([0; 16], 0);
    for ty in 0..h as usize / 8 {
        for tx in 0..w as usize / 8 {
            let mut tile = [0u8; 16];
            for y in 0..8 {
                for x in 0..8 {
                    let c = indexed[(ty * 8 + y) * w as usize + tx * 8 + x];
                    tile[y] |= (c & 1) << (7 - x);
                    tile[y + 8] |= ((c >> 1) & 1) << (7 - x);
                }
            }
            let id = if let Some(id) = unique.get(&tile) {
                *id
            } else {
                if unique.len() >= 256 {
                    return Err(
                        "PNG needs more than 255 nonblank tiles in one background pattern table"
                            .into(),
                    );
                }
                let id = unique.len() as u8;
                d.chr[bank + id as usize * 16..bank + id as usize * 16 + 16].copy_from_slice(&tile);
                unique.insert(tile, id);
                id
            };
            d.nametable[ty * 32 + tx] = id;
        }
    }
    d.rebuild_attributes();
    let report = ImportReport {
        width: w,
        height: h,
        unique_tiles: unique.len(),
        mean_squared_error: total_error as f64 / (w * h * 3) as f64,
        auto_palette,
    };
    Ok((d, report))
}

pub fn preview_png(
    d: &KitaqfcGraphicsDocument,
    path: &Path,
    view: &str,
    scale: u32,
) -> Result<(), String> {
    if !(1..=8).contains(&scale) {
        return Err("scale must be 1..8".into());
    }
    let (w, h, data) = match view {
        "map" => d.render_nametable_rgba(),
        "tiles" => d.render_chr_atlas_rgba(16, 12),
        "sprite" => d.render_metasprite_rgba(d.current_frame, 1),
        "tile" => d.render_tile_rgba_scaled(d.selected_tile, d.selected_palette_group, 1, false),
        _ => return Err("view must be map, tiles, sprite, or tile".into()),
    };
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w as u32, h as u32, data)
        .ok_or("invalid image buffer")?;
    image::imageops::resize(
        &img,
        w as u32 * scale,
        h as u32 * scale,
        image::imageops::FilterType::Nearest,
    )
    .save_with_format(path, image::ImageFormat::Png)
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn png_import_roundtrip_and_attributes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input.png");
        let img = ImageBuffer::from_fn(16, 16, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([236u8, 238, 236, 255])
            } else {
                Rgba([0u8, 0, 0, 255])
            }
        });
        img.save(&path).unwrap();
        let (d, r) = import_png(&KitaqfcGraphicsDocument::new_blank(), &path, true).unwrap();
        assert_eq!(r.mean_squared_error, 0.0);
        assert_eq!(r.unique_tiles, 2);
        let out = dir.path().join("out.png");
        preview_png(&d, &out, "map", 1).unwrap();
        let read = image::open(out).unwrap().to_rgba8();
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(read.get_pixel(x, y), img.get_pixel(x, y));
            }
        }
    }
}
