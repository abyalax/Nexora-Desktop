#!/usr/bin/env python3
"""BASE-03: fail when a `*.rs` file under `crates/*/src` has no `mod` declaration.

A file counts as declared when its parent module file (`mod.rs`, `<parent>.rs`, `lib.rs` or `main.rs`) contains
`mod <name>;` (any visibility or `#[cfg]` before it), or a `#[path = "..."]` attribute that resolves to it.
Crate roots (`lib.rs`, `main.rs`, `src/bin/*.rs`) are exempt. Output is sorted, so identical input gives identical
output. Exit status: 0 clean, 1 orphans found.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MOD_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", re.M)
PATH_RE = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;')


def strip_comments(src: str) -> str:
  src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
  return re.sub(r"//[^\n]*", "", src)


def declared_in(module_file: Path) -> tuple[set[str], set[Path]]:
  """Names declared by `mod x;` and files pulled in by `#[path]` inside `module_file`."""
  if not module_file.is_file():
    return set(), set()
  text = strip_comments(module_file.read_text(encoding="utf-8-sig"))
  names = set(MOD_RE.findall(text))
  paths = {(module_file.parent / p).resolve() for p in PATH_RE.findall(text)}
  return names, paths


def parent_candidates(rs: Path, src: Path) -> list[Path]:
  """Files that may declare `rs`."""
  d = rs.parent
  if rs.name == "mod.rs":
    # a/b/mod.rs is declared by a/mod.rs or a.rs
    grand = d.parent
    return [grand / "mod.rs", grand.parent / f"{grand.name}.rs"] if grand != src else [src / "lib.rs", src / "main.rs"]
  if d == src:
    return [src / "lib.rs", src / "main.rs"]
  return [d / "mod.rs", d.parent / f"{d.name}.rs"]


def main() -> int:
  orphans = []
  for src in sorted(ROOT.glob("crates/*/src")):
    for rs in sorted(src.rglob("*.rs")):
      rel = rs.relative_to(src)
      if rel.parts[0] == "bin" or rs.name in ("lib.rs", "main.rs") and rs.parent == src:
        continue
      name = rs.parent.name if rs.name == "mod.rs" else rs.stem
      found = False
      for cand in parent_candidates(rs, src):
        names, paths = declared_in(cand)
        if name in names or rs.resolve() in paths:
          found = True
          break
      if not found:
        orphans.append(rs.relative_to(ROOT).as_posix())
  for o in sorted(orphans):
    print(f"orphan: {o}")
  if orphans:
    print(f"{len(orphans)} orphan file(s): declare with `mod`, or delete (see TODO.md BASE-02/03)", file=sys.stderr)
    return 1
  print("no orphan .rs files")
  return 0


if __name__ == "__main__":
  sys.exit(main())
