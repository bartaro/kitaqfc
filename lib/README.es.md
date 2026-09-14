# Organización de las bibliotecas estándar FC/NES

[English](README.md) | [日本語](README.ja.md) | **Español**

[Abrir el manual de las bibliotecas KITAQFC en español](https://bartaro.github.io/kitaq-docs/es/fc-library.html)

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
- `physics2d.h` / `physics2d.c`: tipos Q5.3 de un byte y constantes de ajuste. La posición entera en píxeles, la fracción de 1/8 de píxel, la rapidez sin signo, la dirección y el arrastre se guardan por separado. El juego calcula las actualizaciones de posición y velocidad; la cabecera no proporciona funciones para avanzar la simulación física. Esta separación evita pasar agregados o punteros mediante la ABI en los bucles de movimiento más frecuentes.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`
