# Dynamic Loading of Translations

## Why Use It?

By default, the translations are loaded at compile time and are baked into the binary.
This has some performance advantages but comes at a cost: binary size.
This is fine when the number of keys and locales is small and the values are not long,
but when supporting many locales and a lot of keys, binary size can increase significantly.

The `"dynamic_load"` feature reduces this binary size increase by removing the baked translations in the binary and lazy loading them on the client.
The way it does that is by using a server function to request the translations in a given "translation unit".
What I call "translation unit" is a group of translations; they are either one unit per locale or one unit per locale per namespaces if you use them.

## How It Works

When using SSR, the server will register every unit used for a given request and bake only the used one in the sent HTML.
They are then parsed when the client hydrates, so no request for translations is done on page load.
When the client needs access to an unloaded unit, it will request it from the server and will update the view when received.

## What Changes?

### Async Accessors

With `dynamic_load`, accessing translations can require asynchronous loading. The view macros `t!`, `td!`, and `tu!` handle this internally and produce Leptos views.

The `*_display!` and `*_string!` variants return futures that must be awaited. Awaiting a string macro produces a `String`; awaiting a display macro produces a value implementing `Display`.

You can expose a loaded string through Leptos `AsyncDerived`:

```rust,ignore
let i18n = use_i18n();
let translation = AsyncDerived::new(move || t_string!(i18n, key)); // .get() returns Option<String>
```

You can also wrap this pattern in a macro:

```rust,ignore
macro_rules! t_string_async {
    ($($args:tt)*) => {
        leptos::prelude::AsyncDerived::new(move || leptos_i18n::t_string!($($args)*))
    };
}
```

These macros return futures on the server too, even though server translations are baked in. This keeps the same async API on the client and server.

### Server Fn

If you use a backend that needs to manually register server functions,
you can use the `ServerFn` associated type on the `BaseLocale` trait implemented by the generated `Locale` enum:

```rust,ignore
use i18n::Locale;
use leptos_i18n::locale_traits::BaseLocale;

register_server_fn::<<Locale as BaseLocale>::ServerFn>();
```

### CSR

With SSR the translations are served by a server functions, but they don't exist with CSR, so you will need to create a static JSON containing them, so a bit more work is needed.
To do that you can use a build script and use the `leptos_i18n_build` crate:

```toml
# Cargo.toml
[build-dependencies]
leptos_i18n_build = "0.6.0"
```

```rust
use leptos_i18n_build::{Config, TranslationsInfos, options::CodegenOptions};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let cfg = Config::new("en")?.add_locale("fr")?;
    let options = CodegenOptions::default().translations_uri("i18n/{locale}.json");
    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");
    let translations_infos = TranslationsInfos::parse(cfg)?;

    translations_infos.emit_diagnostics();
    translations_infos.rerun_if_locales_changed();
    translations_infos.generate_i18n_module_with_options(i18n_mod_directory, options)?;

    translations_infos
        .get_translations()
        .write_to_dir("path/to/dir")?;

    Ok(())
}
```

This will generate the necessary JSON files in the given directory. For example, you could generate them in `target/i18n`, resulting in this file structure:

```bash
./target
└── i18n
    ├── locale1.json
    └── locale2.json
```

If you are using namespaces it would have this one:

```bash
./target
└── i18n
    └── namespace1
        ├── locale1.json
        └── locale2.json
    └── namespace2
        ├── locale1.json
        └── locale2.json
```

Then if you are using Trunk you just have to add the directory to the build pipeline:

```html
<!-- index.html -->
<!DOCTYPE html>
<html>
  <head>
    <link data-trunk rel="copy-dir" href="./target/i18n" />
  </head>
  <body></body>
</html>
```

Now the translations will be available at `i18n/{locale}.json`.
To inform `leptos_i18n` where to find those translations, set `CodegenOptions::translations_uri` before generating the module, as in the build script above:

```rust
let options = CodegenOptions::default().translations_uri("i18n/{locale}.json");
// With namespaces, use "i18n/{namespace}/{locale}.json" instead.
translations_infos.generate_i18n_module_with_options(i18n_mod_directory, options)?;
```

And this is it!

## Disclaimers

1.  There is a chance that enabling this feature actually increases binary sizes if there aren’t many translations,
    as there is additional code being generated to request, parse, and load the translations. But this is mostly a fixed cost,
    so with enough translations, so with enough translations, the trade-off will be beneficial. So do some testing.

2.  Only the raw strings are removed from the binary; the code to render each key is still baked in it, whatever the locale or the namespace.
