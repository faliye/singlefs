"""读 W3R3S2 输出，按候选列出各计数。用法：python3 -I table.py <kinds,逗号分隔> <输出文件…>"""
import re, sys
kinds = sys.argv[1].split(',')
for path in sys.argv[2:]:
    model = None; rows = []
    for line in open(path):
        if line.startswith('W3R3S2MODEL'):
            model = re.sub(r' elapsed_s=\S+', '', line.strip())
        elif line.startswith('W3R3S2COUNT'):
            f = dict(kv.split('=', 1) for kv in line.split() if '=' in kv)
            rows.append([f['form']] + [f[k] for k in kinds])
    print(model)
    w = [max(len(r[i]) for r in rows + [['form'] + kinds]) for i in range(len(kinds) + 1)]
    print('  '.join(h.ljust(w[i]) for i, h in enumerate(['form'] + kinds)))
    for r in rows:
        print('  '.join(c.ljust(w[i]) for i, c in enumerate(r)))
