#!/usr/bin/env python3
"""文档里的相对链接与「第 N 节」指向，指不指得到。

**两类都查，因为两类都会静默失效**：
  ① `[x](../../CLAUDE.md)` —— 路径少一层就指到一个不存在的地方，读的人点开是空的；
     更坏的一种是它**恰好指到了另一个同名文件**，那时连「打不开」这个信号都没有。
  ② 「见 X.md 第三节」 —— 目标文档加一节、删一节，这个序号就指向别的东西了，
     而没有任何字看起来别扭。

⚠️ **枚举文件必须用 os.walk，不许用 glob('**/*.md')**：后者默认不匹配点开头的目录，
而本仓的文档几乎全住在 `.claude/` 下 ⇒ 检查会扫到接近零个文件、然后报绿。
这一条是实测踩过的：第一版就是那么写的，报「0 处指不到」，而当时正有一条坏链接。
"""
import os, re, sys

SKIP = {'.git', 'node_modules', 'target'}
# 上游副本里的 templates/ 是**给别的项目用的模板**，它的相对路径按「拷到项目根之后」写，
# 在副本里当然指不到。它也不许在本仓就地改（CLAUDE.md：改共享规则只能改上游）。
# `gate.d/fixtures/` 里的红样本**故意**放着坏链接——那是它的判别力来源。
# 样本跑的时候会被拷进临时目录、以那里为根来判，所以在本仓里跳过它不影响自检。
SKIP_PATH = ('.claude/singlefs-ai-sop/', '.claude/gate.d/fixtures/')
CN = {'一':1,'二':2,'三':3,'四':4,'五':5,'六':6,'七':7,'八':8,'九':9,'十':10}

def md_files(root='.'):
    for dp, dns, fns in os.walk(root):
        dns[:] = [d for d in dns if d not in SKIP]
        for f in fns:
            if f.endswith('.md'):
                yield os.path.normpath(os.path.join(dp, f))

def mask_code(s):
    """把代码块与行内代码涂成同长的空格，行号不变。

    ⚠️ 反引号里的东西不是链接：`[本该](决策|实验)` 是一条**正则**，不是 markdown 链接。
    不涂就会报一条不存在的坏链接——2026-08-31 实测撞过一次，报的是 doc-lint 自己的规则串。
    """
    def blank(m):
        return re.sub(r'[^\n]', ' ', m.group(0))
    s = re.sub(r'```.*?```', blank, s, flags=re.S)
    s = re.sub(r'`[^`\n]*`', blank, s)
    return s


def heads(path):
    try:
        return re.findall(r'^## (.+)$', open(path, encoding='utf-8').read(), re.M)
    except OSError:
        return []

bad = []
n_link = n_sec = 0
for p in sorted(md_files()):
    if p.startswith(SKIP_PATH):
        continue
    base = os.path.dirname(p) or '.'
    try:
        s = open(p, encoding='utf-8').read()
    except OSError:
        continue
    s = mask_code(s)
    for m in re.finditer(r'\[([^\]]*)\]\(([^)\s]+)\)', s):
        tgt = m.group(2)
        if tgt.startswith(('http://', 'https://', 'mailto:', '#')):
            continue
        path = tgt.partition('#')[0]
        if not path:
            continue
        n_link += 1
        full = os.path.normpath(os.path.join(base, path))
        if not os.path.exists(full):
            ln = s[:m.start()].count('\n') + 1
            bad.append(f"{p}:{ln} 链接 [{m.group(1)[:24]}]({tgt}) 指到 {full}，那里没有东西")
    # 「见 X.md 第 N 节」：目标文档到底有没有第 N 个 `##` 节
    for m in re.finditer(r'\[[^\]]*\]\(([^)\s]+\.md)\)[^，。；、\n]{0,8}第([一二三四五六七八九十0-9]+)节', s):
        n_sec += 1
        n = m.group(2)
        n = int(n) if n.isdigit() else CN.get(n, 0)
        full = os.path.normpath(os.path.join(base, m.group(1).partition('#')[0]))
        hs = heads(full)
        if not (0 < n <= len(hs)):
            ln = s[:m.start()].count('\n') + 1
            bad.append(f"{p}:{ln} 指「{m.group(1)} 第{m.group(2)}节」，而那个文档只有 {len(hs)} 个 ## 节")

if bad:
    print(f"  ✗ 文档指向失效 {len(bad)} 处")
    for b in bad[:40]:
        print("    ", b)
    print("     → 相对路径按**该文件所在目录**算，不是按仓库根算；「第 N 节」改成指小节标题。")
    sys.exit(1)
print(f"  ✓ 文档指向都到得了（{n_link} 条相对链接、{n_sec} 处「第 N 节」指向）")
