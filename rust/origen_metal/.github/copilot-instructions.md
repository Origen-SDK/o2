# Origen Metal - AI Agent Instructions

## Project Overview

**Origen Metal** is a bare-metal Rust library providing core APIs for semiconductor test program generation and STIL (Standard Test Interface Language) processing. It's designed as the foundation for the Origen SDK, supporting both pure Rust usage and optional Python bindings via PyO3.

**Key Purpose**: Transform abstract test flow specifications into concrete test programs for multiple ATE (Automatic Test Equipment) platforms through AST-based processing pipelines.

## Architecture

### Core Design Pattern: AST Processing Pipeline

The project centers on a powerful AST (Abstract Syntax Tree) processing system located in `src/ast/`:

- **Node<T>**: Generic tree structure where `T` implements the `Attrs` trait (auto-implemented for types that are Clone + PartialEq + Serialize + Display + Debug)
- **AST<T>**: Builder API for constructing node trees with `push()`, `push_and_open()`, `close()` methods
- **Processor<T>**: Trait defining transformation passes that return `Return<T>` enum variants:
  - `None`: Delete node
  - `Unmodified`: Keep as-is (clone node and children without processing)
  - `ProcessChildren`: Clone node, process children recursively
  - `Replace(Node<T>)`: Substitute entire node
  - `Unwrap`: Flatten node, keeping children
  - `Inline(Vec<Node<T>>)`: Replace with multiple nodes
  - `InlineBoxed(Vec<Box<Node<T>>>)`: Same but with boxed nodes
  - `UnwrapWithProcessedChildren`: Unwrap and process children
  - `InlineWithProcessedChildren(Vec<Node<T>>)`: Add nodes then process original children
  - `ReplaceChildren(Vec<Node<T>>)`: Keep node, replace children

**Example Pattern** (from `src/prog_gen/processors/flag_optimizer.rs`):
```rust
impl Processor<PGM> for FlagOptimizer {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::SetFlag(flag, _, auto_generated) => {
                // Transformation logic here
                Return::None  // Or other Return variant
            }
            _ => Return::ProcessChildren,
        })
    }
}
```

### Program Generation (`src/prog_gen/`)

Converts test flow ASTs into tester-specific outputs:

- **PGM enum** (`nodes.rs`): Node attribute types for test flows (Flow, Test, Bin, Condition, etc.)
- **FlowManager** (`flow_manager.rs`): Thread-safe RWLock wrapper managing flow ASTs
  - Access via global `FLOW` static: `FLOW.push(node)`, `FLOW.with_flow(|ast| ...)`
- **Model** (`model/model.rs`): Extracted data structure from AST (tests, patterns, bins, variables)
- **Processor Chain**: Multi-pass transformations (flag optimization, condition nesting, tester-specific codegen)
- **Tester Targets**: `advantest/` (SMT7, SMT8) and `teradyne/` (J750, UltraFlex) implementations

### Global State via lazy_static

Key singletons in `src/lib.rs`:
```rust
lazy_static! {
    pub static ref FLOW: FlowManager = FlowManager::new();
    pub static ref LOGGER: Logger = Logger::default();
    pub static ref USERS: RwLock<Users> = RwLock::new(Users::default());
    pub static ref SESSIONS: Mutex<Sessions> = Mutex::new(Sessions::new());
    pub static ref FRONTEND: RwLock<Frontend> = RwLock::new(Frontend::new());
}
```

**Access Pattern**: Use `with_*` functions to safely interact with locked globals:
- `with_frontend(|f| f.method())`
- `with_current_user(|u| u.dataset())`
- `FLOW.with_flow(|ast| ast.push(node))`

### STIL Processing (`src/stil/`)

Parses and processes Standard Test Interface Language files:
- Parser built with Pest grammar (`stil.pest`)
- Includer processor resolves file dependencies
- Time expression processor for timing calculations

### Framework Services (`src/framework/`)

Supporting infrastructure:
- **Users**: Multi-user session management with dataset hierarchies, LDAP integration
- **Sessions**: Persistent key-value storage across runs
- **Logger**: Centralized logging via `LOGGER` global
- **TypedValue**: Heterogeneous value storage (String, Int, Vec, Map)

### Frontend Abstraction (`src/frontend/`)

Optional trait-based interface allowing higher-level frameworks to inject functionality:
```rust
pub trait FrontendAPI {
    fn method(&self) -> Result<T>;
}
```
Set via `set_frontend(Box<dyn FrontendAPI>)`, access via `with_frontend()`.

## Development Workflow

