"""丙b*：这一批里标题状态从「非已跑」变成「已跑」的实验，正文引用它的每份决策文件都要在这一批的改动范围里。
改动范围取真仓的 research/scripts/changed-paths.sh（gate 取法、带未跟踪）；「已跑」按状态段开头认（「已跑，结论作废」也算，与 10 号同口径）。
用法：python3 t1-judge.py <现场仓根> <changed-paths.sh 路径>"""
import glob, os, re, subprocess, sys
root, lib = sys.argv[1], sys.argv[2]
os.chdir(root)
got = subprocess.run(['bash', '-c', 'source "$1"; b="$(gate_diff_base gate)"; echo "$b"; gate_changed_paths "$b" untracked', '_', lib],
                     capture_output=True, text=True)
if got.returncode != 0:
    print(f'红：取不到改动范围（共用脚本退 {got.returncode}），不当成「这一批什么都没改」'); sys.exit(1)
out = got.stdout.split('\n')
base, changed = out[0], {line for line in out[1:] if line}
def status(title):
    m = re.search(r'—— ?(.*)$', title or '')
    return m.group(1).strip() if m else ''
decision_files = sorted(glob.glob('.claude/kb/decisions/*.md')) + ['.claude/kb/decisions.md']
reds = []
for path in sorted(glob.glob('.claude/kb/experiments/*.md')):
    now = next((l for l in open(path, encoding='utf-8') if l.startswith('## E')), '')
    was = subprocess.run(['git', 'show', f'{base}:{path}'], capture_output=True, text=True).stdout.split('\n')[0]
    if not status(now).startswith('已跑') or status(was).startswith('已跑'):
        continue
    number = re.match(r'## (E\d+) ', now).group(1)
    pattern = re.compile(r'(^|[^A-Za-z0-9/-])' + number + r'([^A-Za-z0-9-]|$)', re.M)
    for d in decision_files:
        if os.path.exists(d) and pattern.search(open(d, encoding='utf-8').read()) and d not in changed:
            reds.append(f'{number} 改成已跑，引用它的 {d} 不在这一批里')
print('红：' + '；'.join(reds) if reds else '绿')
