//! KUROSAKI build metadata generated from native compiler and assembler records.
use crate::{
    assembler::image,
    codegen::{Analysis, Options},
    json::Value,
};
use std::collections::BTreeMap;

fn obj<const N: usize>(items: [(&str, Value); N]) -> Value {
    Value::Object(items.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn text(s: impl Into<String>) -> Value {
    Value::String(s.into())
}
fn num(n: impl Into<i64>) -> Value {
    Value::Number(n.into())
}
fn boolean(b: bool) -> Value {
    Value::Bool(b)
}
fn array(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Array(items.into_iter().collect())
}
struct Identifiers {
    redact: bool,
    names: BTreeMap<String, String>,
}
impl Identifiers {
    fn name(&mut self, name: &str) -> Value {
        if !self.redact {
            return text(name);
        }
        let next = self.names.len() + 1;
        text(
            self.names
                .entry(name.into())
                .or_insert_with(|| format!("function_{next:04}"))
                .clone(),
        )
    }
}

pub fn build(
    analysis: &Analysis,
    image: &image::Output,
    options: &Options,
    output: &str,
    register_flag: bool,
    profile_feedback: bool,
) -> Result<Value, String> {
    let profile = options.image.cartridge.profile()?;
    let surom = profile.is_surom();
    let board = if surom { "surom512" } else { "generic" };
    let mut identifiers = Identifiers {
        redact: surom,
        names: BTreeMap::new(),
    };
    let allocations = array(analysis.ram_allocations.iter().enumerate().map(|(i, a)| {
        obj([
            ("id", text(format!("allocation_{:04}", i + 1))),
            ("kind", text(&a.kind)),
            ("address", num(a.address)),
            ("size", num(a.size)),
            ("end_exclusive", num(a.address + a.size)),
            ("region", text(&a.region)),
            ("conservative", boolean(a.conservative)),
        ])
    }));
    // Alias identifiers in the same first-encounter order as the metadata schema.
    let accesses = array(analysis.ram_accesses.iter().enumerate().map(|(i, a)| {
        obj([
            ("id", text(format!("access_{:04}", i + 1))),
            ("function", identifiers.name(&a.function)),
            ("bank", num(a.bank)),
            ("operation", text(&a.operation)),
            ("address", num(a.address)),
            ("span", num(a.span)),
            ("end_exclusive", num(a.address + a.span)),
            ("region", text(&a.region)),
            ("dynamic_target", boolean(a.dynamic_target)),
        ])
    }));
    let local_window = options.local_ram.map_or(Value::Null, |(start, length)| {
        obj([
            ("start", num(start)),
            ("end_exclusive", num(start + length)),
        ])
    });
    let (temp_base, temp_length) = options.temp_ram.unwrap_or((0xd0, 0x30));
    let functions = array(analysis.functions.iter().map(|f| {
        obj([
            ("name", identifiers.name(&f.name)),
            ("bank", num(f.bank)),
            ("fixed_bank", boolean(f.fixed)),
            ("prototype", boolean(f.prototype)),
            ("inline", boolean(f.is_inline)),
            ("fastcall", boolean(f.fast_call)),
            ("return_size", num(f.return_size)),
            ("param_sizes", array(f.param_sizes.iter().map(|&n| num(n)))),
        ])
    }));
    let calls = array(analysis.calls.iter().map(|c| {
        obj([
            ("caller", identifiers.name(&c.caller)),
            ("caller_bank", num(c.caller_bank)),
            ("callee", identifiers.name(&c.callee)),
            ("callee_bank", num(c.callee_bank)),
            ("kind", text(&c.kind)),
            ("via_thunk", boolean(c.via_thunk)),
            ("via_farcall", boolean(c.via_farcall)),
            ("count", num(c.count)),
        ])
    }));
    let actions = array(analysis.nes_actions.iter().map(|a| {
        obj([
            ("name", text(a.info.name)),
            ("category", text(a.info.category)),
            ("operation", text(a.info.operation)),
            ("kurosaki_kind", text(a.info.kurosaki_kind)),
            ("caller", identifiers.name(&a.caller)),
            ("caller_bank", num(a.caller_bank)),
            ("source", text(if surom { "redacted" } else { &a.source })),
            ("timing", text(a.info.timing)),
            ("direct_ppu", boolean(a.info.direct_ppu)),
            ("queue_ppu", boolean(a.info.queue_ppu)),
            ("oam_shadow", boolean(a.info.oam_shadow)),
            ("oam_dma", boolean(a.info.oam_dma)),
            ("requires_fds", boolean(a.info.requires_fds)),
            ("mapper_requirement", text(a.info.mapper)),
            ("note", text(&a.note)),
        ])
    }));
    let mut extents = image.functions.values().collect::<Vec<_>>();
    extents.sort_by_key(|f| (f.bank, f.cpu_address));
    let sizes = array(extents.into_iter().map(|f| {
        obj([
            ("name", identifiers.name(&f.name)),
            ("bank", num(f.bank)),
            ("cpu_address", num(f.cpu_address)),
            ("size", num(f.size)),
        ])
    }));
    let mut result = obj([
        ("schema", text("kitaqfc-nes-build-metadata-v2")),
        ("producer", text("KITAQFC")),
        ("target", text("nes")),
        (
            "output",
            text(if surom { "output_0001.nes" } else { output }),
        ),
        ("mapper", text(profile.name)),
        ("board", text(board)),
        (
            "cartridge",
            obj([
                ("mapper_number", num(profile.mapper_number as i32)),
                ("mapper", text(profile.name)),
                ("board", text(board)),
                ("header", text("ines1")),
                ("prg_rom_bytes", num(image.prg.len() as i64)),
                ("chr_rom_bytes", num(image.chr.len() as i64)),
                (
                    "chr_ram_bytes",
                    num(if options.image.chr_ram {
                        0x2000
                    } else {
                        profile.chr_ram_bytes as i64
                    }),
                ),
                ("prg_ram_bytes", num(profile.prg_ram_bytes as i64)),
                ("battery", boolean(options.image.cartridge.battery())),
                ("prg_mode", num(3)),
                ("chr_mode", num(0)),
            ]),
        ),
        (
            "optimizations",
            obj([
                ("whole_program_zp", boolean(options.zero_page)),
                ("fastcall_v2", boolean(register_flag)),
                ("static_frame", boolean(options.static_frame)),
                ("small_inline", boolean(options.auto_inline)),
                ("loop_lowering", boolean(options.loop_lowering)),
                ("lto_lite", boolean(options.library_lto)),
                ("profile_feedback", boolean(profile_feedback)),
                ("mapper_aware_placement", boolean(options.mapper_placement)),
            ]),
        ),
        (
            "ram_access_contract",
            obj([
                ("schema", text("kitaqfc-ram-access-v1")),
                ("allocator_complete", boolean(true)),
                (
                    "implicit_stack",
                    obj([
                        ("start", num(256)),
                        ("end_exclusive", num(512)),
                        ("conservative", boolean(true)),
                    ]),
                ),
                ("configured_local_window", local_window),
                (
                    "compiler_temp_window",
                    obj([
                        ("start", num(temp_base)),
                        ("end_exclusive", num(temp_base + temp_length)),
                        ("custom", boolean(options.temp_ram.is_some())),
                    ]),
                ),
                ("allocations", allocations),
                ("function_accesses", accesses),
            ]),
        ),
        (
            "fds",
            obj([
                ("enabled", boolean(profile.has_fds())),
                (
                    "layout",
                    text(if options.image.fds_ram {
                        "fds32"
                    } else {
                        "legacy"
                    }),
                ),
                ("auto_overlay", boolean(options.image.fds.auto_overlay)),
                ("overlay_farcall", boolean(options.fds_overlay_farcall)),
            ]),
        ),
        ("functions", functions),
        ("calls", calls),
        (
            "optimization_events",
            obj([
                ("zp_allocations", num(analysis.zp_allocations.len() as i64)),
                (
                    "static_frame_slots",
                    num(analysis.static_frames.len() as i64),
                ),
                (
                    "ram_allocations",
                    num(analysis.ram_allocations.len() as i64),
                ),
                ("ram_accesses", num(analysis.ram_accesses.len() as i64)),
                (
                    "inline_decisions",
                    num(analysis.inline_decisions.len() as i64),
                ),
                ("loop_lowerings", num(analysis.loop_lowerings.len() as i64)),
                (
                    "lto_removed_functions",
                    num(analysis.removed_functions.len() as i64),
                ),
                (
                    "bank_placements",
                    num(analysis.bank_placements.len() as i64),
                ),
            ]),
        ),
        ("nes_actions", actions),
        ("function_sizes", sizes),
    ]);
    if surom {
        let mut mappings = (1..=30)
            .map(|bank| {
                let physical = profile.logical_to_physical(bank).unwrap();
                obj([
                    ("logical_bank", num(bank)),
                    ("physical_bank", num(physical)),
                    ("outer_256k", num((physical >> 4) & 1)),
                    ("inner_16k", num(physical & 15)),
                    ("is_common_replica", boolean(false)),
                ])
            })
            .collect::<Vec<_>>();
        for physical in [15, 31] {
            mappings.push(obj([
                ("logical_bank", num(0)),
                ("physical_bank", num(physical)),
                ("outer_256k", num((physical >> 4) & 1)),
                ("inner_16k", num(15)),
                ("is_common_replica", boolean(true)),
            ]));
        }
        let common = &image.prg[15 * 0x4000..16 * 0x4000];
        let equal = common == &image.prg[31 * 0x4000..32 * 0x4000];
        if let Value::Object(fields) = &mut result {
            fields.insert(
                "bank_layout".into(),
                obj([
                    ("kind", text("surom_outer256_fixed_top16")),
                    ("logical_common_bank", num(0)),
                    ("logical_switchable_min", num(1)),
                    ("logical_switchable_max", num(30)),
                    ("common_physical_banks", array([num(15), num(31)])),
                    ("common_replica_equal", boolean(equal)),
                    ("common_replica_sha256", text(crate::hash::sha256(common))),
                    ("mappings", Value::Array(mappings)),
                ]),
            );
            fields.insert(
                "reset".into(),
                obj([
                    ("tail_reserve_bytes", num(64)),
                    ("replication", text("every_physical_32k_high_bank")),
                ]),
            );
            fields.insert(
                "redaction".into(),
                obj([
                    ("source_paths_redacted", boolean(true)),
                    ("project_labels_redacted", boolean(true)),
                    ("identifiers_redacted", boolean(true)),
                ]),
            );
        }
    }
    Ok(result)
}
