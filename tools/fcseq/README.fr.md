# FCSEQ 0.5.1

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ est un éditeur musical pour les puces sonores de la Famicom. Son interface, construite avec plita, permet de placer des notes, de façonner les timbres et d'écouter le résultat. L'interface graphique, la CLI et les bindings Python partagent les mêmes projets et le même moteur audio. Un nouveau projet est vide et indépendant de tout jeu.

## Installation et langue

Téléchargez le ZIP adapté à votre ordinateur dans [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads), puis extrayez-le entièrement. Les versions Windows et Linux ciblent x86_64 ; macOS dispose de versions Apple Silicon et Intel. Lancez `fcseq.exe` sous Windows ou `./fcseq` sous Linux et macOS. Si nécessaire, utilisez `chmod +x fcseq fcseq-cli`. Conservez les polices et les notices avec le programme.

L'interface existe en anglais et en japonais. Le menu **SOUND → 日本語 / English** change la langue ; le lecteur NSF propose le même bouton. Le choix est mémorisé. Au premier lancement, FCSEQ utilise la langue du système lorsqu'elle est disponible. La variable `FCSEQ_LANGUAGE=ja` ou `en` permet de la fixer. La police japonaise Noto Sans JP est fournie.

Windows nécessite Direct3D 12 et le runtime Microsoft Visual C++ x64. macOS utilise Metal. Les binaires Linux sont construits sur Ubuntu 24.04 et utilisent ALSA, X11/Wayland et Vulkan/OpenGL ; une distribution plus ancienne peut nécessiter une compilation locale. L'écoute en direct nécessite un périphérique audio, mais pas l'export WAV. Les binaires ne sont pas signés et les versions Mac ne sont pas notariées.

## Composer et ouvrir l'éditeur de notes

Choisissez une puce dans **SOUND**, puis double-cliquez dans une piste pour créer une note. Faites glisser son corps pour changer sa position et sa hauteur, ou son bord droit pour modifier sa durée. Une piste correspond à un canal monophonique : deux notes ne peuvent pas s'y chevaucher.

Double-cliquez sur le nom de la piste, ou appuyez sur **E**, pour ouvrir l'éditeur avec un clavier de piano à gauche. Chaque ligne représente un demi-ton et l'axe horizontal représente le temps. Cliquez sur une touche pour écouter sa note. La molette déplace la plage de hauteurs, les boutons −/+ du clavier règlent son étendue et Maj+molette déplace la vue dans le temps. Les flèches de piste ouvrent la piste voisine. E revient à l'ensemble des pistes. Les pistes de bruit, DPCM et percussion affichent leurs sons ou vitesses disponibles.

Ctrl/Maj+clic et le rectangle de sélection permettent de sélectionner plusieurs notes. Ctrl+A sélectionne la piste courante ; Ctrl+C/X/V copie, coupe et colle au curseur. Ctrl+D duplique, Suppr efface et Ctrl+Z/Y annule ou rétablit. Les flèches déplacent ou transposent les notes ; Maj+Haut/Bas transpose d'une octave. G change la grille, Q y aligne les notes, F ajuste la vue, Début revient au début et +/− règle le zoom horizontal.

Espace démarre ou arrête l'écoute. LOOP répète la section et AUDITION joue la sélection. Muet et solo s'appliquent aussi aux exports. SONG permet d'ajouter, de renommer et de réordonner les sections : jusqu'à 16 sections de 256 mesures chacune. Cliquez sur le tempo ou le nombre de mesures pour les modifier. Ctrl+N/O/S/Maj+S crée, ouvre, enregistre ou enregistre sous un autre nom. En cas d'interruption, ouvrez `out/recovery_<process-id>.fcq`, à côté du programme, puis utilisez Enregistrer sous.

## Puces, timbres et courbes

Les cinq pistes de base sont les deux impulsions, le triangle, le bruit et le DPCM de la 2A03. Une seule extension peut être active : VRC6 ajoute trois voix, VRC7 six voix FM, FDS une table d'onde, N163 jusqu'à huit tables d'onde, MMC5 deux impulsions et un PCM, SUNSOFT 5B trois voix SSG, EPSM trois SSG, six FM et six percussions, et VT02 deux impulsions, un triangle, un bruit et un PCM. FCSEQ refuse un changement qui supprimerait des notes ; videz d'abord les pistes concernées.

Sélectionnez des notes et ouvrez INSTRUMENT / WAVEFORM. Les impulsions ont un rapport cyclique réglable, le bruit une période et un mode court/long, et la dent de scie VRC6 un réglage d'accumulateur. Ces voix n'ont pas de mémoire de forme d'onde librement dessinable.

FDS possède une onde partagée de 64 points sur 6 bits. N163 propose des emplacements d'ondes sur 4 bits, sélectionnés par note. Dessinez à la souris ou partez d'un sinus, d'une impulsion, d'une dent de scie ou d'un triangle ; vous pouvez ensuite inverser ou décaler l'onde. CURVE / LINE alterne courbe lisse et segments droits. Les points visibles restent les valeurs réellement enregistrées : le lissage ne modifie que l'affichage. Le survol indique l'indice et la valeur du point.

Les longueurs N163 sont des multiples de quatre. La mémoire partagée contient `256 − 16 × canaux_actifs` échantillons, soit 128 avec huit canaux. Raccourcissez une autre onde ou réduisez le nombre de canaux pour libérer de la place. L'édition de la table de modulation et de l'enveloppe matérielle FDS n'est pas incluse.

