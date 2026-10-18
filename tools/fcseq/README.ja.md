# FCSEQ 0.4.2 — 説明書

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

作者 **DAISUKE OBA**。**plita**を使用した汎用ファミコン音楽シーケンサーです。GUI・CLI・Python bindingsは編集モデル・音声・出力処理を共有します。空のプロジェクトから起動し、ゲームROMやBIOSは不要です。画面のラベルは英語で、説明書は9言語あります。

**0.4.2** — プレビュー・C出力・NSFのDMC DAC初期値を64に統一しました。DACとフィルターが安定してからプレビューを記録し、DPCM未使用時の三角波／ノイズの音量差と無音区間の立ち上がりノイズを修正しています。DPCM音色ごとの初期レベルは引き続き優先されます。FCQ・NSF・NSFeの形式変更はありません。

## ダウンロードと起動

[downloads/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads)からWindows x86_64、Linux x86_64、macOS Apple Silicon（ARM64）、macOS Intel（x86_64）の完全なZIPを選び、全体を展開してください。Windowsは`fcseq.exe`、Unixは`./fcseq`で起動します。端末用は`fcseq-cli`（Windowsでは`.exe`付き）です。[bin/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/bin)には単体実行ファイルもありますが、再配布時はライセンス類も必要です。

Windows 10/11はDirect3D 12、macOSはMetal、LinuxはVulkan/OpenGL・X11/Wayland・ALSAを利用します。Linux版はUbuntu 24.04でビルドします。古いディストリビューションではソースビルドが必要な場合があります。Ubuntuの実行時ライブラリーは`sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`で導入できます。GUIには対応GPUドライバー、試聴には音声デバイスが必要です。WAV出力は音声デバイス不要です。実行ファイルは未署名、macOS版は未公証です。OSに起動を止められた場合はソースとハッシュを確認し、OS標準の許可操作を使ってください。UnixでZIP展開後に実行権限がなければ`chmod +x fcseq fcseq-cli`を実行します。

Windowsでは[Microsoft Visual C++ x64再頒布可能パッケージ](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)も必要です（実行ファイルが`VCRUNTIME140.dll`を利用します）。不足する場合はMicrosoft公式の対応する最新版x64パッケージを導入してください。GUI/CLIはPythonにリンクしておらず、Pythonは任意のbindingsとラッパーを使う場合だけ必要です。

## 楽曲の作成と編集

SOUNDで音源を選択します。2A03はPulse 1/2・Triangle・Noise・DPCMの5音。VRC6はPulse 2音とSawを追加して8音、VRC7はFM 6音を追加して11音、FDSは波形1音を追加して6音、N163は波形最大8音を追加して13音です。1プロジェクトで使う拡張音源構成は1種類です。音符を失う音源変更・チャンネル削減は拒否します。

レーンをダブルクリックして音符追加。本体のドラッグで移動・移調、右端で長さ変更。Ctrl/Shiftクリックか範囲ドラッグで複数選択、Ctrl+Aで現在レーンの全選択。Ctrl+C/X/Vでコピー・切り取り・カーソル位置へ貼り付け、Ctrl+Dで複製、Deleteで削除、Ctrl+Z/Yで取り消し・やり直し。矢印で移動・移調、Shift+上下でオクターブ変更。Gでグリッド、Qで整列、Fで全体表示、Homeで先頭、+/-でズーム、Shift+ホイールで時間移動、UP/DOWNでチャンネル移動。Spaceで再生・停止、LOOPで反復、AUDITIONで選択音を試聴。MUTE/SOLOは出力にも反映します。SONGで最大16セクション・各256小節を管理し、名前・BPM・小節数はクリックして変更します。Ctrl+N/O/S/Shift+Sで新規・開く・保存・別名保存。重なりや範囲外の一括編集は全体を取り消します。回復用ファイルは実行ファイル横の`out/recovery_<process-id>.fcq`です。OPENで開いて別名保存してください。

## 音色と波形

SOUND > Instrument / waveform editorまたはINSTRUMENT / WAVEFORMを開き、対象チャンネルの音符を選択して編集します。AUDITION/UNDO/REDOも使えます。PulseのDutyは4段階、VRC6 Pulseは8段階。Triangleは固定32段階波形、Noiseは16周期と長短モードです。Pulse/Noiseの音量は0〜15、減衰0は一定音量。VRC6 Sawは6bit加算値0〜63を変え、桁あふれで音色も変わります。これらには自由描画用の波形RAMがありません。

FDSは共有64サンプル・6bit波形を編集できます。N163は4bit波形で初期値32サンプル×4、音符ごとに波形番号を指定し、長さは4単位です。共有容量は`256 - 16 × 使用チャンネル数`サンプル（8音で128）なので、長くする前に別波形を短くするかチャンネルを減らします。ドラッグ描画、SINE/PULSE/SAW/TRIANGLE、反転、回転に対応。音程・RAM・無効チャンネルの制限を検査します。VRC7のカスタムパッチ0は24項目（倍率、ADSR、帰還、音量、キースケーリング、トレモロ、ビブラート、正弦／半波整流）または8バイトを編集できます。プリセット1〜15は独自の近似音色で、実機音色ROMのダンプではありません。FDS変調テーブル／ハードウェアエンベロープの専用編集画面はありません。実機のアナログ音質との一致は保証しません。

