# FC/NES 표준 라이브러리 구성

[English](README.md) | [日本語](README.ja.md) | **한국어**

[KITAQFC 라이브러리 한국어 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html)

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
- `physics2d.h` / `physics2d.c` - 바이트 크기의 Q5.3 자료형과 조정 상수입니다. 정수 픽셀 위치, 1/8픽셀 단위의 소수부, 부호 없는 속력, 방향, 저항을 따로 저장합니다. 시간 경과에 따른 위치·속도 계산은 게임 코드에서 수행하며, 헤더가 물리 시뮬레이션 갱신 함수를 제공하지는 않습니다. 필드를 분리하면 자주 실행하는 이동 루프에서 구조체나 포인터를 ABI를 통해 전달하는 부담을 줄일 수 있습니다.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

KITAQGB 호환 함수는 FC/NES 하드웨어에서 지원할 수 있는 범위에서 짧은 기능 이름을 유지합니다. 콜백 형식의 장면·엔티티 함수는 현재 상태를 저장하기만 하며, 사용자의 함수 포인터를 간접 호출하지 않습니다.
