#!/usr/bin/env python3
"""H4 的粗估：照 doc-experiments.sh 1694-1773 的对照口径（replay.sh 登记着的产物 + 登记着却已出了树的归档版），
数一份「路径<TAB>行文」新增行清单里，格射程内整行抄的 E7RESULT 行有多少在对照里逐字找不到。
用法：estimate-red.py <新增行清单> <仓根（只读）>
不是门禁原判：历史节的行号判（new_line_numbers）这里没做，历史节里与正文重复的同文行按一行数。"""
import glob, os, re, subprocess, sys

added_file, repo = sys.argv[1], sys.argv[2]
ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
registered = set()
with open(os.path.join(repo, 'scripts/experiments/replay.sh'), encoding='utf-8') as handle:
    for line in handle:
        row = ROW_RE.match(line.rstrip('\n'))
        if row:
            registered.add(row.group(4))
quoted = set()
with open(added_file, encoding='utf-8', errors='replace') as handle:
    for record in handle.read().split('\n'):
        path, sep, text = record.partition('\t')
        if not sep:
            continue
        if path.endswith('-history.md') or '/decisions-history/' in path or path.startswith('research/prompts/') or path.startswith('research/results/'):
            continue
        text = text.strip()
        if text.startswith('E7RESULT '):
            quoted.add((path, text))
products = sorted(glob.glob(os.path.join(repo, 'research/results/*.out')))
on_disk = {os.path.basename(p) for p in products}
registered_lines, all_lines = set(), set()
for product in products:
    reg = os.path.basename(product) in registered
    with open(product, encoding='utf-8', errors='ignore') as handle:
        for line in handle:
            text = line.strip().replace('\r', '')
            all_lines.add(text)
            if reg:
                registered_lines.add(text)
full = len(sys.argv) > 3 and sys.argv[3] == 'full'   # 戊：全量口径，对照全部 .out（归档版不取，所以是上界）
compared = all_lines if full else registered_lines
missing = {item for item in quoted if item[1] not in compared}
archived_names = set() if full else registered - on_disk
archived_count = 0
if missing and archived_names:
    log = subprocess.run(['git', '-C', repo, '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only', '--', 'research/results'],
                         capture_output=True, text=True, env=dict(os.environ, GIT_OPTIONAL_LOCKS='0')).stdout
    commit = None
    extra = set()
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(c in '0123456789abcdef' for c in entry):
            commit = entry
            continue
        if commit and entry.endswith('.out') and os.path.basename(entry) in archived_names:
            blob = subprocess.run(['git', '-C', repo, 'show', '%s^:%s' % (commit, entry)], capture_output=True, text=True, errors='ignore')
            if blob.returncode == 0:
                archived_count += 1
                extra |= {one.strip().replace('\r', '') for one in blob.stdout.split('\n')}
    missing = {item for item in missing if item[1] not in extra}
if full and missing and os.environ.get('SCAN_ALL_ARCHIVE') == '1':   # 戊的全量口径：再扫全部归档过的 .out（817 份左右，慢）
    wanted = {item[1] for item in missing}
    log = subprocess.run(['git', '-C', repo, '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only', '--', 'research/results'],
                         capture_output=True, text=True, env=dict(os.environ, GIT_OPTIONAL_LOCKS='0')).stdout
    commit, found, scanned = None, set(), 0
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(c in '0123456789abcdef' for c in entry):
            commit = entry
            continue
        if commit and entry.endswith('.out'):
            blob = subprocess.run(['git', '-C', repo, 'show', '%s^:%s' % (commit, entry)], capture_output=True, text=True, errors='ignore')
            if blob.returncode == 0:
                scanned += 1
                for one in blob.stdout.split('\n'):
                    one = one.strip().replace('\r', '')
                    if one in wanted:
                        found.add(one)
    missing = {item for item in missing if item[1] not in found}
    print(f'  （扫了归档过的 .out {scanned} 份，补找到 {len(found)} 行）')
only_unregistered = {item for item in missing if item[1] in all_lines}
def under_history(item):   # 当前目录是检出了 md 的仓；这行文只出现在「## 历史版本」之后（正文里没有）
    in_history, seen_body, seen_history = False, False, False
    try:
        with open(item[0], encoding='utf-8', errors='ignore') as handle:
            for number, line in enumerate(handle.read().split('\n'), 1):
                if number > 1 and line.startswith('## 历史版本'):
                    in_history = True
                if line.strip() == item[1]:
                    if in_history: seen_history = True
                    else: seen_body = True
    except OSError:
        return False
    return seen_history and not seen_body
history_only = {item for item in missing if under_history(item)}
print(f'射程内 E7RESULT 新增行（路径加行文去重）={len(quoted)}  登记着的产物 {len(registered)} 份（树里 {len(registered & on_disk)} 份，归档版取到 {archived_count} 份）')
print(f'对照里逐字找不到={len(missing)}（其中只在没登记的产物里有={len(only_unregistered)}，哪儿都没有={len(missing) - len(only_unregistered)}）；其中只在「## 历史版本」之后={len(history_only)}（门禁对历史节按行号只判新增的，这里不分）')
