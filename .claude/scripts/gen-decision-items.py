#!/usr/bin/env python3
"""从 decisions/ 下的正文抽出每条决策的分项清单，打印成 markdown。

**它存在的唯一理由是「同一个事实只许一处权威记录」**：分项的权威记录是各决策正文，
索引页只是它的投影。手抄一份就会漂，而漂了没有任何东西会发现——
`.claude/gate.d/21-decision-items-sync.sh` 拿本脚本的输出与索引页比对，不一致判红。
"""
import re, glob, sys

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


def items_of(body):
    """返回 [(编号, 名字, 状态)]。编号可能是 '-'（无编号的单项未定项）。"""
    # 未定项小节的本体：到下一个**同级或更高级**标题为止（`###` / `##`）。
    # 不在 `####` 处停——D5 的分项就住在一个 `#### 已定：…` 之后。
    m = re.search(r'^### 未定项\s*$(.*?)(?=^#{2,3} |\Z)', body, re.M | re.S)
    if not m:
        return []
    sec = m.group(1)

    def harvest(text):
        # **只取顶格的编号项**：缩进的是分项自己的子列表（论证的第 1/2/3 条），不是分项。
        got = re.findall(r'^(\d+)\.\s+(.*)$', text, re.M)
        if got:
            return got
        return re.findall(r'^\|\s*(\d+)\s*\|(.*)$', text, re.M)

    # 小节里若有 `#### 子标题`，先只看它**之前**那一段；那一段抽得到分项就用它，
    # 抽不到才回退到整节——D13 的子标题里另有一张表，不切会把它的行也当成分项。
    cut = re.search(r'^#{4}\s', sec, re.M)
    out = harvest(sec[:cut.start()]) if cut else []
    if not out:
        out = harvest(sec)
    if not out:
        first = [l for l in sec.strip().split('\n') if l.strip()]
        if first:
            out = [('-', first[0])]

    res = []
    for n, txt in out:
        # 状态判定按优先级来，**不许只看「文中有没有出现已定」**：
        # 一条分项的正文里常常提到别处的「已定」（例：「与 D16 已定的 checkpoint 序号怎么共存」）。
        if re.search(r'状态：\s*\*{0,2}已定', txt):
            settled = True
        elif re.search(r'状态：\s*\*{0,2}未定', txt):
            settled = False
        elif re.search(r'——\s*\*{0,2}已定', txt):      # 「名字 —— 已定（…）」
            settled = True
        elif re.search(r'\|\s*\*{0,2}已定', txt):       # 表格里状态列以「已定」开头
            settled = True
        else:
            settled = False
        name = re.sub(r'\*\*|`', '', txt)
        name = re.split(r'——|\||。', name)[0].strip().rstrip('：:')
        res.append((n, clip(name, 46), '已定' if settled else '**未定**'))
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
