# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Português (Brasil)**

**[Manual do compilador](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html)** · **[Manual da biblioteca](https://bartaro.github.io/kitaq-docs/pt/fc-library.html)**

Compilador C e bibliotecas de apoio para programas homebrew originais de NES/Famicom/FDS, derivados de KITAQGB e NORCAL.

Versão de prévia pública: as APIs e o comportamento podem mudar.

## Princípios de desenvolvimento

A KITAQFC favorece um desenvolvimento que respeite as características do hardware do NES/Famicom e permita verificar cada avanço em etapas pequenas e reproduzíveis. Compile a ROM, execute-a no KUROSAKI, analise os resultados com o SARAKURA e repita os testes após cada correção. Registre separadamente os resultados medidos e os comportamentos que ainda não foram testados.

<!-- development-prompt:pt:start -->
## Prompt para desenvolver um jogo

Preencha os requisitos e envie o prompt completo ao assistente de IA. Ele abrange implementação, testes no emulador, análise com SARAKURA e verificação das correções.

[Ler o exemplo prático no manual HTML](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html#loop-prompts)

<details>
<summary>Mostrar o prompt completo</summary>

### Desenvolvimento de jogos com KITAQFC, KUROSAKI e SARAKURA

Preencha os requisitos e envie este documento inteiro ao assistente de IA. Os comandos pressupõem que os repositórios `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` e `kitaq-docs`, além do projeto `game-gb` ou `game-fc`, estejam na mesma pasta pai. Execute os comandos nessa pasta e ajuste os caminhos ao ambiente real.

#### Requisitos

- Nome do jogo: &lt;preencher&gt;
- Gênero e mecânica principal: &lt;preencher&gt;
- Controles e condições de sucesso e fracasso: &lt;preencher&gt;
- Telas, fases, inimigos e itens obrigatórios: &lt;preencher&gt;
- Estilo visual, música e efeitos sonoros: &lt;preencher e indicar os materiais fornecidos&gt;
- Salvamento, comunicação, periféricos e outros requisitos: &lt;preencher ou nenhum&gt;
- Pasta do projeto: &lt;preencher&gt;
- Condições de redistribuição: &lt;por exemplo, código e materiais originais que possam ser publicados sob a licença MIT&gt;

- Plataforma: &lt;cartucho NES/Famicom ou FDS&gt;
- Mapper, tamanho da ROM e espelhamento: &lt;especificar ou escolher conforme os requisitos&gt;
- Padrão de vídeo e desempenho: &lt;por exemplo, NTSC e 60 atualizações da lógica por segundo&gt;

#### Trabalho solicitado

Implemente o jogo com KITAQFC e suas bibliotecas. Use KUROSAKI para execução e depuração, e SARAKURA para organizar os diagnósticos e comparar os resultados antes e depois de uma correção.

Repita este ciclo até atender aos critérios de aceitação: detalhar a especificação → implementar uma pequena mudança → compilar → aplicar entradas e observar → investigar a causa → corrigir → testar novamente nas mesmas condições. Um plano, a apresentação do código ou uma compilação bem-sucedida não significam que o trabalho está concluído.

##### Verificar o ambiente e os critérios de aceitação

1. Leia as instruções da pasta de trabalho, os README, os manuais HTML e os cabeçalhos e implementações das bibliotecas usadas. Registre os caminhos dos executáveis e suas versões ou hashes SHA-256. Confira os comandos na saída real de `--help` e as APIs no código-fonte.
2. Defina critérios verificáveis para entradas, imagem, som, progressão e frequência de atualização. Por exemplo: pressionar e soltar START inicia a partida; uma colisão tira uma vida; a pausa silencia o áudio especificado e, ao continuar, a reprodução é retomada.
3. Pergunte apenas sobre ambiguidades relevantes. Tome decisões comuns e reversíveis de implementação de forma autônoma. Não reduza os requisitos nem flexibilize os critérios de aceitação.
4. Primeiro execute um pequeno exemplo fornecido pelo compilador, pelo emulador e pelo SARAKURA. Isso verifica a integração entre as ferramentas, não a conclusão do jogo solicitado.

##### Implementar uma primeira versão jogável

- Considere NROM para um jogo pequeno; escolha MMC3 ou outro mapper quando o tamanho ou a troca de bancos exigir. Verifique as funções necessárias da placa com `inspect-rom`, `mapper-info`, `audit-board` e a implementação, sem concluir que há suporte apenas pelo nome do mapper.
- Planeje tamanhos de PRG/CHR, CHR-ROM ou CHR-RAM, espelhamento, bancos fixos, vetores de interrupção e RAM de salvamento. Depois de otimizações ou mudanças de banco, compare o cabeçalho com a organização real. `--nes-local-ram` usa a RAM interna da CPU em `$0000–$07FF`; evite sobreposição com página zero, pilha, buffers OAM e áreas do runtime e das bibliotecas.
- Use o dialeto C do KITAQFC, as bibliotecas FC e `void main(void)`. Não presuma compatibilidade com APIs de GB. Alguns cabeçalhos têm apenas declarações: localize as implementações e inclua os `.c` necessários.
- Considere registradores PPU, NMI, OAM DMA, limite de sprites por linha, rolagem, espelhamento, tabelas de atributos e comportamento de APU/DMC. O espaço livre da fila não é a capacidade de VRAM da PPU; dimensione o trabalho por NMI.
- Converta a fonte original `ascii.c` para CHR de FC e confira CHR, paletas, tabelas de nomes e atributos. Para FDS, verifique separadamente acesso ao disco, salvamento e requisitos de BIOS; não presuma as mesmas condições de inicialização de um cartucho.

- Primeiro conecte inicialização, título, personagem controlável, sucesso ou fracasso e reinício. Amplie o conteúdo depois.
- Preserve os originais editáveis de gráficos, música e efeitos, além das etapas de geração. Confirme que a compilação realmente utiliza os dados exportados.
- Escreva comentários no código em inglês e relatórios de andamento em português do Brasil. Mantenha os relatórios padrão do SARAKURA em inglês.

##### Relacionar cada compilação à sua execução

Separe as saídas por iteração, como em `out/iter-001`. Registre comandos, códigos de saída e hashes de código, materiais, ferramentas, ROM e metadados. Nunca execute uma ROM antiga depois de uma compilação com falha. Mapas, mapas de código-fonte e informações de depuração devem ser da mesma compilação da ROM.

O exemplo a seguir verifica NROM sem entrada. Prepare `main.c`, as implementações necessárias e o arquivo CHR; escolha o mapper adequado. Não passe metadados de compilação para `--kitaqfc-debug` do KUROSAKI sem conferir primeiro o formato exigido.

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


Uma execução de 300 quadros sem entrada é apenas uma verificação inicial. Acrescente cenários de jogo com sequências de ações antes de afirmar que o jogo funciona.

##### Reproduzir sequências de entrada

- `--pad1` e `--pad2` usam máscaras de bits NES brutas: A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. Não confunda esses valores com os `BTN_*` da biblioteca.
- `run --pad1` aplica entrada fixa. Para ações em sequência, crie um replay que diferencie pressionar, manter pressionado e soltar. Consulte `Replay` e `ReplayFrame`. Na CLI pública, `replay-record` grava entrada neutra, não uma partida jogada por uma pessoa.
- Confira por conta própria o SHA-256 da ROM, o alvo do replay e o intervalo de quadros. `replay-run --verify` só compara o hash final esperado quando ele existe; não valida integralmente o jogo nem a identidade da ROM. Não sobrescreva os resultados esperados com os observados apenas para fazer o teste passar.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Prepare o replay para a ROM em teste. A CLI pública não oferece `--emit-diagnostics` em `replay-run`. Não invente essa opção nem use rastreamentos de CPU como eventos de diagnóstico. Para analisar sequências de entrada com SARAKURA, crie um programa de testes dentro do projeto usando as APIs públicas de `kurosaki-core`: `RunOptions.replay_frames` e `diagnostic_events_from_trace_and_report`. Execute a mesma ROM, replay e condições de quadros; gere JSONL a partir do rastreamento e do relatório diagnóstico dessa execução. Confira a configuração e o intervalo preservado do rastreamento, e compare imagens e observações do programa de testes com o replay da CLI. Não apresente diagnósticos sem entrada como evidência de um cenário jogável. Se faltar o ambiente necessário, informe que essa verificação está incompleta.

##### Verificar imagem, som, estado e desempenho

- Salve cenários que diferenciem pressionar, segurar e soltar. Percorra todos os caminhos especificados: inicialização, início, movimento, ações, colisões, rolagem, mudanças de fase, fim de jogo, reinício, pausa e, quando aplicável, salvamento ou comunicação.
- Preserve PNGs de quadros relevantes, entradas, relatórios de execução, JSONL de diagnóstico, WAVs e as observações necessárias de estado ou memória. Verifique os quadros alcançados e o motivo da parada. Abra as imagens de fato: uma captura isolada não comprova movimento ou resposta aos controles. Compare contadores, posições e mudanças de estado com o esperado; confira bordas da tela, limites de tiles e atributos e cenas com muitos sprites.
- Verifique música, efeitos, reprodução simultânea, cortes, pausa e retomada. Gerar um WAV não comprova que o áudio está correto. Se não puder ouvi-lo, diferencie os testes numéricos ou de forma de onda das qualidades audíveis ainda não verificadas.
- Meça cenas pesadas, trabalho da CPU de destino, atualizações e transferências; em FC, inclua o trabalho de NMI. A velocidade do emulador no computador não é a frequência do jogo nem prova de desempenho no hardware real. Continuar com `--allow-unimplemented` não comprova suporte ao recurso ausente.

##### Analisar, corrigir e testar novamente

- Forneça ao SARAKURA os metadados da ROM testada e o JSONL de diagnóstico daquela execução. Um rastreamento de CPU ou relatório comum não serve como substituto. `--frames` define condições de análise; SARAKURA não executa a ROM nem modifica o código automaticamente.
- Leia `report.html`, `ai_diagnostics.json`, `repair_prompt.md` e `retest_plan.json`. Compare os diagnósticos com reprodução, imagens, áudio e código. Separe localizações ou causas inferidas de fatos verificados e laços de espera normais de travamentos. Avalie cada aviso e registre eventos não suportados ou limites da análise. Não oculte avisos com filtros nem encurte testes para obter aprovação.
- Reduza falhas a casos mínimos, corrija a causa e recompile. Se a origem estiver no compilador ou emulador, isole o defeito do código do jogo e acrescente verificação de regressão à correção da ferramenta.
- Repita os testes com as mesmas entradas, semente aleatória, máquina e padrão de vídeo, mapper, quadros observados e configurações de diagnóstico. Cada ROM precisa dos metadados correspondentes; não reutilize estados salvos indiscriminadamente após mudar código ou organização da RAM.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Use as diferenças de diagnóstico junto com a avaliação de controles, gráficos e áudio. Se a mesma falha se repetir, reavalie as evidências e a hipótese em vez de continuar fazendo mudanças arbitrárias.

##### Critérios de conclusão e entregáveis

Repita todos os cenários obrigatórios com a ROM final compilada a partir do código e das configurações entregues. Invencibilidade, entradas automáticas de teste ou outro mapper, isoladamente, não verificam uma partida normal na versão final. Entregue uma tabela relacionando requisitos e testes, explique os avisos restantes e identifique o que não foi verificado ou não é suportado. Se não houve teste em hardware físico, informe isso explicitamente.

Entregue código-fonte, identificação de ferramentas e bibliotecas, materiais editáveis, scripts reproduzíveis de compilação e testes, ROM, evidências finais e um README com instalação, controles e limitações conhecidas. Inclua replays e o programa de testes quando necessários. Publique ou envie arquivos externamente apenas no escopo autorizado de forma explícita. Remova compilações intermediárias e rastreamentos temporários desnecessários após a verificação, preservando fontes, materiais, entregáveis e evidências de regressão necessárias.

Se o ambiente ou as permissões impedirem uma verificação obrigatória, informe os passos exatos para reprodução e a ação necessária. Não marque o trabalho como concluído.

</details>
<!-- development-prompt:pt:end -->

## Organização do repositório

```text
kitaqfc/                  # Raiz do repositório
├─ kitaqfc/               # Fontes de compilação do compilador
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Compilador pronto, em modo Release
├─ kitaqfc.exe.config     # Configuração do runtime .NET Framework
├─ lib/                   # Bibliotecas de apoio em C
├─ examples/              # Programas didáticos e fonte de caracteres original
├─ scripts/build.ps1      # Reconstrói o executável Release
├─ LICENSE
└─ LICENSE.ja
```

O compilador pronto exige Windows com .NET Framework 4.8. Baixe o ZIP do repositório para manter juntos o executável, sua configuração, as bibliotecas e os avisos de licença. Para recompilar, também são necessários o .NET Framework 4.8 Developer Pack e o Visual Studio Build Tools. A partir da raiz do repositório:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Uma compilação Release copia o executável e sua configuração para a raiz do repositório. As compilações Debug ficam em `kitaqfc/bin/Debug` e não substituem o compilador Release distribuído. Caches de compilação e arquivos PDB não são distribuídos. Consulte o [registro de compilação binária](BINARY_BUILD.json) para conhecer as entradas e o SHA-256.

## Compilar e começar a usar

Use Windows, .NET Framework 4.8 Developer Pack e Visual Studio Build Tools com MSBuild. Execute a partir de um prompt Developer PowerShell.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuais e licenças

- [Compilador em português](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html) / [Biblioteca em português](https://bartaro.github.io/kitaq-docs/pt/fc-library.html)
- [Manual em inglês](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manual em japonês](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Fontes do manual para leitura offline](https://github.com/bartaro/kitaq-docs)
- [Licença](LICENSE) / [Tradução de referência em japonês](LICENSE.ja)

A licença do projeto não substitui as condições de terceiros relativas a fontes de caracteres, dependências, logotipos ou marcas. Preserve os avisos incluídos ao redistribuir.
