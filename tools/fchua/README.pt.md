# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA edita tiles, tabelas de nomes, paletas e metasprites NES de 8x8 para KITAQFC. A CLI independente compartilha o núcleo da interface e dispensa GUI, GPU, instalação do PLITA e Python.

## Compilar e executar

A compilação exige Rust 1.92 ou superior. No Windows execute target/release/fchua.exe; no Linux e macOS, ./target/release/fchua. Consulte --help. Os exemplos usam fchua como nome do executável. Crie primeiro as pastas superiores da saída.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## Fluxo de trabalho

Crie um projeto vazio ou importe PNG; confira o relatório e a prévia antes de exportar. --auto-palette escolhe quatro paletas NES de fundo reduzindo o erro; sem a opção, usa as atuais. --project base.fch fornece paletas e tabelas. Pixels transparentes são compostos sobre o fundo universal atual. A importação substitui o mapa e a tabela de fundo selecionada, alterando também sprites que compartilhem essa tabela.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## Referência de comandos

```text
new --out FILE.fch [--demo] [--force]
inspect FILE.fch
validate FILE.fch [--strict]
import-image IMAGE.png --out FILE.fch [--project BASE.fch] [--auto-palette] [--force]
import-raw --out FILE.fch [--project BASE.fch] [--chr FILE.chr]
           [--palette FILE.pal] [--nametable FILE.nam] [--attributes FILE.atr] [--force]
apply INPUT.fch --script EDITS.json --out OUTPUT.fch [--dry-run] [--force]
preview FILE.fch --out IMAGE.png [--view map|tiles|sprite|tile] [--scale 1..8] [--force]
export FILE.fch --out DIRECTORY [--prefix C_IDENTIFIER] [--force]
migrate OLD.kfcproject --out FILE.fch [--force]
```

## Edição em lote

O script é uma matriz JSON. Todas as operações são validadas antes da atualização; uma falha não grava nada. --dry-run apenas valida e informa. Operações e campos desconhecidos são erros. Tiles usam índices absolutos 0..511; mapas e sprites usam 0..255 na tabela selecionada.

```json
[
  {"op":"pixel","tile":1,"x":2,"y":3,"color":1},
  {"op":"fill","tile":2,"color":0},
  {"op":"checker","tile":3,"a":1,"b":2},
  {"op":"line","tile":4,"x0":0,"y0":0,"x1":7,"y1":7,"color":3},
  {"op":"rect","tile":5,"x0":1,"y0":1,"x1":6,"y1":6,"color":2},
  {"op":"mirror","tile":1,"horizontal":true,"vertical":false},
  {"op":"rotate","tile":1},
  {"op":"copy","source":1,"target":6},
  {"op":"palette","index":1,"value":48},
  {"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0},
  {"op":"tables","background":0,"sprites":1},
  {"op":"sprite","frame":0,"index":0,"x":-8,"y":0,"tile":1,"attr":64},
  {"op":"clone_frame","frame":0},
  {"op":"remove_sprite","frame":1,"index":0},
  {"op":"shift_map","dx":2,"dy":0},
  {"op":"dedupe"}
]
```

`tile: 0..511; x/y (pixel): 0..7; color: 0..3; palette index: 0..31; value: 0..63`

`map: 32x30; palette: 0..3; tables: 0..1; frame: 0..255; index: 0..63`

`attr: palette bits 0..1, priority 0x20, flip H 0x40, flip V 0x80; reserved: 0x1c`

`shift_map: dx=-32..32, dy=-30..30; dx % 2 == 0, dy % 2 == 0`

## Interface e Python

A GUI PLITA local usa Roboto, menus agrupados com ícones Lucide, estados de apontar/pressionar/soltar e dicas. File oferece criar, abrir, salvar, salvar como, importar PNG e exportar. Ctrl+O/S/Shift+S abre/salva/salva como; Ctrl+Z/Y desfaz/refaz; Ctrl+C/V copia/cola um tile. Apenas selecionar tiles ou ferramentas não marca alterações. Ao fechar, criar ou abrir outro projeto, alterações pendentes exigem confirmação para descarte.

