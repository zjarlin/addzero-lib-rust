# az-micro-dict

Dictionary contribution SPI and build-time Rust enum generator.

This crate sits before `az-dict-macros`: contributors normalize dictionary metadata
from PostgreSQL, RuoYi-style admin tables, fixtures, or any other source into
`DictionaryContribution`. `DictBuildGenerator` validates the metadata and first
returns a portable in-memory `DictSourceBundle`. The host may commit that bundle
to a repository or write it to Cargo `OUT_DIR`. Generated `include_str!` paths are
relative to `enums.rs`.

## Build script shape

```rust,ignore
use az_micro_dict::contribution::{DictBuildGenerator, StaticDictionaryContributor};

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
