# 네이티브 보조 도구

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md)

모든 보조 실행 파일은 Rust로 구현되었습니다. 실행 시 .NET, Python, Pillow가 필요하지 않습니다. `cargo build --locked --release`로 모든 실행 파일을 빌드합니다. Windows에서는 명령 이름에 `.exe`를 붙입니다.

```text
kitaqfc-zx0 input.bin output.zx0
kitaqfc-zx0 output.zx0 restored.bin --decompress
kitaqfc-zx0 input.bin asset.h --header=level_data
kitaqfc-zx0 input.bin output.kqa --format=auto
kitaqfc-asset-pack assets/example_manifest.json
kitaqfc-png-index-build assets/example_png_manifest.json
kitaqfc-asset-pipeline assets/example_pipeline_manifest.json
kitaqfc-rights-name-guard --root . --denylist deny_terms.local.txt
```

ZX0는 1~65535바이트를 받아 C# 인코더의 출력을 유지합니다. `raw`, 개수/값 `rle`, 9바이트 `KQA1` 자동 컨테이너를 지원합니다. 크기가 같으면 raw, RLE, ZX0 순서로 선택합니다. `--decompress`는 출력 크기를 제한한 순방향 ZX0 v2 스트림을 받습니다. 역방향 스트림과 v1은 지원하지 않습니다. 형식 설계자는 Einar Saukas이며 KITAQ 구현의 라이선스는 MIT입니다.

기존 JSON 매니페스트로 8 KiB CHR, C 배열/헤더와 JSON 보고서를 만듭니다. 메타스프라이트는 data, frames, JSON 입력을 지원합니다. 인덱스 PNG에서 CHR, 네임테이블, 속성, 팔레트와 프레임 JSON을 생성합니다. 배경은 256×240, 시트는 8×8 타일이며 타일 및 16×16 배경 구역 안에서 팔레트 그룹이 같아야 합니다. 색 인덱스는 0~15, 깊이는 1/2/4/8이며 Adam7을 지원합니다. 누락된 PNG 원본은 로컬 사양과 예제 매니페스트에서 복원했으므로 원본 바이트 비교는 불가능합니다. RGB는 NES 근사 팔레트로 매핑합니다. 파이프라인은 두 단계를 내부 실행합니다. 명칭 검사는 외부 목록을 읽고 단어를 노출하지 않고 파일/행만 보고하며 생성 디렉터리는 제외합니다.
