# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA édite les tuiles, tables de noms, palettes et métasprites NES 8x8 pour KITAQFC. La CLI autonome partage le moteur de l'éditeur et ne nécessite ni interface graphique, ni GPU, ni installation de plita, ni Python.

## Télécharger l'interface

Téléchargez le ZIP Windows x64, extrayez-le entièrement et lancez FCHUA.exe. Conservez assets/fonts et les dossiers de licences à côté du programme. Rust et Python ne sont pas nécessaires à son exécution ; un périphérique/pilote graphique compatible avec wgpu est requis. L'interface est en anglais ; les manuels complets existent dans les neuf langues ci-dessus.

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

Commencez par File > New project ou Open project / raw. Sélectionnez une tuile et une palette, dessinez dans Tile puis placez les tuiles dans Map. La palette de la carte change par blocs de 16x16 pixels. Choisissez les tables de fond/sprites dans leurs menus. Enregistrez en .fch pour garder les données modifiables ; l'aperçu PNG n'est pas un projet. Export KITAQFC pack produit les ressources du jeu. Si la police diffère, vérifiez l'extraction complète et PLITA_UI_FONT. Si la fenêtre ne s'ouvre pas, mettez à jour le pilote graphique ou utilisez la CLI sans interface.

Le code public inclut les crates plita nécessaires. Depuis tools/fchua, compilez et lancez l'interface avec ces commandes ; le chemin final du projet est facultatif. L'exécutable GUI de cette version a été vérifié sous Windows x64.

La compilation de distribution, les dispositions rendues par logiciel et les tests automatiques d'entrée/état ont réussi sous Windows x64. Les clics de fenêtre et les dialogues de fichiers restent non vérifiés : l'outil de bureau a expiré en attendant l'autorisation de l'application. Voir la [vérification](VERIFICATION.md).

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

## Compilation et lancement

La compilation nécessite Rust 1.92 ou ultérieur. Sous Windows : target/release/fchua.exe ; sous Linux et macOS : ./target/release/fchua. Consultez --help. Les exemples nomment l'exécutable fchua. Créez d'abord les répertoires parents de sortie.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## Procédure

Créez un projet vide ou importez un PNG, puis examinez rapport et aperçu avant l'export. --auto-palette choisit quatre palettes NES de fond en réduisant l'erreur ; sinon les palettes actuelles sont utilisées. --project base.fch fournit palettes et tables. Les pixels transparents sont composés sur la couleur de fond universelle actuelle. L'import remplace la carte et la table de fond choisie, modifiant aussi les sprites qui partagent cette table.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## Référence des commandes

```text
new --out FILE.fch [--demo] [--force]
inspect FILE.fch
validate FILE.fch [--strict]
import-image IMAGE.png --out FILE.fch [--project BASE.fch] [--auto-palette] [--force]
import-raw --out FILE.fch [--project BASE.fch] [--chr FILE.chr]
           [--palette FILE.pal] [--nametable FILE.nam] [--attributes FILE.atr] [--force]
apply INPUT.fch --script EDITS.json --out OUTPUT.fch [--dry-run] [--force]
preview FILE.fch --out IMAGE.png [--view map|tiles|sprite|tile] [--scale 1..8] [--force]
export FILE.fch --out DIRECTORY [--prefix C_IDENTIFIER] [--force]
migrate OLD.kfcproject --out FILE.fch [--force]
```

## Édition par lot

Le script est un tableau JSON. Toutes les opérations sont validées avant modification ; un lot invalide n'écrit rien. --dry-run vérifie sans sauvegarder. Opérations et champs inconnus sont refusés. Les opérations de tuile utilisent 0..511 ; cartes et sprites utilisent 0..255 dans leur table respective.

```json
[
  {"op":"pixel","tile":1,"x":2,"y":3,"color":1},
  {"op":"fill","tile":2,"color":0},
  {"op":"checker","tile":3,"a":1,"b":2},
  {"op":"line","tile":4,"x0":0,"y0":0,"x1":7,"y1":7,"color":3},
  {"op":"rect","tile":5,"x0":1,"y0":1,"x1":6,"y1":6,"color":2},
  {"op":"mirror","tile":1,"horizontal":true,"vertical":false},
  {"op":"rotate","tile":1},
  {"op":"copy","source":1,"target":6},
  {"op":"palette","index":1,"value":48},
  {"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0},
  {"op":"tables","background":0,"sprites":1},
  {"op":"sprite","frame":0,"index":0,"x":-8,"y":0,"tile":1,"attr":64},
  {"op":"clone_frame","frame":0},
  {"op":"remove_sprite","frame":1,"index":0},
  {"op":"shift_map","dx":2,"dy":0},
  {"op":"dedupe"}
]
```

`tile: 0..511; x/y (pixel): 0..7; color: 0..3; palette index: 0..31; value: 0..63`

