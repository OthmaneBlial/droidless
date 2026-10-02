use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn compiled_themed_context_snapshots_factories_callbacks_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let wrapper = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/ThemedContextContract;".into(),
                name: "run".into(),
                parameters: vec![],
                returns: "Landroid/view/ContextThemeWrapper;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    let method = Method {
        class: "Landroid/content/Context;".into(),
        name: "getResources".into(),
        parameters: vec![],
        returns: "Landroid/content/res/Resources;".into(),
    };
    let cycle = vm
        .heap
        .instance("Landroid/content/ContextWrapper;")
        .unwrap();
    vm.heap
        .get_mut(cycle)
        .unwrap()
        .fields
        .insert("droidless:context:base".into(), vec![cycle]);
    let error = vm.invoke(method.clone(), vec![cycle], true).unwrap_err();
    assert!(
        format!("{error:#}").contains("Context wrapper nesting limit"),
        "{error:#}"
    );
    vm.invoke(method, vec![wrapper], true).unwrap();
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(wrapper).is_err(),
        "Context wrapper leaked temporary roots"
    );
    assert!(
        vm.heap.get(cycle).is_err(),
        "Recursive wrapper leaked temporary roots"
    );
}

#[test]
fn themed_context_default_ids_and_invalid_constructor_arguments() {
    let constructor = Method {
        class: "Landroid/view/ContextThemeWrapper;".into(),
        name: "<init>".into(),
        parameters: vec!["Landroid/content/Context;".into(), "I".into()],
        returns: "V".into(),
    };
    for (target, expected) in [
        (10, 16973829),
        (13, 16973931),
        (28, 16974120),
        (10000, 16974143),
    ] {
        let mut apk = Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap();
        apk.manifest.target_sdk = Some(target);
        let mut vm = Runtime::new(apk).unwrap();
        let base = vm.heap.instance("Landroid/app/Activity;").unwrap();
        let wrapper = vm.heap.instance(&constructor.class).unwrap();
        let invalid = vm.heap.instance("Ljava/lang/Object;").unwrap();
        for args in [
            vec![wrapper, invalid, Word::ZERO],
            vec![invalid, base, Word::ZERO],
            vec![wrapper, base, invalid],
        ] {
            assert!(vm.invoke(constructor.clone(), args, false).is_err());
            assert_eq!(vm.stack_depth(), 0);
        }
        vm.invoke(constructor.clone(), vec![wrapper, base, Word::ZERO], false)
            .unwrap();
        let id = Method {
            class: constructor.class.clone(),
            name: "getThemeResId".into(),
            parameters: vec![],
            returns: "I".into(),
        };
        assert_eq!(
            vm.invoke(id.clone(), vec![wrapper], true).unwrap(),
            [Word::ZERO]
        );
        vm.invoke(
            Method {
                class: constructor.class.clone(),
                name: "getTheme".into(),
                parameters: vec![],
                returns: "Landroid/content/res/Resources$Theme;".into(),
            },
            vec![wrapper],
            true,
        )
        .unwrap();
        assert_eq!(
            vm.invoke(id, vec![wrapper], true).unwrap(),
            [Word::from(expected)]
        );
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
        assert!(
            vm.heap.get(wrapper).is_err(),
            "Failed constructor leaked roots"
        );
    }
}

