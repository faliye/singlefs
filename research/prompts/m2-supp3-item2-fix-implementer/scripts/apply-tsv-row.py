#!/usr/bin/env python3
"""apply-tsv-row.py <copy-root> <tsv-file> <mutation-name>: apply one row of a mutations.tsv-shaped table (name, file, old, new, ...)
to the copy, with \\n in old/new turned into newlines (same convention as crates/mutations.tsv)."""
import sys, pathlib, os
root, tsv, name = sys.argv[1:4]
for line in pathlib.Path(tsv).read_text(encoding='utf-8').splitlines():
    if not line.strip() or line.startswith('#'):
        continue
    parts = line.split('\t')
    if parts[0] != name:
        continue
    rel, old, new = parts[1], parts[2].replace('\\n', '\n'), parts[3].replace('\\n', '\n')
    p = pathlib.Path(root) / rel
    s = p.read_text(encoding='utf-8')
    n = s.count(old)
    if n != 1:
        sys.exit(f"{rel}: anchor count {n} for {name}")
    p.write_text(s.replace(old, new), encoding='utf-8')
    os.utime(p, None)
    print(f"applied {name} to {rel}")
    sys.exit(0)
sys.exit(f"no row named {name}")
