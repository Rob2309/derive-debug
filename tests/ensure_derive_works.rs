use derive_debug::Dbg;

#[derive(Dbg, Default)]
#[dbg(alias = "TestAAlias")]
struct TestA {
    normal_field: (),
    #[dbg(skip)]
    skipped_field: (),
    #[dbg(placeholder = "...")]
    placeholder_field: (),
}

#[test]
fn test_a_output() {
    let a = TestA::default();

    assert_eq!(
        format!("{:?}", a),
        "TestA { normal_field: (), placeholder_field: ... }"
    );
}

#[derive(Dbg)]
enum TestB {
    UnitVariant,
    TupleVariant(u32, bool),
    StructVariant {
        a: bool,
        b: u32,
    },

    #[dbg(skip)]
    SkippedUnitVariant,
    #[dbg(skip)]
    SkippedTupleVarian(u32, bool),
    #[dbg(skip)]
    SkippedStructVariant {
        a: bool,
        b: u32,
    },

    SkipTupleField(u32, #[dbg(skip)] bool, u32),
    SkipStructField {
        a: bool,
        #[dbg(skip)]
        b: bool,
    },

    #[dbg(alias = "NotAliasVariant")]
    AliasVariant(u32),
}
