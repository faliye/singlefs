import sys
tsv, prefix, out = sys.argv[1:4]
rows = [l.rstrip('\n') for l in open(tsv, encoding='utf-8') if l.strip() and not l.startswith('#')]
sel = [r for r in rows if r.startswith(prefix)]
assert len(sel) == 1, len(sel)
r = sel[0].split('\t')
open(out + '.rel', 'w').write(r[1]); open(out + '.old', 'w').write(r[2].replace('\\n', '\n'))
open(out + '.new', 'w').write(r[3].replace('\\n', '\n')); open(out + '.args', 'w').write(r[4]); open(out + '.name', 'w').write(r[0])
