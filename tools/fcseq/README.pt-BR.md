# FCSEQ 0.5.0

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ é um editor musical para os chips de som do Famicom. A interface, feita com plita, permite escrever notas, ajustar timbres e ouvir o resultado enquanto você trabalha. A aplicação gráfica, a CLI e os bindings Python usam o mesmo formato de projeto e o mesmo mecanismo de áudio. Cada projeto começa vazio, sem músicas de jogos.

## Instalação e idioma

Baixe em [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) o ZIP correspondente ao seu sistema e processador e extraia todo o conteúdo. Windows e Linux têm versões x86_64; macOS tem versões Apple Silicon e Intel. Abra `fcseq.exe` no Windows ou `./fcseq` no Linux e no macOS. Se necessário, execute `chmod +x fcseq fcseq-cli`. Mantenha as fontes e os arquivos de licença junto do programa.

A interface está disponível em inglês e japonês. Use **SOUND → 日本語 / English** para mudar; o reprodutor NSF oferece o mesmo botão. A escolha é salva para a próxima sessão. Na primeira execução, o idioma do sistema é usado quando disponível. Você também pode definir `FCSEQ_LANGUAGE=ja` ou `en`. A fonte Noto Sans JP acompanha o programa.

O Windows exige Direct3D 12 e o runtime Microsoft Visual C++ x64. O macOS usa Metal. Os binários Linux são compilados no Ubuntu 24.04 e usam ALSA, X11/Wayland e Vulkan/OpenGL; distribuições antigas podem exigir compilação local. A reprodução precisa de um dispositivo de áudio, mas a exportação WAV não. Os binários não têm assinatura digital e os aplicativos Mac não são notarizados.

## Notas e editor com teclado

Escolha um chip em SOUND e dê dois cliques em uma área vazia de uma pista para criar uma nota. Arraste o corpo para mudar posição e altura, ou a borda direita para mudar a duração. Cada pista corresponde a um canal monofônico; notas sobrepostas não são aceitas.

Dê dois cliques no nome da pista ou pressione E para abrir seu editor com um piano à esquerda. As linhas representam semitons, e o eixo horizontal representa o tempo. Clique em uma tecla para ouvir a nota. A roda do mouse desloca o registro visível; −/+ junto ao teclado altera sua extensão; Shift+roda desloca a visualização no tempo. As setas de pista abrem a anterior ou a seguinte. E retorna à visão geral. Ruído, DPCM e percussão mostram os sons ou velocidades disponíveis.

Use Ctrl/Shift+clique ou arraste um retângulo para selecionar várias notas. Ctrl+A seleciona a pista atual; Ctrl+C/X/V copia, recorta e cola no cursor. Ctrl+D duplica, Delete exclui e Ctrl+Z/Y desfaz ou refaz. As setas movem ou transpõem notas; Shift+Cima/Baixo transpõe uma oitava. G altera a grade, Q alinha as notas, F enquadra a seção, Home volta ao início e +/− ajusta o zoom horizontal.

Espaço inicia ou para a reprodução. LOOP repete a seção, e AUDITION toca a seleção. Silenciar e solo também afetam as exportações. SONG permite criar, renomear e reorganizar até 16 seções de 256 compassos cada. Clique no andamento ou na quantidade de compassos para editar. Ctrl+N/O/S/Shift+S cria, abre, salva ou salva com outro nome. Para recuperar trabalho, abra `out/recovery_<process-id>.fcq`, ao lado do programa, e use Salvar como.

## Chips, timbres e formas de onda

A base 2A03 oferece dois pulsos, triângulo, ruído e DPCM. Uma expansão pode ser adicionada: VRC6 traz três vozes; VRC7, seis FM; FDS, uma tabela de ondas; N163, até oito; MMC5, dois pulsos e PCM; SUNSOFT 5B, três SSG; EPSM, três SSG, seis FM e seis percussões; VT02, dois pulsos, triângulo, ruído e PCM. O programa impede mudanças de chip que apagariam notas. Esvazie primeiro as pistas afetadas.

Selecione notas e abra INSTRUMENT / WAVEFORM. Pulsos têm ajuste de ciclo de trabalho; ruído tem período e modo curto/longo; a serra VRC6 tem controle do acumulador. Esses canais não oferecem memória de onda que possa ser desenhada livremente.

FDS usa uma onda compartilhada de 64 pontos e 6 bits. N163 usa ondas de 4 bits, escolhidas por nota. Desenhe arrastando o mouse ou comece com seno, pulso, serra ou triângulo. Também é possível inverter ou deslocar a onda. CURVE / LINE alterna curvas suaves e segmentos retos. Os pontos visíveis continuam sendo os valores gravados no chip; a suavização altera apenas a exibição. Ao passar o mouse sobre um ponto, aparecem seu índice e valor.

O comprimento das ondas N163 deve ser múltiplo de quatro. Todas as vozes compartilham `256 − 16 × canais_ativos` amostras, ou 128 com oito canais. Encurte outras ondas ou reduza os canais se faltar espaço. A tabela de modulação e o envelope de hardware FDS ainda não são editáveis.

