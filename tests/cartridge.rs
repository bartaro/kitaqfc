use kitaqfc::cartridge::{Board, Mapper, Options};
#[test]
fn surom_skips_common_banks_and_reserves_reset_space_by_physical_parity() {
    let mut o = Options::default();
    o.set_mapper("surom512").unwrap();
    let p = o.profile().unwrap();
    assert_eq!(p.logical_to_physical(15).unwrap(), 14);
    assert_eq!(p.logical_to_physical(16).unwrap(), 16);
    assert_eq!(p.logical_to_physical(30).unwrap(), 30);
    assert!(p.logical_to_physical(31).is_err());
    assert!(!p.needs_reset_tail(15));
    assert!(!p.needs_reset_tail(16));
    assert!(p.needs_reset_tail(17));
    assert!(o.battery());
    assert!(p.requires_chr_ram);
    assert_eq!(p.exact_prg_bytes, 0x80000);
    o.battery = Some(false);
    assert!(!o.battery());
}
#[test]
fn invalid_board_combination_is_rejected_without_overriding_mapper() {
    let o = Options {
        board: Board::Surom512,
        mapper: Mapper::Mmc3,
        ..Default::default()
    };
    assert!(o.profile().unwrap_err().contains("KQFC2601"));
    let (mapper, board) = Mapper::parse("mapper26").unwrap();
    assert_eq!(mapper, Mapper::Vrc6);
    assert!(!board);
    assert_eq!(
        Options {
            mapper,
            ..Default::default()
        }
        .profile()
        .unwrap()
        .mapper_number,
        24
    );
}
