# FCSEQ 0.5.0

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ는 패미컴 음원으로 곡을 만드는 음악 편집기입니다. plita 기반 화면에서 음표를 배치하고 음색과 파형을 다듬으며 소리를 확인할 수 있습니다. GUI, CLI, Python bindings는 같은 프로젝트 형식과 오디오 엔진을 사용합니다. 처음에는 특정 게임과 관계없는 빈 프로젝트가 열립니다.

## 설치와 표시 언어

[downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads)에서 운영체제와 CPU에 맞는 ZIP을 받아 전체를 압축 해제하세요. Windows와 Linux는 x86_64, macOS는 Apple Silicon과 Intel 버전을 제공합니다. Windows에서는 `fcseq.exe`, Linux와 macOS에서는 `./fcseq`를 실행합니다. 실행 권한이 없다면 `chmod +x fcseq fcseq-cli`를 사용하세요. 글꼴과 라이선스 파일도 프로그램과 함께 보관해야 합니다.

화면은 영어와 일본어를 지원합니다. **SOUND → 日本語 / English**에서 바꾸면 다음 실행에도 유지됩니다. NSF 플레이어에는 같은 기능의 버튼이 있습니다. 첫 실행에서는 시스템 언어를 따르며, `FCSEQ_LANGUAGE=ja` 또는 `en`으로 지정할 수도 있습니다. 일본어 표시용 Noto Sans JP가 포함되어 있습니다.

Windows에는 Direct3D 12 드라이버와 Microsoft Visual C++ x64 런타임이 필요합니다. macOS는 Metal을 사용합니다. Linux 배포판은 Ubuntu 24.04에서 빌드하며 ALSA, X11/Wayland, Vulkan/OpenGL을 사용합니다. 오래된 배포판에서는 소스 빌드가 필요할 수 있습니다. 실시간 재생에는 오디오 장치가 필요하지만 WAV 출력에는 필요하지 않습니다. 바이너리는 서명되지 않았으며 macOS 공증도 받지 않았습니다.

## 음표 입력과 키 에디터

**SOUND**에서 음원을 선택한 뒤 트랙의 빈 곳을 두 번 클릭하면 음표가 생깁니다. 음표 본체를 드래그하면 위치와 음높이가 바뀌고, 오른쪽 끝을 드래그하면 길이가 바뀝니다. 한 트랙은 하나의 하드웨어 채널이므로 음표를 겹칠 수 없습니다.

트랙 이름을 두 번 클릭하거나 **E**를 누르면 왼쪽에 피아노 건반이 있는 키 에디터가 열립니다. 세로축은 반음 단위의 음높이, 가로축은 시간입니다. 건반을 클릭해 소리를 들어 보세요. 휠은 표시 음역을 옮기고, 건반 쪽 −/+ 버튼은 음역 표시를 확대·축소합니다. Shift+휠은 시간을 이동합니다. 트랙 화살표로 다른 트랙을 열 수 있으며 E를 다시 누르면 전체 트랙으로 돌아갑니다. 노이즈·DPCM·리듬 트랙은 해당 소리나 재생 속도에 맞는 범위를 표시합니다.

Ctrl/Shift+클릭 또는 빈 공간 드래그로 여러 음표를 선택합니다. Ctrl+A는 현재 트랙 전체 선택, Ctrl+C/X/V는 복사·잘라내기·커서 위치에 붙여넣기, Ctrl+D는 복제, Delete는 삭제입니다. Ctrl+Z/Y로 실행을 취소하거나 다시 적용합니다. 방향키는 이동·이조, Shift+위/아래는 옥타브 이조입니다. G로 격자를 바꾸고 Q로 음표를 격자에 맞춥니다. F는 전체 보기, Home은 시작 위치, +/−는 시간축 확대·축소입니다.

Space로 재생·정지합니다. LOOP는 현재 섹션을 반복하고 AUDITION은 선택 음표를 재생합니다. 음소거와 솔로는 내보내기에도 적용됩니다. SONG에서 섹션을 추가·복제·이름 변경·재배열할 수 있습니다. 최대 16개 섹션, 섹션마다 256마디를 지원합니다. 템포와 마디 수는 표시를 클릭해 바꿉니다. Ctrl+N/O/S/Shift+S는 새 프로젝트·열기·저장·다른 이름으로 저장입니다. 복구 파일은 실행 파일 옆 `out/recovery_<process-id>.fcq`에 기록됩니다. 파일을 열고 다른 이름으로 저장하세요.

## 음원과 음색

기본 2A03은 펄스 2개, 삼각파, 노이즈, DPCM의 5개 트랙입니다. 프로젝트마다 확장 음원 하나를 추가할 수 있습니다.

