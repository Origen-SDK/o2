(developers-release-process)=
# Release Process

This document is the implementation contract for releasing Origen 2 and
Origen Metal. It preserves the useful behavior of Origen v1's `origen rc tag`
while supporting two independently versioned products and optional GitHub
Actions publication.

## Goals

- One release coordinator owns versioning, notes, history, validation, Git
  state, publication, and website deployment.
- Origen and Origen Metal can be released separately or together.
- Local interactive and CI non-interactive operation use the same engine.
- Release state is reviewable, reproducible, resumable, and configuration
  driven.
- No package can be published without its versioned release history.

## Command contract

The primary interface is:

```text
origen rc tag [--product origen|origen-metal]...
              [--type development|patch|minor|major|production|current]
              [--origen-type TYPE --metal-type TYPE]
              [--note TEXT | --file PATH]
              [--local | --dry-run]
              [--non-interactive --yes]
```

For the O2 source repository this command is invoked from the repository root.
The CLI detects the development workspace and re-enters the core application
under `python/origen` in a fresh process because application discovery occurs
during CLI boot.

With no product, type, note, or version supplied, the command prompts for each
value and shows the complete release plan before changing files. Combined
releases prompt for each product independently.

`--non-interactive` converts every missing required value into an error.
`--yes` accepts the final plan but never invents missing metadata.
`--dry-run` performs validation and renders proposed changes without external
writes. `--local` may update/build locally but does not commit, tag, push,
publish, or deploy.

The old `origen develop_origen publish` command is retained only to emit a
deprecation message and then stops with an error. It does not delegate or
publish, because doing so would bypass canonical history and exact-reference
safeguards.

## Canonical release histories

The histories are version-controlled inputs, not generated from PyPI during a
documentation build:

```text
python/origen/doc/history
python/origen/doc/metal/history
```

The Sphinx pages contain only an include directive for the corresponding
history. `rc tag` prepends an anchor, product/version heading, author, date,
tag, registry/source links, and the supplied release note.

The anchor is `(release-<tag-prefix>-<version with dots replaced by dashes>)=`,
for example `(release-origen-2-0-0-dev9)=` or
`(release-origen-metal-1-6-0)=`. The prefix is not cosmetic: `rc tag` uses the
anchor to refuse a duplicate release, `ci/validate_release.py` requires it
before publishing, and `ci/extract_release_notes.py` slices the GitHub release
body from it. An entry written without the product prefix is invisible to all
three.

## Configuration

Source revision control is resolved through the application's
`[revision_control]` configuration and O2's existing
`RevisionControl`/`Git` driver. Package `project.urls.Repository` values are
cross-checked against the configured remote. A mismatch stops the release.

The release branch, source remote, local repository, website remote, and
website subdirectory must not be hardcoded. Optional publication providers are
configured separately:

```toml
[release]
provider = "github_actions"

origen_workflow = "publish.yml"
origen_metal_workflow = "publish_metal.yml"
```

The website uses `website_release_location` and `website_release_name`.

## Tags

Product-qualified tags avoid collisions in the shared repository:

```text
origen-v2.0.0.dev9
origen-metal-v1.6.0
```

Tags must point to the commit containing the matching manifest versions and
history entries. Existing local or remote tags are fatal unless an explicit
recovery operation proves the release is identical.

## Transaction

1. Resolve configuration, products, release types, proposed versions, notes,
   author, source repository, and publication provider.
2. Verify the release branch, clean workspace, upstream alignment,
   credentials, unused registry versions, and unused tags.
3. Run formatting, unit/regression tests, package validation, and a strict full
   documentation build.
4. Update every Python and Rust manifest for the selected product. Update
   Origen's Metal dependency only when the selected release plan requires it.
5. Prepend each selected product's history and rebuild documentation.
6. Present the final diff and request confirmation.
7. Commit version and history files atomically through the configured RC
   driver, then create and push product-qualified tags.
8. Dispatch the configured publication workflow with the exact tag and expected
   version, monitor it, and verify registry publication.
9. Deploy the already validated website only after all selected publications
   succeed.
10. Record completion or a resumable failure state.

Validation failures happen before the release commit. A failure after the tag
must never silently rewrite the tag; `--resume RELEASE_ID` loads the frozen
release plan, detects completed phases, and
continues from the first incomplete phase.

## GitHub Actions contract

The Origen and Metal workflows receive an exact `release_ref` and `version`.
They check out that ref and verify the manifest, history entry, and tag before
building. They do not calculate versions, write history, commit to the release
branch, or deploy documentation.

Metal additionally verifies that Python and Rust package versions match before
publishing to PyPI or crates.io. Both workflows publish only after every
platform artifact has built and passed metadata validation.

A future GA-initiated release uses a preparation workflow to run the same engine
non-interactively and open a release PR. Publication begins only after that PR
is reviewed and merged.

## Required tests

- Origen-only, Metal-only, and combined release plans.
- Every release type and development-to-production promotion.
- Interactive prompting and non-interactive missing-input failures.
- Note text, note file, and rejected empty notes.
- Configured HTTPS/SSH remotes, alternate branches, and metadata mismatches.
- Dirty, behind, diverged, wrong-branch, duplicate-version, and duplicate-tag
  failures.
- Dry-run and local modes with zero external mutations.
- Atomic version/history updates and product-qualified tags against local bare
  Git remotes.
- Publication failure, website failure, and safe resume behavior.
- Exact-ref workflow validation for Linux and Windows artifacts.
- Strict full documentation builds with both histories and correct release-page
  navigation.

No test may contact the production repository, registries, or website remote.
