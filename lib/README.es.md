# Organización de las bibliotecas estándar FC/NES

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | **Español** | [Deutsch](README.de.md) | [Português (Brasil)](README.pt-BR.md)
<!-- readme-language-links:end -->

<!-- fc-current-20261004:es:start -->
### Distribución actual del compilador y las bibliotecas — 4 de octubre de 2026

fc.h incluye las declaraciones de physics3d y audio_vblank. Enlace por separado los archivos de implementación necesarios. entity_update_all y entity_draw_all llaman a la función de retorno para las posiciones activas en orden ascendente de ID. Coloque esa función en el banco común 0 o mantenga su banco PRG seleccionado durante las llamadas. Incluya zx0.h explícitamente para usar ZX0. El orden de los registros audio_vblank es delay, CH1, CH2, CH3, CH4.

Se comparó la compilación C# actual con la distribución anterior mediante 29 entradas. Quince entradas aceptadas generaron los mismos bytes ROM; las otras catorce produjeron los mismos diagnósticos de rechazo. Dos ROM adicionales se ejecutaron durante cuatro fotogramas en KUROSAKI: asignación de entidades, llamadas y reutilización de posiciones; aritmética con signo, disposición de KQBody3D y constantes de la cola de audio. El informe recoge 18 bytes RAM esperados y observados. Estas pruebas no comprueban la reproducción de audio ni todas las API; las regresiones anteriores se identifican por separado en BINARY_BUILD.json.

[Ejemplos fuente y pruebas de ejecución](https://bartaro.github.io/kitaq-docs/es/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:es:end -->


**[KITAQFC · Manual HTML](https://bartaro.github.io/kitaq-docs/es/fc-library.html)**

Las bibliotecas públicas siguen el estilo de KITAQGB: nombres breves que describen su función, sin el prefijo `kitaqfc_`.

## Funciones básicas

- `fc.h`: cabecera que reúne las demás interfaces.
- `core.h`: tipos básicos.
- `intrinsics.h`: puntos de entrada de las operaciones intrínsecas reconocidas por el compilador.
- `runtime.h` / `runtime.c`: soporte de ejecución para NMI, el búfer espejo de OAM y la cola de VRAM.
- `system.h` / `system.c`: funciones de fotograma y espera con el estilo de KITAQGB.
- `debug.h` / `debug.c`: registros ligeros de trazas y aserciones en RAM.

## Gráficos

- `vram.h` / `vram.c`: funciones al estilo de KITAQGB para encolar actualizaciones de VRAM.
- `sprite.h` / `sprite.c`: asignación de sprites y metasprites al estilo de KITAQGB.
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
- `oam_fair.h` / `oam_fair_impl.h`: rotación del orden de los 64 candidatos de OAM respetando sus prioridades; véase [oam_fair.md](oam_fair.md).

## Estructura del juego

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## Audio y periféricos

- `audio.h` / `audio.c`
- `audio_vblank.h` / `audio_vblank.c` — Música durante NMI con una cola BGM de siete registros, pausa y reanudación, un búfer SFX independiente de siete registros y envolventes de ruido de un solo disparo. [Manual HTML](https://bartaro.github.io/kitaq-docs/es/fc-library.html#module-audio_vblank)
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c`: capa compatible con la API de estados de botones de KITAQGB.
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## Mapeadores y FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## Cálculo numérico

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` — Integración de cuerpos rectangulares, gravedad, contactos AABB, respuesta de superficies y tipos Q5.3 opcionales. Compila con `fixed.c`.
- `physics3d.h` / `physics3d.c` — Cajas 3D sin rotación, rebote ponderado por la masa y valores de intensidad del impacto. Compila con `fixed.c` y `physics2d.c`.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

Las transiciones de escena, las actualizaciones y el dibujo llaman de forma síncrona a los manejadores registrados. Consulta cada API para conocer el orden de los callbacks y las restricciones de reentrada.
