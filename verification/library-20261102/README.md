# Library verification, 2026-11-02

Sixteen module groups compiled; thirteen original test ROMs passed RAM, CHR,
attribute and raster-pixel assertions in KUROSAKI. These are emulator checks.
The compiler binary is unchanged; run-inputs.json binds the compiler, emulator
and tested library inputs. The public README retains public-only documentation.

To reproduce the runtime cases, install Python and Pillow, then run:

```
python verification/library-20261102/runtime_libraries.py --compiler /path/to/kitaqfc --zx0 /path/to/kitaqfc-zx0 --kurosaki /path/to/kurosaki
```

Use the `.exe` tool names on Windows. Generated ROMs and snapshots are written
to a temporary directory unless `--out` is supplied. No BIOS or third-party ROM
is needed. Pinball and GB communication are excluded.

All four native compiler CI jobs also compile these 13 cases against the current
`lib/`, using `--compile-only` instead of `--kurosaki`. This mode requires neither
KUROSAKI nor Pillow; it checks compilation and ROM creation, not runtime behavior.
The historical parser/lowerer and metadata JSON comparisons use frozen headers in
`tests/frontend/reference/`, so new API declarations cannot change their inputs.
Existing full-ROM library tests continue to use current headers and modules.
The existing FC Q5.3 movement types and native physics helper names are preserved.
Including `fc.h` also retains the native `__manhattan` intrinsic without linking
the RPG runtime. Game ROM and optimization-report parity tests cover compatibility.
