use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[cfg(any(feature = "gui", test))]
use std::env;

use serde::{Deserialize, Serialize};

pub const CHR_ROM_SIZE: usize = 8192;
pub const TILE_BYTES: usize = 16;
pub const TILE_COUNT: usize = CHR_ROM_SIZE / TILE_BYTES;
pub const TILE_SIDE: usize = 8;
pub const NAMETABLE_W: usize = 32;
pub const NAMETABLE_H: usize = 30;
pub const NAMETABLE_SIZE: usize = NAMETABLE_W * NAMETABLE_H;
pub const ATTRIBUTE_SIZE: usize = 64;
pub const PALETTE_SIZE: usize = 32;

const NES_PALETTE_RGB: [[u8; 3]; 64] = [
    [84, 84, 84],
    [0, 30, 116],
    [8, 16, 144],
    [48, 0, 136],
    [68, 0, 100],
    [92, 0, 48],
    [84, 4, 0],
    [60, 24, 0],
    [32, 42, 0],
    [8, 58, 0],
    [0, 64, 0],
    [0, 60, 0],
    [0, 50, 60],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [152, 150, 152],
    [8, 76, 196],
    [48, 50, 236],
    [92, 30, 228],
    [136, 20, 176],
    [160, 20, 100],
    [152, 34, 32],
    [120, 60, 0],
    [84, 90, 0],
    [40, 114, 0],
    [8, 124, 0],
    [0, 118, 40],
    [0, 102, 120],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [236, 238, 236],
    [76, 154, 236],
    [120, 124, 236],
    [176, 98, 236],
    [228, 84, 236],
    [236, 88, 180],
    [236, 106, 100],
    [212, 136, 32],
    [160, 170, 0],
    [116, 196, 0],
    [76, 208, 32],
    [56, 204, 108],
    [56, 180, 204],
    [60, 60, 60],
    [0, 0, 0],
    [0, 0, 0],
    [236, 238, 236],
    [168, 204, 236],
    [188, 188, 236],
    [212, 178, 236],
    [236, 174, 236],
    [236, 174, 212],
    [236, 180, 176],
    [228, 196, 144],
    [204, 210, 120],
    [180, 222, 120],
    [168, 226, 144],
    [152, 226, 180],
    [160, 214, 228],
    [160, 162, 160],
    [0, 0, 0],
    [0, 0, 0],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetaSpriteQuad {
    pub dx: u8,
    pub dy: u8,
    pub tile: u8,
    pub attr: u8,
}

impl MetaSpriteQuad {
    pub const fn new(dx: u8, dy: u8, tile: u8, attr: u8) -> Self {
        Self { dx, dy, tile, attr }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetaSpriteFrame {
    pub quads: Vec<MetaSpriteQuad>,
}

impl MetaSpriteFrame {
    pub fn new(quads: Vec<MetaSpriteQuad>) -> Self {
        Self { quads }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AnalysisReport {
    pub unique_tiles: usize,
    pub duplicate_tiles: usize,
    pub empty_tiles: usize,
    pub nametable_used_tiles: usize,
    pub metasprite_max_quads: usize,
    pub chr_bytes: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportReport {
    pub chr_path: PathBuf,
    pub palette_path: PathBuf,
    pub nametable_path: PathBuf,
    pub attribute_path: PathBuf,
    pub metasprite_path: PathBuf,
    pub manifest_path: PathBuf,
    pub c_path: PathBuf,
    pub h_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KitaqfcGraphicsDocument {
    chr: Vec<u8>,
    palette: [u8; PALETTE_SIZE],
    nametable: Vec<u8>,
    palette_map: Vec<u8>,
    attributes: Vec<u8>,
    metasprites: Vec<MetaSpriteFrame>,
    selected_tile: usize,
    selected_color: u8,
    selected_palette_group: u8,
    current_frame: usize,
    #[serde(default)]
    background_pattern_table: u8,
    #[serde(default)]
    sprite_pattern_table: u8,
}

impl Default for KitaqfcGraphicsDocument {
    fn default() -> Self {
        Self::new_demo()
    }
}

impl KitaqfcGraphicsDocument {
    pub fn new_blank() -> Self {
        let mut doc = Self::new_demo();
        doc.chr.fill(0);
        doc.nametable.fill(0);
        doc.palette_map.fill(0);
        doc.attributes.fill(0);
        doc.metasprites = vec![MetaSpriteFrame::new(Vec::new())];
        doc.selected_tile = 0;
        doc
    }

    /// View selections are not edits and must not trigger a discard warning.
    pub fn same_content(&self, other: &Self) -> bool {
        self.chr == other.chr
            && self.palette == other.palette
            && self.nametable == other.nametable
            && self.attributes == other.attributes
            && self.metasprites == other.metasprites
            && self.background_pattern_table == other.background_pattern_table
            && self.sprite_pattern_table == other.sprite_pattern_table
    }

    pub fn new_demo() -> Self {
        let mut doc = Self {
            chr: vec![0; CHR_ROM_SIZE],
            palette: [
                0x0F, 0x00, 0x10, 0x30, 0x0F, 0x06, 0x16, 0x26, 0x0F, 0x09, 0x19, 0x29, 0x0F, 0x02,
                0x12, 0x22, 0x0F, 0x11, 0x21, 0x31, 0x0F, 0x05, 0x15, 0x25, 0x0F, 0x08, 0x18, 0x28,
                0x0F, 0x0C, 0x1C, 0x2C,
            ],
            nametable: vec![0; NAMETABLE_SIZE],
            palette_map: vec![0; NAMETABLE_SIZE],
            attributes: vec![0; ATTRIBUTE_SIZE],
            metasprites: vec![MetaSpriteFrame::new(vec![
                MetaSpriteQuad::new(0, 0, 128, 0),
                MetaSpriteQuad::new(8, 0, 129, 0),
                MetaSpriteQuad::new(0, 8, 130, 0),
                MetaSpriteQuad::new(8, 8, 131, 0),
            ])],
            selected_tile: 1,
            selected_color: 1,
            selected_palette_group: 0,
            current_frame: 0,
            background_pattern_table: 0,
            sprite_pattern_table: 0,
        };

        doc.fill_tile(0, 0);
        doc.checker_tile(1, 1, 2);
        doc.outline_tile(1, 3);
        doc.fill_tile(2, 0);
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                if x == y || x + y == 7 {
                    doc.set_tile_pixel(2, x, y, 2);
                }
            }
        }
        doc.fill_tile(3, 0);
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                if x % 2 == 0 || y % 2 == 0 {
                    doc.set_tile_pixel(3, x, y, ((x + y) % 3 + 1) as u8);
                }
            }
        }
        for tile in 128..132 {
            doc.fill_tile(tile, 0);
            doc.outline_tile(tile, ((tile - 127) % 3 + 1) as u8);
            for y in 2..6 {
                for x in 2..6 {
                    if (x + y + tile) % 2 == 0 {
                        doc.set_tile_pixel(tile, x, y, ((tile - 126) % 3 + 1) as u8);
                    }
                }
            }
        }

        for y in 0..NAMETABLE_H {
            for x in 0..NAMETABLE_W {
                let index = y * NAMETABLE_W + x;
                doc.nametable[index] =
                    if x == 0 || y == 0 || x == NAMETABLE_W - 1 || y == NAMETABLE_H - 1 {
                        1
                    } else if (x / 4 + y / 4) % 2 == 0 {
                        2
                    } else {
                        3
                    };
                doc.palette_map[index] = ((x / 8 + y / 8) % 4) as u8;
            }
        }
        doc.rebuild_attributes();
        doc
    }

    pub fn chr(&self) -> &[u8] {
        &self.chr
    }

    pub fn palette(&self) -> &[u8; PALETTE_SIZE] {
        &self.palette
    }

    pub fn nametable(&self) -> &[u8] {
        &self.nametable
    }

    pub fn attributes(&self) -> &[u8] {
        &self.attributes
    }

    pub fn metasprites(&self) -> &[MetaSpriteFrame] {
        &self.metasprites
    }

    pub fn selected_tile(&self) -> usize {
        self.selected_tile
    }

    pub fn selected_color(&self) -> u8 {
        self.selected_color
    }

    pub fn selected_palette_group(&self) -> u8 {
        self.selected_palette_group
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn set_selected_tile(&mut self, tile: usize) {
        self.selected_tile = tile.min(TILE_COUNT - 1);
    }

    pub fn set_selected_color(&mut self, color: u8) {
        self.selected_color = color.min(3);
    }

    pub fn set_selected_palette_group(&mut self, group: u8) {
        self.selected_palette_group = group.min(7);
    }

    pub fn load_chr_bytes(&mut self, bytes: &[u8]) {
        self.chr.fill(0);
        let len = bytes.len().min(CHR_ROM_SIZE);
        self.chr[..len].copy_from_slice(&bytes[..len]);
    }

    /// Strict import for editor/Python callers: oversized banks must not lose data.
    pub fn try_load_chr_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if bytes.is_empty() || bytes.len() > CHR_ROM_SIZE || bytes.len() % TILE_BYTES != 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                "CHR must contain whole 16-byte tiles and fit one 8 KiB bank; open each mapper bank separately"));
        }
        self.load_chr_bytes(bytes);
        Ok(())
    }

    pub fn set_pattern_tables(&mut self, background: u8, sprites: u8) -> bool {
        if background > 1 || sprites > 1 {
            return false;
        }
        self.background_pattern_table = background;
        self.sprite_pattern_table = sprites;
        true
    }

    pub fn save_project(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::io::Write;
        let path = path.as_ref();
        let data = serde_json::json!({"format":"fchua", "version":1, "document":self});
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        temp.write_all(&serde_json::to_vec_pretty(&data)?)?;
        temp.as_file().sync_all()?;
        temp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }

    pub fn load_project(path: impl AsRef<Path>) -> std::io::Result<Self> {
        if fs::metadata(path.as_ref())?.len() > 16 * 1024 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Project exceeds 16 MiB",
            ));
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        let invalid =
            || std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid FCHUA project");
        if (value["format"] != "fchua" && value["format"] != "kitaqfc-graphics-project")
            || value["version"] != 1
        {
            return Err(invalid());
        }
        let mut doc: Self =
            serde_json::from_value(value["document"].clone()).map_err(|_| invalid())?;
        if doc.chr.len() != CHR_ROM_SIZE
            || doc.nametable.len() != NAMETABLE_SIZE
            || doc.attributes.len() != ATTRIBUTE_SIZE
            || doc.palette_map.len() != NAMETABLE_SIZE
            || doc.palette_map.iter().any(|v| *v > 3)
            || doc.palette.iter().any(|v| *v > 63)
            || doc.background_pattern_table > 1
            || doc.sprite_pattern_table > 1
            || doc.selected_tile >= TILE_COUNT
            || doc.selected_color > 3
            || doc.selected_palette_group > 7
            || doc.metasprites.is_empty()
            || doc.metasprites.len() > 256
            || doc.current_frame >= doc.metasprites.len()
            || doc.metasprites.iter().any(|f| f.quads.len() > 64)
        {
            return Err(invalid());
        }
        // Hardware attributes are authoritative, including when migrating old projects.
        doc.palette_map = palette_map_from_attributes(&doc.attributes);
        for i in (0..16).step_by(4) {
            doc.palette[i + 16] = doc.palette[i];
        }
        Ok(doc)
    }

    pub fn load_palette_bytes(&mut self, bytes: &[u8]) {
        for (index, value) in bytes.iter().take(PALETTE_SIZE).enumerate() {
            self.set_palette_entry(index, *value);
        }
    }

    pub fn load_nametable_bytes(&mut self, bytes: &[u8]) {
        self.nametable.fill(0);
        let len = bytes.len().min(NAMETABLE_SIZE);
        self.nametable[..len].copy_from_slice(&bytes[..len]);
    }

    pub fn load_attribute_bytes(&mut self, bytes: &[u8]) {
        self.attributes.fill(0);
        let len = bytes.len().min(ATTRIBUTE_SIZE);
        self.attributes[..len].copy_from_slice(&bytes[..len]);
        self.palette_map = palette_map_from_attributes(&self.attributes);
    }

    pub fn set_palette_entry(&mut self, index: usize, value: u8) -> bool {
        if index >= PALETTE_SIZE {
            return false;
        }
        self.palette[index] = value & 0x3F;
        if index % 4 == 0 {
            self.palette[index ^ 16] = value & 0x3F;
        }
        true
    }

    pub fn set_palette_group_colors(&mut self, group: u8, colors: [u8; 4]) -> bool {
        let group = group as usize;
        if group >= 8 {
            return false;
        }
        let base = group * 4;
        for (index, color) in colors.into_iter().enumerate() {
            self.set_palette_entry(base + index, color);
        }
        true
    }

    pub fn tile_pixel(&self, tile: usize, x: usize, y: usize) -> u8 {
        if tile >= TILE_COUNT || x >= TILE_SIDE || y >= TILE_SIDE {
            return 0;
        }
        let offset = tile * TILE_BYTES + y;
        let bit = 7 - x;
        let lo = (self.chr[offset] >> bit) & 1;
        let hi = (self.chr[offset + 8] >> bit) & 1;
        lo | (hi << 1)
    }

    pub fn set_tile_pixel(&mut self, tile: usize, x: usize, y: usize, color: u8) -> bool {
        if tile >= TILE_COUNT || x >= TILE_SIDE || y >= TILE_SIDE {
            return false;
        }
        let offset = tile * TILE_BYTES + y;
        let bit = 7 - x;
        let mask = 1 << bit;
        let color = color & 0x03;
        if color & 1 == 1 {
            self.chr[offset] |= mask;
        } else {
            self.chr[offset] &= !mask;
        }
        if color & 2 == 2 {
            self.chr[offset + 8] |= mask;
        } else {
            self.chr[offset + 8] &= !mask;
        }
        true
    }

    pub fn fill_tile(&mut self, tile: usize, color: u8) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                self.set_tile_pixel(tile, x, y, color);
            }
        }
        true
    }

    pub fn checker_tile(&mut self, tile: usize, color_a: u8, color_b: u8) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                let color = if (x + y) % 2 == 0 { color_a } else { color_b };
                self.set_tile_pixel(tile, x, y, color);
            }
        }
        true
    }

    pub fn outline_tile(&mut self, tile: usize, color: u8) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                if x == 0 || y == 0 || x == TILE_SIDE - 1 || y == TILE_SIDE - 1 {
                    self.set_tile_pixel(tile, x, y, color);
                }
            }
        }
        true
    }

    pub fn copy_tile(&mut self, src: usize, dst: usize) -> bool {
        if src >= TILE_COUNT || dst >= TILE_COUNT {
            return false;
        }
        let src_offset = src * TILE_BYTES;
        let dst_offset = dst * TILE_BYTES;
        let mut bytes = [0_u8; TILE_BYTES];
        bytes.copy_from_slice(&self.chr[src_offset..src_offset + TILE_BYTES]);
        self.chr[dst_offset..dst_offset + TILE_BYTES].copy_from_slice(&bytes);
        true
    }

    pub fn mirror_tile(&mut self, tile: usize, horizontal: bool, vertical: bool) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        let mut pixels = [[0_u8; TILE_SIDE]; TILE_SIDE];
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                pixels[y][x] = self.tile_pixel(tile, x, y);
            }
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                let sx = if horizontal { TILE_SIDE - 1 - x } else { x };
                let sy = if vertical { TILE_SIDE - 1 - y } else { y };
                self.set_tile_pixel(tile, x, y, pixels[sy][sx]);
            }
        }
        true
    }

    pub fn rotate_tile_cw(&mut self, tile: usize) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        let mut pixels = [[0_u8; TILE_SIDE]; TILE_SIDE];
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                pixels[y][x] = self.tile_pixel(tile, x, y);
            }
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                self.set_tile_pixel(tile, x, y, pixels[TILE_SIDE - 1 - x][y]);
            }
        }
        true
    }

    pub fn invert_tile(&mut self, tile: usize) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        for y in 0..TILE_SIDE {
            for x in 0..TILE_SIDE {
                let color = self.tile_pixel(tile, x, y);
                self.set_tile_pixel(tile, x, y, if color == 0 { 0 } else { 3 - color });
            }
        }
        true
    }

    pub fn draw_line_tile(
        &mut self,
        tile: usize,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u8,
    ) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        let mut x0 = x0.clamp(0, 7);
        let mut y0 = y0.clamp(0, 7);
        let x1 = x1.clamp(0, 7);
        let y1 = y1.clamp(0, 7);
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_tile_pixel(tile, x0 as usize, y0 as usize, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err * 2;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
        true
    }

    pub fn fill_tile_rect(
        &mut self,
        tile: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: u8,
    ) -> bool {
        if tile >= TILE_COUNT {
            return false;
        }
        let min_x = x0.min(x1).min(7);
        let max_x = x0.max(x1).min(7);
        let min_y = y0.min(y1).min(7);
        let max_y = y0.max(y1).min(7);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.set_tile_pixel(tile, x, y, color);
            }
        }
        true
    }

    pub fn flood_fill_tile(&mut self, tile: usize, x: usize, y: usize, color: u8) -> bool {
        if tile >= TILE_COUNT || x >= TILE_SIDE || y >= TILE_SIDE {
            return false;
        }
        let target = self.tile_pixel(tile, x, y);
        let color = color & 3;
        if target == color {
            return true;
        }
        let mut stack = vec![(x, y)];
        let mut visited = [[false; TILE_SIDE]; TILE_SIDE];
        while let Some((cx, cy)) = stack.pop() {
            if cx >= TILE_SIDE || cy >= TILE_SIDE || visited[cy][cx] {
                continue;
            }
            visited[cy][cx] = true;
            if self.tile_pixel(tile, cx, cy) != target {
                continue;
            }
            self.set_tile_pixel(tile, cx, cy, color);
            if cx > 0 {
                stack.push((cx - 1, cy));
            }
            if cx + 1 < TILE_SIDE {
                stack.push((cx + 1, cy));
            }
            if cy > 0 {
                stack.push((cx, cy - 1));
            }
            if cy + 1 < TILE_SIDE {
                stack.push((cx, cy + 1));
            }
        }
        true
    }

    pub fn set_nametable_tile(&mut self, x: usize, y: usize, tile: u8, palette_group: u8) -> bool {
        if x >= NAMETABLE_W || y >= NAMETABLE_H {
            return false;
        }
        let index = y * NAMETABLE_W + x;
        self.nametable[index] = tile;
        self.set_attribute_quadrant(x, y, palette_group.min(3));
        self.rebuild_attributes();
        true
    }

    pub fn fill_nametable_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        tile: u8,
        palette_group: u8,
    ) -> bool {
        if x >= NAMETABLE_W || y >= NAMETABLE_H || w == 0 || h == 0 {
            return false;
        }
        let max_x = (x + w).min(NAMETABLE_W);
        let max_y = (y + h).min(NAMETABLE_H);
        for ty in y..max_y {
            for tx in x..max_x {
                let index = ty * NAMETABLE_W + tx;
                self.nametable[index] = tile;
                self.set_attribute_quadrant(tx, ty, palette_group.min(3));
            }
        }
        self.rebuild_attributes();
        true
    }

    pub fn rebuild_attributes(&mut self) {
        self.attributes.fill(0);
        for ay in 0..8 {
            for ax in 0..8 {
                let byte_index = ay * 8 + ax;
                let mut value = 0_u8;
                for qy in 0..2 {
                    for qx in 0..2 {
                        let group = self.dominant_palette_group(ax * 4 + qx * 2, ay * 4 + qy * 2);
                        let shift = (qy * 2 + qx) * 2;
                        value |= (group & 0x03) << shift;
                    }
                }
                self.attributes[byte_index] = value;
            }
        }
        self.palette_map = palette_map_from_attributes(&self.attributes);
    }

    fn set_attribute_quadrant(&mut self, x: usize, y: usize, group: u8) {
        let (x, y) = (x & !1, y & !1);
        for yy in y..(y + 2).min(NAMETABLE_H) {
            for xx in x..(x + 2).min(NAMETABLE_W) {
                self.palette_map[yy * NAMETABLE_W + xx] = group;
            }
        }
    }

    fn dominant_palette_group(&self, tile_x: usize, tile_y: usize) -> u8 {
        let mut counts = [0_u8; 4];
        for y in tile_y..(tile_y + 2).min(NAMETABLE_H) {
            for x in tile_x..(tile_x + 2).min(NAMETABLE_W) {
                let group = self.palette_map[y * NAMETABLE_W + x].min(3) as usize;
                counts[group] = counts[group].saturating_add(1);
            }
        }
        counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(index, _)| index as u8)
            .unwrap_or(0)
    }

    pub fn dedupe_tiles(&mut self) -> usize {
        let mut canonical: HashMap<[u8; TILE_BYTES], usize> = HashMap::new();
        let mut remap = [0_usize; TILE_COUNT];
        let mut duplicates = 0;
        for tile in 0..TILE_COUNT {
            // Tile IDs are eight bits within the selected 4 KiB PPU table.
            if tile % 256 == 0 {
                canonical.clear();
            }
            let offset = tile * TILE_BYTES;
            let mut key = [0_u8; TILE_BYTES];
            key.copy_from_slice(&self.chr[offset..offset + TILE_BYTES]);
            if let Some(first) = canonical.get(&key).copied() {
                remap[tile] = first;
                duplicates += 1;
            } else {
                canonical.insert(key, tile);
                remap[tile] = tile;
            }
        }
        for tile in &mut self.nametable {
            let old = *tile as usize + self.background_pattern_table as usize * 256;
            let new = remap[old];
            *tile = (new % 256) as u8;
        }
        for frame in &mut self.metasprites {
            for quad in &mut frame.quads {
                let old = quad.tile as usize + self.sprite_pattern_table as usize * 256;
                quad.tile = (remap[old] % 256) as u8;
            }
        }
        duplicates
    }

    pub fn add_metasprite_frame(&mut self) -> usize {
        self.metasprites.push(MetaSpriteFrame::new(Vec::new()));
        self.current_frame = self.metasprites.len() - 1;
        self.current_frame
    }

    pub fn set_current_frame(&mut self, frame: usize) -> bool {
        if frame >= self.metasprites.len() {
            return false;
        }
        self.current_frame = frame;
        true
    }

    pub fn set_metasprite_quad(
        &mut self,
        frame: usize,
        quad_index: usize,
        quad: MetaSpriteQuad,
    ) -> bool {
        if frame >= 256 || quad_index >= 64 || quad.dx == 255 || quad.attr & 0x1c != 0 {
            return false;
        }
        while self.metasprites.len() <= frame {
            self.metasprites.push(MetaSpriteFrame::new(Vec::new()));
        }
        let quads = &mut self.metasprites[frame].quads;
        while quads.len() <= quad_index {
            quads.push(MetaSpriteQuad::new(0, 0, 0, 0));
        }
        quads[quad_index] = quad;
        self.current_frame = frame;
        true
    }

    pub fn analysis_report(&self) -> AnalysisReport {
        let mut unique = HashSet::new();
        let mut empty_tiles = 0;
        for tile in 0..TILE_COUNT {
            let offset = tile * TILE_BYTES;
            let chunk = &self.chr[offset..offset + TILE_BYTES];
            if chunk.iter().all(|byte| *byte == 0) {
                empty_tiles += 1;
            }
            let mut key = [0_u8; TILE_BYTES];
            key.copy_from_slice(chunk);
            unique.insert(key);
        }
        let nametable_used_tiles = self.nametable.iter().copied().collect::<HashSet<_>>().len();
        let metasprite_max_quads = self
            .metasprites
            .iter()
            .map(|frame| frame.quads.len())
            .max()
            .unwrap_or(0);
        let mut warnings = Vec::new();
        for (index, frame) in self.metasprites.iter().enumerate() {
            let max_line = (-128..135)
                .map(|y| {
                    frame
                        .quads
                        .iter()
                        .filter(|q| {
                            let top = i32::from(q.dy as i8);
                            y >= top && y < top + 8
                        })
                        .count()
                })
                .max()
                .unwrap_or(0);
            if max_line > 8 {
                warnings.push(format!(
                    "frame {index}: {max_line} sprites on one scanline (NES limit: 8)"
                ));
            }
            if frame
                .quads
                .iter()
                .any(|q| q.dx == 255 || q.attr & 0x1c != 0)
            {
                warnings.push(format!(
                    "frame {index}: reserved metasprite offset or attribute bits"
                ));
            }
        }
        if metasprite_max_quads > 64 {
            warnings.push("metasprite frame exceeds NES OAM capacity".to_string());
        }
        if nametable_used_tiles > 192 {
            warnings.push("nametable uses many unique BG tiles; check CHR budget".to_string());
        }
        if empty_tiles < 16 {
            warnings.push("CHR bank is almost full; reserve blank tiles for streaming".to_string());
        }
        AnalysisReport {
            unique_tiles: unique.len(),
            duplicate_tiles: TILE_COUNT.saturating_sub(unique.len()),
            empty_tiles,
            nametable_used_tiles,
            metasprite_max_quads,
            chr_bytes: self.chr.len(),
            warnings,
        }
    }

    pub fn render_tile_rgba_scaled(
        &self,
        tile: usize,
        palette_group: u8,
        scale: usize,
        with_grid: bool,
    ) -> (usize, usize, Vec<u8>) {
        let scale = scale.max(1);
        let width = TILE_SIDE * scale;
        let height = TILE_SIDE * scale;
        let mut pixels = vec![0_u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let tx = x / scale;
                let ty = y / scale;
                let mut rgba = self.rgba_for_index(palette_group, self.tile_pixel(tile, tx, ty));
                if with_grid && (x % scale == 0 || y % scale == 0) {
                    rgba = [26, 27, 26, 255];
                }
                put_pixel(&mut pixels, width, x, y, rgba);
            }
        }
        (width, height, pixels)
    }

    pub fn render_chr_atlas_rgba(&self, cols: usize, cell: usize) -> (usize, usize, Vec<u8>) {
        let cols = cols.max(1);
        let cell = cell.max(10);
        let rows = (TILE_COUNT + cols - 1) / cols;
        let width = cols * cell;
        let height = rows * cell;
        let mut pixels = vec![18_u8; width * height * 4];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[18, 19, 18, 255]);
        }
        for tile in 0..TILE_COUNT {
            let col = tile % cols;
            let row = tile / cols;
            let ox = col * cell + 2;
            let oy = row * cell + 2;
            for y in 0..TILE_SIDE {
                for x in 0..TILE_SIDE {
                    let color = self.tile_pixel(tile, x, y);
                    let rgba = self.rgba_for_index(self.selected_palette_group, color);
                    put_pixel(&mut pixels, width, ox + x, oy + y, rgba);
                }
            }
        }
        (width, height, pixels)
    }

    pub fn render_nametable_rgba(&self) -> (usize, usize, Vec<u8>) {
        let width = NAMETABLE_W * TILE_SIDE;
        let height = NAMETABLE_H * TILE_SIDE;
        let mut pixels = vec![0_u8; width * height * 4];
        for ty in 0..NAMETABLE_H {
            for tx in 0..NAMETABLE_W {
                let nt_index = ty * NAMETABLE_W + tx;
                let tile = self.nametable[nt_index] as usize
                    + self.background_pattern_table as usize * 256;
                let group = self.palette_map[nt_index];
                for py in 0..TILE_SIDE {
                    for px in 0..TILE_SIDE {
                        let color = self.tile_pixel(tile, px, py);
                        let rgba = self.rgba_for_index(group, color);
                        put_pixel(
                            &mut pixels,
                            width,
                            tx * TILE_SIDE + px,
                            ty * TILE_SIDE + py,
                            rgba,
                        );
                    }
                }
            }
        }
        (width, height, pixels)
    }

    pub fn render_metasprite_rgba(&self, frame: usize, scale: usize) -> (usize, usize, Vec<u8>) {
        let scale = scale.max(1);
        let width = 160 * scale;
        let height = 160 * scale;
        let mut pixels = vec![0_u8; width * height * 4];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[25, 26, 24, 255]);
        }
        let grid = [47, 50, 46, 255];
        for y in (0..height).step_by(8 * scale) {
            for x in 0..width {
                put_pixel(&mut pixels, width, x, y, grid);
            }
        }
        for x in (0..width).step_by(8 * scale) {
            for y in 0..height {
                put_pixel(&mut pixels, width, x, y, grid);
            }
        }
        let Some(frame) = self.metasprites.get(frame) else {
            return (width, height, pixels);
        };
        let base_x = 72_i32 * scale as i32;
        let base_y = 72_i32 * scale as i32;
        // Lower OAM indices take priority when opaque sprite pixels overlap.
        for quad in frame.quads.iter().rev() {
            let pal_group = 4 + (quad.attr & 0x03);
            let flip_h = quad.attr & 0x40 != 0;
            let flip_v = quad.attr & 0x80 != 0;
            let ox = base_x + i32::from(quad.dx as i8) * scale as i32;
            let oy = base_y + i32::from(quad.dy as i8) * scale as i32;
            for py in 0..TILE_SIDE {
                for px in 0..TILE_SIDE {
                    let sx = if flip_h { TILE_SIDE - 1 - px } else { px };
                    let sy = if flip_v { TILE_SIDE - 1 - py } else { py };
                    let color = self.tile_pixel(
                        quad.tile as usize + self.sprite_pattern_table as usize * 256,
                        sx,
                        sy,
                    );
                    if color == 0 {
                        continue;
                    }
                    let rgba = self.rgba_for_index(pal_group, color);
                    for yy in 0..scale {
                        for xx in 0..scale {
                            let dx = ox + (px * scale + xx) as i32;
                            let dy = oy + (py * scale + yy) as i32;
                            if dx >= 0 && dy >= 0 {
                                let dx = dx as usize;
                                let dy = dy as usize;
                                if dx < width && dy < height {
                                    put_pixel(&mut pixels, width, dx, dy, rgba);
                                }
                            }
                        }
                    }
                }
            }
        }
        (width, height, pixels)
    }

    pub fn rgba_for_index(&self, palette_group: u8, color: u8) -> [u8; 4] {
        let palette_index = if color == 0 {
            0
        } else {
            (usize::from(palette_group.min(7)) * 4 + usize::from(color.min(3)))
                .min(PALETTE_SIZE - 1)
        };
        let nes_index = self.palette[palette_index] as usize & 0x3F;
        let [r, g, b] = NES_PALETTE_RGB[nes_index];
        [r, g, b, 255]
    }

    pub fn export_raw_assets(
        &self,
        out_dir: impl AsRef<Path>,
        prefix: &str,
    ) -> std::io::Result<ExportReport> {
        if self.metasprites.iter().any(|f| {
            f.quads.len() > 64 || f.quads.iter().any(|q| q.dx == 255 || q.attr & 0x1c != 0)
        }) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid metasprite: capacity, sentinel offset, or reserved attribute bits",
            ));
        }
        let out_dir = out_dir.as_ref();
        fs::create_dir_all(out_dir)?;
        let prefix = sanitize_prefix(prefix);
        let chr_path = out_dir.join(format!("{prefix}.chr"));
        let palette_path = out_dir.join(format!("{prefix}.pal"));
        let nametable_path = out_dir.join(format!("{prefix}.nam"));
        let attribute_path = out_dir.join(format!("{prefix}.atr"));
        let metasprite_path = out_dir.join(format!("{prefix}.metasprite.json"));
        let manifest_path = out_dir.join(format!("{prefix}_pack_manifest.json"));
        let c_path = out_dir.join(format!("{prefix}_assets.c"));
        let h_path = out_dir.join(format!("{prefix}_assets.h"));

        fs::write(&chr_path, &self.chr)?;
        self.save_project(out_dir.join(format!("{prefix}.fch")))?;
        fs::write(&palette_path, self.palette)?;
        fs::write(&nametable_path, &self.nametable)?;
        fs::write(&attribute_path, &self.attributes)?;
        fs::write(&metasprite_path, self.metasprite_json(&prefix))?;
        fs::write(&manifest_path, self.pack_manifest_json(&prefix))?;
        fs::write(&h_path, self.generated_c_header(&prefix))?;
        fs::write(&c_path, self.generated_c_source(&prefix, &h_path))?;

        Ok(ExportReport {
            chr_path,
            palette_path,
            nametable_path,
            attribute_path,
            metasprite_path,
            manifest_path,
            c_path,
            h_path,
        })
    }

    pub fn pack_manifest_json(&self, prefix: &str) -> String {
        let prefix = sanitize_prefix(prefix);
        format!(
            "{{\n  \"chr_out\": \"generated/{prefix}_chr8k.chr\",\n  \"c_out\": \"generated/{prefix}_assets.c\",\n  \"h_out\": \"generated/{prefix}_assets.h\",\n  \"report_out\": \"generated/{prefix}_assets_report.json\",\n  \"chr\": [{{\"name\": \"{prefix}_chr\", \"input\": \"{prefix}.chr\", \"tile_offset\": 0}}],\n  \"palettes\": [{{\"name\": \"{prefix}_palette32\", \"input\": \"{prefix}.pal\"}}],\n  \"nametables\": [{{\"name\": \"{prefix}_nametable\", \"input\": \"{prefix}.nam\"}}],\n  \"attributes\": [{{\"name\": \"{prefix}_attributes\", \"input\": \"{prefix}.atr\"}}],\n  \"metasprites\": [{{\"name\": \"{prefix}_metasprite\", \"input\": \"{prefix}.metasprite.json\"}}]\n}}\n"
        )
    }

    pub fn metasprite_json(&self, name: &str) -> String {
        let name = json_escape(name);
        let mut text = String::new();
        text.push_str("{\n");
        text.push_str(&format!("  \"name\": \"{name}\",\n"));
        text.push_str("  \"frames\": [\n");
        for (frame_index, frame) in self.metasprites.iter().enumerate() {
            text.push_str("    [");
            if !frame.quads.is_empty() {
                text.push('\n');
                for (quad_index, quad) in frame.quads.iter().enumerate() {
                    text.push_str(&format!(
                        "      [{}, {}, {}, {}]{}",
                        quad.dx,
                        quad.dy,
                        quad.tile,
                        quad.attr,
                        if quad_index + 1 == frame.quads.len() {
                            "\n"
                        } else {
                            ",\n"
                        }
                    ));
                }
                text.push_str("    ]");
            } else {
                text.push(']');
            }
            if frame_index + 1 != self.metasprites.len() {
                text.push(',');
            }
            text.push('\n');
        }
        text.push_str("  ]\n");
        text.push_str("}\n");
        text
    }

    pub fn generated_c_header(&self, prefix: &str) -> String {
        let prefix = sanitize_prefix(prefix);
        let mut text = format!(
            "/* Generated by FCHUA for KITAQFC. */\n#pragma once\n\nextern __prg_rom const unsigned char {prefix}_palette32[{PALETTE_SIZE}];\nextern __prg_rom const unsigned char {prefix}_nametable[{NAMETABLE_SIZE}];\nextern __prg_rom const unsigned char {prefix}_attributes[{ATTRIBUTE_SIZE}];\nextern __prg_rom const unsigned char {prefix}_metasprite_0[];\n"
        );
        for i in 1..self.metasprites.len() {
            text.push_str(&format!(
                "extern __prg_rom const unsigned char {prefix}_metasprite_{i}[];\n"
            ));
        }
        text
    }

    pub fn generated_c_source(&self, prefix: &str, header_path: &Path) -> String {
        let prefix = sanitize_prefix(prefix);
        let header = header_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("kitaqfc_assets.h");
        let mut text = String::new();
        text.push_str("/* Generated by FCHUA for KITAQFC. */\n");
        text.push_str(&format!("#include \"{}\"\n\n", json_escape(header)));
        text.push_str(&c_array(
            &format!("{prefix}_palette32"),
            "palette",
            &self.palette,
        ));
        text.push_str(&c_array(
            &format!("{prefix}_nametable"),
            "nametable",
            &self.nametable,
        ));
        text.push_str(&c_array(
            &format!("{prefix}_attributes"),
            "attribute",
            &self.attributes,
        ));
        for (i, frame) in self.metasprites.iter().enumerate() {
            text.push_str(&c_array(
                &format!("{prefix}_metasprite_{i}"),
                "metasprite_frame",
                &flatten_metasprite_frame(frame),
            ));
        }
        text
    }

    pub fn apply_ai_command(&mut self, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return "empty command".to_string();
        }
        let op = parts[0].to_ascii_lowercase();
        match op.as_str() {
            "clear_chr" => {
                for tile in 0..TILE_COUNT {
                    self.fill_tile(tile, 0);
                }
                "cleared all CHR tiles".to_string()
            }
            "pixel" if parts.len() >= 5 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let x = parse_usize(parts[2], 0);
                let y = parse_usize(parts[3], 0);
                let color = parse_u8(parts[4], self.selected_color);
                self.set_tile_pixel(tile, x, y, color);
                format!("pixel tile {tile} at {x},{y} color {color}")
            }
            "line" if parts.len() >= 7 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let x0 = parse_i32(parts[2], 0);
                let y0 = parse_i32(parts[3], 0);
                let x1 = parse_i32(parts[4], 0);
                let y1 = parse_i32(parts[5], 0);
                let color = parse_u8(parts[6], self.selected_color);
                self.draw_line_tile(tile, x0, y0, x1, y1, color);
                format!("line tile {tile} {x0},{y0} to {x1},{y1} color {color}")
            }
            "rect" if parts.len() >= 7 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let x = parse_usize(parts[2], 0);
                let y = parse_usize(parts[3], 0);
                let w = parse_usize(parts[4], 1);
                let h = parse_usize(parts[5], 1);
                let color = parse_u8(parts[6], self.selected_color);
                self.fill_tile_rect(tile, x, y, w, h, color);
                format!("rect tile {tile} at {x},{y} {w}x{h} color {color}")
            }
            "tile_rows" if parts.len() >= 10 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                if tile < TILE_COUNT {
                    for y in 0..TILE_SIDE {
                        let row = parts[y + 2].as_bytes();
                        for x in 0..TILE_SIDE {
                            let color = row
                                .get(x)
                                .copied()
                                .filter(|value| (b'0'..=b'3').contains(value))
                                .map(|value| value - b'0')
                                .unwrap_or(0);
                            self.set_tile_pixel(tile, x, y, color);
                        }
                    }
                }
                format!("set tile {tile} from 8 pixel rows")
            }
            "palette" if parts.len() >= 3 => {
                let index = parse_usize(parts[1], 0);
                let value = parse_u8(parts[2], 0x0F);
                self.set_palette_entry(index, value);
                format!("palette entry {index} color {value}")
            }
            "palette_group" if parts.len() >= 6 => {
                let group = parse_u8(parts[1], 0);
                let c0 = parse_u8(parts[2], 0x0F);
                let c1 = parse_u8(parts[3], 0);
                let c2 = parse_u8(parts[4], 0x10);
                let c3 = parse_u8(parts[5], 0x30);
                self.set_palette_group_colors(group, [c0, c1, c2, c3]);
                format!("palette group {group} colors {c0}/{c1}/{c2}/{c3}")
            }
            "metasprite" if parts.len() >= 7 => {
                let frame = parse_usize(parts[1], 0);
                let quad = parse_usize(parts[2], 0);
                let dx = parse_u8(parts[3], 0);
                let dy = parse_u8(parts[4], 0);
                let tile = parse_u8(parts[5], 0);
                let attr = parse_u8(parts[6], 0);
                self.set_metasprite_quad(frame, quad, MetaSpriteQuad::new(dx, dy, tile, attr));
                format!(
                    "metasprite frame {frame} quad {quad} dx {dx} dy {dy} tile {tile} attr {attr}"
                )
            }
            "checker" if parts.len() >= 4 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let a = parse_u8(parts[2], 1);
                let b = parse_u8(parts[3], 2);
                self.checker_tile(tile, a, b);
                format!("checker tile {tile} colors {a}/{b}")
            }
            "outline" if parts.len() >= 3 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let color = parse_u8(parts[2], self.selected_color);
                self.outline_tile(tile, color);
                format!("outlined tile {tile} color {color}")
            }
            "fill_tile" if parts.len() >= 3 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                let color = parse_u8(parts[2], self.selected_color);
                self.fill_tile(tile, color);
                format!("filled tile {tile} color {color}")
            }
            "mirror_h" if parts.len() >= 2 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                self.mirror_tile(tile, true, false);
                format!("mirrored tile {tile} horizontally")
            }
            "mirror_v" if parts.len() >= 2 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                self.mirror_tile(tile, false, true);
                format!("mirrored tile {tile} vertically")
            }
            "rotate" if parts.len() >= 2 => {
                let tile = parse_usize(parts[1], self.selected_tile);
                self.rotate_tile_cw(tile);
                format!("rotated tile {tile}")
            }
            "nametable_fill" if parts.len() >= 7 => {
                let x = parse_usize(parts[1], 0);
                let y = parse_usize(parts[2], 0);
                let w = parse_usize(parts[3], 1);
                let h = parse_usize(parts[4], 1);
                let tile = parse_u8(parts[5], self.selected_tile as u8);
                let group = parse_u8(parts[6], self.selected_palette_group);
                self.fill_nametable_rect(x, y, w, h, tile, group);
                format!("filled nametable rect {x},{y} {w}x{h} tile {tile} pal {group}")
            }
            "dedupe" => {
                let count = self.dedupe_tiles();
                format!("deduped {count} duplicate tiles")
            }
            "rebuild_attrs" | "rebuild_attributes" => {
                self.rebuild_attributes();
                "rebuilt attribute table".to_string()
            }
            "sample_project" | "reset_demo" => {
                *self = Self::new_demo();
                "loaded demo project".to_string()
            }
            _ => format!("unknown ai command: {command}"),
        }
    }
}

