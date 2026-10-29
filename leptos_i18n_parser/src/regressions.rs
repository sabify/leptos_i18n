use std::collections::BTreeSet;

use serde::de::DeserializeSeed;

use crate::{
    error::{Diagnostics, Error, Warning},
    extraction::{
        Literal, Locales, LocalesOrNamespaces, ParsedLocales, Value, Values, ValuesOrSubkeys,
        extract_locales,
    },
    options::{Config, ParseOptions},
    parser::{
        ParseContext, ValuesSeed,
        locale::{RawLocale, RawLocalesOrNamespaces, RawNamespace},
        raw_value::{
            RawLiteral, RawValue,
            component::{Component, RawAttributeValue, RawAttributes},
        },
    },
    utils::{Key, KeyPath, Loc},
};

fn key(name: &str) -> Key {
    Key::new(name).unwrap()
}

fn with_context(f: impl FnOnce(&ParseContext)) {
    let cfg = Config::new("en").unwrap();
    let path = KeyPath::default();
    let diag = Diagnostics::new();
    f(&ParseContext {
        loc: Loc {
            key_path: &path,
            locale: &cfg.locales[0],
        },
        diag: &diag,
        formatters: &cfg.options.formatters,
        parse_fns: RawValue::DEFAULT_FNS,
    });
}

fn raw_locales(cfg: &Config, sources: &[&str], diag: &Diagnostics) -> Vec<RawLocale> {
    cfg.locales
        .iter()
        .zip(sources)
        .map(|(locale, source)| {
            let seed = ValuesSeed {
                name: locale.key.clone(),
                top_locale_name: locale,
                key_path: KeyPath::default(),
                diag,
                formatters: &cfg.options.formatters,
            };
            let values = seed
                .deserialize(&mut serde_json::Deserializer::from_str(source))
                .unwrap();
            RawLocale {
                name: locale.clone(),
                values,
            }
        })
        .collect()
}

fn extract(cfg: Config, sources: &[&str]) -> ParsedLocales {
    let diag = Diagnostics::new();
    let raw = raw_locales(&cfg, sources, &diag);
    extract_locales(RawLocalesOrNamespaces::Locales(raw), cfg, Some(diag)).unwrap()
}

fn locales(parsed: &ParsedLocales) -> &Locales {
    let LocalesOrNamespaces::Locales(locales) = &parsed.values else {
        panic!()
    };
    locales
}

fn values<'a>(parsed: &'a ParsedLocales, name: &str) -> &'a Values {
    let ValuesOrSubkeys::Values(values) = &locales(parsed).keys.values[&key(name)] else {
        panic!()
    };
    values
}

fn assert_default_coverage(parsed: &ParsedLocales, name: &str) {
    let values = values(parsed, name);
    let mut covered: BTreeSet<_> = values.values.keys().cloned().collect();
    for defaults in values.defaults.compute().values() {
        for locale in defaults {
            assert!(
                covered.insert(locale.clone()),
                "locale {locale} covered twice"
            );
        }
    }
    assert_eq!(
        covered,
        parsed.cfg.locales.iter().map(|l| l.key.clone()).collect()
    );
}

#[test]
fn a1_nested_component_attributes() {
    with_context(|ctx| {
        let found = Component::find_component(
            "<comp>before<comp attr=\"x\">inner</comp>after</comp>end",
            ctx.loc,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            found.inner,
            Some("before<comp attr=\"x\">inner</comp>after")
        );
        assert_eq!(found.after, "end");
        let nested = Component::find_component(found.inner.unwrap(), ctx.loc)
            .unwrap()
            .unwrap();
        assert_eq!(
            (nested.before, nested.inner, nested.attrs, nested.after),
            ("before", Some("inner"), "attr=\"x\"", "after")
        );
    });
}

