use super::control::Context;
use super::*;
use crate::{
    asm::Region,
    ctype::{Aggregate, AggregateLayout, CType, Field, Kind, Simple},
};
#[derive(Clone)]
pub(super) struct Slot {
    pub ty: CType,
    pub address: i32,
    pub size: i32,
    pub readonly: bool,
}
pub(super) struct Storage {
    pub slots: BTreeMap<String, Slot>,
    pub signatures: BTreeMap<String, (CType, Vec<Field>)>,
    pub aggregates: BTreeMap<String, Aggregate>,
    pub readonly: Vec<super::readonly::Definition>,
    pub pending: Vec<(Region, CType, String)>,
    pub fixed: Vec<(i32, i32)>,
    pub next_global: i32,
    pub next_zp: i32,
    pub next_oam: i32,
    pub free_temps: Vec<i32>,
    pub temp_pool: Vec<i32>,
    pub next_local: Option<i32>,
}
impl Default for Storage {
    fn default() -> Self {
        Self {
            slots: BTreeMap::new(),
            signatures: BTreeMap::new(),
            aggregates: BTreeMap::new(),
            readonly: Vec::new(),
            pending: Vec::new(),
            fixed: Vec::new(),
            next_global: 0x300,
            next_zp: 0x10,
            next_oam: 0x200,
            free_temps: (0xd0..0x100).rev().collect(),
            temp_pool: (0xd0..0x100).collect(),
            next_local: None,
        }
    }
}
impl Storage {
    pub fn skip_fixed(&self, mut address: i32, size: i32) -> i32 {
        while let Some((_, end)) = self
            .fixed
            .iter()
            .find(|(start, end)| address < *end && *start < address + size)
        {
            address = *end;
        }
        address
    }
}
impl Generator {
    pub(super) fn storage_size(&self, ty: &CType) -> i32 {
        match &ty.kind {
            Kind::Simple(Simple::UInt8 | Simple::Int8) | Kind::Enum(_) => 1,
            Kind::Simple(Simple::UInt16 | Simple::Int16) | Kind::Pointer(_) => 2,
            Kind::Array(sub, n) => self.storage_size(sub).wrapping_mul((*n).max(0)),
            Kind::ArrayExpression(sub, expr) => self
                .evaluate_constant(expr)
                .map(|n| self.storage_size(sub).wrapping_mul(n.max(0)))
                .unwrap_or(0),
            Kind::Struct(n) | Kind::Union(n) => self
                .storage
                .aggregates
                .get(n)
                .map(|a| a.total_size.max(0))
                .unwrap_or(0),
            _ => 0,
        }
    }
    fn natural_align(&self, ty: &CType) -> i32 {
        if ty.forced_align > 0 {
            return ty.forced_align;
        }
        match &ty.kind {
            Kind::Struct(n) | Kind::Union(n) => self
                .storage
                .aggregates
                .get(n)
                .map(|a| a.alignment)
                .unwrap_or(1),
            Kind::Array(t, _) | Kind::ArrayExpression(t, _) => self.natural_align(t),
            _ => {
                if self.storage_size(ty) == 2 {
                    2
                } else {
                    1
                }
            }
        }
    }
    pub(super) fn collect_storage(&mut self, node: &Expr, bank: i32, fixed: bool, order: i32) {
        match node.tag() {
            Some(t::READONLY_DATA) => self.collect_readonly(node, bank, fixed, order),
            Some(t::CONSTANT) => {
                if let (Some(n), Some(e)) = (node.text(2), node.child(3)) {
                    if self.constants.contains_key(n)
                        || self.storage.pending.iter().any(|(_, _, name)| name == n)
                    {
                        self.errors.push(format!("duplicate top-level symbol: {n}"));
                        return;
                    }
                    if let Some(v) = self.evaluate_constant(e) {
                        self.constants.insert(n.into(), v & 65535);
                    } else {
                        self.errors
                            .push(format!("top-level constant must be integer-foldable: {n}"));
                    }
                }
            }
            Some(t::VARIABLE) => {
                if let (Some(Arg::Region(r)), Some(ty), Some(n)) =
                    (node.args.get(1), node.ty(2), node.text(3))
                {
                    self.storage.pending.push((*r, ty.clone(), n.into()));
                } else {
                    self.errors
                        .push(format!("invalid global variable: {}", node.show()));
                }
            }
            Some(t::STRUCT | t::UNION) => {
                if let (Some(n), Some(Arg::Fields(raw))) = (node.text(1), node.args.get(2)) {
                    if self.storage.aggregates.contains_key(n) {
                        return;
                    }
                    let packed = node.int(3).unwrap_or(0) != 0;
                    let union = node.is(t::UNION);
                    let mut offset = 0;
                    let mut alignment = 1;
                    let mut fields = Vec::new();
                    for f in raw {
                        let a = if packed && !union {
                            1
                        } else {
                            self.natural_align(&f.ty)
                        };
                        let size = self.storage_size(&f.ty);
                        alignment = alignment.max(a);
                        if !union && !packed && a > 1 {
                            offset = (offset + a - 1) & !(a - 1);
                        }
                        fields.push(Field {
                            offset: if union { 0 } else { offset },
                            ..f.clone()
                        });
                        offset = if union {
                            offset.max(size)
                        } else {
                            offset + size
                        };
                    }
                    alignment = node.int(4).filter(|n| *n > 0).unwrap_or(alignment);
                    if !packed && alignment > 1 {
                        offset = (offset + alignment - 1) & !(alignment - 1);
                    }
                    self.storage.aggregates.insert(
                        n.into(),
                        Aggregate {
                            layout: if union {
                                AggregateLayout::Union
                            } else {
                                AggregateLayout::Struct
                            },
                            total_size: offset,
                            alignment,
                            is_packed: packed,
                            fields,
                        },
                    );
                }
            }
            _ => self.errors.push(format!(
                "readonly initialization is not yet connected: {}",
                node.show()
            )),
        }
    }
    pub(super) fn prepare_storage(&mut self) {
        self.record_allocation(
            "__kq_call_arg_area",
            "abi_call_args",
            CALL_ARG_BASE,
            16,
            true,
        );
        self.record_allocation(
            "__kq_temp_pool",
            "compiler_temp_pool",
            self.storage.temp_pool[0],
            self.storage.temp_pool.len() as i32,
            true,
        );
        for (region, ty, _) in &self.storage.pending {
            if let Region::Fixed(start) = region {
                let size = self.storage_size(ty);
                let mut start = *start;
                let end = 0x2000.min(start.saturating_add(size));
                if start < 0 || size <= 0 {
                    continue;
                }
                while start < end {
                    let physical = start & 0x7ff;
                    let len = (end - start).min(0x800 - physical);
                    self.storage.fixed.push((physical, physical + len));
                    start += len;
                }
            }
        }
        let (runtime, globals) = Runtime::allocate_in(
            self.options.zero_page,
            self.profile.has_fds(),
            &mut self.storage,
        );
        self.runtime = runtime;
        self.globals = globals;
        for (name, address) in self.globals.clone() {
            let size = match name.as_str() {
                "__kq_vramq_buf" => 128,
                "__kq_fds_load_list" => 2,
                "__kq_fds_file_header" => {
                    FDS_FILE_HEADER_SIZE + if self.profile.has_fds() { 32 } else { 0 }
                }
                _ => 1,
            };
            self.storage.slots.insert(
                name.clone(),
                Slot {
                    ty: CType::simple(Simple::UInt8),
                    address,
                    size,
                    readonly: false,
                },
            );
            self.record_allocation(
                &name,
                if size > 1 {
                    "runtime_buffer"
                } else {
                    "runtime"
                },
                address,
                size,
                false,
            );
            if address < 256 {
                let reason = match name.as_str() {
                    "__kq_prg_bank_current" => "mapper current bank hot state",
                    "__kq_thunk_ret_lo" | "__kq_thunk_ret_hi" => "farcall return hot state",
                    "__kq_mapper_tmp" => "mapper switch temporary",
                    "__kq_mapper_tmp2" => "mapper serial temporary",
                    "__kq_mmc1_outer_shadow" => "MMC1 outer PRG bank shadow",
                    "__kq_mmc1_inner_shadow" => "MMC1 inner PRG bank shadow",
                    "__kq_mmc1_control_shadow" => "MMC1 control register shadow",
                    "__kq_mapper_fault" => "mapper fail-safe status",
                    "__kq_ppuctrl_shadow" => "PPUCTRL shadow hot state",
                    "__kq_ppumask_shadow" => "PPUMASK shadow hot state",
                    "__kq_scroll_x" | "__kq_scroll_y" => "scroll hot state",
                    "__kq_nmi_counter" => "NMI counter hot state",
                    "__kq_intr_tmp0" | "__kq_intr_tmp1" | "__kq_intr_tmp2" => {
                        "intrinsic temporary virtual register"
                    }
                    "__kq_vramq_len" => "VRAM queue length hot state",
                    "__kq_vramq_ready" => "VRAM queue ready flag hot state",
                    "__kq_vramq_overflow" => "VRAM queue overflow flag hot state",
                    "__kq_axrom_mirror_shadow" => "AxROM mirror shadow hot state",
                    "__kq_fds_resident_bank" => "FDS overlay resident bank hot state",
                    "__kq_rng_lo" | "__kq_rng_hi" => "RNG state hot byte",
                    "__kq_sret_ptr_lo" | "__kq_sret_ptr_hi" => "struct return destination pointer",
                    _ => "runtime hot state",
                };
                self.analysis.zp_allocations.push(ZpAllocation {
                    name,
                    kind: "runtime".into(),
                    address,
                    size,
                    reason: reason.into(),
                });
            }
        }
        for (r, ty, n) in self.storage.pending.clone() {
            let size = self.storage_size(&ty);
            if let Some(old) = self.storage.slots.get(&n) {
                if matches!(r,Region::Fixed(a)if a == old.address && old.size == size) {
                    continue;
                }
                self.errors.push(format!("duplicate top-level symbol: {n}"));
                continue;
            }
            if self.constants.contains_key(&n) {
                self.errors.push(format!("duplicate top-level symbol: {n}"));
                continue;
            }
            if size <= 0 {
                self.errors.push(format!("unknown storage size: {n}"));
                continue;
            }
            let address = match r {
                Region::Fixed(a) => a,
                Region::HighMem => self.allocate_local(&n, size, "global_highmem"),
                Region::Oam => {
                    self.storage.next_oam = self.storage.skip_fixed(self.storage.next_oam, size);
                    let a = self.storage.next_oam;
                    self.storage.next_oam += size;
                    if self.storage.next_oam > 0x300 {
                        self.errors.push(format!("OAM RAM exhausted: {n}"));
                    }
                    a
                }
                _ => self.allocate_global(size),
            };
            self.globals.insert(n.clone(), address);
            if r != Region::HighMem {
                self.record_allocation(
                    &n,
                    match r {
                        Region::Fixed(_) => "global_fixed",
                        Region::Oam => "global_oam",
                        _ => "global",
                    },
                    address,
                    size,
                    false,
                );
            }
            self.storage.slots.insert(
                n,
                Slot {
                    ty,
                    address,
                    size,
                    readonly: false,
                },
            );
        }
        for (name, (_, fields)) in &self.storage.signatures {
            let sizes = fields
                .iter()
                .map(|f| self.storage_size(&f.ty))
                .collect::<Vec<_>>();
            if sizes.iter().any(|n| *n <= 0) || sizes.iter().sum::<i32>() > 16 {
                self.errors
                    .push(format!("invalid parameter area for {name}"));
            }
        }
        let overlaps = |a: (i32, i32), b: (i32, i32)| a.0 < b.0 + b.1 && b.0 < a.0 + a.1;
        for (kind, window) in [
            ("local", self.options.local_ram),
            ("temporary", self.options.temp_ram),
        ] {
            if let Some(window) = window {
                let slots_overlap = self
                    .storage
                    .slots
                    .values()
                    .any(|s| !s.readonly && overlaps(window, (s.address, s.size)));
                let args_overlap = overlaps(window, (CALL_ARG_BASE, 16));
                let temp_overlap = kind == "local"
                    && overlaps(window, self.options.temp_ram.unwrap_or((0xd0, 0x30)));
                if slots_overlap || args_overlap || temp_overlap {
                    self.errors.push(format!(
                        "configured {kind} RAM window overlaps reserved RAM"
                    ));
                }
            }
        }
    }
    pub(super) fn allocate_global(&mut self, size: i32) -> i32 {
        self.storage.next_global = self.storage.skip_fixed(self.storage.next_global, size);
        let address = self.storage.next_global;
        self.storage.next_global += size;
        if self.storage.next_global > 0x800 {
            self.errors.push("internal RAM exhausted".into());
        }
        address
    }
    fn allocate_local(&mut self, name: &str, size: i32, kind: &str) -> i32 {
        self.storage.next_zp = self.storage.skip_fixed(self.storage.next_zp, size);
        let address = if self.options.zero_page && self.storage.next_zp + size <= 0xc0 {
            let address = self.storage.next_zp;
            self.storage.next_zp += size;
            self.record_zp_storage(name, address, size);
            address
        } else if let (Some(address), Some((start, length))) =
            (self.storage.next_local, self.options.local_ram)
        {
            if address < start || address + size > start + length {
                self.errors.push("configured local RAM exhausted".into());
                start
            } else {
                self.storage.next_local = Some(address + size);
                address
            }
        } else {
            self.allocate_global(size)
        };
        self.record_allocation(name, kind, address, size, false);
        address
    }
    pub(super) fn declare_local(
        &mut self,
        ctx: &mut Context,
        name: &str,
        ty: &CType,
    ) -> Option<Slot> {
        if ctx.locals.contains_key(name) || self.constants.contains_key(name) {
            self.errors
                .push(format!("duplicate local {}::{name}", ctx.name));
            return None;
        }
        let size = self.storage_size(ty);
        if size <= 0 {
            self.errors
                .push(format!("unknown local storage size: {name}"));
            return None;
        }
        let slot = Slot {
            ty: ty.clone(),
            size,
            address: self.allocate_local(name, size, "local"),
            readonly: false,
        };
        ctx.locals.insert(name.into(), slot.clone());
        if self.options.static_frame {
            self.analysis.static_frames.push(StaticSlot {
                function: ctx.name.clone(),
                name: name.into(),
                address: slot.address,
                size: slot.size,
            });
        }
        Some(slot)
    }
    pub(super) fn slot(&self, ctx: &Context, name: &str) -> Option<Slot> {
        ctx.locals
            .get(name)
            .or_else(|| self.storage.slots.get(name))
            .cloned()
    }
    pub(super) fn acquire(&mut self, count: usize) -> Vec<i32> {
        if count > self.storage.free_temps.len() {
            self.errors
                .push(format!("temporary scratch exhausted (needed {count})"));
            return vec![self.storage.temp_pool[0]; count];
        }
        (0..count)
            .map(|_| self.storage.free_temps.pop().unwrap())
            .collect()
    }
    pub(super) fn release(&mut self, temps: &[i32]) {
        for t in temps.iter().rev() {
            if !self.storage.free_temps.contains(t) {
                self.storage.free_temps.push(*t);
            }
        }
    }
}
