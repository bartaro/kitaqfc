# FCSEQ 0.4.2 — 사용 설명서

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

제작자 **DAISUKE OBA**. **plita** 기반의 범용 패미컴 음악 시퀀서입니다. GUI·CLI·Python 바인딩은 프로젝트 모델, 음성 생성 및 내보내기를 공유합니다. 빈 프로젝트로 시작하며 게임 ROM이나 BIOS가 필요하지 않습니다. 화면은 영어이며 설명서는 9개 언어로 제공됩니다.

**0.4.2** — 미리 듣기·C 출력·NSF의 DMC DAC 초기값을 64로 통일했습니다. DAC와 필터가 안정된 뒤 미리 듣기를 기록하여 DPCM 미사용 시 Triangle/Noise 음량 차이와 무음 구간의 시작 잡음을 수정했습니다. 개별 DPCM 샘플의 초기 레벨은 계속 우선합니다. FCQ/NSF/NSFe 형식은 그대로입니다.

## 설치와 실행

[downloads/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads)에서 Windows x86_64, Linux x86_64, macOS Apple Silicon ARM64 또는 Intel x86_64용 전체 ZIP을 받으세요. 리소스와 라이선스를 포함해 모두 압축 해제합니다. Windows는 `fcseq.exe`, Unix는 `./fcseq`로 실행합니다. 터미널용은 `fcseq-cli`이며 Windows에서는 `.exe`를 붙입니다. [bin/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/bin)에도 개별 실행 파일이 있습니다. Windows 10/11은 Direct3D 12, macOS는 Metal, Linux는 Vulkan/OpenGL·X11/Wayland·ALSA를 사용합니다. Linux 빌드 환경은 Ubuntu 24.04이며 이전 배포판은 직접 빌드가 필요할 수 있습니다. Ubuntu: `sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. GUI에는 호환 그래픽 드라이버, 실시간 재생에는 오디오 장치가 필요하지만 WAV 출력에는 필요 없습니다. 실행 파일은 서명되지 않았고 macOS 공증도 없습니다. 차단되면 소스와 해시를 확인한 뒤 OS의 일반 허용 절차를 사용하세요. 필요하면 `chmod +x fcseq fcseq-cli`로 실행 권한을 줍니다.

Windows에는 [Microsoft Visual C++ x64 재배포 패키지](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)도 필요합니다(`VCRUNTIME140.dll` 사용). 없으면 Microsoft의 지원되는 최신 x64 패키지를 설치하세요. GUI/CLI는 Python에 링크하지 않으며 Python은 선택적인 바인딩과 래퍼에만 필요합니다.

## 음악 편집

SOUND에서 2A03(Pulse 1/2, Triangle, Noise, DPCM: 5음), VRC6(Pulse 2음과 Saw 추가: 8음), VRC7(FM 6음 추가: 11음), FDS(파형 1음 추가: 6음), N163(파형 1~8음 추가: 최대 13음)을 선택합니다. 프로젝트당 확장 구성은 하나이며 음표를 잃는 변경은 거부됩니다.

더블클릭으로 음표 추가, 본체 드래그로 이동·이조, 오른쪽 끝 드래그로 길이를 변경합니다. Ctrl/Shift 클릭 또는 사각형으로 다중 선택, Ctrl+A로 현재 레인 전체 선택. Ctrl+C/X/V 복사·잘라내기·커서에 붙이기, Ctrl+D 복제, Delete 삭제, Ctrl+Z/Y 실행 취소·다시 실행. 방향키로 이동·이조, Shift+위/아래로 옥타브 변경. G 격자, Q 정렬, F 전체 보기, Home 시작, +/- 확대, Shift+휠 시간 이동, UP/DOWN 채널 이동. Space 재생·정지, LOOP 반복, AUDITION 선택 음 미리 듣기. MUTE/SOLO는 출력에도 반영됩니다. SONG에서 최대 16섹션, 각 256마디를 관리합니다. 이름·BPM·마디 수를 클릭해 편집합니다. Ctrl+N/O/S/Shift+S 새로 만들기·열기·저장·다른 이름으로 저장. 유효하지 않은 일괄 변경은 전부 취소됩니다. 복구 파일은 실행 파일 옆 `out/recovery_<process-id>.fcq`이며 열어서 다른 이름으로 저장하세요.

## 음색과 파형

SOUND > Instrument / waveform editor에서 채널 음표를 선택해 편집하고 AUDITION/UNDO/REDO를 사용합니다. Pulse 듀티 4단계, VRC6 Pulse 8단계, Triangle 고정 32단계 파형, Noise 16주기와 긴/짧은 모드. Pulse/Noise 음량 0~15, 감쇠 0은 일정 음량. VRC6 Saw는 0~63 누산기 증가값을 바꾸며 넘침으로 음색도 변합니다. 이 음원에는 자유 파형 RAM이 없습니다.

FDS는 공유 64샘플·6비트 파형, N163은 4비트 파형으로 초기 32샘플×4개이며 음표별 파형 번호를 선택합니다. 길이는 4의 배수, 공유 용량은 `256 - 16 × 활성 채널 수`샘플(8음에서 128)입니다. 길이를 늘리기 전에 다른 파형을 줄이거나 채널 수를 줄이세요. 드래그 그리기, SINE/PULSE/SAW/TRIANGLE, 반전·회전을 지원합니다. 음정·RAM·채널 제한을 검사합니다. VRC7 커스텀 패치 0은 24항목(배율, ADSR, 피드백, 레벨, 키 스케일, 트레몰로/비브라토, 사인/반파) 또는 8바이트로 편집합니다. 프리셋 1~15는 합성 근사 음색이며 ROM 덤프가 아닙니다. FDS 변조 테이블/하드웨어 엔벌로프 전용 편집기는 없습니다. 실기 아날로그 음질의 일치를 보장하지 않습니다.

## DPCM

SAMPLES 또는 SOUND > Import audio file로 WAV/FLAC/Ogg Vorbis/MP3를 불러오거나 드롭합니다. START/END(ms), GAIN, NORMALIZE, REVERSE, RATE 0~15를 조절하고 AUDITION DPCM으로 확인, EXPORT .DMC로 저장합니다. DPCM 레인에 음표를 배치하고 SAMPLE과 재생 RATE를 선택합니다. 원본 PCM을 유지하므로 가공을 되돌릴 수 있습니다. 입력 최대 60초, 스테레오는 모노, DC 제거 및 저역통과 필터 리샘플링을 수행합니다. 음색당 `16n+1`바이트·최대 4081바이트, 최대 64음색, 전체 16320바이트(FDS 8192). 초과 변경은 거부됩니다. 변환 속도와 음표 재생 속도는 별도입니다. 내부 루프와 자동 음정 보정은 없습니다.

## 파일과 출력

`.fcq`로 저장하며 기존 `.fcseq`는 이름을 바꿔 읽습니다. 음표 문서는 `fcseq-2` JSON, `fcseq-1`도 지원하며 NSF 재생 문서는 `fcseq-3`입니다. Import NSF, OPEN, 드롭, 실행 인수로 NSF/NSFe를 엽니다. PREV/NEXT나 번호로 곡 선택, PLAY/STOP·LOOP·1~600초를 설정합니다. 곡 길이 메타데이터를 사용하며 없으면 기본 30초입니다. 재생 전 음성 생성을 취소할 수 있습니다. FCQ에는 원본·선택 곡·시간이 포함되며 자동 채보는 하지 않습니다. 다시 내보내면 원본 바이트를 유지합니다.

Export NSF는 각 섹션을 곡 하나로 변환하여 음원·DPCM을 넣고 길이·페이드 0의 NSFe도 만듭니다. NSF 길이를 무시하는 플레이어는 NSFe를 사용하세요. 섹션 끝에서 정지하며 편집기 LOOP와 별개입니다. NSF v1/NSFe, NTSC 또는 NTSC 지원 듀얼, 비뱅크/4KiB 뱅크, 2A03/DPCM·VRC6·VRC7·FDS·N163를 지원합니다. PAL 전용, NSF2 실행 확장, MMC5, Sunsoft 5B, VT02는 거부되며 반환하지 않는 INIT/PLAY는 제한됩니다.

Export C + DPCM은 `score.c`, `fcseq_score.h`, `fcseq_player.c`, 섹션 바이너리, `.dmc`, `dpcm_bank.bin`, `fcseq_export.json`을 출력합니다. 완전한 게임 ROM은 만들지 않으므로 매퍼와 CPU에서 보이는 코드·데이터·DPCM 배치를 설정해야 합니다. VRC6 매퍼 24(26은 주소 변환 필요), VRC7 85, N163 19 또는 FDS를 가정합니다. KITAQFC에서는 include 전에 `FCSEQ_KITAQFC`를 정의하고 `FCSEQ_WRITE(address, value)`로 사용자 레지스터 쓰기를 연결할 수 있습니다.

## CLI와 Python

아래 예제는 압축 해제한 폴더에서 실행합니다. 섹션·채널·음표·샘플은 0부터, NSF 곡은 1부터 시작합니다. Python 3.10+용 abi3 확장을 같은 OS/CPU 폴더에서 사용합니다. GUI/CLI는 Python 없이 실행됩니다. 전체 API는 `fcseq_native.pyi`. CLI는 음표 이동·이조·길이·정렬·복제·삭제, GUI/Python은 파형 편집을 제공합니다.

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## 빌드 및 검증

이 소스 폴더에서 Rust 1.93.1과 아래 명령을 사용합니다. 로컬 라이브러리는 vendor/에 포함됩니다. Linux 개발 패키지는 저장소 루트 `.github/workflows/fcseq.yml`, macOS는 Xcode Command Line Tools, Windows는 MSVC Build Tools가 필요합니다. Python은 선택 사항입니다. 스크립트로 네이티브 CLI/Python, WAV, NSF/NSFe 왕복, GUI 시작을 확인합니다. 결과/범위는 VERIFICATION.md에 있으며 사람의 마우스/키보드 조작이나 실기 청취 인증은 아닙니다.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
python -c "import shutil,sys; from pathlib import Path; n={'win32':'fcseq_native.dll','darwin':'libfcseq_native.dylib'}.get(sys.platform,'libfcseq_native.so'); shutil.copy2(Path('target/release')/n,'fcseq_native.pyd' if sys.platform=='win32' else 'fcseq_native.so')"
```

## 라이선스

FCSEQ는 MIT, Copyright (c) 2026 DAISUKE OBA입니다. 의존성은 각 라이선스를 유지합니다. 재배포 시 LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/, third_party_sources/를 포함하세요. MPL-2.0 Symphonia 소스와 글꼴/아이콘 표기를 포함합니다. 가져온 음악과 샘플의 권리는 별도입니다.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
