# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA用于编辑KITAQFC的NES图块、名称表、调色板和8x8组合精灵。独立CLI与GUI共用编辑核心，不需要图形界面、GPU、PLITA安装或Python。

## 构建与运行

构建需要Rust 1.92或更高版本。Windows运行target/release/fchua.exe；Linux和macOS运行./target/release/fchua。使用--help查看语法。以下示例将程序写作fchua。请先创建输出文件的父目录。

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## 工作流程

新建空项目或导入PNG，检查报告和预览后再导出。--auto-palette以降低转换误差为目标选择四组NES背景调色板；省略时使用当前调色板。--project base.fch提供已有调色板和图案表设置。透明像素与当前公共背景色合成。导入会替换地图和选中的背景图案表，因此共用该表的精灵图形也会变化。

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## 命令索引

```text
new --out FILE.fch [--demo] [--force]
inspect FILE.fch
validate FILE.fch [--strict]
import-image IMAGE.png --out FILE.fch [--project BASE.fch] [--auto-palette] [--force]
import-raw --out FILE.fch [--project BASE.fch] [--chr FILE.chr]
           [--palette FILE.pal] [--nametable FILE.nam] [--attributes FILE.atr] [--force]
apply INPUT.fch --script EDITS.json --out OUTPUT.fch [--dry-run] [--force]
preview FILE.fch --out IMAGE.png [--view map|tiles|sprite|tile] [--scale 1..8] [--force]
export FILE.fch --out DIRECTORY [--prefix C_IDENTIFIER] [--force]
migrate OLD.kfcproject --out FILE.fch [--force]
```

## 批量编辑

脚本是JSON数组。所有操作验证成功后才更新文档；批处理失败不会写入文件。--dry-run只验证并报告。不明操作和字段均报错。图块操作使用绝对编号0..511，地图和精灵使用各自图案表内的0..255。

```json
[
  {"op":"pixel","tile":1,"x":2,"y":3,"color":1},
  {"op":"fill","tile":2,"color":0},
  {"op":"checker","tile":3,"a":1,"b":2},
  {"op":"line","tile":4,"x0":0,"y0":0,"x1":7,"y1":7,"color":3},
  {"op":"rect","tile":5,"x0":1,"y0":1,"x1":6,"y1":6,"color":2},
  {"op":"mirror","tile":1,"horizontal":true,"vertical":false},
  {"op":"rotate","tile":1},
  {"op":"copy","source":1,"target":6},
  {"op":"palette","index":1,"value":48},
  {"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0},
  {"op":"tables","background":0,"sprites":1},
  {"op":"sprite","frame":0,"index":0,"x":-8,"y":0,"tile":1,"attr":64},
  {"op":"clone_frame","frame":0},
  {"op":"remove_sprite","frame":1,"index":0},
  {"op":"shift_map","dx":2,"dy":0},
  {"op":"dedupe"}
]
```

`tile: 0..511; x/y (pixel): 0..7; color: 0..3; palette index: 0..31; value: 0..63`

`map: 32x30; palette: 0..3; tables: 0..1; frame: 0..255; index: 0..63`

`attr: palette bits 0..1, priority 0x20, flip H 0x40, flip V 0x80; reserved: 0x1c`

`shift_map: dx=-32..32, dy=-30..30; dx % 2 == 0, dy % 2 == 0`

## GUI与Python

本地PLITA GUI使用Roboto、Lucide图标分组菜单、悬停/按下/释放反馈和提示。File包含新建、打开、保存、另存为、PNG导入及导出。Ctrl+O/S/Shift+S分别打开/保存/另存为；Ctrl+Z/Y撤销/重做；Ctrl+C/V复制/粘贴图块。仅切换选区或工具不会标记为未保存。关闭、新建或切换项目时，若有编辑会先确认是否丢弃。

Map/Tile/Sprite/Check切换中央视图。B/E/F/L/R/I选择铅笔/橡皮/填充/直线/矩形/吸管，1..4选择颜色。一次拖动对应一次撤销，Escape取消。可用Fit、缩放按钮、滚轮、滚动条滑块和轨道。窄窗口用Tiles/Palette切换侧栏。先点调色板色块，再点下方NES颜色；颜色0修改公共背景。Sprite在8像素网格上放置图块，吸管选择部件，橡皮删除部件；菜单可添加/复制帧及翻转/删除选中部件。Check仅检查单帧，不代表整个场景或模拟器验证。

独立CLI包也包含可选Python绑定（Python 3.10以上），不依赖GUI、GPU或PLITA。在tools/fchua用pip构建，或安装附带的Windows x64 wheel。Controller支持项目读写、PNG转换、批量编辑及RAW/C导出；此版本GUI_AVAILABLE=False。本地PLITA版另外提供run_editor和ButtonInteraction。Controller编辑独立文档，不远程连接运行中的GUI；通过.fch交换。Python保存会替换已有文件，与CLI的--force保护不同。

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Local PLITA workspace
cargo run -p kitaqfc_graphics_editor --bin fchua_gui
python -m pip install ./examples/kitaqfc_graphics_editor
```

```python
import fchua
editor = fchua.Controller()
editor.new_blank()
editor.apply_commands('[{"op":"fill","tile":1,"color":2},{"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0}]')
editor.save_project("scene.fch")
editor.preview_png("preview.png", "map", 2)
editor.export_raw_assets("assets", "scene")
print(fchua.GUI_AVAILABLE, editor.snapshot().unique_tiles)
```

## 硬件限制与文件

.fch是带版本的JSON（format=fchua，version=1），保存8192字节CHR、32字节调色板、32x30地图、64字节属性、两个图案表选择和最多256帧。可以迁移旧.kfcproject；旧调色板映射不一致时以硬件属性为准。PNG尺寸必须为8的倍数，范围8x8..256x240。每个16x16块共用一组四色调色板。背景表最多256个图块；PNG导入保留0号空白块并拒绝超限图像。

导出.chr（8192）、.pal（32）、.nam（960）、.atr（64）、组合精灵JSON、所有帧的C/H数组、资源清单及.fch。C数组使用__prg_rom。游戏需自行将PPU图案表配置与项目匹配；资源包不生成配置代码。RAW CHR需为16字节整块且不超过8192字节，其他三种格式必须等于指定长度。NES颜色为0..63。每帧最多64部件；坐标-128..127，但x=-1被0xFF结束标记占用。只支持8x8精灵，同一扫描线超过8个部件会警告。

## 保护与错误

已有输出必须加--force，但输入与输出不能同路径。项目使用临时文件原子替换。export --out指定目录；--force替换同名资源，不删除无关文件。--prefix须为C标识符。成功输出一个JSON对象，错误向标准错误输出JSON。退出码0为成功，1为validate --strict发现警告，2为参数/项目/转换/读写错误。覆盖前请备份。不调用网络或AI生成服务。

原创代码采用MIT许可；依赖保留各自许可。参见LICENSE与THIRD_PARTY_NOTICES.md。