#[test]
fn a2_boolean_attribute_boundaries() {
    with_context(|ctx| {
        for (attrs, expected) in [
            ("active=true", true),
            ("a=false", false),
            ("a=true\tb", true),
        ] {
            let parsed = RawAttributes::parse(ctx, attrs).unwrap();
            assert_eq!(
                parsed.attrs[0].value,
                Some(RawAttributeValue::Literal(RawLiteral::Bool(expected)))
            );
            assert!(ctx.diag.errors().is_empty());
        }
        RawAttributes::parse(ctx, "a=trueish").unwrap();
        assert!(!ctx.diag.errors().is_empty());
    });
}

#[test]
fn a3_skipped_empty_tags_preserve_prefix() {
    with_context(|ctx| {
        for prefix in ["a <> b ", "a </> b ", "é <> </> b ", "a <unclosed> <> b "] {
            let input = format!("{prefix}<c/>");
            let found = Component::find_component(&input, ctx.loc).unwrap().unwrap();
            assert_eq!(found.before, prefix);
            assert_eq!(found.key.name.as_ref(), "comp_c");
        }
    });
}

#[test]
fn a4_malformed_foreign_key_args() {
    for input in [
        "$t(key,",
        "$t(key,é)",
        "$t(key, é)",
        "$t(key, foo)",
        "$t(key, {",
        "$t(key, })",
    ] {
        with_context(|ctx| {
            assert!(RawValue::parse(ctx, input).is_err(), "{input}");
            assert!(ctx.diag.errors().iter().any(|e| matches!(
                e,
                Error::UnexpectedToken { .. } | Error::InvalidForeignKeyArgs { .. }
            )));
        });
    }
}