fn put_pixel(pixels: &mut [u8], width: usize, x: usize, y: usize, rgba: [u8; 4]) {
    let index = (y * width + x) * 4;
    if index + 4 <= pixels.len() {
        pixels[index..index + 4].copy_from_slice(&rgba);
    }
}

fn palette_map_from_attributes(attributes: &[u8]) -> Vec<u8> {
    let mut map = vec![0_u8; NAMETABLE_SIZE];
    for ay in 0..8 {
        for ax in 0..8 {
            let value = attributes.get(ay * 8 + ax).copied().unwrap_or(0);
            for qy in 0..2 {
                for qx in 0..2 {
                    let shift = (qy * 2 + qx) * 2;
                    let group = (value >> shift) & 0x03;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let x = ax * 4 + qx * 2 + dx;
                            let y = ay * 4 + qy * 2 + dy;
                            if x < NAMETABLE_W && y < NAMETABLE_H {
                                map[y * NAMETABLE_W + x] = group;
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

fn flatten_metasprite_frame(frame: &MetaSpriteFrame) -> Vec<u8> {
    let mut out = Vec::with_capacity(frame.quads.len() * 4 + 1);
    for quad in &frame.quads {
        out.extend_from_slice(&[quad.dx, quad.dy, quad.tile, quad.attr]);
    }
    out.push(0xFF);
    out
}

fn c_array(name: &str, kind: &str, data: &[u8]) -> String {
    let mut text = String::new();
    text.push_str(&format!(
        "/* kind: {kind} */\n__prg_rom const unsigned char {name}[{}] = {{\n",
        data.len()
    ));
    for chunk in data.chunks(16) {
        text.push_str("    ");
        for (index, byte) in chunk.iter().enumerate() {
            text.push_str(&format!("0x{byte:02X}"));
            if index + 1 != chunk.len() {
                text.push_str(", ");
            }
        }
        text.push_str(",\n");
    }
    text.push_str("};\n\n");
    text
}

fn sanitize_prefix(prefix: &str) -> String {
    let out: String = prefix
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if out.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        format!("asset_{out}")
    } else if out.is_empty() {
        "kitaqfc_asset".to_string()
    } else {
        out
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn parse_usize(value: &str, fallback: usize) -> usize {
    value.parse().unwrap_or(fallback)
}

fn parse_u8(value: &str, fallback: u8) -> u8 {
    value.parse().unwrap_or(fallback)
}

fn parse_i32(value: &str, fallback: i32) -> i32 {
    value.parse().unwrap_or(fallback)
}

#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "gui")]
pub use gui::{run_editor_from_args, run_editor_with_initial_asset};
pub mod commands;
pub mod image_io;
#[cfg(feature = "python")]
mod python_api {
    use super::*;
    #[cfg(feature = "gui")]
    use plita::python as plita_python;
    use pyo3::exceptions::{PyRuntimeError, PyValueError};
    use pyo3::prelude::*;
    use pyo3::types::{PyBytes, PyModule};

    #[pyclass(name = "Snapshot")]
    #[derive(Clone)]
    pub struct PyEditorSnapshot {
        #[pyo3(get)]
        pub selected_tile: usize,
        #[pyo3(get)]
        pub selected_color: u8,
        #[pyo3(get)]
        pub selected_palette_group: u8,
        #[pyo3(get)]
        pub current_frame: usize,
        #[pyo3(get)]
        pub unique_tiles: usize,
        #[pyo3(get)]
        pub duplicate_tiles: usize,
        #[pyo3(get)]
        pub empty_tiles: usize,
        #[pyo3(get)]
        pub nametable_used_tiles: usize,
        #[pyo3(get)]
        pub metasprite_max_quads: usize,
        #[pyo3(get)]
        pub warnings: Vec<String>,
    }

    #[pyclass(name = "Controller")]
    pub struct PyKitaqfcGraphicsController {
        doc: KitaqfcGraphicsDocument,
    }

    impl PyKitaqfcGraphicsController {
        fn snapshot_inner(&self) -> PyEditorSnapshot {
            let report = self.doc.analysis_report();
            PyEditorSnapshot {
                selected_tile: self.doc.selected_tile(),
                selected_color: self.doc.selected_color(),
                selected_palette_group: self.doc.selected_palette_group(),
                current_frame: self.doc.current_frame(),
                unique_tiles: report.unique_tiles,
                duplicate_tiles: report.duplicate_tiles,
                empty_tiles: report.empty_tiles,
                nametable_used_tiles: report.nametable_used_tiles,
                metasprite_max_quads: report.metasprite_max_quads,
                warnings: report.warnings,
            }
        }
    }

    #[pymethods]
    impl PyKitaqfcGraphicsController {
        #[new]
        fn new() -> Self {
            Self {
                doc: KitaqfcGraphicsDocument::new_demo(),
            }
        }

        fn snapshot(&self) -> PyEditorSnapshot {
            self.snapshot_inner()
        }

        fn reset_demo(&mut self) {
            self.doc = KitaqfcGraphicsDocument::new_demo();
        }

        fn new_blank(&mut self) {
            self.doc = KitaqfcGraphicsDocument::new_blank();
        }

        fn apply_commands(&mut self, json: &str) -> PyResult<String> {
            let commands: Vec<commands::Command> =
                serde_json::from_str(json).map_err(|e| PyValueError::new_err(e.to_string()))?;
            commands::apply(&mut self.doc, &commands).map_err(PyValueError::new_err)?;
            serde_json::to_string(&self.doc.analysis_report())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        }

        #[pyo3(signature=(path, auto_palette=true))]
        fn import_png(&mut self, path: &str, auto_palette: bool) -> PyResult<String> {
            let (doc, report) = image_io::import_png(&self.doc, Path::new(path), auto_palette)
                .map_err(PyValueError::new_err)?;
            self.doc = doc;
            serde_json::to_string(&report).map_err(|e| PyRuntimeError::new_err(e.to_string()))
        }

        #[pyo3(signature=(path, view="map", scale=1))]
        fn preview_png(&self, path: &str, view: &str, scale: u32) -> PyResult<()> {
            image_io::preview_png(&self.doc, Path::new(path), view, scale)
                .map_err(PyValueError::new_err)
        }

        fn select_tile(&mut self, tile: usize) {
            self.doc.set_selected_tile(tile);
        }

        fn select_color(&mut self, color: u8) {
            self.doc.set_selected_color(color);
        }

        fn select_palette_group(&mut self, group: u8) {
            self.doc.set_selected_palette_group(group);
        }

        fn load_chr_bytes(&mut self, data: &[u8]) -> PyResult<()> {
            self.doc
                .try_load_chr_bytes(data)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        fn set_pattern_tables(&mut self, background: u8, sprites: u8) -> PyResult<()> {
            self.doc
                .set_pattern_tables(background, sprites)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("pattern table must be 0 or 1"))
        }

        fn save_project(&self, path: &str) -> PyResult<()> {
            self.doc
                .save_project(path)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        }

        fn load_project(&mut self, path: &str) -> PyResult<()> {
            self.doc = KitaqfcGraphicsDocument::load_project(path)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(())
        }

        fn load_palette_bytes(&mut self, data: &[u8]) {
            self.doc.load_palette_bytes(data);
        }

        fn load_nametable_bytes(&mut self, data: &[u8]) {
            self.doc.load_nametable_bytes(data);
        }

        fn load_attribute_bytes(&mut self, data: &[u8]) {
            self.doc.load_attribute_bytes(data);
        }

        fn set_palette_entry(&mut self, index: usize, value: u8) -> PyResult<()> {
            self.doc
                .set_palette_entry(index, value)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("palette index out of range"))
        }

        fn set_palette_group_colors(
            &mut self,
            group: u8,
            c0: u8,
            c1: u8,
            c2: u8,
            c3: u8,
        ) -> PyResult<()> {
            self.doc
                .set_palette_group_colors(group, [c0, c1, c2, c3])
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("palette group out of range"))
        }

        fn get_tile_pixel(&self, tile: usize, x: usize, y: usize) -> u8 {
            self.doc.tile_pixel(tile, x, y)
        }

        fn set_tile_pixel(&mut self, tile: usize, x: usize, y: usize, color: u8) -> PyResult<()> {
            self.doc
                .set_tile_pixel(tile, x, y, color)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile pixel coordinate out of range"))
        }

        fn fill_tile(&mut self, tile: usize, color: u8) -> PyResult<()> {
            self.doc
                .fill_tile(tile, color)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn checker_tile(&mut self, tile: usize, color_a: u8, color_b: u8) -> PyResult<()> {
            self.doc
                .checker_tile(tile, color_a, color_b)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn outline_tile(&mut self, tile: usize, color: u8) -> PyResult<()> {
            self.doc
                .outline_tile(tile, color)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn copy_tile(&mut self, src: usize, dst: usize) -> PyResult<()> {
            self.doc
                .copy_tile(src, dst)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn mirror_tile(&mut self, tile: usize, horizontal: bool, vertical: bool) -> PyResult<()> {
            self.doc
                .mirror_tile(tile, horizontal, vertical)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn rotate_tile_cw(&mut self, tile: usize) -> PyResult<()> {
            self.doc
                .rotate_tile_cw(tile)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn invert_tile(&mut self, tile: usize) -> PyResult<()> {
            self.doc
                .invert_tile(tile)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn draw_line_tile(
            &mut self,
            tile: usize,
            x0: i32,
            y0: i32,
            x1: i32,
            y1: i32,
            color: u8,
        ) -> PyResult<()> {
            self.doc
                .draw_line_tile(tile, x0, y0, x1, y1, color)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile out of range"))
        }

        fn flood_fill_tile(&mut self, tile: usize, x: usize, y: usize, color: u8) -> PyResult<()> {
            self.doc
                .flood_fill_tile(tile, x, y, color)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("tile pixel coordinate out of range"))
        }

        fn set_nametable_tile(
            &mut self,
            x: usize,
            y: usize,
            tile: u8,
            palette_group: u8,
        ) -> PyResult<()> {
            self.doc
                .set_nametable_tile(x, y, tile, palette_group)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("nametable coordinate out of range"))
        }

        fn fill_nametable_rect(
            &mut self,
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            tile: u8,
            palette_group: u8,
        ) -> PyResult<()> {
            self.doc
                .fill_nametable_rect(x, y, w, h, tile, palette_group)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("nametable rectangle out of range"))
        }

        fn rebuild_attributes(&mut self) {
            self.doc.rebuild_attributes();
        }

        fn dedupe_tiles(&mut self) -> usize {
            self.doc.dedupe_tiles()
        }

        fn add_metasprite_frame(&mut self) -> usize {
            self.doc.add_metasprite_frame()
        }

        fn set_current_frame(&mut self, frame: usize) -> PyResult<()> {
            self.doc
                .set_current_frame(frame)
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("metasprite frame out of range"))
        }

        fn set_metasprite_quad(
            &mut self,
            frame: usize,
            quad_index: usize,
            dx: u8,
            dy: u8,
            tile: u8,
            attr: u8,
        ) -> PyResult<()> {
            self.doc
                .set_metasprite_quad(frame, quad_index, MetaSpriteQuad::new(dx, dy, tile, attr))
                .then_some(())
                .ok_or_else(|| PyValueError::new_err("Invalid frame/quad/offset/attribute"))
        }

        fn apply_ai_command(&mut self, command: &str) -> String {
            self.doc.apply_ai_command(command)
        }

        fn pack_manifest_json(&self, prefix: &str) -> String {
            self.doc.pack_manifest_json(prefix)
        }

        fn metasprite_json(&self, name: &str) -> String {
            self.doc.metasprite_json(name)
        }

        fn generated_c_header(&self, prefix: &str) -> String {
            self.doc.generated_c_header(prefix)
        }

        fn generated_c_source(&self, prefix: &str, header_name: &str) -> String {
            self.doc.generated_c_source(prefix, Path::new(header_name))
        }

        fn export_raw_assets(&self, path: &str, prefix: &str) -> PyResult<Vec<String>> {
            let report = self
                .doc
                .export_raw_assets(path, prefix)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
            Ok(vec![
                report.chr_path.display().to_string(),
                report.palette_path.display().to_string(),
                report.nametable_path.display().to_string(),
                report.attribute_path.display().to_string(),
                report.metasprite_path.display().to_string(),
                report.manifest_path.display().to_string(),
                report.c_path.display().to_string(),
                report.h_path.display().to_string(),
            ])
        }

        fn chr_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new_bound(py, self.doc.chr())
        }

        fn palette_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new_bound(py, self.doc.palette())
        }

        fn nametable_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new_bound(py, self.doc.nametable())
        }

        fn attribute_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new_bound(py, self.doc.attributes())
        }

        #[pyo3(signature = (tile, palette_group = None, scale = 4, grid = false))]
        fn tile_rgba<'py>(
            &self,
            py: Python<'py>,
            tile: usize,
            palette_group: Option<u8>,
            scale: usize,
            grid: bool,
        ) -> Bound<'py, PyBytes> {
            let (_, _, pixels) = self.doc.render_tile_rgba_scaled(
                tile,
                palette_group.unwrap_or_else(|| self.doc.selected_palette_group()),
                scale,
                grid,
            );
            PyBytes::new_bound(py, &pixels)
        }

        fn tile_rgba_size(&self, scale: usize) -> (usize, usize) {
            let scale = scale.max(1);
            (TILE_SIDE * scale, TILE_SIDE * scale)
        }

        fn nametable_rgba<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            let (_, _, pixels) = self.doc.render_nametable_rgba();
            PyBytes::new_bound(py, &pixels)
        }

        fn nametable_rgba_size(&self) -> (usize, usize) {
            (NAMETABLE_W * TILE_SIDE, NAMETABLE_H * TILE_SIDE)
        }

        #[pyo3(signature = (frame = None, scale = 2))]
        fn metasprite_rgba<'py>(
            &self,
            py: Python<'py>,
            frame: Option<usize>,
            scale: usize,
        ) -> Bound<'py, PyBytes> {
            let (_, _, pixels) = self
                .doc
                .render_metasprite_rgba(frame.unwrap_or_else(|| self.doc.current_frame()), scale);
            PyBytes::new_bound(py, &pixels)
        }

        fn metasprite_rgba_size(&self, scale: usize) -> (usize, usize) {
            let scale = scale.max(1);
            (160 * scale, 160 * scale)
        }
    }

    #[cfg(feature = "gui")]
    #[pyfunction]
    #[pyo3(signature = (initial_asset = None))]
    fn run_editor(py: Python<'_>, initial_asset: Option<String>) -> PyResult<()> {
        let initial_asset = initial_asset.map(PathBuf::from);
        py.allow_threads(move || run_editor_with_initial_asset(initial_asset))
            .map_err(plita_python::app_error_to_py_err)
    }

    #[pymodule]
    fn fchua(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
        #[cfg(feature = "gui")]
        plita_python::add_plita_classes(module)?;
        module.add_class::<PyKitaqfcGraphicsController>()?;
        module.add_class::<PyEditorSnapshot>()?;
        #[cfg(feature = "gui")]
        module.add_function(wrap_pyfunction!(run_editor, module)?)?;
        module.add("GUI_AVAILABLE", cfg!(feature = "gui"))?;
        module.add("__version__", env!("CARGO_PKG_VERSION"))?;
        module.add("CHR_ROM_SIZE", CHR_ROM_SIZE)?;
        module.add("TILE_COUNT", TILE_COUNT)?;
        module.add("NAMETABLE_W", NAMETABLE_W)?;
        module.add("NAMETABLE_H", NAMETABLE_H)?;
        module.add("ATTRIBUTE_SIZE", ATTRIBUTE_SIZE)?;
        module.add("PALETTE_SIZE", PALETTE_SIZE)?;
        Ok(())
    }

    #[pymodule]
    fn kitaqfc_graphics_editor(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
        fchua(py, module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chr_pixel_round_trips_through_2bpp_planes() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.fill_tile(10, 0);
        assert_eq!(doc.tile_pixel(10, 3, 4), 0);

        doc.set_tile_pixel(10, 3, 4, 3);
        assert_eq!(doc.tile_pixel(10, 3, 4), 3);

        let offset = 10 * TILE_BYTES + 4;
        assert_ne!(doc.chr()[offset], 0);
        assert_ne!(doc.chr()[offset + 8], 0);
    }

    #[test]
    fn attribute_rebuild_packs_quadrant_bits() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.fill_nametable_rect(0, 0, 2, 2, 1, 1);
        doc.fill_nametable_rect(2, 0, 2, 2, 1, 2);
        doc.fill_nametable_rect(0, 2, 2, 2, 1, 3);
        doc.fill_nametable_rect(2, 2, 2, 2, 1, 0);
        doc.rebuild_attributes();

        assert_eq!(doc.attributes()[0] & 0b0000_0011, 1);
        assert_eq!((doc.attributes()[0] >> 2) & 0b11, 2);
        assert_eq!((doc.attributes()[0] >> 4) & 0b11, 3);
        assert_eq!((doc.attributes()[0] >> 6) & 0b11, 0);
    }

    #[test]
    fn dedupe_remaps_nametable_to_first_matching_tile() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.copy_tile(1, 9);
        doc.set_nametable_tile(5, 5, 9, 0);
        let count = doc.dedupe_tiles();

        assert!(count > 0);
        assert_eq!(doc.nametable()[5 + 5 * NAMETABLE_W], 1);
    }

    #[test]
    fn export_writes_kitaqfc_asset_set() {
        let doc = KitaqfcGraphicsDocument::new_demo();
        let out = env::temp_dir().join(format!(
            "kitaqfc_graphics_editor_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&out);
        let report = doc.export_raw_assets(&out, "smoke").unwrap();

        assert_eq!(fs::read(report.chr_path).unwrap().len(), CHR_ROM_SIZE);
        assert_eq!(fs::read(report.palette_path).unwrap().len(), PALETTE_SIZE);
        assert_eq!(
            fs::read(report.nametable_path).unwrap().len(),
            NAMETABLE_SIZE
        );
        assert_eq!(
            fs::read(report.attribute_path).unwrap().len(),
            ATTRIBUTE_SIZE
        );
        assert!(fs::read_to_string(report.manifest_path)
            .unwrap()
            .contains("smoke.chr"));
        assert_eq!(
            KitaqfcGraphicsDocument::load_project(out.join("smoke.fch")).unwrap(),
            doc
        );

        let _ = fs::remove_dir_all(out);
    }

    #[test]
    fn strict_chr_import_rejects_truncation_without_changing_document() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        let before = doc.clone();
        assert!(doc.try_load_chr_bytes(&vec![7; CHR_ROM_SIZE + 16]).is_err());
        assert!(doc.try_load_chr_bytes(&[7; 17]).is_err());
        assert_eq!(doc, before);
    }

    #[test]
    fn sprite_preview_uses_selected_table_and_signed_offsets() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.chr.fill(0);
        doc.metasprites = vec![MetaSpriteFrame::new(vec![MetaSpriteQuad::new(
            248, 248, 0, 0,
        )])];
        doc.set_tile_pixel(256, 0, 0, 3);
        assert!(doc.set_pattern_tables(0, 1));
        let (w, _, rgba) = doc.render_metasprite_rgba(0, 1);
        let index = (64 * w + 64) * 4;
        assert_eq!(&rgba[index..index + 4], &doc.rgba_for_index(4, 3));
        doc.set_pattern_tables(0, 0);
        let (_, _, other) = doc.render_metasprite_rgba(0, 1);
        assert_ne!(&rgba[index..index + 4], &other[index..index + 4]);
    }

    #[test]
    fn universal_background_and_sprite_priority_match_ppu() {
        let mut doc = KitaqfcGraphicsDocument::new_blank();
        doc.set_palette_entry(4, 48);
        assert_eq!(doc.rgba_for_index(1, 0), doc.rgba_for_index(0, 0));
        doc.fill_tile(1, 1);
        doc.fill_tile(2, 2);
        doc.set_metasprite_quad(0, 0, MetaSpriteQuad::new(0, 0, 1, 0));
        doc.set_metasprite_quad(0, 1, MetaSpriteQuad::new(0, 0, 2, 0));
        let (w, _, pixels) = doc.render_metasprite_rgba(0, 1);
        assert_eq!(
            &pixels[(72 * w + 72) * 4..(72 * w + 72) * 4 + 4],
            &doc.rgba_for_index(4, 1)
        );
    }

    #[test]
    fn palette_aliases_survive_raw_export_and_reload() {
        let mut doc = KitaqfcGraphicsDocument::new_blank();
        doc.set_palette_entry(0, 48);
        assert_eq!(doc.palette[16], 48);
        doc.set_palette_entry(20, 34);
        assert_eq!(doc.palette[4], 34);
        let mut reload = KitaqfcGraphicsDocument::new_blank();
        reload.load_palette_bytes(doc.palette());
        assert_eq!(doc.palette, reload.palette);
        assert_eq!(doc.render_nametable_rgba(), reload.render_nametable_rgba());
    }

    #[test]
    fn dedupe_preserves_both_pattern_table_previews() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.chr.fill(0);
        doc.fill_tile(5, 1);
        doc.fill_tile(261, 2);
        doc.copy_tile(261, 266);
        doc.set_pattern_tables(1, 1);
        doc.fill_nametable_rect(0, 0, 32, 30, 10, 0);
        doc.metasprites = vec![MetaSpriteFrame::new(vec![MetaSpriteQuad::new(0, 0, 10, 0)])];
        let bg = doc.render_nametable_rgba();
        let sp = doc.render_metasprite_rgba(0, 1);
        doc.dedupe_tiles();
        assert_eq!(bg, doc.render_nametable_rgba());
        assert_eq!(sp, doc.render_metasprite_rgba(0, 1));
        assert_eq!(doc.nametable[0], 5);
        assert_eq!(doc.metasprites[0].quads[0].tile, 5);
    }

    #[test]
    fn project_roundtrip_preserves_nondefault_tables_and_rejects_invalid_lengths() {
        let mut doc = KitaqfcGraphicsDocument::new_demo();
        doc.set_pattern_tables(1, 1);
        let path = env::temp_dir().join(format!(
            "kfc_project_roundtrip_{}.kfcproject",
            std::process::id()
        ));
        doc.save_project(&path).unwrap();
        assert_eq!(doc, KitaqfcGraphicsDocument::load_project(&path).unwrap());
        let mut json: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        json["document"]["nametable"] = serde_json::json!([0]);
        fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        assert!(KitaqfcGraphicsDocument::load_project(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
