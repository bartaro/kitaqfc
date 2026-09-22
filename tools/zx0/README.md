# ZX0-compatible asset compression

[English](#english) | [日本語](#日本語)

## English

The PC compressor and FC decompressor are independent KITAQ implementations
for ZX0 v2 forward streams. The KITAQ implementation is released under the
MIT License, copyright (c) 2026 DAISUKE OBA.

The ZX0 format and original compression algorithm were designed by
[Einar Saukas](https://github.com/einar-saukas/ZX0).
This format acknowledgment is separate from the copyright and license of
the KITAQ implementation. See [LICENSE](../../LICENSE) and
[LICENSE.ja](../../LICENSE.ja).

Build the host tool from the repository root:

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

The host tool uses .NET Framework 4.x. The compressor uses a bounded
hash-chain search; it does not guarantee the smallest possible output.
Backward streams, external prefix dictionaries and ZX0 v1 are outside
this interface. Rights in the original assets remain with their authors.

Input and encoded output must each fit 65535 bytes. `--format=auto` compares
raw, RLE and ZX0 payloads and wraps the smallest in a nine-byte KQA1 header;
the header counts toward the output limit. Use `asset_decompress` for KQA1.
Split assets to fit actual CPU bank windows and RAM capacity as well.
An empty raw C header contains one placeholder byte with logical `_SIZE` 0.

Include `zx0.h` and compile `lib/zx0.c`. `zx0_decompress` accepts a
destination, destination capacity, compressed source and compressed size.
Check both the returned byte count and `zx0_error`. An error may leave
partial output. Source and destination must not overlap or cross their
currently mapped CPU bank windows. These routines use shared scratch and
must not be re-entered from interrupts.

`zx0_decompress_vram` requires a RAM workspace large enough for the entire
decoded asset. With rendering disabled, it decodes into that workspace
and uploads the result to CHR RAM or nametable memory. It preserves
PPUCTRL; set the scroll position before enabling rendering. It does not
upload palettes or write CHR ROM.

## 日本語

PC側の圧縮器とFC側の展開器は、ZX0 v2の順方向ストリームに対応するKITAQの
独自実装です。KITAQ実装の著作権表示はCopyright (c) 2026 DAISUKE OBAで、
MITライセンスを適用します。

ZX0の圧縮形式と元の圧縮アルゴリズムの設計者は
[Einar Saukas氏](https://github.com/einar-saukas/ZX0)です。
形式への謝辞とKITAQ実装の著作権・ライセンスを分けて記載しています。
[LICENSE](../../LICENSE)と[LICENSE.ja](../../LICENSE.ja)も参照してください。

上のPowerShellコマンドは、順にツールのビルド、ZX0ファイルの作成、
Cヘッダーの作成、PC上での展開を行います。実行には.NET Framework 4.xを使います。
圧縮には探索回数を制限したハッシュチェーンを使い、最小サイズは保証しません。
逆方向展開、外部辞書、ZX0 v1は対象外です。素材自体の権利は素材の作者に帰属します。

入力と出力はそれぞれ65535バイト以内に収めます。`--format=auto`ではraw・RLE・
ZX0のペイロードを比較し、最小のものに9バイトのKQA1ヘッダーを付けます。
ヘッダーも出力上限に含みます。KQA1には`asset_decompress`を使ってください。
実際のCPUバンクやRAM容量にも合わせて素材を分割します。空のrawのCヘッダーは
保存領域を1バイト用意しますが、論理的な`_SIZE`は0です。

ターゲット側では`zx0.h`を読み込み、`lib/zx0.c`もコンパイルしてください。
`zx0_decompress`には出力先、出力容量、入力元、圧縮バイト数を指定し、戻り値と
`zx0_error`を確認します。失敗時は途中まで書き込まれる場合があるため、出力を
使わないでください。入力と出力は重ねず、現在のCPUバンクの境界をまたがない
ようにします。共有作業領域を使うため、割り込みから再入できません。

`zx0_decompress_vram`には、展開後の素材全体を置けるRAM作業領域が必要です。
描画を停止した状態でRAMへ展開し、CHR RAMまたはネームテーブルへ転送します。
PPUCTRLは保持します。描画を再開する前にスクロール位置を設定してください。
パレットへの転送やCHR ROMへの書き込みには使えません。
