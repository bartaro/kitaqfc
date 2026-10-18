# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUAはKITAQFC向けのNESタイル・ネームテーブル・パレット・8×8メタスプライトを編集します。独立CLIはGUIと同じ編集コアを使い、画面・GPU・plitaのインストール・Pythonを必要としません。

## GUI版のダウンロード

下のWindows x64用ZIPをダウンロードし、全体を展開してFCHUA.exeを起動してください。assets/fontsとライセンスフォルダを実行ファイルと一緒に保持します。実行にRust・Pythonは不要です。wgpuに対応するGPUとドライバが必要です。GUIは日本語・英語を切り替えられます。説明書は上記9言語を用意しています。

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

「ファイル > 新規作成」で作成し、「プロジェクト・素材を開く」で読み込みます。タイルとパレットを選び、「タイル」で描き、「マップ」に配置します。マップのパレットは16×16ピクセル単位で変わります。背景／スプライトのパターンテーブルは各メニューで選択します。編集可能な状態は.fchで保存してください。PNGプレビューは画像であり、プロジェクトではありません。「KITAQFC素材パックを書き出す」でゲーム用資産を出力します。フォントが異なる場合はZIP全体の展開とPLITA_UI_FONTの設定を確認してください。起動できない場合はGPUドライバを更新するか、ヘッドレスCLIを利用してください。

必要なplitaのクレートも公開ソースに含まれます。tools/fchuaで以下を実行するとGUIをビルド・起動できます。最後のプロジェクトパスは省略可能です。このリリースでGUI実行ファイルを検証した環境はWindows x64です。

右上の言語アイコン、または「編集 > 日本語 / English」で表示言語をすぐに切り替えられます。選択は次回起動時にも復元されます。初回はOSの言語に従い、日本語環境では日本語で起動します。環境変数FCHUA_LANGUAGE=jaまたはenを指定すると保存済み設定より優先されます。language.txtの保存先はWindowsでは%APPDATA%/FCHUA、macOSでは~/Library/Application Support/FCHUA、Linuxでは$XDG_CONFIG_HOME/fchua (未設定時は~/.config/fchua)です。FCHUA_CONFIG_DIRで保存フォルダを指定できます。基本フォントはRobotoのままで、日本語の文字には同梱のNoto Sans JPを使います。PLITA_UI_FONT_FALLBACKで代替フォントを指定できます。言語を変えても作業内容・Undo履歴は保持されます。CLIコマンドとPython API名は変更されません。

Windows x64でリリースビルド、ソフトウェア描画、入力・状態の自動テストに合格しています。実ウィンドウの作成は確認できましたが、画面取得が再試行でもタイムアウトしたため、実画面の描画内容・クリック・ファイルダイアログ操作は未検証です。[検証記録](VERIFICATION.md)を参照してください。

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

## ビルドと起動

ビルドにはRust 1.92以降が必要です。Windowsではtarget/release/fchua.exe、Linux・macOSでは./target/release/fchuaを実行します。--helpで構文を確認できます。以下では実行ファイルをfchuaと表記します。出力先の親ディレクトリは事前に作成してください。

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## 基本の流れ

空のプロジェクトを作るか、PNGを取り込みます。レポートとプレビューを確認してから出力してください。--auto-paletteは変換誤差を減らすようにNES背景パレットを4組選び、省略時はプロジェクトのパレットを使います。--project base.fchで既存パレットとパターンテーブル設定を指定できます。透明部分は現在の共通背景色に合成されます。取り込みはマップと選択中の背景テーブルを置換するため、同じテーブルを参照するスプライトの絵も変わります。

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## コマンド一覧

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

## 一括編集

スクリプトはJSON配列です。すべての操作を検証してから編集結果を反映します。途中で失敗した場合は何も保存しません。--dry-runは検査とレポートのみ行います。不明な操作名やフィールドはエラーです。タイル操作の番号は0～511、マップ・スプライトの番号は選択中テーブル内の0～255です。

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

## GUIとPython

