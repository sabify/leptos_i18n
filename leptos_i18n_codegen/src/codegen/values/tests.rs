use leptos_i18n_parser::{
    extraction::{Literal, Locale, LocalesOrNamespaces, ParsedLocales, Value, extract_locales},
    options::{Config, FileFormat, ParseOptions},
    parsing::parse_locales_raw,
};
use quote::quote;

use super::{fmt, into_view};
use crate::{CodegenOptions, gen_code};

fn parse_fixture(namespaces: bool, format: FileFormat) -> ParsedLocales {
    let cfg = Config::new("en")
        .unwrap()
        .add_locale("fr")
        .unwrap()
        .parse_options(ParseOptions::new().file_format(format));
    let cfg = if namespaces {
        cfg.add_namespace("my-ns").unwrap()
    } else {
        cfg
    };
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/regressions");
    let raw = parse_locales_raw(Some(root), cfg).unwrap();
    let parsed = extract_locales(raw.values, raw.cfg, Some(raw.diag)).unwrap();
    assert!(parsed.diag.errors().is_empty(), "{:?}", parsed.diag);
    parsed
}

fn const_arm(locale: &Locale, value: Value, multi_kind: bool, show_key: Option<&str>) -> String {
    into_view::gen_const_values_match_arms(
        &[(locale, &value)],
        &Default::default(),
        &syn::parse_quote!(Locale),
        &syn::parse_quote!(Keys),
        multi_kind,
        show_key,
    )
    .next()
    .unwrap()
    .to_string()
}

#[test]
fn b1_scope_markers_use_idents() {
    let parsed = parse_fixture(false, FileFormat::Json);
    let code = gen_code(&parsed, CodegenOptions::new())
        .unwrap()
        .to_string();
    assert!(code.contains("Marker < '_' >"));
    assert!(!code.contains("Marker < '-' >"));
}

#[test]
fn b2_namespaces_have_scope_accessors() {
    let code = gen_code(
        &parse_fixture(true, FileFormat::Json),
        CodegenOptions::new(),
    )
    .unwrap()
    .to_string();
    assert!(code.contains("SubkeyKeyAccessor"));
    assert!(code.contains("pub const fn my_ns (self)"));
}

#[test]
fn b3_namespaces_respect_crate_alias() {
    let options =
        CodegenOptions::new().crate_path(Some(syn::parse_str("renamed_runtime").unwrap()));
    let code = gen_code(&parse_fixture(true, FileFormat::Json), options)
        .unwrap()
        .to_string();
    assert!(code.contains("use renamed_runtime as __l_i18n_crate"));
    assert!(!code.contains("leptos_i18n :: reexports"));
}

#[test]
fn b4_non_finite_float_paths() {
    for format in [FileFormat::Yaml, FileFormat::Json5] {
        let code = gen_code(&parse_fixture(false, format), CodegenOptions::new())
            .unwrap()
            .to_string();
        for constant in ["NAN", "INFINITY", "NEG_INFINITY"] {
            assert!(code.contains(&format!("f64 :: {constant}")));
        }
    }
    let parsed = parse_fixture(false, FileFormat::Json);
    let LocalesOrNamespaces::Locales(locales) = &parsed.values else {
        panic!()
    };
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.5] {
        let literal = Literal::Float(number);
        let view = into_view::gen_render_lit(&literal, &syn::parse_quote!(strings), 0);
        let display = fmt::gen_fmt_lit(
            &literal,
            &syn::parse_quote!(strings),
            0,
            &syn::parse_quote!(formatter),
        );
        assert!(!view.is_empty() && !display.is_empty());
        for multi_kind in [false, true] {
            assert!(
                !const_arm(
                    &locales.locales[0],
                    Value::Literal(literal.clone()),
                    multi_kind,
                    None,
                )
                .is_empty()
            );
        }
    }
}

#[test]
fn b5_const_bloc_elements_are_wrapped() {
    let parsed = parse_fixture(false, FileFormat::Json);
    let code = gen_code(&parsed, CodegenOptions::new())
        .unwrap()
        .to_string();
    assert!(code.contains("Literal :: Unsigned (45u64)"));
    assert!(code.contains("Literal :: String"));
    let LocalesOrNamespaces::Locales(locales) = &parsed.values else {
        panic!()
    };
    for multi_kind in [false, true] {
        let bloc = Value::Bloc(
            [
                Literal::String(0),
                Literal::Signed(-1),
                Literal::Unsigned(45),
                Literal::Float(f64::NAN),
                Literal::Bool(true),
            ]
            .map(Value::Literal)
            .to_vec(),
        );
        let code = const_arm(&locales.locales[0], bloc, multi_kind, None);
        for variant in ["String", "Signed", "Unsigned", "Float", "Bool"] {
            assert!(code.contains(&format!("Literal :: {variant}")));
        }
        assert_eq!(code.contains("Literal :: Multiple"), multi_kind);
        let single = Value::Bloc(vec![Value::Literal(Literal::Unsigned(45))]);
        let code = const_arm(&locales.locales[0], single, multi_kind, None);
        assert!(code.contains("__VALUES"));
    }
}

#[test]
fn b6_show_keys_only_preserves_api_and_const_types() {
    let parsed = parse_fixture(false, FileFormat::Json);
    let code = gen_code(&parsed, CodegenOptions::new().show_keys_only(true))
        .unwrap()
        .to_string();
    assert!(code.contains("pub fn var_count"));
    for key in [
        "text",
        "with_arg",
        "with_comp",
        "bloc",
        "mixed",
        "number",
        "boolean",
        "my-key.child",
    ] {
        assert!(code.contains(&quote!(#key).to_string()));
    }
    assert!(code.contains("write_str (\"with_arg\")"));
    assert!(code.contains("Literal :: String (\"mixed\")"));
    assert!(code.contains("Literal :: String (\"bloc\")"));
    assert!(!code.contains("index_translations"));
    let LocalesOrNamespaces::Locales(locales) = &parsed.values else {
        panic!()
    };
    for literal in [
        Literal::Unsigned(45),
        Literal::Signed(-1),
        Literal::Float(1.5),
        Literal::Bool(true),
    ] {
        let value = Value::Literal(literal);
        assert_eq!(
            const_arm(&locales.locales[0], value.clone(), false, Some("key")),
            const_arm(&locales.locales[0], value.clone(), false, None)
        );
        assert!(
            const_arm(&locales.locales[0], value, true, Some("key"))
                .contains("Literal :: String (\"key\")")
        );
    }
    let namespaced = parse_fixture(true, FileFormat::Json);
    let code = gen_code(&namespaced, CodegenOptions::new().show_keys_only(true))
        .unwrap()
        .to_string();
    assert!(code.contains("write_str (\"my-ns::text\")"));
}
