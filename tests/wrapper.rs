use optfield::optfield;

#[derive(Debug, Clone, PartialEq)]
struct SWrapper<T> {
    item: T,
    number: i64,
}

impl<T> SWrapper<T> {
    fn new(item: T) -> Self {
        Self { item, number: 0 }
    }
}

// needed for merging
impl<T> From<SWrapper<T>> for Option<T> {
    fn from(s: SWrapper<T>) -> Self {
        Some(s.item)
    }
}

// needed for from
impl<T> From<T> for SWrapper<T> {
    fn from(value: T) -> Self {
        SWrapper::new(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TWrapper<T>(String, T);

impl<T> TWrapper<T> {
    fn new(t: T) -> Self {
        Self("test".to_string(), t)
    }
}

// needed for merging
impl<T> From<TWrapper<T>> for Option<T> {
    fn from(t: TWrapper<T>) -> Self {
        Some(t.1)
    }
}

// needed for from
impl<T> From<T> for TWrapper<T> {
    fn from(value: T) -> Self {
        TWrapper::new(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EWrapper<T> {
    Variant1,
    Variant2,
    VariantT(T),
}

impl<T> EWrapper<T> {
    fn t(t: T) -> Self {
        Self::VariantT(t)
    }
}

// needed for merging
impl<T> From<EWrapper<T>> for Option<T> {
    fn from(e: EWrapper<T>) -> Self {
        match e {
            EWrapper::VariantT(t) => Some(t),
            EWrapper::Variant1 | EWrapper::Variant2 => None,
        }
    }
}

// needed for from
impl<T> From<T> for EWrapper<T> {
    fn from(value: T) -> Self {
        EWrapper::t(value)
    }
}

#[optfield(OptS, wrapper = SWrapper)]
#[optfield(OptT, wrapper = TWrapper)]
#[optfield(OptE, wrapper = EWrapper)]
#[optfield(OptSRewrap, wrapper = SWrapper, rewrap)]
#[optfield(OptTRewrap, wrapper = TWrapper, rewrap)]
#[optfield(OptERewrap, wrapper = EWrapper, rewrap)]
#[optfield(OptSMerge, wrapper = SWrapper, merge_fn = merge_s)]
#[optfield(OptTMerge, wrapper = TWrapper, merge_fn = merge_t)]
#[optfield(OptEMerge, wrapper = EWrapper, merge_fn = merge_e)]
#[optfield(OptSMergeRewrap, wrapper = SWrapper,
    rewrap, merge_fn = merge_s_rewrap, attrs)]
#[optfield(OptTMergeRewrap, wrapper = TWrapper,
    rewrap, merge_fn = merge_t_rewrap, attrs)]
#[optfield(OptEMergeRewrap, wrapper = EWrapper,
    rewrap, merge_fn = merge_e_rewrap, attrs)]
#[optfield(OptSFrom, wrapper = SWrapper, from)]
#[optfield(OptTFrom, wrapper = TWrapper, from)]
#[optfield(OptEFrom, wrapper = EWrapper, from)]
#[optfield(OptSFromRewrap, wrapper = SWrapper, from, rewrap)]
#[optfield(OptTFromRewrap, wrapper = TWrapper, from, rewrap)]
#[optfield(OptEFromRewrap, wrapper = EWrapper, from, rewrap)]
#[derive(Debug, Clone)]
struct Original<'a, T> {
    number: u32,
    text: &'a str,
    generic: T,
    optional: Option<&'a [u8]>,
    swrapped: SWrapper<&'a T>,
    twrapped: TWrapper<T>,
    ewrapped: EWrapper<String>,
}