#[test]
fn compiled_host_editor_focus_callbacks_gc_rejection_and_failure() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let call = |vm: &mut Runtime, name: &str, parameters: Vec<String>, returns: &str, args| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/HostFocusContract;".into(),
                name: name.into(),
                parameters,
                returns: returns.into(),
            },
            args,
            false,
        )
    };
    let first = call(
        &mut vm,
        "prepare",
        vec!["Landroid/app/Activity;".into()],
        "Landroid/widget/EditText;",
        vec![activity],
    )
    .unwrap()[0];
    let second = call(
        &mut vm,
        "second",
        vec![],
        "Landroid/widget/EditText;",
        vec![],
    )
    .unwrap()[0];
    let state = |vm: &mut Runtime, id, gains, losses| {
        call(
            vm,
            "state",
            vec!["I".into(); 3],
            "V",
            vec![Word::from(id), Word::from(gains), Word::from(losses)],
        )
        .unwrap();
    };
    assert!(vm.focus(first.reference().unwrap()).unwrap());
    assert!(vm.focus(first.reference().unwrap()).unwrap());
    state(&mut vm, 1, 1, 0);
    assert!(vm.focus(second.reference().unwrap()).unwrap());
    state(&mut vm, 2, 2, 1);
    for (name, ty, value, reset) in [
        ("setEnabled", "Z", 0, 1),
        ("setVisibility", "I", 4, 0),
        ("setFocusable", "Z", 0, 1),
    ] {
        for (value, accepted) in [(value, false), (reset, true)] {
            vm.invoke(
                Method {
                    class: "Landroid/view/View;".into(),
                    name: name.into(),
                    parameters: vec![ty.into()],
                    returns: "V".into(),
                },
                vec![first, Word::from(value)],
                true,
            )
            .unwrap();
            if !accepted {
                assert!(!vm.focus(first.reference().unwrap()).unwrap());
                state(&mut vm, 2, 2, 1);
            }
        }
    }
    call(&mut vm, "fail", vec!["Z".into()], "V", vec![Word::from(1)]).unwrap();
    let error = vm.focus(first.reference().unwrap()).unwrap_err();
    assert!(format!("{error:#}").contains("host focus failure"));
    assert_eq!(vm.stack_depth(), 0);
    state(&mut vm, 1, 3, 2);
    call(&mut vm, "fail", vec!["Z".into()], "V", vec![Word::ZERO]).unwrap();
    assert!(vm.focus(second.reference().unwrap()).unwrap());
    state(&mut vm, 2, 4, 3);
    let wrong = vm.heap.instance("Ljava/lang/Object;").unwrap();
    assert!(
        format!("{:#}", vm.focus(wrong.reference().unwrap()).unwrap_err()).contains("not a View")
    );
    assert!(vm.focus(usize::MAX).is_err());
    call(&mut vm, "release", vec![], "V", vec![]).unwrap();
    vm.collect();
    assert!(
        vm.heap.get(first).is_err()
            && vm.heap.get(second).is_err()
            && vm.heap.get(activity).is_err()
    );
}

#[cfg(target_os = "macos")]
#[test]
fn compiled_host_font_metrics_size_faces_gc_and_zero() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    vm.invoke(
        Method {
            class: "Lorg/droidless/images/FontMetricsContract;".into(),
            name: "run".into(),
            parameters: vec![],
            returns: "V".into(),
        },
        vec![],
        false,
    )
    .unwrap();
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn compiled_scalar_animation_clock_values_listeners_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/ValueAnimatorContract;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let animator = call(&mut vm, "start", "Landroid/animation/ValueAnimator;")[0];
    vm.collect();
    vm.advance_time(500).unwrap();
    call(&mut vm, "halfway", "V");
    vm.collect();
    vm.advance_time(100).unwrap();
    call(&mut vm, "delayed", "V");
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(animator).is_err(),
        "animation fault retained roots"
    );
}

#[test]
fn compiled_text_watchers_buffers_utf16_reentrancy_host_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/TextWatcherContract;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let editor = call(&mut vm, "run", "Landroid/widget/EditText;")[0];
    vm.edit(editor.reference().unwrap(), "host").unwrap();
    call(&mut vm, "hostCheck", "V");
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(editor).is_err(),
        "text callbacks retained temporary roots"
    );
}

#[test]
fn compiled_typeface_identity_paint_view_metadata_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let text = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/TypefaceContract;".into(),
                name: "run".into(),
                parameters: vec![],
                returns: "Landroid/view/View;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    let view = vm.heap.get(text).unwrap().view.as_ref().unwrap();
    assert_eq!((view.font_family, view.font_style), (2, 3));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(text).is_err(),
        "typeface calls leaked temporary roots"
    );
}

