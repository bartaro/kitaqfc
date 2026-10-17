# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA edita tiles, tablas de nombres, paletas y metasprites NES de 8x8 para KITAQFC. La CLI independiente comparte el núcleo del editor y no requiere interfaz gráfica, GPU, plita ni Python.

## Descarga de la interfaz

Descargue el ZIP para Windows x64, extraiga todo y ejecute FCHUA.exe. Conserve assets/fonts y las carpetas de licencias junto al ejecutable. No necesita Rust ni Python para ejecutarlo, pero sí un dispositivo/controlador gráfico compatible con wgpu. La interfaz está en inglés; los manuales completos están en los nueve idiomas indicados.

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

Empiece con File > New project u Open project / raw. Seleccione un tile y una paleta, dibuje en Tile y coloque tiles en Map. La paleta del mapa cambia por bloques de 16x16 píxeles. Seleccione las tablas de fondo/sprites en sus menús. Guarde en .fch para conservar los datos editables; la vista previa PNG no es un proyecto. Export KITAQFC pack genera recursos del juego. Si la fuente cambia, compruebe la extracción completa y PLITA_UI_FONT. Si no se abre la ventana, actualice el controlador gráfico o use la CLI sin interfaz.

El código público incluye los crates necesarios de plita. Desde tools/fchua, compile y ejecute la interfaz con estos comandos; la ruta final del proyecto es opcional. El ejecutable GUI de esta versión se verificó en Windows x64.

La compilación de publicación, los diseños por software y las pruebas automáticas de entrada/estado pasaron en Windows x64. Los clics en ventanas y los diálogos de archivos siguen sin verificar: la herramienta de escritorio agotó el tiempo esperando autorización de la aplicación. Consulte la [verificación](VERIFICATION.md).

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

## Compilar y ejecutar

Se necesita Rust 1.92 o posterior para compilar. En Windows ejecute target/release/fchua.exe; en Linux y macOS, ./target/release/fchua. Consulte --help. Los ejemplos usan el nombre fchua. Cree primero los directorios padre de salida.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## Flujo de trabajo

Cree un proyecto vacío o importe un PNG; revise el informe y la vista previa antes de exportar. --auto-palette elige cuatro paletas NES de fondo minimizando el error; sin esa opción se usan las actuales. --project base.fch aporta paletas y selección de tablas. La transparencia se compone sobre el color universal actual. La importación sustituye el mapa y la tabla de fondo seleccionada, por lo que también cambia los sprites que comparten esa tabla.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## Referencia de comandos

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

## Edición por lotes

El script es una matriz JSON. Todas las operaciones se validan antes de actualizar el documento; un lote fallido no escribe nada. --dry-run solo valida e informa. Las operaciones y campos desconocidos son errores. Los tiles usan índices absolutos 0..511; mapa y sprites usan 0..255 dentro de su tabla seleccionada.

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

## Interfaz y Python

La interfaz plita usa Roboto, menús agrupados con iconos Lucide, estados al pasar/pulsar/soltar y descripciones emergentes. File ofrece crear, abrir, guardar, guardar como, importar PNG y exportar. Ctrl+O/S/Shift+S abre/guarda/guarda como; Ctrl+Z/Y deshace/rehace; Ctrl+C/V copia/pega un tile. Cambiar selección o herramienta no marca el proyecto como modificado. Cerrar, abrir otro proyecto o crear uno pide confirmación si hay cambios.

Map/Tile/Sprite/Check cambian la vista central. B/E/F/L/R/I seleccionan lápiz/borrador/relleno/línea/rectángulo/cuentagotas; 1..4 eligen color. Cada arrastre es un paso de deshacer y Escape lo cancela. Use Fit, zoom, rueda y barras de desplazamiento arrastrables o sus pistas. En ventanas estrechas, Tiles/Palette alterna el panel. Seleccione una muestra y después un color NES; el índice cero modifica el fondo universal. Sprite coloca tiles en una cuadrícula de 8 píxeles; el cuentagotas selecciona y el borrador elimina. Su menú añade/clona fotogramas y refleja/elimina el elemento seleccionado. Check solo comprueba fotogramas individuales, no la escena completa ni un emulador.

El paquete CLI incluye enlaces Python opcionales (Python 3.10+), sin GUI, GPU ni plita. Compile con pip desde tools/fchua o instale el wheel Windows x64 incluido. Controller admite proyectos, PNG, lotes transaccionales y exportación RAW/C; GUI_AVAILABLE=False. La versión con funciones gui,python añade run_editor y ButtonInteraction. Controller edita su propio documento, no una GUI abierta; intercambie archivos .fch. El guardado Python sustituye archivos existentes, a diferencia de la protección --force de la CLI.

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

## Hardware y archivos

.fch es JSON versionado (format=fchua, version=1): CHR de 8192 bytes, paleta de 32, mapa 32x30, 64 atributos, selección de dos tablas y hasta 256 fotogramas. Puede migrar .kfcproject; los atributos de hardware prevalecen ante mapas de paleta incoherentes. El PNG debe medir múltiplos de 8 entre 8x8 y 256x240. Cada bloque 16x16 usa una paleta de cuatro colores. La tabla de fondo tiene 256 tiles; la importación reserva el cero vacío y rechaza excesos.

Exporta .chr (8192), .pal (32), .nam (960), .atr (64), JSON de metasprites, matrices C/H de todos los fotogramas, manifiesto y .fch. C usa __prg_rom de KITAQFC. Configure las tablas PPU en el juego según el proyecto; el paquete no genera esa lógica. CHR crudo admite tiles completos de 16 bytes hasta 8192; los otros tres formatos requieren longitudes exactas. Colores NES: 0..63. Máximo 64 elementos por fotograma, coordenadas -128..127 salvo x=-1, reservado al terminador 0xFF. Solo sprites 8x8; más de ocho elementos en una línea genera advertencia.

## Protección y errores

Para sustituir una salida existente use --force, pero entrada y salida deben ser distintas. El guardado reemplaza atómicamente mediante un archivo temporal. export --out es un directorio; --force sustituye los recursos con esos nombres sin borrar archivos ajenos. --prefix debe ser identificador C. El éxito produce un objeto JSON; los errores, JSON en stderr. Códigos: 0 éxito, 1 advertencias con validate --strict, 2 error de argumentos, proyecto, conversión o E/S. Haga copias antes de sustituir. No se llama a servicios de red ni de generación de IA.

Código original: MIT. Las dependencias conservan sus licencias; consulte LICENSE y THIRD_PARTY_NOTICES.md.
