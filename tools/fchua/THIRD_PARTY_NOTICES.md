# FCHUA Notices

Original editor code is licensed under MIT. The standalone CLI depends on Rust
crates for PNG, JSON and atomic file IO. Optional headless Python bindings use
PyO3 (MIT OR Apache-2.0). Neither includes PLITA or a GUI runtime.
The generated DEPENDENCIES.json and licenses/ directory identify those crates
and retain their license texts in the standalone package.

The local GUI additionally uses PLITA, Roboto (SIL Open Font License) and Lucide
(ISC, with MIT notices for Feather-derived icons). Their license texts are in
the PLITA workspace. Additional menu/tool icons are vendored from the official
Lucide 0.468.0 release into PLITA's existing icon renderer.

Imported artwork is not relicensed by FCHUA. Keep the original artwork's license.
