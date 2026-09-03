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

Configure your Git identity and export a GitHub token for the short local
commit, tag, and workflow-dispatch phase:

```console
git config user.name "Your Name"
git config user.email "you@example.com"
export github_pat=...
```

Set `ORIGEN_GIT_SSH_KEY` when the checkout uses SSH and more than one GitHub
identity exists locally. The release workflow itself uses the
`O2_RELEASE_APP_ID` and `O2_RELEASE_APP_PRIVATE_KEY` repository secrets to
create a short-lived GitHub App token. The app must have Actions and Contents
write access to both `Origen-SDK/o2` and
`Origen-SDK/Origen-SDK.github.io`. Package registry credentials remain in
their existing GitHub Actions secrets.

The command reads the source remote, release branch, and orchestrator workflow
from `config/application.toml`, verifies each package's repository metadata,
and uses O2's configured revision-control driver.

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
workflow, author, and provider, then asks for final confirmation. Locally it:

1. validates the workspace, branch, upstream, registry versions, and tags;
2. runs Rust and Python tests;
3. updates Python/Rust manifests, lockfiles, and release histories atomically;
4. validates wheels, the Metal crate, and strict full documentation;
5. creates one release commit and product-qualified tags;
6. pushes the commit and tags;
7. dispatches the combined GitHub Actions release workflow; and
8. records the workflow run ID and URL before returning.

GitHub Actions then publishes and verifies Metal before publishing dependent
Origen packages, creates GitHub Releases, and deploys the website. The local
terminal does not wait by default. Add `--wait` to retain synchronous
monitoring:

```console
origen rc tag --product origen --product origen-metal --wait
```

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

Missing product, type, note, or `--yes` is an error. Publication workflows
consume exact prepared tags; they never calculate versions or edit history.

## Monitoring and resuming a release

Release state, including the remote workflow run ID and URL, is stored under
`.origen/releases/`. Check it without redispatching:

```console
origen rc tag --status origen-v2.0.0.dev9__origen-metal-v1.6.0
```

Add `--wait` to block until the combined workflow completes:

```console
origen rc tag \
  --status origen-v2.0.0.dev9__origen-metal-v1.6.0 \
  --wait
```

If preparation, commit, tagging, or dispatch fails, retain the checkout and
resume with the release ID. Completed local phases are skipped. A queued or
running combined workflow is reused; a failed combined workflow is
redispatched:

```console
origen rc tag --resume origen-v2.0.0.dev9__origen-metal-v1.6.0
```

Do not delete release state or move published tags to recover a failed release.

## Verification

After success, confirm:

- the product-qualified tags point to the release commit;
- the expected versions exist on PyPI;
- Metal exists on crates.io when selected;
- GitHub Releases contain the canonical release note; and
- the published O2 website shows the new entry under the correct product.

`origen develop_origen publish` is intentionally disabled because it bypassed
canonical history and exact-reference safeguards.
