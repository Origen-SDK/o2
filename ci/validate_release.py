#!/usr/bin/env python3
"""Validate prepared release sources and built wheels without publishing them."""

import argparse
import email
import pathlib
import re
import sys
import zipfile


def normalized_name(name):
    return re.sub(r"[-_.]+", "-", name).lower()


def normalized_version(version):
    return re.sub(r"-(dev|alpha|beta)\.?", r".\1", version)


def python_versions_to_tags(raw):
    tags = set()
    for entry in raw.split(","):
        entry = entry.strip()
        if entry:
            tags.add("cp" + entry.replace(".", ""))
    return tags


def wheel_tags(wheel):
    """Return (python_tag, abi_tag, platform_tag) from a wheel filename."""
    pieces = wheel.stem.split("-")
    if len(pieces) < 5:
        raise ValueError(f"Malformed wheel filename: {wheel.name}")
    return tuple(pieces[-3:])


def manifest_version(path):
    match = re.search(r'^version\s*=\s*["\']([^"\']+)["\']', path.read_text(), re.M)
    if not match:
        raise ValueError(f"No version found in {path}")
    return match.group(1)


def validate_source(args):
    expected_tag = f"{args.tag_prefix}-v{args.version}"
    tags = set(args.tags.splitlines())
    if expected_tag not in tags:
        raise ValueError(f"{expected_tag} does not point at the checked-out commit")

    for manifest in args.manifest:
        actual = manifest_version(manifest)
        if normalized_version(actual) != normalized_version(args.version):
            raise ValueError(
                f"Version mismatch in {manifest}: expected {args.version}, found {actual}"
            )

    anchor_version = args.version.replace(".", "-")
    anchor = f"(release-{args.tag_prefix}-{anchor_version})="
    if anchor not in args.history.read_text():
        raise ValueError(f"Missing release anchor {anchor} in {args.history}")


def validate_wheels(args):
    wheels = sorted(args.directory.glob("*.whl"))
    if not wheels:
        raise ValueError(f"No wheels found in {args.directory}")
    expected_name = normalized_name(args.package)
    for wheel in wheels:
        with zipfile.ZipFile(wheel) as archive:
            metadata_path = next(
                (name for name in archive.namelist() if name.endswith(".dist-info/METADATA")),
                None,
            )
            if metadata_path is None:
                raise ValueError(f"No METADATA found in {wheel}")
            metadata = email.message_from_bytes(archive.read(metadata_path))
        actual_name = normalized_name(metadata["Name"] or "")
        actual_version = metadata["Version"]
        if actual_name != expected_name or actual_version != args.version:
            raise ValueError(
                f"Unexpected metadata in {wheel}: {actual_name} {actual_version}; "
                f"expected {expected_name} {args.version}"
            )

    # A build that resolves the wrong interpreter emits duplicate ABI tags,
    # which then collapse silently during artifact merge. Metadata alone cannot
    # detect that, so assert tag coverage explicitly.
    seen = {}
    for wheel in wheels:
        python_tag, _abi_tag, platform_tag = wheel_tags(wheel)
        key = (python_tag, platform_tag)
        if key in seen:
            raise ValueError(
                f"Duplicate {python_tag}/{platform_tag} wheels: {seen[key].name} and {wheel.name}"
            )
        seen[key] = wheel

    if args.expect_python_versions:
        expected_tags = python_versions_to_tags(args.expect_python_versions)
        platforms = {platform for _python, platform in seen}
        for platform in sorted(platforms):
            built = {python for python, plat in seen if plat == platform}
            missing = expected_tags - built
            if missing:
                raise ValueError(
                    f"Missing {', '.join(sorted(missing))} wheels for platform {platform}; "
                    f"built {', '.join(sorted(built))}"
                )

    for wheel in wheels:
        print(f"  {wheel.name}")
    print(f"Validated {len(wheels)} {expected_name} wheels for {args.version}")


def parser():
    root = argparse.ArgumentParser()
    commands = root.add_subparsers(dest="command", required=True)
    source = commands.add_parser("source")
    source.add_argument("--version", required=True)
    source.add_argument("--tag-prefix", required=True)
    source.add_argument("--tags", required=True)
    source.add_argument("--history", type=pathlib.Path, required=True)
    source.add_argument("--manifest", type=pathlib.Path, action="append", required=True)
    source.set_defaults(run=validate_source)
    wheels = commands.add_parser("wheels")
    wheels.add_argument("--package", required=True)
    wheels.add_argument("--version", required=True)
    wheels.add_argument("--directory", type=pathlib.Path, required=True)
    wheels.add_argument(
        "--expect-python-versions",
        default="",
        help="Comma-separated Python versions that must be present for every platform",
    )
    wheels.set_defaults(run=validate_wheels)
    return root


def main():
    args = parser().parse_args()
    try:
        args.run(args)
    except (OSError, ValueError, KeyError, StopIteration, zipfile.BadZipFile) as error:
        print(f"Release validation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
