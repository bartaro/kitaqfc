# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Français**

**[Manuel du compilateur](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html)** · **[Manuel de la bibliothèque](https://bartaro.github.io/kitaq-docs/fr/fc-library.html)**

Compilateur C et bibliothèques pour créer des logiciels homebrew originaux NES/Famicom/FDS, issus de KITAQGB et de NORCAL.

Version publique préliminaire : les API et leur comportement peuvent évoluer.

## Principes de développement

KITAQFC privilégie un développement adapté au matériel NES/Famicom, dont les progrès se vérifient par petites étapes reproductibles. Compilez la ROM, exécutez-la dans KUROSAKI, analysez les résultats avec SARAKURA, puis recommencez les tests après chaque correction. Consignez séparément les résultats mesurés et les comportements qui n’ont pas encore été testés.

<!-- development-prompt:fr:start -->
## Prompt pour développer un jeu

Renseignez les besoins, puis transmettez le prompt complet à votre assistant IA. Il couvre l’implémentation, les tests dans l’émulateur, l’analyse avec SARAKURA et la vérification des corrections.

[Lire l’exemple pratique dans le manuel HTML](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html#loop-prompts)

<details>
<summary>Afficher le prompt complet</summary>

### Développer un jeu avec KITAQFC, KUROSAKI et SARAKURA

Renseignez les besoins, puis transmettez ce document entier à l’assistant IA. Les commandes supposent que les dépôts `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` et `kitaq-docs`, ainsi que le projet `game-gb` ou `game-fc`, se trouvent dans le même dossier parent. Exécutez-les depuis ce dossier et adaptez les chemins à votre environnement.

#### Besoins

- Titre du jeu : &lt;à renseigner&gt;
- Genre et mécanique principale : &lt;à renseigner&gt;
- Commandes et conditions de réussite ou d’échec : &lt;à renseigner&gt;
- Écrans, niveaux, ennemis et objets indispensables : &lt;à renseigner&gt;
- Style graphique, musique et effets sonores : &lt;à renseigner, avec les chemins des ressources fournies&gt;
- Sauvegarde, communication, périphériques et autres besoins : &lt;à renseigner, ou aucun&gt;
- Dossier du projet : &lt;à renseigner&gt;
- Conditions de redistribution : &lt;par exemple, code et ressources originaux pouvant être publiés sous licence MIT&gt;

- Cible : &lt;cartouche NES/Famicom ou FDS&gt;
- Mapper, taille de ROM et mirroring : &lt;à préciser ou à choisir selon les besoins&gt;
- Standard vidéo et performances : &lt;par exemple, NTSC et 60 mises à jour de la logique par seconde&gt;

#### Travail demandé

Implémentez le jeu avec KITAQFC et ses bibliothèques. Utilisez KUROSAKI pour l’exécution et le débogage, et SARAKURA pour organiser les diagnostics et comparer les résultats avant et après correction.

Répétez ce cycle jusqu’à satisfaire les critères d’acceptation : préciser la spécification → implémenter une petite modification → compiler → appliquer des entrées et observer → rechercher la cause → corriger → refaire les tests dans les mêmes conditions. Un plan, du code fourni ou une compilation réussie ne suffisent pas à terminer le travail.

##### Vérifier l’environnement et les critères d’acceptation

1. Lisez les consignes du dossier de travail, les README, les manuels HTML ainsi que les en-têtes et implémentations des bibliothèques utilisées. Relevez les chemins des exécutables et leurs versions ou empreintes SHA-256. Vérifiez les commandes dans la sortie réelle de `--help` et les API dans le code source.
2. Définissez des critères mesurables pour les entrées, l’image, le son, la progression et la fréquence de mise à jour. Par exemple : appuyer puis relâcher START lance la partie ; une collision retire une vie ; la pause coupe les sons prévus et la reprise rétablit la lecture.
3. Ne posez de questions que sur les ambiguïtés importantes. Prenez de façon autonome les décisions courantes et réversibles. Ne réduisez pas les exigences ni les critères d’acceptation.
4. Commencez par faire passer un petit exemple fourni dans le compilateur, l’émulateur et SARAKURA. Cela vérifie leur articulation, pas l’achèvement du jeu demandé.

##### Réaliser une première version jouable

- Envisagez NROM pour un petit jeu, puis MMC3 ou un autre mapper si la taille ou les changements de banque le nécessitent. Vérifiez les fonctions de la carte avec `inspect-rom`, `mapper-info`, `audit-board` et leur implémentation ; le nom du mapper ne garantit pas leur prise en charge.
- Prévoyez les tailles PRG/CHR, CHR-ROM ou CHR-RAM, mirroring, banques fixes, vecteurs d’interruption et RAM de sauvegarde. Après optimisation ou modification des banques, comparez l’en-tête à la disposition réelle. `--nes-local-ram` utilise la RAM interne du CPU `$0000–$07FF` ; évitez tout chevauchement avec page zéro, pile, tampons OAM et zones du runtime ou des bibliothèques.
- Utilisez le dialecte C de KITAQFC, les bibliothèques FC et `void main(void)`. Ne présumez pas la compatibilité des API GB. Certains en-têtes ne contiennent que des déclarations : repérez les implémentations et incluez les fichiers `.c` nécessaires.
- Tenez compte des registres PPU, de NMI, d’OAM DMA, du nombre de sprites par ligne, du défilement, du mirroring, des tables d’attributs et d’APU/DMC. L’espace libre de la file n’est pas la capacité de VRAM du PPU ; dimensionnez le travail de chaque NMI.
- Convertissez la police originale `ascii.c` au format CHR FC, puis vérifiez CHR, palettes, tables de noms et attributs. Pour FDS, vérifiez séparément accès disque, sauvegarde et exigences de BIOS, sans supposer les mêmes conditions de démarrage qu’une cartouche.

- Reliez d’abord démarrage, titre, personnage contrôlable, réussite ou échec, et nouvelle partie. Enrichissez ensuite le contenu.
- Conservez les sources modifiables des graphismes, musiques et effets ainsi que les étapes de génération. Vérifiez que la compilation utilise réellement les données exportées.
- Rédigez les commentaires du code en anglais et les comptes rendus d’avancement en français. Gardez les rapports standard de SARAKURA en anglais.

##### Relier chaque compilation à son exécution

Séparez les sorties par itération, par exemple dans `out/iter-001`. Consignez commandes, codes de sortie et empreintes du code, des ressources, outils, ROM et métadonnées. N’exécutez jamais une ancienne ROM après une compilation échouée. Les cartes mémoire, correspondances avec les sources et informations de débogage doivent provenir de la même compilation que la ROM.

Cet exemple vérifie NROM sans entrée. Préparez `main.c`, les implémentations nécessaires et le fichier CHR ; choisissez le mapper approprié. Ne transmettez pas les métadonnées de compilation à `--kitaqfc-debug` de KUROSAKI sans vérifier le format attendu.

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```


Une exécution de 300 images sans entrée n’est qu’une vérification initiale. Ajoutez des scénarios avec des actions successives avant d’affirmer que le jeu fonctionne.

##### Reproduire une séquence d’entrées

- `--pad1` et `--pad2` utilisent des masques de bits NES bruts : A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. Ne les confondez pas avec les valeurs `BTN_*` de la bibliothèque.
- `run --pad1` applique une entrée fixe. Pour enchaîner des actions, préparez un replay distinguant pression, maintien et relâchement. Consultez les définitions `Replay` et `ReplayFrame`. Dans la CLI publique, `replay-record` enregistre des entrées neutres, pas une partie jouée par une personne.
- Contrôlez vous-même le SHA-256 de la ROM, la cible du replay et la plage d’images. `replay-run --verify` ne compare l’empreinte finale attendue que si elle existe ; il ne valide pas entièrement le jeu ni l’identité de la ROM. Ne remplacez pas systématiquement les résultats attendus par les résultats observés pour faire réussir un test.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Préparez le replay pour la ROM testée. La CLI publique ne propose pas `--emit-diagnostics` avec `replay-run`. N’inventez pas cette option et ne transmettez pas de trace CPU à la place d’événements de diagnostic. Pour analyser des entrées successives avec SARAKURA, créez dans le projet un programme de test utilisant les API publiques de `kurosaki-core` : `RunOptions.replay_frames` et `diagnostic_events_from_trace_and_report`. Exécutez la même ROM, le même replay et les mêmes conditions d’images ; produisez le JSONL à partir de la trace et du rapport diagnostic de cette exécution. Vérifiez la configuration et la plage conservée de la trace, puis comparez images et observations du programme de test avec le replay de la CLI. Ne présentez pas des diagnostics sans entrée comme preuve d’un scénario de jeu. Si l’environnement nécessaire manque, signalez cette vérification comme incomplète.

##### Vérifier l’image, le son, l’état et les performances

- Conservez les scénarios d’entrée en distinguant pression, maintien et relâchement. Parcourez toutes les voies prévues : démarrage, début de partie, déplacement, actions, collisions, défilement, changement de niveau, fin de partie, redémarrage, pause et, si nécessaire, sauvegarde ou communication.
- Gardez les PNG des images pertinentes, les entrées, rapports d’exécution, diagnostics JSONL, WAV et observations nécessaires d’état ou de mémoire. Vérifiez le nombre d’images atteint et la raison de l’arrêt. Ouvrez réellement les images : une seule capture ne prouve ni mouvement ni réponse aux commandes. Comparez compteurs, positions et transitions aux valeurs attendues ; examinez bords d’écran, limites de tuiles et d’attributs, et scènes chargées en sprites.
- Vérifiez musique, effets, lecture simultanée, coupures, pause et reprise. Produire un WAV ne prouve pas la justesse du son. Si l’écoute est impossible, distinguez les contrôles numériques ou de forme d’onde réalisés des qualités sonores non vérifiées.
- Mesurez les scènes exigeantes, le travail du CPU cible, les mises à jour et les transferts ; sur FC, incluez NMI. Le débit de l’émulateur sur l’ordinateur hôte n’est ni la fréquence de mise à jour du jeu ni une preuve de vitesse sur matériel réel. Poursuivre avec `--allow-unimplemented` ne prouve pas la prise en charge de la fonction manquante.

##### Analyser, corriger et refaire les tests

- Fournissez à SARAKURA les métadonnées de la ROM testée et le JSONL diagnostic de cette exécution. Une trace CPU ou un rapport d’exécution ordinaire ne les remplace pas. `--frames` précise les conditions d’analyse ; SARAKURA n’exécute pas la ROM et ne modifie pas automatiquement les sources.
- Lisez `report.html`, `ai_diagnostics.json`, `repair_prompt.md` et `retest_plan.json`. Confrontez les diagnostics aux étapes de reproduction, images, sons et sources. Distinguez les emplacements ou causes supposés des faits vérifiés, et les boucles d’attente normales des blocages. Examinez chaque avertissement et notez les événements non pris en charge ou les limites d’analyse. Ne masquez pas les avertissements avec des filtres et ne raccourcissez pas les tests pour obtenir un succès.
- Réduisez les défauts à des cas minimaux, corrigez leur cause et recompilez. Si le compilateur ou l’émulateur est en cause, isolez son défaut du code du jeu et ajoutez une vérification de non-régression à la correction de l’outil.
- Refaites les tests avec les mêmes entrées, graine aléatoire, machine et standard vidéo, mapper, images observées et réglages diagnostics. Utilisez les métadonnées propres à chaque ROM ; ne réutilisez pas aveuglément les états sauvegardés après modification du code ou de la disposition RAM.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Associez les différences de diagnostic aux critères d’acceptation des commandes, graphismes et sons. Si le même échec se répète, réexaminez les preuves et l’hypothèse au lieu d’enchaîner des modifications arbitraires.

##### Conditions de fin et livrables

Rejouez tous les scénarios obligatoires avec la ROM finale compilée depuis les sources et réglages livrés. L’invincibilité, des entrées automatiques de test ou un autre mapper ne suffisent pas à valider une partie normale dans la version finale. Fournissez un tableau reliant besoins et tests, expliquez les avertissements restants et indiquez les éléments non vérifiés ou non pris en charge. Précisez explicitement l’absence de tests sur matériel physique, le cas échéant.

Livrez les sources, l’identification des outils et bibliothèques, les ressources modifiables, les scripts reproductibles de compilation et de test, la ROM, les preuves finales et un README expliquant installation, commandes et limites connues. Incluez les replays et le programme de test si nécessaire. Publiez ou transmettez des fichiers à l’extérieur uniquement dans le périmètre expressément autorisé. Supprimez les compilations intermédiaires et traces temporaires inutiles après vérification, mais conservez sources, ressources, livrables et preuves de non-régression nécessaires.

Si l’environnement ou les permissions empêchent un contrôle obligatoire, indiquez les étapes exactes de reproduction et l’action nécessaire. Ne déclarez pas le travail terminé.

</details>
<!-- development-prompt:fr:end -->

## Organisation du dépôt

```text
kitaqfc/                  # Racine du dépôt
├─ kitaqfc/               # Sources nécessaires à la compilation du compilateur
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Compilateur prêt à l'emploi, en mode Release
├─ kitaqfc.exe.config     # Configuration du runtime .NET Framework
├─ lib/                   # Bibliothèques C
├─ examples/              # Programmes pédagogiques et police originale
├─ scripts/build.ps1      # Reconstruction de l'exécutable Release
├─ LICENSE
└─ LICENSE.ja
```

Le compilateur prêt à l'emploi nécessite Windows avec .NET Framework 4.8. Téléchargez l'archive ZIP du dépôt pour conserver ensemble l'exécutable, sa configuration, les bibliothèques et les mentions de licence. Pour recompiler, il faut également le Developer Pack .NET Framework 4.8 et Visual Studio Build Tools. Depuis la racine du dépôt :

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Une compilation Release copie l'exécutable et sa configuration à la racine du dépôt. Les compilations Debug se trouvent dans `kitaqfc/bin/Debug` et ne remplacent pas le compilateur Release distribué. Les caches de compilation et fichiers PDB ne sont pas distribués. Consultez le [compte rendu de compilation binaire](BINARY_BUILD.json) pour les entrées et les empreintes SHA-256.

## Compiler et commencer à utiliser l'outil

Utilisez Windows, le Developer Pack .NET Framework 4.8 et Visual Studio Build Tools avec MSBuild. Lancez les commandes depuis une invite Developer PowerShell.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuels et licences

- [Compilateur en français](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html) / [Bibliothèque en français](https://bartaro.github.io/kitaq-docs/fr/fc-library.html)
- [Manuel en anglais](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manuel en japonais](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Sources du manuel pour lecture hors connexion](https://github.com/bartaro/kitaq-docs)
- [Licence](LICENSE) / [Traduction japonaise à titre de référence](LICENSE.ja)

La licence du projet ne remplace pas les conditions de tiers relatives aux polices, dépendances, logos ou marques. Conservez les mentions jointes lors de la redistribution.
