use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub(super) struct Runtime {
    pub axrom_mirror_shadow: i32,
    pub current_bank: i32,
    pub fds_file_header: i32,
    pub fds_load_list: i32,
    pub fds_resident_bank: i32,
    pub intrinsic_tmp0: i32,
    pub intrinsic_tmp1: i32,
    pub intrinsic_tmp2: i32,
    pub mapper_fault: i32,
    pub mapper_temp2: i32,
    pub mapper_temp: i32,
    pub mmc1_control_shadow: i32,
    pub mmc1_inner_shadow: i32,
    pub mmc1_outer_shadow: i32,
    pub nmi_counter: i32,
    pub ppu_ctrl_shadow: i32,
    pub ppu_mask_shadow: i32,
    pub return_hi: i32,
    pub return_lo: i32,
    pub rng_hi: i32,
    pub rng_lo: i32,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub sret_ptr_hi: i32,
    pub sret_ptr_lo: i32,
    pub vramq_buffer: i32,
    pub vramq_len: i32,
    pub vramq_overflow: i32,
    pub vramq_ready: i32,
}
impl Runtime {
    pub fn allocate(zero_page: bool, fds: bool) -> (Self, BTreeMap<String, i32>) {
        Self::allocate_in(zero_page, fds, &mut super::storage::Storage::default())
    }
    pub fn allocate_in(
        zero_page: bool,
        fds: bool,
        storage: &mut super::storage::Storage,
    ) -> (Self, BTreeMap<String, i32>) {
        let mut globals = BTreeMap::new();
        let mut reserve = |name: &str, size: i32, fast: bool| {
            storage.next_zp = storage.skip_fixed(storage.next_zp, size);
            let address = if fast && zero_page && storage.next_zp + size <= 0xc0 {
                let a = storage.next_zp;
                storage.next_zp += size;
                a
            } else {
                storage.next_global = storage.skip_fixed(storage.next_global, size);
                let a = storage.next_global;
                storage.next_global += size;
                a
            };
            globals.insert(name.into(), address);
            address
        };
        let runtime = Self {
            current_bank: reserve("__kq_prg_bank_current", 1, true),
            return_lo: reserve("__kq_thunk_ret_lo", 1, true),
            return_hi: reserve("__kq_thunk_ret_hi", 1, true),
            mapper_temp: reserve("__kq_mapper_tmp", 1, true),
            mapper_temp2: reserve("__kq_mapper_tmp2", 1, true),
            mmc1_outer_shadow: reserve("__kq_mmc1_outer_shadow", 1, true),
            mmc1_inner_shadow: reserve("__kq_mmc1_inner_shadow", 1, true),
            mmc1_control_shadow: reserve("__kq_mmc1_control_shadow", 1, true),
            mapper_fault: reserve("__kq_mapper_fault", 1, true),
            ppu_ctrl_shadow: reserve("__kq_ppuctrl_shadow", 1, true),
            ppu_mask_shadow: reserve("__kq_ppumask_shadow", 1, true),
            scroll_x: reserve("__kq_scroll_x", 1, true),
            scroll_y: reserve("__kq_scroll_y", 1, true),
            nmi_counter: reserve("__kq_nmi_counter", 1, true),
            intrinsic_tmp0: reserve("__kq_intr_tmp0", 1, true),
            intrinsic_tmp1: reserve("__kq_intr_tmp1", 1, true),
            intrinsic_tmp2: reserve("__kq_intr_tmp2", 1, true),
            vramq_len: reserve("__kq_vramq_len", 1, true),
            vramq_ready: reserve("__kq_vramq_ready", 1, true),
            vramq_overflow: reserve("__kq_vramq_overflow", 1, true),
            vramq_buffer: reserve("__kq_vramq_buf", 128, false),
            axrom_mirror_shadow: reserve("__kq_axrom_mirror_shadow", 1, true),
            fds_load_list: reserve("__kq_fds_load_list", 2, false),
            fds_file_header: reserve(
                "__kq_fds_file_header",
                super::FDS_FILE_HEADER_SIZE + if fds { 32 } else { 0 },
                false,
            ),
            fds_resident_bank: reserve("__kq_fds_resident_bank", 1, true),
            rng_lo: reserve("__kq_rng_lo", 1, true),
            rng_hi: reserve("__kq_rng_hi", 1, true),
            sret_ptr_lo: reserve("__kq_sret_ptr_lo", 1, true),
            sret_ptr_hi: reserve("__kq_sret_ptr_hi", 1, true),
        };
        (runtime, globals)
    }
}