#[test]
fn compiled_child_overloads_dispatch_getters_factories_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let parent = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/SizedChildContract;".into(),
                name: "overloads".into(),
                parameters: vec![],
                returns: "Landroid/view/View;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap
            .get(parent)
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .children
            .len(),
        4
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(parent).is_err(),
        "child attachment leaked temporary roots"
    );
}

#[test]
fn compiled_sized_children_virtual_factories_attachment_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let parent = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/SizedChildContract;".into(),
                name: "run".into(),
                parameters: vec![],
                returns: "Landroid/view/View;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap
            .get(parent)
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .children
            .len(),
        2
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(parent).is_err(),
        "sized attachment leaked temporary roots"
    );
}

#[test]
fn compiled_text_appearance_context_callbacks_gc_size_and_fault_recovery() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let view = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/StyleColorContract;".into(),
                name: "appearance".into(),
                parameters: vec![],
                returns: "Landroid/widget/TextView;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap.get(view).unwrap().view.as_ref().unwrap().text_color,
        0xff224466
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(view).is_err(),
        "text appearance leaked temporary roots"
    );
}

#[test]
fn compiled_typed_colors_theme_dispatch_snapshot_gc_and_faults() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let method = |name: &str, returns: &str| Method {
        class: "Lorg/droidless/images/StyleColorContract;".into(),
        name: name.into(),
        parameters: vec![],
        returns: returns.into(),
    };
    assert_eq!(
        vm.invoke(method("run", "I"), vec![], false).unwrap(),
        [Word::from(1)]
    );
    let resources = vm.heap.instance("Landroid/content/res/Resources;").unwrap();
    for id in [0, 0x0106ffff] {
        let error = vm
            .invoke(
                Method {
                    class: "Landroid/content/res/Resources;".into(),
                    name: "getColor".into(),
                    parameters: vec!["I".into()],
                    returns: "I".into(),
                },
                vec![resources, Word::from(id)],
                true,
            )
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("missing or complex"),
            "{error:#}"
        );
    }
    let cycle = vm
        .invoke(
            method("cycle", "Landroid/content/res/TypedArray;"),
            vec![],
            false,
        )
        .unwrap()[0];
    for index in [-1, 1, i32::MAX] {
        let error = vm
            .invoke(
                Method {
                    class: "Landroid/content/res/TypedArray;".into(),
                    name: "getColorStateList".into(),
                    parameters: vec!["I".into()],
                    returns: "Landroid/content/res/ColorStateList;".into(),
                },
                vec![cycle, Word::from(index)],
                true,
            )
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("ArrayIndexOutOfBoundsException"),
            "{error:#}"
        );
    }
    let error = vm
        .invoke(
            Method {
                class: "Landroid/content/res/TypedArray;".into(),
                name: "getColorStateList".into(),
                parameters: vec!["I".into()],
                returns: "Landroid/content/res/ColorStateList;".into(),
            },
            vec![cycle, Word::ZERO],
            true,
        )
        .unwrap_err();
    assert!(
        format!("{error:#}").contains("theme attribute reference cycle"),
        "{error:#}"
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(cycle).is_err(),
        "typed color faults leaked temporary roots"
    );
}

#[test]
fn compiled_child_drawable_states_capacity_callbacks_gc_faults_and_cycle_bound() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let method = |name: &str, returns: &str| Method {
        class: "Lorg/droidless/images/ChildStateContract;".into(),
        name: name.into(),
        parameters: vec!["Landroid/app/Activity;".into()],
        returns: returns.into(),
    };
    let group = vm
        .invoke(
            method("run", "Landroid/widget/LinearLayout;"),
            vec![activity],
            false,
        )
        .unwrap()[0];
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(group).is_err(),
        "child states leaked temporary roots"
    );
    for name in ["cycle", "queryCycle", "deep"] {
        let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
        let error = vm
            .invoke(method(name, "V"), vec![activity], false)
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("drawable state nesting limit"),
            "{error:#}"
        );
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
        assert!(
            vm.heap.get(activity).is_err(),
            "state failure leaked temporary roots"
        );
    }
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    vm.invoke(
        method("run", "Landroid/widget/LinearLayout;"),
        vec![activity],
        false,
    )
    .unwrap();
}

