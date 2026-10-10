# 每个日志里「报被宽护卡住、随后自动推环」的次数，按推完重试写得下 / 仍被拒分开（只数每行第一个 NS）
import glob, re, os, collections
root = os.path.dirname(os.path.abspath(__file__)) + '/../logs/'
for f in sorted(glob.glob(root + 'sweeps/*.log') + glob.glob(root + 'final/*.log')):
    nm = os.path.basename(f)[:-4]
    if nm.startswith('ctl') or nm.startswith('base-raise') or nm.startswith('mutant'): continue
    c = collections.Counter()
    for line in open(f, errors='replace'):
        if 'DIAG' not in line or 'PUDIAG' in line or 'CRASHDIAG' in line: continue
        m = re.search(r'step=\d+[A-Za-z] (NS\[[^|]*)', line)
        if not m or ';S(' not in m.group(1): continue
        r = re.search(r'\|(pushed\d+|pushRef:[^|]*)\|stillIso=(\d+)/(\d+)\|(retry\S*)', line)
        if r: c['retryOk' if r.group(4).startswith('retryOk') else 'retryNS'] += 1
    if c: print(f'{nm}: 推后写得下 {c["retryOk"]}，推后仍被拒 {c["retryNS"]}')