VRC7 permet d'éditer le timbre personnalisé 0 ; ses préréglages 1–15 sont des approximations synthétiques. EPSM offre 16 timbres FM à quatre opérateurs, avec algorithme, rétroaction, enveloppes et modulation. Une modification affecte toutes les notes utilisant ce timbre. Les voix FM et rythmiques EPSM ont un panoramique : les registres exportés le conservent, mais l'écoute et les WAV actuels sont mono. Les percussions EPSM sont synthétisées spécialement pour FCSEQ ; aucune ROM Yamaha n'est fournie, et le matériel EPSM joue ses propres sons internes.

Les trois voix SSG de 5B ou d'EPSM partagent la période du bruit et le générateur d'enveloppe. Une nouvelle valeur agit donc aussi sur les autres voix qui l'utilisent.

## Utiliser un fichier audio

Dans SAMPLES, importez un WAV, FLAC, Ogg Vorbis ou MP3, ou déposez-le dans la fenêtre. Réglez le début, la fin, le gain, la normalisation et l'inversion temporelle. L'original est conservé ; les modifications sont réversibles. Les sources stéréo sont ramenées en mono et peuvent durer jusqu'à 60 secondes.

Sur la piste DPCM, choisissez l'échantillon et sa vitesse. AUDITION DPCM écoute le résultat encodé et EXPORT .DMC l'enregistre. La conversion retire la composante continue et utilise un filtre passe-bas avant rééchantillonnage. Chaque échantillon est limité à 4081 octets ; la banque partagée à 16320 octets, ou 8192 avec FDS. Réduisez la région ou la fréquence de conversion si nécessaire. La fréquence de conversion et la vitesse de lecture de la note sont deux réglages distincts.

MMC5 et VT02 PCM utilisent les mêmes sources et traitements, sous forme d'un flux 8 bits à 4 kHz. La hauteur change la vitesse de lecture. La limite DPCM ne s'applique pas au PCM. VT02 partage son unité d'échantillons entre DPCM et PCM : choisissez un seul de ces modes par section. Il n'y a ni boucle d'échantillon ni correction automatique de hauteur.

## Fichiers et exports

Enregistrez les projets en `.fcq`. Un ancien fichier `.fcseq` peut être renommé avant ouverture. Les projets de notes utilisent `fcseq-2` et acceptent aussi `fcseq-1` ; les projets de lecture intégrant un NSF utilisent `fcseq-3`.

Ouvrez NSF, NSF2 ou NSFe par le menu FILE, par glisser-déposer ou comme argument de lancement. Choisissez le morceau et sa durée, puis PLAY. La durée indiquée dans les métadonnées est utilisée ; sinon elle vaut 30 secondes, réglables de 1 à 600. L'enregistrement FCQ conserve les données originales et les réglages. L'import ne transforme pas un NSF arbitraire en notes éditables. Un réexport dans le même conteneur préserve les octets d'origine.

Choisissez `.nsf`, `.nsf2` ou `.nsfe` lors de l'export. Chaque section devient un morceau. NSF et NSF2 produisent aussi un NSFe avec les durées et sans fondu. La lecture s'arrête à la fin de la section, indépendamment du bouton LOOP de l'éditeur.

Le lecteur utilise le rythme NTSC pour les fichiers NTSC ou multirégions, gère les banques de 4 Kio et les fonctions NSF2 d'IRQ temporisée, d'INIT en arrière-plan et de suppression de PLAY. Les fichiers exclusivement PAL sont refusés. EPSM et le temporisateur NSF2 partagent des ports : leur combinaison avec le drapeau d'IRQ NSF2 est donc refusée. Les limites sont détaillées dans [AUDIO_FORMATS](docs/AUDIO_FORMATS.md).

Export C + DPCM écrit les flux de registres, le pilote C, l'en-tête, la banque d'échantillons et le rapport de placement. Ce n'est pas une ROM complète : le programme hôte doit configurer le mapper et garder les données accessibles au CPU. Appelez `fcseq_tick` à `FCSEQ_TICK_RATE_HZ`, soit normalement la fréquence NTSC, ou 4000 Hz avec PCM. `FCSEQ_KITAQFC` sélectionne le compilateur KITAQFC ; `FCSEQ_WRITE` permet un accès personnalisé aux registres.

## CLI, Python et compilation

`fcseq-cli help` décrit les commandes. Sous Windows, ajoutez `.exe`. Les indices commencent à 0, sauf les numéros de morceaux NSF, qui commencent à 1.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli import-nsf music.nsf2 imported.fcq 1 60
```

Python 3.10 ou ultérieur peut charger l'extension fournie pour le même système et processeur. L'interface et la CLI n'ont pas besoin de Python. Consultez [fcseq_native.pyi](fcseq_native.pyi) pour les méthodes, notamment `set_sound` et `set_epsm_parameter`.

Pour compiler, utilisez Rust 1.93.1 et un compilateur C++14 : `cargo build --locked --release`, puis `cargo test --locked --tests`. La CLI seule utilise `--no-default-features --bin fcseq-cli` ; l'extension Python, `--features python --lib`. Windows nécessite MSVC Build Tools, macOS les Xcode Command Line Tools. Les paquets Linux sont indiqués dans `.github/workflows/fcseq.yml`.

Le code de FCSEQ est sous licence MIT, Copyright (c) 2026 DAISUKE OBA. Les dépendances gardent leurs licences, dont BSD-3-Clause pour ymfm, MPL-2.0 pour Symphonia et OFL pour les polices. Conservez LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ et third_party_sources/ lors d'une redistribution. Aucune ROM de jeu, aucun BIOS FDS ni aucune ROM commerciale de timbres n'est inclus. Les musiques et échantillons importés conservent leurs droits. Consultez [les notices](THIRD_PARTY_NOTICES.md), [l'audit](LICENSE_AUDIT.json) et [les vérifications](VERIFICATION.md).

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
