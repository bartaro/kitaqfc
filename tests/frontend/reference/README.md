# Frozen compiler compatibility headers

These headers are the transitive include closure of the 29 historical frontend
fixtures, copied byte for byte from the pre-library-update commit recorded in
`manifest.json`. The parser/lowerer goldens include every declaration and source
coordinate, so their inputs must remain fixed along with the expected JSON.
No expected JSON, rejected-input diagnostics, or equality assertions were changed.

`frontend_compat.rs` and the frozen metadata comparisons in `metadata.rs` use
this include tree. Tests of the current library
(`library.rs`, `source_roms.rs`, and the 13-case library compile/runtime runner)
continue using the repository's current `lib/`. These files are fixtures, not
headers to ship with a new application. License: the repository MIT license.
