# 独立数出的「要推几次」(needs) 与推环实发次数 (pushedN) 逐条比：自动推环行 NS[...needs=Some(X)]|pushedY 与显式 P[...needs=Some(X)...]pushedY[...]
import glob, re, os, collections
root = os.path.dirname(os.path.abspath(__file__)) + '/../logs/'
c = collections.Counter()
for f in sorted(glob.glob(root + 'sweeps/*.log') + glob.glob(root + 'final/*.log')):
    nm = os.path.basename(f)[:-4]
    if nm.startswith('ctl') or nm.startswith('base-raise') or nm.startswith('mutant') or nm.startswith('k'): continue
    for line in open(f, errors='replace'):
        if 'DIAG' not in line: continue
        for m in re.finditer(r'S\(.*?\);hold=\d+;isoC=\d+;needs=Some\((\d+)\)\]\|pushed(\d+)', line):
            c['自动推环行', m.group(1) == m.group(2)] += 1
        for m in re.finditer(r'P\[cur,rOcc=.*?,needs=Some\((\d+)\),.*?\]pushed(\d+)\[', line):
            c['显式P行', m.group(1) == m.group(2)] += 1
for k, v in sorted(c.items()): print(v, k)
