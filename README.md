

# Origen 2 [![Build Status](https://github.com/Origen-SDK/o2/workflows/Regression%20Tests/badge.svg)](https://github.com/Origen-SDK/o2/actions?query=workflow%3A%22Regression+Tests%22)

See here for how to setup an Origen 2 development environment - https://origen-sdk.org/o2/guides/developers/installation.html


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
   origen origen build --metal
   ```

Repeat step 3 after making any changes to `rust/origen_metal` (the Rust library) or
`rust/pyapi_metal` (the Python bindings for it).

To test out any updates in your application add `python/origen_metal` to your application's
virtual environment.

If using another venv manager than Poetry, you might need to uncomment the `[project]` section
in `pyproject.toml`.


### Publishing Origen Metal

Origen Metal's Python package and Rust crate are published with the
[`Publish Origen Metal`](https://github.com/Origen-SDK/o2/actions/workflows/publish_metal.yml)
GitHub Actions workflow. The workflow publishes directly to the production PyPI and crates.io
registries; it does not publish test or prerelease packages.

Before starting a release:

1. Update and commit the Origen Metal version in the Python and Rust manifests, including the
   Python-binding crate:
   - `python/origen_metal/pyproject.toml`
   - `rust/origen_metal/Cargo.toml`
   - `rust/pyapi_metal/Cargo.toml`
2. Regenerate and commit the corresponding Cargo lockfiles. In particular,
   `rust/origen_metal/Cargo.lock` must record the same `origen_metal` version as
   `rust/origen_metal/Cargo.toml`; the Rust publish validation uses `--locked` and will fail if
   they differ.
3. Ensure the version has not already been published to the selected registry. Published
   versions cannot be overwritten.
4. Push the release commit to the Git ref that will be selected when dispatching the workflow.

To publish:

1. Open **Actions > Publish Origen Metal > Run workflow**.
2. Select the release branch or ref.
3. Select `publish_python`, `publish_rust`, or both, then run the workflow. Selecting neither
   intentionally fails the precheck.

When Python is selected, the workflow builds and merges the supported Linux and Windows wheels
before publishing them to PyPI. When Rust is selected, the workflow installs the current stable
Rust toolchain and runs
`cargo publish --dry-run --locked` before publishing the crate to crates.io. When both are
selected, their manifest versions must match, all builds and validation must pass, and Python is
published before Rust.

If Python publication fails during a combined release, Rust is not published. If Python succeeds
but Rust publication fails, correct the Rust issue and rerun the workflow with only
`publish_rust` selected.

The Python builds use these repository settings:

- `PYTHON_VERSIONS_FOR_RELEASE`, `PYTHON_VERSIONS`, and `RUST_VERSION` variables

Publishing uses these repository secrets:

- `PYPI_OM_API_TOKEN` for PyPI authentication
- `CARGO_ORIGEN_METAL` for crates.io authentication
