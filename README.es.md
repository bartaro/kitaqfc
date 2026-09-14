# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Español**

**[Manual del compilador](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html)** · **[Manual de las bibliotecas](https://bartaro.github.io/kitaq-docs/es/fc-library.html)**

Compilador de C y bibliotecas de apoyo para crear software casero original para NES/Famicom/FDS, derivados de KITAQGB y NORCAL.

El proyecto está en fase de versión preliminar pública. Las API y su comportamiento pueden cambiar.

## Criterios de diseño

KITAQFC propone desarrollar teniendo en cuenta el hardware de NES/Famicom y comprobar los avances en pasos pequeños y reproducibles. Compila la ROM, ejecútala en KUROSAKI, analiza los resultados con SARAKURA y repite las pruebas tras cada corrección. Documenta por separado los resultados medidos y el comportamiento que todavía no se ha probado.

<!-- development-prompt:es:start -->
## Prompt para desarrollar un juego

Completa los requisitos y entrega el prompt íntegro a tu asistente de IA. Incluye implementación, pruebas en el emulador, análisis con SARAKURA y verificación de las correcciones.

[Leer el ejemplo práctico en el manual HTML](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html#loop-prompts)

<details>
<summary>Mostrar el prompt completo</summary>

### Desarrollo de juegos con KITAQFC, KUROSAKI y SARAKURA

Completa los requisitos y entrega este documento íntegro al asistente de IA. Los comandos suponen que los repositorios `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` y `kitaq-docs`, junto con el proyecto `game-gb` o `game-fc`, comparten un directorio padre. Ejecútalos desde ese directorio y adapta las rutas al entorno real.

#### Requisitos

- Título del juego: &lt;completar&gt;
- Género y mecánica principal: &lt;completar&gt;
- Controles y condiciones de éxito y fracaso: &lt;completar&gt;
- Pantallas, niveles, enemigos y objetos obligatorios: &lt;completar&gt;
- Estilo visual, música y efectos de sonido: &lt;completar e indicar los recursos proporcionados&gt;
- Guardado, comunicación, periféricos y otros requisitos: &lt;completar o ninguno&gt;
- Directorio del proyecto: &lt;completar&gt;
- Condiciones de redistribución: &lt;por ejemplo, código y recursos originales aptos para publicarse con licencia MIT&gt;

- Destino: &lt;cartucho NES/Famicom o FDS&gt;
- Mapper, tamaño de ROM y mirroring: &lt;especificar o elegir según los requisitos&gt;
- Norma de vídeo y rendimiento: &lt;por ejemplo, NTSC y 60 actualizaciones de la lógica por segundo&gt;

#### Trabajo solicitado

Implementa el juego con KITAQFC y sus bibliotecas. Utiliza KUROSAKI para ejecutar y depurar, y SARAKURA para organizar los diagnósticos y comparar los resultados antes y después de una corrección.

Repite este ciclo hasta cumplir los criterios de aceptación: concretar la especificación → implementar un cambio pequeño → compilar → aplicar entradas y observar → investigar la causa → corregir → repetir las pruebas en las mismas condiciones. Un plan, un listado de código o una compilación correcta no bastan para dar el trabajo por terminado.

##### Comprobar el entorno y los criterios de aceptación

1. Lee las instrucciones del directorio de trabajo, los README, los manuales HTML y las cabeceras e implementaciones de las bibliotecas que vayas a usar. Registra las rutas de los ejecutables y sus versiones o hashes SHA-256. Comprueba los comandos con la salida real de `--help` y las API con el código fuente.
2. Define criterios verificables para entradas, imagen, sonido, progreso y frecuencia de actualización. Por ejemplo: pulsar y soltar START inicia la partida; una colisión resta una vida; la pausa silencia el audio indicado y al continuar se reanuda la reproducción.
3. Pregunta solo por ambigüedades importantes. Resuelve de forma autónoma las decisiones habituales y reversibles. No rebajes los requisitos ni los criterios de aceptación.
4. Primero ejecuta un pequeño ejemplo incluido con el compilador, el emulador y SARAKURA. Esto comprueba la conexión entre herramientas, no la finalización del juego solicitado.

##### Implementar una primera versión jugable

- Considera NROM para un juego pequeño y elige MMC3 u otro mapper cuando el tamaño o los cambios de banco lo requieran. Comprueba las funciones necesarias de la placa con `inspect-rom`, `mapper-info`, `audit-board` y su implementación; el nombre del mapper no demuestra que estén soportadas.
- Planifica tamaños PRG/CHR, CHR-ROM o CHR-RAM, mirroring, bancos fijos, vectores de interrupción y RAM de guardado. Tras optimizar o cambiar bancos, compara la cabecera con la distribución real. `--nes-local-ram` utiliza la RAM interna de CPU `$0000–$07FF`; evita solapamientos con página cero, pila, búferes OAM y áreas del runtime o las bibliotecas.
- Utiliza el dialecto C de KITAQFC, las bibliotecas FC y `void main(void)`. No supongas compatibilidad con las API de GB. Algunas cabeceras solo contienen declaraciones: localiza las implementaciones e incluye los `.c` necesarios.
- Ten en cuenta registros PPU, NMI, OAM DMA, límite de sprites por línea, desplazamiento, mirroring, tablas de atributos y APU/DMC. El espacio libre de la cola no es la capacidad de VRAM del PPU; limita el trabajo por NMI.
- Convierte la fuente original `ascii.c` a CHR de FC y verifica CHR, paletas, tablas de nombres y atributos. Para FDS, comprueba por separado acceso al disco, guardado y requisitos de BIOS; no presupongas el arranque de un cartucho.

- Conecta primero arranque, título, personaje controlable, éxito o fracaso y reinicio. Amplía el contenido después.
- Conserva los originales editables de gráficos, música y efectos, así como los pasos de generación. Comprueba que la compilación consume realmente los datos exportados.
- Escribe los comentarios del código en inglés y los informes de progreso en español. Mantén los informes estándar de SARAKURA en inglés.

##### Vincular cada compilación con su ejecución

Separa las salidas por iteración, por ejemplo en `out/iter-001`. Registra comandos, códigos de salida y hashes de código, recursos, herramientas, ROM y metadatos. Nunca ejecutes una ROM anterior después de una compilación fallida. Los mapas, mapas de código fuente y datos de depuración deben corresponder a la misma compilación que la ROM.

Este ejemplo comprueba NROM sin entrada. Prepara `main.c`, las implementaciones necesarias y el archivo CHR; elige el mapper adecuado. No pases los metadatos de compilación a `--kitaqfc-debug` de KUROSAKI sin comprobar primero el formato requerido.

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


Una ejecución de 300 fotogramas sin entrada es solo una comprobación inicial. Añade escenarios de juego con secuencias de acciones antes de afirmar que funciona.

##### Reproducir secuencias de entrada

- `--pad1` y `--pad2` utilizan máscaras de bits NES sin convertir: A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. No las confundas con los valores `BTN_*` de la biblioteca.
- `run --pad1` aplica una entrada fija. Para acciones sucesivas, crea una reproducción que distinga pulsación, mantenimiento y liberación. Consulta `Replay` y `ReplayFrame`. En la CLI pública, `replay-record` registra entrada neutra, no una partida jugada por una persona.
- Verifica tú mismo el SHA-256 de la ROM, el destino de la reproducción y el intervalo de fotogramas. `replay-run --verify` solo compara el hash final esperado cuando existe; no valida exhaustivamente el juego ni la identidad de la ROM. No sustituyas sin más los resultados esperados por los observados para que una prueba pase.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Prepara la reproducción para la ROM que vas a probar. La CLI pública no admite `--emit-diagnostics` en `replay-run`. No inventes esa opción ni uses trazas CPU como eventos de diagnóstico. Para analizar secuencias de entrada con SARAKURA, crea un programa de pruebas dentro del proyecto con las API públicas de `kurosaki-core`: `RunOptions.replay_frames` y `diagnostic_events_from_trace_and_report`. Ejecuta la misma ROM, reproducción y condiciones de fotogramas; genera JSONL a partir de la traza y el informe diagnóstico de esa ejecución. Revisa la configuración y el intervalo conservado de la traza, y compara imágenes y observaciones del programa de pruebas con la reproducción de la CLI. No presentes diagnósticos sin entrada como evidencia de una partida. Si falta el entorno necesario, declara esta verificación incompleta.

##### Comprobar imagen, sonido, estado y rendimiento

- Guarda escenarios que distingan pulsar, mantener y soltar. Recorre todas las rutas especificadas: arranque, inicio, movimiento, acciones, colisiones, desplazamiento, cambios de nivel, fin de partida, reinicio, pausa y, cuando corresponda, guardado o comunicación.
- Conserva PNG de fotogramas relevantes, entradas, informes de ejecución, JSONL de diagnóstico, WAV y las observaciones necesarias de estado o memoria. Comprueba los fotogramas alcanzados y el motivo de parada. Abre las imágenes: una sola captura no demuestra movimiento ni respuesta a los controles. Compara contadores, posiciones y cambios de estado con lo esperado; revisa bordes de pantalla, límites de tiles y atributos, y escenas con muchos sprites.
- Comprueba música, efectos, reproducción simultánea, cortes, pausa y reanudación. Crear un WAV no demuestra que el sonido sea correcto. Si no puedes escucharlo, distingue las comprobaciones numéricas o de forma de onda de las cualidades audibles aún sin verificar.
- Mide escenas exigentes, trabajo de la CPU de destino, actualizaciones y transferencias; en FC, incluye el trabajo de NMI. La velocidad del emulador en el equipo anfitrión no es la frecuencia del juego ni prueba la velocidad en hardware real. Continuar con `--allow-unimplemented` no demuestra soporte para la función ausente.

##### Analizar, corregir y volver a probar

- Proporciona a SARAKURA los metadatos de la ROM probada y el JSONL de diagnóstico de esa ejecución. Una traza CPU o un informe ordinario no los sustituyen. `--frames` establece condiciones de análisis; SARAKURA no ejecuta la ROM ni modifica automáticamente el código.
- Lee `report.html`, `ai_diagnostics.json`, `repair_prompt.md` y `retest_plan.json`. Contrasta los diagnósticos con reproducción, imágenes, audio y código. Distingue ubicaciones o causas inferidas de hechos comprobados, y bucles de espera normales de bloqueos. Evalúa las advertencias una por una y registra eventos no soportados o límites del análisis. No ocultes advertencias con filtros ni acortes las pruebas para conseguir un resultado favorable.
- Reduce cada fallo a un caso mínimo, corrige su causa y recompila. Si procede del compilador o emulador, aísla el defecto del código del juego y añade comprobaciones de regresión para la corrección de la herramienta.
- Repite las pruebas con las mismas entradas, semilla aleatoria, máquina y norma de vídeo, mapper, fotogramas observados y ajustes de diagnóstico. Cada ROM requiere sus metadatos; no reutilices estados guardados a ciegas tras cambiar código o distribución de RAM.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Usa las diferencias de diagnóstico junto con la aceptación de controles, gráficos y audio. Si el mismo fallo se repite, revisa las pruebas y la hipótesis en lugar de encadenar cambios arbitrarios.

##### Criterios de finalización y entregables

Repite todos los escenarios obligatorios con la ROM final compilada a partir del código y los ajustes entregados. La invencibilidad, entradas automáticas de prueba u otro mapper, por sí solos, no verifican una partida normal en la versión final. Entrega una tabla de requisitos y pruebas, explica las advertencias restantes e identifica lo no comprobado o no soportado. Si no se ha probado en hardware físico, indícalo explícitamente.

Entrega código fuente, identificación de herramientas y bibliotecas, recursos editables, scripts reproducibles de compilación y pruebas, ROM, evidencia final y un README de instalación, controles y limitaciones conocidas. Incluye reproducciones y el programa de pruebas cuando sean necesarios. Publica o envía archivos al exterior solo dentro del alcance autorizado expresamente. Elimina compilaciones intermedias y trazas temporales innecesarias tras verificarlas, conservando fuentes, recursos, entregables y evidencia de regresión necesaria.

Si el entorno o los permisos impiden una comprobación obligatoria, comunica los pasos exactos de reproducción y la acción necesaria. No des el trabajo por terminado.

</details>
<!-- development-prompt:es:end -->

## Organización del repositorio

El subdirectorio homónimo `kitaqfc/` reúne el código fuente del compilador, el archivo de proyecto y la configuración de compilación. El ejecutable Release ya compilado y su configuración de ejecución se encuentran en la raíz. `lib/` contiene las bibliotecas de C y `examples/` los programas introductorios y la tipografía original.

```text
kitaqfc/                  # Repository root
├─ kitaqfc/               # Compiler build sources
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Prebuilt Release compiler
├─ kitaqfc.exe.config     # .NET Framework runtime configuration
├─ lib/                # C support libraries
├─ examples/           # Tutorial programs and original font
├─ scripts/build.ps1   # Rebuild the Release executable
├─ LICENSE
└─ LICENSE.ja
```

El compilador incluido requiere Windows y .NET Framework 4.8. Descargue el ZIP del repositorio para mantener juntos el ejecutable, su configuración, las bibliotecas y los avisos de licencia. Para recompilar también necesita .NET Framework 4.8 Developer Pack y Visual Studio Build Tools. Ejecute lo siguiente desde la raíz del repositorio:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

La compilación Release copia el programa y su configuración a la raíz. La versión Debug permanece en `kitaqfc/bin/Debug` y no sobrescribe el compilador Release distribuido. No se incluyen cachés de compilación ni archivos PDB. El [registro de compilación del binario](BINARY_BUILD.json) detalla las entradas y sus valores SHA-256.

## Compilar y empezar a usar KITAQFC

Tras instalar .NET Framework 4.8 Developer Pack y Visual Studio Build Tools en Windows, puede utilizar MSBuild directamente. Abra Developer PowerShell y ejecute:

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuales y licencias

- [Compilador: manual en español](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html) / [Bibliotecas: manual en español](https://bartaro.github.io/kitaq-docs/es/fc-library.html)
- [Manual en inglés](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manual en japonés](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Archivos del manual para consultarlo sin conexión](https://github.com/bartaro/kitaq-docs)
- [Licencia](LICENSE) / [Traducción japonesa de referencia](LICENSE.ja)

La licencia del proyecto no sustituye las condiciones de terceros sobre tipografías, dependencias, logotipos o marcas. Conserve los avisos adjuntos al redistribuir el software.
