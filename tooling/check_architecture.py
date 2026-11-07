#!/usr/bin/env python3
"""Enforce the Cargo dependency graph, pure inner layers and named roots."""

import copy
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {
    "anyq-queue-api": set(),
    "anyq-queue-dispatch": {"anyq-queue-api"},
    "anyq-queue-sqlite": {
        "anyq-queue-api",
        "anyq-queue-dispatch",
        "rusqlite",
        "tempfile",
    },
    "anyq-queue-impl": {
        "anyq-queue-api",
        "anyq-queue-dispatch",
        "anyq-queue-sqlite",
    },
    "anyq": {
        "anyq-queue-api",
        "anyq-queue-impl",
        "clap",
        "rusqlite",
        "tempfile",
    },
    "anyq-architecture-lint": {"syn"},
}
INNER = {
    "anyq-queue-api",
    "anyq-queue-dispatch",
}
IO = re.compile(
    r"\bstd\s*::\s*(?:fs|io|net|process|thread|env)\b|\b(?:SystemTime|Instant)\b|\b(?:include|include_str|include_bytes)\s*!|#\s*\[\s*path\s*=|\b(?:unsafe|extern)\b|\b(?:print|println|eprint|eprintln|dbg)\s*!|#\s*\[\s*link\b"
)


def check(graph, sources):
    errors = []
    if set(graph) != set(ALLOWED):
        errors.append(
            "workspace crate inventory changed; review architecture ownership"
        )
    for name, dependencies in graph.items():
        forbidden = dependencies - ALLOWED.get(name, set())
        if forbidden:
            errors.append(f"{name}: forbidden dependencies {sorted(forbidden)}")
    for name, files in sources.items():
        for path, raw in files.items():
            # Comments are documentation, not imports. Strings still get checked
            # conservatively to avoid hiding include! or IO in macros.
            source = re.sub(r"//[^\n]*|/\*.*?\*/", "", raw, flags=re.S)
            if name in INNER and (
                IO.search(source)
                or re.search(
                    r"\bstd\s*::\s*\{[^}]*\b(?:fs|io|net|process|thread|env)\b",
                    source,
                    re.S,
                )
            ):
                errors.append(f"{path}: IO/FFI/external include in an inner layer")
            if (
                name == "anyq"
                and path != "apps/cli/src/main.rs"
                and re.search(r"\banyq_queue_impl\b", source)
            ):
                errors.append(f"{path}: factory imports belong only to main.rs")

    # Cargo rejects cycles too; check here for readable diagnostics/negative tests.
    def visit(name, trail):
        if name in trail:
            errors.append("dependency cycle: " + " -> ".join(trail + [name]))
            return
        for dep in graph.get(name, set()):
            if dep in graph:
                visit(dep, trail + [name])

    for name in graph:
        visit(name, [])
    return errors


def dependency_items(manifest):
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        for key, value in manifest.get(section, {}).items():
            yield key, value
    for target in manifest.get("target", {}).values():
        yield from dependency_items(target)


def main():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    graph, sources, paths, errors = {}, {}, {}, []
    for member in workspace["workspace"]["members"]:
        path = ROOT / member
        manifest = tomllib.loads((path / "Cargo.toml").read_text())
        name = manifest["package"]["name"]
        if name in graph:
            errors.append(f"duplicate package {name}")
        deps = list(dependency_items(manifest))
        graph[name] = {
            v.get("package", key) if isinstance(v, dict) else key for key, v in deps
        }
        paths[name] = path.resolve()
        if "build" in manifest["package"] or (path / "build.rs").exists():
            errors.append(f"{name}: unreviewed build script")
        sources[name] = {
            p.relative_to(ROOT).as_posix(): p.read_text(encoding="utf-8")
            for p in path.rglob("*.rs")
        }
    for member in workspace["workspace"]["members"]:
        path = ROOT / member
        manifest = tomllib.loads((path / "Cargo.toml").read_text())
        for key, value in dependency_items(manifest):
            dep = value.get("package", key) if isinstance(value, dict) else key
            if dep in paths and (
                not isinstance(value, dict)
                or "path" not in value
                or (path / value["path"]).resolve() != paths[dep]
            ):
                errors.append(
                    f"{member}: dependency {dep} must resolve to its workspace crate"
                )
    errors += check(graph, sources)
    # These fixtures must fail, preventing a checker that only passes today's graph.
    for owner, dep in [
        ("anyq-queue-api", "anyq-queue-dispatch"),
        ("anyq-queue-dispatch", "rusqlite"),
        ("anyq-queue-api", "anyq-queue-impl"),
    ]:
        mutant = copy.deepcopy(graph)
        mutant[owner].add(dep)
        assert check(mutant, sources), (owner, dep)
    for bad in [
        "use std::fs;",
        "use std::{process::Command};",
        'include!("outside.rs");',
        '#[path="outside.rs"] mod bypass;',
    ]:
        # Nested std imports are separately prohibited below, including aliases.
        mutant = copy.deepcopy(sources)
        mutant["anyq-queue-api"]["negative.rs"] = bad
        assert check(graph, mutant), bad
    mutant = copy.deepcopy(sources)
    mutant["anyq"]["apps/cli/src/controllers.rs"] = "use anyq_queue_impl::open;"
    assert check(graph, mutant)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Architecture OK; negative fixtures rejected")
    return 0


if __name__ == "__main__":
    sys.exit(main())
