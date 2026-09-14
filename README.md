# KITAQFC

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [KITAQFC](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [KITAQFC](https://bartaro.github.io/kitaq-docs/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [KITAQFC](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [KITAQFC](https://bartaro.github.io/kitaq-docs/zh-CN/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [KITAQFC](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Español | [KITAQFC](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Português (Brasil) | [KITAQFC](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/pt/fc-library.html) |
| Français | [KITAQFC](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Deutsch | [KITAQFC](https://bartaro.github.io/kitaq-docs/de/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->


[English](#english) | [日本語](#japanese) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

<a name="english"></a>

## English

C compiler and support libraries for original NES/Famicom/FDS homebrew software, derived from KITAQGB and NORCAL.

Public preview: APIs and behavior may change.

### Development philosophy

KITAQFC supports development that respects the NES/Famicom hardware and can be checked in small, reproducible steps. Build the ROM, exercise it in KUROSAKI, inspect the evidence with SARAKURA, and repeat after each correction. Keep measured results separate from behavior that has not yet been tested.

<!-- development-prompt:en:start -->
### Game development prompt

Fill in the requirements, then give the complete prompt to your AI assistant. It covers implementation, emulator testing, SARAKURA analysis and retesting.

[Read the reference example in the HTML manual](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html#loop-prompts)

<details>
<summary>Show the complete prompt</summary>

#### Game development with KITAQFC, KUROSAKI and SARAKURA

Fill in the requirements and give this entire document to the AI assistant. Commands assume sibling repositories named `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` and `kitaq-docs`, with your `game-gb` or `game-fc` project beside them. Run commands from their parent directory; adapt paths to the actual environment.

##### Requirements

- Game title: &lt;fill in&gt;
- Genre and core gameplay: &lt;fill in&gt;
- Controls, success and failure conditions: &lt;fill in&gt;
- Required screens, stages, enemies and items: &lt;fill in&gt;
- Visual style, music and sound effects: &lt;fill in; identify any supplied assets&gt;
- Saving, communication, peripherals and other requirements: &lt;fill in, or none&gt;
- Project directory: &lt;fill in&gt;
- Redistribution requirements: &lt;for example, original code and assets suitable for MIT publication&gt;

- Target: &lt;NES/Famicom cartridge or FDS&gt;
- Mapper, ROM size and mirroring: &lt;specify, or select from the requirements&gt;
- Video standard and performance: &lt;for example, NTSC and 60 gameplay updates per second&gt;

##### Task

Implement the game with KITAQFC and its libraries. Use KUROSAKI for execution and debugging, and SARAKURA to organize diagnostics and compare results before and after a fix.

Repeat this cycle until the acceptance criteria are met: make the specification concrete → implement a small change → build → apply inputs and observe → investigate the cause → fix → retest under the same conditions. A plan, a code listing or a successful compilation is not completion.

###### Establish the environment and acceptance criteria

1. Read workspace instructions, tool READMEs, HTML manuals, and the headers and implementations of the libraries you will use. Record executable paths and versions or SHA-256 hashes. Verify commands against actual `--help` output and APIs against source.
2. Define measurable acceptance criteria for inputs, images, audio, progression and update frequency. Examples: pressing and releasing START begins the game; a collision removes one life; pausing silences the intended audio and resuming restores playback.
3. Ask only about material ambiguities. Make ordinary reversible implementation decisions autonomously. Do not weaken requirements or acceptance criteria.
4. First run a small supplied sample through the compiler, emulator and SARAKURA. This checks the tool connection, not completion of the requested game.

###### Implement a small playable slice

- Start with NROM for a small game; choose MMC3 or another mapper when banking or size requires it. Verify required board features using `inspect-rom`, `mapper-info`, `audit-board` and the implementation, rather than relying on the mapper name alone.
- Plan PRG/CHR size, CHR-ROM or CHR-RAM, mirroring, fixed banks, interrupt vectors and save RAM. Check headers against actual placement after optimizations or banking changes. `--nes-local-ram` uses internal CPU RAM in `$0000–$07FF`; avoid overlaps with zero page, stack, OAM buffers and runtime/library storage.
- Use the KITAQFC C dialect, FC libraries and `void main(void)`. Do not assume GB APIs are compatible. Check implementation providers and include the required `.c` files; some header entries are declarations only.
- Account for PPU registers, NMI, OAM DMA, per-scanline sprite limits, scrolling, mirroring, attribute tables and APU/DMC behavior. Transfer-queue free space is not PPU VRAM capacity; budget the work done per NMI.
- Convert the supplied original `ascii.c` font to FC CHR for letters, digits and symbols. Verify CHR, palettes, nametables and attributes. For FDS, separately verify disk access, saving and BIOS requirements; do not assume cartridge boot conditions.

- First connect boot, title, a controllable player, success or failure, and restart. Then expand the game.
- Keep editable graphics, music and sound-effect sources and their generation steps. Verify that the build actually consumes their exports.
- Write source comments in English and progress reports in English. Keep SARAKURA’s standard reports in English.

###### Connect each build to its execution

Use a separate output directory for each iteration, such as `out/iter-001`. Record commands, exit codes, and hashes of source, assets, tools, ROM and metadata. Never run an older ROM after a failed build. Maps, source maps and debug information must come from the same build as the ROM.

The following is an NROM check with no input. Supply `main.c`, required library implementation units and the CHR file; select the appropriate mapper. Do not pass compiler build metadata to KUROSAKI’s `--kitaqfc-debug` without first checking its required format.

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```


A 300-frame run with no input is only an initial check. Add ordered gameplay scenarios before claiming that the game works.

###### Reproduce ordered input

- `--pad1` and `--pad2` use raw NES bitmasks: A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. Do not confuse these with library `BTN_*` values.
- `run --pad1` applies fixed input. For sequences of actions, create a replay that distinguishes presses, holds and releases. Inspect the `Replay` and `ReplayFrame` definitions. The public CLI’s `replay-record` records neutral input, not a person’s interactive play.
- Check ROM SHA-256, replay identity and frame range yourself. `replay-run --verify` compares an expected final hash only when one is provided; it does not comprehensively validate gameplay or ROM identity. Never overwrite expected results with observed results merely to make a test pass.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Prepare the replay for the ROM being tested. The public CLI’s `replay-run` has no `--emit-diagnostics` option. Do not invent that option or pass CPU trace data as diagnostic events. To analyze ordered-input scenarios with SARAKURA, create a project-local test harness using public `kurosaki-core` APIs: `RunOptions.replay_frames` and `diagnostic_events_from_trace_and_report`. Run the same ROM, replay and frame conditions; generate diagnostic JSONL from that execution’s trace and diagnostic report. Check trace configuration and retained ranges, and compare the harness’s images and observations with CLI replay results. Do not present no-input diagnostics as evidence for a gameplay scenario. If the required environment is unavailable, report this verification as incomplete.

###### Check images, audio, state and performance

- Save input scenarios with distinct presses, holds and releases. Exercise every specified path: boot, start, movement, actions, collisions, scrolling, stage changes, game over, restart, pause and saving or communication where applicable.
- Preserve PNGs at relevant frames, input data, execution reports, diagnostic JSONL, WAVs and any necessary state or memory observations. Check the reached frame count and stop reason. Actually open the images; one screenshot cannot establish motion or input response. Compare counters, positions and state changes with expected values. Check screen edges, tile/attribute boundaries and crowded sprite scenes.
- Check music, effects, simultaneous playback, dropouts, pause and resume. A generated WAV alone does not establish correct sound. If listening is unavailable, distinguish the waveform/numerical checks performed from unverified audible qualities.
- Measure heavy scenes, target CPU/update workload and transfers; on FC, include NMI work. Host emulator throughput is not game update frequency or proof of hardware speed. Continuing with `--allow-unimplemented`, where available, does not demonstrate support for the missing feature.

###### Analyze, repair and retest

- Feed SARAKURA the build metadata for the tested ROM and diagnostic JSONL from the tested execution. A CPU trace or ordinary run report is not a substitute. `--frames` specifies analysis conditions; SARAKURA does not execute the ROM or automatically edit the source.
- Read `report.html`, `ai_diagnostics.json`, `repair_prompt.md` and `retest_plan.json`. Compare diagnoses with reproduction steps, images, audio and source. Distinguish inferred source locations or causes from verified facts, and normal waiting loops from hangs. Assess warnings individually and record unsupported events or analysis limits. Do not hide warnings with filters or shorten tests to obtain a passing result.
- Reduce failures to minimal reproductions, fix their causes and rebuild. If the compiler or emulator is responsible, isolate its defect from game code and add regression verification for the tool fix.
- Retest with matching input, random seed, hardware/video mode, mapper, observed frames and diagnostic settings. Use new metadata for each new ROM; do not blindly reuse save states after code or RAM layout changes.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Use diagnostic differences alongside gameplay, graphics and audio acceptance checks. If the same failure repeats, revisit the evidence and hypothesis instead of continuing arbitrary changes.

###### Completion and deliverables

Rerun all required scenarios against the final ROM built from the delivered source and settings. Invincibility, automatic test input or another mapper alone does not verify normal play in the final build. Provide a requirement-to-test table, reasons for remaining warnings and explicit unverified or unsupported items. State “not tested on physical hardware” when applicable.

Deliver source, tool/library identities, editable assets, reproducible build and test scripts, the ROM, final verification evidence, and a README covering setup, controls and known limits. Include replay data and a test harness where needed. Publish or send files externally only within explicitly authorized scope. Delete unnecessary intermediate builds and temporary traces after verification, preserving source, assets, final deliverables and needed regression evidence.

If environment or permission constraints prevent a required check, report the exact reproduction steps and required action. Do not mark the work complete.

</details>
<!-- development-prompt:en:end -->

### Repository layout

```text
kitaqfc/                  # Repository root
├─ kitaqfc/               # Compiler build sources
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Prebuilt Release compiler
├─ kitaqfc.exe.config     # .NET Framework runtime configuration
├─ lib/                # C support libraries
├─ examples/           # Tutorial programs and original font
├─ scripts/build.ps1   # Rebuild the Release executable
├─ LICENSE
└─ LICENSE.ja
```

The prebuilt compiler requires Windows with .NET Framework 4.8. Download the
repository ZIP to keep the executable, runtime configuration, libraries and
license notices together. Rebuilding additionally requires the .NET Framework
4.8 Developer Pack and Visual Studio Build Tools. From the repository root:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

A Release build copies the executable and its configuration to the repository
root. Debug builds stay inside `kitaqfc/bin/Debug` and do not overwrite the
distributed Release compiler. Build caches and PDB files are not distributed.
See [binary build record](BINARY_BUILD.json) for the build inputs and SHA-256.

### Build and first use

Windows, .NET Framework 4.8 Developer Pack and Visual Studio Build Tools (MSBuild). Run from a Developer PowerShell prompt.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

### Manuals and licenses

- [Japanese HTML manuals](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [English manuals](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [Offline manual source](https://github.com/bartaro/kitaq-docs)
- [License](LICENSE) / [Japanese reference translation](LICENSE.ja)

The project license does not replace third-party font, dependency, logo or trademark terms. Preserve the accompanying notices when redistributing.

---

<a name="japanese"></a>

## 日本語

KITAQGBとNORCALから派生した、自作のNES・ファミコン・FDSソフトウェア向けCコンパイラと支援ライブラリです。

パブリックプレビュー版です。APIや動作は変更される場合があります。

### 開発方針

KITAQFCは、NES・ファミコンのハードウェア特性を踏まえ、小さく再現可能な単位で確認しながら開発を進めることを重視します。ROMをビルドし、KUROSAKIで動かし、SARAKURAで実行結果を解析して、修正後に再検証します。実測した結果と、まだ検証していない動作を区別して記録します。

<!-- development-prompt:ja:start -->
### ゲーム開発プロンプト

依頼内容を記入して、プロンプト全文を生成AIに渡してください。実装、エミュレータ検証、SARAKURA解析、修正後の再検証まで含みます。

[HTMLマニュアルの参考例を読む](https://bartaro.github.io/kitaq-docs/kitaqfc.html#loop-prompts)

<details>
<summary>プロンプト全文を表示</summary>

#### KITAQFC・KUROSAKI・SARAKURAによるゲーム開発プロンプト

以下の「依頼内容」を記入し、このファイル全体を生成AIに渡してください。
コマンドは、`kitaqfc`、`kurosaki`、`sarakura`、`kitaq-docs`、`game-fc` が同じ親フォルダーにある配置を前提にしています。既存環境では実際のパスを使ってください。

##### 依頼内容

- ゲーム名：〈記入〉
- ジャンル・遊びの中心となる仕組み：〈記入〉
- プレイヤーが行う操作と、成功・失敗条件：〈記入〉
- 必須の画面・ステージ・敵・アイテム：〈記入〉
- 見た目、BGM、効果音：〈記入。資料がある場合はファイルも指定〉
- 対象：〈NES/ファミコンのカートリッジ／FDS〉
- マッパー・ROM規模・ミラーリング：〈指定。未定なら要件から選定する〉
- 映像方式・性能目標：〈例：NTSC、通常時に毎秒60回のゲーム更新〉
- 保存・周辺機器・その他の要件：〈記入。不要なら「なし」〉
- プロジェクトの保存先：〈例：game-fc〉
- 再配布条件：〈例：自作コードと素材をMITで公開できる状態にする〉

##### あなたに実行してほしいこと

KITAQFCと付属ライブラリで、上記のゲームを実装してください。
デバッグと実行検証にはKUROSAKI、診断の整理と修正前後の比較にはSARAKURAを使います。
「仕様を具体化 → 小さく実装 → ビルド → 操作して観測 → 原因を調べる → 修正 → 同条件で再検証」を、受け入れ条件を満たすまで繰り返してください。計画、コードの提示、コンパイル成功だけで完了にしないでください。

###### 1. 環境と受け入れ条件を確定する

1. 作業先の指示、各ツールのREADME、対象言語のHTMLマニュアル、使用するライブラリのヘッダーと実装を読んでください。実行ファイルの場所・バージョンまたはSHA-256を記録し、コマンドとAPIは実際の `--help` とソースで確認してください。
2. 操作、画面、音、進行、更新頻度について、合格・不合格を判断できる受け入れ条件を書いてください。「STARTを押して離すと開始する」「被弾で残機が1減る」「ゲームオーバー後に通常の操作で再開できる」など、確認可能な形にしてください。
3. 重要な仕様の曖昧さだけを確認し、通常の可逆な実装判断は自律的に進めてください。仕様や合格基準を勝手に弱めないでください。
4. まず付属の小さなサンプルでコンパイラ・KUROSAKI・SARAKURAの接続を確認してください。これを依頼されたゲームの完成と扱わないでください。

###### 2. マッパーと資源の構成を決める

- 小さな構成ではNROMを検討し、規模やバンク切り替えの要件がある場合はMMC3などを選んでください。マッパー名だけで対応を判断せず、KUROSAKIの `inspect-rom`、`mapper-info`、`audit-board` と実装で必要な機能を確認してください。
- PRG/CHR容量、CHR-ROMまたはCHR-RAM、ミラーリング、固定バンク、割り込みベクター、保存RAMを計画してください。最適化やバンク変更の後も、ヘッダーと実際の配置を照合してください。
- `--nes-local-ram` を使う場合、対象はCPU内蔵RAM `$0000–$07FF` の範囲です。ゼロページ、スタック、OAMの作業領域、ライブラリ用領域との重複を確認してください。
- VRAM転送キューの空き容量と、PPUのVRAM自体の容量を混同しないでください。1回のNMIで処理できる更新量も確認してください。
- FDSではディスクアクセス、保存、BIOSなどの条件を別に検証し、通常のカートリッジと同じ起動条件だと仮定しないでください。

###### 3. 小さく遊べる単位で実装する

- 最初に「起動・タイトル・操作可能なプレイヤー・成功または失敗・再開」をつなぎ、その後に内容を増やしてください。
- KITAQFCのC方言とFC用ライブラリを使ってください。GB版のライブラリやAPIがそのまま使えると仮定しないでください。エントリーポイントは `void main(void)` を使ってください。
- 関数は宣言だけでなく対応する実装を確認し、必要な `.c` をビルド対象に含めてください。付属ヘッダーに宣言のみの項目があることにも注意してください。
- PPUレジスター、NMI、OAM DMA、スプライトの走査線制限、スクロールとミラーリング、属性テーブル、APU/DMCの条件を踏まえて実装してください。
- 英数字・記号には、提供された自作 `ascii.c` から変換したFC用フォントを使ってください。CHR、パレット、ネームテーブル、属性データと編集可能な元データを保存し、ビルドが実際にそれらを読み込むことを確認してください。
- ソースのコメントは英語、作業報告は日本語で記述してください。SARAKURAの標準レポートは英語のまま利用してください。

###### 4. 各反復でビルドと実行を結び付ける

`game-fc/out/iter-001` のように出力先を分け、実行コマンド、終了コード、ROM・メタデータ・素材・ソース・ツールのハッシュを記録してください。
ビルド失敗後に残っている別のROMを実行しないでください。ビルド情報とデバッグ情報は同じROMに対応するものを使ってください。KITAQFCのビルドメタデータを、形式を確かめずにKUROSAKIの `--kitaqfc-debug` へ流用しないでください。

以下は親フォルダーから実行する、NROM・入力なしの動作確認例です。`main.c`、追加のライブラリ、CHRファイル、マッパーは実装に合わせて用意してください。

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```

300フレームの無入力実行はゲーム全体の検証ではありません。次の操作シナリオを必ず追加してください。

###### 5. 操作シナリオを再現して検証する

- KUROSAKIの `--pad1` / `--pad2` は生のNESビットマスクです。A=1、B=2、SELECT=4、START=8、UP=16、DOWN=32、LEFT=64、RIGHT=128です。ライブラリの `BTN_*` の値と混同しないでください。
- `run --pad1` は固定入力です。開始、移動、アクション、ポーズ、再開などの順序付き入力にはリプレイを用意し、押下・保持・解放を区別してください。
- `replay-record` は公開CLIでは無入力の記録を作るコマンドです。人が操作した入力の録画だと扱わないでください。`Replay` / `ReplayFrame` の形式を読み、必要な入力列を作成してください。
- 実行前にROMのSHA-256、リプレイの対象、フレーム範囲を自分で照合してください。`replay-run --verify` は期待ハッシュが存在する場合の比較であり、操作結果やROMの対応を包括的に検査するものではありません。テストを通すために期待値を実測値で無条件に上書きしないでください。

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```

リプレイファイルはそのビルドのROMに対応させて準備してください。修正前後で入力の意味と条件を揃え、ROMごとに適切な識別情報を記録してください。

**順序付き入力とSARAKURAの接続：** 公開CLIの `replay-run` には `--emit-diagnostics` がありません。存在しないオプションを追加して呼び出したり、CPUトレースを診断イベントとして渡したりしないでください。入力シナリオをSARAKURAで解析する際は、公開 `kurosaki-core` の `RunOptions.replay_frames` と `diagnostic_events_from_trace_and_report` を利用する検証ハーネスをプロジェクト配下に用意してください。同じROM・リプレイ・フレーム条件で実行し、その実行のトレースと診断レポートからJSONLを生成します。トレース設定・保存範囲を確認し、ハーネスとCLIリプレイの画面・観測結果も照合してください。無入力実行の診断を、操作シナリオの検証結果として流用しないでください。必要な環境がなく実施できない場合は未完了項目として報告してください。

###### 6. 画面・音・状態・性能を確認する

- 起動、開始、通常プレイ、衝突、スクロール、ステージ遷移、失敗、再開、ポーズ、保存など、仕様にある経路を通してください。1枚のPNGだけで動作を確認したことにしないでください。
- 代表フレームと境界条件のPNG、入力、実行レポート、診断JSONL、WAV、必要に応じた状態・メモリ観測を保存してください。要求したフレームに到達したか、停止理由は何かも確認してください。
- 画像は実際に開き、タイルや属性の境界、画面端、スクロール境界、スプライトが密集する場面を確認してください。残機、スコア、座標、状態遷移は期待値とも照合してください。
- BGM、効果音、同時発音、途切れ、ポーズ・再開を確認してください。WAVの生成だけで音を正しいと判断しないでください。試聴できない場合は、実施した波形・数値検査と未確認事項を分けて報告してください。
- 重い場面のNMI内処理、CPU負荷、転送量、ゲーム更新回数を測定してください。ホスト上のエミュレータ速度を実機やゲーム内の更新頻度と同一視しないでください。
- `--allow-unimplemented` によって処理を続行できても、その機能を対応済みと判断しないでください。

###### 7. SARAKURAで原因を絞り、同条件で再検証する

- `--metadata` は対象ROMのビルド情報、`--events` は対象実行の診断JSONLです。`--frames` は解析条件の指定であり、SARAKURAがROMを実行する命令ではありません。
- `report.html`、`ai_diagnostics.json`、`repair_prompt.md`、`retest_plan.json` を読み、診断を再現手順・画面・音・ソースと照合してください。ソース対応や原因の推定を確認済みの事実として扱わないでください。
- 正常な待機と停止不具合を区別してください。警告をフィルターで隠したり、試験を短くしたりして合格にしないでください。未対応のイベントや解析範囲も記録してください。
- 原因を最小再現例で絞り、必要な箇所を修正し、ビルドから再実行してください。ツール側が原因の場合はゲーム側の問題と切り分け、ツールの修正に回帰試験を付けてください。
- 修正前後の入力、乱数種、マッパー、映像方式、観測フレーム、診断条件を揃えて比較してください。コードやRAM配置が変わったROMに、保存状態を無条件で流用しないでください。

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```

差分診断は、操作・表示・音の合格判定と併用してください。同じ失敗を繰り返す場合はログと仮説を見直してください。

###### 8. 完了条件と納品

完成版のROMと同じソース・設定で、すべての必須シナリオを再実行してください。検証用の無敵状態や別のマッパーだけで完成版を検証済みにしないでください。
必須要件と試験の対応表、残る警告の理由、未対応・未確認事項を明記してください。実機で試していない場合は「実機未確認」と記載してください。

納品物は、ソース、使用ライブラリとツールの識別情報、編集可能な素材、ビルド・検証スクリプト、ROM、リプレイ・検証ハーネス、最終検証の証拠、起動方法・操作・既知の制限を記したREADMEです。
公開・外部送信は明示された範囲で行ってください。不要な中間ビルドや一時トレースは確認後に削除し、ソース、素材、最終成果物、必要な回帰証拠を保存してください。
環境や権限などの障害で必須検証を実施できない場合は、完了とせず、再現手順と必要な対応を具体的に報告してください。

</details>
<!-- development-prompt:ja:end -->

### リポジトリの構成

```text
kitaqfc/                  # リポジトリのルート
├─ kitaqfc/               # コンパイラのビルド用ソース
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # ビルド済みRelease版コンパイラ
├─ kitaqfc.exe.config     # .NET Frameworkの実行設定
├─ lib/                   # C支援ライブラリ
├─ examples/              # 入門プログラムと自作フォント
├─ scripts/build.ps1      # Release版の再ビルド
├─ LICENSE
└─ LICENSE.ja
```

ビルド用のC#ソースとプロジェクトは `kitaqfc/` にまとめています。ビルド済みRelease版はリポジトリ直下の `kitaqfc.exe` です。実行にはWindowsと.NET Framework 4.8が必要です。リポジトリのZIPを取得すると、実行ファイル、設定ファイル、ライブラリ、権利表記をまとめて入手できます。

再ビルドには.NET Framework 4.8 Developer PackとVisual Studio Build Toolsも必要です。リポジトリ直下で実行してください。

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Releaseビルドでは実行ファイルと設定ファイルをリポジトリ直下にコピーします。Debugビルドは `kitaqfc/bin/Debug` に置かれ、配布用Release版を上書きしません。ビルドキャッシュとPDBファイルは配布していません。ビルド入力とSHA-256は[バイナリのビルド記録](BINARY_BUILD.json)を参照してください。

### ビルドと初回利用

Windows、.NET Framework 4.8 Developer Pack、Visual Studio Build Tools（MSBuild）を用意し、Developer PowerShellから実行します。

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

### マニュアルとライセンス

- [日本語HTMLマニュアル](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [英語HTMLマニュアル](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [オフライン用マニュアルのソース](https://github.com/bartaro/kitaq-docs)
- [ライセンス英語原文](LICENSE) / [日本語参考訳](LICENSE.ja)

プロジェクトのライセンスは、第三者のフォント、依存ライブラリ、ロゴ、商標に関する条件を置き換えるものではありません。再配布時は付属の権利表記も保持してください。
