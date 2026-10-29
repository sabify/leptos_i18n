[![crates.io](https://img.shields.io/crates/v/leptos_i18n.svg)](https://crates.io/crates/leptos_i18n)

[Docs.rs](https://docs.rs/leptos_i18n/latest/leptos_i18n/) | [Book](https://baptistemontan.github.io/leptos_i18n)

# Leptos i18n

This crate is made to simplify internationalization in a [Leptos](https://crates.io/crates/leptos) application, that loads locales at **_compile time_** and provides compile time checks for translation keys, interpolation keys and the selected locale.

The main focus is ease of use with leptos, a typical component using this crate will look like this:

```rust
use crate::i18n::*;
use leptos::prelude::*;

#[component]
fn Counter() -> impl IntoView {
  let i18n = use_i18n();

  let (counter, set_counter) = signal(0);
  let inc = move |_| set_counter.update(|count| *count += 1);


  view! {
    <button on:click=inc>
      {t!(i18n, click_to_inc)}
    </button>
    <p>
      {t!(i18n, click_count, count = move || counter.get())}
    </p>
  }
}
```

## Getting started

You can add the crate to your project with

```bash
cargo add leptos_i18n
```

Or by adding this line to your `Cargo.toml` under `[dependencies]`:

```toml
leptos_i18n = "0.6"
```

## Version compatibility with leptos

| Leptos     | Leptos i18n         |
| ---------- | ------------------- |
| `< v0.4.x` | not supported       |
| `v0.4.x`   | `v0.1.x`            |
| `v0.5.x`   | `v0.2.x`            |
| `v0.6.x`   | `v0.3.x` / `v0.4.x` |
| `v0.7.x`   | `v0.5.x`            |
| `v0.8.x`   | `v0.6.x`            |

## How to use

Generate the `i18n` module in `build.rs` with `leptos_i18n_build` as a build dependency:

```rust
use leptos_i18n_build::{Config, TranslationsInfos};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    let cfg = Config::new("en")?.add_locale("fr")?;
    let translations = TranslationsInfos::parse(cfg)?;
    translations.emit_diagnostics();
    translations.rerun_if_locales_changed();
    let directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");
    translations.generate_i18n_module(directory)?;
    Ok(())
}
```

Place translations in `locales/en.json` and `locales/fr.json`, then include the generated module in your application:

```rust
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
```

You can look into the [Book](https://baptistemontan.github.io/leptos_i18n) for documentation, or look for [examples](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples) on the github repo.