#[test]
fn compiled_text_paint_construction_flags_inheritance_fields_and_gc() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let paint = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/TextPaintContract;".into(),
                name: "run".into(),
                parameters: vec![],
                returns: "Landroid/graphics/Paint;".into(),
            },
            vec![],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap.get(paint).unwrap().class,
        "Lorg/droidless/images/TextPaintContract$Probe;"
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(paint).is_err(),
        "TextPaint construction leaked temporary roots"
    );
}

#[test]
fn compiled_touch_focus_default_clickability_cancellation_and_callback_gc() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    assert_eq!(
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/TouchFocusContract;".into(),
                name: "run".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "I".into(),
            },
            vec![activity],
            false
        )
        .unwrap(),
        [Word::from(1)]
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(activity).is_err(),
        "touch focus leaked temporary roots"
    );
}

#[test]
fn compiled_background_resources_dispatch_cache_gc_and_fault_recovery() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let view = vm
        .invoke(
            Method {
                class: "Lorg/droidless/images/BackgroundResourceContract;".into(),
                name: "run".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "Landroid/view/View;".into(),
            },
            vec![activity],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap.get(view).unwrap().view.as_ref().unwrap().background,
        Some(42)
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(vm.heap.get(activity).is_err());
    assert!(
        vm.heap.get(view).is_err(),
        "resource callback leaked temporary roots"
    );
}

