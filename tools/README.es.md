# Herramientas auxiliares nativas

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md)

Todas las herramientas están implementadas en Rust y se ejecutan sin .NET, Python ni Pillow. Compile todos los ejecutables con `cargo build --locked --release`. En Windows, añada `.exe` a los comandos.

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

ZX0 acepta de 1 a 65535 bytes y conserva la salida del codificador C#. Admite `raw`, `rle` cantidad/valor y el contenedor automático `KQA1` de nueve bytes. En caso de empate: raw, RLE y ZX0. `--decompress` decodifica un flujo ZX0 v2 hacia delante con límite de salida. No admite flujos inversos ni v1. Einar Saukas diseñó el formato; esta implementación KITAQ tiene licencia MIT.

Los manifiestos JSON existentes generan CHR de 8 KiB, arrays C/cabeceras e informes JSON. Los metasprites admiten data, frames o archivos JSON. El convertidor PNG indexado genera CHR, nametable, atributos, paleta y JSON de metasprites. Fondos: 256×240; tiles: 8×8; grupos de paleta uniformes por tile y cuadrante de fondo de 16×16. Índices 0–15, profundidades 1/2/4/8 y Adam7. El script PNG ausente se reconstruyó a partir de la especificación local y los manifiestos de ejemplo; no puede compararse con el original no disponible. Los RGB se asignan a una paleta NES aproximada definida. El pipeline ejecuta ambas etapas internamente. El comprobador de nombres lee una lista externa y solo muestra archivo/línea, sin términos protegidos, excluyendo directorios generados.