#[test]
fn struct_wrapper() {
    // basics, should just compile
    let _opts = OptS {
        number: SWrapper::new(4),
        text: SWrapper::new("test"),
        generic: SWrapper::new(1),
        optional: SWrapper::new(None),
        swrapped: SWrapper::new(5),
        twrapped: SWrapper::new(TWrapper::new(8)),
        ewrapped: SWrapper::new(EWrapper::Variant1),
    };

    let _opts_rewrap = OptSRewrap {
        number: SWrapper::new(4),
        text: SWrapper::new("test"),
        generic: SWrapper::new(1),
        optional: SWrapper::new(None),
        swrapped: SWrapper::new(SWrapper::new(5)),
        twrapped: SWrapper::new(TWrapper::new(8)),
        ewrapped: SWrapper::new(EWrapper::Variant1),
    };

    let _optt = OptT {
        number: TWrapper::new(4),
        text: TWrapper::new("test"),
        generic: TWrapper::new(1),
        optional: TWrapper::new(None),
        swrapped: TWrapper::new(SWrapper::new(8)),
        twrapped: TWrapper::new(8),
        ewrapped: TWrapper::new(EWrapper::Variant1),
    };

    let _optt_rewrap = OptTRewrap {
        number: TWrapper::new(4),
        text: TWrapper::new("test"),
        generic: TWrapper::new(1),
        optional: TWrapper::new(None),
        swrapped: TWrapper::new(SWrapper::new(8)),
        twrapped: TWrapper::new(TWrapper::new(8)),
        ewrapped: TWrapper::new(EWrapper::Variant1),
    };

    let _opte = OptE {
        number: EWrapper::t(4),
        text: EWrapper::t("test"),
        generic: EWrapper::t(1),
        optional: EWrapper::t(None),
        swrapped: EWrapper::t(SWrapper::new(8)),
        twrapped: EWrapper::t(TWrapper::new(8)),
        ewrapped: EWrapper::t(7),
    };

    let _opte_rewrap = OptERewrap {
        number: EWrapper::t(4),
        text: EWrapper::t("test"),
        generic: EWrapper::t(1),
        optional: EWrapper::t(None),
        swrapped: EWrapper::t(SWrapper::new(8)),
        twrapped: EWrapper::t(TWrapper::new(8)),
        ewrapped: EWrapper::t(EWrapper::t(5)),
    };

    let original = Original {
        number: 1,
        text: "test",
        generic: 6,
        optional: Some(&[1, 2, 3]),
        swrapped: SWrapper::new(&3),
        twrapped: TWrapper::new(4),
        ewrapped: EWrapper::Variant1,
    };

    // merging struct wrapper
    let mut original_clone = original.clone();

    let opts_merge = OptSMerge {
        number: SWrapper::new(0),
        text: SWrapper::new("merge_test"),
        generic: SWrapper::new(0),
        optional: SWrapper::new(Some(&[0])),
        swrapped: SWrapper::new(&0),
        twrapped: SWrapper::new(TWrapper::new(0)),
        ewrapped: SWrapper::new(EWrapper::Variant2),
    };

    original_clone.merge_s(opts_merge);

    assert_eq!(original_clone.number, 0);
    assert_eq!(original_clone.text, "merge_test");
    assert_eq!(original_clone.generic, 0);
    assert_eq!(original_clone.optional, Some([0].as_slice()));
    assert_eq!(original_clone.swrapped, SWrapper::new(&0));
    assert_eq!(original_clone.twrapped, TWrapper::new(0),);
    assert_eq!(original_clone.ewrapped, EWrapper::Variant2);

    let opts_merge_rewrap = OptSMergeRewrap {
        number: SWrapper::new(0),
        text: SWrapper::new("merge_test"),
        generic: SWrapper::new(0),
        optional: SWrapper::new(Some(&[0])),
        swrapped: SWrapper::new(SWrapper::new(&9)),
        twrapped: SWrapper::new(TWrapper::new(0)),
        ewrapped: SWrapper::new(EWrapper::Variant2),
    };

    original_clone.merge_s_rewrap(opts_merge_rewrap);

    assert_eq!(original_clone.swrapped, SWrapper::new(&9));

    original_clone = original.clone();

    // merging tuple wrapper
    let optt_merge = OptTMerge {
        number: TWrapper::new(0),
        text: TWrapper::new("merge_test"),
        generic: TWrapper::new(0),
        optional: TWrapper::new(Some(&[0])),
        swrapped: TWrapper::new(SWrapper::new(&0)),
        twrapped: TWrapper::new(0),
        ewrapped: TWrapper::new(EWrapper::Variant2),
    };

    original_clone.merge_t(optt_merge);

    assert_eq!(original_clone.number, 0);
    assert_eq!(original_clone.text, "merge_test");
    assert_eq!(original_clone.generic, 0);
    assert_eq!(original_clone.optional, Some([0].as_slice()));
    assert_eq!(original_clone.swrapped, SWrapper::new(&0));
    assert_eq!(original_clone.twrapped, TWrapper::new(0),);
    assert_eq!(original_clone.ewrapped, EWrapper::Variant2);

    let optt_merge_rewrap = OptTMergeRewrap {
        number: TWrapper::new(0),
        text: TWrapper::new("merge_test"),
        generic: TWrapper::new(0),
        optional: TWrapper::new(Some(&[0])),
        swrapped: TWrapper::new(SWrapper::new(&0)),
        twrapped: TWrapper::new(TWrapper::new(9)),
        ewrapped: TWrapper::new(EWrapper::Variant2),
    };

    original_clone.merge_t_rewrap(optt_merge_rewrap);

    assert_eq!(original_clone.twrapped, TWrapper::new(9));

    // merging enum wrapper
    original_clone = original.clone();

    let mut opte_merge = OptEMerge {
        number: EWrapper::t(0),
        text: EWrapper::t("merge_test"),
        generic: EWrapper::t(0),
        optional: EWrapper::t(Some(&[0])),
        swrapped: EWrapper::t(SWrapper::new(&0)),
        twrapped: EWrapper::t(TWrapper::new(0)),
        ewrapped: EWrapper::Variant2,
    };

    original_clone.merge_e(opte_merge);

    assert_eq!(original_clone.number, 0);
    assert_eq!(original_clone.text, "merge_test");
    assert_eq!(original_clone.generic, 0);
    assert_eq!(original_clone.optional, Some([0].as_slice()));
    assert_eq!(original_clone.swrapped, SWrapper::new(&0));
    assert_eq!(original_clone.twrapped, TWrapper::new(0),);
    assert_eq!(original_clone.ewrapped, EWrapper::Variant1);

    opte_merge.ewrapped = EWrapper::t(0);

    original_clone.merge_e(opte_merge);

    assert_eq!(original_clone.ewrapped, EWrapper::t(0));

    let mut opte_merge_rewrap = OptEMergeRewrap {
        number: EWrapper::t(0),
        text: EWrapper::t("merge_test"),
        generic: EWrapper::t(0),
        optional: EWrapper::t(Some(&[0])),
        swrapped: EWrapper::t(SWrapper::new(&0)),
        twrapped: EWrapper::t(TWrapper::new(0)),
        ewrapped: EWrapper::t(EWrapper::Variant2),
    };

    original_clone.merge_e_rewrap(opte_merge_rewrap.clone());

    assert_eq!(original_clone.ewrapped, EWrapper::Variant1);

    opte_merge_rewrap.ewrapped = EWrapper::t(EWrapper::t(9));

    original_clone.merge_e_rewrap(opte_merge_rewrap);

    assert_eq!(original_clone.ewrapped, EWrapper::t(9));

    let original_from = Original {
        number: 1,
        text: "from_test",
        generic: 1,
        optional: Some(&[1, 1]),
        swrapped: SWrapper::new(&1),
        twrapped: TWrapper::new(1),
        ewrapped: EWrapper::Variant1,
    };

    // opt with struct wrapper from original
    let opts_from = OptSFrom::from(original_from.clone());

    assert_eq!(opts_from.number, SWrapper::new(1));
    assert_eq!(opts_from.text, SWrapper::new("from_test"));
    assert_eq!(opts_from.generic, SWrapper::new(1));
    assert_eq!(opts_from.optional, SWrapper::new(Some(&[1, 1])));
    assert_eq!(opts_from.swrapped, SWrapper::new(&1));
    assert_eq!(opts_from.twrapped, SWrapper::new(TWrapper::new(1)));
    assert_eq!(opts_from.ewrapped, SWrapper::new(EWrapper::Variant1));

    let optt_from_rewrap = OptTFromRewrap::from(original_from.clone());

    assert_eq!(optt_from_rewrap.swrapped, SWrapper::new(SWrapper::new(1)));

    // opt with tuple wrapper from original
    let optt_from = OptTFrom::from(original_from.clone());

    assert_eq!(opts_from.number, TWrapper::new(1));
    assert_eq!(opts_from.text, TWrapper::new("from_test"));
    assert_eq!(opts_from.generic, TWrapper::new(1));
    assert_eq!(opts_from.optional, TWrapper::new(Some(&[1, 1])));
    assert_eq!(opts_from.swrapped, TWrapper::new(SWrapper::new(&1)));
    assert_eq!(opts_from.twrapped, TWrapper::new(1));
    assert_eq!(opts_from.ewrapped, TWrapper::new(EWrapper::Variant1));

    let optt_from_rewrap = OptTFromRewrap::from(original_from.clone());

    assert_eq!(optt_from_rewrap.twrapped, TWrapper::new(TWrapper::new(1)));

    // opt with enum wrapper from original
    let opte_from = OptEFrom::from(original_from.clone());

    assert_eq!(opts_from.number, EWrapper::t(1));
    assert_eq!(opts_from.text, EWrapper::t("from_test"));
    assert_eq!(opts_from.generic, EWrapper::t(1));
    assert_eq!(opts_from.optional, EWrapper::t(Some(&[1, 1])));
    assert_eq!(opts_from.swrapped, EWrapper::t(SWrapper::new(&1)));
    assert_eq!(opts_from.twrapped, EWrapper::t(TWrapper::new(1)));
    assert_eq!(opts_from.ewrapped, EWrapper::Variant1);

    let opte_from_rewrap = OptEFromRewrap::from(original_from);

    assert_eq!(opte_from_rewrap.ewrapped, EWrapper::t(EWrapper::Variant1));

    // done!
}