| 확장 음원 | 추가 채널 | 전체 트랙 |
|---|---|---:|
| VRC6 | 펄스 2개, 톱니파 | 8 |
| VRC7 | FM 6개 | 11 |
| FDS | 웨이브테이블 1개 | 6 |
| N163 | 웨이브테이블 최대 8개 | 13 |
| MMC5 | 펄스 2개, PCM | 8 |
| SUNSOFT 5B | SSG 3개, 공통 노이즈·엔벌로프 | 8 |
| EPSM | SSG 3개, FM 6개, 리듬 6개 | 20 |
| VT02 | 펄스 2개, 삼각파, 노이즈, PCM | 10 |

음표가 사라지는 음원 변경이나 채널 축소는 거부됩니다. 해당 트랙을 비우거나 새 프로젝트를 만드세요.

음표를 선택하고 **INSTRUMENT / WAVEFORM**을 열어 음색을 편집합니다. 펄스의 듀티 비율, 노이즈의 주기와 장·단주기 모드, VRC6 톱니파의 누산 속도를 조정할 수 있습니다. 이 채널들은 고정 파형이므로 파형을 자유롭게 그릴 수는 없습니다.

FDS는 모든 음표가 공유하는 64점·6비트 파형을 제공합니다. N163은 4비트 파형을 여러 개 저장하며 음표별로 파형 슬롯을 고릅니다. 그래프를 드래그하거나 사인파·펄스·톱니파·삼각파 프리셋에서 시작하세요. 반전과 회전도 가능합니다. **CURVE / LINE**은 부드러운 곡선과 직선 연결을 전환합니다. 실제로 저장되는 값은 편집점이며, 곡선은 표시용 보간입니다. 사인파의 곡선을 보면서 각 점의 양자화된 값을 정확히 편집할 수 있습니다. 포인터를 올리면 점의 번호와 값이 나타납니다.

N163 파형 길이는 4의 배수입니다. 모든 채널이 `256 − 16 × 활성 채널 수`만큼의 파형 메모리를 공유하므로 8채널에서는 128샘플을 사용할 수 있습니다. FDS 변조 테이블과 하드웨어 엔벌로프 편집은 제공하지 않습니다.

VRC7 사용자 음색 0에서는 오퍼레이터와 엔벌로프 등을 편집합니다. 프리셋 1~15는 합성한 근사 음색입니다. EPSM은 16개의 FM 음색을 제공하며, 각 음색의 오퍼레이터 4개, 알고리즘, 피드백, 엔벌로프, 변조를 편집할 수 있습니다. 변경은 같은 음색을 쓰는 모든 음표에 적용됩니다. FM과 리듬에는 팬 설정도 있습니다. 현재 미리 듣기와 WAV는 모노지만 내보낸 레지스터에는 팬 값이 유지됩니다. EPSM 리듬 미리 듣기는 자체 합성 타악기를 사용하며 Yamaha ROM 덤프를 포함하지 않습니다. 실제 EPSM에서는 장치 내장 리듬음이 재생됩니다.

5B와 EPSM의 SSG에서는 톤·노이즈·엔벌로프를 설정합니다. 노이즈 주기와 엔벌로프 발생기는 세 채널이 공유하므로 나중에 설정한 값이 다른 채널에도 영향을 줄 수 있습니다.

## 오디오를 악기로 사용하기

SAMPLES에서 WAV, FLAC, Ogg Vorbis, MP3를 가져오거나 창에 파일을 놓으세요. 시작·끝, 게인, 정규화, 역재생을 조정할 수 있으며 원본은 유지됩니다. 최대 60초의 원본을 지원하고 스테레오는 모노로 합칩니다.

DPCM 트랙에 음표를 놓고 샘플과 재생 속도를 선택하세요. AUDITION DPCM은 변환된 소리를 들려주며 EXPORT .DMC는 파일로 저장합니다. 변환 과정에서 DC 성분을 제거하고 저역 통과 필터로 리샘플링합니다. 샘플 하나는 최대 4081바이트, 공통 뱅크는 16320바이트이며 FDS에서는 8192바이트입니다. 너무 크면 구간을 줄이거나 변환 속도를 낮추세요. 변환 속도와 음표 재생 속도는 별도 설정입니다.

MMC5·VT02의 PCM 트랙도 같은 원본과 가공 설정을 사용합니다. 8비트 PCM을 4 kHz로 스트리밍하고 음높이에 따라 재생 속도를 바꿉니다. PCM에는 DPCM 바이트 제한을 적용하지 않습니다. VT02의 DPCM과 PCM은 재생 장치를 공유하므로 한 섹션에서는 둘 중 하나만 사용하세요. 샘플 루프와 자동 음정 보정은 지원하지 않습니다.

