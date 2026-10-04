# ネイティブ補助ツール

[en](../README.en.md) · [ja](../README.ja.md) · [ko](../README.ko.md) · [zh-CN](../README.zh-CN.md) · [zh-TW](../README.zh-TW.md) · [fr](../README.fr.md) · [es](../README.es.md) · [de](../README.de.md)

すべての補助実行ファイルはRust実装です。実行時に.NET・Python・Pillowは不要です。`cargo build --locked --release`で全実行ファイルをビルドします。Windowsでは各コマンド名に`.exe`を付けます。

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

ZX0は1～65535バイトを受け付け、C#圧縮器の出力を保持します。`raw`、個数・値の`rle`、9バイトの`KQA1`自動選択コンテナに対応します。同サイズならraw、RLE、ZX0の順に選びます。`--decompress`は裸の順方向ZX0 v2ストリームを上限付きで展開します。逆方向ストリームとv1は対象外です。形式の設計者はEinar Saukas氏で、このKITAQ実装はMITライセンスです。

素材パックは既存JSONマニフェストから8 KiBのCHR、C配列・ヘッダー、JSONレポートを生成します。メタスプライトはdata・frames・JSON入力に対応します。indexed PNG変換はCHR・ネームテーブル・属性・パレット・メタスプライトJSONを生成します。背景は256×240、シートは8×8タイル単位で、タイル内と背景の16×16区画内でパレットグループを揃えます。PNGの色番号は0～15です。深度1/2/4/8とAdam7に対応します。元のPNGスクリプトが存在しないため、ローカル仕様書とサンプルマニフェストから機能を復元しました。その原本とのバイト比較はできません。RGB色は定義したNES近似パレットへ対応付けます。パイプラインは両段階を内部で実行します。名称検査は外部の禁止語リストを読み、語を表示せずファイルと行番号を報告します。生成ディレクトリは除外します。

```sh
sh tools/zx0/build.sh
```