`map: 32x30; palette: 0..3; tables: 0..1; frame: 0..255; index: 0..63`

`attr: palette bits 0..1, priority 0x20, flip H 0x40, flip V 0x80; reserved: 0x1c`

`shift_map: dx=-32..32, dy=-30..30; dx % 2 == 0, dy % 2 == 0`

## Interface et Python

L'interface plita utilise Roboto, des menus groupés avec icônes Lucide, des états de survol/pression/relâchement et des infobulles. File propose nouveau, ouvrir, enregistrer, enregistrer sous, import PNG et exports. Ctrl+O/S/Shift+S ouvre/enregistre/enregistre sous ; Ctrl+Z/Y annule/rétablit ; Ctrl+C/V copie/colle une tuile. Changer de sélection ou d'outil ne rend pas le projet modifié. Fermer, créer ou ouvrir un autre projet demande confirmation avant d'abandonner des modifications.

Map/Tile/Sprite/Check changent la vue centrale. B/E/F/L/R/I sélectionnent crayon/gomme/remplissage/ligne/rectangle/pipette ; 1..4 choisissent la couleur. Un glissement correspond à une annulation ; Escape l'annule. Utilisez Fit, le zoom, la molette et les poignées ou pistes des barres. En fenêtre étroite, Tiles/Palette change le panneau latéral. Sélectionnez une case puis une couleur NES ; zéro modifie le fond universel. Sprite place une tuile sur la grille de 8 pixels ; la pipette sélectionne et la gomme supprime. Son menu ajoute/clone des images et retourne/supprime la pièce sélectionnée. Check vérifie chaque image, pas la scène complète ni l'émulation.

Le paquet CLI inclut des bindings Python optionnels (Python 3.10+), sans GUI, GPU ni plita. Compilez avec pip depuis tools/fchua ou installez le wheel Windows x64 fourni. Controller gère projets, PNG, lots transactionnels et export RAW/C ; GUI_AVAILABLE=False. La version avec les fonctions gui,python ajoute run_editor et ButtonInteraction. Controller édite son propre document, pas une interface ouverte ; échangez des fichiers .fch. L'enregistrement Python remplace les fichiers existants, contrairement à la protection --force de la CLI.

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Optional GUI-enabled Python build, also from tools/fchua
python -m pip install maturin
python -m maturin build --release --locked --features gui,python --out gui-wheels
# Install the wheel generated in gui-wheels instead of the headless wheel.
```

```python
import fchua
editor = fchua.Controller()
editor.new_blank()
editor.apply_commands('[{"op":"fill","tile":1,"color":2},{"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0}]')
editor.save_project("scene.fch")
editor.preview_png("preview.png", "map", 2)
editor.export_raw_assets("assets", "scene")
print(fchua.GUI_AVAILABLE, editor.snapshot().unique_tiles)
```

## Matériel et fichiers

.fch est un JSON versionné (format=fchua, version=1) : CHR de 8192 octets, palette de 32, carte 32x30, 64 attributs, deux sélections de tables et jusqu'à 256 images. Les anciens .kfcproject sont migrables ; les attributs matériels priment sur une carte de palettes incohérente. Les dimensions PNG sont des multiples de 8 entre 8x8 et 256x240. Chaque bloc 16x16 utilise une palette de quatre couleurs. La table de fond contient 256 tuiles ; l'import réserve la tuile zéro vide et refuse les dépassements.

L'export produit .chr (8192), .pal (32), .nam (960), .atr (64), JSON de métasprites, tableaux C/H de toutes les images, manifeste et .fch. C utilise __prg_rom. Réglez les tables PPU dans le jeu selon le projet ; ce code n'est pas généré. Le CHR brut doit contenir des tuiles entières de 16 octets, maximum 8192 ; les autres formats exigent leurs tailles exactes. Couleurs NES : 0..63. Maximum 64 pièces par image, coordonnées -128..127 sauf x=-1 réservé au terminateur 0xFF. Seuls les sprites 8x8 sont pris en charge ; plus de huit pièces sur une ligne entraîne un avertissement.

## Protection et erreurs

Une sortie existante exige --force, mais entrée et sortie doivent rester distinctes. Les projets sont remplacés atomiquement via un fichier temporaire. export --out désigne un répertoire ; --force remplace les ressources nommées sans supprimer les autres fichiers. --prefix doit être un identifiant C. Le succès écrit un objet JSON ; les erreurs écrivent du JSON sur stderr. Codes : 0 succès, 1 avertissements avec validate --strict, 2 erreur d'arguments, projet, conversion ou E/S. Sauvegardez avant remplacement. Aucun service réseau ou de génération IA n'est appelé.

Code original : MIT. Les dépendances conservent leurs licences ; voir LICENSE et THIRD_PARTY_NOTICES.md.
