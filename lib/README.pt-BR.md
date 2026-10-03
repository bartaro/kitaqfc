# Organização da biblioteca padrão FC/NES

<!-- fc-current-20261004:pt:start -->
### Distribuição atual do compilador e das bibliotecas — 4 de outubro de 2026

fc.h inclui as declarações de physics3d e audio_vblank. Vincule separadamente os arquivos de implementação necessários. entity_update_all e entity_draw_all chamam o callback para as posições ativas em ordem crescente de ID. Coloque o callback no banco comum 0 ou mantenha seu banco PRG mapeado durante as chamadas. Inclua zx0.h explicitamente para usar ZX0. A ordem dos registros audio_vblank é delay, CH1, CH2, CH3, CH4.

A compilação C# atual foi comparada com a distribuição anterior usando 29 entradas. Quinze entradas aceitas produziram os mesmos bytes ROM; as outras quatorze produziram os mesmos diagnósticos de rejeição. Duas ROMs adicionais foram executadas por quatro quadros no KUROSAKI: alocação de entidades, callbacks e reutilização de posições; aritmética com sinal, disposição de KQBody3D e constantes da fila de áudio. O relatório registra 18 bytes RAM esperados e observados. Esses testes não verificam a reprodução de áudio nem todas as APIs; as regressões anteriores são identificadas separadamente em BINARY_BUILD.json.

[Exemplos de código-fonte e evidências de execução](https://bartaro.github.io/kitaq-docs/pt/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:pt:end -->


[English](README.md) | [日本語](README.ja.md) | **Português (Brasil)**

Abrir o manual da biblioteca KITAQFC em português

Os nomes públicos seguem o estilo da KITAQGB: são curtos, descrevem a função e não usam o prefixo `kitaqfc_`.

## Funções básicas

- `fc.h` - cabeçalho que reúne as inclusões da biblioteca.
- `core.h` - tipos básicos.
- `intrinsics.h` - pontos de entrada das operações intrínsecas reconhecidas pelo compilador.
- `runtime.h` / `runtime.c` - suporte de execução para NMI, cópia de trabalho da OAM e fila de VRAM.
- `system.h` / `system.c` - funções de quadro e espera no estilo da KITAQGB.
- `debug.h` / `debug.c` - pequeno registro em RAM para rastreamento e asserções.

## Gráficos

- `vram.h` / `vram.c` - funções no estilo da KITAQGB para enfileirar atualizações de VRAM.
- `sprite.h` / `sprite.c` - alocação de sprites e composição de metasprites no estilo da KITAQGB.
- `ppu.h` / `ppu.c`
- `ppu_direct.h`
- `vram_queue.h`
- `palette.h` / `palette.c`
- `scroll.h` / `scroll.c`
- `tilemap.h` / `tilemap.c`
- `nametable_asset.h` / `nametable_asset.c`
- `attribute.h` / `attribute.c`
- `metasprite.h` / `metasprite.c`
- `oam.h`
- `oam_fair.h` / `oam_fair_impl.h` - alternância da ordem de 64 candidatos da OAM, preservando as prioridades; consulte [oam_fair.md](oam_fair.md).

## Estrutura do jogo

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## Áudio e periféricos

- `audio.h` / `audio.c`
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` - camada de compatibilidade com os estados dos botões da KITAQGB.
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## Mappers e FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## Matemática

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` - tipos Q5.3 armazenados em bytes e constantes de ajuste. Mantêm separados a posição em pixels inteiros, a fração de 1/8 de pixel, a rapidez sem sinal, a direção e o arrasto. O jogo calcula a evolução da posição e da velocidade; o cabeçalho não fornece uma rotina que avance a simulação física. Separar os campos permite calcular o movimento nos trechos mais executados sem o custo de passar agregados ou ponteiros pela ABI.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

As funções de compatibilidade com a KITAQGB mantêm nomes funcionais curtos onde o hardware FC/NES permite. Atualmente, as funções de cena e entidade com formato de callback armazenam o estado, mas não fazem chamadas indiretas aos ponteiros de função do usuário.
