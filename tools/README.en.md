# Native helper tools

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md)

All helper executables use Rust. Running them requires neither .NET nor Python/Pillow. Build all executables with `cargo build --locked --release`; on Windows append `.exe` to each command.

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

ZX0 accepts 1–65535 input bytes and preserves the C# encoder output. `raw`, count/value `rle`, and the nine-byte `KQA1` automatic container are supported. Equal payload sizes favor raw, then RLE, then ZX0. `--decompress` accepts a bare forward ZX0 v2 stream, with bounded output; backward streams and v1 are unsupported. The format was designed by Einar Saukas; this KITAQ implementation is MIT licensed.

The asset packer reads the existing JSON manifests and emits an 8 KiB CHR image, C/header arrays, and a JSON report. Metasprites accept inline data, frames, or a JSON input. The indexed PNG builder emits planar CHR, nametable, attributes, palette and metasprite JSON; backgrounds are 256×240, sheets use 8×8 tiles, and palette groups must agree within a tile and each 16×16 background quadrant. PNG palette indices are 0–15. PNG depths 1/2/4/8 and Adam7 are supported. The missing original PNG script was reconstructed from the local specification and sample manifests; its bytes cannot be compared to an unavailable original. RGB colors map to the documented approximate NES palette. The pipeline runs both native stages internally. The name guard reads an external denylist and reports file/line locations without printing protected terms; generated directories are excluded.
