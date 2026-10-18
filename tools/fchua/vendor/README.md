# Vendored plita Foundation

This directory contains only the five plita foundation crates required by the
FCHUA desktop. They were copied from the development workspace on 2026-10-18.
The shared workspace dependency declarations are in FCHUA's Cargo.toml; exact
registry versions and checksums are locked in Cargo.lock.

No emulator frontends, private applications, ROMs, or unrelated example projects
are included. Rust and Python identifiers, environment variables and historical
copyright-holder strings are unchanged. Product-name prose, default titles and
crate descriptions use the official lowercase name `plita`.

Roboto, Roboto Mono and Noto Sans JP are distributed in ../assets/fonts with
their OFL texts. Noto Sans JP supplies Japanese glyphs; its upstream provenance
and file hashes are retained beside the font.
The plita MIT notice is in plita/LICENSE; Lucide's ISC/MIT notices are in
plita/crates/plita_ui/assets/lucide/LICENSE. FCHUA's executable-relative font
lookup is implemented in ../src/gui.rs, outside the vendored foundation.

GUI_DEPENDENCIES.json and gui-licenses cover the Windows registry dependencies.
For packages missing a standalone license file in their crate archive,
../license-overrides contains the upstream license text. The exact official
URLs and downloaded SHA256 values are recorded in ../license-overrides/SOURCES.json
and can be retrieved again with ../scripts/fetch_gui_notices.ps1. Khronos XML
copyright notices are also collected from the registry XML files.
