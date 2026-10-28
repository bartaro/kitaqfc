# FCSEQ 0.5.2

## en

Fixed track mute and solo stopping playback. Mix changes now preserve the playback position and loop; selected-note audition and keyboard preview respect the track settings. Multiple solo tracks play together, and mute takes priority. Undo/redo keeps playback running for mix-only changes, and cancelled updates cannot replace a newer mix.

Tempo timing now rounds cumulative row deadlines instead of rounding every sixteenth note separately. At 108 BPM the NTSC scheduler combines 8- and 9-frame steps; PCM uses the same method on its 4 kHz clock. Note starts, note ends, envelopes, shared SSG mixers, preview, WAV and C/NSF/NSF2/NSFe exports follow the same timeline. CLI and Python timing reports were updated.

## ja

トラックのミュートやソロを押すと再生が停止する不具合を修正しました。再生位置とループを保ちながら設定を反映します。選択ノートと鍵盤の試聴にも設定が反映されます。複数トラックのソロは同時に鳴り、ミュートを優先します。ミュートとソロのUndo／Redoでも再生を継続し、取り消した古い処理が音声を上書きしないようにしました。

テンポ計算を、16分音符ごとに丸める方式から累積時刻を丸める方式へ変更しました。108 BPMではNTSCの8フレームと9フレームを組み合わせ、PCMも4 kHzの時計で同じ方法を使います。発音開始・終了、減衰、SSGの共有ミキサー、プレビュー、WAV、C・NSF・NSF2・NSFeの出力を同じ時刻に揃え、CLIとPythonの時間情報も修正しました。

## ko

트랙의 음소거나 솔로를 누르면 재생이 멈추던 문제를 수정했습니다. 재생 위치와 루프를 유지하며 설정을 반영하고, 선택한 음표와 건반 미리 듣기에도 적용합니다. 여러 트랙을 솔로로 지정하면 함께 재생되며 음소거가 우선합니다. 음소거·솔로의 실행 취소와 다시 실행 중에도 재생이 이어지고, 취소된 이전 처리가 새 믹스를 덮어쓰지 않습니다.

16분음표마다 시간을 반올림하는 대신 누적 시각을 반올림하도록 템포 계산을 바꿨습니다. 108 BPM에서는 NTSC의 8프레임과 9프레임 간격을 조합하며, PCM도 4 kHz 클록에서 같은 방식을 씁니다. 발음 시작·종료, 감쇠, SSG 공유 믹서, 미리 듣기, WAV와 C·NSF·NSF2·NSFe 내보내기가 같은 시간축을 따릅니다. CLI와 Python의 시간 정보도 수정했습니다.

## zh-CN

修复了点击轨道静音或独奏会停止播放的问题。切换时保留播放位置和循环，选中音符及键盘试听也会遵循轨道设置。可同时独奏多条轨道，静音优先。撤销或重做静音、独奏操作时继续播放，并防止已取消的旧任务覆盖新混音。

节奏计算改为对累计时刻取整，而非逐个十六分音符取整。108 BPM 下，NTSC 调度会组合 8 帧和 9 帧间隔；PCM 在 4 kHz 时钟上采用相同方式。音符起止、衰减、SSG 共享混音器、预览、WAV 和 C、NSF、NSF2、NSFe 导出使用统一时间轴，CLI 与 Python 的时间信息也已修正。

## zh-TW

修正了點擊音軌靜音或獨奏會停止播放的問題。切換時保留播放位置與循環，選取音符和鍵盤試聽也會遵循音軌設定。可同時獨奏多條音軌，靜音優先。復原或重做靜音、獨奏操作時繼續播放，並防止已取消的舊工作覆寫新混音。

節奏計算改為對累積時刻取整，而非逐個十六分音符取整。108 BPM 時，NTSC 排程會組合 8 幀和 9 幀間隔；PCM 在 4 kHz 時脈上採用相同方式。音符起訖、衰減、SSG 共享混音器、預覽、WAV 和 C、NSF、NSF2、NSFe 匯出使用統一時間軸，CLI 與 Python 的時間資訊也已修正。

## fr

Les boutons de sourdine et de solo n'arrêtent plus la lecture : la position et la boucle sont conservées. L'écoute des notes sélectionnées et du clavier respecte aussi ces réglages. Plusieurs pistes en solo jouent ensemble ; la sourdine reste prioritaire. Annuler ou rétablir ces réglages ne coupe plus la lecture, et une mise à jour annulée ne peut plus remplacer un mixage récent.

Le tempo arrondit désormais les échéances cumulées plutôt que chaque double croche séparément. À 108 BPM, le séquenceur NTSC alterne des intervalles de 8 et 9 images ; le PCM applique la même méthode à 4 kHz. Débuts et fins de notes, décroissance, mixeur SSG partagé, préécoute, WAV et exports C/NSF/NSF2/NSFe suivent la même chronologie. Les durées annoncées par la CLI et Python ont aussi été corrigées.

## es

Los botones de silencio y solo ya no detienen la reproducción: conservan la posición y el bucle. La escucha de las notas seleccionadas y del teclado también respeta estos ajustes. Varias pistas en solo suenan juntas, con prioridad para el silencio. Deshacer o rehacer estos cambios mantiene la reproducción, y una actualización cancelada no puede sustituir una mezcla más reciente.

