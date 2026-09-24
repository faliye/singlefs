#!/usr/bin/env python3
"""决策分项引用的状态与正文是否一致。

每一处「D<n>（简称） 已定项 k / 未定项 k」的**前缀**都断言了那一条分项的状态。
写错了没有任何东西会发现——检索会把「已定项 5」当成已经定了的东西端出去，
而它可能还开着。本检查拿各决策正文里的分项表当权威，逐处比对。

判据：引用处写「已定项 k」而正文那条标着未定（或反过来）⇒ 判红。
归属靠同一行里最近的 D 记号；行内没有 D 记号时，用文件自身的决策号，
再退到该行之前最近出现过的 D 记号——三级都判不出的也判红，因为
一个归属判不出的编号引用，读的人同样判不出。

**归属规则只有这一份**：44 号阶段（引用写着已定项、紧跟着却说它没定）与
`research/scripts/relabel-item.py`（分项翻状态之后改写全仓引用）都 import 这里的函数，
不各抄一份——两份归属规则一旦分叉，工具改写的与门禁判的就不是同一批引用。
"""
import re, glob, sys, os


def decision_heads(strict=True):
    """decisions/*.md 每一份首行的 ({路径: (决策号, 简称)}, [(首行读不出的路径, 首行)])。

    首行读不出 `## D<n> 简称 —— 状态` 的（多一份 README、首行前有空行或 BOM、漏了破折号）一份都不跳过，
    也不让调用方撞上裸的 AttributeError：strict 时当场打印 ✗ 与出路、退出码 1；
    不 strict 时交回清单，由调用方在报完自己的问题之后调 report_unreadable 并判红。
    跳过它的代价实测过：那条决策的分项从表里消失，写给它的「已定项 k」被改判到行内更早出现的另一条决策上并判绿。
    load_map 与 self_decisions 都从这里取，两处的口径不许分叉（写法照 lib-index-vs-body.py 读首行那一段）。
    """
    heads, unreadable = {}, []
    for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
        first = open(f, encoding='utf-8').read().split('\n', 1)[0]
        mm = re.match(r'## (D\d+) (.+?)\s*——', first)
        if mm:
            heads[f] = (mm.group(1), mm.group(2).strip())
        else:
            unreadable.append((f, first))
    if unreadable and strict:
        report_unreadable(unreadable)
        sys.exit(1)
    return heads, unreadable


def report_unreadable(unreadable):
    print(f"  ✗ decisions/ 下有 {len(unreadable)} 份首行读不出 `## D<n> 简称 —— 状态`，这几份的分项一条都没核过")  # gate-lint:summary
    for f, first in unreadable:
        print(f"     {f}：首行是 {first[:40]!r}")  # gate-lint:detail
    print("     → 怎么办：把首行写成 `## D3 空间分配 —— 半定（…）`，首行之前不许有空行或 BOM；")
    print("       decisions/ 下只放决策正文，说明性的文字写进 decisions.md。读不出来就是这一份没核过，不许当成通过。")


def load_map(heads=None):
    """heads 不给就现读，首行读不出的当场判红退出（decision_heads 的 strict）。"""
    m, name = {}, {}
    for f, (d, short_name) in sorted((heads if heads is not None else decision_heads()[0]).items()):
        s = open(f, encoding='utf-8').read()
        name[d] = short_name; m[d] = {}
        for head, st in (('已定项', '已'), ('未定项', '未')):
            sec = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, s, re.M | re.S)
            if not sec: continue
            seg = sec.group(1)
            cut = re.search(r'^#{4}\s', seg, re.M)
            top = seg[:cut.start()] if cut else seg
            # ⚠️ 正则要与 `.claude/scripts/gen-decision-items.py` 逐字同口径：
            # 那边要求表格行有**闭合的第二根竖线**。两边不一致时，一行畸形表格
            # 会「一边看得见、一边看不见」，而两边都不报错（2026-08-30 对抗验证指出）。
            for n in re.findall(r'^\|\s*(\d+)\s*\|[^|]*\|', top, re.M) or re.findall(r'^(\d+)\.\s', top, re.M):
                m[d][int(n)] = st
    return m, name


def scanned_files():
    files = sorted(set(sum([glob.glob(p, recursive=True) for p in
        ('.claude/kb/**/*.md', 'records/**/*.md', 'research/**/*.md',
         'research/**/*.rs', 'crates/**/*.rs', '.claude/rules/*.md')], [])))
    # research/prompts/ 显式排除，理由与 26 号门禁相同：那是原样发给模型的提示与模型的原样输出，
    # 与 research/results/ 里的产物一一对应，事后改它等于让产物对不上输入。
    # 实测（2026-09-03）：反推腿的输出里有一条复现命令 `grep -n "已定项 8" …`，
    # 没有决策号可归属，按正文规则判红，而那一行按证据链不许改。
    return [f for f in files if '/prompts/' not in f]


def self_decisions(heads=None):
    return {f: d for f, (d, _) in (heads if heads is not None else decision_heads()[0]).items()}