#[test]
fn compiled_descendant_coordinates_scroll_overflow_faults_and_hierarchy_bound() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    assert_eq!(
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/CoordinateContract;".into(),
                name: "run".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "I".into(),
            },
            vec![activity],
            false
        )
        .unwrap(),
        [Word::from(1)]
    );
    let root = vm.heap.instance("Landroid/widget/FrameLayout;").unwrap();
    let child = vm.heap.instance("Landroid/widget/FrameLayout;").unwrap();
    let rect = vm.heap.instance("Landroid/graphics/Rect;").unwrap();
    for (key, value) in [("left", -3), ("top", 7), ("scroll-x", -5), ("scroll-y", 9)] {
        vm.heap
            .get_mut(child)
            .unwrap()
            .fields
            .insert(format!("droidless:view:{key}"), vec![Word::from(value)]);
    }
    vm.heap
        .get_mut(root)
        .unwrap()
        .fields
        .insert("droidless:view:scroll-x".into(), vec![Word::from(1000)]);
    let method = Method {
        class: "Landroid/view/ViewGroup;".into(),
        name: "offsetDescendantRectToMyCoords".into(),
        parameters: vec![
            "Landroid/view/View;".into(),
            "Landroid/graphics/Rect;".into(),
        ],
        returns: "V".into(),
    };
    vm.heap
        .get_mut(child)
        .unwrap()
        .fields
        .insert("droidless:view:parent".into(), vec![root]);
    vm.invoke(method.clone(), vec![root, child, rect], true)
        .unwrap();
    assert_eq!(
        vm.heap.get(rect).unwrap().fields["Landroid/graphics/Rect;->left:I"],
        [Word::from(2)]
    );
    assert_eq!(
        vm.heap.get(rect).unwrap().fields["Landroid/graphics/Rect;->top:I"],
        [Word::from(-2)]
    );
    vm.heap
        .get_mut(child)
        .unwrap()
        .fields
        .insert("droidless:view:parent".into(), vec![child]);
    let error = vm
        .invoke(method.clone(), vec![root, child, rect], true)
        .unwrap_err();
    assert!(format!("{error:#}").contains("cyclic or too deep View coordinate hierarchy"));
    assert_eq!(vm.stack_depth(), 0);
    vm.heap
        .get_mut(child)
        .unwrap()
        .fields
        .insert("droidless:view:parent".into(), vec![root]);
    vm.invoke(method.clone(), vec![root, child, rect], true)
        .unwrap();
    let error = vm.invoke(method, vec![root, child], true).unwrap_err();
    assert!(format!("{error:#}").contains("argument"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    for object in [activity, root, child, rect] {
        assert!(
            vm.heap.get(object).is_err(),
            "coordinate conversion leaked roots"
        );
    }
}

#[test]
fn view_click_listener_query_gc_and_explicit_xml_listener_removal() {
    let call = |vm: &mut Runtime, name: &str, parameters: &[&str], returns: &str, args| {
        vm.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: name.into(),
                parameters: parameters.iter().map(|value| (*value).into()).collect(),
                returns: returns.into(),
            },
            args,
            true,
        )
        .unwrap()
    };
    for (bytes, text) in [
        (
            include_bytes!("../../../fixtures/generated/counter.apk").as_slice(),
            "Increment",
        ),
        (
            include_bytes!("../../../fixtures/generated/intents.apk").as_slice(),
            "Open detail",
        ),
    ] {
        let mut vm = Runtime::new(Apk::parse(bytes).unwrap()).unwrap();
        vm.launch().unwrap();
        let activity = vm.activity;
        let root = vm.root.unwrap();
        let tree = vm.snapshot().unwrap();
        let button = Word::Ref(
            tree.children
                .iter()
                .find(|node| node.view.text == text)
                .unwrap()
                .handle,
        );
        assert_eq!(
            call(&mut vm, "hasOnClickListeners", &[], "Z", vec![root]),
            [Word::ZERO]
        );
        call(
            &mut vm,
            "setClickable",
            &["Z"],
            "V",
            vec![button, Word::ZERO],
        );
        vm.collect();
        assert_eq!(
            call(&mut vm, "hasOnClickListeners", &[], "Z", vec![button]),
            [Word::from(1)]
        );
        if text == "Increment" {
            assert_eq!(
                call(&mut vm, "performClick", &[], "Z", vec![button]),
                [Word::from(1)]
            );
            assert_eq!(vm.snapshot().unwrap().children[0].view.text, "1");
        }
        call(
            &mut vm,
            "setOnClickListener",
            &["Landroid/view/View$OnClickListener;"],
            "V",
            vec![button, Word::ZERO],
        );
        vm.collect();
        assert_eq!(
            call(&mut vm, "hasOnClickListeners", &[], "Z", vec![button]),
            [Word::ZERO]
        );
        assert_eq!(
            call(&mut vm, "performClick", &[], "Z", vec![button]),
            [Word::ZERO]
        );
        assert_eq!(vm.activity, activity);
    }
}

#[test]
fn compiled_focus_ownership_callbacks_gc_removal_and_fault_recovery() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let call = |vm: &mut Runtime, name: &str, parameters: Vec<String>, returns: &str, args| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/FocusContract;".into(),
                name: name.into(),
                parameters,
                returns: returns.into(),
            },
            args,
            false,
        )
    };
    assert_eq!(
        call(
            &mut vm,
            "run",
            vec!["Landroid/app/Activity;".into()],
            "I",
            vec![activity]
        )
        .unwrap(),
        [Word::from(1)]
    );
    let root = call(&mut vm, "root", vec![], "Landroid/view/View;", vec![]).unwrap()[0];
    let error = call(&mut vm, "fault", vec![], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("focus callback failure"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "recover", vec![], "V", vec![]).unwrap();
    let error = call(&mut vm, "cycle", vec![], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("cyclic or too deep View focus parent hierarchy"));
    let error = vm
        .invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "requestFocus".into(),
                parameters: vec!["I".into()],
                returns: "Z".into(),
            },
            vec![root],
            true,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("missing focus argument"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "startWorkerFocus", vec![], "V", vec![]).unwrap();
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("unsupported UI access from a guest worker"));
    assert_eq!(
        call(&mut vm, "workerFocusPreserved", vec![], "Z", vec![]).unwrap(),
        [Word::from(1)]
    );
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "release", vec![], "V", vec![]).unwrap();
    vm.collect();
    assert!(
        vm.heap.get(activity).is_err(),
        "focus callback leaked temporary roots"
    );
    assert!(
        vm.heap.get(root).is_err(),
        "focus ownership retained the released hierarchy"
    );
}

