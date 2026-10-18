# FCSEQ 0.5.0

## en

Added MMC5, SUNSOFT 5B, EPSM and VT02 editing, preview and export, plus NSF2 import, export and playback. A single-track Key Editor now provides piano keys, note dragging and resizing. The interface can switch between English and Japanese. Waveforms offer line or smooth-curve views with editable sample points. CLI, Python bindings and all nine manuals have been updated. EPSM rhythm preview uses original synthesized percussion; see AUDIO_FORMATS for playback limits.

## ja

MMC5・SUNSOFT 5B・EPSM・VT02の編集・試聴・出力と、NSF2の読み込み・書き出し・再生に対応しました。トラックごとに鍵盤付きキーエディタを開き、ノートの移動や長さ変更ができます。表示言語を英語・日本語で切り替えられます。波形は編集点を示した直線・滑らかなカーブで表示します。CLI、Python bindings、9言語の説明書も更新しました。EPSMのリズム試聴には独自合成した打楽器を使用します。再生上の制限はAUDIO_FORMATSをご覧ください。

## ko

MMC5, SUNSOFT 5B, EPSM, VT02 편집·미리 듣기·출력과 NSF2 가져오기·내보내기·재생을 추가했습니다. 트랙별 키 에디터에서 건반과 함께 음표를 이동하고 길이를 바꿀 수 있습니다. 영어·일본어 전환, 편집점을 표시하는 직선·곡선 파형 보기를 제공합니다. CLI, Python bindings와 9개 언어 설명서를 갱신했습니다. EPSM 리듬 미리 듣기는 자체 합성한 타악기를 사용합니다.

## zh-CN

新增 MMC5、SUNSOFT 5B、EPSM、VT02 的编辑、试听和导出，以及 NSF2 导入、导出和播放。可为每条轨道打开带钢琴键盘的编辑器，移动音符并调整长度。界面支持英语和日语切换，波形可显示为带编辑点的折线或平滑曲线。CLI、Python bindings 和九种语言的说明书也已更新。EPSM 节奏试听使用自行合成的打击乐。

## zh-TW

新增 MMC5、SUNSOFT 5B、EPSM、VT02 的編輯、試聽與匯出，以及 NSF2 匯入、匯出與播放。每條音軌可開啟附鋼琴鍵盤的編輯器，移動音符並調整長度。介面支援英語與日語切換；波形可顯示為附編輯點的折線或平滑曲線。CLI、Python bindings 與九種語言的說明書亦已更新。EPSM 節奏試聽使用自行合成的打擊樂。

## fr

Ajout de l'édition, de l'écoute et de l'export MMC5, SUNSOFT 5B, EPSM et VT02, ainsi que de l'import, de l'export et de la lecture NSF2. Le nouvel éditeur de piste propose un clavier, le déplacement et le redimensionnement des notes. L'interface passe de l'anglais au japonais et les ondes se dessinent en segments ou en courbes lisses. CLI, bindings Python et neuf manuels ont été actualisés. Les percussions EPSM sont synthétisées à partir de données originales.

## es

Se añaden edición, escucha y exportación de MMC5, SUNSOFT 5B, EPSM y VT02, e importación, exportación y reproducción NSF2. El editor de pista incorpora piano y permite mover notas y ajustar su duración. La interfaz cambia entre inglés y japonés; las ondas muestran líneas o curvas suaves con puntos editables. Se actualizan la CLI, los bindings Python y los nueve manuales. La percusión EPSM se sintetiza con datos originales.

## de

MMC5, SUNSOFT 5B, EPSM und VT02 lassen sich bearbeiten, vorhören und exportieren; NSF2 wird importiert, exportiert und abgespielt. Der neue Spur-Editor bietet Klaviertasten sowie verschiebbare und verlängerbare Noten. Die Oberfläche ist auf Englisch und Japanisch verfügbar. Wellen erscheinen als Geraden oder glatte Kurven mit editierbaren Punkten. CLI, Python-Bindings und neun Anleitungen wurden aktualisiert. EPSM-Schlagzeug wird aus eigenen Daten synthetisiert.

## pt-BR

Foram adicionados edição, prévia e exportação de MMC5, SUNSOFT 5B, EPSM e VT02, além de importação, exportação e reprodução NSF2. O editor de pista oferece piano, movimentação de notas e ajuste de duração. A interface alterna entre inglês e japonês; as ondas mostram linhas ou curvas suaves com pontos editáveis. CLI, bindings Python e nove manuais foram atualizados. A percussão EPSM é sintetizada com dados originais.

---

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
