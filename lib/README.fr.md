# Organisation de la bibliothèque standard FC/NES

[English](README.md) | **Français**

**Ouvrir le manuel de la bibliothèque KITAQFC**

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
- `physics2d.h` / `physics2d.c` : types sur un octet et constantes Q5.3 pour les déplacements 2D, avec position entière, fraction de 1/8 de pixel, vitesse non signée, direction explicite et réglages de freinage. Le code du jeu effectue l'intégration ; l'en-tête ne fournit pas de fonction de simulation. Conserver ces valeurs séparément permet d'effectuer les calculs dans les boucles critiques sans passer des structures ou des pointeurs par l'ABI.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

Les fonctions de compatibilité KITAQGB conservent des noms courts lorsque le modèle matériel FC/NES le permet. Les fonctions de scène et d'entité fondées sur des rappels conservent actuellement l'état, mais n'effectuent pas d'appels indirects des pointeurs de fonction de l'utilisateur.
