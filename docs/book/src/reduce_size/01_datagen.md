# ICU4X Datagen

This library uses ICU4X as a backend for formatters and plurals, and the default baked data provider can take quite a lot of space as it contains information for _every possible locale_. So if you use only a few, this is a complete waste.

## Disable Compiled Data

The first step to remove this excess information is to disable the default data provider. It is activated by the `"icu_compiled_data"` feature, which is enabled by default. So turn off default features or remove this feature.

## Custom Provider

Great, we lost a lot of size, but now instead of having too much information, we have 0 information. You will now need to bring your own data provider. For that, you will need multiple things.

## 1. Datagen

First, generate the information; you can use [`icu_datagen`](https://docs.rs/crate/icu4x-datagen/2.4.0) for that, either as a [CLI](https://github.com/unicode-org/icu4x/blob/main/tutorials/data-management.md#2-generating-data) or with a [build.rs](https://github.com/unicode-org/icu4x/blob/main/tutorials/cargo.md) (we will come back to it later).

## 2. Load

Then you need to load those informations; this is as simple as

```rust,ignore
use icu::locale as icu_locale_core;

include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));

pub struct MyDataProvider;
impl_data_provider!(MyDataProvider);
```

you will also need some dependencies:

```toml
[dependencies]
leptos_i18n = { version = "0.6", default-features = false, features = ["csr", "all_formats"] }
# "default-features = false" to turn off compiled_data
icu = { version = "2.4", default-features = false }
icu_provider = { version = "2.4", features = ["baked"] } # for databake
zerovec = "0.11" # for databake
```

This enables the CSR example's formatters. Keep only the formatting features your translations need.

ICU 2.4's baked calendar data refers to `icu_locale_core` by name. The example uses the `icu::locale` alias above, so no separate `icu_locale_core` dependency is needed.

This is explained more in depth in the `icu_datagen` doc.

## 3. Supply to leptos_i18n the Provider.

You now just need to tell `leptos_i18n` what provider to use. For that, you first need to implement `IcuDataProvider` for your provider. You can do it manually as it is straightforward, but the lib comes with a derive macro:

```rust,ignore
use icu::locale as icu_locale_core;

include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[derive(leptos_i18n::custom_provider::IcuDataProvider)]
pub struct MyDataProvider;
impl_data_provider!(MyDataProvider);
```

And then pass it to the `set_icu_data_provider` function when the program starts,
so for CSR apps in the main function:

```rust,ignore
fn main() {
    use app::App;
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App /> })
}
```

and for SSR apps in both on hydrate and on server startup:

```rust,ignore
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

```rust,ignore
// example for actix
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    // ..
    Ok(())
}
```

## Build.rs Datagen

The doc for ICU4X datagen can be quite intimidating, but it is actually quite straightforward. Your build.rs can look like this:

For this manual approach, add these build dependencies:

```toml
[build-dependencies]
icu = { version = "2.4", default-features = false, features = ["datagen"] }
icu_provider_export = { version = "2.4", default-features = false, features = ["baked_exporter"] }
icu_provider_source = "2.4"
```

```rust,ignore
use icu::locale::LocaleFallbacker;
use icu_provider_export::{
    baked_exporter::{self, BakedExporter},
    DataLocaleFamily, DeduplicationStrategy, ExportDriver,
};
use icu_provider_source::SourceDataProvider;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    let exporter = BakedExporter::new(mod_directory, {
        let mut options = baked_exporter::Options::default();
        options.overwrite = true;
        options.use_internal_fallback = false;
        options
    })
    .unwrap();

    ExportDriver::new(
        ["en".parse::<DataLocaleFamily>().unwrap(), "fr".parse().unwrap()],
        DeduplicationStrategy::None.into(),
        LocaleFallbacker::new_without_data(),
    )
    .with_markers(icu::plurals::provider::MARKERS.iter().copied())
    .export(&SourceDataProvider::new(), exporter)
    .unwrap();
}
```

Here we are generating the information for locales `"en"` and `"fr"`, with the data needed for plurals.
For this manual example, enable only `plurals` as a formatting feature in `leptos_i18n`, or add the markers needed for your other enabled formatters.

## Using `leptos_i18n_build` Crate

You can use the `leptos_i18n_build` crate that contains utils for the datagen.
The problem with the above `build.rs` is that it can go out of sync with your translations,
when all information is already in the translations.

The build dependency below matches the CSR example; omit its `csr` feature for SSR/hydration builds.

```toml
# Cargo.toml
[build-dependencies]
leptos_i18n_build = { version = "0.6", features = ["pretty_print", "csr"] }
```

```rust,ignore
use leptos_i18n_build::{TranslationsInfos, Config};
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");
    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let cfg = Config::new("en").unwrap().add_locale("fr").unwrap();

    let translations_infos = TranslationsInfos::parse(cfg).unwrap();

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_data(mod_directory).unwrap();
    translations_infos.generate_i18n_module(i18n_mod_directory).unwrap();
}
```

This will parse the config and the translations and generate the data for you using the information gained when parsing the translations.
This will trigger a rerun if the config or translations changed and be kept in sync.
If your code uses plurals, it will build with information for plurals. If it uses a formatter, it will build with the information for that formatter.

If you use more data, for example through `t*_format!` with a formatter not used in the translations, replace `generate_data` above with one of the following calls. Examples using ICU markers also need the `icu` build dependency shown in the manual setup.

```rust,ignore
use leptos_i18n_build::FormatterOptions;

translations_infos.generate_data_with_options(mod_directory, [FormatterOptions::FormatDateTime]).unwrap();
```

This will inject the ICU `DataMarker`s needed for the `date`, `time`, and `datetime` formatters.

```rust,ignore
translations_infos.generate_data_with_data_markers(
    mod_directory,
    icu::plurals::provider::MARKERS.iter().copied()
).unwrap();
```

This will inject the keys for cardinal and ordinal plurals.

If you need both, `FormatterOptions` can be turned into the needed markers:

```rust,ignore
use leptos_i18n_build::FormatterOptions;

let mut markers = icu::plurals::provider::MARKERS.to_vec();
markers.extend(FormatterOptions::FormatDateTime.into_data_markers());

// markers now contains the `DataMarker`s needed for plurals and for the `time`, `date` and `datetime` formatters.

translations_infos.generate_data_with_data_markers(mod_directory, markers).unwrap();
```

## Is It Worth the Trouble?

YES. With `opt-level = "z"` and `lto = true`, the plurals example is at 394 kB (at the time of writing). Now, by just providing a custom provider tailored to the used locales ("en" and "fr"), it shrinks down to 248 kB! It almost cut in half the binary size!
I highly suggest taking the time to implement this.

# Experimental Features

When using experimental features, such as "format_currency", if you follow the step above you will probably have some compilation error in the `impl_data_provider!` macro.
To solve them you will need those few things:

### Enable the `unstable` Feature

Enable the "unstable" feature for `icu`:

```toml
# Cargo.toml
[dependencies]
icu = { version = "2.4", default-features = false, features = ["unstable"] }
```

### Import `icu_pattern`

```toml
# Cargo.toml
[dependencies]
icu_pattern = "0.5" # for databake
```

### Import the `alloc` Crate

The macro directly uses the `alloc` crate instead of std, so you must bring it into scope:

```rust,ignore
extern crate alloc;

use icu::locale as icu_locale_core;

include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));

#[derive(leptos_i18n::custom_provider::IcuDataProvider)]
pub struct MyDataProvider;
impl_data_provider!(MyDataProvider);
```

## Example

You can take a look at the `counter_icu_datagen` example. This is a copy of the `counter_plurals` example but with a custom provider.