#[test]
fn a5_foreign_key_namespace_mismatches() {
    let cfg = Config::new("en").unwrap();
    let diag = Diagnostics::new();
    let raw = raw_locales(&cfg, &[r#"{"key":"ok","fk":"$t(key)"}"#], &diag);
    let ns = RawNamespace {
        name: key("ns"),
        locales: raw,
    };
    let parsed = extract_locales(
        RawLocalesOrNamespaces::Namespaces(vec![ns]),
        cfg.clone(),
        Some(diag),
    )
    .unwrap();
    assert!(
        parsed
            .diag
            .errors()
            .iter()
            .any(|e| matches!(e, Error::InvalidForeignKey { .. }))
    );
    let parsed = extract(cfg, &[r#"{"key":"ok","fk":"$t(ns:key)"}"#]);
    assert!(
        parsed
            .diag
            .errors()
            .iter()
            .any(|e| matches!(e, Error::InvalidForeignKey { .. }))
    );
}

#[test]
fn a6_foreign_key_default_cycles() {
    for cyclic in [false, true] {
        let cfg = Config::new("en")
            .unwrap()
            .add_locales(["fr", "de"])
            .unwrap();
        let cfg = if cyclic {
            cfg.extend_locale("fr", "de")
                .unwrap()
                .extend_locale("de", "fr")
                .unwrap()
        } else {
            cfg.extend_locale("fr", "fr").unwrap()
        };
        let parsed = extract(
            cfg,
            &[
                r#"{"key":"base","fk":"$t(key)"}"#,
                r#"{"key":null,"fk":"$t(key)"}"#,
                r#"{"key":null,"fk":"$t(key)"}"#,
            ],
        );
        assert!(parsed.diag.errors().is_empty(), "{:?}", parsed.diag);
        assert_eq!(
            values(&parsed, "key").defaults.default_of(&key("fr")),
            &key("en")
        );
        let Value::Literal(Literal::String(index)) = values(&parsed, "fk").values[&key("fr")]
        else {
            panic!()
        };
        assert_eq!(locales(&parsed).locales[1].strings[index].as_ref(), "base");
        assert_default_coverage(&parsed, "key");
        assert_default_coverage(&parsed, "fk");
    }
}

#[test]
#[cfg(feature = "plurals")]
fn a7_overlapping_plural_rule_types() {
    for source in [
        r#"{"x_one":"a","x_other":"b","x_ordinal_one":"c","x_ordinal_other":"d"}"#,
        r#"{"x_one":"a","x_ordinal_one":"b"}"#,
    ] {
        let parsed = extract(Config::new("en").unwrap(), &[source]);
        assert!(
            parsed
                .diag
                .errors()
                .iter()
                .any(|e| matches!(e, Error::ConflictingPluralRuleType { .. })),
            "{:?}",
            parsed.diag
        );
    }
}

#[test]
fn a8_surplus_values_and_subkeys_are_ignored() {
    let cfg = Config::new("en")
        .unwrap()
        .add_locales(["fr", "de"])
        .unwrap();
    let parsed = extract(
        cfg,
        &[
            r#"{"key":"ok"}"#,
            r#"{"key":"oui","extra":"x","sub":{"x":"y"}}"#,
            r#"{"key":"ja"}"#,
        ],
    );
    assert_eq!(locales(&parsed).keys.values.len(), 1);
    assert_eq!(
        parsed
            .diag
            .warnings()
            .iter()
            .filter(|w| matches!(w, Warning::SurplusKey { .. }))
            .count(),
        2
    );
    assert!(
        !parsed
            .diag
            .warnings()
            .iter()
            .any(|w| matches!(w, Warning::MissingKey { .. }))
    );
}

#[test]
fn a9_implicit_inheritance_and_fk_defaulting() {
    let cfg = Config::new("en")
        .unwrap()
        .add_locales(["fr", "fr-CA", "fr-Latn-CA", "fr-Latn", "de"])
        .unwrap();
    let parsed = extract(
        cfg,
        &[
            r#"{"key":"en","null":"en","fk":"en"}"#,
            r#"{"key":"fr","null":"fr","fk":"fr"}"#,
            r#"{"key":"ca","null":"ca","fk":"ca"}"#,
            r#"{"null":null,"fk":"$t(null)"}"#,
            r#"{"null":null,"fk":"$t(null)"}"#,
            "{}",
        ],
    );
    for (locale, parent) in [("fr-Latn-CA", "fr-CA"), ("fr-Latn", "fr"), ("de", "en")] {
        for name in ["key", "null"] {
            assert_eq!(
                values(&parsed, name).defaults.default_of(&key(locale)),
                &key(parent)
            );
        }
    }
    for (index, expected) in [(3, "ca"), (4, "fr")] {
        let locale = &locales(&parsed).locales[index];
        let Value::Literal(Literal::String(s)) = values(&parsed, "fk").values[&locale.name.key]
        else {
            panic!()
        };
        assert_eq!(locale.strings[s].as_ref(), expected);
    }
    for name in ["key", "null", "fk"] {
        assert_default_coverage(&parsed, name);
    }
    let mut cfg = Config::new("en")
        .unwrap()
        .add_locales([
            "fr",
            "fr-CA",
            "fr-CA-u-nu-latn",
            "fr-CA-1996",
            "fr-Latn-BE",
            "de-AT",
        ])
        .unwrap()
        .extend_locale("fr-CA", "en")
        .unwrap();
    cfg.compute_locale_parents();
    for (locale, parent) in [
        ("fr-CA", "en"),
        ("fr-CA-u-nu-latn", "fr-CA"),
        ("fr-CA-1996", "fr-CA"),
        ("fr-Latn-BE", "fr"),
        ("de-AT", "en"),
        ("fr", "en"),
    ] {
        assert_eq!(cfg.locale_parent(&key(locale)), &key(parent));
    }
}

#[test]
fn a10_missing_warning_requires_direct_default_fallback() {
    let cfg = Config::new("en")
        .unwrap()
        .add_locales(["fr", "fr-CA", "de"])
        .unwrap()
        .extend_locale("de", "en")
        .unwrap();
    let parsed = extract(cfg, &[r#"{"key":"ok"}"#, "{}", "{}", "{}"]);
    let warned: Vec<_> = parsed
        .diag
        .warnings()
        .iter()
        .filter_map(|w| match w {
            Warning::MissingKey { loc } => Some(loc.locale.key.name.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(warned, ["fr"]);
}

#[test]
fn a10_suppress_missing_and_surplus_warnings() {
    let cfg = Config::new("en")
        .unwrap()
        .add_locale("fr")
        .unwrap()
        .parse_options(ParseOptions::new().suppress_key_warnings(true));
    let parsed = extract(
        cfg,
        &[r#"{"key":"ok"}"#, r#"{"extra":"x","sub":{},"null":null}"#],
    );
    assert!(parsed.diag.warnings().is_empty(), "{:?}", parsed.diag);
}

#[test]
fn a11_translation_and_interpolation_ident_collisions() {
    fn assert_unique(keys: &crate::extraction::Keys) {
        let names: BTreeSet<_> = keys.values.keys().map(|k| k.ident.to_string()).collect();
        assert_eq!(names.len(), keys.values.len());
        for value in keys.values.values() {
            if let ValuesOrSubkeys::Subkeys(keys) = value {
                assert_unique(keys);
            }
        }
    }
    for source in [
        r#"{"a-b":"x","a_b":"y"}"#,
        r#"{"sub":{"a-b":{},"a_b":{}}}"#,
        r#"{"key":"{{a-b}} {{a_b}}"}"#,
        r#"{"key":"<a-b/> <a_b/>"}"#,
        r#"{"key":"<b a={{a-b}}/> {{a_b}}"}"#,
    ] {
        let parsed = extract(Config::new("en").unwrap(), &[source]);
        assert_unique(&locales(&parsed).keys);
        let messages: Vec<_> = parsed
            .diag
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            messages
                .iter()
                .any(|s| s.contains("a-b") && s.contains("a_b") && s.contains("identifier")),
            "{messages:?}"
        );
        for builder in parsed.builders.builders.values() {
            let names: BTreeSet<_> = builder
                .keys
                .vars
                .keys()
                .chain(builder.keys.components.keys())
                .map(|k| k.ident.to_string())
                .collect();
            assert_eq!(
                names.len(),
                builder.keys.vars.len() + builder.keys.components.len()
            );
        }
    }
    let parsed = extract(
        Config::new("en").unwrap(),
        &[r#"{"a-b-c":"x","a_b-c":"y","a_b_c":"z"}"#],
    );
    let errors = parsed.diag.errors();
    let Error::IdentCollision { names, .. } = &errors[0] else {
        panic!()
    };
    assert_eq!(names, &["a-b-c", "a_b-c", "a_b_c"]);
    assert_eq!(locales(&parsed).keys.values.len(), 1);
    let parsed = extract(
        Config::new("en").unwrap().add_locale("fr").unwrap(),
        &[r#"{"key":"{{a-b}}"}"#, r#"{"key":"{{a_b}}"}"#],
    );
    assert_eq!(parsed.diag.errors().len(), 1);
    assert!(
        values(&parsed, "key")
            .values
            .values()
            .all(|v| matches!(v, Value::Dummy(_)))
    );
}

#[test]
fn a11_namespace_ident_collisions() {
    let cfg = Config::new("en").unwrap();
    let diag = Diagnostics::new();
    let namespaces = ["a-b", "a_b"]
        .map(|name| RawNamespace {
            name: key(name),
            locales: raw_locales(&cfg, &["{}"], &diag),
        })
        .to_vec();
    let parsed = extract_locales(
        RawLocalesOrNamespaces::Namespaces(namespaces),
        cfg,
        Some(diag),
    )
    .unwrap();
    assert!(!parsed.diag.errors().is_empty());
    let LocalesOrNamespaces::Namespaces(namespaces) = parsed.values else {
        panic!()
    };
    assert_eq!(namespaces.len(), 1);
}

#[test]
fn a12_builder_ident_collisions() {
    let parsed = extract(
        Config::new("en").unwrap(),
        &[r#"{"first":"{{a-b}}","second":"{{a_b}}","third":"<a-b/>","fourth":"<a_b/>"}"#],
    );
    assert!(parsed.diag.errors().is_empty());
    let names: BTreeSet<_> = parsed
        .builders
        .builders
        .values()
        .map(|b| b.name.ident.to_string())
        .collect();
    assert_eq!(names.len(), parsed.builders.builders.len());
    assert!(
        parsed
            .builders
            .builders
            .values()
            .all(|b| !b.name.name.contains('-'))
    );
}
