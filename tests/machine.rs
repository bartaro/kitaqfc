use kitaqfc::{
    asm::{AddressMode as Mode, Modifier, Operand},
    machine,
};
use std::collections::BTreeMap;

#[test]
fn zero_page_shrinking_uses_full_address_before_byte_extraction() {
    let symbols = BTreeMap::from([("low".into(), 0xfe), ("far".into(), 0x8123)]);
    let low = Operand::symbol("low", Mode::Absolute);
    assert_eq!(
        machine::encode("LDA", &low, 0x8000, &symbols).unwrap(),
        [0xa5, 0xfe]
    );
    let far = Operand::symbol("far", Mode::Absolute).with_modifier(Modifier::LowByte);
    assert_eq!(
        machine::encode("LDA", &far, 0x8000, &symbols).unwrap(),
        [0xad, 0x23, 0]
    );
    let unresolved = Operand::symbol("future", Mode::Absolute);
    assert_eq!(machine::estimate_size("LDA", &unresolved, &symbols), 3);
    assert!(machine::encode("LDA", &unresolved, 0x8000, &symbols).is_err());
}

#[test]
fn branch_displacement_limits_are_relative_to_end_of_instruction() {
    let symbols = BTreeMap::new();
    for delta in [-128, 0, 127] {
        let operand = Operand::integer(0x8002 + delta, Mode::Relative);
        assert_eq!(
            machine::encode("BNE", &operand, 0x8000, &symbols).unwrap(),
            [0xd0, delta as u8]
        );
    }
    for delta in [-129, 128] {
        assert!(
            machine::encode(
                "BNE",
                &Operand::integer(0x8002 + delta, Mode::Relative),
                0x8000,
                &symbols
            )
            .is_err()
        );
    }
}

#[test]
fn indexed_indirect_accumulator_and_immediate_ranges_match_6502() {
    let symbols = BTreeMap::new();
    assert_eq!(
        machine::encode("LDX", &Operand::integer(7, Mode::AbsoluteY), 0, &symbols).unwrap(),
        [0xb6, 7]
    );
    assert_eq!(
        machine::encode("LDA", &Operand::integer(0xd0, Mode::IndirectY), 0, &symbols).unwrap(),
        [0xb1, 0xd0]
    );
    assert_eq!(
        machine::encode("JMP", &Operand::integer(0x80, Mode::Indirect), 0, &symbols).unwrap(),
        [0x6c, 0x80, 0]
    );
    assert_eq!(
        machine::encode("ASL", &Operand::implicit(), 0, &symbols).unwrap(),
        [0x0a]
    );
    assert!(machine::encode("LDA", &Operand::integer(256, Mode::Immediate), 0, &symbols).is_err());
    assert!(
        machine::encode(
            "STX",
            &Operand::integer(0x8000, Mode::AbsoluteY),
            0,
            &symbols
        )
        .is_err()
    );
}