No VRC7, o timbre personalizado 0 é editável; os presets 1–15 são aproximações sintéticas. EPSM oferece 16 timbres FM com quatro operadores, algoritmo, realimentação, envelopes e modulação. A alteração vale para todas as notas que usam o timbre. FM e percussão EPSM têm panorama, preservado nos registros exportados; a prévia e os WAV atuais são mono. As percussões são sintetizadas com dados originais, sem ROM Yamaha incluída. O hardware EPSM toca seus próprios sons internos.

As três vozes SSG de 5B ou EPSM compartilham o período do ruído e o gerador de envelope. Um novo valor também afeta outras notas que usam o mesmo gerador.

## Usar áudio como instrumento

Em SAMPLES, importe WAV, FLAC, Ogg Vorbis ou MP3, ou arraste o arquivo para a janela. Ajuste início, fim, ganho, normalização e reprodução reversa. O original é mantido, portanto as alterações são reversíveis. Fontes estéreo são mixadas em mono e podem ter até 60 segundos.

Na pista DPCM, escolha a amostra e a velocidade de reprodução. AUDITION DPCM toca o resultado codificado; EXPORT .DMC o salva. A conversão remove a componente contínua e aplica um filtro passa-baixas na reamostragem. Cada amostra aceita 4081 bytes, e o banco compartilhado aceita 16320 bytes, ou 8192 com FDS. Recorte a região ou reduza a taxa de conversão se ultrapassar o limite. A taxa de conversão e a velocidade da nota são ajustes separados.

MMC5 e VT02 PCM usam as mesmas fontes e controles para transmitir áudio de 8 bits a 4 kHz. A altura muda a velocidade de reprodução, e o limite de tamanho DPCM não se aplica. VT02 compartilha a unidade de amostras entre DPCM e PCM: use apenas um desses modos por seção. Não há loop de amostra nem correção automática de altura.

## Salvar e exportar

Salve projetos editáveis como `.fcq`. Arquivos antigos `.fcseq` podem ser renomeados antes de abrir. Projetos de notas usam `fcseq-2` e também aceitam `fcseq-1`; projetos que incorporam um NSF para reprodução usam `fcseq-3`.

Abra NSF, NSF2 ou NSFe pelo menu FILE, arrastando o arquivo ou passando um argumento ao iniciar. Escolha a música e a duração, depois PLAY. A duração dos metadados é usada quando existe; caso contrário, começa em 30 segundos, ajustáveis de 1 a 600. Salvar como FCQ incorpora os dados originais e as configurações. Importar não transforma qualquer NSF em notas editáveis. Reexportar para o mesmo contêiner preserva os bytes originais.

Escolha `.nsf`, `.nsf2` ou `.nsfe` ao exportar. Cada seção vira uma música. NSF e NSF2 também geram um NSFe com durações e sem fade. A reprodução termina no fim da seção, independentemente do botão LOOP do editor.

O reprodutor aceita NTSC e arquivos multirregião com temporização NTSC, bancos de 4 KiB e NSF2 com IRQ de temporizador, INIT em segundo plano e supressão de PLAY. Arquivos exclusivamente PAL são recusados. EPSM compartilha portas com o temporizador NSF2, portanto sua combinação com o sinalizador de IRQ NSF2 é recusada. Veja os limites em [AUDIO_FORMATS](docs/AUDIO_FORMATS.md).

Export C + DPCM gera fluxos de registros, driver C, cabeçalho, banco de amostras e relatório de posicionamento. Não gera uma ROM completa: o programa hospedeiro deve configurar o mapper e manter os dados acessíveis à CPU. Chame `fcseq_tick` na taxa `FCSEQ_TICK_RATE_HZ`: normalmente a frequência NTSC, ou 4000 Hz com PCM. Defina `FCSEQ_KITAQFC` para o compilador KITAQFC ou `FCSEQ_WRITE` para seu próprio acesso a registros.

## CLI, Python e compilação

Execute `fcseq-cli help` para ver os comandos. No Windows, acrescente `.exe`. Os índices começam em 0, exceto os números de músicas NSF, que começam em 1.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli import-nsf music.nsf2 imported.fcq 1 60
```

A extensão incluída funciona com Python 3.10 ou posterior no mesmo sistema e arquitetura. GUI e CLI não precisam de Python. [fcseq_native.pyi](fcseq_native.pyi) descreve a API, incluindo `set_sound` e `set_epsm_parameter`.

Para compilar, use Rust 1.93.1 e um compilador C++14: `cargo build --locked --release`, seguido de `cargo test --locked --tests`. Só a CLI usa `--no-default-features --bin fcseq-cli`; Python usa `--features python --lib`. O Windows precisa de MSVC Build Tools; o macOS, de Xcode Command Line Tools. Os pacotes Linux estão em `.github/workflows/fcseq.yml`.

O código do FCSEQ usa a licença MIT, Copyright (c) 2026 DAISUKE OBA. As dependências mantêm suas licenças, incluindo BSD-3-Clause no ymfm, MPL-2.0 no Symphonia e OFL nas fontes. Ao redistribuir, mantenha LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ e third_party_sources/. Não há ROM de jogo, BIOS FDS ou ROM comercial de timbres incluída. Músicas e amostras importadas mantêm seus próprios direitos. Veja [avisos](THIRD_PARTY_NOTICES.md), [auditoria](LICENSE_AUDIT.json) e [verificação](VERIFICATION.md).
