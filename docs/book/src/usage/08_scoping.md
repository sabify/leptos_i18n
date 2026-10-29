# Scoping

If you are using subkeys or namespaces, access keys can get pretty big and repetitive,
wouldn't it be nice to scope a context to a namespace or subkeys?

Well, this page explains how to do it!

## The `scope_i18n!` Macro

Using namespaces and subkeys can make things quite cumbersome very fast. Imagine you have this:

```rust,ignore
let i18n = use_i18n();

t!(i18n, namespace.subkeys.value);
t!(i18n, namespace.subkeys.more_subkeys.subvalue);
t!(i18n, namespace.subkeys.more_subkeys.another_subvalue);
```

This only uses `namespace.subkeys.*`, but we have to repeat it everywhere.
Here comes the `scope_i18n!` macro. You can rewrite it to:

```rust,ignore
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

This macro can be chained:

```rust,ignore
let i18n = use_i18n();
let i18n = scope_i18n!(i18n, namespace);
let i18n = scope_i18n!(i18n, subkeys);

t!(i18n, value);

let i18n = scope_i18n!(i18n, more_subkeys);
t!(i18n, subvalue);
t!(i18n, another_subvalue);
```

## The `use_i18n_scoped!` Macro

In the above example, we do `let i18n = use_i18n();` but only access the context to scope it afterward. We could do

```rust,ignore
let i18n = scope_i18n!(use_i18n(), namespace.subkeys);
```

Well, this is what the `use_i18n_scoped!` macro is for:

```rust,ignore
let i18n = use_i18n_scoped!(namespace.subkeys);

t!(i18n, value);
t!(i18n, more_subkeys.subvalue);
t!(i18n, more_subkeys.another_subvalue);
```

## The `scope_locale!` Macro

The above examples are to scope a context, but maybe you use `td!` a lot and run into the same problems:

```rust,ignore
fn foo(locale: Locale) {
    td!(locale, namespace.subkeys.value);
    td!(locale, namespace.subkeys.more_subkeys.subvalue);
    td!(locale, namespace.subkeys.more_subkeys.another_subvalue);
}
```

You can use the `scope_locale!` macro here:

```rust,ignore
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    td!(locale, more_subkeys.subvalue);
    td!(locale, more_subkeys.another_subvalue);
}
```

And again, it is chainable:

```rust,ignore
fn foo(locale: Locale) {
    let locale = scope_locale!(locale, namespace.subkeys);
    td!(locale, value);
    let locale = scope_locale!(locale, more_subkeys);
    td!(locale, subvalue);
    td!(locale, another_subvalue);
}
```

## The `define_scope!` Macro

Define a scope type alias by passing an existing scope type and a key path. Use the generated `Locale` type for the root scope:

```rust, ignore
type SubkeysScope = define_scope!(Locale, namespace.subkeys);

fn foo(locale: Locale) {
    let locale = locale.scope::<SubkeysScope>();
    td!(locale, value);

    let locale = scope_locale!(locale, more_subkeys);
    td!(locale, subvalue);
    td!(locale, another_subvalue);
}

fn bar() -> impl IntoView {
    let i18n = use_i18n_scoped!(scope = SubkeysScope);
    // equivalent to `use_i18n().scope::<SubkeysScope>()`

    t!(i18n, value)
}
```

Scopes can be extended by passing a previously defined scope type:

```rust,ignore
type MoreSubkeysScope = define_scope!(SubkeysScope, more_subkeys);
```

An optional third argument specifies the path to the `leptos_i18n` crate when needed, for example `define_scope!(Locale, namespace.subkeys, leptos_i18n)`.

This is useful when you have namespaces/subkeys for large parts of your frontend and need types to reuse. `I18nContext` takes one generic parameter, the scope:

```rust, ignore
type SubkeysScope = define_scope!(Locale, namespace.subkeys);

type SubkeysCtx = I18nContext<SubkeysScope>;

let i18n: SubkeysCtx = use_i18n_scoped!(scope = SubkeysScope);
```

For a locale in a const context, use `ScopedLocale::new`:

```rust, ignore
use leptos_i18n::ScopedLocale;

const FR_SUBKEYS: ScopedLocale<SubkeysScope> = ScopedLocale::new(Locale::fr);
```

## How It Works

Each generated keys struct implements `Scope`, which associates it with its base locale and available keys. `define_scope!` selects the nested keys type and checks that the path exists.

`I18nContext<Locale>` uses the root scope. `I18nContext<SubkeysScope>` uses the same locale signal but limits key access to that scope. Translation macros navigate the generated keys structs; the generated `i18n::keys` module contains the corresponding key modules.

Scoping changes which keys are available without creating a separate locale signal. A `ScopedLocale<SubkeysScope>` similarly pairs the base locale with the scope type.
