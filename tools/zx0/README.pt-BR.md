# Ferramentas auxiliares nativas

[en](../README.en.md) · [ja](../README.ja.md) · [ko](../README.ko.md) · [zh-CN](../README.zh-CN.md) · [zh-TW](../README.zh-TW.md) · [fr](../README.fr.md) · [es](../README.es.md) · [de](../README.de.md)

Todas as ferramentas são implementadas em Rust e executam sem .NET, Python ou Pillow. Compile todos os executáveis com `cargo build --locked --release`. No Windows, acrescente `.exe` aos comandos.

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

ZX0 aceita de 1 a 65535 bytes e preserva a saída do codificador C#. Suporta `raw`, `rle` contagem/valor e o contêiner automático `KQA1` de nove bytes. Em empate, prefere raw, RLE e ZX0. `--decompress` decodifica um fluxo ZX0 v2 direto com limite de saída. Fluxos reversos e v1 não são suportados. Einar Saukas criou o formato; esta implementação KITAQ usa a licença MIT.

Os manifestos JSON existentes geram CHR de 8 KiB, arrays C/cabeçalhos e relatórios JSON. Metasprites aceitam data, frames ou arquivos JSON. O conversor PNG indexado gera CHR, nametable, atributos, paleta e JSON de metasprites. Fundos: 256×240; tiles: 8×8; grupos de paleta uniformes por tile e quadrante de fundo de 16×16. Índices 0–15, profundidades 1/2/4/8 e Adam7. O script PNG ausente foi reconstruído da especificação local e dos manifestos de exemplo; não é possível comparar bytes com o original indisponível. Os RGB usam uma paleta NES aproximada definida. O pipeline executa as duas etapas internamente. O verificador de nomes lê uma lista externa e informa apenas arquivo/linha, sem termos protegidos, excluindo diretórios gerados.

```sh
sh tools/zx0/build.sh
```
