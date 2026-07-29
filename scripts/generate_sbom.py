#!/usr/bin/env python3
"""Generate and validate a deterministic CycloneDX SBOM from Cargo metadata."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
import urllib.parse
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def run_command(root: Path, args: list[str]) -> str:
    result = subprocess.run(
        args,
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        raise RuntimeError(f"{' '.join(args)} failed with exit code {result.returncode}")
    return result.stdout


def load_lock_checksums(root: Path) -> dict[tuple[str, str], str]:
    with (root / "Cargo.lock").open("rb") as lock_file:
        lock = tomllib.load(lock_file)
    return {
        (package["name"], package["version"]): package["checksum"]
        for package in lock.get("package", [])
        if package.get("checksum")
    }


def package_ref(package: dict[str, Any]) -> str:
    name = urllib.parse.quote(package["name"], safe="")
    version = urllib.parse.quote(package["version"], safe="")
    return f"pkg:cargo/{name}@{version}"


def component_for(
    package: dict[str, Any], checksums: dict[tuple[str, str], str]
) -> dict[str, Any]:
    component: dict[str, Any] = {
        "type": "library",
        "bom-ref": package_ref(package),
        "name": package["name"],
        "version": package["version"],
        "purl": package_ref(package),
    }
    checksum = checksums.get((package["name"], package["version"]))
    if checksum:
        component["hashes"] = [{"alg": "SHA-256", "content": checksum}]
    if package.get("license"):
        component["licenses"] = [{"expression": package["license"]}]
    if package.get("source"):
        component["properties"] = [
            {"name": "cargo:source", "value": package["source"]}
        ]
    return component


def source_timestamp(root: Path) -> str:
    commit_time = run_command(root, ["git", "show", "-s", "--format=%cI", "HEAD"]).strip()
    parsed = datetime.fromisoformat(commit_time.replace("Z", "+00:00"))
    return parsed.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")


def build_sbom(root: Path) -> dict[str, Any]:
    raw_metadata = run_command(
        root,
        ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
    )
    metadata = json.loads(raw_metadata)
    packages = {package["id"]: package for package in metadata["packages"]}
    root_package = packages[metadata["resolve"]["root"]]
    checksums = load_lock_checksums(root)
    components = [
        component_for(package, checksums)
        for package in packages.values()
        if package["id"] != root_package["id"]
    ]
    components.sort(key=lambda item: item["bom-ref"])
    references = {
        package_id: package_ref(package) for package_id, package in packages.items()
    }
    dependencies = []
    for node in metadata["resolve"]["nodes"]:
        dependencies.append(
            {
                "ref": references[node["id"]],
                "dependsOn": sorted(references[dep] for dep in node["dependencies"]),
            }
        )
    dependencies.sort(key=lambda item: item["ref"])
    identity = "\n".join(item["bom-ref"] for item in components)
    serial = uuid.uuid5(uuid.NAMESPACE_URL, f"{package_ref(root_package)}\n{identity}")
    return {
        "$schema": "https://cyclonedx.org/schema/bom-1.6.schema.json",
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "serialNumber": f"urn:uuid:{serial}",
        "version": 1,
        "metadata": {
            "timestamp": source_timestamp(root),
            "component": component_for(root_package, checksums),
            "tools": {
                "components": [
                    {
                        "type": "application",
                        "name": "rappct repository SBOM generator",
                        "version": "1",
                    }
                ]
            },
        },
        "components": components,
        "dependencies": dependencies,
    }


def validate_sbom(sbom: dict[str, Any]) -> list[str]:
    errors = []
    if sbom.get("$schema") != "https://cyclonedx.org/schema/bom-1.6.schema.json":
        errors.append("document must identify the CycloneDX 1.6 JSON schema")
    if sbom.get("bomFormat") != "CycloneDX" or sbom.get("specVersion") != "1.6":
        errors.append("document must identify CycloneDX 1.6")
    if not re.fullmatch(
        r"urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-"
        r"[0-9a-f]{4}-[0-9a-f]{12}",
        sbom.get("serialNumber", ""),
    ):
        errors.append("serialNumber must be a lowercase UUID URN")
    components = sbom.get("components")
    dependencies = sbom.get("dependencies")
    if not isinstance(components, list) or not components:
        errors.append("components must be a non-empty list")
        return errors
    if not isinstance(dependencies, list) or not dependencies:
        errors.append("dependencies must be a non-empty list")
        return errors
    refs = {item.get("bom-ref") for item in components}
    root_ref = sbom.get("metadata", {}).get("component", {}).get("bom-ref")
    all_refs = refs | {root_ref}
    if None in refs or len(refs) != len(components):
        errors.append("component bom-ref values must be present and unique")
    for component in components:
        if component.get("bom-ref") != component.get("purl"):
            errors.append(f"component purl mismatch: {component.get('bom-ref')}")
        for item in component.get("hashes", []):
            if item.get("alg") != "SHA-256" or not re.fullmatch(
                r"[0-9a-f]{64}", item.get("content", "")
            ):
                errors.append(f"invalid SHA-256: {component.get('bom-ref')}")
    dependency_refs = [item.get("ref") for item in dependencies]
    if len(set(dependency_refs)) != len(dependency_refs):
        errors.append("dependency graph refs must be unique")
    for dependency in dependencies:
        if dependency.get("ref") not in all_refs:
            errors.append(f"unknown dependency ref: {dependency.get('ref')}")
        unknown = set(dependency.get("dependsOn", [])) - all_refs
        if unknown:
            errors.append(f"unknown dependsOn refs: {sorted(unknown)}")
    if len(dependencies) != len(all_refs):
        errors.append("dependency graph must contain every component and the root package")
    return errors


def write_sbom(path: Path, sbom: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    content = json.dumps(sbom, indent=2, sort_keys=True) + "\n"
    path.write_text(content, encoding="utf-8", newline="\n")
    digest = hashlib.sha256(content.encode()).hexdigest()
    print(f"SBOM: PASS ({len(sbom['components']) + 1} components)")
    print(f"wrote {path} (sha256:{digest})")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("target/sbom/rappct.cdx.json"),
        help="repository-relative CycloneDX JSON output path",
    )
    parser.add_argument("--validate", type=Path, help="validate an existing SBOM")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parent.parent
    if args.validate:
        sbom = json.loads((root / args.validate).read_text(encoding="utf-8"))
    else:
        sbom = build_sbom(root)
    errors = validate_sbom(sbom)
    if errors:
        for error in errors:
            print(f"SBOM: FAIL: {error}", file=sys.stderr)
        return 1
    if args.validate:
        print(f"SBOM: PASS ({len(sbom['components']) + 1} components)")
    else:
        write_sbom(root / args.output, sbom)
    return 0


if __name__ == "__main__":
    sys.exit(main())
