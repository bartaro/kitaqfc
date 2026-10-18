# FCSEQ 0.5.0

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ 是一款使用紅白機音源創作音樂的編輯器。你可以在 plita 介面中編寫音符、調整音色和波形，並隨時試聽。GUI、CLI 與 Python bindings 共用同一種專案格式和音訊引擎。程式從空白專案開始，不依賴特定遊戲或 ROM。

## 安裝與介面語言

從 [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) 下載適合電腦的 ZIP，並完整解壓縮。Windows、Linux 提供 x86_64 版本，macOS 提供 Apple Silicon 與 Intel 版本。Windows 執行 `fcseq.exe`，Linux 與 macOS 執行 `./fcseq`。若缺少執行權限，可使用 `chmod +x fcseq fcseq-cli`。請一併保留字型與授權文件。

介面支援英文和日文。選擇 **SOUND → 日本語 / English** 即可切換，程式會記住設定；NSF 播放器的工具列也有相同按鈕。首次啟動會依系統語言選擇，也可用 `FCSEQ_LANGUAGE=ja` 或 `en` 指定。套件內含 Noto Sans JP，讓各系統都能顯示日文。

Windows 需要 Direct3D 12 驅動程式和 Microsoft Visual C++ x64 執行階段；macOS 使用 Metal。Linux 版本在 Ubuntu 24.04 建置，使用 ALSA、X11/Wayland 與 Vulkan/OpenGL，較舊的發行版可能需要自行編譯。即時播放需要音訊裝置，但匯出 WAV 不需要。執行檔沒有數位簽章，macOS 版本也未經公證。

## 編寫音符與鍵盤編輯器

先在 **SOUND** 選擇音源，再於音軌空白處按兩下即可新增音符。拖曳音符本體可改變位置及音高，拖曳右端可改變長度。每條音軌對應一個硬體聲道，因此不能放置重疊的音符。

在音軌名稱上按兩下，或按 **E**，即可單獨開啟該音軌。鍵盤編輯器左側有鋼琴鍵盤，縱向每格是一個半音，橫向是時間。點選琴鍵可以試聽。滾輪移動音域，鍵盤旁的 −/+ 調整顯示音域，Shift+滾輪移動時間位置。音軌箭頭可切換至其他音軌，再按 E 回到全部音軌。雜訊、DPCM 與節奏音軌會顯示適用的聲音或播放速率範圍。

按住 Ctrl/Shift 點選，或在空白處拖曳選取框，可選取多個音符。Ctrl+A 選取目前音軌；Ctrl+C/X/V 複製、剪下或貼到游標處；Ctrl+D 複製一份；Delete 刪除。Ctrl+Z/Y 復原或重做。方向鍵移動或移調，Shift+上/下移動一個八度。G 改變格線，Q 將音符對齊格線，F 顯示整個樂段，Home 回到開頭，+/− 縮放時間軸。

Space 控制播放與停止。LOOP 重複目前樂段，AUDITION 試聽選取的音符。靜音和獨奏也會影響匯出。在 SONG 中可以新增、命名及調整樂段順序，每個專案最多 16 個樂段，每段最多 256 小節。點選速度或小節數即可修改。Ctrl+N/O/S/Shift+S 分別是新增、開啟、儲存、另存新檔。復原檔案位於程式旁的 `out/recovery_<process-id>.fcq`；需要時開啟並另存新檔。

## 音源與音色

基本 2A03 有兩路脈衝波、三角波、雜訊與 DPCM，共 5 條音軌。每個專案可選一種擴充音源。

| 擴充音源 | 增加的聲音 | 總音軌數 |
|---|---|---:|
| VRC6 | 兩路脈衝波、鋸齒波 | 8 |
| VRC7 | 6 路 FM | 11 |
| FDS | 1 路波表 | 6 |
| N163 | 最多 8 路波表 | 13 |
| MMC5 | 兩路脈衝波、PCM | 8 |
| SUNSOFT 5B | 3 路 SSG，共用雜訊與包絡 | 8 |
| EPSM | 3 路 SSG、6 路 FM、6 路節奏 | 20 |
| VT02 | 兩路脈衝波、三角波、雜訊、PCM | 10 |

若更換音源或減少聲道會遺失音符，程式會拒絕變更。請先清空相關音軌，或建立新專案。

選取音符後開啟 **INSTRUMENT / WAVEFORM**。脈衝波可調整工作週期，雜訊可調整週期及長短模式，VRC6 鋸齒波可調整累加速率。這些聲道使用固定波形，不能任意重畫。

FDS 提供一條共用的 64 點、6 位元波形。N163 使用 4 位元波形，可為音符選擇波形槽。你可以拖曳圖形繪製，也能從正弦波、脈衝波、鋸齒波或三角波預設開始，再反轉或循環位移。**CURVE / LINE** 可切換平滑曲線與折線。圖上的編輯點才是儲存至音源的值，曲線只是顯示用的內插。這樣既能看清正弦波的平滑走勢，也能精確編輯量化後的取樣值。將指標移近編輯點可查看編號和值。

N163 波形長度必須是 4 的倍數。所有聲道共用 `256 − 16 × 啟用聲道數` 個取樣值的記憶體，8 聲道時可用 128 個。空間不足時，可以縮短其他波形或減少聲道。尚未提供 FDS 調變表與硬體包絡編輯。