GUIはplita・Robotoを使用し、Lucideアイコン付きグループメニュー、ホバー・押下・リリース表示、ツールチップを備えます。「ファイル」に新規・読み込み・保存・別名保存・PNG取り込み・出力があります。Ctrl+Oで開く、Ctrl+Sで保存、Ctrl+Shift+Sで別名保存、Ctrl+Z/YでUndo/Redo、Ctrl+C/Vでタイルのコピー／貼り付けです。タイルやツールの選択だけでは未保存になりません。編集後の終了・プロジェクト切り替え・新規作成時は破棄前に確認します。

「マップ／タイル／スプライト／検査」で中央の表示を切り替えます。B/E/F/L/R/Iは鉛筆／消しゴム／塗りつぶし／直線／矩形／スポイト、1～4は描画色です。1回のドラッグが1回のUndoになり、Escapeで取り消せます。全体表示・ズームアイコン、ホイール、スクロールバーのつまみ・トラックを利用できます。狭い画面ではTiles/Paletteでサイドバーを切り替えます。パレット色を選んでから下のNES色をクリックして変更します。色0は共通背景色です。Spriteでは8ピクセル格子にタイルを置き、スポイトで部品を選択、消しゴムで削除します。Spriteメニューからフレーム追加・複製、部品の反転・削除ができます。Checkはフレーム単位の走査線警告であり、シーン全体やエミュレータでの検証ではありません。

独立CLIパッケージにもPython bindings（Python 3.10以降）を同梱します。GUI・GPU・plitaは不要です。tools/fchuaからpipでビルドするか、同梱Windows x64 wheelをインストールしてください。Controllerでプロジェクト入出力・PNG変換・一括編集・RAW/C出力が可能です。この版はGUI_AVAILABLE=Falseです。gui,python featureを有効にした版にはrun_editorとButtonInteractionもあります。Controllerは起動中GUIではなく独立文書を編集するため、.fchで受け渡してください。Pythonの保存は既存ファイルを置換します。CLIの--force保護とは異なります。

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

## ハードウェア制約とファイル

.fchはバージョン付きJSON（format=fchua、version=1）です。CHR 8192バイト、パレット32バイト、32×30マップ、属性64バイト、背景／スプライトのテーブル選択、最大256フレームを保存します。旧.kfcprojectも移行可能です。旧パレットマップに矛盾がある場合はハードウェア属性を優先します。PNGは8の倍数で8×8～256×240です。16×16ピクセルごとに4色パレットを1組使います。背景テーブルは256タイルで、PNG取り込みではタイル0を空白用に予約し、上限超過を拒否します。

出力は.chr（8192）・.pal（32）・.nam（960）・.atr（64）、メタスプライトJSON、全フレームのC/H配列、パックマニフェスト、.fchです。C配列はKITAQFCの__prg_romを使用します。PPUのパターンテーブル設定はゲーム側でプロジェクトに合わせてください。パックはゲームの設定コードを生成しません。RAW入力はCHRが16バイト単位で最大8192バイト、他の3形式は指定サイズと一致する必要があります。NES色は0～63です。部品は1フレーム64個まで、座標は符号付き-128～127ですがx=-1は終端0xFFのため使用不可です。対応するスプライトモードは8×8のみです。同一走査線の部品が8個を超えると警告します。

## 保護機能とエラー

既存出力の置換には--forceが必要です。ただし入力と出力を同じパスにはできません。プロジェクトは一時ファイルから原子的に置換保存します。exportの--outはディレクトリで、--forceは同名の出力資産を置換しますが無関係のファイルは削除しません。--prefixにはC識別子を指定します。成功時はJSONを1件、エラー時は標準エラーへJSONを出力します。終了コードは0=成功、1=validate --strictの警告、2=引数・文書・変換・入出力エラーです。置換前にバックアップしてください。ネットワークや画像生成AIは呼び出しません。

オリジナルコードはMITです。依存ライブラリはそれぞれのライセンスに従います。LICENSEとTHIRD_PARTY_NOTICES.mdを参照してください。
