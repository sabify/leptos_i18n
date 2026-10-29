//! Literal translation values and argument types without interpolations.

use super::{args::Args, builder::ArgsBuilder};

/// Marks a builder for a translation requiring no interpolation arguments.
pub trait ConstArgsMarker: ArgsBuilder {
    /// The completed builder type used to check that no arguments are required.
    type Builded;
    /// The literal value type returned by the translation.
    type Value: LiteralValue;
    /// The argument type for the literal translation.
    type Args: ConstArgs<Locale = Self::Locale, Id = Self::Id, Value = Self::Value>;
}

/// Arguments for a literal translation without interpolations.
pub trait ConstArgs: Args + Copy + 'static {
    /// The argument value used for translations without interpolations.
    const THIS: Self;
    /// The literal value type returned by the translation.
    type Value: LiteralValue;

    /// Read the literal value for a key and locale at runtime.
    fn value(id: Self::Id, locale: Self::Locale) -> Self::Value;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum NoRecurse {}

/// Marker for a translation without interpolation arguments.
pub enum NoArgs {}

#[doc(hidden)]
pub trait LiteralValue: Copy + 'static {}

/// A translation literal, including values whose types differ between locales.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Literal<M = &'static [Literal<NoRecurse>]>
where
    M: LiteralValue,
{
    /// A string literal.
    String(&'static str),
    /// A signed integer literal.
    Signed(i64),
    /// An unsigned integer literal.
    Unsigned(u64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal.
    Bool(bool),
    /// A sequence of literal values.
    Multiple(M),
}

impl LiteralValue for bool {}
impl LiteralValue for i64 {}
impl LiteralValue for u64 {}
impl LiteralValue for f64 {}
impl LiteralValue for &'static str {}
impl LiteralValue for &'static [Literal<NoRecurse>] {}
impl LiteralValue for Literal {}
impl LiteralValue for NoRecurse {}

impl<M: LiteralValue> Literal<M> {
    /// Return the string value, or `None` for another variant.
    pub const fn str(self) -> Option<&'static str> {
        if let Literal::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the signed integer value, or `None` for another variant.
    pub const fn signed(self) -> Option<i64> {
        if let Literal::Signed(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the unsigned integer value, or `None` for another variant.
    pub const fn unsigned(self) -> Option<u64> {
        if let Literal::Unsigned(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the floating-point value, or `None` for another variant.
    pub const fn float(self) -> Option<f64> {
        if let Literal::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the boolean value, or `None` for another variant.
    pub const fn bool(self) -> Option<bool> {
        if let Literal::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the sequence of literals, or `None` for another variant.
    pub const fn multiple(self) -> Option<M> {
        if let Literal::Multiple(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
