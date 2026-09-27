# Compression de ressources compatible ZX0

<!-- readme-language-links:start -->
[English](README.md#english) | [日本語](README.md#%E6%97%A5%E6%9C%AC%E8%AA%9E) | [한국어](README.ko.md) | [繁體中文](README.zh-TW.md) | **Français** | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

[API et exemples](https://bartaro.github.io/kitaq-docs/fr/fc-library.html#module-zx0)

Le compresseur PC et le décompresseur FC sont des implémentations indépendantes de KITAQ pour les flux ZX0 v2 en lecture avant. L’implémentation KITAQ est distribuée sous licence MIT, copyright (c) 2026 DAISUKE OBA.

Le format ZX0 et l’algorithme de compression d’origine ont été conçus par [Einar Saukas](https://github.com/einar-saukas/ZX0). Cette reconnaissance du format est distincte du droit d’auteur et de la licence de l’implémentation KITAQ. Voir [LICENSE](../../LICENSE) et [LICENSE.ja](../../LICENSE.ja).

Compilez l’outil PC depuis la racine du dépôt :

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

L’outil PC utilise .NET Framework 4.x. Le compresseur effectue une recherche bornée par chaîne de hachage ; il ne garantit pas la plus petite sortie possible. Les flux inverses, les dictionnaires de préfixes externes et ZX0 v1 ne font pas partie de cette interface. Les droits sur les ressources originales restent à leurs auteurs.

L’entrée et la sortie encodée doivent chacune tenir dans 65535 octets. `--format=auto` compare les charges utiles raw, RLE et ZX0, puis enveloppe la plus petite dans un en-tête KQA1 de neuf octets ; cet en-tête compte dans la limite de sortie. Utilisez `asset_decompress` pour KQA1. Découpez également les ressources selon les fenêtres de banques CPU et la capacité réelle de la RAM. Un en-tête C raw vide réserve un octet, mais sa taille logique `_SIZE` vaut 0.

Incluez `zx0.h` et compilez `lib/zx0.c` pour la cible. `zx0_decompress` reçoit la destination, sa capacité, la source compressée et sa taille. Vérifiez à la fois le nombre d’octets renvoyé et `zx0_error`. Une erreur peut laisser une sortie partielle : ne l’affichez pas et ne l’utilisez pas après un échec. Les tampons source et destination ne doivent ni se chevaucher ni traverser les fenêtres de banques CPU actuellement mappées. Ces fonctions partagent une zone de travail et ne doivent pas être rappelées depuis une interruption pendant leur exécution.

`zx0_decompress_vram` exige une zone RAM assez grande pour toute la ressource décompressée. Lorsque le rendu est désactivé, il décompresse dans cette zone puis transfère le résultat vers la CHR RAM ou la mémoire des tables de noms. Il conserve PPUCTRL ; réglez le défilement avant de réactiver le rendu. Il ne transfère pas de palettes et n’écrit pas dans la CHR ROM.
