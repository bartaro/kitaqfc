# FC/NES 표준 라이브러리 구성

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | **한국어** | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

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
