# FCSEQ 0.4.1 — 使用說明

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

作者 **DAISUKE OBA**。使用 **plita** 的通用紅白機音樂編曲器。GUI、CLI、Python 綁定共用專案模型、音訊產生及匯出功能。啟動時為空白專案，不需要遊戲 ROM 或 BIOS。介面標籤是英文，說明書提供九種語言。

## 下載與啟動

從 [downloads/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) 下載完整 ZIP：Windows x86_64、Linux x86_64、macOS Apple Silicon ARM64 或 Intel x86_64。請完整解壓縮，保留資源與授權文件。Windows 開啟 `fcseq.exe`，Unix 執行 `./fcseq`；命令列工具為 `fcseq-cli`（Windows 加 `.exe`）。[bin/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/bin) 也提供個別執行檔。Windows 10/11 使用 Direct3D 12，macOS 使用 Metal，Linux 使用 Vulkan/OpenGL、X11/Wayland 和 ALSA。Linux 在 Ubuntu 24.04 建置，較舊發行版可能需要自行編譯。Ubuntu 執行函式庫：`sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`。GUI 需要相容的顯示驅動程式，試聽需要音訊裝置；匯出 WAV 不需要。執行檔未簽章，macOS 版本未公證。若遭系統阻擋，請核對原始碼與雜湊，再使用系統正常的允許程序。必要時執行 `chmod +x fcseq fcseq-cli`。

Windows 另需 [Microsoft Visual C++ x64 可轉散發套件](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)（程式使用 `VCRUNTIME140.dll`）。若缺少，請安裝 Microsoft 官方目前支援的 x64 版本。GUI/CLI 不連結 Python；只有選用綁定和包裝腳本需要 Python。

## 作曲與編輯

SOUND 可選 2A03（Pulse 1/2、Triangle、Noise、DPCM，共5聲）、VRC6（增加2個 Pulse 和 Saw，共8聲）、VRC7（增加6個 FM，共11聲）、FDS（增加1個波形聲部，共6聲）、N163（增加1~8個波形聲部，最多13聲）。每個專案只能用一種擴充配置；造成音符遺失的切換或減少聲部會被拒絕。

雙擊新增音符，拖曳本體移動/移調，拖曳右緣改長度。Ctrl/Shift 點擊或框選多選，Ctrl+A 選取目前音軌。Ctrl+C/X/V 複製/剪下/在游標貼上；Ctrl+D 複製副本；Delete 刪除；Ctrl+Z/Y 復原/重做。方向鍵移動/移調，Shift+上下改八度。G 格線、Q 量化、F 全景、Home 起點、+/- 縮放、Shift+滾輪移動時間、UP/DOWN 捲動聲部。Space 播放/停止、LOOP 循環、AUDITION 試聽選取音符。MUTE/SOLO 也影響匯出。SONG 管理最多16段、每段256小節；點擊名稱、BPM、小節數修改。Ctrl+N/O/S/Shift+S 新建/開啟/儲存/另存。無效批次編輯整體取消。復原檔位於執行檔旁 `out/recovery_<process-id>.fcq`，開啟後另存即可。

## 音色與波形

開啟 SOUND > Instrument / waveform editor，選取該聲部音符後調整，使用 AUDITION/UNDO/REDO。Pulse 工作週期4級，VRC6 Pulse 8級；Triangle 為固定32級波形；Noise 有16個週期及長/短模式。Pulse/Noise 音量0~15，衰減0維持固定音量。VRC6 Saw 累加增量0~63，溢位也改變音色。這些音源沒有自由繪圖用波形 RAM。

FDS 有共用64取樣點、6位元波形。N163 為4位元波形，初始4×32點，各音符選擇波形編號，長度須為4的倍數。共用容量 `256 - 16 × 啟用聲部數` 點（8聲為128點）；延長前先縮短其他波形或減少聲部。支援拖曳繪圖、SINE/PULSE/SAW/TRIANGLE、反轉與旋轉，並檢查音高/RAM/聲部限制。VRC7 自訂音色0可編輯24個參數（倍頻、ADSR、回授、音量、鍵盤縮放、顫音/顫頻、正弦/半波）或8位元組。預設1~15是獨立合成的近似音色，不是音色 ROM 傾印。沒有 FDS 調變表/硬體包絡專用編輯器。模擬音質不保證與實機類比輸出一致。

