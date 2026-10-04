# FC/NES 표준 라이브러리 구성



<!-- fc-current-20261004:ko:start -->
### 현재 컴파일러와 라이브러리 배포 구성 — 2026년 10월 4일

fc.h에는 physics3d와 audio_vblank의 선언이 포함됩니다. 필요한 구현 파일은 별도로 링크하세요. entity_update_all과 entity_draw_all은 활성 슬롯을 ID 오름차순으로 콜백에 전달합니다. 콜백을 공통 뱅크 0에 배치하거나 호출하는 동안 해당 PRG 뱅크를 유지하세요. ZX0 사용 시 zx0.h를 명시적으로 포함하세요. audio_vblank 레코드 순서는 delay, CH1, CH2, CH3, CH4입니다.

현재 C# 빌드와 이전 배포판을 29개 입력으로 비교했습니다. 허용된 입력 15개의 ROM 바이트가 일치했으며 나머지 입력 14개에 대한 거부 진단도 일치했습니다. 추가 ROM 두 개를 KUROSAKI에서 각각 4프레임 실행하여 엔티티 할당, 콜백, 슬롯 재사용과 부호 있는 연산, KQBody3D 배치, 오디오 큐 상수를 확인했습니다. 보고서에는 예상 RAM 18바이트와 실제 값이 기록되어 있습니다. 오디오 재생이나 모든 API를 검증한 것은 아니며 이전 회귀 시험 결과는 BINARY_BUILD.json에서 구분합니다.

[소스 예제와 실행 검증 기록](https://bartaro.github.io/kitaq-docs/ko/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:ko:end -->


**[KITAQFC · HTML 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html)**

공개 라이브러리 이름은 KITAQGB의 방식을 따릅니다. `kitaqfc_` 접두사 없이 기능을 나타내는 짧은 이름을 사용합니다.

## 기본 기능

- `fc.h` - 공통 헤더를 한 번에 포함하는 진입점.
- `core.h` - 기본 자료형.
- `intrinsics.h` - 컴파일러가 인식하는 내장 연산의 진입점.
- `runtime.h` / `runtime.c` - NMI, OAM 작업용 복사본, VRAM 큐의 런타임 지원.
- `system.h` / `system.c` - KITAQGB 방식의 프레임 처리와 대기 함수.
- `debug.h` / `debug.c` - RAM에 남기는 작은 트레이스·단언 기록.

## 그래픽

- `vram.h` / `vram.c` - KITAQGB 방식으로 VRAM 갱신을 큐에 넣는 함수.
- `sprite.h` / `sprite.c` - KITAQGB 방식의 스프라이트 할당과 메타스프라이트.
- `ppu.h` / `ppu.c`
- `ppu_direct.h`
- `vram_queue.h`
- `palette.h` / `palette.c`
- `scroll.h` / `scroll.c`
- `tilemap.h` / `tilemap.c`
- `nametable_asset.h` / `nametable_asset.c`
- `attribute.h` / `attribute.c`
- `metasprite.h` / `metasprite.c`
- `oam.h`
- `oam_fair.h` / `oam_fair_impl.h` - 우선순위를 유지하면서 OAM 후보 64개의 순서를 순환시킵니다. 자세한 내용은 [oam_fair.md](oam_fair.md)를 참고하세요.

## 게임 구조

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## 오디오와 주변기기

- `audio.h` / `audio.c`
- `audio_vblank.h` / `audio_vblank.c` — NMI 음악 드라이버입니다. 7레코드 BGM 큐, 일시정지·재개, 별도의 7레코드 SFX 버퍼, 단발성 노이즈 감쇠 엔벌로프를 제공합니다. [HTML 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html#module-audio_vblank)
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` - KITAQGB의 버튼 상태 API에 맞추는 호환 래퍼.
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## 매퍼와 FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## 수치 계산

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` — 사각형 바디 적분, 중력, AABB 접촉, 표면 반응, 선택적으로 사용할 수 있는 Q5.3 자료형을 제공합니다. `fixed.c`와 함께 컴파일하세요.
- `physics3d.h` / `physics3d.c` — 회전하지 않는 3D 상자, 질량을 고려한 반발, 충돌 강도 값을 제공합니다. `fixed.c` 및 `physics2d.c`와 함께 컴파일하세요.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

장면 전환, 갱신, 그리기는 등록한 처리 함수를 동기적으로 호출합니다. 콜백 순서와 재진입 제한은 각 API 항목을 참고하세요.

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [fc-library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [fc-library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [fc-library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Français | [fc-library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Español | [fc-library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Deutsch | [fc-library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->

<!-- rust-native-20261004:start -->
## 네이티브 컴파일러로 라이브러리 사용

lib/는 콘솔 ROM에 컴파일되는 C 헤더와 소스입니다. PC용 컴파일러와 보조 도구는 Rust로 구현되었으며 게임 소스, 대상 콘솔 라이브러리 및 API는 C로 사용합니다. 제공된 SOURCE_MANIFEST.json과 LIBRARY_MERGE.json은 선택한 라이브러리 소스를 기록합니다.

Windows, Linux, macOS ARM 및 macOS Intel에서 네이티브 빌드와 실행 검증이 성공했습니다. KITAQGB는 환경별 테스트 48개와 보조 도구 검사 395개, KITAQFC는 55개와 401개를 통과했습니다. Rust 1.85도 검증했습니다. PUBLIC_DISTRIBUTION.json은 배치한 바이너리의 해시와 검증 출처를 기록합니다. 공개 GitHub Actions는 이 소스를 독립적으로 빌드하고 검증합니다.

저장된 참조 출력으로 ROM 바이트, 진단 및 보조 도구 형식을 검사합니다. 이전 C# 에뮬레이터 검증은 원래 소스 지문에 해당하는 과거 기록입니다. 모든 Rust API, 실제 하드웨어 또는 FDS BIOS를 통한 완전한 게임 시작을 입증하지는 않습니다. 원본 PNG 변환 스크립트는 없어 사양으로 재구현했으며 원본과의 바이트 일치는 확인할 수 없습니다.

[Rust 네이티브 컴파일러와 보조 도구](../tools/README.ko.md)

<!-- rust-native-20261004:end -->