## 프로젝트와 NSF

편집 프로젝트는 `.fcq`로 저장합니다. 기존 `.fcseq`는 확장자를 `.fcq`로 바꿔 열 수 있습니다. 음표 프로젝트는 `fcseq-2` JSON 형식을 유지하며 `fcseq-1`도 읽습니다. 음악 데이터를 포함하는 재생 프로젝트는 `fcseq-3`입니다.

NSF, NSF2, NSFe를 열고 곡 번호와 시간을 정한 뒤 재생하세요. 시간 정보가 없으면 기본 30초이며 1~600초로 바꿀 수 있습니다. FCQ로 저장하면 원본과 재생 설정을 포함합니다. 가져오기는 재생용이며 임의의 NSF를 편집 가능한 음표로 채보하지 않습니다. 같은 컨테이너로 다시 내보내면 원본 바이트가 유지됩니다.

내보내기 대화상자에서 `.nsf`, `.nsf2`, `.nsfe`를 선택하면 섹션마다 하나의 곡이 됩니다. NSF·NSF2와 함께 곡 길이와 페이드 0을 넣은 NSFe도 만듭니다. 곡은 섹션 끝에서 멈추며 편집기의 LOOP와는 별개입니다. NTSC와 NTSC 재생이 가능한 양 지역 파일, 4 KiB 뱅킹, 위 음원을 지원합니다. NSF2의 타이머 IRQ, 계속 실행되는 INIT, PLAY 호출 억제도 처리합니다. PAL 전용 파일은 지원하지 않습니다. EPSM과 NSF2 타이머 포트가 겹치므로 둘을 타이머 IRQ 플래그와 함께 사용하는 파일은 거부합니다. [기술 자료](docs/AUDIO_FORMATS.md)에서 범위를 확인할 수 있습니다.

C + DPCM 내보내기는 C 드라이버, 헤더, 레지스터 데이터, 샘플 뱅크, 배치 설명을 만듭니다. 완성된 게임 ROM은 아닙니다. 대상 매퍼를 설정하고 코드와 데이터를 CPU가 읽을 수 있게 배치하세요. `fcseq_tick`은 `FCSEQ_TICK_RATE_HZ`에 맞춰 호출합니다. 보통 NTSC 프레임 속도이며 PCM이 있는 프로젝트는 4000 Hz입니다. KITAQFC에서는 `FCSEQ_KITAQFC`를 정의하고, 필요하면 `FCSEQ_WRITE(address, value)`를 바꿔 사용할 수 있습니다.

## CLI, Python, 빌드

`fcseq-cli help`로 명령을 확인하세요. Windows에서는 `.exe`를 붙입니다. 섹션·채널·음표·음색·샘플 번호는 0부터, NSF 곡 번호는 1부터 시작합니다.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli nsf-render song.nsf2 preview.wav 1 10
./fcseq-cli set-epsm-parameter song.fcq edited.fcq 0 44 4
```

Python 3.10 이상에서 운영체제와 CPU에 맞는 네이티브 모듈을 사용합니다. GUI·CLI에는 Python이 필요하지 않습니다. [API 목록](fcseq_native.pyi)에서 `Controller`, `set_sound`, `set_epsm_parameter`, 파형 편집과 내보내기를 확인할 수 있습니다. 예를 들어 `c.set_expansion('epsm')`, `c.add_note(0, 8, 0, 8, 69)`, `c.export_nsf('song.nsf2')`로 FM 음표를 만들어 내보낼 수 있습니다.

Rust 1.93.1과 C++14 컴파일러로 `cargo build --locked --release`, `cargo test --locked --tests`를 실행합니다. CLI 전용은 `--no-default-features --bin fcseq-cli`, Python 확장은 `--features python --lib`을 사용합니다. Windows는 MSVC Build Tools, macOS는 Xcode Command Line Tools, Linux는 저장소의 `.github/workflows/fcseq.yml`에 있는 개발 라이브러리가 필요합니다.

FCSEQ 애플리케이션 코드는 MIT 라이선스이며 Copyright (c) 2026 DAISUKE OBA입니다. ymfm(BSD-3-Clause), Symphonia(MPL-2.0), 글꼴(OFL)은 각 라이선스를 유지합니다. 재배포할 때 LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/, third_party_sources/를 포함하세요. 게임 ROM·FDS BIOS·상용 음원 ROM 덤프는 포함하지 않습니다. 가져온 곡과 샘플의 권리는 별도입니다. [고지](THIRD_PARTY_NOTICES.md)와 [검증 기록](VERIFICATION.md)을 참고하세요.
