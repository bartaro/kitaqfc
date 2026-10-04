//! Native, portable KITAQFC compiler.
#![forbid(unsafe_code)]
pub mod asm;
pub mod assembler;
pub mod assembly_json;
pub mod cache;
pub mod cartridge;
pub mod cli;
pub mod codegen;
pub mod ctype;
pub mod diagnostics;
pub mod expr;
pub mod fds;
pub mod hash;
pub mod io;
pub mod ir_json;
pub mod json;
pub mod lowerer;
pub mod machine;
pub mod metadata;
pub mod nes_actions;
pub mod optimizer;
pub mod palettes;
pub mod parser;
pub mod pipeline;
mod preprocessor;
pub mod reports;
pub mod rom_header;
pub mod tags;
pub mod token;
pub mod tokenizer;
pub mod workflow;

pub mod debug_tools;
mod diagnostic_help;
pub mod recipe;
pub mod vibe_tools;

pub mod drivers;
pub mod minimizer;
pub mod observation;
pub mod repro;

pub mod build_helpers;
pub mod zx0;
pub mod assets;
pub mod rights;
