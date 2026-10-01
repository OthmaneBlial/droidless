use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Word, bits64, wide},
};

#[test]
fn boxed_double_preserves_bits_and_dispatches_through_number() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    for bits in [
        12.0f64.to_bits(),
        (-0.0f64).to_bits(),
        f64::INFINITY.to_bits(),
        0x7ff8_0000_0000_0042,
    ] {
        let boxed = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Double;".into(),
                    name: "valueOf".into(),
                    parameters: vec!["D".into()],
                    returns: "Ljava/lang/Double;".into(),
                },
                wide(bits),
                false,
            )
            .unwrap()[0];
        let words = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Number;".into(),
                    name: "doubleValue".into(),
                    parameters: vec![],
                    returns: "D".into(),
                },
                vec![boxed],
                true,
            )
            .unwrap();
        assert_eq!(bits64(&words).unwrap(), bits);
        let text = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Object;".into(),
                    name: "toString".into(),
                    parameters: vec![],
                    returns: "Ljava/lang/String;".into(),
                },
                vec![boxed],
                true,
            )
            .unwrap()[0];
        let expected = if f64::from_bits(bits).is_nan() {
            "NaN"
        } else if bits == f64::INFINITY.to_bits() {
            "Infinity"
        } else if bits == (-0.0f64).to_bits() {
            "-0.0"
        } else {
            "12.0"
        };
        assert_eq!(vm.heap.text(text).unwrap(), expected);
        let nan = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Double;".into(),
                    name: "isNaN".into(),
                    parameters: vec!["D".into()],
                    returns: "Z".into(),
                },
                wide(bits),
                false,
            )
            .unwrap()[0];
        assert_eq!(nan, Word::from(i32::from(f64::from_bits(bits).is_nan())));
        let error = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Object;".into(),
                    name: "equals".into(),
                    parameters: vec!["Ljava/lang/Object;".into()],
                    returns: "Z".into(),
                },
                vec![boxed, boxed],
                true,
            )
            .unwrap_err();
        assert!(format!("{error:#}").contains("unsupported Double method"));
    }
    for value in [i64::MIN, 0, i64::MAX] {
        let text = vm
            .invoke(
                Method {
                    class: "Ljava/lang/Long;".into(),
                    name: "toString".into(),
                    parameters: vec!["J".into()],
                    returns: "Ljava/lang/String;".into(),
                },
                wide(value as u64),
                false,
            )
            .unwrap()[0];
        assert_eq!(vm.heap.text(text).unwrap(), value.to_string());
    }
}

#[test]
fn java_decimal_float_parsing_handles_suffixes_overflow_and_guest_faults() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    for (class, name, returns) in [
        ("Ljava/lang/Float;", "parseFloat", "F"),
        ("Ljava/lang/Double;", "parseDouble", "D"),
    ] {
        let method = Method {
            class: class.into(),
            name: name.into(),
            parameters: vec!["Ljava/lang/String;".into()],
            returns: returns.into(),
        };
        for (text, expected) in [
            ("12.5", 12.5),
            (" -0.0F ", -0.0),
            ("1e+3D", 1000.),
            (".5", 0.5),
            ("\t+Infinity\n", f64::INFINITY),
            ("+NaN", f64::NAN),
            ("3.4028236e38", 3.4028236e38),
            ("1e-50", 1e-50),
        ] {
            let text = vm.heap.string(text.into()).unwrap();
            let words = vm.invoke(method.clone(), vec![text], false).unwrap();
            if returns == "F" {
                let value = f32::from_bits(words[0].int().unwrap() as u32);
                if expected.is_nan() {
                    assert!(value.is_nan());
                } else {
                    assert_eq!(value.to_bits(), (expected as f32).to_bits());
                }
            } else {
                let value = f64::from_bits(bits64(&words).unwrap());
                if expected.is_nan() {
                    assert!(value.is_nan());
                } else {
                    assert_eq!(value.to_bits(), expected.to_bits());
                }
            }
        }
        for text in ["nan", "inf", "12ff", "1e", "--1", "\u{a0}23\u{a0}"] {
            let text = vm.heap.string(text.into()).unwrap();
            let error = vm.invoke(method.clone(), vec![text], false).unwrap_err();
            assert!(format!("{error:#}").contains("NumberFormatException"));
        }
        let hex = vm.heap.string("0x1p1".into()).unwrap();
        assert!(
            format!("{:#}", vm.invoke(method, vec![hex], false).unwrap_err())
                .contains("hexadecimal floating-point literals are unsupported")
        );
    }
}