def references(path, item_map, self_map):
    """逐处给出一个分项引用：(行号, 行, 匹配, 写的状态, 归属或 None, 指名却没有第 k 条的那条决策或 None, 紧挨着指名的决策或 None)。"""
    self_d = self_map.get(path); last = None; hist = None; in_fence = False
    for ln, line in enumerate(open(path, encoding='utf-8').read().split('\n'), 1):
        # 代码围栏里是逐字照抄的东西——变更史的「改前」原行、实验的产物行、源码片段。
        # 按今天的状态判它们，等于要求「原样保存的证据」跟着现状改
        # （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」）。
        # 实测（2026-09-20）：第三批瘦身把 D17（实现分层与第三方管道） 的旧正文抄进变更史围栏之后，
        # 围栏里两处「未定项 6」当场判红——那两行说的是抄下来那天的状态，而立项让第 6 条变成了已定。
        if line.lstrip().startswith('```'):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if (path.endswith('decisions-history.md') or '/decisions-history/' in path) and line.startswith('### '):
            mm = re.search(r'D(\d+)', line); hist = 'D' + mm.group(1) if mm else None
        for m in re.finditer(r'(已定项|未定项)\s*(\d+)', line):
            k = int(m.group(2)); want = '已' if m.group(1) == '已定项' else '未'
            pre = line[:m.start()]; ds = re.findall(r'D(\d+)', pre)
            # ⚠️ 只有**紧挨着**分项引用的那个编号才算「指名」（仓里规范的引用形态是
            # `D6（快照实现模型） 已定项 2`）。行内更早提到的编号不算：同一行里裸写的
            # 「已定项 k」按文件自身的决策号解析，那是下面三级回退存在的理由。
            adj = re.search(r'D(\d+)\s*(?:（[^）]*）)?\s*[*`]*\s*$', pre)
            named = ('D' + adj.group(1)) if adj else None
            # 指名道姓的那条决策，正文里没有第 k 条 ⇒ 当场判红，不许往下落到别的决策。
            # 少了这一句，一个不存在的分项号会被静默改判到「刚好有第 k 条」的另一条决策
            # 头上并判绿：2026-09-06 起 decisions-history 里的「D6 已定项 2」就这样绿了四天，
            # 而 D6 从头到尾只有 1 个分项，同一个号在两份腿输出里还指着两样东西。
            if named in item_map and k not in item_map[named]:
                yield ln, line, m, want, None, named, named
                continue
            owner = next((c for c in (named, 'D' + ds[-1] if ds else None, self_d, hist, last)
                          if c in item_map and k in item_map[c]), None)
            yield ln, line, m, want, owner, None, named
        for d in re.findall(r'D(\d+)', line):
            if 'D' + d in item_map: last = 'D' + d


# 「已定项 k」后面紧跟着说它没定：中间只许有一段括注、标点与「今天 / 仍 / 还」这类词，
# 而且那个词之后句子就断了（标点、括号或行尾）。⚠️ 不收这一尾，「D5 已定项 4 没定的东西」
# 「D2 已定项 6 还没定判据」会被误判——那里的「没定」修饰后面的名词，说的是「那条定了、但没定到这一样」
# （实测：第一版在实验源码的注释里报了这两处）。两头都认 markdown 的 `**` 与反引号：「**D27 已定项 5 未定**」第一版就漏了。
# 44 号阶段判红用它，relabel-item.py 列「要人看的句子」也用它。
OPEN_AFTER = re.compile(r'^(?:（[^）]*）)?[\s、，,：:*`]*(?:今天|目前|仍然|仍|尚|还|都|均)?\s*(?:未定(?!项)|没定|定不下|待定|空着)'
                        r'(?=$|[\s。，、；：:;,.)）」』（(！？!?*`])')


def says_open_after(line, end):
    return OPEN_AFTER.match(line[end:]) is not None


def main():
    heads, unreadable = decision_heads(strict=False)
    item_map, names = load_map(heads)
    self_map = self_decisions(heads)
    bad = []; files = scanned_files(); refs = 0
    for path in files:
        for ln, line, m, want, owner, missing, _ in references(path, item_map, self_map):
            refs += 1
            k = int(m.group(2))
            if missing:
                bad.append(f"{path}:{ln} 「{m.group(0)}」指名 {missing}"
                           f"（{names[missing]}），而它正文里没有第 {k} 条：{line.strip()[:60]}")
            elif owner is None:
                bad.append(f"{path}:{ln} 「{m.group(0)}」归属判不出：{line.strip()[:70]}")
            elif item_map[owner][k] != want:
                bad.append(f"{path}:{ln} 「{m.group(0)}」写的是{want}定，而 {owner}"
                           f"（{names[owner]}） 正文里第 {k} 条是{item_map[owner][k]}定：{line.strip()[:60]}")
    if unreadable:
        report_unreadable(unreadable)
    if bad:
        print(f"  ✗ 分项引用与正文状态不一致 {len(bad)} 处")
        for b in bad[:40]: print("    ", b)
        print("     → 权威是各决策正文的「### 已定项 / ### 未定项」两张表；改引用处，或先改正文再改引用。")
        print("     → 报「指名 Dn 而它没有第 k 条」的：那个编号引不到任何东西，去查它本来想指哪一条。")
        print("     → 一条分项刚翻了状态、引用一大片对不上的：用 research/scripts/relabel-item.py D<n> <k> 按同一份归属规则改写。")
    if bad or unreadable:
        sys.exit(1)
    n = sum(len(v) for v in item_map.values())
    # 一处引用都没扫到，这一轮什么都没比过：退 77，门禁记「本次未跑」而不是通过
    # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
    if refs == 0:
        print(f"  ! 扫了 {len(files)} 个文件，一处分项引用都没有（{len(item_map)} 条决策、{n} 个分项），本阶段无对象可判")
        sys.exit(77)
    print(f"  ✓ 分项引用与正文状态一致（{len(item_map)} 条决策、{n} 个分项；扫了 {len(files)} 个文件、{refs} 处分项引用）")


if __name__ == '__main__':
    main()