Map/Tile/Sprite/Check trocam a visualização central. B/E/F/L/R/I escolhem lápis/borracha/preenchimento/linha/retângulo/conta-gotas; 1..4 escolhem a cor. Um arrasto corresponde a um desfazer; Escape cancela. Use Fit, zoom, roda e barras com alças e trilhos. Em janelas estreitas, Tiles/Palette alterna a lateral. Escolha uma amostra e depois uma cor NES; zero altera o fundo universal. Sprite coloca tiles na grade de 8 pixels; conta-gotas seleciona e borracha remove. O menu adiciona/clona quadros e espelha/remove a peça selecionada. Check verifica quadros individuais, não a cena completa nem emuladores.

O pacote CLI inclui bindings Python opcionais (Python 3.10+), sem GUI, GPU ou PLITA. Compile com pip em tools/fchua ou instale o wheel Windows x64 incluído. Controller oferece projetos, PNG, comandos transacionais e exportação RAW/C; GUI_AVAILABLE=False. A versão PLITA local acrescenta run_editor e ButtonInteraction. Controller edita seu próprio documento, não uma GUI aberta; troque arquivos .fch. Salvar pelo Python substitui arquivos existentes, diferente da proteção --force da CLI.

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Local PLITA workspace
cargo run -p kitaqfc_graphics_editor --bin fchua_gui
python -m pip install ./examples/kitaqfc_graphics_editor
```

```python
import fchua
editor = fchua.Controller()
editor.new_blank()
editor.apply_commands('[{"op":"fill","tile":1,"color":2},{"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0}]')
editor.save_project("scene.fch")
editor.preview_png("preview.png", "map", 2)
editor.export_raw_assets("assets", "scene")
print(fchua.GUI_AVAILABLE, editor.snapshot().unique_tiles)
```

## Hardware e arquivos

.fch é JSON versionado (format=fchua, version=1), contendo CHR de 8192 bytes, paleta de 32, mapa 32x30, 64 atributos, duas seleções de tabelas e até 256 quadros. É possível migrar .kfcproject; atributos de hardware prevalecem sobre mapas de paleta inconsistentes. PNG deve medir múltiplos de 8 entre 8x8 e 256x240. Cada bloco 16x16 usa uma paleta de quatro cores. A tabela de fundo aceita 256 tiles; a importação reserva o zero vazio e rejeita excessos.

Exporta .chr (8192), .pal (32), .nam (960), .atr (64), JSON de metasprites, matrizes C/H de todos os quadros, manifesto e .fch. C usa __prg_rom. Configure as tabelas PPU no jogo de acordo com o projeto; o pacote não gera essa lógica. CHR bruto deve conter tiles inteiros de 16 bytes até 8192; os outros formatos exigem tamanhos exatos. Cores NES: 0..63. Limite de 64 peças por quadro, coordenadas -128..127 exceto x=-1, reservado ao terminador 0xFF. Apenas sprites 8x8; mais de oito peças na mesma linha gera aviso.

## Proteção e erros

Saída existente exige --force, mas entrada e saída devem ter caminhos diferentes. Projetos são substituídos atomicamente usando arquivo temporário. export --out é uma pasta; --force substitui os recursos nomeados sem apagar arquivos alheios. --prefix precisa ser identificador C. Sucesso imprime um objeto JSON; erros imprimem JSON em stderr. Códigos: 0 sucesso, 1 avisos com validate --strict, 2 erro de argumentos, projeto, conversão ou E/S. Faça backup antes de substituir. Não há chamadas de rede ou geração de IA.

Código original: MIT. Dependências mantêm suas licenças; consulte LICENSE e THIRD_PARTY_NOTICES.md.
