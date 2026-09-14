# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **한국어**

**[컴파일러 설명서](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html)** · **[라이브러리 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html)**

NES/Famicom/FDS용 홈브루 소프트웨어를 만드는 C 컴파일러와 지원 라이브러리입니다. KITAQGB와 NORCAL을 바탕으로 개발되었습니다.

공개 미리 보기 버전으로, API와 동작이 변경될 수 있습니다.

## 개발 방향

KITAQFC는 NES·패미컴 하드웨어의 특성을 고려하고, 작고 재현 가능한 단위로 확인하며 개발하는 것을 지향합니다. ROM을 빌드한 뒤 KUROSAKI에서 실행하고 SARAKURA로 실행 결과를 분석하며, 수정할 때마다 다시 검증합니다. 실제로 측정한 결과와 아직 테스트하지 않은 동작은 구분해서 기록합니다.

<!-- development-prompt:ko:start -->
## 게임 개발 프롬프트

요구 사항을 작성한 뒤 프롬프트 전체를 AI에 전달하세요. 구현, 에뮬레이터 테스트, SARAKURA 분석, 수정 후 재검증까지 다룹니다.

[HTML 설명서에서 활용 예 읽기](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html#loop-prompts)

<details>
<summary>프롬프트 전체 보기</summary>

### KITAQFC·KUROSAKI·SARAKURA를 활용한 게임 개발

요구 사항을 작성한 뒤 이 문서 전체를 AI에 전달하세요. 명령은 `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura`, `kitaq-docs` 저장소와 `game-gb` 또는 `game-fc` 프로젝트가 같은 상위 폴더에 있는 구성을 가정합니다. 해당 상위 폴더에서 실행하고, 실제 환경에 맞게 경로를 조정하세요.

#### 요구 사항

- 게임 이름: &lt;작성&gt;
- 장르와 핵심 플레이 방식: &lt;작성&gt;
- 조작 방법과 성공·실패 조건: &lt;작성&gt;
- 필수 화면·스테이지·적·아이템: &lt;작성&gt;
- 그래픽 스타일·배경 음악·효과음: &lt;작성, 제공 자료의 경로도 명시&gt;
- 저장·통신·주변기기 등 추가 요구 사항: &lt;작성 또는 없음&gt;
- 프로젝트 폴더: &lt;작성&gt;
- 재배포 조건: &lt;예: 자체 제작 코드와 소재를 MIT로 공개할 수 있는 상태&gt;

- 대상: &lt;NES/패미컴 카트리지 또는 FDS&gt;
- 매퍼·ROM 크기·미러링: &lt;지정하거나 요구 사항에 따라 선정&gt;
- 영상 방식과 성능 목표: &lt;예: NTSC, 초당 60회 게임 갱신&gt;

#### 수행할 작업

KITAQFC와 해당 라이브러리로 게임을 구현해 주세요. 실행과 디버깅에는 KUROSAKI를, 진단 정리와 수정 전후 비교에는 SARAKURA를 사용하세요.

합격 기준을 충족할 때까지 명세 구체화 → 작은 단위 구현 → 빌드 → 입력과 관찰 → 원인 조사 → 수정 → 동일 조건 재검증을 반복하세요. 계획 작성, 코드 제시 또는 컴파일 성공만으로 완료하지 마세요.

##### 환경과 합격 기준 확인

1. 작업 폴더의 지침, 도구별 README, HTML 설명서, 사용할 라이브러리의 헤더와 구현을 읽으세요. 실행 파일 경로와 버전 또는 SHA-256을 기록하고, 명령은 실제 `--help` 출력으로, API는 소스로 확인하세요.
2. 입력·화면·소리·진행·갱신 빈도를 판정할 수 있는 합격 기준을 정하세요. 예를 들어 START를 눌렀다 놓으면 시작하고, 충돌하면 잔기가 하나 줄며, 일시 정지 시 지정한 소리가 멈추고 해제 후 다시 재생되는지 확인합니다.
3. 중요한 모호함만 질문하고, 일반적인 되돌릴 수 있는 구현 판단은 자율적으로 진행하세요. 요구 사항이나 합격 기준을 임의로 완화하지 마세요.
4. 먼저 작은 제공 예제를 컴파일러·에뮬레이터·SARAKURA로 실행해 도구 간 연결을 확인하세요. 이것을 요청받은 게임의 완성으로 간주하지 마세요.

##### 작게 시작해 플레이 가능한 형태로 구현

- 작은 게임은 NROM부터 검토하고, 규모나 뱅크 전환이 필요하면 MMC3 등 적절한 매퍼를 선택하세요. 이름만으로 지원 여부를 판단하지 말고 `inspect-rom`, `mapper-info`, `audit-board`와 구현으로 필요한 보드 기능을 확인하세요.
- PRG/CHR 크기, CHR-ROM 또는 CHR-RAM, 미러링, 고정 뱅크, 인터럽트 벡터, 저장 RAM을 계획하세요. 최적화나 뱅크 변경 후에는 헤더와 실제 배치를 대조하세요. `--nes-local-ram`은 CPU 내부 RAM `$0000–$07FF`를 사용합니다. 제로 페이지·스택·OAM 버퍼·런타임 및 라이브러리 영역과 겹치지 않게 하세요.
- KITAQFC의 C 문법, FC 라이브러리와 `void main(void)`를 사용하세요. GB API의 호환성을 가정하지 마세요. 헤더에 선언만 있는 항목도 있으므로 구현 위치를 확인하고 필요한 `.c`를 포함하세요.
- PPU 레지스터, NMI, OAM DMA, 주사선별 스프라이트 제한, 스크롤, 미러링, 속성 테이블, APU/DMC를 고려하세요. 전송 큐의 여유 공간을 PPU VRAM 용량과 혼동하지 말고 NMI 한 번에 처리할 작업량을 제한하세요.
- 제공된 자체 제작 `ascii.c` 글꼴을 FC CHR 형식으로 변환하고 CHR·팔레트·네임 테이블·속성 데이터를 확인하세요. FDS는 디스크 접근·저장·BIOS 조건을 별도로 검증하고 카트리지와 같은 부팅 조건을 가정하지 마세요.

- 먼저 부팅·타이틀·조작 가능한 플레이어·성공 또는 실패·재시작을 연결한 뒤 내용을 늘리세요.
- 그래픽·음악·효과음의 편집 가능한 원본과 생성 절차를 보관하고, 빌드가 실제로 내보낸 데이터를 읽는지 확인하세요.
- 소스 주석은 영어, 진행 보고는 한국어로 작성하세요. SARAKURA의 표준 보고서는 영어로 유지하세요.

##### 빌드와 실행 결과 연결

`out/iter-001`처럼 반복별 출력 폴더를 나누세요. 명령, 종료 코드, 소스·소재·도구·ROM·메타데이터의 해시를 기록하세요. 빌드 실패 후 남아 있는 이전 ROM을 실행하지 마세요. 맵·소스 맵·디버그 정보는 ROM과 동일한 빌드에서 나온 것을 사용하세요.

다음은 입력 없는 NROM 확인 예입니다. `main.c`, 필요한 라이브러리 구현, CHR 파일을 준비하고 매퍼를 조정하세요. 요구하는 형식을 확인하지 않은 채 컴파일러 빌드 메타데이터를 KUROSAKI의 `--kitaqfc-debug`에 전달하지 마세요.

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


300프레임의 무입력 실행은 초기 확인에 불과합니다. 게임이 정상이라고 판단하기 전에 순서가 있는 플레이 시나리오를 추가하세요.

##### 순서가 있는 입력 재현

- `--pad1`과 `--pad2`는 NES 원시 비트 마스크입니다. A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128입니다. 라이브러리의 `BTN_*` 값과 혼동하지 마세요.
- `run --pad1`은 고정 입력입니다. 순서가 있는 조작에는 누르기·유지·놓기를 구분한 리플레이를 만드세요. `Replay`와 `ReplayFrame` 정의를 확인하세요. 공개 CLI의 `replay-record`는 사람의 플레이가 아니라 무입력을 기록합니다.
- ROM의 SHA-256, 리플레이 대상과 프레임 범위를 직접 대조하세요. `replay-run --verify`는 기대 최종 해시가 있을 때만 비교하므로 게임 동작이나 ROM 대응을 포괄적으로 검증하지 않습니다. 테스트를 통과시키려고 기대값을 관측값으로 무조건 덮어쓰지 마세요.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


시험할 ROM에 맞는 리플레이를 준비하세요. 공개 CLI의 `replay-run`에는 `--emit-diagnostics`가 없습니다. 없는 옵션을 사용하거나 CPU 트레이스를 진단 이벤트로 넘기지 마세요. 순서가 있는 입력을 SARAKURA로 분석하려면 공개 `kurosaki-core`의 `RunOptions.replay_frames`와 `diagnostic_events_from_trace_and_report`를 이용한 검증 하네스를 프로젝트 안에 만드세요. 같은 ROM·리플레이·프레임 조건으로 실행하고 그 실행의 트레이스와 진단 보고서에서 JSONL을 생성하세요. 트레이스 설정과 보존 범위를 확인하고 하네스의 화면·관측 결과를 CLI 리플레이와 대조하세요. 무입력 진단을 플레이 시나리오의 증거로 사용하지 마세요. 필요한 환경이 없으면 해당 검증을 미완료로 보고하세요.

##### 화면·소리·상태·성능 확인

- 누르기·유지·놓기를 구분한 입력 시나리오를 저장하세요. 부팅, 시작, 이동, 행동, 충돌, 스크롤, 스테이지 전환, 게임 오버, 재시작, 일시 정지와 필요한 저장·통신 등 명세의 모든 경로를 실행하세요.
- 필요한 프레임의 PNG, 입력 데이터, 실행 보고서, 진단 JSONL, WAV와 필요한 상태·메모리 관측을 보관하세요. 도달 프레임과 정지 이유를 확인하세요. 이미지를 실제로 열어 보고, 한 장의 스크린샷만으로 움직임이나 입력 반응을 검증했다고 하지 마세요. 카운터·좌표·상태 전환을 기대값과 비교하고 화면 끝·타일 및 속성 경계·스프라이트 밀집 장면도 확인하세요.
- 음악·효과음·동시 재생·끊김·일시 정지·재개를 확인하세요. WAV 생성만으로 올바른 소리를 증명할 수 없습니다. 들을 수 없는 환경에서는 실시한 파형·수치 검사와 아직 확인하지 못한 청감 품질을 구분하세요.
- 부하가 큰 장면의 대상 CPU 작업량·게임 갱신·전송량을 측정하고 FC에서는 NMI 작업도 포함하세요. 호스트에서의 에뮬레이터 처리 속도를 게임 갱신 빈도나 실기 속도와 동일시하지 마세요. `--allow-unimplemented`로 계속 실행된다고 해서 미구현 기능이 지원되는 것은 아닙니다.

##### 분석·수정·재검증

- 시험한 ROM의 빌드 메타데이터와 해당 실행의 진단 JSONL을 SARAKURA에 전달하세요. CPU 트레이스나 일반 실행 보고서로 대체하지 마세요. `--frames`는 분석 조건이며, SARAKURA는 ROM을 실행하거나 소스를 자동 수정하지 않습니다.
- `report.html`, `ai_diagnostics.json`, `repair_prompt.md`, `retest_plan.json`을 읽고 재현 절차·화면·소리·소스와 대조하세요. 추정한 소스 위치와 원인을 확인된 사실과 구분하고, 정상 대기 루프와 멈춤 버그를 구분하세요. 경고를 개별 판단하고 미지원 이벤트와 분석 한계를 기록하세요. 필터로 경고를 숨기거나 테스트를 줄여 합격시키지 마세요.
- 문제를 최소 재현 예제로 줄이고 원인을 수정한 뒤 다시 빌드하세요. 컴파일러나 에뮬레이터가 원인이면 게임 코드와 분리해 결함을 확인하고 도구 수정에 회귀 검증을 추가하세요.
- 입력·난수 시드·기종 및 영상 방식·매퍼·관측 프레임·진단 설정을 맞춰 재검증하세요. ROM마다 해당 메타데이터를 사용하고 코드나 RAM 배치가 바뀐 뒤 저장 상태를 무조건 재사용하지 마세요.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


진단 차이는 조작·그래픽·소리의 합격 판정과 함께 사용하세요. 같은 실패가 반복되면 근거와 가설을 다시 검토하고 무작정 수정을 이어 가지 마세요.

##### 완료 조건과 결과물

납품할 소스와 설정으로 만든 최종 ROM에서 모든 필수 시나리오를 다시 실행하세요. 무적 상태·자동 시험 입력·다른 매퍼만으로 최종 빌드의 일반 플레이를 검증했다고 하지 마세요. 요구 사항과 시험의 대응표, 남은 경고의 이유, 미확인·미지원 항목을 명시하세요. 실기 시험을 하지 않았다면 ‘실기 미확인’으로 표시하세요.

소스, 도구·라이브러리 식별 정보, 편집 가능한 소재, 재현 가능한 빌드·검증 스크립트, ROM, 최종 검증 증거, 설치·조작·알려진 제한을 설명한 README를 제공하세요. 필요한 리플레이와 검증 하네스도 포함하세요. 공개·외부 전송은 명시적으로 허용된 범위에서만 수행하세요. 검증 후 불필요한 중간 빌드와 임시 트레이스는 지우되 소스·소재·최종 결과물·필요한 회귀 증거는 보관하세요.

환경이나 권한 때문에 필수 검사를 할 수 없다면 정확한 재현 절차와 필요한 조치를 보고하고, 완료로 처리하지 마세요.

</details>
<!-- development-prompt:ko:end -->

## 저장소 구성

컴파일러 소스, 프로젝트 파일, 빌드 설정은 이름이 같은 하위 폴더 `kitaqfc/`에 있습니다. 빌드된 Release 실행 파일과 런타임 설정은 저장소 최상위 폴더에 있습니다. `lib/`에는 C 라이브러리, `examples/`에는 입문 예제와 직접 제작한 글꼴이 들어 있습니다.

```text
kitaqfc/                  # Repository root
├─ kitaqfc/               # Compiler build sources
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Prebuilt Release compiler
├─ kitaqfc.exe.config     # .NET Framework runtime configuration
├─ lib/                # C support libraries
├─ examples/           # Tutorial programs and original font
├─ scripts/build.ps1   # Rebuild the Release executable
├─ LICENSE
└─ LICENSE.ja
```

제공된 컴파일러를 실행하려면 Windows와 .NET Framework 4.8이 필요합니다. 실행 파일, 설정, 라이브러리, 라이선스 고지를 함께 받을 수 있도록 저장소 ZIP을 내려받으세요. 직접 빌드하려면 .NET Framework 4.8 Developer Pack과 Visual Studio Build Tools도 설치해야 합니다. 저장소 최상위 폴더에서 실행하세요.

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Release 빌드는 실행 파일과 설정을 최상위 폴더로 복사합니다. Debug 빌드는 `kitaqfc/bin/Debug`에 남으며, 배포용 Release 컴파일러를 덮어쓰지 않습니다. 빌드 캐시와 PDB 파일은 배포하지 않습니다. 입력 자료와 SHA-256은 [바이너리 빌드 기록](BINARY_BUILD.json)에 있습니다.

## 직접 빌드하고 실행하기

Windows에서 .NET Framework 4.8 Developer Pack과 Visual Studio Build Tools의 MSBuild를 사용합니다. Developer PowerShell을 열고 실행하세요.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## 설명서와 라이선스

- [한국어 컴파일러 설명서](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html) / [한국어 라이브러리 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html)
- [영어 설명서](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [일본어 설명서](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [오프라인 열람용 설명서 소스](https://github.com/bartaro/kitaq-docs)
- [라이선스](LICENSE) / [일본어 참고 번역](LICENSE.ja)

프로젝트 라이선스가 글꼴, 의존 라이브러리, 로고, 상표에 관한 제3자의 조건을 대신하지는 않습니다. 재배포할 때 동봉된 고지를 유지하세요.
