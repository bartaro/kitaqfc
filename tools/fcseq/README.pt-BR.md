# FCSEQ 0.4.1 — manual

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

Autor: **DAISUKE OBA**. Sequenciador musical Famicom de uso geral com **plita**. GUI, CLI e bindings Python compartilham modelo, áudio e exportadores. O projeto inicial é vazio; não exige ROM de jogo nem BIOS. A interface está em inglês; o manual possui nove idiomas.

## Instalação

Baixe o ZIP completo em [downloads/](downloads/): Windows x86_64, Linux x86_64, macOS Apple Silicon ARM64 ou Intel x86_64. Extraia tudo, incluindo recursos e licenças. Execute `fcseq.exe` no Windows ou `./fcseq` no Unix; terminal: `fcseq-cli` (adicione `.exe` no Windows). Há executáveis individuais em [bin/](bin/). Windows 10/11 usa Direct3D 12; macOS, Metal; Linux, Vulkan/OpenGL, X11/Wayland e ALSA. Linux é compilado no Ubuntu 24.04; distribuições antigas podem precisar de compilação local. No Ubuntu: `sudo apt install libasound2t64 libxkbcommon0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. GUI exige driver gráfico compatível e reprodução exige dispositivo de áudio; WAV funciona sem dispositivo. Binários não são assinados e macOS não é notarizado. Se bloqueados, confira fontes e hashes e use a autorização normal do sistema. Se necessário: `chmod +x fcseq fcseq-cli`.

## Edição

SOUND escolhe 2A03 (Pulse 1/2, Triangle, Noise, DPCM: 5 vozes), VRC6 (+2 pulse e saw: 8), VRC7 (+6 FM: 11), FDS (+1 onda: 6), N163 (+1–8 ondas: até 13). Uma expansão por projeto; mudanças que perdem notas são recusadas.

Duplo clique adiciona nota; arraste o corpo para mover/transpor e a borda direita para mudar duração. Ctrl/Shift-clique ou retângulo seleciona várias; Ctrl+A seleciona a pista. Ctrl+C/X/V copia/recorta/cola no cursor; Ctrl+D duplica; Delete apaga; Ctrl+Z/Y desfaz/refaz. Setas movem/transpõem; Shift+cima/baixo muda oitava. G: grade, Q: quantização, F: ajustar vista, Home: início, +/-: zoom, Shift+roda: tempo, UP/DOWN: canais. Space inicia/para; LOOP repete; AUDITION ouve seleção. MUTE/SOLO afetam exportações. SONG gerencia até 16 seções de 256 compassos; clique em nome, BPM ou compassos. Ctrl+N/O/S/Shift+S: novo/abrir/salvar/salvar como. Edições inválidas são canceladas por inteiro. Recuperação: abra `out/recovery_<process-id>.fcq` junto ao programa e salve com outro nome.

## Instrumentos

Abra SOUND > Instrument / waveform editor, selecione notas do canal e ajuste parâmetros; use AUDITION/UNDO/REDO. Pulse tem 4 ciclos de trabalho; VRC6 Pulse, 8; Triangle, onda fixa de 32 passos; Noise, 16 períodos e modo longo/curto. Volume pulse/ruído: 0–15; decaimento 0 mantém volume. VRC6 Saw usa incremento de acumulador 0–63; transbordamento muda timbre. Essas fontes não têm RAM de onda livre.

FDS: uma onda compartilhada de 64 amostras, 6 bits. N163: ondas de 4 bits, inicialmente 4 × 32; notas escolhem posição, comprimento múltiplo de 4. Capacidade: `256 - 16 × canais ativos` amostras (128 com oito vozes). Encurte outra onda ou reduza canais antes de exceder. Desenho, SINE/PULSE/SAW/TRIANGLE, inversão e rotação. Limites de altura/RAM/canais são validados. VRC7: patch personalizado 0 com 24 parâmetros (multiplicadores, ADSR, realimentação, nível, escala de teclado, tremolo/vibrato, seno/meio seno) ou 8 bytes. Presets 1–15 são aproximações sintéticas, não dumps de ROM. Sem editor específico de modulação/envelope FDS. Emulação não garante equivalência analógica.

## DPCM

SAMPLES ou SOUND > Import audio file aceita WAV/FLAC/Ogg Vorbis/MP3, também por arrastar. Ajuste START/END em ms, GAIN, NORMALIZE, REVERSE, RATE 0–15; ouça com AUDITION DPCM, salve com EXPORT .DMC. Insira nota DPCM e escolha SAMPLE e RATE de reprodução. PCM original é preservado; entrada até 60 s, estéreo convertido em mono, remoção de componente contínua e reamostragem com filtro passa-baixas. Amostra: `16n+1` bytes, máximo 4081; até 64 amostras e banco de 16320 bytes (8192 em FDS). Excesso é recusado. Taxa de conversão e reprodução são distintas. Sem loop interno ou correção automática de altura.

## Arquivos e exportação

Salve `.fcq`; renomeie `.fcseq` antigos. Notas usam JSON `fcseq-2`, com leitura de `fcseq-1`; documentos NSF usam `fcseq-3`. Abra NSF/NSFe por Import NSF, OPEN, arrastar ou argumento inicial. PREV/NEXT ou número escolhe música; PLAY/STOP, LOOP e duração 1–600 s controlam reprodução. Metadados de duração são usados ou 30 s por padrão. Geração prévia de áudio pode ser cancelada. FCQ incorpora original, música e duração; não transcreve automaticamente notas. Reexportação preserva bytes originais.

Export NSF converte cada seção em uma música com instrumentos/DPCM e gera NSFe com durações e fade 0. Use NSFe se o player ignora tempos NSF. Para no fim da seção; LOOP do editor é separado. Entrada: NSF v1/NSFe, NTSC ou região dupla compatível NTSC, sem bancos ou bancos 4 KiB, 2A03/DPCM, VRC6/VRC7/FDS/N163. PAL exclusivo, extensões de execução NSF2, MMC5, Sunsoft 5B e VT02 são recusados; INIT/PLAY sem retorno é limitado.

Export C + DPCM gera `score.c`, `fcseq_score.h`, `fcseq_player.c`, binários de seção, `.dmc`, `dpcm_bank.bin`, `fcseq_export.json`. Integre mapper e endereços de código/dados/DPCM visíveis à CPU; não gera ROM completa. VRC6 pressupõe mapper 24 (26 exige tradução de endereços), VRC7 85, N163 19 ou FDS. No KITAQFC defina `FCSEQ_KITAQFC` antes de incluir; `FCSEQ_WRITE(address, value)` conecta escrita própria.

## CLI e Python

Execute exemplos na pasta extraída. Índices de seção/canal/nota/amostra começam em 0, músicas NSF em 1. Python 3.10+ usa extensão abi3 incluída para mesmo sistema/CPU. GUI/CLI dispensam Python. API completa: `fcseq_native.pyi`. CLI move, transpõe, redimensiona, quantiza, duplica e apaga notas; ondas são editadas via GUI/Python.

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## Compilação e verificação

Nesta pasta de fontes use Rust 1.93.1 e os comandos abaixo. Bibliotecas locais: vendor/. Pacotes de desenvolvimento Linux: `.github/workflows/fcseq.yml` na raiz; macOS: Xcode Command Line Tools; Windows: MSVC Build Tools. Python é opcional. Scripts verificam CLI/Python nativos, WAV, ida e volta NSF/NSFe e início GUI. Resultados/limites: VERIFICATION.md. Não certificam interação humana com mouse/teclado nem escuta em hardware real.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
```

## Licença

FCSEQ usa MIT, Copyright (c) 2026 DAISUKE OBA. Dependências mantêm suas licenças. Redistribua LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ e third_party_sources/. Fontes Symphonia MPL-2.0 e avisos de fontes/ícones estão incluídos. Músicas e amostras importadas têm direitos próprios.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)
