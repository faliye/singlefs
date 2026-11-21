import sys
root = sys.argv[1]
names = sys.argv[2:]
rows = {}
for lineno, line in enumerate(open(root + '/crates/singlefs-replay-prefix/verus-must-red-worlds.tsv', encoding='utf-8'), 1):
    if line.startswith('#') or not line.strip():
        continue
    f = line.rstrip('\n').split('\t')
    rows[f[0]] = (lineno, f)
for n in names:
    lineno, f = rows[n]
    src = open(root + '/' + f[1], encoding='utf-8').read()
    orig = f[2].replace('\\n', '\n')
    hits = src.count(orig)
    # line number of first hit
    idx = src.find(orig)
    ln = src[:idx].count('\n') + 1 if idx >= 0 else -1
    print(f"{lineno}\t{n}\tsegments={len(f)}\thits={hits}\tsource_line={ln}\t{f[4]}\t{f[5]}")
