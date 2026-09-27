# Organisation de la bibliothèque standard FC/NES

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | **Français** | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

**[KITAQFC · Manuel HTML](https://bartaro.github.io/kitaq-docs/fr/fc-library.html)**

Les noms publics suivent volontairement le style KITAQGB : ils sont courts, décrivent leur fonction et ne portent pas de préfixe `kitaqfc_`.

## Base

- `fc.h` : en-tête de regroupement.
- `core.h` : types de base.
- `intrinsics.h` : points d'entrée intrinsèques reconnus par le compilateur.
- `runtime.h` / `runtime.c` : prise en charge de la NMI, de la copie de travail de l'OAM et de la file VRAM.
- `system.h` / `system.c` : fonctions de gestion d'images et d'attente de style KITAQGB.
- `debug.h` / `debug.c` : petit journal de traces et d'assertions en RAM.

## Graphismes

- `vram.h` / `vram.c` : mises à jour VRAM en file de style KITAQGB.
- `sprite.h` / `sprite.c` : allocation de sprites et fonctions de métasprites de style KITAQGB.
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
- `oam_fair.h` / `oam_fair_impl.h` : rotation de 64 candidats OAM conservant les priorités ; voir [oam_fair.md](oam_fair.md).

## Organisation du jeu

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## Audio et périphériques

- `audio.h` / `audio.c`
- `audio_vblank.h` / `audio_vblank.c` — Pilote musical NMI avec une file BGM de sept enregistrements, pause et reprise, un tampon SFX distinct de sept enregistrements et des enveloppes de bruit à déclenchement unique. [Manuel HTML](https://bartaro.github.io/kitaq-docs/fr/fc-library.html#module-audio_vblank)
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` : adaptation des états de boutons à l'interface KITAQGB.
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## Mappers et FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## Mathématiques

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` — Intégration de corps rectangulaires, gravité, contacts AABB, réponse des surfaces et types Q5.3 facultatifs. Compiler avec `fixed.c`.
- `physics3d.h` / `physics3d.c` — Boîtes 3D sans rotation, rebond pondéré par les masses et valeurs d’intensité des chocs. Compiler avec `fixed.c` et `physics2d.c`.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

Les transitions de scène, les mises à jour et le dessin appellent leurs gestionnaires enregistrés de façon synchrone. Consultez chaque API pour l’ordre des rappels et les restrictions de réentrance.