## DPCM音色

SAMPLESまたはSOUND > Import audio fileでWAV・FLAC・Ogg Vorbis・MP3を読み込みます。ドロップも可能です。START/END（ms）、GAIN、NORMALIZE、REVERSE、RATE 0〜15を設定してAUDITION DPCMで確認。EXPORT .DMCで保存し、DPCMレーンへ音符を置いてインスペクターでSAMPLEと再生RATEを選びます。原音PCMを保持して加工を戻せます。入力最大60秒、ステレオはモノラル化、変換時にDC除去とローパス付きリサンプリングを行います。1音色`16n+1`バイト・最大4081バイト、最大64音色、全体16320バイト（FDSは8192）。超過する変更は拒否します。変換レートと音符の再生レートは別設定です。音色内ループと音階の自動補正はありません。

## FCQ・NSF/NSFe・C出力

標準拡張子は`.fcq`。旧`.fcseq`は拡張子を変更して開けます。音符文書は従来の`fcseq-2` JSONで、`fcseq-1`も読み込めます。NSF再生文書は`fcseq-3`です。

FILE > Import NSF、OPEN、ドロップ、起動引数からNSF/NSFeを読み込みます。PREV/NEXTや曲番号で選び、PLAY/STOP・LOOP・再生時間1〜600秒を操作します。曲長情報があれば使用し、なければ初期値30秒。再生前の音声生成は中止できます。FCQ保存時は元データ・曲番号・尺を埋め込みます。インポートは再生文書であり自動採譜ではありません。再出力は元コンテナーのバイト列を保持します。

編集曲のExport NSFは各セクションを1曲にし、音源・DPCMを含めます。NSFと同時に曲長・フェード0のNSFeを生成します。NSFの曲長情報を扱わないプレーヤーではNSFeを使ってください。セクション終了時に停止し、エディターのLOOPとは別です。入力はNSF version 1とNSFe、NTSCまたはNTSC対応デュアル、非バンク／4KiBバンク、2A03/DPCM・VRC6・VRC7・FDS・N163。PAL専用、NSF2実行拡張、MMC5、Sunsoft 5B、VT02は拒否します。戻らないINIT/PLAYは制限時間で停止します。

Export C + DPCMは`score.c`、`fcseq_score.h`、`fcseq_player.c`、セクションバイナリー、`.dmc`、`dpcm_bank.bin`、`fcseq_export.json`を出力します。ゲームROMの自動生成ではないため、音源対応マッパーとCPUから見えるコード・データ・DPCMの配置を設定してください。VRC6はmapper 24（26はアドレス線変換）、VRC7は85、N163は19、またはFDSを想定します。KITAQFCではドライバーinclude前に`FCSEQ_KITAQFC`を定義し、独自の書き込み関数は`FCSEQ_WRITE(address, value)`で接続できます。

## CLIとPython

以下は展開したフォルダーで実行します。セクション・チャンネル・音符・音色番号は0から、NSF曲番号は1からです。Python 3.10以降のabi3拡張を同梱しており、同じOS・CPU用のファイルを同じフォルダーから読み込みます。GUI/CLIの起動にPythonは不要です。API一覧は`fcseq_native.pyi`。CLIは音符の移動・移調・長さ変更・整列・複製・削除に対応し、波形編集はGUI/Pythonから行えます。

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## ビルドと検証

このソースフォルダーでRust 1.93.1を使い、下のコマンドを実行します。ローカル依存ソースはvendor/に同梱。Linuxの開発パッケージはリポジトリー直下の`.github/workflows/fcseq.yml`、macOSはXcode Command Line Tools、WindowsはMSVC Build Toolsが必要です。Pythonは任意。検証スクリプトはネイティブCLI/Python、WAV、NSF/NSFe往復、GUI起動を確認します。結果と範囲はVERIFICATION.mdに記載します。人によるマウス・キーボード操作や実機音質の認証ではありません。

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
python -c "import shutil,sys; from pathlib import Path; n={'win32':'fcseq_native.dll','darwin':'libfcseq_native.dylib'}.get(sys.platform,'libfcseq_native.so'); shutil.copy2(Path('target/release')/n,'fcseq_native.pyd' if sys.platform=='win32' else 'fcseq_native.so')"
```

## ライセンス

FCSEQ本体はMIT、Copyright (c) 2026 DAISUKE OBA。依存物には各ライセンスが適用されます。再配布時はLICENSE、THIRD_PARTY_NOTICES.md、licenses/、assets/、third_party_sources/を含めてください。MPL-2.0のSymphoniaソースを同梱し、フォント・アイコンの表記も保持しています。読み込んだ音楽・音声の権利は別です。

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
