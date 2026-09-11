#!/usr/bin/env python3
"""生成三方论证材料要带的那张小节清单——**含第一个标题之前的引言段**。

`.claude/rules/three-way-inference.md` 要求材料带一张小节清单，逐节标「抄 / 不抄 /
为什么不抄」，好让漏掉的那一节留下可见的空格。此前这张表是每轮临时用 awk 抓 `^#{2,5} `
生成的，于是**文档开头那段正文没有自己的行**：它住在 H2 标题之下、第一个 H3 之前，
在清单里只表现为那条 H2 标题行。而 H2 标题行看着就是个标题，标「不抄」几乎不需要理由,
一标下去，那段正文连同它一起无声消失。

⇒ 本脚本给那段正文**单独发一行**，标出它的行区间，让它必须被单独判一次抄不抄。
⚠️ 射程只有开头这一段：别的标题的正文仍然由它自己那一行代表。

**一级标题同样算标题**（2026-09-11 补）。此前只认二到六级，于是顶上是一级标题的文件——
`checks-owed.md`、`invariants.md`、`first-txn-layout.md`——那个一级标题连同它下面、第一个
下级标题之前的正文**一行都不发**。`checks-owed.md` 的整张欠账表（第 2-243 行）就住在那里，
清单上根本看不见它，也就没有地方写「不抄，因为……」。实测是给 D19 未定项 6 第二轮备材料时
逐行对清单才发现的。自证会红：`KB_SECTIONS_NO_H1=1` 强制走回旧行为，自检里那一行必须消失。

实测（2026-09-10，D6（快照实现模型） 未定项 2 第一轮）：
`.claude/kb/decisions/05-快照-空间记账机制.md` 的第 1-14 行是整个模型的定义段，
逐字写着「销毁快照时把它的 deadlist 合并到下一个更新的那一侧……**不做全盘扫描**」。
它无声消失，而那一轮的枢纽问题正是「销毁快照是不是一次全盘遍历」——
材料因此把一个假前提发给了三条腿，是正推腿自己去读原文才捞回来的。

用法：

    kb-sections.py 文件.md [文件.md ...]      # 打印清单，抄 / 不抄两列留空待填

自证会红：`kb-sections.py --selftest` 造一份「标题之前有正文」的样本，
断言清单里有那一行；再用 KB_SECTIONS_NO_PREAMBLE=1 强制走回旧行为，确认判红。
"""
import io, os, re, sys, tempfile

HEAD = re.compile(r'^(#{1,6}) .*$')
HEAD_NO_H1 = re.compile(r'^(#{2,6}) .*$')


def fenced_lines(lines):
    """返回落在代码栅栏里的行号集合（1 起）。栅栏里的 `## ` 不是标题。"""
    inside, fence = set(), None
    for i, l in enumerate(lines, 1):
        fm = re.match(r'(`{3,}|~{3,})', l)
        if fence is None and fm:
            fence = (fm.group(1)[0], len(fm.group(1))); inside.add(i); continue
        if fence is not None:
            inside.add(i)
            cm = re.fullmatch(r'(`{3,}|~{3,})\s*', l)
            if cm and cm.group(1)[0] == fence[0] and len(cm.group(1)) >= fence[1]:
                fence = None
    return inside


def is_head(i, l, fenced):
    head = HEAD_NO_H1 if os.environ.get('KB_SECTIONS_NO_H1') else HEAD
    return i not in fenced and head.match(l)


