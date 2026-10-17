# FCSEQ 0.4.1 — manual

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

Autor: **DAISUKE OBA**. Secuenciador musical Famicom de uso general con **plita**. GUI, CLI y bindings Python comparten modelo, audio y exportadores. Se inicia con un proyecto vacío; no requiere ROM de juego ni BIOS. La interfaz está en inglés y el manual en nueve idiomas.

## Instalación

Descargue el ZIP completo de [downloads/](downloads/): Windows x86_64, Linux x86_64, macOS Apple Silicon ARM64 o Intel x86_64. Extraiga todo, incluidos recursos y licencias. Ejecute `fcseq.exe` en Windows o `./fcseq` en Unix; terminal: `fcseq-cli` (añada `.exe` en Windows). También hay ejecutables individuales en [bin/](bin/). Windows 10/11 usa Direct3D 12; macOS, Metal; Linux, Vulkan/OpenGL, X11/Wayland y ALSA. Linux se compila en Ubuntu 24.04; sistemas anteriores pueden necesitar compilación local. En Ubuntu: `sudo apt install libasound2t64 libxkbcommon0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. GUI necesita controlador gráfico compatible y la escucha un dispositivo de audio; WAV funciona sin dispositivo. Los binarios no están firmados y macOS no está notarizado. Si se bloquean, compruebe fuentes y hashes y use la autorización normal del sistema. Si hace falta: `chmod +x fcseq fcseq-cli`.

## Edición

SOUND elige 2A03 (Pulse 1/2, Triangle, Noise, DPCM: 5 voces), VRC6 (+2 pulse y saw: 8), VRC7 (+6 FM: 11), FDS (+1 onda: 6), N163 (+1–8 ondas: hasta 13). Una expansión por proyecto; se rechazan cambios que pierdan notas.

Doble clic añade una nota; arrastre el cuerpo para mover/transponer y el borde derecho para cambiar duración. Ctrl/Shift-clic o rectángulo selecciona varias; Ctrl+A selecciona la pista. Ctrl+C/X/V copia/corta/pega en el cursor; Ctrl+D duplica; Delete elimina; Ctrl+Z/Y deshace/rehace. Flechas mueven/transponen, Shift+arriba/abajo cambia octava. G: rejilla, Q: cuantizar, F: ajustar vista, Home: inicio, +/-: zoom, Shift+rueda: tiempo, UP/DOWN: canales. Space inicia/detiene; LOOP repite; AUDITION escucha selección. MUTE/SOLO afectan también exportaciones. SONG gestiona hasta 16 secciones de 256 compases; haga clic en nombre, BPM o compases. Ctrl+N/O/S/Shift+S: nuevo/abrir/guardar/guardar como. Los cambios inválidos se cancelan enteros. Recuperación: abra `out/recovery_<process-id>.fcq` junto al ejecutable y guarde con otro nombre.

## Instrumentos

Abra SOUND > Instrument / waveform editor, seleccione notas del canal y ajuste parámetros; AUDITION/UNDO/REDO permiten escuchar/deshacer/rehacer. Pulse tiene 4 ciclos de trabajo; VRC6 Pulse, 8. Triangle usa una onda fija de 32 pasos; Noise tiene 16 períodos y modo largo/corto. Volumen pulse/ruido: 0–15; caída 0 mantiene volumen. VRC6 Saw usa incremento de acumulador 0–63; el desbordamiento cambia el timbre. No tienen RAM para dibujar ondas libres.

FDS: una onda compartida de 64 muestras y 6 bits. N163: ondas de 4 bits, inicialmente 4 × 32; cada nota elige ranura, longitud múltiplo de 4. Capacidad compartida: `256 - 16 × canales activos` muestras (128 con ocho voces). Acorte otra onda o reduzca canales antes de excederla. Dibujo, SINE/PULSE/SAW/TRIANGLE, inversión y rotación. Se validan límites de tono/RAM/canales. VRC7: patch 0 personalizado con 24 parámetros (multiplicadores, ADSR, realimentación, nivel, escalado de teclado, trémolo/vibrato, seno/medio seno) u 8 bytes. Presets 1–15 son aproximaciones sintéticas, sin volcado de ROM. No hay editor específico de modulación/envolvente FDS. La emulación no garantiza equivalencia analógica.

## DPCM

SAMPLES o SOUND > Import audio file acepta WAV/FLAC/Ogg Vorbis/MP3, también arrastrando archivos. Ajuste START/END en ms, GAIN, NORMALIZE, REVERSE, RATE 0–15; escuche con AUDITION DPCM, guarde con EXPORT .DMC. Coloque una nota DPCM y elija SAMPLE y RATE de reproducción. Se conserva el PCM original; entrada máxima 60 s, estéreo a mono, eliminación de continua y remuestreo con filtro paso bajo. Cada muestra: `16n+1` bytes, máximo 4081; hasta 64 muestras y 16320 bytes de banco (8192 en FDS). Se rechazan excesos. Tasa de conversión y reproducción son distintas. No hay bucle interno ni corrección automática de tono.

## Archivos y exportación

Guarde `.fcq`; renombre `.fcseq` antiguos. Notas: JSON `fcseq-2`, también se lee `fcseq-1`; reproducción NSF: `fcseq-3`. Abra NSF/NSFe con Import NSF, OPEN, arrastre o argumento de inicio. PREV/NEXT o número selecciona canción; PLAY/STOP, LOOP y duración 1–600 s controlan reproducción. Se usa duración de metadatos o 30 s por defecto. Puede cancelar la generación previa de audio. FCQ incorpora original, selección y duración; no transcribe automáticamente a notas. Reexportar conserva bytes originales.

Export NSF convierte cada sección en una canción con instrumentos/DPCM; genera además NSFe con duración y fundido 0. Use NSFe si el reproductor ignora tiempos NSF. Se detiene al terminar la sección; LOOP del editor es independiente. Entrada: NSF v1/NSFe, NTSC o región dual compatible NTSC, sin bancos o bancos de 4 KiB, 2A03/DPCM, VRC6/VRC7/FDS/N163. Se rechazan PAL exclusivo, extensiones de ejecución NSF2, MMC5, Sunsoft 5B y VT02; INIT/PLAY sin retorno tiene límite.

Export C + DPCM genera `score.c`, `fcseq_score.h`, `fcseq_player.c`, binarios de sección, `.dmc`, `dpcm_bank.bin`, `fcseq_export.json`. Debe integrar mapper y direcciones de código/datos/DPCM visibles a CPU; no crea una ROM completa. VRC6 supone mapper 24 (26 requiere traducir direcciones), VRC7 85, N163 19 o FDS. Para KITAQFC defina `FCSEQ_KITAQFC` antes de incluir; `FCSEQ_WRITE(address, value)` permite escritura propia.

## CLI y Python

Ejecute ejemplos en la carpeta extraída. Índices de sección/canal/nota/muestra desde 0; canciones NSF desde 1. Python 3.10+ usa extensión abi3 incluida para mismo sistema/CPU. GUI/CLI no necesitan Python. API completa: `fcseq_native.pyi`. CLI mueve, transpone, cambia duración, cuantiza, duplica y elimina notas; ondas se editan mediante GUI/Python.

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

## Compilación y verificación

En esta carpeta de fuentes use Rust 1.93.1 y los comandos siguientes. Bibliotecas locales: vendor/. Paquetes de desarrollo Linux: `.github/workflows/fcseq.yml` en la raíz; macOS: Xcode Command Line Tools; Windows: MSVC Build Tools. Python es opcional. Los scripts comprueban CLI/Python nativos, WAV, ida y vuelta NSF/NSFe e inicio GUI. Resultados/límites: VERIFICATION.md. No certifican interacción humana con ratón/teclado ni escucha de hardware real.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python
```

## Licencia

FCSEQ usa MIT, Copyright (c) 2026 DAISUKE OBA. Dependencias conservan sus licencias. Redistribuya LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ y third_party_sources/. Se incluyen fuentes Symphonia MPL-2.0 y avisos de fuentes/iconos. Música y muestras importadas tienen derechos separados.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)
