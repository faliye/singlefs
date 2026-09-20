#!/usr/bin/env python3
"""apply-mutation.py <copy-root> <id>: apply one row of model/mutants.tsv; the old text must hit exactly once."""
import sys, pathlib
root, mid = pathlib.Path(sys.argv[1]), sys.argv[2]
table = pathlib.Path(__file__).with_name('mutants.tsv')
for line in table.read_text().splitlines():
    if line.startswith('#') or not line.strip():
        continue
    i, name, f, old, new = line.split('\t')
    if i != mid:
        continue
    old, new = old.replace('\\n', '\n'), new.replace('\\n', '\n')
    p = root / f
    s = p.read_text()
    n = s.count(old)
    if n != 1:
        sys.exit(f'{mid}: old text hits {n} times in {f}')
    p.write_text(s.replace(old, new))
    print(f'{mid} applied to {f}: {name}')
    break
else:
    sys.exit(f'{mid}: not in table')
