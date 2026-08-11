"""Fail when Cargo resolves an unreviewed duplicate dependency family."""

from __future__ import annotations

import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

# windows-rs still uses syn 2 while current derive dependencies use syn 3.
ALLOWED_MAJOR_VERSIONS = {"syn": {"2", "3"}}


def cargo_packages(root: Path) -> list[dict[str, object]]:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        raise RuntimeError("cargo metadata failed")
    return json.loads(result.stdout)["packages"]


def duplicate_majors(packages: list[dict[str, object]]) -> dict[str, set[str]]:
    versions: dict[str, set[str]] = defaultdict(set)
    for package in packages:
        name = str(package["name"])
        major = str(package["version"]).split(".", maxsplit=1)[0]
        versions[name].add(major)
    return {name: majors for name, majors in versions.items() if len(majors) > 1}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    duplicates = duplicate_majors(cargo_packages(root))
    if duplicates == ALLOWED_MAJOR_VERSIONS:
        print("Duplicate dependency policy: PASS (reviewed syn 2.x/3.x split only)")
        return 0
    print("Duplicate dependency policy: FAIL")
    print(f"Expected: {ALLOWED_MAJOR_VERSIONS}")
    print(f"Resolved: {duplicates}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
