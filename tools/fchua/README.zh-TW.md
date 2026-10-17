# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA用於編輯KITAQFC的NES圖塊、名稱表、調色盤與8x8組合精靈。獨立CLI和GUI共用編輯核心，不需要圖形介面、GPU、安裝plita或Python。

## 下載GUI版

下載下方Windows x64 ZIP，完整解壓縮後執行FCHUA.exe。請保留程式旁的assets/fonts與授權資料夾。執行不需要Rust或Python，但需要相容wgpu的圖形裝置與驅動程式。GUI標籤為英語，完整說明書提供上述九種語言。

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

透過File > New project新增，或Open project / raw開啟。選擇圖塊和調色盤，在Tile繪製並於Map放置。地圖調色盤以16x16像素為單位變更。透過對應選單選擇背景/精靈圖案表。以.fch儲存可編輯資料；PNG預覽是圖片而非專案。Export KITAQFC pack匯出遊戲資產。字型異常時請檢查ZIP是否完整解壓及PLITA_UI_FONT設定。無法開啟視窗時請更新顯示驅動或使用無介面CLI。

公開原始碼已包含所需的plita crate。在tools/fchua執行以下命令建置並啟動GUI，最後的專案路徑可省略。本次發行的GUI執行檔於Windows x64驗證。

Windows x64發行建置、軟體繪圖版面與輸入/狀態自動測試已通過。桌面自動化工具等待應用程式授權時逾時，因此實際視窗點擊和檔案對話框操作尚未驗證。參閱[驗證紀錄](VERIFICATION.md)。

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

## 建置與執行

建置需要Rust 1.92以上。Windows執行target/release/fchua.exe；Linux與macOS執行./target/release/fchua。使用--help查看語法。以下以fchua表示執行檔。請先建立輸出檔案的上層目錄。

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## 工作流程

建立空白專案或匯入PNG，檢查報告和預覽後再匯出。--auto-palette以降低轉換誤差為目標選擇四組NES背景調色盤；省略時使用目前調色盤。--project base.fch提供既有調色盤與圖案表設定。透明像素會與目前共用背景色合成。匯入會取代地圖與選取的背景圖案表，因此共用此表的精靈圖形也會改變。

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

## 批次編輯

腳本是JSON陣列。全部操作通過驗證才更新文件；批次失敗不寫入檔案。--dry-run只檢查和回報。不明操作或欄位均視為錯誤。圖塊操作使用絕對編號0..511；地圖與精靈使用各自圖案表內的0..255。

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

## GUI與Python

plita GUI使用Roboto、Lucide圖示群組選單、滑入/按下/釋放回饋與工具提示。File提供新增、開啟、儲存、另存新檔、PNG匯入及匯出。Ctrl+O/S/Shift+S為開啟/儲存/另存；Ctrl+Z/Y復原/重做；Ctrl+C/V複製/貼上圖塊。僅切換選取項或工具不會標記為未儲存。編輯後關閉、新增或切換專案會先確認是否捨棄。

Map/Tile/Sprite/Check切換中央檢視。B/E/F/L/R/I為鉛筆/橡皮擦/填滿/直線/矩形/滴管，1..4選色。一次拖曳對應一次復原，Escape取消。可用Fit、縮放按鈕、滾輪、捲軸滑塊與軌道。窄視窗以Tiles/Palette切換側欄。先點調色盤色塊，再點下方NES顏色；色0修改共用背景。Sprite在8像素格線上放圖塊，滴管選部件，橡皮擦刪除；選單可新增/複製影格及翻轉/刪除部件。Check只檢查單一影格，不代表完整場景或模擬器驗證。

獨立CLI套件也包含選用Python繫結（Python 3.10以上），不依賴GUI、GPU或plita。在tools/fchua以pip建置，或安裝附帶的Windows x64 wheel。Controller支援專案讀寫、PNG轉換、批次編輯與RAW/C匯出；此版本GUI_AVAILABLE=False。啟用gui,python功能的版本另外提供run_editor和ButtonInteraction。Controller編輯獨立文件，不遠端連線至執行中的GUI；透過.fch交換。Python儲存會取代既有檔案，與CLI的--force保護不同。

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Optional GUI-enabled Python build, also from tools/fchua
python -m pip install maturin
python -m maturin build --release --locked --features gui,python --out gui-wheels
# Install the wheel generated in gui-wheels instead of the headless wheel.
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

## 硬體限制與檔案

.fch為版本化JSON（format=fchua、version=1），保存8192位元組CHR、32位元組調色盤、32x30地圖、64位元組屬性、兩個圖案表選擇和最多256影格。可遷移舊.kfcproject；舊調色盤映射不一致時以硬體屬性優先。PNG尺寸須為8的倍數，範圍8x8..256x240。每個16x16區塊共用一組四色調色盤。背景表最多256圖塊，PNG匯入保留0號空白圖塊並拒絕超限影像。

匯出.chr（8192）、.pal（32）、.nam（960）、.atr（64）、組合精靈JSON、所有影格C/H陣列、資產清單及.fch。C陣列使用__prg_rom。遊戲須自行將PPU圖案表配置與專案一致；資產包不產生設定程式碼。RAW CHR須為16位元組整塊且最多8192位元組；其餘三種格式須符合精確長度。NES色碼為0..63。每影格最多64部件，座標-128..127，但x=-1保留給0xFF終止標記。僅支援8x8精靈；同一扫描線超過8部件會警告。

## 保護與錯誤

已有輸出需加--force，但輸入與輸出不得同一路徑。專案透過暫存檔原子取代。export --out為目錄；--force只取代同名資產，不刪除無關檔案。--prefix須為C識別字。成功輸出一個JSON物件，錯誤向標準錯誤輸出JSON。結束碼0為成功，1為validate --strict發現警告，2為參數/專案/轉換/讀寫錯誤。覆寫前請備份。不呼叫網路或AI生成服務。

原創程式碼採MIT授權，依賴保留各自授權。參閱LICENSE與THIRD_PARTY_NOTICES.md。
