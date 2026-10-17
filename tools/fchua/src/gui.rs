use super::*;
use plita::{prelude::*, DrawCommand};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::result::Result;

const BG: Color = Color::rgb8(26, 28, 31);
const PANEL: Color = Color::rgb8(34, 37, 41);
const LINE: Color = Color::rgb8(64, 69, 75);
const TEXT: Color = Color::rgb8(232, 236, 240);
const MUTED: Color = Color::rgb8(162, 174, 184);
const ACCENT: Color = Color::rgb8(70, 178, 154);
const WARN: Color = Color::rgb8(232, 180, 86);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Menu {
    File,
    Edit,
    Tile,
    Map,
    Sprite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Map,
    Tile,
    Sprite,
    Check,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
    Picker,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Menu(Menu),
    New,
    Open,
    Save,
    SaveAs,
    Import,
    Export,
    Preview,
    Undo,
    Redo,
    Tool(Tool),
    View(View),
    Sidebar(bool),
    Swatch(u8, u8),
    NesColor(u8),
    PrevTile,
    NextTile,
    CopyTile,
    PasteTile,
    MirrorH,
    MirrorV,
    Rotate,
    Invert,
    Dedupe,
    BgTable,
    SpriteTable,
    Grid,
    Zoom(i8),
    Fit,
    Shift(i32, i32),
    PrevFrame,
    NextFrame,
    CloneFrame,
    NewFrame,
    RemoveSprite,
    FlipSpriteH,
    FlipSpriteV,
}
#[derive(Clone, Copy)]
struct Control {
    action: Action,
    rect: Rect,
    label: &'static str,
    icon: Option<&'static str>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Atlas,
    Inspector,
    Canvas,
    Menu,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaintTarget {
    Tile,
    Map,
    Sprite,
}
struct Stroke {
    before: KitaqfcGraphicsDocument,
    target: PaintTarget,
    anchor: (usize, usize),
    last: (usize, usize),
}
#[derive(Clone, Copy)]
struct Layout {
    left: Rect,
    main: Rect,
    right: Option<Rect>,
    canvas: Rect,
    atlas: Option<Rect>,
    inspector: Option<Rect>,
    status: Rect,
}

pub struct Editor {
    size: Size,
    doc: KitaqfcGraphicsDocument,
    saved: KitaqfcGraphicsDocument,
    path: Option<PathBuf>,
    undo: Vec<KitaqfcGraphicsDocument>,
    redo: Vec<KitaqfcGraphicsDocument>,
    view: View,
    tool: Tool,
    menu: Option<Menu>,
    palette_tab: bool,
    grid: bool,
    zoom: u8,
    buttons: ButtonInteractionState<Action>,
    scrolls: [ScrollAreaState; 4],
    scroll_drag: Option<Target>,
    stroke: Option<Stroke>,
    pointer: Option<Point>,
    clipboard: Option<[u8; 16]>,
    quad: Option<usize>,
    status: String,
}
impl Editor {
    fn new(size: Size) -> Self {
        let doc = KitaqfcGraphicsDocument::new_demo();
        Self {
            size,
            saved: doc.clone(),
            doc,
            path: None,
            undo: vec![],
            redo: vec![],
            view: View::Map,
            tool: Tool::Pencil,
            menu: None,
            palette_tab: false,
            grid: true,
            zoom: 0,
            buttons: ButtonInteractionState::new(),
            scrolls: std::array::from_fn(|_| ScrollAreaState::new()),
            scroll_drag: None,
            stroke: None,
            pointer: None,
            clipboard: None,
            quad: None,
            status: "Ready".into(),
        }
    }
    fn dirty(&self) -> bool {
        !self.doc.same_content(&self.saved)
    }
    fn layout(&self) -> Layout {
        let w = self.size.width.max(1.0);
        let h = self.size.height.max(1.0);
        let side = 240.0_f32.min((w * 0.35).max(100.0));
        let wide = w >= 1120.0;
        let left = Rect::new(0.0, 48.0, side, (h - 74.0).max(1.0));
        let right = wide.then(|| Rect::new(w - 252.0, 48.0, 252.0, (h - 74.0).max(1.0)));
        let main = Rect::new(
            side,
            48.0,
            (right.map(|r| r.x).unwrap_or(w) - side).max(1.0),
            (h - 74.0).max(1.0),
        );
        let body = |r: Rect| {
            Rect::new(
                r.x + 8.0,
                132.0,
                (r.width - 16.0).max(1.0),
                (h - 166.0).max(1.0),
            )
        };
        Layout {
            left,
            main,
            right,
            canvas: body(main),
            atlas: (!self.palette_tab || wide).then(|| body(left)),
            inspector: right
                .map(body)
                .or_else(|| self.palette_tab.then(|| body(left))),
            status: Rect::new(0.0, h - 26.0, w, 26.0),
        }
    }
    fn index(t: Target) -> usize {
        match t {
            Target::Atlas => 0,
            Target::Inspector => 1,
            Target::Canvas => 2,
            Target::Menu => 3,
        }
    }
    fn canvas_size(&self) -> Size {
        let base = match self.view {
            View::Tile => (8.0, 8.0),
            View::Map => (256.0, 240.0),
            View::Sprite => (160.0, 160.0),
            View::Check => (360.0, 460.0),
        };
        if self.view == View::Check {
            return Size::new(base.0, base.1);
        }
        let area = self.layout().canvas;
        let fit = ((area.width - 16.0) / base.0)
            .min((area.height - 16.0) / base.1)
            .max(0.1);
        let scale = if self.zoom == 0 {
            fit
        } else {
            self.zoom as f32 * if self.view == View::Tile { 16.0 } else { 1.0 }
        };
        Size::new(base.0 * scale, base.1 * scale)
    }
    fn area(&self, t: Target) -> Option<(Rect, Size)> {
        let l = self.layout();
        match t {
            Target::Atlas => l.atlas.map(|r| (r, Size::new(208.0, 464.0))),
            Target::Inspector => l
                .inspector
                .map(|r| (r, Size::new((r.width - 16.0).max(1.0), 690.0))),
            Target::Canvas => Some((l.canvas, self.canvas_size())),
            Target::Menu => self
                .menu_rect()
                .map(|r| (r, Size::new(r.width, self.menu_items().len() as f32 * 32.0))),
        }
    }
    fn origin(&self, t: Target) -> Point {
        let (r, s) = self.area(t).unwrap();
        self.scrolls[Self::index(t)].content_origin(r, s)
    }
    fn canvas_rect(&self) -> Rect {
        let p = self.origin(Target::Canvas);
        let s = self.canvas_size();
        Rect::new(p.x, p.y, s.width, s.height)
    }
    fn inspector_tile(&self) -> Option<Rect> {
        let (r, _) = self.area(Target::Inspector)?;
        let p = self.origin(Target::Inspector);
        let side = (r.width - 20.0).min(216.0);
        Some(Rect::new(p.x, p.y, side, side))
    }
    fn menu_items(&self) -> Vec<(Action, &'static str, &'static str)> {
        match self.menu {
            Some(Menu::File) => vec![
                (Action::New, "New project", "file-plus"),
                (Action::Open, "Open project / raw...", "folder-open"),
                (Action::Save, "Save  Ctrl+S", "save"),
                (Action::SaveAs, "Save as...", "save"),
                (Action::Import, "Import PNG...", "image"),
                (Action::Export, "Export KITAQFC pack...", "package"),
                (Action::Preview, "Export preview PNG...", "image"),
            ],
            Some(Menu::Edit) => vec![
                (Action::Undo, "Undo  Ctrl+Z", "undo-2"),
                (Action::Redo, "Redo  Ctrl+Y", "redo-2"),
                (Action::CopyTile, "Copy tile  Ctrl+C", "copy"),
                (Action::PasteTile, "Paste tile  Ctrl+V", "clipboard-paste"),
            ],
            Some(Menu::Tile) => vec![
                (Action::Tool(Tool::Pencil), "Pencil  B", "pencil"),
                (Action::Tool(Tool::Eraser), "Eraser  E", "eraser"),
                (Action::Tool(Tool::Fill), "Flood fill  F", "paint-bucket"),
                (Action::Tool(Tool::Line), "Line  L", "minus"),
                (Action::Tool(Tool::Rect), "Rectangle  R", "square"),
                (Action::Tool(Tool::Picker), "Pick color  I", "pipette"),
                (Action::MirrorH, "Mirror horizontal", "flip-horizontal-2"),
                (Action::MirrorV, "Mirror vertical", "flip-vertical-2"),
                (Action::Rotate, "Rotate clockwise", "rotate-cw"),
                (Action::Invert, "Invert indices", "contrast"),
                (Action::Dedupe, "Remap duplicate tiles", "layers"),
            ],
            Some(Menu::Map) => vec![
                (Action::BgTable, "Background table 0 / 1", "layers"),
                (Action::Grid, "Toggle grid", "grid-2x2"),
                (Action::Shift(-2, 0), "Shift left 16px", "arrow-left"),
                (Action::Shift(2, 0), "Shift right 16px", "arrow-right"),
                (Action::Shift(0, -2), "Shift up 16px", "arrow-up"),
                (Action::Shift(0, 2), "Shift down 16px", "arrow-down"),
            ],
            Some(Menu::Sprite) => vec![
                (Action::SpriteTable, "Sprite table 0 / 1", "layers"),
                (Action::PrevFrame, "Previous frame", "chevron-left"),
                (Action::NextFrame, "Next frame", "chevron-right"),
                (Action::NewFrame, "New frame", "file-plus"),
                (Action::CloneFrame, "Duplicate frame", "copy"),
                (Action::RemoveSprite, "Remove selected quad", "trash-2"),
                (
                    Action::FlipSpriteH,
                    "Flip selected quad H",
                    "flip-horizontal-2",
                ),
                (
                    Action::FlipSpriteV,
                    "Flip selected quad V",
                    "flip-vertical-2",
                ),
            ],
            None => vec![],
        }
    }
    fn menu_rect(&self) -> Option<Rect> {
        let menu = self.menu?;
        let i = match menu {
            Menu::File => 0,
            Menu::Edit => 1,
            Menu::Tile => 2,
            Menu::Map => 3,
            Menu::Sprite => 4,
        };
        let w = 248.0_f32.min(self.size.width);
        Some(Rect::new(
            (112.0 + i as f32 * 76.0).min((self.size.width - w).max(0.0)),
            46.0,
            w,
            (self.menu_items().len() as f32 * 32.0)
                .min((self.size.height - 82.0).max(32.0))
                .min(288.0),
        ))
    }
    fn controls(&self) -> Vec<Control> {
        let l = self.layout();
        let mut out = Vec::new();
        let mut add = |action, rect, label, icon| {
            out.push(Control {
                action,
                rect,
                label,
                icon,
            })
        };
        for (i, (m, label)) in [
            (Menu::File, "File"),
            (Menu::Edit, "Edit"),
            (Menu::Tile, "Tile"),
            (Menu::Map, "Map"),
            (Menu::Sprite, "Sprite"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                Action::Menu(m),
                Rect::new(112.0 + i as f32 * 76.0, 9.0, 72.0, 32.0),
                label,
                None,
            );
        }
        for (i, (a, label, icon)) in [
            (Action::Open, "Open  Ctrl+O", "folder-open"),
            (Action::Save, "Save  Ctrl+S", "save"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                a,
                Rect::new(self.size.width - 78.0 + i as f32 * 36.0, 9.0, 32.0, 32.0),
                label,
                Some(icon),
            );
        }
        for (i, (v, label)) in [
            (View::Map, "Map"),
            (View::Tile, "Tile"),
            (View::Sprite, "Sprite"),
            (View::Check, "Check"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                Action::View(v),
                Rect::new(l.main.x + 8.0 + i as f32 * 70.0, 58.0, 66.0, 28.0),
                label,
                None,
            );
        }
        for (i, (t, label, icon)) in [
            (Tool::Pencil, "Pencil (B)", "pencil"),
            (Tool::Eraser, "Eraser (E)", "eraser"),
            (Tool::Fill, "Flood fill (F)", "paint-bucket"),
            (Tool::Line, "Line (L)", "minus"),
            (Tool::Rect, "Rectangle (R)", "square"),
            (Tool::Picker, "Picker (I)", "pipette"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                Action::Tool(t),
                Rect::new(l.main.x + 8.0 + i as f32 * 32.0, 94.0, 30.0, 30.0),
                label,
                Some(icon),
            );
        }
        for (i, (a, label, icon)) in [
            (Action::Zoom(-1), "Zoom out", "zoom-out"),
            (Action::Fit, "Fit canvas", "maximize"),
            (Action::Zoom(1), "Zoom in", "zoom-in"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                a,
                Rect::new(l.main.right() - 108.0 + i as f32 * 32.0, 94.0, 30.0, 30.0),
                label,
                Some(icon),
            );
        }
        if l.right.is_none() {
            add(
                Action::Sidebar(false),
                Rect::new(8.0, 58.0, 88.0, 28.0),
                "Tiles",
                None,
            );
            add(
                Action::Sidebar(true),
                Rect::new(100.0, 58.0, 112.0, 28.0),
                "Palette",
                None,
            );
        }
        for (i, (a, label, icon)) in [
            (Action::PrevTile, "Previous tile", "chevron-left"),
            (Action::NextTile, "Next tile", "chevron-right"),
        ]
        .into_iter()
        .enumerate()
        {
            add(
                a,
                Rect::new(8.0 + i as f32 * 34.0, 94.0, 30.0, 30.0),
                label,
                Some(icon),
            );
        }
        if let Some((r, _)) = self.area(Target::Inspector) {
            let p = self.origin(Target::Inspector);
            let side = self.inspector_tile().unwrap().height;
            for g in 0..8u8 {
                for c in 0..4u8 {
                    let rect = Rect::new(
                        p.x + 38.0 + c as f32 * 34.0,
                        p.y + side + 36.0 + g as f32 * 28.0,
                        28.0,
                        23.0,
                    );
                    if rect.intersects(r) {
                        add(Action::Swatch(g, c), rect, "Select palette color", None);
                    }
                }
            }
            for v in 0..64u8 {
                let rect = Rect::new(
                    p.x + (v % 8) as f32 * 23.0,
                    p.y + side + 294.0 + (v / 8) as f32 * 20.0,
                    20.0,
                    18.0,
                );
                if rect.intersects(r) {
                    add(Action::NesColor(v), rect, "Set NES color", None);
                }
            }
        }
        out
    }
    fn hit(&self, p: Point) -> Option<Action> {
        let menu = self.menu_rect();
        let l = self.layout();
        if let Some(r) = menu {
            if r.contains(p) && p.x < r.right() - 12.0 {
                let index = ((p.y - r.y + self.scrolls[3].offset().y) / 32.0) as usize;
                return self.menu_items().get(index).map(|item| item.0);
            }
        }
        self.controls()
            .into_iter()
            .rev()
            .find(|b| {
                if !b.rect.contains(p) {
                    return false;
                }
                if menu.is_some() {
                    return p.y < 46.0 && matches!(b.action, Action::Menu(_));
                }
                if matches!(b.action, Action::Swatch(..) | Action::NesColor(_)) {
                    return l.inspector.is_some_and(|r| r.contains(p));
                }
                true
            })
            .map(|b| b.action)
    }
    fn active(&self, a: Action) -> bool {
        match a {
            Action::Menu(m) => self.menu == Some(m),
            Action::View(v) => self.view == v,
            Action::Tool(t) => self.tool == t,
            Action::Sidebar(v) => self.palette_tab == v,
            Action::Swatch(g, c) => {
                self.doc.selected_palette_group == g && self.doc.selected_color == c
            }
            Action::Grid => self.grid,
            _ => false,
        }
    }
    fn remember(&mut self, before: KitaqfcGraphicsDocument) {
        if !before.same_content(&self.doc) {
            self.undo.push(before);
            if self.undo.len() > 128 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }
    fn confirm_discard(&self) -> bool {
        !self.dirty()
            || MessageDialog::new()
                .set_title("FCHUA")
                .set_description("Discard unsaved changes?")
                .set_level(MessageLevel::Warning)
                .set_buttons(MessageButtons::OkCancel)
                .show()
                == MessageDialogResult::Ok
    }
    fn save(&mut self, save_as: bool) {
        let path = if save_as || self.path.is_none() {
            FileDialog::new()
                .add_filter("FCHUA project", &["fch"])
                .set_file_name("graphics.fch")
                .save_file()
                .map(|p| p.with_extension("fch"))
        } else {
            self.path.clone()
        };
        if let Some(p) = path {
            match self.doc.save_project(&p) {
                Ok(()) => {
                    self.saved = self.doc.clone();
                    self.status = format!("Saved {}", p.display());
                    self.path = Some(p);
                }
                Err(e) => self.status = e.to_string(),
            }
        }
    }
    fn load(&mut self, path: &Path) -> Result<(), String> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "png" {
            let (d, r) = image_io::import_png(&self.doc, path, true)?;
            let before = self.doc.clone();
            self.doc = d;
            self.remember(before);
            self.status = format!(
                "PNG: {} tiles, MSE {:.1}",
                r.unique_tiles, r.mean_squared_error
            );
            return Ok(());
        }
        if ["fch", "kfcproject"].contains(&ext.as_str()) {
            let d = KitaqfcGraphicsDocument::load_project(path).map_err(|e| e.to_string())?;
            if !self.confirm_discard() {
                return Ok(());
            }
            self.doc = d;
            self.saved = self.doc.clone();
            self.path = Some(path.with_extension("fch"));
            self.undo.clear();
            self.redo.clear();
            self.quad = None;
        } else {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            let mut d = self.doc.clone();
            match ext.as_str() {
                "chr" => d.try_load_chr_bytes(&bytes).map_err(|e| e.to_string())?,
                "pal" if bytes.len() == 32 && bytes.iter().all(|c| *c < 64) => {
                    d.load_palette_bytes(&bytes)
                }
                "nam" if bytes.len() == 960 => d.load_nametable_bytes(&bytes),
                "atr" | "attr" if bytes.len() == 64 => d.load_attribute_bytes(&bytes),
                _ => return Err("Unsupported file or invalid raw length".into()),
            }
            let before = self.doc.clone();
            self.doc = d;
            self.remember(before);
        }
        self.status = format!("Loaded {}", path.display());
        Ok(())
    }
    fn action(&mut self, a: Action) {
        if let Action::Menu(m) = a {
            self.menu = if self.menu == Some(m) { None } else { Some(m) };
            self.scrolls[3] = ScrollAreaState::new();
            return;
        }
        self.menu = None;
        let before = self.doc.clone();
        match a {
            Action::New => {
                if self.confirm_discard() {
                    self.doc = KitaqfcGraphicsDocument::new_blank();
                    self.saved = self.doc.clone();
                    self.path = None;
                    self.undo.clear();
                    self.redo.clear();
                }
                return;
            }
            Action::Open => {
                if let Some(p) = FileDialog::new()
                    .add_filter(
                        "FCHUA / graphics",
                        &["fch", "kfcproject", "chr", "pal", "nam", "atr"],
                    )
                    .pick_file()
                {
                    if let Err(e) = self.load(&p) {
                        self.status = e;
                    }
                }
                return;
            }
            Action::Save => {
                self.save(false);
                return;
            }
            Action::SaveAs => {
                self.save(true);
                return;
            }
            Action::Import => {
                if let Some(p) = FileDialog::new().add_filter("PNG", &["png"]).pick_file() {
                    if let Err(e) = self.load(&p) {
                        self.status = e;
                    }
                }
                return;
            }
            Action::Export => {
                if let Some(p) = FileDialog::new().pick_folder() {
                    let existing = [
                        "scene.chr",
                        "scene.pal",
                        "scene.nam",
                        "scene.atr",
                        "scene.fch",
                        "scene.metasprite.json",
                        "scene_pack_manifest.json",
                        "scene_assets.c",
                        "scene_assets.h",
                    ]
                    .iter()
                    .any(|name| p.join(name).exists());
                    if existing
                        && MessageDialog::new()
                            .set_title("FCHUA")
                            .set_description("Replace existing scene asset files in this folder?")
                            .set_buttons(MessageButtons::OkCancel)
                            .show()
                            != MessageDialogResult::Ok
                    {
                        return;
                    }
                    self.status = match self.doc.export_raw_assets(&p, "scene") {
                        Ok(_) => format!("Exported {}", p.display()),
                        Err(e) => e.to_string(),
                    };
                }
                return;
            }
            Action::Preview => {
                if let Some(p) = FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .set_file_name("preview.png")
                    .save_file()
                {
                    self.status = match image_io::preview_png(&self.doc, &p, "map", 2) {
                        Ok(()) => format!("Exported {}", p.display()),
                        Err(e) => e,
                    };
                }
                return;
            }
            Action::Undo => {
                if let Some(d) = self.undo.pop() {
                    self.redo.push(self.doc.clone());
                    self.doc = d;
                }
                return;
            }
            Action::Redo => {
                if let Some(d) = self.redo.pop() {
                    self.undo.push(self.doc.clone());
                    self.doc = d;
                }
                return;
            }
            Action::Tool(t) => self.tool = t,
            Action::View(v) => {
                self.view = v;
                self.zoom = 0;
                self.scrolls[2] = ScrollAreaState::new();
            }
            Action::Sidebar(v) => self.palette_tab = v,
            Action::Swatch(g, c) => {
                self.doc.selected_palette_group = g;
                self.doc.selected_color = c;
            }
            Action::NesColor(v) => {
                let index = if self.doc.selected_color == 0 {
                    0
                } else {
                    self.doc.selected_palette_group as usize * 4 + self.doc.selected_color as usize
                };
                self.doc.set_palette_entry(index, v);
            }
            Action::PrevTile => self.doc.selected_tile = self.doc.selected_tile.saturating_sub(1),
            Action::NextTile => self.doc.selected_tile = (self.doc.selected_tile + 1).min(511),
            Action::CopyTile => {
                self.clipboard = Some(
                    self.doc.chr[self.doc.selected_tile * 16..self.doc.selected_tile * 16 + 16]
                        .try_into()
                        .unwrap(),
                );
                self.status = "Tile copied".into();
            }
            Action::PasteTile => {
                if let Some(b) = self.clipboard {
                    let start = self.doc.selected_tile * 16;
                    self.doc.chr[start..start + 16].copy_from_slice(&b);
                }
            }
            Action::MirrorH => {
                self.doc.mirror_tile(self.doc.selected_tile, true, false);
            }
            Action::MirrorV => {
                self.doc.mirror_tile(self.doc.selected_tile, false, true);
            }
            Action::Rotate => {
                self.doc.rotate_tile_cw(self.doc.selected_tile);
            }
            Action::Invert => {
                self.doc.invert_tile(self.doc.selected_tile);
            }
            Action::Dedupe => {
                self.status = format!(
                    "{} duplicate tile references remapped",
                    self.doc.dedupe_tiles()
                );
            }
            Action::BgTable => self.doc.background_pattern_table ^= 1,
            Action::SpriteTable => self.doc.sprite_pattern_table ^= 1,
            Action::Grid => self.grid = !self.grid,
            Action::Zoom(d) => {
                self.zoom = (self.zoom as i8 + d).clamp(1, 8) as u8;
            }
            Action::Fit => {
                self.zoom = 0;
                self.scrolls[2] = ScrollAreaState::new();
            }
            Action::Shift(dx, dy) => {
                let _ = commands::apply(&mut self.doc, &[commands::Command::ShiftMap { dx, dy }]);
            }
            Action::PrevFrame => {
                self.doc.current_frame = self.doc.current_frame.saturating_sub(1);
                self.quad = None;
            }
            Action::NextFrame => {
                self.doc.current_frame =
                    (self.doc.current_frame + 1).min(self.doc.metasprites.len() - 1);
                self.quad = None;
            }
            Action::NewFrame => {
                if self.doc.metasprites.len() < 256 {
                    self.doc.add_metasprite_frame();
                    self.quad = None;
                }
            }
            Action::CloneFrame => {
                let frame = self.doc.current_frame;
                let _ = commands::apply(&mut self.doc, &[commands::Command::CloneFrame { frame }]);
                self.quad = None;
            }
            Action::RemoveSprite => {
                if let Some(i) = self.quad.take() {
                    if i < self.doc.metasprites[self.doc.current_frame].quads.len() {
                        self.doc.metasprites[self.doc.current_frame].quads.remove(i);
                    }
                }
            }
            Action::FlipSpriteH | Action::FlipSpriteV => {
                if let Some(i) = self.quad {
                    if let Some(q) = self.doc.metasprites[self.doc.current_frame]
                        .quads
                        .get_mut(i)
                    {
                        q.attr ^= if a == Action::FlipSpriteH { 0x40 } else { 0x80 };
                    }
                }
            }
            Action::Menu(_) => {}
        }
        self.remember(before);
        self.clamp_scrolls();
    }
    fn clamp_scrolls(&mut self) {
        for t in [
            Target::Atlas,
            Target::Inspector,
            Target::Canvas,
            Target::Menu,
        ] {
            if let Some((r, s)) = self.area(t) {
                let state = &mut self.scrolls[Self::index(t)];
                state.set_offset(state.offset(), r, s);
            }
        }
    }
    fn paint_hit(&self, p: Point) -> Option<(PaintTarget, usize, usize)> {
        let l = self.layout();
        if l.inspector.is_some_and(|r| r.contains(p)) {
            if let Some(r) = self.inspector_tile() {
                if r.contains(p) {
                    return Some((
                        PaintTarget::Tile,
                        ((p.x - r.x) / r.width * 8.0) as usize,
                        ((p.y - r.y) / r.height * 8.0) as usize,
                    ));
                }
            }
        }
        let r = self.canvas_rect();
        if !l.canvas.contains(p) || !r.contains(p) {
            return None;
        }
        let (target, w, h) = match self.view {
            View::Tile => (PaintTarget::Tile, 8, 8),
            View::Map => (PaintTarget::Map, 32, 30),
            View::Sprite => (PaintTarget::Sprite, 20, 20),
            View::Check => return None,
        };
        Some((
            target,
            ((p.x - r.x) / r.width * w as f32) as usize,
            ((p.y - r.y) / r.height * h as f32) as usize,
        ))
    }
    fn paint(&mut self, x: usize, y: usize) {
        let Some(stroke) = self.stroke.as_ref() else {
            return;
        };
        let target = stroke.target;
        let (ax, ay) = stroke.anchor;
        let (lx, ly) = stroke.last;
        let color = if self.tool == Tool::Eraser {
            0
        } else {
            self.doc.selected_color
        };
        if matches!(self.tool, Tool::Line | Tool::Rect) {
            self.doc = stroke.before.clone();
        }
        if target == PaintTarget::Tile {
            let t = self.doc.selected_tile;
            match self.tool {
                Tool::Pencil | Tool::Eraser => {
                    self.doc
                        .draw_line_tile(t, lx as i32, ly as i32, x as i32, y as i32, color);
                }
                Tool::Fill => {
                    self.doc.flood_fill_tile(t, x, y, color);
                }
                Tool::Line => {
                    self.doc
                        .draw_line_tile(t, ax as i32, ay as i32, x as i32, y as i32, color);
                }
                Tool::Rect => {
                    self.doc.fill_tile_rect(t, ax, ay, x, y, color);
                }
                Tool::Picker => self.doc.selected_color = self.doc.tile_pixel(t, x, y),
            }
        } else if target == PaintTarget::Map {
            let tile = if self.tool == Tool::Eraser {
                0
            } else {
                self.doc.selected_tile as u8
            };
            let group = self.doc.selected_palette_group % 4;
            match self.tool {
                Tool::Picker => {
                    self.doc.selected_tile = self.doc.nametable[y * 32 + x] as usize
                        + self.doc.background_pattern_table as usize * 256;
                    self.doc.selected_palette_group = self.doc.palette_map[y * 32 + x];
                }
                Tool::Rect => {
                    self.doc.fill_nametable_rect(
                        x.min(ax),
                        y.min(ay),
                        x.abs_diff(ax) + 1,
                        y.abs_diff(ay) + 1,
                        tile,
                        group,
                    );
                }
                Tool::Fill => {
                    let old = self.doc.nametable[y * 32 + x];
                    let mut seen = [false; 960];
                    let mut queue = vec![(x, y)];
                    while let Some((xx, yy)) = queue.pop() {
                        let i = yy * 32 + xx;
                        if seen[i] || self.doc.nametable[i] != old {
                            continue;
                        }
                        seen[i] = true;
                        self.doc.set_nametable_tile(xx, yy, tile, group);
                        if xx > 0 {
                            queue.push((xx - 1, yy));
                        }
                        if xx < 31 {
                            queue.push((xx + 1, yy));
                        }
                        if yy > 0 {
                            queue.push((xx, yy - 1));
                        }
                        if yy < 29 {
                            queue.push((xx, yy + 1));
                        }
                    }
                }
                _ => {
                    let start = if self.tool == Tool::Line {
                        (ax, ay)
                    } else {
                        (lx, ly)
                    };
                    for (xx, yy) in grid_line(start, (x, y)) {
                        self.doc.set_nametable_tile(xx, yy, tile, group);
                    }
                }
            }
        } else {
            let (dx, dy) = (
                (x as i16 * 8 - 72) as i8 as u8,
                (y as i16 * 8 - 72) as i8 as u8,
            );
            let f = self.doc.current_frame;
            let index = self.doc.metasprites[f]
                .quads
                .iter()
                .position(|q| q.dx == dx && q.dy == dy);
            if self.tool == Tool::Eraser {
                if let Some(i) = index {
                    self.doc.metasprites[f].quads.remove(i);
                    self.quad = None;
                }
            } else if self.tool == Tool::Picker {
                self.quad = index;
            } else {
                let i = index.unwrap_or(self.doc.metasprites[f].quads.len());
                if self.doc.set_metasprite_quad(
                    f,
                    i,
                    MetaSpriteQuad::new(
                        dx,
                        dy,
                        self.doc.selected_tile as u8,
                        self.doc.selected_palette_group % 4,
                    ),
                ) {
                    self.quad = Some(i);
                }
            }
        }
        if let Some(s) = &mut self.stroke {
            s.last = (x, y);
        }
    }
    fn end_stroke(&mut self, cancel: bool) {
        if let Some(s) = self.stroke.take() {
            if cancel {
                self.doc = s.before;
            } else {
                self.remember(s.before);
            }
        }
    }
    fn pointer_down(&mut self, p: Point) {
        for t in [
            Target::Menu,
            Target::Atlas,
            Target::Inspector,
            Target::Canvas,
        ] {
            if self.menu.is_some() && t != Target::Menu {
                continue;
            }
            if let Some((r, s)) = self.area(t) {
                if self.scrolls[Self::index(t)]
                    .begin_scrollbar_drag(r, s, p, ScrollbarStyle::default())
                    .is_some()
                {
                    self.scroll_drag = Some(t);
                    return;
                }
            }
        }
        if let Some(a) = self.hit(p) {
            self.buttons.handle_pointer_down(Some(a));
            return;
        }
        if self.menu.take().is_some() {
            return;
        }
        if let Some((r, _)) = self.area(Target::Atlas) {
            let o = self.origin(Target::Atlas);
            if r.contains(p) && p.x >= o.x && p.x < o.x + 208.0 && p.y >= o.y && p.y < o.y + 448.0 {
                self.doc.selected_tile =
                    (((p.y - o.y) / 14.0) as usize * 16 + ((p.x - o.x) / 13.0) as usize).min(511);
                return;
            }
        }
        if let Some((target, x, y)) = self.paint_hit(p) {
            self.stroke = Some(Stroke {
                before: self.doc.clone(),
                target,
                anchor: (x, y),
                last: (x, y),
            });
            self.paint(x, y);
        }
    }
    fn pointer_move(&mut self, p: Point) {
        self.pointer = Some(p);
        if let Some(t) = self.scroll_drag {
            if let Some((r, s)) = self.area(t) {
                self.scrolls[Self::index(t)].drag_scrollbar(r, s, p, ScrollbarStyle::default());
            }
            return;
        }
        self.buttons.handle_pointer_move(self.hit(p));
        if let Some(stroke) = &self.stroke {
            if !matches!(self.tool, Tool::Fill | Tool::Picker) {
                if let Some((t, x, y)) = self.paint_hit(p) {
                    if t == stroke.target && stroke.last != (x, y) {
                        self.paint(x, y);
                    }
                }
            }
        }
    }
    fn key(&mut self, key: &str, ctrl: bool, shift: bool) {
        let key = key.split('|').next().unwrap_or(key).to_ascii_uppercase();
        let key = key.trim_start_matches("NAMED(").trim_end_matches(')');
        if key == "ESCAPE" {
            self.menu = None;
            self.end_stroke(true);
            self.buttons.clear();
            self.scroll_drag = None;
            return;
        }
        if ctrl {
            let a = match key {
                "O" => Action::Open,
                "S" if shift => Action::SaveAs,
                "S" => Action::Save,
                "Z" => Action::Undo,
                "Y" => Action::Redo,
                "C" => Action::CopyTile,
                "V" => Action::PasteTile,
                _ => return,
            };
            self.end_stroke(false);
            self.action(a);
            return;
        }
        let a = match key {
            "B" => Action::Tool(Tool::Pencil),
            "E" => Action::Tool(Tool::Eraser),
            "F" => Action::Tool(Tool::Fill),
            "L" => Action::Tool(Tool::Line),
            "R" => Action::Tool(Tool::Rect),
            "I" => Action::Tool(Tool::Picker),
            "G" => Action::Grid,
            "ARROWLEFT" => Action::PrevTile,
            "ARROWRIGHT" => Action::NextTile,
            "DELETE" => Action::RemoveSprite,
            "1" | "2" | "3" | "4" => Action::Swatch(
                self.doc.selected_palette_group,
                key.parse::<u8>().unwrap() - 1,
            ),
            _ => return,
        };
        self.end_stroke(false);
        self.action(a);
    }
}
impl PlitaProgram for Editor {
    fn close_requested(&mut self) -> bool {
        self.end_stroke(false);
        self.confirm_discard()
    }
    fn resize(&mut self, size: Size) -> UiResponse {
        self.size = size;
        self.clamp_scrolls();
        UiResponse::redraw()
    }
    fn handle_event(&mut self, event: UiEvent) -> UiResponse {
        match event {
            UiEvent::PointerDown {
                position,
                button: PointerButton::Primary,
            } => self.pointer_down(position),
            UiEvent::PointerMove { position } => self.pointer_move(position),
            UiEvent::PointerUp {
                position,
                button: PointerButton::Primary,
            } => {
                if let Some(t) = self.scroll_drag.take() {
                    self.scrolls[Self::index(t)].end_scrollbar_drag();
                }
                self.end_stroke(false);
                let release = self.buttons.handle_pointer_up(self.hit(position));
                if let Some(a) = release.activated {
                    self.action(a);
                }
            }
            UiEvent::PointerWheel {
                position,
                delta_x,
                delta_y,
            } => {
                for t in [
                    Target::Menu,
                    Target::Atlas,
                    Target::Inspector,
                    Target::Canvas,
                ] {
                    if self.menu.is_some() && t != Target::Menu {
                        continue;
                    }
                    if let Some((r, s)) = self.area(t) {
                        if r.contains(position) {
                            self.scrolls[Self::index(t)].wheel(delta_x, delta_y, r, s);
                            break;
                        }
                    }
                }
            }
            UiEvent::KeyDown {
                key,
                ctrl,
                meta,
                shift,
                ..
            } => self.key(&key, ctrl || meta, shift),
            UiEvent::FileDropped { path } => {
                if let Err(e) = self.load(&path) {
                    self.status = e;
                }
            }
            UiEvent::FocusChanged { focused: false } => {
                self.end_stroke(false);
                self.buttons.clear();
                self.menu = None;
                self.scroll_drag = None;
                for s in &mut self.scrolls {
                    s.end_scrollbar_drag();
                }
            }
            _ => return UiResponse::none(),
        }
        UiResponse::redraw()
    }
    fn draw_list(&self) -> DrawList {
        linear_geometry(self.draw_srgb())
    }
}

impl Editor {
    fn draw_srgb(&self) -> DrawList {
        let l = self.layout();
        let mut list = DrawList::new(BG);
        list.fill_rect(Rect::new(0.0, 0.0, self.size.width, 48.0), PANEL);
        draw_text(&mut list, 14.0, 16.0, "FCHUA", 1.8, ACCENT);
        for r in [Some(l.left), l.right].into_iter().flatten() {
            list.fill_rect(r, PANEL);
            list.stroke_rect(r, 1.0, LINE);
        }
        if l.right.is_some() {
            draw_text(&mut list, 12.0, 66.0, "CHR TILES", 1.2, TEXT);
        }
        draw_text(
            &mut list,
            86.0,
            103.0,
            &format!("{:03} / 511", self.doc.selected_tile),
            1.0,
            MUTED,
        );
        if let Some(r) = l.right {
            draw_text(&mut list, r.x + 12.0, 66.0, "TILE / PALETTE", 1.2, TEXT);
            draw_text(
                &mut list,
                r.x + 12.0,
                105.0,
                &format!(
                    "BG {}   SPR {}",
                    self.doc.background_pattern_table, self.doc.sprite_pattern_table
                ),
                1.05,
                MUTED,
            );
        }
        if let Some((r, s)) = self.area(Target::Atlas) {
            let p = self.origin(Target::Atlas);
            list.push_clip(r);
            for tile in 0..512 {
                let rect = Rect::new(
                    p.x + (tile % 16) as f32 * 13.0,
                    p.y + (tile / 16) as f32 * 14.0,
                    12.0,
                    12.0,
                );
                let (_, _, pixels) = self.doc.render_tile_rgba_scaled(
                    tile,
                    self.doc.selected_palette_group,
                    1,
                    false,
                );
                list.image_rgba8(rect, 8, 8, pixels);
                if tile == self.doc.selected_tile {
                    list.stroke_rect(rect, 2.0, ACCENT);
                }
            }
            list.pop_clip();
            self.scrolls[0].draw_scrollbars(&mut list, r, s, ScrollbarStyle::default());
        }
        if let Some((r, s)) = self.area(Target::Inspector) {
            let p = self.origin(Target::Inspector);
            list.push_clip(r);
            let tile = self.inspector_tile().unwrap();
            let (w, h, data) = self.doc.render_tile_rgba_scaled(
                self.doc.selected_tile,
                self.doc.selected_palette_group,
                24,
                self.grid,
            );
            list.image_rgba8(tile, w as u32, h as u32, data);
            draw_text(
                &mut list,
                p.x,
                p.y + tile.height + 13.0,
                &format!(
                    "Tile {:03}  /  Color {}",
                    self.doc.selected_tile, self.doc.selected_color
                ),
                1.05,
                TEXT,
            );
            for g in 0..8 {
                draw_text(
                    &mut list,
                    p.x,
                    p.y + tile.height + 42.0 + g as f32 * 28.0,
                    &format!("{}{}", if g < 4 { "B" } else { "S" }, g % 4),
                    1.05,
                    MUTED,
                );
            }
            draw_text(
                &mut list,
                p.x,
                p.y + tile.height + 273.0,
                "NES COLOR",
                1.05,
                TEXT,
            );
            for c in self
                .controls()
                .into_iter()
                .filter(|c| matches!(c.action, Action::Swatch(..) | Action::NesColor(_)))
            {
                self.draw_control(&mut list, c);
            }
            list.pop_clip();
            self.scrolls[1].draw_scrollbars(&mut list, r, s, ScrollbarStyle::default());
        }
        list.push_clip(l.canvas);
        let r = self.canvas_rect();
        if self.view == View::Check {
            let report = self.doc.analysis_report();
            let mut rows = vec![
                format!("CHR: {} bytes / 512 tiles", report.chr_bytes),
                format!("Unique patterns: {}", report.unique_tiles),
                format!("Duplicate patterns: {}", report.duplicate_tiles),
                format!("Blank tiles: {}", report.empty_tiles),
                format!("Map tile IDs: {}", report.nametable_used_tiles),
                format!(
                    "Frames: {}   Max OAM: {}",
                    self.doc.metasprites.len(),
                    report.metasprite_max_quads
                ),
            ];
            if report.warnings.is_empty() {
                rows.push("Hardware checks: no warnings".into());
            } else {
                rows.extend(report.warnings);
            }
            for (i, row) in rows.iter().enumerate() {
                draw_text(
                    &mut list,
                    r.x + 12.0,
                    r.y + 16.0 + i as f32 * 30.0,
                    row,
                    1.15,
                    if i > 5 { WARN } else { TEXT },
                );
            }
        } else {
            let (w, h, data) = match self.view {
                View::Tile => self.doc.render_tile_rgba_scaled(
                    self.doc.selected_tile,
                    self.doc.selected_palette_group,
                    1,
                    false,
                ),
                View::Map => self.doc.render_nametable_rgba(),
                _ => self.doc.render_metasprite_rgba(self.doc.current_frame, 1),
            };
            list.image_rgba8(r, w as u32, h as u32, data);
            if self.grid {
                let (cols, rows) = match self.view {
                    View::Tile => (8, 8),
                    View::Map => (32, 30),
                    _ => (20, 20),
                };
                for x in 0..=cols {
                    list.fill_rect(
                        Rect::new(r.x + r.width * x as f32 / cols as f32, r.y, 1.0, r.height),
                        Color::rgba8(100, 110, 120, 100),
                    );
                }
                for y in 0..=rows {
                    list.fill_rect(
                        Rect::new(r.x, r.y + r.height * y as f32 / rows as f32, r.width, 1.0),
                        Color::rgba8(100, 110, 120, 100),
                    );
                }
            }
            if self.view == View::Sprite {
                if let Some(q) = self
                    .quad
                    .and_then(|i| self.doc.metasprites[self.doc.current_frame].quads.get(i))
                {
                    list.stroke_rect(
                        Rect::new(
                            r.x + (72.0 + q.dx as i8 as f32) * r.width / 160.0,
                            r.y + (72.0 + q.dy as i8 as f32) * r.height / 160.0,
                            r.width / 20.0,
                            r.height / 20.0,
                        ),
                        2.0,
                        ACCENT,
                    );
                }
            }
        }
        list.pop_clip();
        let (_, s) = self.area(Target::Canvas).unwrap();
        self.scrolls[2].draw_scrollbars(&mut list, l.canvas, s, ScrollbarStyle::default());
        for c in self.controls().into_iter().filter(|c| {
            c.rect.y < 132.0 && !matches!(c.action, Action::Swatch(..) | Action::NesColor(_))
        }) {
            if self.menu.is_none()
                || c.rect.y < 46.0
                || matches!(
                    c.action,
                    Action::View(_)
                        | Action::Tool(_)
                        | Action::Zoom(_)
                        | Action::Fit
                        | Action::Sidebar(_)
                        | Action::PrevTile
                        | Action::NextTile
                )
            {
                self.draw_control(&mut list, c);
            }
        }
        list.fill_rect(l.status, PANEL);
        list.push_clip(l.status);
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled.fch");
        draw_text(
            &mut list,
            10.0,
            l.status.y + 7.0,
            &format!(
                "{}{}  |  {}  |  Frame {}/{}  |  {}",
                if self.dirty() { "* " } else { "" },
                name,
                if self.zoom == 0 {
                    "Fit".to_string()
                } else {
                    format!("{}x", self.zoom)
                },
                self.doc.current_frame + 1,
                self.doc.metasprites.len(),
                self.status
            ),
            1.0,
            MUTED,
        );
        list.pop_clip();
        if let Some(r) = self.menu_rect() {
            list.fill_rect(r, PANEL);
            list.stroke_rect(r, 1.0, LINE);
            list.push_clip(r);
            let y = r.y - self.scrolls[3].offset().y;
            for (i, (a, label, icon)) in self.menu_items().into_iter().enumerate() {
                self.draw_control(
                    &mut list,
                    Control {
                        action: a,
                        rect: Rect::new(r.x, y + i as f32 * 32.0, r.width - 12.0, 32.0),
                        label,
                        icon: Some(icon),
                    },
                );
            }
            list.pop_clip();
            let (_, s) = self.area(Target::Menu).unwrap();
            self.scrolls[3].draw_scrollbars(&mut list, r, s, ScrollbarStyle::default());
        } else if self.stroke.is_none() {
            if let Some(p) = self.pointer {
                if let Some(c) = self
                    .controls()
                    .into_iter()
                    .find(|c| c.rect.contains(p) && c.icon.is_some())
                {
                    let w = measure_text_width(c.label, 1.05) + 20.0;
                    let r = Rect::new(
                        c.rect.x.min((self.size.width - w - 4.0).max(0.0)),
                        c.rect.bottom() + 5.0,
                        w,
                        28.0,
                    );
                    list.fill_rect(r, PANEL);
                    list.stroke_rect(r, 1.0, LINE);
                    draw_text(&mut list, r.x + 10.0, r.y + 8.0, c.label, 1.05, TEXT);
                }
            }
        }
        list
    }
    fn draw_control(&self, list: &mut DrawList, c: Control) {
        let visual = self.buttons.visual_state(&c.action, self.active(c.action));
        let fill = match visual {
            ButtonVisualState::Normal => PANEL,
            ButtonVisualState::Hovered => Color::rgb8(58, 66, 73),
            ButtonVisualState::Pressed => Color::rgb8(28, 100, 86),
            ButtonVisualState::Active => Color::rgb8(39, 81, 75),
        };
        let swatch = match c.action {
            Action::Swatch(g, i) => Some(self.doc.rgba_for_index(g, i)),
            Action::NesColor(i) => {
                let [r, g, b] = NES_PALETTE_RGB[i as usize];
                Some([r, g, b, 255])
            }
            _ => None,
        };
        if let Some([r, g, b, a]) = swatch {
            list.fill_rect(c.rect, Color::rgba8(r, g, b, a));
            list.stroke_rect(
                c.rect,
                if self.active(c.action) { 2.0 } else { 1.0 },
                if visual != ButtonVisualState::Normal {
                    TEXT
                } else {
                    LINE
                },
            );
            return;
        }
        list.fill_rect(c.rect, fill);
        if self.active(c.action) {
            list.fill_rect(
                Rect::new(c.rect.x, c.rect.bottom() - 2.0, c.rect.width, 2.0),
                ACCENT,
            );
        }
        if let Some(icon) = c.icon {
            draw_lucide_icon(
                list,
                Rect::new(c.rect.x + 7.0, c.rect.y + 7.0, 18.0, 18.0),
                icon,
                TEXT,
            );
            if c.rect.width > 40.0 {
                draw_text(list, c.rect.x + 34.0, c.rect.y + 10.0, c.label, 1.05, TEXT);
            }
        } else {
            draw_text(list, c.rect.x + 8.0, c.rect.y + 9.0, c.label, 1.1, TEXT);
            if matches!(c.action, Action::Menu(_)) {
                draw_lucide_icon(
                    list,
                    Rect::new(c.rect.right() - 16.0, c.rect.y + 10.0, 12.0, 12.0),
                    "chevron-down",
                    MUTED,
                );
            }
        }
    }
}
fn grid_line(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (a.0 as i32, a.1 as i32);
    let (tx, ty) = (b.0 as i32, b.1 as i32);
    let dx = (tx - x).abs();
    let dy = -(ty - y).abs();
    let sx = if x < tx { 1 } else { -1 };
    let sy = if y < ty { 1 } else { -1 };
    let mut e = dx + dy;
    let mut out = vec![];
    loop {
        out.push((x as usize, y as usize));
        if x == tx && y == ty {
            break;
        }
        let twice = 2 * e;
        if twice >= dy {
            e += dy;
            x += sx;
        }
        if twice <= dx {
            e += dx;
            y += sy;
        }
    }
    out
}
fn linear_geometry(source: DrawList) -> DrawList {
    let convert = |mut c: Color| {
        for v in [&mut c.r, &mut c.g, &mut c.b] {
            *v = if *v <= 0.04045 {
                *v / 12.92
            } else {
                ((*v + 0.055) / 1.055).powf(2.4)
            };
        }
        c
    };
    let mut out = DrawList::new(convert(source.clear_color));
    for command in source.commands() {
        let mut c = command.clone();
        if let DrawCommand::FillRect { color, .. } = &mut c {
            *color = convert(*color);
        }
        if let DrawCommand::FillTriangles { vertices } = &mut c {
            *vertices = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    v.color = convert(v.color);
                    v
                })
                .collect::<Vec<_>>()
                .into();
        }
        out.push(c);
    }
    out
}
fn portable_font_path(
    executable_dir: Option<&Path>,
    source_dir: &Path,
    file: &str,
) -> Option<PathBuf> {
    executable_dir
        .into_iter()
        .map(|p| p.join("assets/fonts").join(file))
        .chain(std::iter::once(source_dir.join("assets/fonts").join(file)))
        .find(|p| p.is_file())
}

