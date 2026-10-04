use kitaqfc::{
    asm::{AddressMode, Operand},
    assembler::layout,
    cartridge::{Mapper, Options},
    expr::{Arg, Expr},
    tags as t,
};
fn node(tag: &str, args: Vec<Arg>) -> Expr {
    Expr::new(tag, args)
}
#[test]
fn far_symbolic_branch_is_inverted_and_expanded_before_following_labels() {
    let nodes = vec![
        node(t::FUNCTION, vec!["main".into()]),
        Expr::asm("BNE", Operand::symbol("far", AddressMode::Relative)),
        node(
            t::READONLY_DATA,
            vec!["padding".into(), Arg::Bytes(vec![0; 200])],
        ),
        node(t::LABEL, vec!["far".into()]),
        Expr::asm("RTS", Operand::implicit()),
    ];
    let output = layout::layout(&nodes, &Options::default().profile().unwrap(), false).unwrap();
    assert_eq!(output.symbols["far"], 0x8000 + 205);
    assert_eq!(output.units[0].nodes[1].asm_parts().unwrap().0, "BEQ");
    assert_eq!(output.units[0].nodes[2].asm_parts().unwrap().0, "JMP");
}
#[test]
fn fixed_banks_reserve_space_before_movable_units_and_align_follows_its_object() {
    let nodes = vec![
        node(t::PRG_BANK, vec![1.into(), 0.into()]),
        node(t::READONLY_DATA, vec!["a".into(), Arg::Bytes(vec![0; 100])]),
        node(t::PRG_BANK, vec![1.into(), 1.into()]),
        node(
            t::READONLY_DATA,
            vec!["fixed".into(), Arg::Bytes(vec![0; 16300])],
        ),
        node(t::PRG_BANK, vec![2.into(), 0.into()]),
        node(t::ALIGN, vec![256.into()]),
        node(
            t::READONLY_DATA,
            vec!["aligned".into(), Arg::Bytes(vec![1; 10])],
        ),
    ];
    let options = Options {
        mapper: Mapper::Mmc3,
        ..Default::default()
    };
    let output = layout::layout(&nodes, &options.profile().unwrap(), false).unwrap();
    assert_eq!(output.units[0].bank, 2);
    assert_eq!(output.units[1].bank, 1);
    assert_eq!(output.symbols["aligned"], 0x8100);
    assert!(output.units[2].nodes[0].is(t::ALIGN));
    assert!(layout::layout(&nodes, &Options::default().profile().unwrap(), false).is_err());
}
