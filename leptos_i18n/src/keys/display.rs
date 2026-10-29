//! Translation arguments and keys used for display formatting.

use crate::locale_traits::BaseLocale;

use super::Key;
use super::args::{AnyArgsId, AnyArgsInner, Args};
use super::builder::ArgsBuilder;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::Arc;

/// Formats a translation with its interpolation arguments.
pub trait DisplayArgs: Args {
    /// Translation data needed for formatting in a selected locale.
    type Data;

    /// Obtain the translation data for a key and locale.
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, id: Self::Id, locale: Self::Locale) -> Self::Data;

    /// Load the translation data for a key and locale.
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data(&self, id: Self::Id, locale: Self::Locale) -> impl Future<Output = Self::Data>;

    /// Format the translation using the supplied locale and translation data.
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        id: Self::Id,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result;
}

/// Converts display arguments to a representation shared by compatible keys.
pub trait DowngradableDisplayArgs: DisplayArgs {
    /// The shared display argument representation.
    type Downgraded: DisplayArgs<Locale = Self::Locale, Data = Self::Data>;

    /// Convert a key while preserving its display formatting.
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

/// Converts a completed generated builder into display arguments.
pub trait DisplayArgsMarker<B>: ArgsBuilder {
    /// The display arguments produced by the builder.
    type Args: DisplayArgs<Locale = Self::Locale, Id = Self::Id>;
    /// Convert the completed builder into display arguments.
    fn into_args(builder: B) -> Self::Args;
}

/// Creates the generated builder for display interpolation arguments.
pub trait DisplayArgsBuilder: ArgsBuilder {
    /// The builder accepting display interpolation arguments.
    type DisplayBuilder;

    /// Create a display builder with no interpolation arguments supplied.
    fn new_display() -> Self::DisplayBuilder;
}

/// A translation key, locale, arguments, and data ready for display formatting.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayKey<A: DisplayArgs> {
    pub(crate) id: A::Id,
    pub(crate) locale: A::Locale,
    pub(crate) args: A,
    pub(crate) data: A::Data,
}

#[doc(hidden)]
#[cfg(not(feature = "dynamic_load"))]
pub type DisplayData = ();

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub type DisplayData = &'static [&'static str];

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub type DisplayData = &'static [Box<str>];

/// Display arguments with erased concrete types and a shared translation data type.
pub struct AnyDisplayArgs<'a, L: BaseLocale, Data = DisplayData> {
    args: Arc<dyn DynAnyDisplayArgs<'a, Locale = L, Data = Data>>,
}

impl<L: BaseLocale, Data> Clone for AnyDisplayArgs<'_, L, Data> {
    fn clone(&self) -> Self {
        AnyDisplayArgs {
            args: self.args.clone(),
        }
    }
}

impl<L: BaseLocale, Data> Args for AnyDisplayArgs<'_, L, Data> {
    type Id = AnyArgsId;
    type Locale = L;
}

impl<'a, L, Data> DisplayArgs for AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a,
{
    type Data = Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> Self::Data {
        DynAnyDisplayArgs::get_data(&*self.args, locale)
    }

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> impl Future<Output = Self::Data> {
        DynAnyDisplayArgs::get_data(&*self.args, locale)
    }

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        _id: Self::Id,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result {
        DynAnyDisplayArgs::fmt(&*self.args, formatter, locale, data)
    }
}

impl<'a, L, Data> DowngradableDisplayArgs for AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a,
{
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<'a, L, Data> AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a + Send + Sync,
{
    /// Erase the concrete argument and identifier types for display formatting.
    pub fn from_args<A>(args: A, id: A::Id) -> Self
    where
        A: DisplayArgs<Locale = L, Data = Data> + Clone + Send + Sync + 'a,
    {
        let inner = AnyArgsInner {
            id,
            args,
            data_marker: PhantomData,
        };
        let boxed = Arc::new(inner);
        AnyDisplayArgs { args: boxed }
    }
}

impl<A: DisplayArgs> Debug for DisplayKey<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisplayKey")
            .field("key", &super::KeyId::key(self.id))
            .finish()
    }
}

impl<A: DisplayArgs> Display for DisplayKey<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        A::fmt(&self.args, f, self.id, self.locale, &self.data)
    }
}

trait DynAnyDisplayArgs<'a>: Send + Sync + 'a {
    type Locale: BaseLocale;
    type Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, locale: Self::Locale) -> Self::Data;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data<'b>(
        &'b self,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = Self::Data> + 'b>>;

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result;
}

impl<'a, A: DisplayArgs + Clone + Send + Sync + 'a> DynAnyDisplayArgs<'a>
    for AnyArgsInner<A, A::Data>
where
    A: DisplayArgs + Clone + Send + Sync + 'a,
    A::Data: Send + Sync + 'a,
{
    type Locale = A::Locale;
    type Data = A::Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, locale: Self::Locale) -> Self::Data {
        A::get_data(&self.args, self.id, locale)
    }

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data<'b>(
        &'b self,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = Self::Data> + 'b>> {
        Box::pin(A::get_data(&self.args, self.id, locale))
    }

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result {
        A::fmt(&self.args, formatter, self.id, locale, data)
    }
}
