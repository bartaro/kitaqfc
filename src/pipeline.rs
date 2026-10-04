//! Recompile bank-dependent calls and literal placement after assembler spill decisions.
use crate::{assembler::image, codegen, expr::Expr, lowerer, optimizer, parser};
use std::{collections::BTreeMap, path::PathBuf};

pub struct Output {
    pub assembly: Vec<Expr>,
    pub analysis: codegen::Analysis,
    pub image: image::Output,
    pub relayout_passes: usize,
    pub optimizer_report: optimizer::Report,
}
pub fn assemble_stable(
    initial_tree: &Expr,
    files: &[PathBuf],
    includes: &[PathBuf],
    initial_options: &codegen::Options,
    opt_level: i32,
) -> Result<Output, String> {
    let mut session = crate::observation::Session::default();
    session.hosted = true;
    assemble_stable_observed(
        initial_tree,
        files,
        includes,
        initial_options,
        opt_level,
        &mut session,
    )
}
pub fn assemble_stable_observed(
    initial_tree: &Expr,
    files: &[PathBuf],
    includes: &[PathBuf],
    initial_options: &codegen::Options,
    opt_level: i32,
    session: &mut crate::observation::Session,
) -> Result<Output, String> {
    let mut options = initial_options.clone();
    let mut tree = initial_tree.clone();
    for pass in 0..=4 {
        let (original, analysis) = codegen::compile_with_analysis(&tree, &options)?;
        if session.debug {
            session.debug_file(
                "assembly_code.txt",
                &crate::observation::assembly(&original),
            )?;
        }
        let optimized = optimizer::optimize_with_report(&original, opt_level);
        let assembly = optimized.lines;
        if session.traced("asm") {
            session.trace_file("asm.txt", &crate::observation::assembly(&assembly))?;
        }
        let image = image::assemble(&assembly, &options.image)?;
        let mut functions = BTreeMap::new();
        let mut readonly = BTreeMap::new();
        let mut conflicts = Vec::new();
        for f in &analysis.functions {
            if f.is_inline || f.prototype {
                continue;
            }
            let Some(actual) = image.functions.get(&f.name) else {
                continue;
            };
            if actual.bank == f.bank {
                continue;
            }
            if f.fixed {
                conflicts.push(format!(
                    "fixed-bank function {} requested b{} actual b{}",
                    f.name, f.bank, actual.bank
                ));
            } else {
                functions.insert(f.name.clone(), actual.bank);
            }
        }
        for d in &analysis.readonly_data {
            let Some(actual) = image.readonly.get(&d.name) else {
                continue;
            };
            if actual.bank == d.bank {
                continue;
            }
            if d.fixed {
                conflicts.push(format!(
                    "fixed-bank readonly data {} requested b{} actual b{}",
                    d.name, d.bank, actual.bank
                ));
            } else {
                readonly.insert(d.name.clone(), actual.bank);
            }
        }
        if !conflicts.is_empty() {
            return Err(conflicts.join(", "));
        }
        if functions.is_empty() && readonly.is_empty() {
            return Ok(Output {
                assembly,
                analysis,
                image,
                relayout_passes: pass,
                optimizer_report: optimized.report,
            });
        }
        let notes = functions
            .iter()
            .chain(&readonly)
            .map(|(n, b)| format!("{n}->b{b}"))
            .collect::<Vec<_>>()
            .join(", ");
        if pass == 4 {
            return Err(format!(
                "symbol bank layout did not stabilize after 5 passes: {notes}"
            ));
        }
        if !session.hosted {
            eprintln!("[bank-layout] recompiling with actual symbol banks: {notes}");
        }
        options.function_banks.extend(functions);
        options.readonly_banks.extend(readonly);
        let parsed =
            parser::parse_files_with_banks(files, includes, false, &options.function_banks);
        session.capture_stage(&parsed.diagnostics);
        if session.errors() != 0 {
            return Err("bank relayout parsing failed".into());
        }
        let lowered = lowerer::lower(
            &parsed.tree,
            &lowerer::Options {
                const_scalar_in_rom: options.const_scalar_in_rom,
                ..lowerer::Options::default()
            },
        );
        session.capture_stage(&lowered.diagnostics);
        if session.errors() != 0 {
            return Err("bank relayout lowering failed".into());
        }
        tree = lowered.tree;
    }
    unreachable!()
}
