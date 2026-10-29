use crate::I18nContext;
use std::fmt::Display;

crate::declare_locales! {
    path: crate,
    default: "en",
    locales: ["en", "fr"],
    en: {
        plain: "Hello",
        greeting: "Hello {{ name }}",
    },
    fr: {
        plain: "Bonjour",
        greeting: "Bonjour {{ name }}",
    },
}

use i18n::Locale;

fn assert_string(value: String, expected: &str) {
    assert_eq!(value, expected);
}

fn assert_display(value: impl Display, expected: &str) {
    assert_eq!(value.to_string(), expected);
}

async fn check_macros(i18n: I18nContext<Locale>, locale: Locale, plain: &str, greeting: &str) {
    assert_string(crate::t_string!(i18n, plain).await, plain);
    assert_string(crate::td_string!(locale, plain).await, plain);
    assert_string(crate::tu_string!(i18n, plain).await, plain);
    assert_display(crate::t_display!(i18n, plain).await, plain);
    assert_display(crate::td_display!(locale, plain).await, plain);
    assert_display(crate::tu_display!(i18n, plain).await, plain);

    let name = String::from("Ada");
    assert_string(
        crate::t_string!(i18n, greeting, name = &name).await,
        greeting,
    );
    assert_string(
        crate::td_string!(locale, greeting, name = &name).await,
        greeting,
    );
    assert_string(
        crate::tu_string!(i18n, greeting, name = &name).await,
        greeting,
    );
    assert_display(
        crate::t_display!(i18n, greeting, name = &name).await,
        greeting,
    );
    assert_display(
        crate::td_display!(locale, greeting, name = &name).await,
        greeting,
    );
    assert_display(
        crate::tu_display!(i18n, greeting, name = &name).await,
        greeting,
    );
}

#[test]
fn dynamic_macros_compile() {
    // Type-check the async body without fetching translations on the client.
    let _ = check_macros;
}

#[cfg(feature = "ssr")]
#[test]
fn dynamic_macros_render_server_translations() {
    use crate::context::{I18nContextOptions, init_i18n_context_with_options};
    use leptos::prelude::Owner;

    let owner = Owner::new();
    owner.with(|| {
        let options = I18nContextOptions::default().enable_cookie(false);
        let i18n = init_i18n_context_with_options::<Locale>(options);
        i18n.set_locale(Locale::en);
        futures::executor::block_on(check_macros(i18n, Locale::en, "Hello", "Hello Ada"));
        i18n.set_locale(Locale::fr);
        futures::executor::block_on(check_macros(i18n, Locale::fr, "Bonjour", "Bonjour Ada"));
    });
}
