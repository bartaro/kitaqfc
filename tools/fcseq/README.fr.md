# FCSEQ 0.4.1 — manuel

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

Auteur : **DAISUKE OBA**. Séquenceur musical Famicom généraliste utilisant **plita**. L’interface graphique, la CLI et les bindings Python partagent le modèle, le rendu audio et les exports. Le projet initial est vide ; aucune ROM de jeu ni aucun BIOS n’est nécessaire. L’interface est en anglais ; le manuel existe en neuf langues.

## Installation

Téléchargez l’archive complète dans [downloads/](downloads/) : Windows x86_64, Linux x86_64, macOS Apple Silicon ARM64 ou Intel x86_64. Extrayez tous les fichiers, y compris les ressources et licences. Lancez `fcseq.exe` sous Windows ou `./fcseq` sous Unix ; la CLI est `fcseq-cli` (`.exe` sous Windows). [bin/](bin/) contient aussi les exécutables seuls. Windows 10/11 utilise Direct3D 12, macOS Metal, Linux Vulkan/OpenGL, X11/Wayland et ALSA. Linux est compilé sous Ubuntu 24.04 ; une distribution plus ancienne peut nécessiter une compilation locale. Sur Ubuntu : `sudo apt install libasound2t64 libxkbcommon0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. Il faut un pilote graphique compatible et, pour l’écoute, un périphérique audio ; l’export WAV fonctionne sans périphérique. Les exécutables ne sont pas signés ni notariés sur macOS. En cas de blocage, vérifiez les sources et empreintes puis utilisez l’autorisation normale du système. Au besoin : `chmod +x fcseq fcseq-cli`.

## Composition

SOUND sélectionne une configuration : 2A03 (Pulse 1/2, Triangle, Noise, DPCM : 5 voix), VRC6 (+2 pulse, +1 saw : 8), VRC7 (+6 FM : 11), FDS (+1 onde : 6), N163 (+1 à 8 ondes : jusqu’à 13). Une seule extension est active ; les changements supprimant des notes sont refusés.

Double-cliquez pour ajouter une note, glissez son corps pour déplacer/transposer, son bord droit pour redimensionner. Ctrl/Shift-clic ou rectangle sélectionne plusieurs notes ; Ctrl+A sélectionne la piste. Ctrl+C/X/V copie/coupe/colle au curseur ; Ctrl+D duplique ; Delete supprime ; Ctrl+Z/Y annule/rétablit. Flèches : déplacement/transposition ; Shift+haut/bas : octave. G : grille, Q : quantification, F : vue complète, Home : début, +/- : zoom, Shift+molette : temps, UP/DOWN : pistes. Space démarre/arrête, LOOP répète, AUDITION écoute la sélection. MUTE/SOLO affectent aussi les exports. SONG gère 16 sections de 256 mesures maximum ; cliquez sur nom, BPM ou mesures pour modifier. Ctrl+N/O/S/Shift+S : nouveau/ouvrir/enregistrer/enregistrer sous. Les éditions invalides sont annulées entièrement. Récupération : ouvrir `out/recovery_<process-id>.fcq` à côté du programme, puis enregistrer sous un autre nom.

## Timbres

Ouvrez SOUND > Instrument / waveform editor, sélectionnez les notes du canal puis modifiez et utilisez AUDITION/UNDO/REDO. Pulse : 4 rapports cycliques ; VRC6 pulse : 8 ; Triangle : onde fixe de 32 pas ; Noise : 16 périodes et mode long/court. Volume pulse/bruit : 0–15 ; décroissance 0 : volume constant. VRC6 saw : incrément d’accumulateur 0–63, avec débordement modifiant le timbre. Ces sources n’ont pas de RAM d’onde libre.

FDS : une onde partagée de 64 échantillons, 6 bits. N163 : ondes 4 bits, initialement 4 × 32, sélectionnées par note ; longueur multiple de 4. Capacité partagée : `256 - 16 × canaux actifs` échantillons (128 à huit voix). Raccourcissez une autre onde ou réduisez les canaux avant de dépasser cette limite. Dessin par glissement, SINE/PULSE/SAW/TRIANGLE, inversion et rotation. Les limites de hauteur/RAM/canaux sont contrôlées. VRC7 : patch personnalisé 0, 24 paramètres (multiplicateurs, ADSR, retour, niveau, suivi de clavier, trémolo/vibrato, sinus/demi-sinus) ou 8 octets. Les presets 1–15 sont des approximations synthétiques, sans copie de ROM. Pas d’éditeur dédié à la modulation/enveloppe matérielle FDS. Le son émulé ne garantit pas l’équivalence analogique.

## DPCM

SAMPLES ou SOUND > Import audio file importe WAV/FLAC/Ogg Vorbis/MP3, également par dépôt. Réglez START/END en ms, GAIN, NORMALIZE, REVERSE, RATE 0–15 ; écoutez avec AUDITION DPCM, sauvegardez avec EXPORT .DMC. Placez une note DPCM puis choisissez SAMPLE et sa vitesse RATE. Le PCM original reste conservé ; entrée maximale 60 s, stéréo convertie en mono, suppression du continu et rééchantillonnage filtré. Un échantillon : `16n+1` octets, maximum 4081 ; au plus 64 échantillons, banque totale 16320 octets (8192 en FDS). Les dépassements sont refusés. La fréquence de conversion et celle de lecture sont distinctes. Pas de boucle interne ni de correction automatique de hauteur.

## Fichiers et exports

Enregistrez en `.fcq` ; les anciens `.fcseq` peuvent être renommés. Les notes utilisent `fcseq-2` JSON, `fcseq-1` reste lisible ; les documents de lecture NSF utilisent `fcseq-3`. Import NSF, OPEN, dépôt ou argument de lancement ouvre NSF/NSFe. PREV/NEXT ou numéro choisit le morceau ; PLAY/STOP, LOOP et durée 1–600 s contrôlent la lecture. Les métadonnées de durée sont respectées, sinon 30 s par défaut. La génération audio préalable est annulable. Le FCQ conserve le fichier original, le morceau et la durée ; ce n’est pas une transcription en notes. Le réexport conserve les octets originaux.

Export NSF transforme chaque section éditée en sous-morceau, avec instruments et DPCM ; un NSFe compagnon contient les durées et un fondu nul. Préférez NSFe si le lecteur ignore les durées NSF. Arrêt à la fin de section ; LOOP de l’éditeur est indépendant. Entrées prises en charge : NSF v1/NSFe, NTSC ou double région compatible NTSC, sans banques ou banques 4 Kio, 2A03/DPCM, VRC6/VRC7/FDS/N163. PAL seul, extensions d’exécution NSF2, MMC5, Sunsoft 5B, VT02 sont refusés ; INIT/PLAY sans retour est limité.

Export C + DPCM produit `score.c`, `fcseq_score.h`, `fcseq_player.c`, binaires de sections, `.dmc`, `dpcm_bank.bin`, `fcseq_export.json`. Il faut intégrer le mapper et les adresses CPU du code/données/DPCM ; aucune ROM complète n’est générée. VRC6 suppose mapper 24 (26 exige une traduction d’adresses), VRC7 85, N163 19, ou FDS. Dans KITAQFC, définissez `FCSEQ_KITAQFC` avant l’inclusion ; `FCSEQ_WRITE(address, value)` permet une écriture personnalisée.

## CLI et Python

Exécutez les exemples dans le dossier extrait. Indices de section/canal/note/échantillon : base 0 ; morceaux NSF : base 1. Python 3.10+ utilise l’extension abi3 fournie pour le même OS et processeur. GUI/CLI n’exigent pas Python. API complète : `fcseq_native.pyi`. La CLI déplace, transpose, redimensionne, quantifie, duplique et supprime les notes ; les ondes se modifient via GUI/Python.

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

## Compilation et vérification

Dans ce dossier source, utilisez Rust 1.93.1 et les commandes ci-dessous. Sources locales : vendor/. Linux : paquets de développement dans `.github/workflows/fcseq.yml` à la racine du dépôt ; macOS : Xcode Command Line Tools ; Windows : MSVC Build Tools. Python est facultatif. Les scripts vérifient CLI/Python natifs, WAV, allers-retours NSF/NSFe et démarrage GUI. Résultats et limites : VERIFICATION.md ; ni interaction humaine souris/clavier ni écoute sur matériel réel ne sont certifiées.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python
```

## Licence

FCSEQ est sous MIT, Copyright (c) 2026 DAISUKE OBA. Les dépendances gardent leurs licences. Redistribuez LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ et third_party_sources/. Sources Symphonia MPL-2.0 et notices des polices/icônes incluses. Les musiques et échantillons importés ont leurs propres droits.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)
