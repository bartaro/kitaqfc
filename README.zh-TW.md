# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **繁體中文**

**[編譯器手冊](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html)** · **[程式庫手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html)**

用於開發原創 NES／Famicom／FDS 自製軟體的 C 編譯器與支援程式庫，源自 KITAQGB 與 NORCAL。

本專案目前為公開預覽版，API 與行為仍可能調整。

## 開發理念

KITAQFC 重視 NES／紅白機的硬體特性，並以可重現的小步驟推進開發與驗證。建置 ROM 後，在 KUROSAKI 中執行，使用 SARAKURA 分析執行結果，並在每次修正後重新驗證。記錄時應區分實測結果與尚未測試的行為。

<!-- development-prompt:zh-TW:start -->
## 遊戲開發提示詞

填寫需求後，將完整提示詞交給 AI。內容涵蓋實作、模擬器測試、SARAKURA 分析，以及修正後的重新驗證。

[閱讀 HTML 手冊中的參考範例](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html#loop-prompts)

<details>
<summary>展開完整提示詞</summary>

### 使用 KITAQFC、KUROSAKI 與 SARAKURA 開發遊戲

填寫需求後，將本文完整交給 AI。命令假設 `kitaqgb`、`kitaqfc`、`kokura`、`kurosaki`、`sarakura`、`kitaq-docs` 儲存庫與 `game-gb` 或 `game-fc` 專案位於同一個上層目錄。請從該目錄執行，並依實際環境調整路徑。

#### 需求

- 遊戲名稱：&lt;填寫&gt;
- 類型與核心玩法：&lt;填寫&gt;
- 操作方式及成功、失敗條件：&lt;填寫&gt;
- 必備畫面、關卡、敵人與道具：&lt;填寫&gt;
- 美術風格、背景音樂與音效：&lt;填寫，並註明提供素材的路徑&gt;
- 存檔、通訊、周邊設備及其他需求：&lt;填寫，或無&gt;
- 專案目錄：&lt;填寫&gt;
- 再散布條件：&lt;例如，自行撰寫的程式與原創素材可採 MIT 授權公開&gt;

- 目標：&lt;NES/紅白機卡匣或 FDS&gt;
- Mapper、ROM 大小與鏡像方式：&lt;指定，或依需求選擇&gt;
- 影像制式與效能目標：&lt;例如 NTSC，每秒更新遊戲邏輯 60 次&gt;

#### 請執行的工作

請以 KITAQFC 及其函式庫實作遊戲。使用 KUROSAKI 執行與除錯，使用 SARAKURA 整理診斷並比較修正前後的結果。

持續重複以下流程，直到符合驗收標準：具體化規格 → 實作小幅變更 → 建置 → 輸入操作並觀察 → 調查原因 → 修正 → 在相同條件下重新驗證。不可只因提出計畫、提供程式碼或編譯成功，就認定工作完成。

##### 確認環境與驗收標準

1. 閱讀工作目錄的指示、各工具 README、HTML 手冊，以及所用函式庫的標頭檔和實作。記錄執行檔路徑及版本或 SHA-256；以實際 `--help` 輸出確認命令，以原始碼確認 API。
2. 為輸入、畫面、聲音、遊戲流程和更新頻率訂出可判定的驗收標準。例如，按下再放開 START 後開始遊戲；碰撞減少一條命；暫停時指定聲音靜音，繼續後恢復播放。
3. 只針對重要的模糊需求提問，一般可復原的實作決策請自主處理。不得自行降低需求或驗收標準。
4. 先用隨附的小範例串接編譯器、模擬器與 SARAKURA。這只能確認工具銜接，不能代表所需遊戲已完成。

##### 先完成可玩的最小流程

- 小型遊戲先考慮 NROM；若規模或分頁切換需求需要，再選用 MMC3 等 Mapper。不能只憑名稱判斷支援程度，應透過 `inspect-rom`、`mapper-info`、`audit-board` 和實作確認所需卡匣電路板功能。
- 規劃 PRG/CHR 大小、CHR-ROM 或 CHR-RAM、鏡像、固定分頁、中斷向量與存檔 RAM。最佳化或變更分頁後，核對標頭與實際配置。`--nes-local-ram` 使用 CPU 內部 RAM `$0000–$07FF`；避免與零頁、堆疊、OAM 緩衝區及執行階段、函式庫區域重疊。
- 使用 KITAQFC 的 C 方言、FC 函式庫與 `void main(void)`。不要假設 GB API 相容。有些標頭項目只有宣告，需確認實作位置並加入必要的 `.c` 檔案。
- 考慮 PPU 暫存器、NMI、OAM DMA、每掃描線精靈上限、捲動、鏡像、屬性表及 APU/DMC 行為。佇列剩餘容量不等於 PPU VRAM 容量，應規劃每次 NMI 可負擔的工作量。
- 將提供的原創 `ascii.c` 字型轉換成 FC CHR，確認 CHR、調色盤、名稱表與屬性資料。FDS 的磁碟存取、存檔與 BIOS 條件需另行驗證，不可套用卡匣的啟動條件。

- 先串起開機、標題畫面、可操控角色、成功或失敗與重新開始，再擴充內容。
- 保留可編輯的圖形、音樂、音效原始檔及產生步驟，並確認建置確實讀取匯出的資料。
- 原始碼註解使用英文，進度報告使用繁體中文。SARAKURA 的標準報告維持英文。

##### 對應每次建置與執行結果

以 `out/iter-001` 等目錄區分每輪輸出。記錄命令、結束碼，以及原始碼、素材、工具、ROM、中繼資料的雜湊值。建置失敗後，不可執行殘留的舊 ROM。配置映射、原始碼映射與除錯資訊必須和 ROM 來自同一次建置。

以下是不提供輸入的 NROM 檢查範例。請準備 `main.c`、必要的函式庫實作與 CHR 檔案，並選擇合適的 Mapper。未確認格式前，不要把編譯器的建置中繼資料傳給 KUROSAKI 的 `--kitaqfc-debug`。

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


無輸入執行 300 影格只是初步檢查。在認定遊戲正常之前，必須加入具有操作順序的遊玩情境。

##### 重現有先後順序的輸入

- `--pad1` 和 `--pad2` 使用 NES 原始位元遮罩：A=1、B=2、SELECT=4、START=8、UP=16、DOWN=32、LEFT=64、RIGHT=128。不要與函式庫的 `BTN_*` 數值混淆。
- `run --pad1` 使用固定輸入。連續操作應製作重播資料，區分按下、按住與放開。閱讀 `Replay`、`ReplayFrame` 定義。公開 CLI 的 `replay-record` 記錄的是無輸入狀態，不是玩家的互動操作。
- 自行核對 ROM SHA-256、重播對象與影格範圍。`replay-run --verify` 只在提供預期最終雜湊時進行比較，並不完整驗證遊戲行為或 ROM 對應關係。不得只為通過測試，就把預期值一律改成實測值。

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


為待測 ROM 準備相符的重播資料。公開 CLI 的 `replay-run` 沒有 `--emit-diagnostics` 選項。不要捏造選項，也不要把 CPU 追蹤當成診斷事件。若要以 SARAKURA 分析順序輸入情境，請在專案內製作測試驅動程式，使用公開 `kurosaki-core` 的 `RunOptions.replay_frames` 和 `diagnostic_events_from_trace_and_report`。以相同 ROM、重播和影格條件執行，從該次執行的追蹤與診斷報告產生 JSONL。確認追蹤設定與保留範圍，並將驅動程式的畫面、觀測結果與 CLI 重播比對。不得把無輸入執行的診斷當作遊玩情境的證據。若缺少必要環境，應將此項驗證列為未完成。

##### 檢查畫面、聲音、狀態與效能

- 保存輸入情境，區分按下、按住與放開。走過規格中的全部路徑：開機、開始、移動、動作、碰撞、捲動、關卡切換、遊戲結束、重新開始、暫停，以及適用的存檔與通訊。
- 保留關鍵影格 PNG、輸入資料、執行報告、診斷 JSONL、WAV 和必要的狀態、記憶體觀測。確認實際到達的影格數與停止原因。務必開啟圖片查看；一張截圖無法證明移動或輸入反應。將計數器、座標與狀態變化和預期值比對，也要檢查畫面邊緣、圖塊與屬性邊界及精靈密集情境。
- 檢查音樂、音效、同時發聲、斷音、暫停與恢復。僅產生 WAV 不能證明聲音正確。無法試聽時，請區分已執行的波形、數值檢查與尚未確認的聽感。
- 測量高負載場景的目標 CPU 工作量、遊戲更新與傳輸量；FC 還要計入 NMI 工作。主機上模擬器的執行速度不等於遊戲更新頻率，也不能證明實機速度。使用 `--allow-unimplemented` 後能繼續執行，不代表未實作功能已受支援。

##### 分析、修正並重新驗證

- 將待測 ROM 的建置中繼資料與該次執行的診斷 JSONL 交給 SARAKURA。CPU 追蹤或一般執行報告不能取代它。`--frames` 指定分析條件；SARAKURA 不會執行 ROM，也不會自動修改原始碼。
- 閱讀 `report.html`、`ai_diagnostics.json`、`repair_prompt.md`、`retest_plan.json`，並與重現步驟、畫面、聲音及原始碼核對。區分推測的位置、原因與已確認事實，也要區分正常等待迴圈與當機。逐項判讀警告，記錄未支援事件與分析限制。不可透過隱藏警告或縮短測試來取得通過結果。
- 將問題縮減成最小重現案例，修正原因後重新建置。若根源在編譯器或模擬器，應與遊戲程式問題分開確認，並為工具修正加入回歸驗證。
- 重新驗證時，保持輸入、亂數種子、機型與影像制式、Mapper、觀測影格及診斷設定一致。新 ROM 使用相符的中繼資料；程式或 RAM 配置變更後，不可直接沿用即時存檔。

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


診斷差異應搭配操作、畫面與聲音的驗收結果判斷。相同失敗反覆發生時，請重新檢視證據與假設，不要漫無根據地繼續修改。

##### 完成條件與交付內容

以交付的原始碼與設定建置最終 ROM，再執行所有必要情境。僅使用無敵狀態、自動測試輸入或另一種 Mapper，無法驗證最終版本的正常遊玩。提供需求與測試對照表，說明剩餘警告的原因，明列未驗證、未支援項目。未做實機測試時，請標註「實機未驗證」。

交付原始碼、工具與函式庫識別資訊、可編輯素材、可重現的建置與測試指令稿、ROM、最終驗證證據，以及說明安裝、操作與已知限制的 README。視需要附上重播資料和測試驅動程式。只在明確授權範圍內公開或對外傳送檔案。驗證後刪除不必要的中間建置與暫存追蹤，但保留原始碼、素材、最終成果及必要的回歸證據。

若環境或權限阻礙必要檢查，請回報確切的重現步驟與所需處理，不得標記為完成。

</details>
<!-- development-prompt:zh-TW:end -->

## 儲存庫結構

同名子目錄 `kitaqfc/` 集中存放編譯器原始碼、專案檔與建置設定。已建置的 Release 執行檔及其執行階段設定檔位於根目錄。`lib/` 為 C 程式庫，`examples/` 則提供入門程式與原創字型。

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

隨附編譯器須在 Windows 與 .NET Framework 4.8 環境執行。請下載整個儲存庫的 ZIP，將執行檔、設定、程式庫與授權聲明保存在一起。若要自行建置，還須安裝 .NET Framework 4.8 Developer Pack 和 Visual Studio Build Tools。請在儲存庫根目錄執行：

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Release 建置會將程式與設定檔複製到根目錄。Debug 版本保留在 `kitaqfc/bin/Debug`，不會覆寫發行用的 Release 編譯器。發行內容不包含建置快取或 PDB 檔。建置輸入與 SHA-256 請見[二進位檔建置紀錄](BINARY_BUILD.json)。

## 從原始碼建置與開始使用

在 Windows 安裝 .NET Framework 4.8 Developer Pack 與 Visual Studio Build Tools 後，也可以直接呼叫 MSBuild。請開啟 Developer PowerShell 並執行：

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## 手冊與授權

- [繁體中文編譯器手冊](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html)／[繁體中文程式庫手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html)
- [英文手冊](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)／[日文手冊](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [可供離線閱讀的手冊原始檔](https://github.com/bartaro/kitaq-docs)
- [授權條款](LICENSE)／[日文參考譯文](LICENSE.ja)

本專案的授權不取代第三方對字型、相依套件、標誌或商標訂定的條件。再散布時，請一併保留隨附聲明。
