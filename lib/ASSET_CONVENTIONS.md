# KITAQFC asset conventions (phase15)

## ディレクトリ推奨
- `assets/raw_png/` : indexed PNG 入力
- `assets/generated_png/` : PNG 前段 tool の出力
- `assets/generated_pipeline/` : pack 後の C / H / CHR

## indexed PNG ルール
- mode は `P` の indexed PNG
- BG は 256x240
- sprite sheet は幅・高さとも 8 の倍数
- index 0..15 を 4 palette x 4 color の grouped-4 とみなす
  - group = `index >> 2`
  - shade = `index & 3`

## BG / attribute ルール
- 1 tile 内で palette group を混在させない
- 16x16 quadrant 内でも palette group を混在させない
- attr は quadrant ごとに自動生成する

## metasprite JSON ルール
```json
{
  "name": "player",
  "frames": [
    [[0,0,128,0],[8,0,129,0],[0,8,130,0],[8,8,131,0]],
    [[0,0,132,0],[8,0,133,0],[0,8,134,0],[8,8,135,0]]
  ]
}
```

## pack manifest ルール
- raw `.chr/.nam/.atr/.pal` を packer に渡す
- metasprite は inline か `input` JSON を使う
- final CHR blob は `--nes-chr` で compiler に渡す
