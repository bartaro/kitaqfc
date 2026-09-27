# Compresión de recursos compatible con ZX0

<!-- readme-language-links:start -->
[English](README.md#english) | [日本語](README.md#%E6%97%A5%E6%9C%AC%E8%AA%9E) | [한국어](README.ko.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | **Español** | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

[API y ejemplos](https://bartaro.github.io/kitaq-docs/es/fc-library.html#module-zx0)

El compresor para PC y el descompresor FC son implementaciones independientes de KITAQ para flujos ZX0 v2 de lectura hacia delante. La implementación de KITAQ se distribuye con licencia MIT, copyright (c) 2026 DAISUKE OBA.

[Einar Saukas](https://github.com/einar-saukas/ZX0) diseñó el formato ZX0 y el algoritmo de compresión original. Este reconocimiento del formato es independiente de los derechos de autor y la licencia de la implementación de KITAQ. Consulta [LICENSE](../../LICENSE) y [LICENSE.ja](../../LICENSE.ja).

Compila la herramienta de PC desde la raíz del repositorio:

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

La herramienta de PC utiliza .NET Framework 4.x. El compresor realiza una búsqueda acotada mediante cadenas hash; no garantiza la salida más pequeña posible. Esta interfaz no admite flujos inversos, diccionarios externos de prefijos ni ZX0 v1. Los derechos de los recursos originales siguen perteneciendo a sus autores.

La entrada y la salida codificada deben ocupar como máximo 65535 bytes cada una. `--format=auto` compara las cargas útiles raw, RLE y ZX0 y envuelve la menor en una cabecera KQA1 de nueve bytes; la cabecera cuenta para el límite de salida. Usa `asset_decompress` con KQA1. Divide también los recursos para respetar las ventanas de banco de la CPU y la capacidad real de RAM. Una cabecera C raw vacía contiene un byte de reserva con `_SIZE` lógico igual a 0.

Incluye `zx0.h` y compila `lib/zx0.c` para el destino. `zx0_decompress` recibe la dirección de destino, su capacidad, el origen comprimido y su tamaño. Comprueba tanto el número de bytes devuelto como `zx0_error`. Un error puede dejar una salida parcial: no la muestres ni la utilices si la llamada falla. Los búferes de origen y destino no deben solaparse ni atravesar los límites de las ventanas de banco de CPU mapeadas. Estas rutinas comparten una zona de trabajo y no deben volver a invocarse desde una interrupción mientras se ejecutan.

`zx0_decompress_vram` necesita un área de trabajo RAM con espacio para todo el recurso descomprimido. Con el renderizado desactivado, descomprime en esa área y transfiere el resultado a CHR RAM o a la memoria de tablas de nombres. Conserva PPUCTRL; configura el desplazamiento antes de volver a activar el renderizado. No carga paletas ni escribe en CHR ROM.
