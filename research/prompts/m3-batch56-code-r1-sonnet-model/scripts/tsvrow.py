import sys, re
tsv = sys.argv[1]; idx = int(sys.argv[2]); out = sys.argv[3]
rows = [l.rstrip('\n') for l in open(tsv, encoding='utf-8') if l.strip() and not l.startswith('#')]
sel = [r for r in rows if re.match(r'^第 ?56 ?批', r)]
r = sel[idx].split('\t')
open(out + '.rel', 'w').write(r[1])
open(out + '.old', 'w').write(r[2].replace('\\n', '\n'))
open(out + '.new', 'w').write(r[3].replace('\\n', '\n'))
open(out + '.args', 'w').write(r[4])
open(out + '.name', 'w').write(r[0])
print(len(sel), r[0][:80], '|', r[4][:100])
