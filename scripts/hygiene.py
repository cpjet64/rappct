"""Portable repository hygiene checks used by local and hosted CI."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

BINARY_EXTENSIONS = {
    ".bin",
    ".dll",
    ".exe",
    ".gif",
    ".ico",
    ".jar",
    ".jpeg",
    ".jpg",
    ".lib",
    ".obj",
    ".pdb",
    ".pdf",
    ".png",
    ".wasm",
    ".zip",
}
LARGE_FILE_LIMIT = 10 * 1024 * 1024
CONFLICT_MARKER = re.compile(rb"(?m)^(<<<<<<<|>>>>>>>|=======$)")
ACTION_USE = re.compile(r"^\s*(?:-\s*)?uses:\s*([^@\s]+)@([^\s#]+)(?:\s+#\s+(\S+))?")
FULL_COMMIT_SHA = re.compile(r"^[0-9a-f]{40}$")


def tracked_files(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr.decode("utf-8", errors="replace"))
        raise RuntimeError("git ls-files failed")
    names = [name for name in result.stdout.decode("utf-8").split("\0") if name]
    return [root / name for name in names]


def is_binary_by_extension(path: Path) -> bool:
    return path.suffix.lower() in BINARY_EXTENSIONS


def test_large_files(files: list[Path], root: Path) -> bool:
    large_files = []
    for path in files:
        if not path.exists() or path.stat().st_size <= LARGE_FILE_LIMIT:
            continue
        size_mb = path.stat().st_size / (1024 * 1024)
        large_files.append(f"  {path.relative_to(root).as_posix()} ({size_mb:.2f} MB)")
    return print_result("Large files (>10MB)", large_files)


def test_nul_bytes(files: list[Path], root: Path) -> bool:
    nul_files = []
    for path in files:
        if not path.exists() or is_binary_by_extension(path):
            continue
        if b"\0" in path.read_bytes():
            nul_files.append(f"  {path.relative_to(root).as_posix()}")
    return print_result("Tracked text file NUL bytes", nul_files)


def test_conflict_markers(files: list[Path], root: Path) -> bool:
    marked_files = []
    for path in files:
        if not path.exists() or is_binary_by_extension(path):
            continue
        if CONFLICT_MARKER.search(path.read_bytes()):
            marked_files.append(f"  {path.relative_to(root).as_posix()}")
    return print_result("Merge conflict markers", marked_files)


def test_required_files(root: Path) -> bool:
    missing_files = []
    for name in [".gitignore"]:
        if not (root / name).exists():
            missing_files.append(f"  {name}")
    return print_result("Required files", missing_files)


def test_action_pins(files: list[Path], root: Path) -> bool:
    failures = []
    workflows = [
        path
        for path in files
        if path.parent.parent == root / ".github" and path.parent.name == "workflows"
    ]
    for path in workflows:
        lines = path.read_text(encoding="utf-8").splitlines()
        for line_number, line in enumerate(lines, 1):
            match = ACTION_USE.match(line)
            if not match or match.group(1).startswith(("./", "docker://")):
                continue
            action, revision, version = match.groups()
            relative = path.relative_to(root).as_posix()
            if not FULL_COMMIT_SHA.fullmatch(revision):
                failures.append(f"  {relative}:{line_number}: {action} is not SHA-pinned")
            elif not version:
                failures.append(f"  {relative}:{line_number}: {action} lacks update metadata")
    return print_result("GitHub Action immutable pins", failures)


def print_result(label: str, failures: list[str]) -> bool:
    if not failures:
        print(f"{label}: PASS")
        return True
    print(f"{label}: FAIL")
    for failure in failures:
        print(failure)
    return False


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    files = tracked_files(root)
    checks = [
        test_large_files(files, root),
        test_nul_bytes(files, root),
        test_conflict_markers(files, root),
        test_required_files(root),
        test_action_pins(files, root),
    ]
    if all(checks):
        print("All hygiene checks passed")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
