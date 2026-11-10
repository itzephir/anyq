#!/usr/bin/env python3
"""Enforce the Cargo dependency graph, pure inner layers and named roots."""

import copy
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {
    "anyq-shell-api": {"anyq-queue-api"},
    "anyq-shell-runtime": {
        "anyq-shell-api",
        "anyq-queue-api",
        "anyq-queue-wire",
        "anyq-plugin-wire",
    },
    "anyq-shell-process": {
        "anyq-shell-api",
        "anyq-shell-runtime",
        "anyq-queue-api",
        "rustix",
        "sysinfo",
        "windows-spawn",
        "win32job",
    },
    "anyq-timeout-api": {"anyq-queue-api"},
    "anyq-timeout-impl": {
        "anyq-timeout-api",
        "anyq-queue-api",
        "anyq-queue-wire",
        "anyq-plugin-wire",
    },
    "anyq-queue-api": set(),
    "anyq-queue-wire": {"anyq-queue-api", "serde"},
    "anyq-plugin-api": {"anyq-queue-api"},
    "anyq-plugin-wire": {
        "anyq-plugin-api",
        "anyq-queue-api",
        "anyq-queue-wire",
        "serde",
        "serde_json",
    },
    "anyq-plugin-dispatch": {"anyq-plugin-api", "anyq-queue-api"},
    "anyq-plugin-sqlite": {
        "anyq-plugin-api",
        "anyq-plugin-dispatch",
        "anyq-plugin-wire",
        "anyq-queue-api",
        "anyq-queue-wire",
        "rusqlite",
        "serde",
        "serde_json",
        "tempfile",
    },
    "anyq-plugin-process": {
        "anyq-plugin-api",
        "anyq-plugin-runtime",
        "anyq-queue-api",
        "tokio",
    },
    "anyq-plugin-runtime": {
        "anyq-plugin-api",
        "anyq-plugin-wire",
        "anyq-plugin-dispatch",
        "anyq-queue-api",
        "anyq-queue-wire",
    },
    "anyq-plugin-impl": {
        "anyq-plugin-api",
        "anyq-plugin-dispatch",
        "anyq-plugin-sqlite",
        "anyq-plugin-process",
        "anyq-plugin-runtime",
        "anyq-queue-api",
        "rusqlite",
    },
    "anyq-queue-dispatch": {"anyq-queue-api"},
    "anyq-queue-sqlite": {
        "anyq-queue-wire",
        "serde",
        "serde_json",
        "anyq-queue-metadata",
        "anyq-queue-api",
        "anyq-queue-dispatch",
        "rusqlite",
        "tempfile",
        "anyq-integration-test-hooks",
    },
    "anyq-queue-impl": {
        "anyq-queue-api",
        "anyq-queue-dispatch",
        "anyq-queue-sqlite",
    },
    "anyq-queue-metadata": {
        "anyq-queue-api",
        "serde",
        "serde_json",
        "toml_edit",
        "yaml-rust2",
    },
    "anyq-config-api": set(),
    "anyq-config-dispatch": {"anyq-config-api"},
    "anyq-config-file": {
        "anyq-config-api",
        "anyq-config-dispatch",
        "anyq-integration-test-hooks",
        "toml",
        "tempfile",
        "fs2",
    },
    "anyq-config-impl": {
        "anyq-config-api",
        "anyq-config-dispatch",
        "anyq-config-file",
    },
    "anyq-daemon-api": {
        "anyq-queue-api",
        "anyq-queue-wire",
        "anyq-plugin-api",
        "anyq-plugin-wire",
        "serde",
    },
    "anyq-daemon-transport": {
        "anyq-queue-api",
        "tokio",
        "tempfile",
        "anyq-daemon-api",
        "interprocess",
        "fs2",
        "serde",
        "serde_json",
        "sha2",
        "rustix",
        "widestring",
    },
    "anyq-daemon-client": {
        "anyq-plugin-api",
        "anyq-daemon-api",
        "anyq-daemon-transport",
        "anyq-queue-api",
        "windows-spawn",
    },
    "anyq-daemon-runtime": {
        "anyq-plugin-api",
        "anyq-daemon-api",
        "anyq-daemon-transport",
        "anyq-queue-api",
        "anyq-integration-test-hooks",
    },
    "anyq-daemon-impl": {
        "anyq-queue-dispatch",
        "anyq-plugin-runtime",
        "anyq-plugin-api",
        "anyq-plugin-wire",
        "anyq-plugin-impl",
        "anyq-queue-sqlite",
        "anyq-daemon-api",
        "anyq-daemon-client",
        "anyq-daemon-runtime",
        "anyq-daemon-transport",
        "anyq-queue-api",
        "anyq-queue-impl",
    },
    "anyq-integration-test-hooks": set(),
    "anyq": {
        "anyq-shell-runtime",
        "anyq-shell-process",
        "anyq-timeout-api",
        "anyq-queue-wire",
        "anyq-timeout-impl",
        "anyq-plugin-api",
        "anyq-plugin-wire",
        "anyq-queue-metadata",
        "anyq-daemon-api",
        "anyq-daemon-transport",
        "serde_json",
        "anyq-queue-api",
        "anyq-daemon-impl",
        "anyq-config-api",
        "anyq-config-impl",
        "anyq-integration-test-hooks",
        "ctrlc",
        "clap",
        "rusqlite",
        "tempfile",
    },
    "anyq-architecture-lint": {"syn"},
}
INNER = {
    "anyq-shell-api",
    "anyq-timeout-api",
    "anyq-plugin-api",
    "anyq-plugin-dispatch",
    "anyq-plugin-wire",
    "anyq-queue-wire",
    "anyq-daemon-api",
    "anyq-queue-api",
    "anyq-queue-dispatch",
    "anyq-config-api",
    "anyq-config-dispatch",
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
                and re.search(
                    r"\b(?:anyq_(?:daemon|queue|config|plugin|timeout|shell)_impl|anyq_queue_metadata|anyq_shell_(?:process|runtime))\b",
                    source,
                )
            ):
                errors.append(f"{path}: factory imports belong only to main.rs")

            if name == "anyq" and re.search(r"\banyq_queue_impl\b", source):
                errors.append(f"{path}: CLI queue IO belongs behind the daemon")

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
        ("anyq-queue-api", "serde_json"),
        ("anyq-queue-dispatch", "yaml-rust2"),
        ("anyq-config-api", "anyq-config-file"),
        ("anyq-config-dispatch", "toml"),
        ("anyq-queue-dispatch", "anyq-integration-test-hooks"),
        ("anyq-daemon-api", "interprocess"),
        ("anyq-daemon-runtime", "anyq-queue-impl"),
        ("anyq", "anyq-queue-impl"),
        ("anyq-shell-api", "anyq-shell-process"),
        ("anyq-shell-runtime", "anyq-queue-sqlite"),
        ("anyq-plugin-api", "anyq-plugin-sqlite"),
        ("anyq-plugin-dispatch", "rusqlite"),
        ("anyq-plugin-runtime", "anyq-queue-sqlite"),
        ("anyq-plugin-runtime", "anyq-queue-impl"),
        ("anyq", "anyq-plugin-sqlite"),
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
        for inner in INNER:
            mutant = copy.deepcopy(sources)
            mutant[inner]["negative.rs"] = bad
            assert check(graph, mutant), (inner, bad)
    mutant = copy.deepcopy(sources)
    mutant["anyq"]["apps/cli/src/controllers.rs"] = "use anyq_queue_impl::open;"
    assert check(graph, mutant)
    mutant = copy.deepcopy(sources)
    mutant["anyq"]["apps/cli/src/controllers.rs"] = (
        "use anyq_queue_metadata::parse_json;"
    )
    assert check(graph, mutant)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Architecture OK; negative fixtures rejected")
    return 0


if __name__ == "__main__":
    sys.exit(main())
