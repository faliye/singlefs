#!/usr/bin/env python3
"""K5：wu_cf（全量判 + 计数豁免 + 固定基线）在产物重跑 / 换登记之后，正文早先抄的行有多少变红。
取样：本仓克隆上「改了 scripts/experiments/replay.sh」或「原地改了 research/results/*.out」的每一个提交 C（不抽样，全取），父提交 P。
对 P 树里正文（射程照冻结 doc-experiments.sh 第 1630–1644 行：.claude/kb 与 research 下的 .md，去掉 *-history.md、decisions-history/、
research/prompts/、research/results/；只取「## 历史版本」之前、去空白后以 `E7RESULT ` 开头的行，全量判下历史节整节不判）里整行抄的每一行，
按两种对照集各判一次「在 P 找得到、拿 P 的行文放到 C 的对照集里找不到」= 这次重跑让它变红：
  registered：replay.sh 登记着的产物（树里的，加登记着而已出树的最后一版；第 1765、1772 行那种对照）
  all：树里全部产物加全部归档（--diff-filter=D 的前一版；第 1763、1769 行那种对照）
再数其中 C 自己把那一页的那一行改掉或删掉了的（当场回改）与留着的。用法：k5-rerun.py <克隆根>
"""
import re, subprocess, sys
ROOT = sys.argv[1]
def git(*a, text=True):
    return subprocess.run(['git', '-C', ROOT, '-c', 'core.quotepath=false', *a], capture_output=True, text=text, errors='replace' if text else None).stdout
ROW = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
blob_cache = {}
def e7_lines_of_blob(blob):
    if blob not in blob_cache:
        data = git('cat-file', 'blob', blob)
        blob_cache[blob] = {l.strip().replace('\r', '') for l in data.split('\n') if 'E7RESULT' in l}
    return blob_cache[blob]
def products_in_tree(c):
    out = {}
    for line in git('ls-tree', '-r', c, '--', 'research/results').split('\n'):
        if not line: continue
        meta, path = line.split('\t', 1)
        if path.endswith('.out'): out[path] = meta.split()[2]
    return out
def registered(c):
    return {m.group(4) for l in git('show', f'{c}:scripts/experiments/replay.sh').split('\n') if (m := ROW.match(l))}
def archived(c):   # (路径, 删它的提交之前那一版的 blob)，按删的先后，后删的在后
    out, commit = [], None
    for e in git('log', c, '--diff-filter=D', '--format=%H', '--name-only', '--', 'research/results').split('\n'):
        e = e.strip()
        if re.fullmatch(r'[0-9a-f]{40}', e): commit = e; continue
        if commit and e.endswith('.out'):
            b = git('rev-parse', f'{commit}^:{e}').strip()
            if b: out.append((e, b))
    return out
def compare_sets(c):
    tree = products_in_tree(c); reg = registered(c); arch = archived(c)
    all_lines, reg_lines = set(), set()
    for p, b in tree.items():
        ls = e7_lines_of_blob(b); all_lines |= ls
        if p.rsplit('/', 1)[-1] in reg: reg_lines |= ls
    on_disk = {p.rsplit('/', 1)[-1] for p in tree}
    latest = {}
    for p, b in arch:
        all_lines |= e7_lines_of_blob(b)
        name = p.rsplit('/', 1)[-1]
        if name in reg and name not in on_disk and name not in latest: latest[name] = b   # git log 新的在前：第一次见到的是最后那一版
    for b in latest.values(): reg_lines |= e7_lines_of_blob(b)
    return reg_lines, all_lines
def quoted(c):
    out = []
    files = [f for f in git('ls-tree', '-r', '--name-only', c, '--', '.claude/kb', 'research').split('\n')
             if f.endswith('.md') and (f.startswith('.claude/kb/') or f.startswith('research/')) and not f.endswith('-history.md')
             and '/decisions-history/' not in f and not f.startswith('research/prompts/') and not f.startswith('research/results/')]
    for f in files:
        text = git('show', f'{c}:{f}')
        if 'E7RESULT' not in text: continue
        for n, line in enumerate(text.split('\n'), 1):
            if n > 1 and line.startswith('## 历史版本'): break
            s = line.strip()
            if s.startswith('E7RESULT '): out.append((f, n, s))
    return out
events = sorted(set(git('log', '--format=%H', '--', 'scripts/experiments/replay.sh').split()) |
                set(git('log', '--diff-filter=M', '--format=%H', '--', 'research/results/*.out').split()),
                key=lambda h: (int(git('log', '-1', '--format=%ct', h).strip()), h))   # 同一秒的按提交号排，次序逐次一样
total = {'registered': 0, 'all': 0}
for c in events:
    parents = git('log', '-1', '--format=%P', c).split()
    if not parents: continue
    p = parents[0]
    q = quoted(p); q_c = {(f, s) for f, _, s in quoted(c)}
    sets_p, sets_c = compare_sets(p), compare_sets(c)
    fields = []
    for i, mode in enumerate(('registered', 'all')):
        red_at_p = sum(1 for _, _, s in q if s not in sets_p[i])
        turned = [(f, n, s) for f, n, s in q if s in sets_p[i] and s not in sets_c[i]]
        fixed_in_c = sum(1 for f, _, s in turned if (f, s) not in q_c)
        total[mode] += len(turned)
        fields.append(f'{mode}_red_before={red_at_p} {mode}_turned_red={len(turned)} {mode}_fixed_in_same_commit={fixed_in_c} {mode}_left_red={len(turned) - fixed_in_c}')
    rows_at_p = len(registered(p))
    fields.append(f'registry_rows_at_parent={rows_at_p}')
    by_file = {}
    for f, n, s in q:
        if s in sets_p[0] and s not in sets_c[0]: by_file[f] = by_file.get(f, 0) + 1
    if by_file: fields.append('registered_turned_red_by_page=' + ','.join(f'{f}:{k}' for f, k in sorted(by_file.items())))
    touched = len([x for x in git('diff', '--name-only', p, c, '--', 'research/results').split('\n') if x.endswith('.out')])
    print(f'name=k5_event commit={c[:12]} date={git("log", "-1", "--format=%cd", "--date=short", c).strip()} quoted_body_lines={len(q)} products_touched={touched} ' + ' '.join(fields))
print(f'name=k5_total events={len(events)} registry_valid_events={sum(1 for c in events if git("log","-1","--format=%P",c).split() and registered(git("log","-1","--format=%P",c).split()[0]))} registered_turned_red={total["registered"]} all_turned_red={total["all"]}')
