#!/usr/bin/env python3
"""Check vendored Quiet Glass library equality against sibling repo paths."""
import sys
from pathlib import Path


def main():
    source = Path(__file__).resolve().parents[1] / "crates/krub-icons"
    if len(sys.argv) < 2:
        print("usage: python3 packaging/check-icon-sync.py sibling-repo [sibling-repo ...]")
        return 2
    ok = True
    for repo in sys.argv[1:]:
        target = Path(repo) / "crates/krub-icons"
        # Finder metadata is local UI state, not part of the vendored library.
        source_files = {p.relative_to(source) for p in source.rglob("*") if p.is_file() and p.name != ".DS_Store"}
        target_files = {p.relative_to(target) for p in target.rglob("*") if p.is_file() and p.name != ".DS_Store"}
        if source_files != target_files:
            print(f"File inventory differs: {target}")
            ok = False
        for relative in sorted(source_files):
            file = target / relative
            if not file.is_file() or file.read_bytes() != (source / relative).read_bytes():
                print(f"DIFF: {file}")
                ok = False
    if ok:
        print("Quiet Glass libraries are identical")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
