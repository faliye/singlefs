#!/usr/bin/env bash
# gate-stage: kb 形状
#
# kb 形状检查：查「同一件事在 kb 里有两种写法」和「检索时会断掉的指代」。
#
# 这是一个**项目本地门禁阶段**：共享 gate.sh 会扫 .claude/gate.d/*.sh 逐个当阶段跑。
# 单独跑也可以： bash .claude/gate.d/20-kb-shape.sh
#
# 为什么这几条要判红而不是写成提醒：
#   kb 按「被单条取出」设计（.claude/singlefs-ai-sop/rules/kb-discipline.md）。
#   「上表 / 见下方」被单独检索出来时当场断掉，而模型不会说看不懂，它会补一个；
#   同一概念两个名字（未答项 / 未定项）会让按其中一个名字的检索漏掉另一半；
#   标题里写死的条数与列表对不上，检索到标题的人拿到的就是错的。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=.claude/kb
fail=0
bad() { printf '  ✗ %s\n' "$*"; fail=1; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

echo "══ kb 形状检查 ══"
echo

echo "── 1. 同一概念只许一个名字 ──"
hit=$(grep -rn '未答项\|已答项' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "kb 里出现「未答项 / 已答项」"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "统一写「未定项 / 已定项」。records/ 是当时的会话记录，不在本检查范围。"
else ok "未定项 / 已定项 用词统一"; fi

hit=$(grep -rn '^### 未定$\|^### 已定$' $KB/*.md $KB/decisions/*.md 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "小节标题写作「### 未定」/「### 已定」"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "统一写「### 未定项」/「### 已定项」——按标题检索时两种写法只能命中一种。"
else ok "已定项 / 未定项 小节标题统一"; fi

echo
echo "── 2. 跨小节的上下文指代（检索时会断掉）──"
# ⚠️ **已知假阳性：模式是子串匹配，会跨词边界命中。**
# 实测（2026-08-29）：「这条**上表**现相同」里的「上表」被判成按位置指代。
# 收紧模式要冒漏判的风险（真正的「上表」几乎总是紧跟标点或「里/中/的」），
# **现在的处置是接受这个假阳性并改措辞**——它逼出来的改写通常也更自足。
# ⚠️ **「本决策 / 这条决策 / 该决策」也在这一类里，而且更险**：它不像「上表」那样
# 一眼看得出是指代，读起来像个名字。实测（2026-08-30）：全仓 353 处「本决策」被展开成
# 显式编号时，按「行内最近的 D 记号」解析**判错了 15 处**——错的那些一个字都不别扭。
# 决策正文（decisions/）里一处都不许留；历史与记录里同样不许，那里更容易解错。
hit=$(grep -rnE '上表|下表|上面那|上面这|下面那|下面这|上一节|下一节|见上方|见下方' \
        $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "kb 里出现按位置指代的写法"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "换成被指对象的名字（小节标题、表名、决策号），让这一条被单独取出时仍然成立。"
else ok "没有按位置指代的写法"; fi

hit=$(grep -rnE '本决策|这条决策|该决策' $(find "$KB/decisions" -name "*.md") 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "决策正文里出现「本决策 / 这条决策 / 该决策」"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "换成显式的「D<n>（简称）」。实测 2026-08-30：全仓 353 处「本决策」按「行内最近的 D 记号」展开时判错 15 处，错的那些一个字都不别扭。"
else ok "决策正文没有自指式指代"; fi

echo
echo "── 3. 文件内自指链接 ──"
hit=$(grep -n '\[decisions\.md\](decisions\.md)' $KB/decisions.md || true)
if [[ -n "$hit" ]]; then
  bad "decisions.md 里链接到它自己"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "同一文件内直接写决策号（D8），不要链回本文件。"
else ok "没有文件内自指链接"; fi

echo
echo "── 4. 上游规则的路径写法 ──"
# 裸 singlefs-ai-sop/rules/… 从仓库根解析不到，副本在 .claude/ 下。
hit=$(grep -rn '[^/.]singlefs-ai-sop/rules/' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null \
        | grep -v '\.claude/singlefs-ai-sop/rules/' || true)
if [[ -n "$hit" ]]; then
  bad "上游规则写成了裸路径"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "统一写 .claude/singlefs-ai-sop/rules/<文件>.md ——裸路径从仓库根打不开。"
else ok "上游规则路径统一"; fi

echo
echo "── 5. 决策标题：编号连续、带状态；分项两节严格分开、未定条数与列表相符 ──"
# ⚠️ 2026-08-29：此前这一段只读 decisions.md，而决策正文早已拆到 decisions/ 下，
# decisions.md 里一个 `## D<n>` 标题都没有 ⇒ **整段恒绿**，标题声明的未定项条数从没被核过。
# 现在逐个读 decisions/*.md。
python3 - $KB/decisions/*.md <<'PY'
import re, sys
body = ""
for p in sys.argv[1:]:
    body += open(p, encoding="utf-8").read().split("\n## 历史版本", 1)[0] + "\n"
CN = {"一":1,"二":2,"两":2,"三":3,"四":4,"五":5,"六":6,"七":7,"八":8,"九":9,"十":10}
fail = 0
def bad(m, how=""):
    """每一条拒绝都要给下一步——`.claude/singlefs-ai-sop/scripts/gate-lint.sh` 管的是
    上游脚本里的 `bad "..."`，够不着这里的 python，所以这条纪律在本文件里只能自己守。"""
    global fail
    print(f"  ✗ {m}")
    if how:
        print(f"     → {how}")
    fail = 1

# 编号连续（每个文件一个标题，按文件名顺序读进来，所以直接比对）
nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', body, flags=re.M)]
if not nums:
    bad("一个 `## D<n>` 标题都没读到——这一段又变成恒绿了，先修检查再谈别的",
        "确认 decisions/*.md 首行是 `## D<n> 简称 —— 状态`；读不到就是这段检查在空转。")
elif nums != list(range(1, len(nums) + 1)):
    bad(f"决策编号不连续或不从 D1 起：{nums}",
         "决策编号必须从 D1 起连号；废弃一条也要留占位并在变更史里写明。")

# 切成每条决策
starts = [m.start() for m in re.finditer(r'^## D\d+ ', body, flags=re.M)]
starts.append(len(body))
for a, b in zip(starts, starts[1:]):
    blk = body[a:b]
    title = blk.split("\n", 1)[0]
    if not re.search(r'——\s*(已定|半定|待定)', title):
        bad(f"标题缺状态（已定/半定/待定）：{title[:50]}",
            "首行写成 `## D<n> 简称 —— 已定/半定/待定（…）`，索引行要与它一致。")
    # 分项分住两节：「### 已定项」全是已定的，「### 未定项」全是未定的。
    # 只看每节的**索引**（第一个 `####` 之前那一段）——之后是各分项各自的论证，
    # 那里面另有编号列表，混进来会把论证的第 1/2/3 条当成分项。
    def index_of(head):
        sec = re.search(r'^### %s$(.*?)(?=^#{1,3} |\Z)' % head, blk, flags=re.M | re.S)
        if not sec:
            return None
        t = sec.group(1)
        cut = re.search(r'^#{4}\s', t, flags=re.M)
        return t[:cut.start()] if cut else t
    # ① + ② 严格区分：每条分项索引行必须**自带状态词**，且那个状态词要与它所在的小节一致。
    #    kb 按「被单条取出」设计（kb-discipline 第 1 条）：检索端出来的是那一行，
    #    不是它上面的小节标题 ⇒ 状态只挂在位置上，单条取出时状态就没了。
    #
    #    ⚠️ **判状态之前必须先剥掉自引用短语**（「正文见 D2（RAID 条带策略）「已定项 2」」）。
    #    第一版没剥，于是本仓通行的那句自引用**本身就含「已定项」三个字**，
    #    足以让一条状态词被整段删空的行照样通过——2026-08-30 的对抗验证用故障注入当场击穿。
    #    ⚠️ **也不许只认加粗写法**：第一版的正则是 `\*\*未定` 与 `——\s*已定`，
    #    把状态词写成不加粗的「未定」就逃过去了，同一轮验证一并击穿。
    #    ⚠️ **也不许靠「取行内第一个状态词」来猜**：一条合法的分项常常先提到**别处**的状态
    #    （D12 未定项 3 逐字是「判据已定（2026-08-27），但答案的前置未定」，
    #    D13 未定项 2 先提到「D8（核心索引结构） 已定的 write buffer」）——猜法当场两条假阳性。
    #    现在要求每行带一个**规范标记**「状态：已定」/「状态：未定」，判据因此不是猜的。
    #    ⚠️ **判不了的那一半**：行首状态词写对、而后半句自相矛盾（「未定：…这一条已定为…」），
    #    本检查看不见。落点 [checks-owed.md](../kb/checks-owed.md) C51（分项引用只验状态不验身份）。
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        for row in re.findall(r'^(?:\|\s*\d+\s*\||\d+\.\s).*$', seg, flags=re.M):
            marks = re.findall(r'状态：\s*\*{0,2}(已定|未定)', row)
            if not marks:
                bad(f"{title.split()[1]} {head}里有一条分项没写「状态：已定/未定」这个规范标记，"
                    f"状态只挂在小节位置上：{row[:56]}",
                    "在这一行末尾补上 **状态：已定。** 或 **状态：未定。**——检索端出来的是这一行，不是它上面的小节标题。")
            elif len(set(marks)) > 1:
                bad(f"{title.split()[1]} {head}里有一条分项写了两个互相冲突的状态标记："
                    f"{row[:56]}",
                    "一行只许有一个「状态：」标记；要提别处的状态就写成「D<n>（简称） 已定项 k」。")
            elif marks[0] + "项" != head:
                bad(f"{title.split()[1]} {head}里有一条分项的状态标记写着「{marks[0]}」："
                    f"{row[:56]}",
                    "要么把它挪到对应的小节去（编号不变），要么改正状态标记；两节的编号是同一套。")
    # ③ 已定项那一侧的正文不许自陈「还没定」。
    #    实测（2026-08-30 五轮验证）：D23 已定项 8 的正文末尾留着一句
    #    「只走了本地腿，两条云端腿欠着——所以它落成未定项，不是定案」，
    #    而同一分项的标题、以及下游三条分项都以它已定为前提。**状态一致性检查看不见这一类**：
    #    它比的是引用处与索引表，比不了同一分项正文内部自相矛盾。
    #    措辞是窄的、只认「说本项自己没定」那几句，避免误伤「别处仍未定」这种合法陈述
    #    （本仓当前 0 假阳性：同样的词在未定项小节里 0 命中）。
    SELF_UNDECIDED = r'不是定案|落成未定项|退回未定|本项未定|本项仍未定|本项还没定|该项未定|仍未定案|尚未定案'
    regions = []
    m0 = re.search(r'^### 已定项\s*$(.*?)(?=^### |\Z)', blk, flags=re.M | re.S)
    if m0:
        regions.append(m0.group(1))
    for hm in re.finditer(r'^(#{3,4}) 已定项 \d+[^\n]*$', blk, flags=re.M):
        rest = blk[hm.end():]
        nx = re.search(r'^#{1,%d} ' % len(hm.group(1)), rest, flags=re.M)
        regions.append(rest[:nx.start()] if nx else rest)
    for reg in regions:
        for hit in re.finditer(SELF_UNDECIDED, reg):
            a = max(0, hit.start() - 34)
            bad(f"{title.split()[1]} 已定项一侧的正文自陈还没定："
                f"…{reg[a:hit.end() + 10]}…",
                "正文只写现状：定了就删掉这句陈旧的证据等级；真没定就把这条分项挪回未定项小节。")
    # ④ 两节合起来，编号必须**唯一且从 1 连到 n**。
    #    一条决策的分项只有一套编号，分住两节；重号会让「D8 已定项 2」同时指两件事，
    #    断号会让读的人以为中间那条被删了。两者都不改任何状态词 ⇒ 前两条检查看不见。
    nums = []
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        nums += [int(x) for x in (re.findall(r'^\|\s*(\d+)\s*\|', seg, flags=re.M)
                                  or re.findall(r'^(\d+)\.\s', seg, flags=re.M))]
    if nums:
        dup = sorted({n for n in nums if nums.count(n) > 1})
        if dup:
            bad(f"{title.split()[1]} 分项编号重号：{dup}（一套编号分住两节，重号等于一个号指两件事）",
                "给后加的那条换一个没用过的号，并同步改全仓引用；改完跑 .claude/gate.d/22-item-ref-status.sh")
        elif sorted(nums) != list(range(1, len(nums) + 1)):
            bad(f"{title.split()[1]} 分项编号不是从 1 连到 {len(nums)}：{sorted(nums)}",
                "断号说明有分项被删了却没交代。要么补回那一条，要么在变更史里写明它去哪了。")
    # ⑤ 标题声明的未定条数与「### 未定项」小节的分项数相符
    m = re.search(r'([0-9]+|[一二两三四五六七八九十])\s*[项条]未定', title)
    if not m:
        continue
    want = int(m.group(1)) if m.group(1).isdigit() else CN[m.group(1)]
    t = index_of("未定项")
    if t is None:
        bad(f"标题声明了「{m.group(0)}」却没有「### 未定项」小节：{title[:40]}",
                "补一个「### 未定项」小节，或把标题里的未定条数改成 0 并去掉那句。")
        continue
    got = len(re.findall(r'^\|\s*\d+\s*\|', t, flags=re.M)) or \
          len(re.findall(r'^\d+\.\s', t, flags=re.M))
    if not got:
        first = [l for l in t.strip().split("\n") if l.strip()]
        got = 1 if first else 0
        if not got:
            bad(f"「### 未定项」小节里既没有编号条目也没有表格：{title[:40]}",
                "分项要写成带编号的表格行或编号列表，生成器与门禁都按这两种形状抽。")
            continue
    if got != want:
        bad(f"{title.split()[1]} 标题写「{m.group(0)}」，「### 未定项」小节里实际 {got} 项",
                "改正文标题里的条数，并同步 decisions.md 索引行；改完跑 .claude/gate.d/21-decision-items-sync.sh --write")
if not fail:
    print("  ✓ 决策标题与未定项列表相符，且两节没有互相串味")
sys.exit(1 if fail else 0)
PY
[[ $? -ne 0 ]] && fail=1

echo
echo "── 6. 正文（文末历史版本之前）不写历史 ──"
# 正文只写现状，历史进文末（kb-discipline 第 8 条）。
# ⚠️ **此前只扫 decisions.md 一个文件**——25 份决策正文与 69 份实验正文全在视野之外。
# 2026-09-01 审核实测：扩到全部 kb 之后当场抓到 7 处，形态是
# 「此处此前写的是 0.069」「95 已经不是现行头部宽度了」「I-8.1 此前不存在」，
# 其中一处（D23 说那条几何不变量「既不在 invariants.md 也不在 checks-owed.md」）
# 连事实都已经反了——I-8.1 与 C28 两处都在。
# ⚠️ **词表只认「以前写的是 X」这一种形态**，不认「此前没有 / 此前只量了两样」——
# 后者陈述的是一个**至今仍然成立**的缺口，是论证的一部分，不是历史。
# 实测：不做这个区分时 30 处里只有 7 处是真的，假红压倒真红。
hit=$(for f in $(find "$KB" -name '*.md' ! -name '*-history.md' | sort); do
        awk -v F="$f" '/^## 历史版本/{exit} {print F":"FNR": "$0}' "$f"
      done | grep -E '本决策此前|正文此前写|此前写的是|此前写着|此前从未写|此前没有写过|此前没写过|此前不存在|曾经写作|曾经写着|原先写的是|已经不是现行' || true)
if [[ -n "$hit" ]]; then
  bad "正文里出现历史语气"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "正文改写成现状，把「原先如何 / 依据」写进文末「历史版本」。"
else ok "正文没有历史语气"; fi

echo
echo "── 7. 决策索引行的状态 vs 正文标题的状态 ──"
# ⚠️ 2026-08-30 实测踩过：给 D21 加了两个未定项、改了正文标题（两项→四项），
# 而 decisions.md 的索引行还写「两项未定」。第 5 段只比「正文标题 vs 正文列表」，
# 索引页在它的视野之外 ⇒ 这类不一致此前无人拦。
if python3 "$(dirname "$0")/lib-index-vs-body.py" "$KB/decisions.md" $KB/decisions/*.md; then :; else fail=1; fi

echo
if [[ $fail -eq 0 ]]; then echo "  ✓ kb 形状检查通过"; else echo "  ✗ kb 形状检查未通过"; fi
exit $fail
