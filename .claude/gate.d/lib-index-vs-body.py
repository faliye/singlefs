"""决策索引表的「状态」列 vs 各决策正文（20-kb-shape 第 7 段用）。

索引表那一格写的是**分项计数**：`已定 6 项 / 未定 0 项`，没有分项的写
`无分项 · 整条已定`。它是各正文「### 已定项」/「### 未定项」两节的投影
（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：同一个事实只许一处权威记录）。

⚠️ **本段自己去数正文，不经过 `.claude/scripts/gen-decision-items.py`**。
写回索引页的是那个生成器，21 阶段拿它的输出与索引页逐字比对——
**审计与被审计用同一段代码时，那个比对什么也证明不了**（`.claude/rules/fs-design.md`
「记账是事务的副产品」那一格的同一条道理）。生成器自己数错时，21 阶段两边一起错，
本段才会红。

⚠️ 三态词（已定 / 半定 / 待定）**不在索引列里**，它的权威记录是正文首行
`## D<n> 简称 —— 状态`。所以本段只对没有分项的那种行核一次三态词——
那种行的索引格里确实写着它。
"""
import re, sys

index_path, *bodies = sys.argv[1:]
idx = open(index_path, encoding="utf-8").read()
CN = {"一":1,"二":2,"两":2,"三":3,"四":4,"五":5,"六":6,"七":7,"八":8,"九":9,"十":10}

CELL = re.compile(r'^已定\s*(\d+)\s*项\s*/\s*未定\s*(\d+)\s*项$')
NOITEM = re.compile(r'^无分项\s*·\s*整条(已定|半定|待定)$')


def title_open(s):
    """正文标题里声明的未定条数。写法不拘（「一项未定」「3 条未定」），没写就是 0。"""
    m = re.search(r'([0-9]+|[一二两三四五六七八九十])\s*[项条]未定', s)
    if not m:
        return 0
    g = m.group(1)
    return int(g) if g.isdigit() else CN[g]


def kind(s):
    for k in ("已定", "半定", "待定"):
        if s.lstrip("*").startswith(k):
            return k
    return "?"


def count_items(body):
    """数两节各自索引表里的顶格编号项，返回 (已定数, 未定数)。

    只看**第一个 `####` 之前**那一段：子标题之下是各分项各自的论证，
    里面另有编号列表与表格，那些不是分项。
    """
    got = []
    for head in ("已定项", "未定项"):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, flags=re.M | re.S)
        seg = m.group(1) if m else ""
        cut = re.search(r'^#{4}\s', seg, flags=re.M)
        top = seg[:cut.start()] if cut else seg
        got.append(len(re.findall(r'^\|\s*\d+\s*\|', top, flags=re.M))
                   or len(re.findall(r'^\d+\.\s', top, flags=re.M)))
    return got[0], got[1]


fail = 0
seen = 0
for p in bodies:
    body = open(p, encoding="utf-8").read()
    t = body.split("\n", 1)[0]
    # 破折号前的空格是可选的（D14 的标题就没有）。老版本只认带空格的写法，
    # 于是那一条决策**整条被跳过**，而输出里看不出少核了一条 —— 现在读不出来就判红。
    m = re.match(r'## (D\d+) .+?\s*——\s*(.+)$', t)
    if not m:
        print(f"  ✗ {p} 首行读不出 `## D<n> 简称 —— 状态`：{t[:40]}")
        print("     → 怎么办：把首行写成 `## D3 空间分配 —— 半定（…）`；")
        print("       读不出来就是这一条没核过，不许当成通过。")
        fail = 1
        continue
    num, bs = m.group(1), m.group(2)
    row = re.search(r'^\| %s（[^|]*\) *\| ([^|]+)\|' % num, idx, flags=re.M)
    if not row:
        row = re.search(r'^\| %s（[^|]*） *\| ([^|]+)\|' % num, idx, flags=re.M)
    if not row:
        print(f"  ✗ {num} 在 decisions.md 索引里找不到对应行")
        print("     → 怎么办：给它补一行 `| D<n>（简称） | 状态 | 结论（简报） | 正文 |`，")
        print("       状态列跑 bash .claude/gate.d/21-decision-items-sync.sh --write 生成。")
        fail = 1
        continue
    seen += 1
    isx = row.group(1).strip()
    # 正文里实际有几条分项 —— 本段自己数的那一份
    n_settled, n_open = count_items(body.split("\n## 历史版本")[0])

    mc, mn = CELL.match(isx), NOITEM.match(isx)
    if not mc and not mn:
        print(f"  ✗ {num} 状态列写着「{isx}」，不是分项计数的写法")
        print("     → 怎么办：状态列一律写 `已定 N 项 / 未定 M 项`（没有分项的写 `无分项 · 整条已定`）；")
        print("       跑 bash .claude/gate.d/21-decision-items-sync.sh --write 从正文重新生成。")
        fail = 1
        continue
    if mn:
        if n_settled or n_open:
            print(f"  ✗ {num} 状态列写「无分项」，而正文里数出 {n_settled + n_open} 条分项")
            print("     → 怎么办：跑 bash .claude/gate.d/21-decision-items-sync.sh --write")
            fail = 1
        elif mn.group(1) != kind(bs):
            print(f"  ✗ {num} 状态列写「整条{mn.group(1)}」，正文标题是「{kind(bs)}」")
            print("     → 怎么办：三态词的权威记录是正文首行；改完正文跑 --write 重新生成索引列。")
            fail = 1
        continue
    if (int(mc.group(1)), int(mc.group(2))) != (n_settled, n_open):
        print(f"  ✗ {num} 状态列写「已定 {mc.group(1)} 项 / 未定 {mc.group(2)} 项」，"
              f"而正文两节里数出「已定 {n_settled} 项 / 未定 {n_open} 项」")
        print("     → 怎么办：权威记录是正文的「### 已定项」/「### 未定项」两节。")
        print("       改完正文跑 bash .claude/gate.d/21-decision-items-sync.sh --write")
        fail = 1
        continue
    # 正文标题若自己声明了未定条数，也要与实数相符（写在标题里的那句同样会被检索到）
    if title_open(bs) != n_open:
        print(f"  ✗ {num} 正文标题说「{title_open(bs)} 项未定」，而「### 未定项」一节里是 {n_open} 项")
        print("     → 怎么办：改正文标题里的那个数，或把分项挪到对的那一节去。")
        fail = 1

if not seen:
    print("  ✗ 一条决策的索引行都没核到——本段等于没跑")
    print("     → 怎么办：确认 decisions.md 的索引行形如 `| D1（简称） | … |`，")
    print("       以及 decisions/*.md 首行形如 `## D1 简称 —— 已定`。")
    sys.exit(1)
if not fail:
    print(f"  ✓ 决策索引表的状态列与正文相符（{seen} 条决策，分项数是本段自己数的）")
sys.exit(1 if fail else 0)
