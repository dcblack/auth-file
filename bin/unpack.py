#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path


RUST_ARCHIVE_RE = re.compile(r".+-rust-v\d+-\d+-\d+\.zip$")


def run_git(args: list[str], cwd: Path) -> str:
    completed = subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def find_repo_root() -> Path:
    """Return the git working tree root, or fail with a clear error."""

    try:
        return Path(run_git(["rev-parse", "--show-toplevel"], Path.cwd()))
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Error: must be inside git repo", file=sys.stderr)
        raise SystemExit(1)


def copy_tree_contents(source_dir: Path, destination_dir: Path, ignore_names: set[str]) -> None:
    """Recursively copy extracted project files into the repository.

    The old shell script used `rsync --exclude-from=../.gitignore -av`. For
    Windows portability we do the copy directly in Python and skip only simple
    name matches from `.gitignore`. That keeps the behavior understandable and
    removes the dependency on `rsync`.
    """

    for entry in source_dir.iterdir():
        if entry.name in ignore_names:
            continue

        destination = destination_dir / entry.name
        if entry.is_dir():
            destination.mkdir(parents=True, exist_ok=True)
            copy_tree_contents(entry, destination, ignore_names)
        else:
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(entry, destination)


def load_simple_gitignore_names(gitignore_path: Path) -> set[str]:
    """Load simple path-name exclusions from `.gitignore`.

    We intentionally keep this conservative and easy to follow. Only plain file
    or directory names are treated as exclusions. Pattern-heavy gitignore rules
    are ignored because the legacy use here mainly filtered project-local build
    directories such as `target`.
    """

    ignored: set[str] = set()
    if not gitignore_path.is_file():
        return ignored

    for raw_line in gitignore_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if any(token in line for token in "*?[]!"):
            continue
        ignored.add(line.strip("/"))
    return ignored


def unpack_zip(archive_path: Path, destination_dir: Path) -> None:
    with zipfile.ZipFile(archive_path) as archive:
        archive.extractall(destination_dir)


def is_rust_snapshot_archive(archive_path: Path) -> bool:
    """Return `True` only for project snapshot archives.

    We deliberately exclude note archives such as
    `auth-rust-update-notes-v0-8-5.zip`, which also contain `-rust-` in their
    names but extract to a single markdown file instead of a directory tree.
    """

    return RUST_ARCHIVE_RE.fullmatch(archive_path.name) is not None


def print_header(version: str) -> None:
    # Keep the output simple and obvious instead of depending on a shell helper.
    print(f"== {version} ==")


def main() -> int:
    parser = argparse.ArgumentParser(description="Unpack archived project material for a version.")
    parser.add_argument("version")
    args = parser.parse_args()

    repo_root = find_repo_root()
    archive_dir = repo_root / "ARCHIVE"
    notes_dir = repo_root / "NOTES"

    if not archive_dir.is_dir():
        print(f"Error: Missing directory {archive_dir}", file=sys.stderr)
        return 1

    version = args.version
    print("Entered archive")
    print_header(version)

    ignore_names = load_simple_gitignore_names(repo_root / ".gitignore")
    matched_archives = sorted(archive_dir.glob(f"*{version}*.zip"))

    for archive_path in matched_archives:
        extracted_dir = archive_dir / archive_path.stem
        if extracted_dir.exists():
            shutil.rmtree(extracted_dir)

        unpack_zip(archive_path, archive_dir)

        if is_rust_snapshot_archive(archive_path):
            copy_tree_contents(extracted_dir, repo_root, ignore_names)
        else:
            notes_dir.mkdir(parents=True, exist_ok=True)
            note_path = extracted_dir.with_suffix(".md")
            destination = notes_dir / note_path.name
            shutil.move(str(note_path), str(destination))
            print(f"Saved note {archive_path.stem}")

        shutil.rmtree(extracted_dir, ignore_errors=True)

    print(f"Unpacked {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())