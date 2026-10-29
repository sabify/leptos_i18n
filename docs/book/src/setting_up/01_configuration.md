# Load The Translations

To load the translations we need codegen, and for that you can use the `leptos_i18n_build` package.

You use it with a `build.rs` file to generate the code to properly use you translations:

```rust,ignore
// build.rs

use leptos_i18n_build::{TranslationsInfos, Config};
use std::path::PathBuf;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=Cargo.toml");

  // where to generate the translations
  let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

  let cfg = Config::new("en")?.add_locale("fr")?; // "en" is the default locale, "fr" is another locale.

  let translations_infos = TranslationsInfos::parse(cfg)?;

  // emit the errors and warnings found during parsing
  translations_infos.emit_diagnostics();

  // emit "cargo::rerun-if-changed" for every translation file
  translations_infos.rerun_if_locales_changed();

  // codegen
  translations_infos.generate_i18n_module(i18n_mod_directory)?;

  Ok(())
}
```

## The `i18n` Module

You can then import the generated code with:

```rust, ignore
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
```

This will include a module called `i18n`. This module contains everything you need to use your translations.

```rust, ignore
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
use i18n::*;
```

# Configuration

This crate is basically entirely based around the code generated in a build script. We will cover it in a later chapter, but for now just know that it looks at your translation files and generates code for them.

To load those translations it first needs to know what to look for, so you need to declare what locales you are supporting and which one is the default.
To do that you use the `Config` builder:

To declare `en` and `fr` as locales, with `en` being the default you would write:

```rust
let cfg = Config::new("en")?.add_locale("fr")?;
```

There are more optional values you can supply:

- `add_namespace`: This is to split your translations into multiple files, we will cover it in a later chapter
- `locales_path`: This is to have a custom path to the directory containing the locales files, it defaults to `"./locales"`.
- `extend_locale`: Allows you to describe the inheritance structure for locales, covered in a later chapter.
- `parse_options`: Parsing options, covered in the next segment

Once this configuration is done, you can start writing your translations.

## Parsing Options

`Config` can take some options as an argument, for now we use the default but you can import the `ParseOptions` struct to tell the parser what to expect and produce, here we change the file format to `yaml`:

```rust, ignore
use leptos_i18n_build::{Config, TranslationsInfos, options::{FileFormat, ParseOptions}};
use std::path::PathBuf;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = ParseOptions::default().file_format(FileFormat::Yaml);

    let cfg = Config::new("en")?.add_locale("fr")?.parse_options(options);

    let translations_infos = TranslationsInfos::parse(cfg)?;

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_i18n_module(i18n_mod_directory)?;

    Ok(())
}
```

There are other options:

- `file_format`: Choose `Json` (the default), `Json5`, `Yaml`, or `Toml`.
- `suppress_key_warnings`: Suppress missing-key and surplus-key warnings; defaults to `false`.
- `add_formatter`: Register a custom formatter. See the [custom formatter appendix](../appendix_custom_formatter.md).
- `with_custom_parser`: Use a custom implementation of `leptos_i18n_parser::options::parser::Parser` instead of a built-in file format.

example:

```rust, ignore
let options = ParseOptions::default()
  .file_format(FileFormat::Json5)
  .suppress_key_warnings(true);
```

There is also a way to inject your own formatter, this needs its own chapter, which you can find in an appendix.

## Codegen Options

`TranslationsInfos::generate_i18n_module_with_options` takes an output directory and a `CodegenOptions` argument:

- `top_level_attributes`: Add attributes to the generated module.
- `module_file_name`: Set the generated file name; defaults to `"mod.rs"`.
- `crate_path`: Set the path to `leptos_i18n` if it is accessed through a reexport.
- `gen_docs`: Generate documentation for locales, namespaces, subkeys, keys, and interpolation arguments; defaults to `true`.
- `translations_uri`: Set the URI template for a CSR application with `dynamic_load`. See [dynamic loading](../reduce_size/02_dynamic_load.md).
- `show_keys_only`: Render translation key paths instead of their text, useful for finding untranslated strings; defaults to `false`. Arguments are still checked, and numeric and boolean literal values keep their types.
- `interpolate_display`: Retained for compatibility and has no effect. Display implementations are always generated, so string and display macros support interpolations without an option.

example:

```rust, ignore
use leptos_i18n_build::options::CodegenOptions;

let attributes = "#![allow(missing_docs)]".parse()?;

let options = CodegenOptions::default()
  .top_level_attributes(Some(attributes))
  .module_file_name(&"i18n.rs") // "mod.rs" by default
  .gen_docs(true) // `true` by default
  .show_keys_only(false);

translations_infos.generate_i18n_module_with_options(i18n_mod_directory, options)?;
```

The later (`gen_docs`) is purely for UX ergonomics, disable it if you don't use intellisense or for CI runs
