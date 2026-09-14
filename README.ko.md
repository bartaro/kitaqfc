# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **한국어**

**[컴파일러 설명서](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html)** · **[라이브러리 설명서](https://bartaro.github.io/kitaq-docs/ko/fc-library.html)**

NES/Famicom/FDS용 홈브루 소프트웨어를 만드는 C 컴파일러와 지원 라이브러리입니다. KITAQGB와 NORCAL을 바탕으로 개발되었습니다.

공개 미리 보기 버전으로, API와 동작이 변경될 수 있습니다.

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
