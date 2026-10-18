# FCSEQ 0.5.0

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ es un editor musical para los chips de sonido de Famicom. Su interfaz, creada con plita, permite escribir notas, ajustar timbres y escuchar el resultado mientras trabajas. La aplicación gráfica, la CLI y los bindings de Python comparten el formato de proyecto y el motor de audio. Cada proyecto comienza vacío, sin música de ningún juego.

## Instalación e idioma

Descarga el ZIP de tu sistema y procesador desde [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) y extrae todo su contenido. Hay versiones x86_64 para Windows y Linux, y versiones Apple Silicon e Intel para macOS. Abre `fcseq.exe` en Windows o `./fcseq` en Linux y macOS. Si faltan permisos, ejecuta `chmod +x fcseq fcseq-cli`. Mantén las fuentes y los archivos de licencia junto al programa.

La interfaz admite inglés y japonés. Cambia entre ambos en **SOUND → 日本語 / English**; el reproductor NSF tiene el mismo botón. La elección se guarda para la próxima sesión. Al iniciarse por primera vez, FCSEQ consulta el idioma del sistema cuando está disponible. También puedes usar `FCSEQ_LANGUAGE=ja` o `en`. Se incluye Noto Sans JP para mostrar japonés.

Windows necesita Direct3D 12 y el runtime Microsoft Visual C++ x64. macOS utiliza Metal. Los binarios de Linux se compilan en Ubuntu 24.04 y usan ALSA, X11/Wayland y Vulkan/OpenGL; las distribuciones antiguas pueden necesitar una compilación local. La reproducción requiere un dispositivo de audio, pero la exportación WAV no. Los binarios no están firmados y las aplicaciones de Mac no están notarizadas.

## Notas y editor de teclado

Elige un chip en SOUND y haz doble clic en una zona vacía de una pista para crear una nota. Arrastra el cuerpo para cambiar su posición y altura, o el borde derecho para ajustar su duración. Cada pista representa un canal monofónico: no se permiten notas superpuestas.

Haz doble clic en el nombre de una pista o pulsa E para abrir su editor con un piano a la izquierda. Las filas representan semitonos y el eje horizontal, el tiempo. Pulsa una tecla del piano para escucharla. La rueda desplaza el registro visible, −/+ junto al teclado cambia su amplitud y Mayús+rueda desplaza la vista en el tiempo. Las flechas de pista abren la anterior o la siguiente. Pulsa E para volver a todas las pistas. Ruido, DPCM y percusión muestran sus sonidos o velocidades disponibles.

Usa Ctrl/Mayús+clic o arrastra un rectángulo para seleccionar varias notas. Ctrl+A selecciona la pista actual; Ctrl+C/X/V copia, corta y pega en el cursor. Ctrl+D duplica, Supr elimina y Ctrl+Z/Y deshace o rehace. Las flechas mueven o transportan las notas; Mayús+Arriba/Abajo cambia una octava. G cambia la cuadrícula, Q ajusta las notas a ella, F encaja la sección, Inicio vuelve al principio y +/− controla el zoom horizontal.

Espacio inicia o detiene la reproducción. LOOP repite la sección y AUDITION reproduce la selección. Silencio y solo también afectan a los archivos exportados. SONG permite añadir, renombrar y ordenar hasta 16 secciones de 256 compases cada una. Pulsa el tempo o los compases para editarlos. Ctrl+N/O/S/Mayús+S crea, abre, guarda o guarda con otro nombre. Para recuperar trabajo, abre `out/recovery_<process-id>.fcq`, junto al programa, y usa Guardar como.

## Chips y edición de timbres

La base 2A03 tiene dos pulsos, triángulo, ruido y DPCM. Puedes añadir una extensión: VRC6 aporta tres voces; VRC7, seis FM; FDS, una tabla de ondas; N163, hasta ocho; MMC5, dos pulsos y PCM; SUNSOFT 5B, tres SSG; EPSM, tres SSG, seis FM y seis percusiones; VT02, dos pulsos, triángulo, ruido y PCM. No se permite cambiar de chip si se perderían notas: vacía antes las pistas afectadas.

Selecciona notas y abre INSTRUMENT / WAVEFORM. Los pulsos permiten ajustar el ciclo de trabajo; el ruido, el período y el modo corto/largo; la sierra VRC6, la velocidad del acumulador. Estos canales no tienen una memoria de onda que se pueda dibujar libremente.

FDS ofrece una onda compartida de 64 puntos y 6 bits. N163 utiliza ondas de 4 bits y cada nota elige su ranura. Dibuja arrastrando o parte de los ajustes seno, pulso, sierra y triángulo; también puedes invertir o desplazar la onda. CURVE / LINE alterna curvas suaves y segmentos rectos. Los puntos son los valores guardados en el chip: el suavizado solo cambia la representación. Al pasar el ratón se muestra el índice y el valor.

Las longitudes N163 deben ser múltiplos de cuatro. La memoria compartida admite `256 − 16 × canales_activos` muestras: 128 con ocho canales. Acorta otras ondas o reduce los canales para liberar espacio. No se incluye la edición de la tabla de modulación ni de la envolvente de hardware FDS.

