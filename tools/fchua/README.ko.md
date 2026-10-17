# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA는 KITAQFC용 NES 타일, 네임테이블, 팔레트, 8x8 메타스프라이트를 편집합니다. 독립 CLI는 GUI와 같은 코어를 사용하며 화면, GPU, plita 설치, Python이 필요 없습니다.

## GUI 다운로드

아래 Windows x64 ZIP 전체를 풀고 FCHUA.exe를 실행하세요. assets/fonts와 라이선스 폴더를 실행 파일 옆에 유지하세요. 실행에 Rust나 Python은 필요 없으며 wgpu 호환 그래픽 장치와 드라이버가 필요합니다. GUI 표시는 영어이고 설명서는 위의 9개 언어로 제공됩니다.

[FCHUA GUI 0.1.0 - Windows x64 ZIP](https://github.com/bartaro/kitaqfc/raw/refs/heads/main/tools/fchua/dist/FCHUA-GUI-0.1.0-windows-x86_64.zip)

File > New project 또는 Open project / raw로 시작합니다. 타일과 팔레트를 선택하여 Tile에서 그리고 Map에 배치합니다. 맵 팔레트는 16x16픽셀 단위입니다. 각 메뉴에서 배경/스프라이트 패턴 테이블을 선택합니다. 편집 데이터는 .fch로 저장하세요. PNG 미리보기는 프로젝트가 아닌 이미지입니다. Export KITAQFC pack으로 게임 자산을 내보냅니다. 글꼴이 다르면 ZIP 전체와 PLITA_UI_FONT 설정을 확인하세요. 창이 열리지 않으면 그래픽 드라이버를 갱신하거나 CLI를 사용하세요.

공개 소스에는 필요한 plita 크레이트가 포함됩니다. tools/fchua에서 아래 명령으로 GUI를 빌드하고 실행합니다. 마지막 프로젝트 경로는 생략 가능합니다. 이번 릴리스에서 GUI 실행 파일은 Windows x64에서 검증했습니다.

Windows x64에서 릴리스 빌드, 소프트웨어 렌더링과 입력/상태 자동 테스트를 통과했습니다. 데스크톱 자동화 도구가 앱 승인 대기 중 시간 초과되어 실제 창 클릭과 파일 대화상자는 미검증입니다. [검증 기록](VERIFICATION.md)을 참조하세요.

```sh
cd tools/fchua
cargo run --release --locked --features gui --bin fchua_gui
cargo run --release --locked --features gui --bin fchua_gui -- scene.fch
cargo test --release --locked --features gui
```

## 빌드와 실행

빌드에는 Rust 1.92 이상이 필요합니다. Windows는 target/release/fchua.exe, Linux와 macOS는 ./target/release/fchua를 실행합니다. --help로 구문을 확인합니다. 아래 예제는 실행 파일을 fchua로 표기합니다. 출력 상위 폴더를 먼저 만드세요.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## 작업 순서

빈 프로젝트를 만들거나 PNG를 가져온 뒤 보고서와 미리보기를 확인하고 내보냅니다. --auto-palette는 변환 오차를 줄이는 NES 배경 팔레트 네 개를 선택합니다. 생략하면 현재 팔레트를 사용합니다. --project base.fch로 팔레트와 패턴 테이블 설정을 가져옵니다. 투명 픽셀은 공통 배경색과 합성됩니다. 가져오기는 맵과 선택한 배경 테이블을 교체하므로 같은 테이블의 스프라이트 그림도 바뀝니다.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## 명령 목록

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

## 일괄 편집

스크립트는 JSON 배열입니다. 모든 명령을 검증한 뒤 문서를 갱신하며 실패하면 저장하지 않습니다. --dry-run은 검사만 합니다. 알 수 없는 명령과 필드는 오류입니다. 타일 명령은 0..511, 맵과 스프라이트는 선택한 테이블 안의 0..255를 사용합니다.

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

## GUI와 Python

plita GUI는 Roboto와 Lucide 아이콘 메뉴, 호버·누름·놓음 표시 및 툴팁을 사용합니다. File에서 새 문서, 열기, 저장, 다른 이름으로 저장, PNG 가져오기와 내보내기를 합니다. Ctrl+O/S/Shift+S는 열기/저장/다른 이름 저장, Ctrl+Z/Y는 실행 취소/다시 실행, Ctrl+C/V는 타일 복사/붙여넣기입니다. 선택만 바꾸면 수정 상태가 되지 않습니다. 수정 후 닫기, 새 문서, 다른 문서 열기는 변경 폐기를 확인합니다.

Map/Tile/Sprite/Check로 중앙 보기를 바꿉니다. B/E/F/L/R/I는 연필/지우개/채우기/선/사각형/색 선택, 1..4는 색입니다. 한 드래그는 한 실행 취소이며 Escape로 취소합니다. Fit, 확대 아이콘, 휠, 스크롤바 손잡이와 트랙을 사용할 수 있습니다. 좁은 화면은 Tiles/Palette 탭으로 전환합니다. 팔레트 칸을 선택한 뒤 아래 NES 색을 누르세요. 색 0은 공통 배경입니다. Sprite는 8픽셀 격자에 타일을 놓고 색 선택 도구로 부품을 선택하며 지우개로 제거합니다. 메뉴에서 프레임 추가/복제와 부품 반전/삭제를 합니다. Check는 프레임별 검사이며 전체 장면이나 에뮬레이터 검증이 아닙니다.

독립 CLI 패키지에도 선택적 Python 바인딩(Python 3.10 이상)이 포함됩니다. GUI, GPU, plita 없이 tools/fchua에서 pip로 빌드하거나 제공된 Windows x64 wheel을 설치합니다. Controller는 프로젝트 입출력, PNG 변환, 일괄 명령과 RAW/C 내보내기를 지원하며 GUI_AVAILABLE=False입니다. gui,python 기능을 켠 빌드는 run_editor와 ButtonInteraction도 제공합니다. Controller는 실행 중 GUI가 아닌 독립 문서를 편집하므로 .fch로 교환합니다. Python 저장은 기존 파일을 교체하며 CLI의 --force 보호와 다릅니다.

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

## 하드웨어와 파일

.fch는 format=fchua, version=1인 JSON입니다. CHR 8192바이트, 팔레트 32바이트, 32x30 맵, 속성 64바이트, 두 패턴 테이블 선택과 최대 256프레임을 저장합니다. 이전 .kfcproject를 변환할 수 있으며 불일치하는 팔레트 맵보다 하드웨어 속성을 우선합니다. PNG는 8의 배수인 8x8..256x240입니다. 16x16 블록마다 4색 팔레트 하나를 사용합니다. 배경 테이블은 256타일이며 가져올 때 0번을 빈 타일로 예약하고 초과는 거부합니다.

출력은 .chr(8192), .pal(32), .nam(960), .atr(64), 메타스프라이트 JSON, 전체 프레임 C/H, 매니페스트와 .fch입니다. C 배열은 __prg_rom을 사용합니다. 게임에서 PPU 테이블을 프로젝트와 맞춰야 하며 설정 코드는 생성하지 않습니다. RAW CHR은 16바이트 단위로 최대 8192바이트, 나머지는 정확한 길이가 필요합니다. NES 색은 0..63입니다. 프레임당 최대 64부품, 좌표 -128..127이며 x=-1은 0xFF 종료 표시 때문에 금지됩니다. 8x8 스프라이트만 지원하고 한 스캔라인의 부품이 8개를 넘으면 경고합니다.

## 보호 기능과 오류

기존 출력에는 --force가 필요하지만 입력과 출력 경로는 같을 수 없습니다. 프로젝트는 임시 파일을 통해 원자적으로 교체합니다. export --out은 폴더입니다. --force는 해당 이름의 자산만 교체하며 다른 파일을 삭제하지 않습니다. --prefix는 C 식별자여야 합니다. 성공은 JSON 하나, 오류는 표준 오류에 JSON을 출력합니다. 종료 코드: 0 성공, 1 validate --strict 경고, 2 인수/문서/변환/입출력 오류. 교체 전 백업하세요. 네트워크나 AI 생성 서비스를 호출하지 않습니다.

원본 코드는 MIT입니다. 종속성은 각 라이선스를 따릅니다. LICENSE와 THIRD_PARTY_NOTICES.md를 확인하세요.
