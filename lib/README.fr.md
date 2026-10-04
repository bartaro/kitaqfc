# Organisation de la bibliothèque standard FC/NES



<!-- fc-current-20261004:fr:start -->
### Distribution actuelle du compilateur et des bibliothèques — 4 octobre 2026

fc.h inclut les déclarations de physics3d et audio_vblank. Ajoutez séparément les fichiers d'implémentation nécessaires. entity_update_all et entity_draw_all appellent le rappel pour les emplacements actifs par ID croissant. Placez ce rappel dans la banque commune 0 ou gardez sa banque PRG sélectionnée pendant les appels. Incluez explicitement zx0.h pour ZX0. L'ordre des enregistrements audio_vblank est delay, CH1, CH2, CH3, CH4.

La compilation C# actuelle a été comparée à la distribution précédente sur 29 entrées. Quinze entrées acceptées ont produit les mêmes octets ROM ; les quatorze autres entrées ont produit les mêmes diagnostics de rejet. Deux ROM supplémentaires ont tourné pendant quatre images dans KUROSAKI : allocation d'entités, rappels et réutilisation des emplacements ; calcul signé, disposition de KQBody3D et constantes de la file audio. Le rapport donne 18 octets RAM attendus et observés. Ces essais ne couvrent ni la lecture audio ni toutes les API ; les anciennes régressions sont distinguées dans BINARY_BUILD.json.

[Exemples source et preuves d'exécution](https://bartaro.github.io/kitaq-docs/fr/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:fr:end -->


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

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [fc-library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [fc-library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [fc-library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Français | [fc-library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Español | [fc-library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Deutsch | [fc-library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->

<!-- rust-native-20261004:start -->
## Utiliser la bibliothèque avec le compilateur natif

lib/ contient les en-têtes et sources C compilés dans la ROM de la console. Le compilateur pour PC et les outils auxiliaires sont en Rust ; les sources de jeux, bibliothèques cibles et API restent en C. SOURCE_MANIFEST.json et LIBRARY_MERGE.json consignent les sources retenues lorsqu'ils sont fournis.

Les constructions natives et vérifications d'exécution ont réussi sous Windows, Linux, macOS ARM et macOS Intel. KITAQGB a passé 48 tests et 395 vérifications d'outils par environnement ; KITAQFC en a passé 55 et 401. Rust 1.85 a également été testé. PUBLIC_DISTRIBUTION.json consigne les empreintes des binaires installés et la provenance des validations. Les workflows GitHub Actions publics reconstruisent et testent ces sources indépendamment.

Les sorties de référence conservées vérifient les octets ROM, les diagnostics et les formats des outils. Les anciennes preuves d'émulation C# restent des résultats historiques liés à leurs empreintes sources. Elles ne prouvent pas automatiquement toutes les API Rust, le matériel réel ou le démarrage complet d'un jeu par le BIOS FDS. Le script PNG d'origine, indisponible, a été réimplémenté d'après sa spécification ; l'identité des octets avec ce script ne peut être affirmée.

[Compilateur natif Rust et outils auxiliaires](../tools/README.fr.md)

<!-- rust-native-20261004:end -->
