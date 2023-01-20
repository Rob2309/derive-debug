#![allow(unused, clippy::enum_variant_names, clippy::disallowed_names)]

use derive_debug::Dbg;

#[derive(Dbg, Default)]
#[dbg(alias = "TestStructAlias")]
struct TestStruct {
    plain_field: u32,
    #[dbg(skip)]
    skipped_field: u32,
    #[dbg(alias = "alias_field_alias")]
    alias_field: u32,
    #[dbg(placeholder = "...")]
    placeholder_field: u32,
    #[dbg(alias = "alias_placeholder_field_alias", placeholder = "abc")]
    alias_placeholder_field: u32,
    #[dbg(fmt = "{:#06X}")]
    fmt_field: u32,
}

#[test]
fn test_struct() {
    let foo = TestStruct {
        fmt_field: 0xAB,
        ..TestStruct::default()
    };

    assert_eq!(
        format!("\n{:#?}\n", foo),
        r#"
TestStructAlias {
    plain_field: 0,
    alias_field_alias: 0,
    placeholder_field: ...,
    alias_placeholder_field_alias: abc,
    fmt_field: 0x00AB,
}
"#
    );
}

#[derive(Dbg)]
struct TestTuple(u32, #[dbg(formatter = "fmt_not_zero")] u32);

fn fmt_not_zero(v: &u32) -> &'static str {
    if *v == 0 {
        "0"
    } else {
        "not 0"
    }
}

#[test]
fn test_tuple() {
    let foo = TestTuple(42, 17);

    assert_eq!(format!("{:?}", foo), "TestTuple(42, not 0)");
}

#[derive(Dbg)]
enum TestEnum {
    UnitVariant,
    TupleVariant(u32, u32),
    StructVariant {
        a: u32,
        b: u32,
    },

    #[dbg(skip)]
    SkippedUnitVariant,
    #[dbg(skip)]
    SkippedTupleVariant(u32, u32),
    #[dbg(skip)]
    SkippedStructVariant {
        a: u32,
        b: u32,
    },

    #[dbg(alias = "AliasVariant")]
    AliasedUnitVariant,
    #[dbg(alias = "AliasVariant")]
    AliasedTupleVariant(u32, u32),
    #[dbg(alias = "AliasVariant")]
    AliasedStructVariant {
        a: u32,
        b: u32,
    },
}

#[test]
fn test_unit_variant() {
    let foo = TestEnum::UnitVariant;
    assert_eq!(format!("{:?}", foo), "UnitVariant");
}

#[test]
fn test_tuple_variant() {
    let foo = TestEnum::TupleVariant(0, 1);
    assert_eq!(format!("{:?}", foo), "TupleVariant(0, 1)");
}

#[test]
fn test_struct_variant() {
    let foo = TestEnum::StructVariant { a: 0, b: 1 };
    assert_eq!(format!("{:?}", foo), "StructVariant { a: 0, b: 1 }");
}

#[test]
fn test_skipped_unit_variant() {
    let foo = TestEnum::SkippedUnitVariant;
    assert_eq!(format!("{:?}", foo), "SkippedUnitVariant");
}

#[test]
fn test_skipped_tuple_variant() {
    let foo = TestEnum::SkippedTupleVariant(0, 1);
    assert_eq!(format!("{:?}", foo), "SkippedTupleVariant");
}

#[test]
fn test_skipped_struct_variant() {
    let foo = TestEnum::SkippedStructVariant { a: 0, b: 1 };
    assert_eq!(format!("{:?}", foo), "SkippedStructVariant");
}

#[test]
fn test_aliased_unit_variant() {
    let foo = TestEnum::AliasedUnitVariant;
    assert_eq!(format!("{:?}", foo), "AliasVariant");
}

#[test]
fn test_aliased_tuple_variant() {
    let foo = TestEnum::AliasedTupleVariant(0, 1);
    assert_eq!(format!("{:?}", foo), "AliasVariant(0, 1)");
}

#[test]
fn test_aliased_struct_variant() {
    let foo = TestEnum::AliasedStructVariant { a: 0, b: 1 };
    assert_eq!(format!("{:?}", foo), "AliasVariant { a: 0, b: 1 }");
}
