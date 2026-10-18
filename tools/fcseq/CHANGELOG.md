# FCSEQ 0.4.2

## en

Preview, C export and NSF initialize the DMC DAC to 64 consistently. Preview capture starts after the DAC/filter settles, fixing triangle/noise balance without DPCM samples and preventing startup transients in silent passages. Individual DPCM sample levels still override the baseline. FCQ/NSF/NSFe formats are unchanged.

## ja

プレビュー・C出力・NSFのDMC DAC初期値を64に統一しました。DACとフィルターが安定してからプレビューを記録し、DPCM未使用時の三角波／ノイズの音量差と無音区間の立ち上がりノイズを修正しています。DPCM音色ごとの初期レベルは引き続き優先されます。FCQ・NSF・NSFeの形式変更はありません。

## ko

미리 듣기·C 출력·NSF의 DMC DAC 초기값을 64로 통일했습니다. DAC와 필터가 안정된 뒤 미리 듣기를 기록하여 DPCM 미사용 시 Triangle/Noise 음량 차이와 무음 구간의 시작 잡음을 수정했습니다. 개별 DPCM 샘플의 초기 레벨은 계속 우선합니다. FCQ/NSF/NSFe 형식은 그대로입니다.

## zh-CN

预览、C 导出和 NSF 的 DMC DAC 初值统一为 64。预览在 DAC/滤波器稳定后开始采集，修复未使用 DPCM 样本时 Triangle/Noise 的音量差和无声片段的起始瞬态。各 DPCM 样本的初始电平仍优先。FCQ/NSF/NSFe 格式不变。

## zh-TW

預覽、C 匯出和 NSF 的 DMC DAC 初值統一為 64。預覽在 DAC/濾波器穩定後開始擷取，修正未使用 DPCM 取樣時 Triangle/Noise 的音量差及無聲片段的起始暫態。各 DPCM 取樣的初始電位仍優先。FCQ/NSF/NSFe 格式不變。

## fr

La préécoute, l’export C et le NSF initialisent le DAC DMC à 64. La capture commence après stabilisation du DAC/filtre : équilibre Triangle/Noise corrigé sans échantillons DPCM et suppression des transitoires au début des silences. Les niveaux propres aux échantillons DPCM restent prioritaires. Formats FCQ/NSF/NSFe inchangés.

## es

Vista previa, exportación C y NSF inicializan el DAC DMC en 64. La captura comienza tras estabilizar DAC/filtro: corrige el equilibrio Triangle/Noise sin muestras DPCM y los transitorios al inicio de silencios. Los niveles de cada muestra DPCM siguen teniendo prioridad. Formatos FCQ/NSF/NSFe sin cambios.

## de

Vorschau, C-Export und NSF initialisieren den DMC-DAC einheitlich mit 64. Die Aufnahme beginnt nach dem Einschwingen von DAC/Filter: Triangle/Noise-Pegel ohne DPCM-Samples und Einschaltimpulse in stillen Passagen sind korrigiert. Individuelle DPCM-Samplepegel haben weiterhin Vorrang. FCQ/NSF/NSFe-Formate bleiben unverändert.

## pt-BR

Prévia, exportação C e NSF inicializam o DAC DMC em 64. A captura começa após a estabilização do DAC/filtro: corrige o equilíbrio Triangle/Noise sem amostras DPCM e transientes no início dos silêncios. Níveis individuais das amostras DPCM continuam prioritários. Formatos FCQ/NSF/NSFe inalterados.
