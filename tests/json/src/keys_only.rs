include!(concat!(env!("OUT_DIR"), "/keys_only/mod.rs"));

use self::i18n::*;
use leptos_i18n::keys::comp_time::Literal;
use tests_common::*;

#[test]
fn renders_keys_and_accepts_arguments() {
    for locale in [Locale::en, Locale::fr] {
        assert_eq_rendered!(td!(locale, click_count, count = 12), "click_count");
        assert_eq!(td_string!(locale, click_count, count = 12), "click_count");
        assert_eq_rendered!(td!(locale, subkeys.subkey_1), "subkeys.subkey_1");
        assert_eq_rendered!(
            td!(locale, interpolate_variable_and_comp_self_closed, <br/> = <br/>),
            "interpolate_variable_and_comp_self_closed"
        );
        assert_eq!(
            td_string!(locale, interpolate_variable_and_comp_self_closed, <br/> = "br"),
            "interpolate_variable_and_comp_self_closed"
        );
        assert_eq!(td_string!(locale, mixed_lit_type), "mixed_lit_type");
        assert_eq_rendered!(td!(locale, same_lit_type), "same_lit_type");
    }
}

#[test]
fn const_values_preserve_types() {
    assert_eq!(
        i18n::keys::click_to_inc::__const_value(Locale::en), "click_to_inc"
    );
    assert_eq!(
        i18n::keys::mixed_lit_type::__const_value(Locale::fr),
        Literal::String("mixed_lit_type")
    );
    assert_eq!(
        i18n::keys::populated_foreign_key::__const_value(Locale::en),
        &[Literal::String("populated_foreign_key")]
    );
    assert!(i18n::keys::same_lit_type::__const_value(Locale::en));
    assert!(!i18n::keys::same_lit_type::__const_value(Locale::fr));
}
