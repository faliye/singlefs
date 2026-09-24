"""T1 真仓只读扫描。
一、「D 的正文提到 E、E 的影响表里没有 D 那一行」的对子有多少（75 号 ① 只从实验一侧看、③ 只看「**依据**」段，这些对子两条都看不见；
    这一批把 E 改成已跑、回看表里别的行，75 号 ⑤ 放行，丙b* 判红）。
二、10 号第 3 段（已跑实验要有决策引用）与 75 号 ⑨（实验必须对应决策）逐页各判什么，数分歧。
用法：python3 t1-scan.py <真仓根>"""
import glob, os, re, sys
root = sys.argv[1]; os.chdir(root)
kb = '.claude/kb'
def body(path):
    text = open(path, encoding='utf-8').read(); cut = re.search(r'^## 历史版本\s*$', text, re.M)
    return text if not cut else text[:cut.start()]
def ten_regex(e): return re.compile(r'(^|[^A-Za-z0-9/-])' + e + r'([^A-Za-z0-9-]|$)', re.M)
pending = set()
if os.path.exists('.claude/decision-links-pending'):
    for line in open('.claude/decision-links-pending', encoding='utf-8'):
        m = re.match(r'^(E|D)(\d+)\s', line.strip()); pending |= {(m.group(1), int(m.group(2)))} if m else set()
pages = {}
for path in sorted(glob.glob(f'{kb}/experiments/*.md')):
    text = open(path, encoding='utf-8').read()
    head = next((l for l in text.split('\n') if re.match(r'^## E\d+ ', l)), None)
    if not head: continue
    n = int(re.match(r'^## E(\d+) ', head).group(1))
    rows = re.findall(r'^\|\s*\**D(\d+)（[^|]*\|\s*([^|]*?)\s*\|', body(path), re.M)
    pages[n] = {'head': head, 'path': path, 'rows': {int(d) for d, _ in rows}, 'relations': {r.strip('* ') for _, r in rows},
                'reserve': re.search(r'^\*\*备料\*\*：.*?((D|C)\d{1,3})（', text, re.M) is not None}
decisions = {}
for path in sorted(glob.glob(f'{kb}/decisions/*.md')):
    text = open(path, encoding='utf-8').read(); m = re.search(r'^## D(\d+) ', text, re.M)
    if m: decisions[int(m.group(1))] = path
blind, blind_body, blind_not_ran = [], [], []
for d, path in sorted(decisions.items()):
    if ('D', d) in pending: continue
    full, head = open(path, encoding='utf-8').read(), body(path)
    for n, page in pages.items():
        if ('E', n) in pending or d in page['rows']: continue
        if ten_regex(f'E{n}').search(full):
            blind.append((d, n))
            if re.search(r'(?<![A-Za-z0-9])E%d（' % n, head):
                blind_body.append((d, n))
                if '已跑' not in page['head']: blind_not_ran.append((d, n))
print(f'一、对子（D 不在待回填、E 不在待回填、E 的表里没有 D）：D 的文件里任一处按 10 号的正则提到 E 的 {len(blind)} 对；'
      f'其中 D 的正文（历史版本之前）用「E<号>（」形态提到的 {len(blind_body)} 对，这里面 E 的标题还没有「已跑」的 {len(blind_not_ran)} 对')
print('   正文形态的前 12 对：' + '、'.join(f'D{d}→E{n}' for d, n in blind_body[:12]))
decision_text = ''.join(open(p, encoding='utf-8').read() for p in list(decisions.values()) + [f'{kb}/decisions.md'] if os.path.exists(p))
cells = {}
for n, page in sorted(pages.items()):
    if '已跑' not in page['head']: continue
    ten = 'cited' if ten_regex(f'E{n}').search(decision_text) else ('reserve' if page['reserve'] else 'RED')
    voided = re.search(r'作废|退役', page['head']) is not None
    seventy_five = 'skip' if ('E', n) in pending or voided else ('ok' if page['relations'] & {'支撑', '推翻', '备料'} else 'RED')
    cells.setdefault((ten, seventy_five), []).append(n)
print('二、标题带「已跑」的实验页，按（10 号第 3 段，75 号 ⑨）分格：')
for key in sorted(cells):
    print(f'   10={key[0]:<8} 75⑨={key[1]:<5} {len(cells[key]):>4} 页  例：' + '、'.join(f'E{n}' for n in cells[key][:8]))
