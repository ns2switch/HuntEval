#!/usr/bin/env python3
"""Reject unexpected or generated files in the R7 Python wheel."""

import pathlib
import sys
import zipfile


def inventory(path: pathlib.Path) -> dict[str, bytes]:
    with zipfile.ZipFile(path) as archive:
        return {name: archive.read(name) for name in archive.namelist()}


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print("error: expected one or two wheel paths", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1])
    if not path.is_file() or path.suffix != ".whl":
        print("error: wheel path is invalid", file=sys.stderr)
        return 1
    contents = inventory(path)
    names = list(contents)
    if not names or len(names) > 64:
        print("error: wheel inventory is empty or unbounded", file=sys.stderr)
        return 1
    for name in names:
        safe = (
            not name.startswith("/")
            and ".." not in pathlib.PurePosixPath(name).parts
            and (name.startswith("hunteval_sdk/") or ".dist-info/" in name)
            and "__pycache__" not in name
            and not name.endswith((".pyc", ".pyo"))
        )
        if not safe:
            print("error: wheel contains an unexpected path", file=sys.stderr)
            return 1
    if len(sys.argv) == 3:
        second = pathlib.Path(sys.argv[2])
        if not second.is_file() or inventory(second) != contents:
            print("error: repeated wheel contents are not equivalent", file=sys.stderr)
            return 1
    print(f"Python wheel inventory is valid and reproducible ({len(names)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
