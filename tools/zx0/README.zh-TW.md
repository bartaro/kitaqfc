# ZX0 相容素材壓縮

<!-- readme-language-links:start -->
[English](README.md#english) | [日本語](README.md#%E6%97%A5%E6%9C%AC%E8%AA%9E) | [한국어](README.ko.md) | **繁體中文** | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

[API 與範例](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html#module-zx0)

PC 壓縮器與 FC 解壓縮器是 KITAQ 獨立撰寫的實作，支援 ZX0 v2 順向串流。KITAQ 實作採用 MIT 授權條款，著作權為 Copyright (c) 2026 DAISUKE OBA。

ZX0 格式與原始壓縮演算法由 [Einar Saukas](https://github.com/einar-saukas/ZX0) 設計。這項格式致謝與 KITAQ 實作的著作權、授權分別表述；另請參閱 [LICENSE](../../LICENSE) 與 [LICENSE.ja](../../LICENSE.ja)。

從儲存庫根目錄建置 PC 工具：

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

PC 工具使用 .NET Framework 4.x。壓縮器採用限制搜尋次數的雜湊鏈，不保證得到最小輸出。此介面不支援逆向串流、外部前綴字典或 ZX0 v1。原始素材的權利仍屬於各自作者。

輸入與編碼後的輸出各自須在 65535 位元組以內。`--format=auto` 比較 raw、RLE、ZX0 的酬載，選擇最小者並加上九位元組 KQA1 標頭；標頭也計入輸出上限。KQA1 請使用 `asset_decompress` 展開。素材還須依實際 CPU bank 視窗與 RAM 容量分割。空的 raw C 標頭會保留一個佔位位元組，但邏輯 `_SIZE` 為 0。

目標程式須引入 `zx0.h`，並編譯 `lib/zx0.c`。`zx0_decompress` 接受輸出位址、輸出容量、壓縮來源與壓縮大小。請同時檢查回傳位元組數與 `zx0_error`。發生錯誤時可能已寫入部分輸出，因此失敗後不要顯示或使用結果。來源與目的緩衝區不可重疊，也不可跨越目前映射的 CPU bank 視窗邊界。函式使用共用工作區，不可從中斷重入。

`zx0_decompress_vram` 需要足以容納整份解壓縮素材的 RAM 工作區。關閉繪製時，函式先解壓至工作區，再將結果上傳到 CHR RAM 或名稱表記憶體。PPUCTRL 會保持不變；重新啟用繪製前請設定捲動位置。此函式不傳送調色盤，也不寫入 CHR ROM。
