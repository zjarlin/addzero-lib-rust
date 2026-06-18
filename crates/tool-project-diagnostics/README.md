# tool-project-diagnostics

Environment-independent project syntax diagnostics built on tree-sitter.

The crate is intended for service and web backends: upload or unpack a project
into a server-side directory, call `scan_project`, and return the serializable
diagnostic report as JSON. It does not invoke IDEs, language servers, package
managers, compilers, or user-local tools.

Supported languages in this first pass are Rust, JavaScript, TypeScript, TSX,
Python, and Java. The report contains tree-sitter syntax errors and missing
nodes, not type-checker or dependency-resolution diagnostics.
