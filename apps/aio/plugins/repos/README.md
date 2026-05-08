Place independent `aio-plugin-*` repositories here.

Recommended model:

- one plugin repo per directory
- plugin repo may be attached as a git submodule
- plugin build output is packaged into `../catalog/*.azplugin`
- the AIO host only loads packaged wasm plugins, not Rust crate plugins from the main repo