def sections(path):
    """返回 [(标签, 首行号)]；开头那段正文单独占一行，紧跟在它所属的标题之后。"""
    lines = io.open(path, encoding='utf-8').read().split('\n')
    fenced = set() if os.environ.get('KB_SECTIONS_NO_FENCE') else fenced_lines(lines)
    heads = [i for i, l in enumerate(lines, 1) if is_head(i, l, fenced)]
    out = []
    for i, l in enumerate(lines, 1):
        if not is_head(i, l, fenced):
            continue
        out.append((l.replace('|', '/'), i))
        if os.environ.get('KB_SECTIONS_NO_PREAMBLE') or i != heads[0]:
            continue
        nxt = next((h for h in heads if h > i), len(lines) + 1)
        body = lines[i:nxt - 1]
        if any(x.strip() for x in body):
            out.append((f'（{l.split()[1] if len(l.split()) > 1 else ""} 标题之下、'
                        f'第一个下级标题之前的正文：第 {i + 1}-{nxt - 1} 行）', i + 1))
    return out


def render(paths):
    buf = []
    for p in paths:
        buf.append(f'### 小节清单：`{p}`（{os.path.basename(__file__)} 全量生成，未经任何过滤）')
        buf.append('')
        buf.append('| 小节 | 抄 / 不抄 | 理由 |')
        buf.append('|---|---|---|')
        for title, _ in sections(p):
            buf.append(f'| {title} | | |')
        buf.append('')
    return '\n'.join(buf)


def selftest():
    with tempfile.TemporaryDirectory() as d:
        f = os.path.join(d, 'sample.md')
        io.open(f, 'w', encoding='utf-8').write(
            '## D99 样本 —— 已定\n\n这一段在第一个三级标题之前，是承重的定义段。\n\n### 甲节\n\n正文。\n')
        rows = [t for t, _ in sections(f)]
        has_pre = any('之前的正文' in r for r in rows)
        if os.environ.get('KB_SECTIONS_NO_PREAMBLE'):
            if has_pre:
                print('selftest: 强制关掉那一行之后它仍然在 —— 检查坏了'); return 1
            print('selftest: 关掉那条分支确认判红（清单里没有开头正文那一行）'); return 0
        if not has_pre:
            print('selftest: 清单里没有开头正文那一行 —— 正是本脚本要防的形态'); return 1
        h = os.path.join(d, 'h1.md')
        io.open(h, 'w', encoding='utf-8').write(
            '# 样本清单\n\n| C1 | 一整张表住在一级标题之下 |\n\n### 已还清\n\n正文。\n')
        hrows = [t for t, _ in sections(h)]
        has_h1 = any(r.startswith('（样本清单 标题之下') for r in hrows)
        if os.environ.get('KB_SECTIONS_NO_H1'):
            if has_h1:
                print('selftest: 强制不认一级标题之后那一行仍然在 —— 检查坏了'); return 1
            print('selftest: 关掉一级标题那条分支确认判红（一级标题下的正文没有自己的行）'); return 0
        if not has_h1:
            print('selftest: 一级标题之下的正文没有自己的行 —— 整张表会从清单上消失'); return 1
        g = os.path.join(d, 'fenced.md')
        io.open(g, 'w', encoding='utf-8').write(
            '## D98 样本 —— 已定\n\n### 甲节\n\n```bash\n## 这一行在栅栏里，不是标题\n```\n\n### 乙节\n')
        frows = [t for t, _ in sections(g)]
        if any('栅栏里' in r for r in frows):
            print('selftest: 代码栅栏里的 `## ` 行被当成标题发了一行'); return 1
        print(f'selftest: 通过（{len(rows)} 行，开头那段正文单独占一行；栅栏里的 `## ` 不算标题）')
        print('           证明会红：KB_SECTIONS_NO_PREAMBLE=1 / KB_SECTIONS_NO_H1=1 各再跑一遍，对应那一行必须消失')
        return 0


if __name__ == '__main__':
    args = sys.argv[1:]
    if not args:
        print(__doc__); sys.exit(2)
    if args[0] == '--selftest':
        sys.exit(selftest())
    missing = [p for p in args if not os.path.exists(p)]
    if missing:
        print('kb-sections: 这些文件不存在：' + ' '.join(missing), file=sys.stderr)
        print('下一步：核对路径，或先 ls 一下那个目录。', file=sys.stderr)
        sys.exit(2)
    print(render(args))