### Building
```bash
cargo build          # Pure Rust build
cargo build --features python  # With Python bindings
```

The `build.rs` script generates `test_templates.rs` at compile time, embedding template files as a `phf::Map` for zero-cost template access.

### Testing
```bash
cargo test           # Run unit tests
cargo test --features origen_skip_frontend_tests  # Skip frontend-dependent tests
```

Tests are colocated with code in `#[cfg(test)]` modules. See `src/utils/encryption.rs` and `src/utils/version.rs` for examples.

### Key Macros

- **`node!()`** (`src/macros.rs`): Ergonomic AST node construction with multiple forms:
  ```rust
  node!(PGM::Test, id, name)                      // Basic node with attrs
  node!(PGM::Test, id, name => child1, child2)    // With children
  node!(PGM::Flow, name ; meta)                   // With metadata
  node!(PGM::Nil)                                 // No-arg variants
  ```
- **`bail!()`**: Early return with `Error` - expands to `return Err(error!(...))`. Use instead of `return Err(Error::new(...))`
- **`error!()`**: Construct `Error` from string/format args - `error!("msg")` or `error!("msg: {}", val)`
- **`trace!()`**: Wrap Results with AST node context for better error messages - used in `prog_gen` module
- **`display!()` / `displayln!()`**: Log output via `LOGGER` global - thread-safe display without println!

## Critical Conventions

1. **Error Handling**: Use `crate::Result<T>` (alias for `std::result::Result<T, crate::Error>`). Custom `Error` type in `src/error.rs` with conversions for git2, io, regex, PyO3 errors.

2. **AST Modification**: Never directly manipulate `node.children` - always use the Processor API's `Return` variants.

3. **Node Transformation Helpers**: Use these methods when implementing processors:
   - `node.process_and_box_children(processor)` - Process children, returns `Vec<Box<Node<T>>>`
   - `node.updated(attrs, children, meta)` - Create modified copy with optional replacements (use `None` to keep existing)
   - `node.without_children()` - Clone node without children for rebuilding
   - Pattern: `Return::Replace(node.updated(None, Some(new_children), None))`

4. **Template System**: Test templates loaded via `src/prog_gen/model/template_loader.rs` from embedded `TEST_TEMPLATES` map (generated by build script).

5. **Tester-Specific Code**: Isolate in `advantest/` or `teradyne/` subdirectories. Common processors in `prog_gen/processors/`.

6. **Feature Flag**: `#[cfg(feature = "python")]` gates PyO3 code. Library is Rust-first, Python-optional.

7. **Indexmap Over HashMap**: Use `indexmap::IndexMap` for insertion-order preservation (critical for test flow ordering).

8. **Processor Lifecycle**: Processors can implement three hooks:
   - `on_node()` - Called before processing children (required, default: `ProcessChildren`)
   - `on_end_of_block()` - Called after all children processed (optional, legacy)
   - `on_processed_node()` - Called after children processed (optional, preferred over `on_end_of_block`)

## Common Tasks

**Add a new PGM node type**:
1. Extend `PGM` enum in `src/prog_gen/nodes.rs`
2. Update relevant processors to handle new variant
3. Add to tester-specific generators

**Create a new processor**:
1. Define struct implementing `Processor<PGM>` trait
2. Implement `on_node()` and optionally `on_processed_node()` (preferred) or `on_end_of_block()` (legacy)
3. Call via `node.process(&mut processor)?`

**Access test flow**:
```rust
FLOW.with_flow(|ast| {
    ast.push(node!(PGM::Test, id, name));
    Ok(())
})?;
```

**Add global state**:
Use `lazy_static!` + RwLock/Mutex pattern, provide `with_*` accessor functions following existing conventions.

**Build and test**:
```bash
cargo build                                          # Pure Rust build
cargo build --features python                        # With Python bindings
cargo test                                           # Run unit tests
cargo test --features origen_skip_frontend_tests     # Skip frontend-dependent tests
```

## Project-Specific Knowledge

- **FlowID**: Unique identifier for nodes that can be referenced (tests, groups). Used for on_passed/on_failed relationships.
- **ResourcesType**: Controls where tests appear (flow vs. resources-only sheets)
- **UniquenessOption**: Test name disambiguation strategies (Signature, Flowname, custom string)
- **Multi-pass Processing**: Processors often run 2+ times (count references, then optimize) - see `flag_optimizer.rs`

## Dependencies Note

Uses Rust 1.71.0 (2021 edition). Key crates: pest (parsing), pyo3 (Python), indexmap (ordered maps), lazy_static (globals), serde (serialization), tera/minijinja (templating).
