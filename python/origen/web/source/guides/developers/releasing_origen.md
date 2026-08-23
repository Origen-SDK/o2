(releasing-origen)=
# Releasing Origen 2 and Origen Metal

Origen 2 and Origen Metal share a repository but have independent versions,
package registries, tags, and release histories. `origen rc tag` coordinates the
whole release transaction and replaces the legacy `develop_origen publish`
command.

## Before starting

Run from the O2 repository root on the configured release branch in an
up-to-date, clean checkout. The CLI automatically enters the internal
`python/origen` application context; maintainers should not change directories
or invoke an internal Python entry point.
Release lock generation requires UV 0.12.5 or newer, which is the same minimum
the CLI enforces and the exact version CI installs. Install it with:

```console
curl -LsSf https://astral.sh/uv/0.12.5/install.sh | sh
uv --version
```

Configure your Git name and credentials, plus the GitHub token used to dispatch
the configured publication workflow:

```console
git config user.name "Your Name"
git config user.email "you@example.com"
export github_pat=...
```

The command reads the source remote and release branch from
`config/application.toml`, verifies each package's repository metadata, and
uses O2's configured revision-control driver. It does not assume a GitHub owner,
repository, branch, workflow filename, or website checkout.

Prepare a release note in `release_note.txt`, pass another file with `--file`,
or enter the note when prompted. Empty notes are rejected.

## Previewing a release

Use `--dry-run` first. It calculates and displays the complete plan without
changing files or external state:

```console
origen rc tag --product origen-metal --type minor --dry-run
```

For a combined release, each product has its own release type:

```console
origen rc tag \
  --product origen --origen-type development \
  --product origen-metal --metal-type minor \
  --dry-run
```

`--type` is deliberately restricted to a single-product release. This prevents
one shorthand value from accidentally advancing both products the same way.

## Release types

When no type is supplied, interactive mode asks for one. The supported types
are:

| Type | Example |
| --- | --- |
| `development` | `2.0.0.dev8` to `2.0.0.dev9`, or `1.5.0` to `1.5.1.dev0` |
| `patch` | `1.5.0` to `1.5.1` |
| `minor` | `1.5.0` to `1.6.0` |
| `major` | `1.5.0` to `2.0.0` |
| `production` | `2.0.0.dev8` to `2.0.0` |
| `current` | publish an already prepared, unpublished current version |

A production version cannot be promoted again. When a combined release changes
Metal, Origen must also advance so its `origen-metal` requirement can be updated.

## Performing a release

The normal interactive commands are:

```console
origen rc tag --product origen
origen rc tag --product origen-metal
origen rc tag --product origen --product origen-metal
```

The command displays the versions, tags, dependency update, configured source,
workflows, author, and provider, then asks for final confirmation. It performs:

1. workspace, branch, upstream, registry-version, and tag checks;
2. Rust and Python tests;
3. atomic Python/Rust manifest and history updates;
4. wheel, crate, and strict full-documentation validation;
5. one release commit through the configured RC driver;
6. product-qualified tags such as `origen-v2.0.0.dev9` and
   `origen-metal-v1.6.0`;
7. exact-tag GitHub Actions publication and monitoring;
8. PyPI verification, plus crates.io verification for Metal;
9. GitHub Releases rendered from the canonical history entry; and
10. website build and deployment after every selected package succeeds.

The canonical histories are:

```text
python/origen/doc/history
python/origen/doc/metal/history
```

The website release pages include these files directly.

## Local preparation

To update and validate versions and histories without committing, tagging,
pushing, publishing, or deploying, use:

```console
origen rc tag --product origen-metal --type patch --local
```

This is useful for reviewing the exact generated diff. `--allow-local-changes`
is accepted only with `--local` or `--dry-run`; a real release always requires a
clean workspace.

## Non-interactive operation

Automation must supply every decision explicitly:

```console
origen rc tag \
  --product origen-metal \
  --type minor \
  --file release_note.txt \
  --non-interactive \
  --yes
```

Missing product, type, note, or `--yes` is an error. A future GA preparation
workflow can use this interface to open a reviewable release PR; publication
workflows never calculate versions or edit history themselves.

## Resuming a release

Release phase state is stored under `.origen/releases/`. If a commit, tag,
workflow, registry verification, or website phase fails, retain the checkout and
resume with the ID printed in the release plan/error output:

```console
origen rc tag --resume origen-v2.0.0.dev9
```

For a combined release the ID joins both tags with `__`. Resume loads the frozen
versions and workflows instead of recalculating them. Completed phases are
skipped. Existing commits and tags are pushed idempotently, and an existing or
running publication workflow is reused rather than dispatched again.

Do not delete the release state or move tags to recover a failed release. Fix
the reported credential, registry, workflow, or website problem and resume.

## Verification

After success, confirm:

- the product-qualified tags point to the release commit;
- the expected versions exist on PyPI;
- Metal exists on crates.io when selected;
- GitHub Releases contain the canonical release note; and
- the published O2 website shows the new entry under the correct product.

`origen develop_origen publish` is intentionally disabled because it bypassed
canonical history and exact-reference safeguards.
