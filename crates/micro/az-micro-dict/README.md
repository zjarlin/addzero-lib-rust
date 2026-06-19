# az-micro-dict

Dictionary contribution SPI and build-time Rust enum generator.

This crate sits before `az-dict-macros`: contributors normalize dictionary metadata
from PostgreSQL, RuoYi-style admin tables, fixtures, or any other source into
`DictionaryContribution`. `DictBuildGenerator` validates the metadata, writes
dictionary specs to an output directory such as Cargo `OUT_DIR`, and emits Rust
source that calls `az_dict_macros::dict_enum!`.

## Build script shape

```rust,ignore
use az_micro_dict::api::{DictBuildGenerator, StaticDictionaryContributor};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    DictBuildGenerator::new()
        .add_contributor(StaticDictionaryContributor::new(vec![my_dict()]))
        .generate_to(out_dir)?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
```

Then include the generated source from normal Rust code:

```rust,ignore
include!(concat!(env!("OUT_DIR"), "/az_micro_dict/enums.rs"));
```

The generated source references `az_dict_macros`, `az_dict_spec`, and
`derive_more`, so runtime crates that include it should depend on those crates.
