

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

