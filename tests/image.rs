use kitaqfc::{
    asm::{AddressMode, Operand},
    assembler::image,
    cartridge::{Board, Mapper},
    expr::{Arg, Expr},
    tags as t,
};
fn program() -> Vec<Expr> {
    vec![
        Expr::new(t::PRG_BANK, vec![0.into(), 1.into()]),
        Expr::new(t::FUNCTION, vec!["main".into()]),
        Expr::asm("RTS", Operand::implicit()),
        Expr::new(
            t::READONLY_DATA,
            vec![
                "pointer".into(),
                Arg::Bytes(vec![0, 0]),
                Arg::Exprs(vec![Expr::new(
                    t::WORD,
                    vec![
                        0.into(),
                        Arg::Operand(Operand::symbol("main", AddressMode::Absolute)),
                    ],
                )]),
            ],
        ),
    ]
}
#[test]
fn near_rom_pointer_relocates_to_cpu_window_and_vectors_use_reserved_tail() {
    let output = image::assemble(&program(), &image::Options::default()).unwrap();
    assert_eq!(&output.rom[..8], &[0x4e, 0x45, 0x53, 0x1a, 2, 1, 0, 0]);
    assert_eq!(&output.prg[0x4000..0x4003], &[0x60, 0, 0xc0]);
    assert_eq!(&output.prg[0x7fc0..0x7fc3], &[0x4c, 0, 0xc0]);
    assert_eq!(&output.prg[0x7ffc..0x7ffe], &[0xc0, 0xff]);
}
#[test]
fn axrom_common_copies_and_surom_replication_are_identical() {
    let mut nodes = program();
    nodes.extend([
        Expr::new(t::PRG_BANK, vec![2.into(), 1.into()]),
        Expr::new(t::FUNCTION, vec!["bank2".into()]),
        Expr::asm("RTS", Operand::implicit()),
    ]);
    let mut options = image::Options::default();
    options.cartridge.mapper = Mapper::Axrom;
    let output = image::assemble(&nodes, &options).unwrap();
    assert_eq!(output.prg.len(), 0x10000);
    assert_eq!(&output.prg[0x4000..0x8000], &output.prg[0xc000..]);
    options.cartridge.mapper = Mapper::Mmc1;
    options.cartridge.board = Board::Surom512;
    let output = image::assemble(&nodes, &options).unwrap();
    assert_eq!(output.prg.len(), 0x80000);
    assert!(output.chr.is_empty());
    assert_eq!(output.rom[6], 0x12);
    assert_eq!(output.rom[8], 1);
    assert_eq!(
        &output.prg[15 * 0x4000..16 * 0x4000],
        &output.prg[31 * 0x4000..32 * 0x4000]
    );
    assert_eq!(&output.prg[0x7fc0..0x8000], &output.prg[0x7ffc0..]);
}
