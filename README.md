

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

GitHub Actions tests, builds, and publishes releases, and deploys the website.
Nothing is built or tested locally. Locally, you write the release note and run
`origen rc tag`, which:

1. checks that the "Regression Tests" run for the `master` HEAD passed,
   waiting for it if it is still running;
2. bumps the versions in the Python and Rust manifests and lockfiles, and adds
   the release note to the product's release history
   (`python/origen/doc/history` or `python/origen/doc/metal/history`);
3. commits that, tags it (e.g. `origen-metal-v1.8.0`), and pushes both;
4. starts the release workflow, which publishes the tag to PyPI (and
   crates.io for Origen Metal) and then deploys the website.

Before you start, you need:

- the `origen` CLI built from this repository (see above) and
  [uv](https://docs.astral.sh/uv/) 0.12.5 or newer, which updates the lockfiles;
- a clean checkout of `master` that is up to date with GitHub, and push access
  to it with your normal Git setup;
- a GitHub token that can run workflows in `Origen-SDK/o2`, in `github_pat`:
  ```text
  export github_pat=$(gh auth token)
  ```
  Add `--user <account>` to choose the GitHub CLI account that has access.

Write the release note in `release_note.txt` in the repository root (Git
ignores it). It is added to the history as Markdown, below a heading with the
version, date, author and tag. Follow the style of the previous entries: a
one-sentence summary, then one bullet per change crediting the author and pull
request, e.g. `Contributed by Jane Doe in #123.`

Then run, from the repository root:

```text
origen rc tag --product origen-metal --type minor
```

Use `--product origen` for Origen, or `--product origen --origen-type <type>
--product origen-metal --metal-type <type>` to release both. The types are
`development`, `patch`, `minor`, `major`, `production`, and `current` (release
the version already in the manifests). The command shows the release plan and
asks for confirmation before changing anything. It prints the URL of the
release workflow once it has started it, and `--wait` makes it wait for that
workflow to finish.

If it stops before committing, its file changes are undone, so fix the problem
and run it again. If it stops after that, continue with the release ID it
printed; finished steps are skipped:

```text
origen rc tag --resume origen-metal-v1.8.0
```

Do not bump versions by hand or run `publish.yml`/`publish_metal.yml`
directly. See the
[complete release guide](https://origen-sdk.org/o2/guides/developers/releasing_origen.html)
for more detail.