## DPCM

SAMPLES 或 SOUND > Import audio file 匯入 WAV/FLAC/Ogg Vorbis/MP3，也可拖放。設定 START/END（毫秒）、GAIN、NORMALIZE、REVERSE、RATE 0~15，以 AUDITION DPCM 試聽，EXPORT .DMC 儲存。向 DPCM 音軌加入音符，在檢查器選 SAMPLE 與播放 RATE。保留原始 PCM 供還原；輸入最多60秒，立體聲轉單聲道，轉換時移除直流並低通重新取樣。每個音色 `16n+1` 位元組、最多4081位元組，最多64音色，整庫16320位元組（FDS 8192）。超限操作拒絕。轉換取樣率與音符播放速率不同。沒有音色內部循環或自動音高校正。

## 檔案與匯出

專案儲存為 `.fcq`，舊 `.fcseq` 可改名讀取。音符專案為 `fcseq-2` JSON，也支援 `fcseq-1`；NSF 播放文件為 `fcseq-3`。透過 Import NSF、OPEN、拖放或啟動參數開啟 NSF/NSFe。PREV/NEXT 或曲號選曲，PLAY/STOP、LOOP、1~600秒控制播放。有曲長資訊時採用，否則預設30秒。播放前音訊產生可取消。FCQ 內嵌原檔、選曲、時間；不會自動轉成音符。重新匯出保留原容器位元組。

Export NSF 將各段轉為子曲，含音源和 DPCM，並產生帶曲長及淡出0的 NSFe。播放器忽略 NSF 曲長時請用 NSFe。段落結束即停止，編輯器 LOOP 獨立運作。輸入支援 NSF v1/NSFe、NTSC 或相容 NTSC 的雙區、無分庫或4KiB分庫、2A03/DPCM、VRC6/VRC7/FDS/N163。拒絕 PAL 專用、NSF2 執行擴充、MMC5、Sunsoft 5B、VT02；不返回的 INIT/PLAY 有執行時間限制。

Export C + DPCM 輸出 `score.c`、`fcseq_score.h`、`fcseq_player.c`、段落二進位、`.dmc`、`dpcm_bank.bin`、`fcseq_export.json`。不自動建立完整遊戲 ROM；需自行配置 mapper 與 CPU 可存取的程式碼/資料/DPCM 位址。VRC6 假設 mapper 24（26需轉換位址線）、VRC7 85、N163 19，或 FDS。KITAQFC 中在 include 前定義 `FCSEQ_KITAQFC`；`FCSEQ_WRITE(address, value)` 可接上自訂暫存器寫入函式。

## CLI 與 Python

以下範例在解壓縮目錄執行。段落/聲部/音符/樣本編號從0開始，NSF 曲號從1開始。附帶 Python 3.10+ abi3 擴充，須配合 OS/CPU 並從同目錄載入。GUI/CLI 不需要 Python。完整 API 見 `fcseq_native.pyi`。CLI 支援音符移動、移調、改長度、量化、複製、刪除；波形由 GUI/Python 編輯。

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

## 建置與驗證

在此原始碼目錄使用 Rust 1.93.1 及以下指令。本地函式庫原始碼位於 vendor/。Linux 開發套件見儲存庫根目錄 `.github/workflows/fcseq.yml`；macOS 需要 Xcode Command Line Tools，Windows 需要 MSVC Build Tools。Python 為選用。腳本驗證原生 CLI/Python、WAV、NSF/NSFe 往返及 GUI 啟動。結果與範圍見 VERIFICATION.md；不認證人工滑鼠/鍵盤操作或實機聆聽。

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
python -c "import shutil,sys; from pathlib import Path; n={'win32':'fcseq_native.dll','darwin':'libfcseq_native.dylib'}.get(sys.platform,'libfcseq_native.so'); shutil.copy2(Path('target/release')/n,'fcseq_native.pyd' if sys.platform=='win32' else 'fcseq_native.so')"
```

## 授權

FCSEQ 為 MIT，Copyright (c) 2026 DAISUKE OBA。相依套件保留各自授權。重新散布請包含 LICENSE、THIRD_PARTY_NOTICES.md、licenses/、assets/、third_party_sources/。已附 MPL-2.0 Symphonia 原始碼與字型/圖示聲明。匯入音樂與取樣另有其權利。

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