#[test]
fn compiled_text_layout_measurement_invalidation_and_callback_gc() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let result = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: "run".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "I".into(),
            },
            vec![activity],
            false,
        )
        .unwrap();
    assert_eq!(result, [Word::from(1)]);
    assert_eq!(vm.stack_depth(), 0);
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let root = call(&mut vm, "root", "Landroid/view/View;")[0];
    let failure = vm
        .invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "setPaddingRelative".into(),
                parameters: vec!["I".into(); 4],
                returns: "V".into(),
            },
            vec![root, Word::from(-1), Word::ZERO, Word::ZERO, Word::ZERO],
            true,
        )
        .unwrap_err();
    assert!(format!("{failure:#}").contains("padding must have four non-negative values"));
    assert_eq!(
        vm.heap.get(root).unwrap().view.as_ref().unwrap().padding,
        [6.0, 3.0, 6.0, 3.0]
    );
    assert!(!vm.heap.get(root).unwrap().fields["droidless:view:padding-relative"][0].truth());
    assert_eq!(vm.stack_depth(), 0);
    let tree = droidless_runtime::ui::layout(&vm.heap, root, 162.0, 1000.0).unwrap();
    let row = &tree.children[0];
    assert_eq!(
        (row.rect.x, row.rect.y, row.rect.width, row.rect.height),
        (6.0, 3.0, 150.0, 48.0)
    );
    assert_eq!(row.children.len(), 2);
    assert_eq!(
        (row.children[0].rect.x, row.children[0].rect.width),
        (14.0, 90.0)
    );
    assert_eq!(
        (row.children[1].rect.x, row.children[1].rect.width),
        (108.0, 42.0)
    );
    vm.root = Some(root);
    vm.width = 162.0;
    vm.height = 1000.0;
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(1)]);
    call(&mut vm, "remeasure", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(2)]);
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(2)]);
    call(&mut vm, "failLayout", "V");
    assert!(format!("{:#}", vm.layout_snapshot().unwrap_err()).contains("layout failure"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "recoverLayout", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(4)]);
    vm.root = None;
    call(&mut vm, "release", "V");
    let failure = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: "mutate".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "V".into(),
            },
            vec![activity],
            false,
        )
        .unwrap_err();
    assert!(format!("{failure:#}").contains("hierarchy mutation during container measurement"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(activity).is_err(),
        "measurement retained temporary roots"
    );
}