El tempo ahora redondea los tiempos acumulados en vez de cada semicorchea por separado. A 108 BPM, el secuenciador NTSC combina intervalos de 8 y 9 fotogramas; el PCM aplica el mismo método con su reloj de 4 kHz. El inicio y final de las notas, el decaimiento, el mezclador SSG compartido, la escucha, WAV y las exportaciones C/NSF/NSF2/NSFe usan la misma línea temporal. También se corrigen los tiempos que muestran la CLI y Python.

## de

Stumm- und Solo-Schalter stoppen die Wiedergabe nicht mehr. Wiedergabeposition und Schleife bleiben erhalten; auch ausgewählte Noten und die Klaviatur berücksichtigen diese Einstellungen. Mehrere Solo-Spuren erklingen gemeinsam, wobei Stummschaltung Vorrang hat. Rückgängig und Wiederherstellen dieser Änderungen unterbrechen die Wiedergabe nicht; abgebrochene Berechnungen können keinen neueren Mix überschreiben.

Die Tempoberechnung rundet nun die aufsummierten Zeitpunkte statt jeder Sechzehntelnote einzeln. Bei 108 BPM werden im NTSC-Takt Abstände von 8 und 9 Frames kombiniert; PCM verwendet dasselbe Verfahren mit 4 kHz. Notenanfänge und -enden, Abklingzeiten, gemeinsamer SSG-Mixer, Vorschau, WAV und C-/NSF-/NSF2-/NSFe-Exporte folgen derselben Zeitachse. Die Zeitangaben in CLI und Python wurden ebenfalls korrigiert.

## pt-BR

Os botões de silenciar e solo não interrompem mais a reprodução: a posição e o loop são mantidos. A prévia das notas selecionadas e do teclado também respeita esses ajustes. Várias faixas em solo tocam juntas, com prioridade para silenciar. Desfazer e refazer essas alterações mantém a reprodução, e uma atualização cancelada não substitui uma mixagem mais recente.

O cálculo do andamento agora arredonda os tempos acumulados, em vez de cada semicolcheia separadamente. A 108 BPM, o sequenciador NTSC combina intervalos de 8 e 9 quadros; o PCM usa o mesmo método com seu relógio de 4 kHz. Inícios e finais de notas, decaimento, mixer SSG compartilhado, prévia, WAV e exportações C/NSF/NSF2/NSFe seguem a mesma linha do tempo. Os tempos informados pela CLI e pelo Python também foram corrigidos.

---

# FCSEQ 0.5.1

## en

Fixed Esc cancellation in the section-name dialog and native named-key handling for Enter, Backspace, arrows and other shortcuts. Shortened the Key Editor white keys from 150 to 88 pixels, adjusted black keys and labels, and gave the recovered width to the note grid.

## ja

セクション名の入力画面をEscでキャンセルできない不具合を修正しました。Enter、Backspace、矢印キーなども、plitaから届くキー名を正しく認識します。キーエディタの白鍵を150pxから88pxへ短くし、黒鍵と音名表示を調整しました。空いた幅はノート編集領域に割り当てています。

## ko

섹션 이름 입력 창에서 Esc로 취소할 수 없던 문제를 수정했습니다. Enter, Backspace, 방향키 등 plita가 전달하는 특수 키도 올바르게 인식합니다. 키 에디터의 흰 건반을 150px에서 88px로 줄이고 검은 건반과 음이름 표시를 조정해, 확보한 공간을 음표 편집 영역에 더했습니다.

## zh-CN

修复了节名称输入框无法按 Esc 取消的问题，并正确识别 plita 传入的 Enter、Backspace、方向键等特殊按键。钢琴编辑器的白键由 150px 缩短至 88px，黑键和音名标注也相应调整，腾出的空间用于音符编辑。

## zh-TW

修正了樂段名稱輸入框無法按 Esc 取消的問題，並正確辨識 plita 傳入的 Enter、Backspace、方向鍵等特殊按鍵。鋼琴編輯器的白鍵由 150px 縮短至 88px，黑鍵和音名標示也一併調整，騰出的空間用於音符編輯。

## fr

La touche Échap annule désormais la saisie du nom d'une section. Les touches Entrée, Retour arrière, les flèches et les autres touches spéciales transmises par plita sont aussi reconnues correctement. Dans l'éditeur de piste, les touches blanches passent de 150 à 88 pixels ; les touches noires et les noms de notes ont été adaptés, et la grille profite de l'espace libéré.

## es

Se corrige la cancelación con Esc al cambiar el nombre de una sección, así como el reconocimiento de Intro, Retroceso, las flechas y otras teclas especiales que envía plita. Las teclas blancas del editor de pista se acortan de 150 a 88 píxeles; se ajustan las negras y los nombres de notas, y el espacio ganado se añade a la cuadrícula.

## de

Esc bricht die Eingabe eines Abschnittsnamens jetzt zuverlässig ab. Auch Eingabetaste, Rücktaste, Pfeiltasten und weitere von plita gemeldete Sondertasten werden korrekt erkannt. Im Spur-Editor sind die weißen Tasten nun 88 statt 150 Pixel breit. Schwarze Tasten und Notennamen wurden angepasst; der gewonnene Platz erweitert das Notenraster.

## pt-BR

A tecla Esc agora cancela a edição do nome de uma seção. Enter, Backspace, as setas e outras teclas especiais enviadas pelo plita também são reconhecidas corretamente. As teclas brancas do editor de pista passaram de 150 para 88 pixels; as pretas e os nomes das notas foram ajustados, e o espaço liberado foi acrescentado à grade de notas.

---

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
