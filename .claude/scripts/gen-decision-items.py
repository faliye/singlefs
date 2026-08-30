#!/usr/bin/env python3
"""从 decisions/ 下的正文抽出每条决策的分项清单，打印成 markdown。

**它存在的唯一理由是「同一个事实只许一处权威记录」**：分项的权威记录是各决策正文，
索引页只是它的投影。手抄一份就会漂，而漂了没有任何东西会发现——
`.claude/gate.d/21-decision-items-sync.sh` 拿本脚本的输出与索引页比对，不一致判红。

**每条决策的分项只有一套编号，分住两个小节**：`### 已定项` 与 `### 未定项`。
状态由**它住在哪一节**决定，不由行内文字决定——正文里一句「与 D16（发布语义） 已定的
checkpoint 序号怎么共存」曾让按行内关键字判状态的老版本把一条未定项判成已定。
"""
import re, glob

def clip(text, n):
    """截到 n 个字符，**但不许留下没闭合的括注**。

    「编号（简称）」被截在括号中间会让 doc-lint 判「括注没闭合」——
    而那条检查是对的：半个简称比没有简称更容易被误读。
    """
    t = text[:n]
    while t.count('（') > t.count('）'):
        t = t[:t.rindex('（')].rstrip()
    # 截断还可能在编号后面切掉它的简称，留下一个**裸引用**——doc-lint 同样判红，
    # 而它判得对：一个只剩符号的编号，含义可以被悄悄改掉而没有一个字看起来别扭
    # （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条）。把这种尾巴一并去掉。
    while True:
        t2 = re.sub(r'[A-Z]-?\d+(?:\.\d+)?\s*$', '', t)
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()


def name_of(txt):
    name = re.sub(r'\*\*|`', '', txt)
    name = re.split(r'——|\||。', name)[0].strip().rstrip('：:')
    return clip(name, 46)


def harvest(sec):
    """取一节里**顶格的**编号项：表格行优先，其次编号列表。

    只看该节的**索引表**，即第一个 `####` 子标题之前那一段——
    子标题之下是各分项各自的论证，里面另有编号列表与表格，那些不是分项。
    """
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    rows = re.findall(r'^\|\s*(\d+)\s*\|([^|]*)\|', top, re.M)
    if rows:
        return rows
    return re.findall(r'^(\d+)\.\s+(.*)$', top, re.M)


def items_of(body):
    """返回 [(编号, 名字, 状态)]，两节合起来按编号排。"""
    res = []
    for head, st in (('已定项', '已定'), ('未定项', '**未定**')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, re.M | re.S)
        if not m:
            continue
        got = harvest(m.group(1))
        if not got and head == '未定项':
            first = [l for l in m.group(1).strip().split('\n') if l.strip()]
            if first:
                res.append(('-', name_of(first[0]), st))
            continue
        for n, txt in got:
            res.append((n, name_of(txt), st))
    res.sort(key=lambda r: (r[0] != '-', int(r[0]) if r[0] != '-' else 0))
    return res


lines = []
for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
    s = open(f, encoding='utf-8').read()
    title = s.split('\n', 1)[0]
    # 破折号前有没有空格两种都有（D14 没有），不能只认一种
    mm = re.match(r'## (D\d+) (.+?)\s*—— *(.+)$', title)
    if not mm:
        continue
    num, name, status = mm.group(1), mm.group(2).strip(), mm.group(3).strip()
    body = s.split('\n## 历史版本')[0]
    its = items_of(body)
    open_n = sum(1 for _, _, st in its if '未定' in st)
    st = re.match(r'(已定|半定|待定)', status)
    head = f"- **{num}（{name}）** —— {st.group(1) if st else status[:6]}"
    head += f"（分项 {len(its)}，其中未定 {open_n}）" if its else "（无分项）"
    lines.append(head)
    for n, nm, st in its:
        lines.append(f"  - {n}. {nm} —— {st}")
print('\n'.join(lines))
