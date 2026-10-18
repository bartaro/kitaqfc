# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA edita tiles, tabelas de nomes, paletas e metasprites NES de 8x8 para KITAQFC. A CLI independente compartilha o núcleo da interface e dispensa GUI, GPU, instalação do plita e Python.

## Download da interface

Baixe o ZIP para Windows x64, extraia tudo e execute FCHUA.exe. Mantenha assets/fonts e as pastas de licenças ao lado do executável. Não é necessário Rust nem Python para executar, mas é preciso um dispositivo/driver gráfico compatível com wgpu. A interface oferece japonês e inglês; os manuais completos estão nos nove idiomas acima.

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

Comece por File > New project ou Open project / raw. Selecione tile e paleta, desenhe em Tile e coloque tiles em Map. A paleta do mapa muda em blocos de 16x16 pixels. Selecione as tabelas de fundo/sprites nos respectivos menus. Salve em .fch para manter os dados editáveis; a prévia PNG não é um projeto. Export KITAQFC pack gera os recursos do jogo. Se a fonte estiver diferente, confira a extração completa e PLITA_UI_FONT. Se a janela não abrir, atualize o driver gráfico ou use a CLI sem interface.

O código público inclui os crates necessários do plita. Em tools/fchua, compile e execute a interface com os comandos abaixo; o caminho final do projeto é opcional. O executável GUI desta versão foi verificado no Windows x64.

Alterne imediatamente pelo ícone de idioma no canto superior direito ou Edit > 日本語 / English. A escolha é mantida na próxima execução. O primeiro início segue o idioma do sistema; sistemas japoneses iniciam em japonês. FCHUA_LANGUAGE=ja ou en tem prioridade sobre a preferência salva. language.txt fica em %APPDATA%/FCHUA no Windows, ~/Library/Application Support/FCHUA no macOS e $XDG_CONFIG_HOME/fchua (ou ~/.config/fchua) no Linux. FCHUA_CONFIG_DIR substitui essa pasta. Roboto continua como fonte principal e Noto Sans JP, incluída, fornece os caracteres japoneses; PLITA_UI_FONT_FALLBACK pode substituí-la. A troca preserva projeto e histórico de desfazer. Comandos CLI e nomes da API Python não mudam.

A compilação de distribuição, os layouts por software e os testes automáticos de entrada/estado passaram no Windows x64. A janela nativa do FCHUA foi criada, mas a captura de tela excedeu o tempo nas duas tentativas. O conteúdo nativo, os cliques e os diálogos de arquivos permanecem não verificados. Veja a [verificação](VERIFICATION.md).

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

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

A GUI plita usa Roboto, menus agrupados com ícones Lucide, estados de apontar/pressionar/soltar e dicas. File oferece criar, abrir, salvar, salvar como, importar PNG e exportar. Ctrl+O/S/Shift+S abre/salva/salva como; Ctrl+Z/Y desfaz/refaz; Ctrl+C/V copia/cola um tile. Apenas selecionar tiles ou ferramentas não marca alterações. Ao fechar, criar ou abrir outro projeto, alterações pendentes exigem confirmação para descarte.

Map/Tile/Sprite/Check trocam a visualização central. B/E/F/L/R/I escolhem lápis/borracha/preenchimento/linha/retângulo/conta-gotas; 1..4 escolhem a cor. Um arrasto corresponde a um desfazer; Escape cancela. Use Fit, zoom, roda e barras com alças e trilhos. Em janelas estreitas, Tiles/Palette alterna a lateral. Escolha uma amostra e depois uma cor NES; zero altera o fundo universal. Sprite coloca tiles na grade de 8 pixels; conta-gotas seleciona e borracha remove. O menu adiciona/clona quadros e espelha/remove a peça selecionada. Check verifica quadros individuais, não a cena completa nem emuladores.

O pacote CLI inclui bindings Python opcionais (Python 3.10+), sem GUI, GPU ou plita. Compile com pip em tools/fchua ou instale o wheel Windows x64 incluído. Controller oferece projetos, PNG, comandos transacionais e exportação RAW/C; GUI_AVAILABLE=False. A versão com recursos gui,python acrescenta run_editor e ButtonInteraction. Controller edita seu próprio documento, não uma GUI aberta; troque arquivos .fch. Salvar pelo Python substitui arquivos existentes, diferente da proteção --force da CLI.

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Optional GUI-enabled Python build, also from tools/fchua
python -m pip install maturin
python -m maturin build --release --locked --features gui,python --out gui-wheels
# Install the wheel generated in gui-wheels instead of the headless wheel.
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