#[test]
fn compiled_widget_metadata_adapter_and_timed_scroll_contracts() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    vm.launch().unwrap();
    let tree = vm.snapshot().unwrap();
    assert_eq!(
        tree.children[1].view.content_description.as_deref(),
        Some("Packaged PNG source")
    );
    assert_eq!(
        tree.children[2].view.content_description.as_deref(),
        Some("Decoded PNG image")
    );
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/WidgetProbe;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let state = |vm: &mut Runtime| {
        let word = call(vm, "scrollState", "Ljava/lang/String;")[0];
        vm.heap.text(word).unwrap().to_owned()
    };
    call(&mut vm, "startScroll", "V");
    assert_eq!(state(&mut vm), "true:false:10:20");
    vm.advance_time(500).unwrap();
    assert_eq!(state(&mut vm), "true:false:21:0");
    call(&mut vm, "stopScroll", "V");
    assert_eq!(state(&mut vm), "false:true:21:0");
    call(&mut vm, "abortScroll", "V");
    assert_eq!(state(&mut vm), "false:true:31:-21");
    call(&mut vm, "startScroll", "V");
    vm.advance_time(1000).unwrap();
    assert_eq!(state(&mut vm), "true:true:31:-21");
    assert_eq!(state(&mut vm), "false:true:31:-21");
    call(&mut vm, "startDefaultScroll", "V");
    assert_eq!(state(&mut vm), "true:false:0:0");
    vm.advance_time(125).unwrap();
    assert_eq!(state(&mut vm), "true:false:969:-969");
    vm.advance_time(125).unwrap();
    assert_eq!(state(&mut vm), "true:true:1000:-1000");
    call(&mut vm, "startLargeScroll", "V");
    assert_eq!(state(&mut vm), "true:false:16777215:0");
    call(&mut vm, "startInstantScroll", "V");
    assert_eq!(state(&mut vm), "true:true:4:6");
    let failed = call(&mut vm, "failedScroll", "Landroid/widget/OverScroller;")[0];
    call(&mut vm, "clearScroll", "V");
    vm.collect();
    assert!(
        vm.heap.get(failed).is_err(),
        "interpolator failure retained temporary roots"
    );
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "invalidateFrame", "V");
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    call(&mut vm, "invalidateDetachedFrame", "V");
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(call(&mut vm, "attachedWindowFocus", "Z"), [Word::ZERO]);
    vm.set_host_window_focus(true);
    assert_eq!(call(&mut vm, "attachedWindowFocus", "Z"), [Word::from(1)]);
    assert_eq!(call(&mut vm, "detachedWindowFocus", "Z"), [Word::ZERO]);
    vm.set_host_window_focus(false);
    assert_eq!(call(&mut vm, "attachedWindowFocus", "Z"), [Word::ZERO]);
    let removed = call(&mut vm, "startFrame", "Landroid/view/View;")[0];
    let root = call(&mut vm, "frameRoot", "Landroid/view/View;")[0];
    let frame_state = |vm: &mut Runtime| {
        let word = call(vm, "frameState", "Ljava/lang/String;")[0];
        vm.heap.text(word).unwrap().to_owned()
    };
    vm.layout_snapshot().unwrap();
    assert_eq!(frame_state(&mut vm), "1:0:0");
    vm.collect();
    assert!(
        vm.heap.get(removed).is_err(),
        "detached child retained after frame"
    );
    assert_eq!(vm.advance_time(500).unwrap(), 1);
    let half = vm.layout_snapshot().unwrap();
    assert_eq!(half.children.len(), 2);
    assert_eq!(half.children[0].rect.x, 50.0);
    assert_eq!(frame_state(&mut vm), "2:50:0");
    assert_eq!(vm.advance_time(500).unwrap(), 1);
    let done = vm.layout_snapshot().unwrap();
    assert_eq!(done.children[0].rect.x, 100.0);
    assert_eq!(frame_state(&mut vm), "3:100:0");
    assert_eq!(vm.poll_messages().unwrap(), 0);
    call(&mut vm, "failFrame", "V");
    assert!(format!("{:#}", vm.layout_snapshot().unwrap_err()).contains("scroll frame failure"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "recoverFrame", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(frame_state(&mut vm), "4:100:0");
    call(&mut vm, "releaseFrame", "V");
    vm.collect();
    assert!(
        vm.heap.get(root).is_err(),
        "scroll failure retained temporary roots"
    );
    vm.close().unwrap();

    for (target, minimum, expected) in [
        (Some(28), Some(21), 28),
        (None, Some(7), 7),
        (None, None, 1),
    ] {
        let mut apk = Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap();
        apk.manifest.target_sdk = target;
        apk.manifest.min_sdk = minimum;
        let mut vm = Runtime::new(apk).unwrap();
        let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
        assert_eq!(
            vm.invoke(
                Method {
                    class: "Lorg/droidless/images/WidgetProbe;".into(),
                    name: "metadataTarget".into(),
                    parameters: vec!["Landroid/app/Activity;".into()],
                    returns: "I".into(),
                },
                vec![activity],
                false
            )
            .unwrap(),
            [Word::from(expected)]
        );
    }
}
