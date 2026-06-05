# az-line-crdt

`az-line-crdt` provides a narrow Rust API for synchronizing text file content with CRDT semantics.

The crate intentionally hides the underlying CRDT engine from callers. The public surface works with:

- full text snapshots,
- incremental update blobs,
- version cursors,
- line insert, replace, and delete operations,
- exact text insert and delete operations for finer patches.

The first implementation uses `loro` internally because it already provides a maintained Rust CRDT document model, text container, snapshots, incremental updates, and version vectors. File watching, WebSocket transport, database indexes, and binary file handling belong in the sync service layer, not in this crate.
