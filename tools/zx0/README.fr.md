# Outils auxiliaires natifs

[en](../README.en.md) · [ja](../README.ja.md) · [ko](../README.ko.md) · [zh-CN](../README.zh-CN.md) · [zh-TW](../README.zh-TW.md) · [fr](../README.fr.md) · [es](../README.es.md) · [de](../README.de.md)

Tous les outils sont implémentés en Rust et fonctionnent sans .NET, Python ni Pillow. Compilez tous les exécutables avec `cargo build --locked --release`. Sous Windows, ajoutez `.exe` aux noms des commandes.

```text
kitaqfc-zx0 input.bin output.zx0
kitaqfc-zx0 output.zx0 restored.bin --decompress
kitaqfc-zx0 input.bin asset.h --header=level_data
kitaqfc-zx0 input.bin output.kqa --format=auto
kitaqfc-asset-pack assets/example_manifest.json
kitaqfc-png-index-build assets/example_png_manifest.json
kitaqfc-asset-pipeline assets/example_pipeline_manifest.json
kitaqfc-rights-name-guard --root . --denylist deny_terms.local.txt
```

ZX0 accepte 1 à 65535 octets et conserve la sortie du codeur C#. Les formats `raw`, `rle` nombre/valeur et le conteneur automatique `KQA1` de neuf octets sont pris en charge. À égalité : raw, RLE, puis ZX0. `--decompress` décode un flux ZX0 v2 direct avec une taille bornée. Les flux inversés et v1 ne sont pas pris en charge. Einar Saukas a conçu le format ; cette implémentation KITAQ est sous licence MIT.

Les manifestes JSON existants produisent une image CHR de 8 KiB, des tableaux C/en-têtes et un rapport JSON. Les métasprites acceptent data, frames ou un fichier JSON. Le convertisseur PNG indexé produit CHR, nametable, attributs, palette et JSON de métasprites. Fonds : 256×240 ; tuiles : 8×8 ; groupes de palette cohérents par tuile et quadrant de fond de 16×16. Indices 0–15, profondeurs 1/2/4/8, Adam7. Le script PNG absent a été reconstruit à partir de la spécification locale et des manifestes exemples ; aucune comparaison avec cet original indisponible n’est possible. Les RGB utilisent une palette NES approximative définie. Le pipeline exécute les deux étapes en interne. Le contrôleur de noms lit une liste externe et indique seulement fichiers/lignes, sans termes protégés, en excluant les dossiers générés.

```sh
sh tools/zx0/build.sh
```