VRC7 permite editar el timbre personalizado 0; los ajustes 1–15 son aproximaciones sintéticas. EPSM tiene 16 timbres FM de cuatro operadores, con algoritmo, realimentación, envolventes y modulación. Los cambios afectan a todas las notas que usan ese timbre. FM y percusión EPSM permiten panoramización, conservada en los registros exportados; la escucha y los WAV actuales son mono. Las percusiones se sintetizan con datos originales: no se incluye una ROM Yamaha, y el dispositivo EPSM reproduce sus propios sonidos internos.

Las tres voces SSG de 5B o EPSM comparten el período del ruido y el generador de envolvente. El último ajuste también afecta a otras notas que usen ese generador.

## Convertir audio en un instrumento

Abre SAMPLES e importa WAV, FLAC, Ogg Vorbis o MP3; también puedes soltar el archivo en la ventana. Ajusta inicio, final, ganancia, normalización e inversión temporal. Se conserva el audio original para deshacer los cambios. El estéreo se mezcla a mono y el archivo de origen puede durar hasta 60 segundos.

En una nota DPCM elige la muestra y la velocidad. AUDITION DPCM reproduce el resultado codificado y EXPORT .DMC lo guarda. La conversión elimina la componente continua y remuestrea con un filtro paso bajo. Cada muestra admite 4081 bytes y el banco compartido 16320, o 8192 con FDS. Si excede el límite, recorta o reduce la frecuencia de conversión. Esta frecuencia es independiente de la velocidad de reproducción de cada nota.

MMC5 y VT02 PCM usan las mismas fuentes y controles para generar PCM de 8 bits a 4 kHz. La altura modifica la velocidad de reproducción y no se aplica el límite de tamaño DPCM. VT02 comparte la unidad de muestras entre DPCM y PCM: utiliza solo uno de esos modos por sección. No hay bucles de muestras ni corrección automática de altura.

## Proyectos y formatos de salida

Guarda los proyectos editables como `.fcq`. Puedes renombrar los antiguos `.fcseq` antes de abrirlos. Los proyectos de notas utilizan `fcseq-2` y también se admite `fcseq-1`; los proyectos que guardan un NSF para reproducirlo usan `fcseq-3`.

Abre NSF, NSF2 o NSFe desde FILE, arrastrando el archivo o mediante un argumento de inicio. Elige la canción y la duración y pulsa PLAY. Se usan los tiempos de los metadatos; si faltan, se parte de 30 segundos, ajustables entre 1 y 600. Guardar en FCQ incorpora el archivo original y sus ajustes. Importar no convierte cualquier NSF en notas editables. Reexportar al mismo contenedor conserva los bytes originales.

Elige `.nsf`, `.nsf2` o `.nsfe` al exportar. Cada sección será una canción. NSF y NSF2 crean además un NSFe con duraciones y sin fundido. La reproducción termina al finalizar la sección; LOOP solo controla la repetición en el editor.

Se admite NTSC y archivos multirregión con temporización NTSC, bancos de 4 KiB y NSF2 con IRQ de temporizador, INIT en segundo plano y supresión de PLAY. Se rechazan archivos exclusivamente PAL. EPSM comparte puertos con el temporizador NSF2, por lo que se rechaza su combinación con la bandera de IRQ NSF2. Consulta [AUDIO_FORMATS](docs/AUDIO_FORMATS.md) para conocer los límites.

Export C + DPCM genera flujos de registros, controlador C, cabecera, banco y un informe de ubicación. No es una ROM completa: el programa anfitrión debe configurar el mapper y mantener los datos visibles para la CPU. Llama a `fcseq_tick` a `FCSEQ_TICK_RATE_HZ`: normalmente la frecuencia NTSC, o 4000 Hz con PCM. Define `FCSEQ_KITAQFC` para el compilador KITAQFC o `FCSEQ_WRITE` para tu propia escritura de registros.

## CLI, Python y compilación

Ejecuta `fcseq-cli help` para ver las órdenes; añade `.exe` en Windows. Los índices empiezan en 0, salvo las canciones NSF, que empiezan en 1.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli import-nsf music.nsf2 imported.fcq 1 60
```

La extensión incluida funciona con Python 3.10 o posterior en el mismo sistema y arquitectura. La GUI y la CLI no necesitan Python. [fcseq_native.pyi](fcseq_native.pyi) describe la API, incluidos `set_sound` y `set_epsm_parameter`.

Para compilar necesitas Rust 1.93.1 y C++14. Usa `cargo build --locked --release` y prueba con `cargo test --locked --tests`. La CLI sola usa `--no-default-features --bin fcseq-cli`; Python, `--features python --lib`. En Windows instala MSVC Build Tools y en macOS Xcode Command Line Tools. Los paquetes de Linux están en `.github/workflows/fcseq.yml`.

FCSEQ se distribuye bajo MIT, Copyright (c) 2026 DAISUKE OBA. Las dependencias conservan sus licencias: ymfm usa BSD-3-Clause, Symphonia MPL-2.0 y las fuentes OFL. Al redistribuir, conserva LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ y third_party_sources/. No se incluyen ROM de juegos, BIOS FDS ni ROM comerciales de timbres. La música y las muestras importadas conservan sus derechos. Consulta [avisos](THIRD_PARTY_NOTICES.md), [auditoría](LICENSE_AUDIT.json) y [verificación](VERIFICATION.md).
