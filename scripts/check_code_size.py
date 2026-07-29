"""Repository-local code size guard for maintained source and tooling."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

FILE_LIMIT = 500
FUNCTION_LIMIT = 75
EXCLUDED_DIRS = {
    ".git",
    ".cargo-home",
    ".rustup-home",
    "target",
    "book",
    ".tmp",
    ".cache",
    ".agent-logs",
    ".worktrees",
}
DOC_SUFFIXES = {".md", ".txt"}
SCRIPT_SUFFIXES = {".ps1", ".py", ".sh", ".cjs", ".js"}
CONFIG_SUFFIXES = {".toml", ".yml", ".yaml"}
RUST_SUFFIXES = {".rs"}
LOCKFILES = {"Cargo.lock"}
LOGFILES = {"ci-local.log"}


@dataclass
class FileRecord:
    path: str
    category: str
    lines: int
    enforced: bool
    exception: str = ""


@dataclass
class SymbolRecord:
    path: str
    symbol: str
    start: int
    end: int
    lines: int
    limit: int = FUNCTION_LIMIT


def run_git(args: list[str], root: Path) -> list[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return []
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def discover_files(root: Path) -> list[Path]:
    names = set(run_git(["ls-files"], root))
    names.update(run_git(["ls-files", "--others", "--exclude-standard"], root))
    if not names:
        names.update(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file())
    paths = []
    for name in sorted(names):
        path = root / name
        if should_skip_path(path.relative_to(root)):
            continue
        if path.is_file():
            paths.append(path)
    return paths


def should_skip_path(relative: Path) -> bool:
    return any(part in EXCLUDED_DIRS for part in relative.parts)


def classify(relative: Path) -> tuple[str, bool, str]:
    path = relative.as_posix()
    name = relative.name
    suffix = relative.suffix.lower()
    if path.startswith("legacy/"):
        return "legacy", False, "legacy archived material"
    if path.startswith(".AGENTS/") or path.startswith("AGENTS/"):
        return "agent-notes", False, "agent worklog or historical plan"
    if name in LOCKFILES:
        return "lockfile", False, "package-manager lockfile"
    if name in LOGFILES:
        return "log", False, "captured command log"
    if suffix in DOC_SUFFIXES:
        if name == "AGENTS.md":
            return "agent-instructions", True, ""
        return "documentation", False, "documentation is inspected but not executable logic"
    if suffix in RUST_SUFFIXES:
        if path.startswith("tests/"):
            return "test-source", True, ""
        if path.startswith("examples/"):
            return "example-source", True, ""
        return "production-source", True, ""
    if suffix in SCRIPT_SUFFIXES or path.startswith(".githooks/"):
        return "scripts-tooling", True, ""
    if suffix in CONFIG_SUFFIXES or name in {"Justfile", "deny.toml"}:
        return "executable-config", True, ""
    return "other", False, "not source, test, script, or executable configuration"


def count_logical(lines: list[str], suffix: str) -> int:
    count = 0
    in_block = False
    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        if suffix in {".rs", ".cjs", ".js"}:
            stripped, in_block = strip_c_style_comment(stripped, in_block)
            if not stripped or stripped.startswith("//") or stripped.startswith("*"):
                continue
        elif suffix == ".py" and stripped.startswith("#"):
            continue
        elif suffix in {".ps1", ".sh"} and stripped.startswith("#"):
            continue
        count += 1
    return count


def strip_c_style_comment(line: str, in_block: bool) -> tuple[str, bool]:
    remaining = line
    output = ""
    while remaining:
        if in_block:
            end = remaining.find("*/")
            if end < 0:
                return output.strip(), True
            remaining = remaining[end + 2 :]
            in_block = False
            continue
        line_comment = remaining.find("//")
        block_comment = remaining.find("/*")
        if line_comment >= 0 and (block_comment < 0 or line_comment < block_comment):
            output += remaining[:line_comment]
            return output.strip(), False
        if block_comment >= 0:
            output += remaining[:block_comment]
            remaining = remaining[block_comment + 2 :]
            in_block = True
            continue
        output += remaining
        break
    return output.strip(), in_block


def sanitized_lines(lines: list[str]) -> list[str]:
    clean = []
    in_block = False
    for line in lines:
        stripped, in_block = strip_c_style_comment(line, in_block)
        clean.append(re.sub(r'"(?:\\.|[^"\\])*"', '""', stripped))
    return clean


def brace_span(lines: list[str], start: int, column: int) -> int | None:
    depth = 0
    started = False
    for index in range(start, len(lines)):
        segment = lines[index][column:] if index == start else lines[index]
        for char in segment:
            if char == "{":
                depth += 1
                started = True
            elif char == "}":
                depth -= 1
                if started and depth == 0:
                    return index
        column = 0
    return None


def rust_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    rel = path.as_posix()
    clean = sanitized_lines(lines)
    records: list[SymbolRecord] = []
    pending: tuple[int, str] | None = None
    for index, line in enumerate(clean):
        match = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", line)
        if match and not line.lstrip().startswith("//"):
            pending = (index, match.group(1))
        elif pending is None:
            closure = re.search(r"\|[^|]*\|\s*(?:->[^{}]+)?\{", line)
            if closure:
                end = brace_span(clean, index, line.find("{"))
                if end is not None:
                    records.append(make_symbol(rel, "<closure>", index, end, lines))
            continue
        if pending is not None and "{" in line:
            start, name = pending
            end = brace_span(clean, index, line.find("{"))
            if end is not None:
                records.append(make_symbol(rel, name, start, end, lines))
            pending = None
        elif pending is not None and ";" in line:
            pending = None
    return records


def make_symbol(path: str, name: str, start: int, end: int, lines: list[str]) -> SymbolRecord:
    suffix = Path(path).suffix.lower()
    logical = count_logical(lines[start : end + 1], suffix)
    return SymbolRecord(path, name, start + 1, end + 1, logical)


def brace_language_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    rel = path.as_posix()
    clean = sanitized_lines(lines)
    records: list[SymbolRecord] = []
    for index, line in enumerate(clean):
        match = re.search(r"\bfunction\s+([A-Za-z_$][\w$-]*)\s*(?:\([^)]*\))?\s*\{", line)
        if not match:
            match = re.search(r"\b(?:const|let|var)\s+([A-Za-z_$][\w$-]*)\s*=\s*\([^)]*\)\s*=>\s*\{", line)
        if match:
            end = brace_span(clean, index, line.find("{"))
            if end is not None:
                records.append(make_symbol(rel, match.group(1), index, end, lines))
    records.extend(top_level_symbol(rel, lines, records))
    return records


def powershell_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    rel = path.as_posix()
    records: list[SymbolRecord] = []
    for index, line in enumerate(lines):
        match = re.match(r"\s*function\s+([A-Za-z_][\w-]*)\s*\{", line, re.IGNORECASE)
        if match:
            end = brace_span(lines, index, line.find("{"))
            if end is not None:
                records.append(make_symbol(rel, match.group(1), index, end, lines))
    records.extend(top_level_symbol(rel, lines, records))
    return records


def python_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    rel = path.as_posix()
    records: list[SymbolRecord] = []
    for index, line in enumerate(lines):
        match = re.match(r"^(\s*)(?:async\s+)?def\s+([A-Za-z_][\w]*)\s*\(", line)
        if not match:
            continue
        indent = len(match.group(1).replace("\t", "    "))
        end = find_python_end(lines, index + 1, indent)
        records.append(make_symbol(rel, match.group(2), index, end, lines))
    records.extend(top_level_symbol(rel, lines, records))
    return records


def find_python_end(lines: list[str], start: int, indent: int) -> int:
    end = start - 1
    for index in range(start, len(lines)):
        stripped = lines[index].strip()
        if not stripped or stripped.startswith("#"):
            continue
        current = len(lines[index]) - len(lines[index].lstrip(" "))
        if current <= indent:
            return max(end, start - 1)
        end = index
    return max(end, start - 1)


def shell_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    rel = path.as_posix()
    records: list[SymbolRecord] = []
    for index, line in enumerate(lines):
        match = re.match(r"\s*([A-Za-z_][\w-]*)\s*\(\)\s*\{", line)
        if match:
            end = brace_span(lines, index, line.find("{"))
            if end is not None:
                records.append(make_symbol(rel, match.group(1), index, end, lines))
    records.extend(top_level_symbol(rel, lines, records))
    return records


def top_level_symbol(path: str, lines: list[str], symbols: list[SymbolRecord]) -> list[SymbolRecord]:
    covered = set()
    for symbol in symbols:
        covered.update(range(symbol.start - 1, symbol.end))
    outside = [line for index, line in enumerate(lines) if index not in covered]
    logical = count_logical(outside, Path(path).suffix.lower())
    if logical > FUNCTION_LIMIT:
        return [SymbolRecord(path, "<top-level>", 1, len(lines), logical)]
    return []


def just_symbols(path: Path, lines: list[str]) -> list[SymbolRecord]:
    records: list[SymbolRecord] = []
    rel = path.as_posix()
    starts = []
    for index, line in enumerate(lines):
        if re.match(r"^[A-Za-z0-9_.-][A-Za-z0-9_.\s${}:'\"-]*:\s*$", line):
            starts.append(index)
    for offset, start in enumerate(starts):
        end = starts[offset + 1] - 1 if offset + 1 < len(starts) else len(lines) - 1
        name = lines[start].split(":", 1)[0].strip()
        records.append(make_symbol(rel, name, start, end, lines))
    return records


def symbols_for(path: Path, lines: list[str]) -> list[SymbolRecord]:
    suffix = path.suffix.lower()
    if suffix == ".rs":
        return rust_symbols(path, lines)
    if suffix == ".ps1":
        return powershell_symbols(path, lines)
    if suffix == ".py":
        return python_symbols(path, lines)
    if suffix == ".sh":
        return shell_symbols(path, lines)
    if suffix in {".js", ".cjs"}:
        return brace_language_symbols(path, lines)
    if path.name == "Justfile":
        return just_symbols(path, lines)
    return []


def scan(root: Path) -> dict[str, object]:
    files: list[FileRecord] = []
    file_violations: list[FileRecord] = []
    symbol_violations: list[SymbolRecord] = []
    exceptions: list[FileRecord] = []
    for path in discover_files(root):
        relative = path.relative_to(root)
        text = path.read_text(encoding="utf-8", errors="replace")
        lines = text.splitlines()
        category, enforced, exception = classify(relative)
        record = FileRecord(relative.as_posix(), category, len(lines), enforced, exception)
        files.append(record)
        if enforced and len(lines) > FILE_LIMIT:
            file_violations.append(record)
        if enforced:
            for symbol in symbols_for(relative, lines):
                if symbol.lines > FUNCTION_LIMIT:
                    symbol_violations.append(symbol)
        elif exception:
            exceptions.append(record)
    return {
        "limits": {"file": FILE_LIMIT, "function": FUNCTION_LIMIT},
        "totals": {
            "files": len(files),
            "maintained_files": sum(1 for item in files if item.enforced),
            "file_violations": len(file_violations),
            "function_violations": len(symbol_violations),
            "exceptions": len(exceptions),
        },
        "files": [asdict(item) for item in files],
        "file_violations": [asdict(item) for item in file_violations],
        "function_violations": [asdict(item) for item in symbol_violations],
        "exceptions": [asdict(item) for item in exceptions],
    }


def print_text(report: dict[str, object]) -> None:
    totals = report["totals"]
    print(
        "scanned {maintained_files} maintained files out of {files}; "
        "{file_violations} file violation(s), {function_violations} symbol violation(s)".format(
            **totals
        )
    )
    print_section("file violations", report["file_violations"])
    print_section("function violations", report["function_violations"])


def print_section(title: str, rows: object) -> None:
    print(f"\n{title}:")
    if not rows:
        print("  none")
        return
    for row in rows:
        if "symbol" in row:
            print(
                "  {path}:{start}-{end} {symbol} has {lines}/{limit} logical lines".format(
                    **row
                )
            )
        else:
            print("  {path} has {lines}/500 physical lines [{category}]".format(**row))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    report = scan(root)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    totals = report["totals"]
    return 1 if totals["file_violations"] or totals["function_violations"] else 0


if __name__ == "__main__":
    sys.exit(main())