fn configure_portable_fonts() {
    let executable_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    for (variable, file) in [
        ("PLITA_UI_FONT", "Roboto[wdth,wght].ttf"),
        ("PLITA_MEMORY_FONT", "RobotoMono-Regular.ttf"),
    ] {
        if env::var_os(variable).is_some() {
            continue;
        }
        if let Some(path) = portable_font_path(
            executable_dir.as_deref(),
            Path::new(env!("CARGO_MANIFEST_DIR")),
            file,
        ) {
            env::set_var(variable, path);
        }
    }
}

pub fn run_editor_with_initial_asset(initial: Option<PathBuf>) -> plita::Result<()> {
    configure_portable_fonts();
    App::new()
        .title("FCHUA - KITAQFC Graphics Editor")
        .window_size(1360, 820)
        .continuous_redraw(false)
        .run_program(move |size| {
            let mut app = Editor::new(size);
            if let Some(p) = initial {
                if let Err(e) = app.load(&p) {
                    app.status = e;
                }
            }
            app
        })
}
pub fn run_editor_from_args() -> Result<(), Box<dyn std::error::Error>> {
    run_editor_with_initial_asset(env::args_os().nth(1).map(PathBuf::from))?;
    Ok(())
}

#[cfg(test)]
#[path = "snapshot.rs"]
mod snapshot;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_fonts_prefer_executable_assets_and_fall_back_to_source() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("unpacked");
        let source = temp.path().join("source");
        let name = "Roboto[wdth,wght].ttf";
        for root in [&executable, &source] {
            fs::create_dir_all(root.join("assets/fonts")).unwrap();
        }
        let source_font = source.join("assets/fonts").join(name);
        fs::write(&source_font, b"source-font").unwrap();
        assert_eq!(
            portable_font_path(Some(&executable), &source, name),
            Some(source_font)
        );
        let portable_font = executable.join("assets/fonts").join(name);
        fs::write(&portable_font, b"portable-font").unwrap();
        assert_eq!(
            portable_font_path(Some(&executable), &source, name),
            Some(portable_font)
        );
        assert_eq!(
            portable_font_path(Some(&executable), &source, "missing.ttf"),
            None
        );
    }

    fn click(app: &mut Editor, a: Action) {
        let r = app
            .controls()
            .into_iter()
            .find(|c| c.action == a)
            .unwrap()
            .rect;
        let p = Point::new(r.x + 10.0, r.y + 10.0);
        app.pointer_down(p);
        app.handle_event(UiEvent::PointerUp {
            position: p,
            button: PointerButton::Primary,
        });
    }
    #[test]
    fn buttons_use_shared_release_state_and_menus_block_paint() {
        let mut a = Editor::new(Size::new(1360.0, 820.0));
        let p = Point::new(120.0, 20.0);
        a.pointer_move(p);
        assert_eq!(
            a.buttons.visual_state(&Action::Menu(Menu::File), false),
            ButtonVisualState::Hovered
        );
        a.pointer_down(p);
        assert_eq!(a.menu, None);
        a.handle_event(UiEvent::PointerUp {
            position: p,
            button: PointerButton::Primary,
        });
        assert_eq!(a.menu, Some(Menu::File));
        let d = a.doc.clone();
        a.pointer_down(Point::new(600.0, 300.0));
        assert_eq!(a.doc, d);
        assert_eq!(a.menu, None);
    }
    #[test]
    fn edits_are_one_undo_and_selection_is_clean() {
        let mut a = Editor::new(Size::new(1360.0, 820.0));
        a.action(Action::NextTile);
        a.action(Action::Swatch(2, 3));
        assert!(!a.dirty());
        a.action(Action::View(View::Tile));
        let r = a.canvas_rect();
        a.pointer_down(Point::new(r.x + 2.0, r.y + 2.0));
        a.pointer_move(Point::new(r.x + r.width * 0.8, r.y + 2.0));
        a.end_stroke(false);
        assert_eq!(a.undo.len(), 1);
        a.action(Action::Undo);
        assert!(!a.dirty());
    }
    #[test]
    fn layout_and_icons_and_snapshots() {
        let path = env::var_os("FCHUA_UI_SNAPSHOTS").map(PathBuf::from);
        if let Some(p) = &path {
            fs::create_dir_all(p).unwrap();
        }
        for (w, h) in [
            (900.0, 600.0),
            (1024.0, 600.0),
            (1360.0, 820.0),
            (1920.0, 1080.0),
        ] {
            let mut a = Editor::new(Size::new(w, h));
            let l = a.layout();
            assert!(l.main.right() <= w);
            assert!(l.canvas.bottom() <= l.status.y);
            assert_eq!(l.atlas.unwrap().y, l.canvas.y);
            for c in a.controls() {
                if let Some(icon) = c.icon {
                    assert!(lucide_icon_names().any(|i| i == icon), "{icon}");
                }
                assert!(c.rect.right() <= w);
            }
            if let Some(p) = &path {
                snapshot::save(&a.draw_srgb(), a.size, &p.join(format!("{w}-{h}.png"))).unwrap();
                a.menu = Some(Menu::Tile);
                snapshot::save(&a.draw_srgb(), a.size, &p.join(format!("{w}-menu.png"))).unwrap();
            }
        }
    }
    #[test]
    fn scrollbars_drag_and_palette_hits_are_clipped() {
        let mut a = Editor::new(Size::new(900.0, 600.0));
        click(&mut a, Action::Sidebar(true));
        let (r, s) = a.area(Target::Inspector).unwrap();
        let geo = a.scrolls[1].geometry(r, s, ScrollbarStyle::default());
        let _ = geo;
        a.handle_event(UiEvent::PointerWheel {
            position: Point::new(r.x + 10.0, r.y + 10.0),
            delta_x: 0.0,
            delta_y: -200.0,
        });
        assert!(a.scrolls[1].offset().y > 0.0);
        assert!(!matches!(
            a.hit(Point::new(r.x + 50.0, r.y - 1.0)),
            Some(Action::Swatch(..))
        ));
        a.pointer_down(Point::new(r.right() - 4.0, r.bottom() - 8.0));
        assert_eq!(a.scroll_drag, Some(Target::Inspector));
        assert_eq!(default_font_family(), "Roboto");
    }
}
