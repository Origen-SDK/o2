#!/usr/bin/env python3
"""Extract one MyST release entry for use as a GitHub Release body."""

import argparse
import pathlib
import sys


def extract(history, tag_prefix, version):
    anchor = f"(release-{tag_prefix}-{version.replace('.', '-')})="
    lines = history.read_text().splitlines()
    try:
        start = lines.index(anchor) + 1
    except ValueError:
        raise ValueError(f"Release anchor {anchor} was not found in {history}")
    end = next((i for i in range(start, len(lines)) if lines[i].strip() == "---"), len(lines))
    body = "\n".join(lines[start:end]).strip()
    if not body:
        raise ValueError(f"Release entry {anchor} is empty")
    return body + "\n"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--history", type=pathlib.Path, required=True)
    parser.add_argument("--tag-prefix", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        args.output.write_text(extract(args.history, args.tag_prefix, args.version))
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
