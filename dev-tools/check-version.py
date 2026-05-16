#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


HELP_TEXT = """
Name
----

check-version - verify version syntax and expand as needed.

Synopsis
--------

```bash
check-version [OPTIONS] VERSION [MESSAGE ARGS]
```

Message arguments are simply echoed back.

| Option       | Description                                          |
| ------------ | ---------------------------------------------------- |
| --help       | Display this text                                    |
| --use-dots   | Output the result in dotted form (default is dashes) |
| --no-message | Suppress anything other than the version             |
| --silent     | Suppress all messages (i.e., return code only)       |
| --verbose    | More messages                                        |

Description
-----------

Examines the supplied version number and expands if necessary to the standard tuple of MAJOR-MINOR-PATCH.
Obtains the largest git tag matching v#.#.#, which is considered the current version.
Compares the supplied version to the current version and returns an error if there is not a strict advance in the version number.
If comparison passed, display the full formatted version with two possibilites. Either use hyphens (default) or dots.
""".strip("\n")

VERSION_RE = re.compile(r"^\d+(?:[-.]\d+){0,2}$")
TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
NAME_RE = re.compile(r'^name\s*=\s*"([^"]+)"')
PACKAGE_VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"')
BIN_SECTION_RE = re.compile(r"^\[\[bin\]\]$")


def run_git(args: list[str], cwd: Path) -> str:
    """Run a git command and return trimmed stdout.

    A small wrapper keeps subprocess usage in one place and makes the main
    logic easier to read.
    """

    completed = subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def find_git_root() -> Path:
    """Locate the repository root from the current working directory."""

    try:
        return Path(run_git(["rev-parse", "--show-toplevel"], Path.cwd()))
    except (subprocess.CalledProcessError, FileNotFoundError) as error:
        raise SystemExit(f"Error: unable to determine git repository root: {error}") from error


def parse_cargo_metadata(cargo_toml: Path) -> tuple[str, str]:
    """Read the binary name and package version from `Cargo.toml`.

    The existing shell script extracted these values with short Perl snippets.
    Here we keep the logic deliberately simple and line-oriented so it remains
    easy to debug on any platform without extra dependencies.
    """

    tool_name = ""
    source_version = ""
    in_bin_section = False
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
        stripped_line = line.strip()

        if BIN_SECTION_RE.match(stripped_line):
            in_bin_section = True
            continue

        if stripped_line.startswith("[") and stripped_line != "[[bin]]":
            in_bin_section = False

        if not source_version and not in_bin_section:
            version_match = PACKAGE_VERSION_RE.match(line)
            if version_match is not None:
                source_version = version_match.group(1)

        if not tool_name and in_bin_section:
            name_match = NAME_RE.match(line)
            if name_match is not None:
                tool_name = name_match.group(1)

        if tool_name and source_version:
            break

    if not tool_name or not source_version:
        raise SystemExit(f"Error: unable to read package metadata from {cargo_toml}")

    return tool_name, source_version


def find_current_tag_version(repo_root: Path) -> tuple[int, int, int]:
    """Return the numerically largest `vMAJOR.MINOR.PATCH` git tag.

    The shell version used a plain lexical sort. This Python version sorts the
    numeric triples directly so versions behave consistently across platforms.
    If no matching tag exists, we keep the previous fallback behavior of 0.0.0.
    """

    try:
        raw_tags = run_git(["tag", "--list"], repo_root).splitlines()
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"Error: unable to list git tags: {error}") from error

    versions: list[tuple[int, int, int]] = []
    for tag in raw_tags:
        match = TAG_RE.match(tag.strip())
        if match is not None:
            versions.append(tuple(int(part) for part in match.groups()))

    if not versions:
        return (0, 0, 0)

    return max(versions)


def version_tuple_to_dots(version: tuple[int, int, int]) -> str:
    return ".".join(str(part) for part in version)


def version_tuple_to_dashes(version: tuple[int, int, int]) -> str:
    return "-".join(str(part) for part in version)


def expand_incoming_version(raw_version: str, current_version: tuple[int, int, int]) -> tuple[int, int, int]:
    """Expand short forms into a complete three-part version.

    Accepted inputs mirror the shell script:
    - `X`
    - `X-Y`
    - `X-Y-Z`
    - dotted forms of the same shapes
    """

    normalized = raw_version.replace(".", "-")
    if not VERSION_RE.match(normalized):
        raise ValueError(
            f"'{normalized}' does not match [:digit:]+([-.][:digit:]+){{1,3}}"
        )

    parts = [int(part) for part in normalized.split("-")]
    if len(parts) == 3:
        return tuple(parts)  # type: ignore[return-value]
    if len(parts) == 2:
        return (current_version[0], parts[0], parts[1])
    return (current_version[0], parts[0], current_version[2])


def compare_versions(left: tuple[int, int, int], right: tuple[int, int, int]) -> int:
    """Return 1 when left > right, -1 when left < right, else 0."""

    if left > right:
        return 1
    if left < right:
        return -1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--help", "-h", action="store_true")
    parser.add_argument("--debug", action="store_true")
    parser.add_argument("--use-dots", action="store_true")
    parser.add_argument("--no-message", "-nm", action="store_true")
    parser.add_argument("--show", action="store_true")
    parser.add_argument("--silent", "-s", action="store_true")
    parser.add_argument("--verbose", "-v", action="store_true")
    parser.add_argument("version", nargs="?")
    parser.add_argument("message_args", nargs=argparse.REMAINDER)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    if args.help:
        print(HELP_TEXT)
        return 0

    repo_root = find_git_root()
    tool_name, source_version = parse_cargo_metadata(repo_root / "Cargo.toml")
    current_version = find_current_tag_version(repo_root)
    current_version_dots = version_tuple_to_dots(current_version)

    # The shell script falls back to the source version when no positional
    # version is supplied or when the first remaining argument is not numeric.
    incoming_raw = args.version
    message_args = list(args.message_args)
    if incoming_raw is None or not incoming_raw[:1].isdigit():
        if incoming_raw is not None:
            message_args.insert(0, incoming_raw)
        incoming_raw = source_version

    try:
        expanded_version = expand_incoming_version(incoming_raw, current_version)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    expanded_version_dots = version_tuple_to_dots(expanded_version)

    if args.show or args.verbose:
        print(f"Binary tool name    {tool_name}")
        print(f"Current tag version {current_version_dots}")
        print(f"Source code version {source_version}")
        print(f"Specified   version {expanded_version_dots}")
        if args.show:
            return 0

    if compare_versions(expanded_version, current_version) != 1:
        print(
            f"Error: Specified version {version_tuple_to_dashes(expanded_version)} "
            f"must be greater than current version {version_tuple_to_dashes(current_version)}",
            file=sys.stderr,
        )
        return 1

    final_version = expanded_version_dots if args.use_dots else version_tuple_to_dashes(expanded_version)
    if not args.silent:
        if message_args and not args.no_message:
            print(final_version, *message_args)
        else:
            print(final_version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())