

# Origen 2 [![Build Status](https://github.com/Origen-SDK/o2/workflows/Regression%20Tests/badge.svg)](https://github.com/Origen-SDK/o2/actions?query=workflow%3A%22Regression+Tests%22)

See here for how to setup an Origen 2 development environment - https://origen-sdk.org/o2/guides/developers/installation.html

To build or serve O2's own documentation from a source checkout, enter the
embedded core application first:

```text
cd python/origen
origen web build
origen web serve
```

The repository root is an O2 development workspace, not itself an Origen
application, so application-scoped ``web`` commands are not currently exposed
there. The release coordinator is the exception and can be run from the root.


### Origen Metal Python Development

1. Compile the Origen CLI by running:
   ```text
   cd rust/origen
   cargo build --bin origen --workspace
   ```
2. Add `<workspace>/rust/origen/target/debug/` to your `$PATH` or otherwise make the `origen` CLI
   that was just compiled to this location available for execution
3. Compile Origen Metal by running:
   ```text
   origen develop_origen build --metal
   ```

Repeat step 3 after making any changes to `rust/origen_metal` (the Rust library) or
`rust/pyapi_metal` (the Python bindings for it).

Run the Rust tests for the Python bindings without PyO3's extension-module linking mode:

```text
cargo test --manifest-path rust/pyapi/Cargo.toml --no-default-features
cargo test --manifest-path rust/pyapi_metal/Cargo.toml --no-default-features
```

To test local updates in an application, declare `python/origen_metal` as a
`[tool.uv.sources]` path dependency and run `uv sync --no-editable`.


### Releasing Origen and Origen Metal

`origen rc tag` is the release coordinator for both independently versioned
products. Run it from the O2 repository root. It calculates versions, updates
all Python and Rust manifests and lockfiles atomically, writes the canonical
history entry, validates packages and documentation, commits and tags through
the configured revision-control driver, and dispatches the exact-tag GitHub
Actions publication workflow. The website is deployed only after every
selected package has been published successfully.

Always preview first; this changes no files or external state:

```text
origen rc tag --product origen-metal --type minor --dry-run
origen rc tag --product origen --origen-type development \
  --product origen-metal --metal-type minor --dry-run
```

For a local, reviewable preparation that may update files but never commits,
tags, pushes, publishes, or deploys, use `--local`. Normal interactive releases
can omit `--type`; the CLI prompts for development, patch, minor, major,
production, or current. Do not manually bump manifests or directly dispatch
`publish.yml`/`publish_metal.yml`; those workflows are exact-ref publication
backends rather than release coordinators.

Canonical release notes live in `python/origen/doc/history` and
`python/origen/doc/metal/history` and are rendered into the website. See the
[complete release guide](https://origen-sdk.org/o2/guides/developers/releasing_origen.html)
for combined releases, non-interactive automation, credentials, recovery, and
verification. `origen web build --release` remains available for a docs-only
deployment outside a product release.