#[test]
fn merge_from_impls() {
    // merge and from with generic impls
    #[derive(Debug, Clone)]
    struct GWrapper<T>(T);

    impl<T> From<GWrapper<T>> for Option<T> {
        fn from(value: GWrapper<T>) -> Self {
            Some(value.0)
        }
    }

    impl<T> From<T> for GWrapper<T> {
        fn from(value: T) -> Self {
            GWrapper(value)
        }
    }

    // merge and from with specific impls
    #[derive(Debug, Clone)]
    struct UWrapper<T>(T);

    impl From<UWrapper<u32>> for Option<u32> {
        fn from(value: UWrapper<u32>) -> Self {
            Some(value.0)
        }
    }

    impl From<u32> for UWrapper<u32> {
        fn from(value: u32) -> Self {
            UWrapper(value)
        }
    }

    #[optfield(OptG, wrapper = GWrapper, merge_fn = merge_g, from)]
    #[optfield(OptU, wrapper = GWrapper, merge_fn = merge_u, from)]
    #[derive(Debug, Clone)]
    struct Original {
        field: u32,
    }

    let mut original = Original { field: 0 };

    let mut optg = OptG::from(original.clone());

    assert_eq!(optg.field, GWrapper(0));

    optg.field = GWrapper(1);

    original.merge_g(optg);

    assert_eq!(original.field, 1);

    let mut optu = OptU::from(original.clone());

    assert_eq!(optu.field, UWrapper(1));

    optu.field = UWrapper(2);

    original.merge_u(optu);

    assert_eq!(original.field, 2);
}

#[test]
fn nested_wrappers() {
    // successful compilation is enough for this test

    #[derive(Debug)]
    struct Wrap1<T>(T);

    #[derive(Debug)]
    struct Wrap2<T>(T);

    #[optfield(Opt, wrapper = Wrap1, attrs = add(
        optfield(Opt1, wrapper = Wrap2),
        optfield(Opt2, wrapper = Wrap2, rewrap)
    ))]
    #[derive(Debug)]
    struct Ogiginal {
        opt: Option<u32>,
        wrap1: Wrap1<u32>,
        wrap2: Wrap2<u32>,
    }

    let _ = Opt {
        opt: Wrap1(Some(1)),
        wrap1: Wrap1(1),
        wrap2: Wrap1(Wrap2(1)),
    };

    let _ = Opt1 {
        opt: Wrap2(Wrap1(None)),
        wrap1: Wrap2(Wrap1(2)),
        wrap2: Wrap2(2),
    };

    let _ = Opt2 {
        opt: Wrap2(Wrap1(None)),
        wrap1: Wrap2(Wrap1(2)),
        wrap2: Wrap2(Wrap2(2)),
    };
}
