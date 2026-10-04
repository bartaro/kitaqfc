# 原生辅助工具

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md)

所有辅助程序均以 Rust 实现，运行时不需要 .NET、Python 或 Pillow。使用 `cargo build --locked --release` 构建全部程序。Windows 命令名须加 `.exe`。

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

ZX0 接受 1～65535 字节并保留 C# 编码器的输出。支持 `raw`、计数/值 `rle` 和九字节 `KQA1` 自动容器；大小相同依次选择 raw、RLE、ZX0。`--decompress` 对纯正向 ZX0 v2 流进行有界解码，不支持反向流及 v1。格式由 Einar Saukas 设计，KITAQ 实现采用 MIT 许可证。

读取既有 JSON 清单生成 8 KiB CHR、C 数组/头文件及 JSON 报告。元精灵支持 data、frames 和 JSON 输入。索引 PNG 转换生成 CHR、名称表、属性、调色板及元精灵 JSON。背景为 256×240，图集以 8×8 图块组成，单图块及背景 16×16 区块内调色板组须一致。色索引为 0～15，支持 1/2/4/8 位深度及 Adam7。缺失的 PNG 原脚本根据本地规格与示例清单重建，无法与原件进行字节比较。RGB 映射至定义的 NES 近似调色板。流水线在内部执行两阶段。名称检查读取外部词表，只报告文件和行号，不打印保护词，排除生成目录。
