# Access Translations in a Const Context

This version does not expose a supported public accessor for reading translations in a const context. The internal const helpers used by generated code are hidden implementation details.

For a translation such as:

```json
{
  "subkeys": {
    "key": "my value"
  }
}
```

Read it at runtime with a translation macro:

```rust,ignore
use crate::i18n::*;
let value = td_string!(Locale::en, subkeys.key);
assert_eq!(value, "my value");
```

The public `leptos_i18n::keys::comp_time::ConstArgs::value` trait method also reads literal values without interpolation arguments, but it is an ordinary function and cannot be called in a const context.

With `dynamic_load`, await the runtime accessor: `td_string!(Locale::en, subkeys.key).await`. This applies on both the client and server; see [dynamic loading](../reduce_size/02_dynamic_load.md).