VRC7 自訂音色 0 可編輯運算子與包絡等參數；1～15 是合成的近似預設。EPSM 提供 16 個可編輯 FM 音色，每個音色有 4 個運算子，以及演算法、回授、包絡與調變設定。修改會套用到使用同一音色的所有音符。FM 與節奏還可設定聲像。目前試聽與 WAV 為單聲道，但匯出的暫存器資料會保留聲像設定。EPSM 節奏試聽使用自行合成的打擊樂，不包含 Yamaha ROM 傾印；實際 EPSM 裝置會播放內建的節奏音。

5B 與 EPSM 的 SSG 可組合音調、雜訊與包絡。三個 SSG 聲道共用雜訊週期與包絡產生器，因此後寫入的設定也會影響其他使用該產生器的音符。

## 把音訊用作樂器

在 SAMPLES 匯入 WAV、FLAC、Ogg Vorbis 或 MP3，也可以把檔案拖入視窗。調整起點、終點、增益、正規化與反向播放。原始音訊會保留，因此處理可以重新調整。來源音訊最長 60 秒，立體聲會混合為單聲道。

在 DPCM 音軌新增音符並選擇取樣與播放速率。AUDITION DPCM 試聽轉換結果，EXPORT .DMC 儲存資料。轉換時會移除直流成分，並透過低通濾波重新取樣。單一取樣最多 4081 位元組，共用取樣區最多 16320 位元組，FDS 則為 8192 位元組。超過時請縮短範圍或降低轉換速率。取樣的轉換速率和音符的播放速率是兩個獨立設定。

MMC5、VT02 的 PCM 音軌也使用同一份音訊與處理設定。它們以 4 kHz 串流播放 8 位元 PCM，音符音高控制播放速度，不受 DPCM 位元組限制。VT02 的 DPCM 與 PCM 共用取樣播放單元，因此同一樂段只能使用其中一種模式。尚未提供取樣循環與自動音高校正。

## 專案、NSF 與匯出

可編輯專案儲存為 `.fcq`。舊 `.fcseq` 可改副檔名為 `.fcq` 後開啟。音符專案沿用 `fcseq-2` JSON，也能讀取 `fcseq-1`。包含匯入音樂的播放專案使用 `fcseq-3`。

開啟 NSF、NSF2 或 NSFe 後，選擇曲目與時長即可播放。有時間中繼資料時依其播放，否則預設為 30 秒，可改成 1～600 秒。存為 FCQ 會嵌入原始音樂和播放設定。匯入是為了播放，不會把任意 NSF 自動轉成可編輯音符。再次匯出為同一容器時會保留原始位元組。

匯出時選擇 `.nsf`、`.nsf2` 或 `.nsfe`，每個樂段會成為一首曲目。匯出 NSF、NSF2 時也會產生帶曲目時長與零淡出的 NSFe。曲目在樂段結尾停止，與編輯器的 LOOP 分開。播放器支援 NTSC、可用 NTSC 播放的雙區域檔案、4 KiB 分頁和上表音源，也支援 NSF2 的計時器 IRQ、持續執行 INIT 和停止呼叫 PLAY。PAL 專用檔案不支援。EPSM 與 NSF2 計時器連接埠重疊，因此會拒絕同時使用 EPSM 和 NSF2 計時器 IRQ 旗標的檔案。詳見[音訊格式說明](docs/AUDIO_FORMATS.md)。

C + DPCM 匯出包含 C 驅動程式、標頭檔、暫存器串流、取樣區與配置說明，不會產生完整遊戲 ROM。你需要設定目標映射器，讓 CPU 能持續存取程式碼與資料。依 `FCSEQ_TICK_RATE_HZ` 呼叫 `fcseq_tick`：通常為 NTSC 畫面更新率，使用 PCM 的專案為 4000 Hz。KITAQFC 使用時定義 `FCSEQ_KITAQFC`，也可透過 `FCSEQ_WRITE(address, value)` 替換暫存器寫入操作。

## CLI、Python 與建置

執行 `fcseq-cli help` 可查看命令；Windows 檔名加上 `.exe`。樂段、聲道、音符、音色及取樣索引從 0 開始，NSF 曲目從 1 開始。

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli nsf-render song.nsf2 preview.wav 1 10
./fcseq-cli set-epsm-parameter song.fcq edited.fcq 0 44 4
```

Python 3.10 以上可載入套件中符合系統、CPU 的原生擴充模組。GUI、CLI 不需要 Python。[API 清單](fcseq_native.pyi)包含 `Controller`、`set_sound`、`set_epsm_parameter`、波形編輯與匯出。依序呼叫 `c.set_expansion('epsm')`、`c.add_note(0, 8, 0, 8, 69)`、`c.export_nsf('song.nsf2')`，即可建立 FM 音符並匯出。

建置需要 Rust 1.93.1 與 C++14 編譯器。使用 `cargo build --locked --release` 建置，`cargo test --locked --tests` 測試。僅建置 CLI 加上 `--no-default-features --bin fcseq-cli`；Python 擴充模組用 `--features python --lib`。Windows 需要 MSVC Build Tools，macOS 需要 Xcode Command Line Tools，Linux 開發函式庫請參考儲存庫的 `.github/workflows/fcseq.yml`。

FCSEQ 應用程式碼採 MIT 授權，Copyright (c) 2026 DAISUKE OBA。ymfm（BSD-3-Clause）、Symphonia（MPL-2.0）與字型（OFL）保留各自授權。重新散布時請保留 LICENSE、THIRD_PARTY_NOTICES.md、licenses/、assets/ 與 third_party_sources/。不含遊戲 ROM、FDS BIOS 或商業音源 ROM 傾印。匯入的音樂與取樣另有權利歸屬。請參閱[第三方聲明](THIRD_PARTY_NOTICES.md)和[驗證紀錄](VERIFICATION.md)。

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
