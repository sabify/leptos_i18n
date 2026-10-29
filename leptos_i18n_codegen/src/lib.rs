#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
//! # About Leptos i18n codegen
//!
//! This crate expose the codegen functions for `leptos_i18n`
//!
//! This crate must be used with `leptos_i18n` and should'nt be used outside of it.
//!
//! See [`CodegenOptions`] for the available generation settings.

use leptos_i18n_parser::{error::Result, extraction::ParsedLocales};

use proc_macro2::TokenStream;

mod codegen;
mod options;
pub mod utils;

/// Options controlling the generated translation module.
///
/// - `top_level_attributes`: module attributes, with none added by default.
/// - `module_file_name`: output file name, defaulting to `"mod.rs"`.
/// - `crate_path`: path to `leptos_i18n` when accessed through a reexport.
/// - `gen_docs`: generate documentation for locales, keys, and arguments; defaults to `true`.
/// - `translations_uri`: URI template for CSR dynamic loading, using `{locale}`
///   and optionally `{namespace}` placeholders.
/// - `show_keys_only`: render key paths while preserving argument checks and numeric
///   and boolean literal types; defaults to `false`.
/// - `interpolate_display`: retained for compatibility and has no effect;
///   display implementations are always generated.
pub use options::CodegenOptions;

/// Generate the translation module from parsed locales and codegen options.
pub fn gen_code(parsed_locales: &ParsedLocales, options: CodegenOptions) -> Result<TokenStream> {
    codegen::gen_code(parsed_locales, options)
}
