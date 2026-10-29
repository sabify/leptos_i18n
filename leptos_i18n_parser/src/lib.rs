#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments, clippy::result_unit_err)]

pub mod error;
mod extractor;
pub mod formatters;
mod parser;
#[cfg(test)]
mod regressions;
pub mod utils;

pub mod options {
    pub use super::parser::options::{Config, FileFormat, LocaleName, ParseOptions, parser};
}

pub mod parsing {
    pub use super::parser::{
        ParseContext, RawParsedLocales,
        locale::{RawLocale, RawLocalesOrNamespaces, RawValueOrSubkeys, RawValues},
        parse_locales_raw,
        raw_value::{RawValue, component::Component, variable::Variable},
    };
}

pub mod extraction {
    pub use super::extractor::{
        Builder, BuilderId, Builders, ChildrenKind, CompInfos, InterpolationKeys, Locale, Locales,
        LocalesOrNamespaces, Namespace, ParsedLocales, VarInfos,
        defaults::DefaultedLocales,
        extract_locales,
        values::{
            Keys, Literal, Value, Values, ValuesOrSubkeys,
            attributes::{Attribute, AttributeValue, Attributes},
            plurals::{PluralForm, PluralForms, PluralRuleType, Plurals},
        },
    };
}
