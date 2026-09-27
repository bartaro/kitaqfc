# ZX0 호환 리소스 압축

<!-- readme-language-links:start -->
[English](README.md#english) | [日本語](README.md#%E6%97%A5%E6%9C%AC%E8%AA%9E) | **한국어** | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

[API와 예제](https://bartaro.github.io/kitaq-docs/ko/fc-library.html#module-zx0)

PC 압축기와 FC 압축 해제기는 ZX0 v2 정방향 스트림을 지원하는 KITAQ의 독자 구현입니다. KITAQ 구현은 MIT 라이선스로 배포하며, 저작권은 Copyright (c) 2026 DAISUKE OBA입니다.

ZX0 형식과 원래 압축 알고리즘의 설계자는 [Einar Saukas](https://github.com/einar-saukas/ZX0)입니다. 형식에 대한 이 표기는 KITAQ 구현의 저작권·라이선스와 구분됩니다. [LICENSE](../../LICENSE)와 [LICENSE.ja](../../LICENSE.ja)도 참고하세요.

저장소 최상위 디렉터리에서 PC 도구를 빌드합니다.

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

PC 도구는 .NET Framework 4.x를 사용합니다. 압축기는 탐색 횟수를 제한한 해시 체인을 사용하며, 가장 작은 출력 크기를 보장하지 않습니다. 역방향 스트림, 외부 접두 사전, ZX0 v1은 이 인터페이스의 지원 대상이 아닙니다. 원본 리소스의 권리는 각 제작자에게 있습니다.

입력과 인코딩된 출력은 각각 65535바이트 이하여야 합니다. `--format=auto`는 raw, RLE, ZX0의 페이로드 크기를 비교해 가장 작은 것에 9바이트 KQA1 헤더를 붙입니다. 헤더도 출력 한도에 포함합니다. KQA1에는 `asset_decompress`를 사용하세요. 실제 CPU 뱅크 창과 RAM 용량에도 맞게 리소스를 나누어야 합니다. 빈 raw C 헤더는 저장 공간용 바이트 하나를 가지며 논리적 `_SIZE`는 0입니다.

대상 프로그램에서 `zx0.h`를 포함하고 `lib/zx0.c`를 함께 컴파일하세요. `zx0_decompress`에는 출력 위치, 출력 용량, 압축 입력 위치, 압축 크기를 전달합니다. 반환된 바이트 수와 `zx0_error`를 모두 확인하세요. 오류가 나면 출력이 일부만 기록될 수 있으므로 실패한 결과를 표시하거나 사용하지 마세요. 입력과 출력 버퍼는 겹치거나 현재 매핑된 CPU 뱅크 창의 경계를 넘어서는 안 됩니다. 공유 작업 영역을 사용하므로 인터럽트에서 재진입하면 안 됩니다.

`zx0_decompress_vram`에는 압축 해제된 리소스 전체를 담을 수 있는 RAM 작업 영역이 필요합니다. 렌더링을 끈 상태에서 그 영역에 압축을 풀고 결과를 CHR RAM 또는 네임테이블 메모리로 전송합니다. PPUCTRL은 유지합니다. 렌더링을 다시 켜기 전에 스크롤 위치를 설정하세요. 팔레트 전송이나 CHR ROM 쓰기에는 사용할 수 없습니다.
