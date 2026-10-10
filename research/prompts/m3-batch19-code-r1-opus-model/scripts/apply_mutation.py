#!/usr/bin/env python3
"""apply_mutation.py <copy-root> <frozen-root> <tsv> <row-substring>|--restore <file>
Restores file from frozen tree then applies the mutation row (columns: name, file, old, new, ...), old/new with \\n escapes.
"""
import sys, shutil, pathlib
copy, frozen = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
if sys.argv[3] == '--restore':
    rel = sys.argv[4]
    shutil.copyfile(frozen/rel, copy/rel); print('restored', rel); sys.exit(0)
tsv, key = sys.argv[3], sys.argv[4]
rows = [l.rstrip('\n').split('\t') for l in open(tsv, encoding='utf-8') if key in l.split('\t')[0]]
assert len(rows) == 1, f'{len(rows)} rows match {key!r}'
name, rel, old, new = rows[0][:4]
dec = lambda s: s.replace('\\n', '\n').replace('\\t', '\t')
old, new = dec(old), dec(new)
shutil.copyfile(frozen/rel, copy/rel)
text = (copy/rel).read_text(encoding='utf-8')
assert text.count(old) == 1, f'old occurs {text.count(old)} times in {rel}'
(copy/rel).write_text(text.replace(old, new), encoding='utf-8')
print('applied', name, 'to', rel)
