# 原生輔助工具

[en](../README.en.md) · [ja](../README.ja.md) · [ko](../README.ko.md) · [zh-CN](../README.zh-CN.md) · [zh-TW](../README.zh-TW.md) · [fr](../README.fr.md) · [es](../README.es.md) · [de](../README.de.md)

所有輔助程式均以 Rust 實作，執行時不需要 .NET、Python 或 Pillow。使用 `cargo build --locked --release` 建置全部程式。Windows 指令名稱須加 `.exe`。

```text
kitaqfc-zx0 input.bin output.zx0
kitaqfc-zx0 output.zx0 restored.bin --decompress
kitaqfc-zx0 input.bin asset.h --header=level_data
kitaqfc-zx0 input.bin output.kqa --format=auto
kitaqfc-asset-pack assets/example_manifest.json
kitaqfc-png-index-build assets/example_png_manifest.json
kitaqfc-asset-pipeline assets/example_pipeline_manifest.json
kitaqfc-rights-name-guard --root . --denylist deny_terms.local.txt
```

ZX0 接受 1～65535 位元組並保留 C# 編碼器的輸出。支援 `raw`、個數/值 `rle` 與九位元組 `KQA1` 自動容器；大小相同依序選擇 raw、RLE、ZX0。`--decompress` 對純順向 ZX0 v2 串流進行有界解碼，不支援逆向串流及 v1。格式由 Einar Saukas 設計，KITAQ 實作採用 MIT 授權。

讀取既有 JSON 清單產生 8 KiB CHR、C 陣列/標頭及 JSON 報告。metasprite 支援 data、frames 和 JSON 輸入。索引 PNG 轉換產生 CHR、名稱表、屬性、調色盤及 metasprite JSON。背景為 256×240，圖集以 8×8 圖塊組成，單圖塊及背景 16×16 區塊內調色盤組須一致。色索引為 0～15，支援 1/2/4/8 位元深度及 Adam7。缺少的 PNG 原腳本依本機規格與範例清單重建，無法與原件進行位元組比較。RGB 對應至定義的 NES 近似調色盤。管線在內部執行兩階段。名稱檢查讀取外部詞表，只回報檔案與行號，不顯示保護詞，排除產生目錄。

```sh
sh tools/zx0/build.sh
```
