# FCHUA Notices

Original editor code is licensed under MIT. The standalone CLI depends on Rust
crates for PNG, JSON and atomic file IO. Optional headless Python bindings use
PyO3 (MIT OR Apache-2.0). The CLI and default Python wheel do not include plita
or a GUI runtime.
The generated DEPENDENCIES.json and licenses/ directory identify those crates
and retain their license texts in the standalone package.

The optional GUI uses plita (MIT), Roboto, Roboto Mono and Noto Sans JP (SIL Open Font
License), and Lucide (ISC, with MIT notices for Feather-derived icons).
The public source includes the five required plita foundation crates under
vendor/plita, without other applications or emulator frontends. See
vendor/plita/LICENSE and vendor/plita/crates/plita_ui/assets/lucide for the
unmodified notices. Additional menu/tool icons originate from Lucide 0.468.0.
Font files and their unmodified OFL texts are under assets/fonts.
Noto Sans JP provides Japanese glyphs alongside the primary Roboto font.
Its pinned upstream source, original license and SHA256 hashes are recorded in
assets/fonts/NotoSansJP-Regular-PROVENANCE.json. The font is unmodified.

GUI_DEPENDENCIES.json and gui-licenses identify the GUI's Windows registry
dependency closure, including build-time code generators. The GUI ZIP contains
these notices, the plita and icon notices, and the fonts. It does not bundle
Microsoft's proprietary Windows fonts. Historical copyright-holder strings
and API/environment identifiers are preserved; the product name is plita.

Imported artwork is not relicensed by FCHUA. Keep the original artwork's license.
