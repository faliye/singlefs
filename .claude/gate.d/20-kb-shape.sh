#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: kb 形状（1 未定项 / 已定项只许一种叫法；3 kb 文件不许链回自己；4 上游规则路径带 .claude/；5 决策标题连号带状态、分项两节不串味不重号、标题未定条数与列表相符；7 索引页状态列的分项计数与正文相符）
#
# kb 形状检查：查「同一件事在 kb 里有两种写法」。
# 指代（位置指代、自指称呼、时间指代）由上游 doc-lint.sh 判，这里不重复。
#
# 这是一个**项目本地门禁阶段**：共享 gate.sh 会扫 .claude/gate.d/*.sh 逐个当阶段跑。
# 单独跑也可以： bash .claude/gate.d/20-kb-shape.sh
#
# 为什么这几条要判红而不是写成提醒：
#   kb 按「被单条取出」设计（.claude/singlefs-ai-sop/rules/kb-discipline.md）。
#   同一概念两个名字（未答项 / 未定项）会让按其中一个名字的检索漏掉另一半；
#   标题里写死的条数与列表对不上，检索到标题的人拿到的就是错的。
#
# 判别力：fixtures/20-kb-shape.sh/red 每一段都至少犯一次，必须判红；green 必须判绿。
# 第 7 段「一条决策的索引行都没核到」与「状态列的分项计数与正文不符」互斥（后者要先核到一行），
# 红样本里放的是后者。
set -uo pipefail
# lib 的路径要在 cd 之前算：$0 是相对路径时，cd 进项目根之后就指不到了
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
LIB_INDEX_VS_BODY="$(cd "$(dirname "$0")" && pwd)/lib-index-vs-body.py"
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=.claude/kb
fail=0
bad() { printf '  ✗ %s\n' "$*"; fail=1; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

# 扫的文件装进数组，不靠 $(find …) 的分词：文件名里有空格时会被切成两个不存在的路径，
# 而 grep 打不开它们的报错原先丢进 /dev/null——那一份里写了什么都判不到。
kb_md_files=()
while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
  < <(find "$KB" -name '*.md' -print0 2>/dev/null)
shopt -s nullglob
rule_md_files=(.claude/rules/*.md)
kb_top_md_files=("$KB"/*.md "$KB"/decisions/*.md)
shopt -u nullglob
kb_and_rule_md_files=(${kb_md_files[@]+"${kb_md_files[@]}"} ${rule_md_files[@]+"${rule_md_files[@]}"})
grep_error_file="$(mktemp)"
trap 'rm -f "${grep_error_file:?}"' EXIT
# 用法：grep_files <这一段叫什么> <grep 参数…> -- <文件…>；命中写进 $grep_hits。
# 退 0：都读到了。退 1：一份要扫的文件都没有（判红）。退 2：有文件读不了（判红；读到的那些照样交命中）——
# 出错的那几份里写了什么都没判，不能读成「没命中」，所以调用方只在退 0 时报绿。
grep_files() {
  local stage_label="$1" grep_exit_code=0; shift
  local -a grep_options=()
  while [[ $# -gt 0 && "$1" != "--" ]]; do grep_options+=("$1"); shift; done
  shift
  grep_hits=""
  if [[ $# -eq 0 ]]; then
    bad "$stage_label：一份要扫的文件都没有，这一段没判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 份不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
    return 1
  fi
  grep_hits="$(grep -Hn "${grep_options[@]}" -- "$@" 2>"$grep_error_file")" || grep_exit_code=$?
  if [[ "$grep_exit_code" -ge 2 ]]; then
    bad "$stage_label：有文件读不了（grep 退 $grep_exit_code），那几份没判："; sed 's/^/     /' "$grep_error_file"
    howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；读不了的那几份不能当成没命中。"
    return 2
  fi
  return 0
}

echo "══ kb 形状检查 ══"
echo

echo "── 1. 同一概念只许一个名字 ──"
grep_status=0
grep_files "「未答项 / 已答项」" -e '未答项' -e '已答项' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
if [[ -n "$grep_hits" ]]; then
  bad "kb 里出现「未答项 / 已答项」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
  howto "统一写「未定项 / 已定项」。records/ 是当时的会话记录，不在本检查范围。"
elif [[ "$grep_status" -eq 0 ]]; then ok "未定项 / 已定项 用词统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

grep_status=0
grep_files "「### 未定」/「### 已定」小节标题" -e '^### 未定$' -e '^### 已定$' -- ${kb_top_md_files[@]+"${kb_top_md_files[@]}"} || grep_status=$?
if [[ -n "$grep_hits" ]]; then
  bad "小节标题写作「### 未定」/「### 已定」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
  howto "统一写「### 未定项」/「### 已定项」——按标题检索时两种写法只能命中一种。"
elif [[ "$grep_status" -eq 0 ]]; then ok "已定项 / 未定项 小节标题统一（扫 $KB 顶层与 decisions/ 下 ${#kb_top_md_files[@]} 份）"; fi

echo
echo "── 3. 文件内自指链接 ──"
# 扫 kb 下每一份 .md：链接目标（去掉 #锚点）按这份文件自己的目录解析之后就是它自己，判红。
# 历史类文件（`*-history.md`、`decisions-history/` 下的月份文件）整份跳过：它们逐字记着当时的原文，
# 里面抄录的链接多半是当时的原文，这一道分不出哪一句换了会成假话，所以整份不判；这是这一道的射程，
# 不是豁免：`.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」里「历史类文件同样换名」那一段要逐句判，历史类文件里的自链靠人看。
self_link_report=$(python3 - "$KB" <<'PY'
import os, re, sys, glob
kb = sys.argv[1]
scanned = 0
for path in sorted(glob.glob(os.path.join(kb, "**", "*.md"), recursive=True)):
    if os.path.basename(path).endswith("-history.md") or "/decisions-history/" in path:
        continue
    scanned += 1
    own = os.path.normpath(path)
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            for link in re.finditer(r'\[[^\]\n]*\]\(([^)\s]+)\)', line):
                target = link.group(1).split("#", 1)[0]
                if not target or "://" in target:
                    continue
                if os.path.normpath(os.path.join(os.path.dirname(path), target)) == own:
                    print(f"HIT {path}:{line_number}: {link.group(0)}")
print(f"SCANNED {scanned}")
PY
)
self_link_scanned=$(sed -n 's/^SCANNED //p' <<<"$self_link_report")
hit=$(sed -n 's/^HIT //p' <<<"$self_link_report")
if [[ -z "$self_link_scanned" ]]; then
  bad "扫自指链接的 python 没跑完（没报出扫了几份），这一段等于没判"
  howto "单独跑这一段看它报什么错（多半是某份文件不是 UTF-8）；没跑完不许当成没有自指链接。"
elif [[ "$self_link_scanned" -eq 0 ]]; then
  bad "一份 kb 文件都没扫到（$KB 下的 .md，历史类文件除外）——这一段没有对象可判"
  howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md）。"
elif [[ -n "$hit" ]]; then
  bad "kb 文件里有链接指回它自己：$(grep -c . <<<"$hit") 处（扫 $self_link_scanned 份）"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "同一文件内直接写编号带简称（例：D8（核心索引结构））或小节标题，不要链回本文件。"
else ok "没有文件内自指链接（扫 $self_link_scanned 份 kb 文件，历史类文件除外）"; fi

echo
echo "── 4. 上游规则的路径写法 ──"
# 裸 singlefs-ai-sop/rules/… 从仓库根解析不到，副本在 .claude/ 下。行首的裸路径也算（前面没有字符可配）。
grep_status=0
grep_files "上游规则的裸路径" -E -e '(^|[^/.])singlefs-ai-sop/rules/' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
hit=""
[[ -n "$grep_hits" ]] && hit="$(grep -v '\.claude/singlefs-ai-sop/rules/' <<<"$grep_hits")"
if [[ -n "$hit" ]]; then
  bad "上游规则写成了裸路径"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "统一写 .claude/singlefs-ai-sop/rules/<文件>.md ——裸路径从仓库根打不开。"
elif [[ "$grep_status" -eq 0 ]]; then ok "上游规则路径统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

echo
echo "── 5. 决策标题：编号连续、带状态；分项两节严格分开、未定条数与列表相符 ──"
# ⚠️ 2026-08-29：此前这一段只读 decisions.md，而决策正文早已拆到 decisions/ 下，
# decisions.md 里一个 `## D<n>` 标题都没有 ⇒ **整段恒绿**，标题声明的未定项条数从没被核过。
# 现在逐个读 decisions/*.md。
LIB_INDEX_VS_BODY="$LIB_INDEX_VS_BODY" python3 - $KB/decisions/*.md <<'PY'
import importlib.util, os, re, sys
# 标题里未定条数的读法与第 7 段同一份（lib-index-vs-body.py 的 declared_open_count），不各抄一份
spec = importlib.util.spec_from_file_location("lib_index_vs_body", os.environ["LIB_INDEX_VS_BODY"])
lib_index_vs_body = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_index_vs_body)
body = ""
for p in sys.argv[1:]:
    body += open(p, encoding="utf-8").read().split("\n## 历史版本", 1)[0] + "\n"
fail = 0
index_rows_checked = 0
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
decision_count = len(starts) - 1
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
            index_rows_checked += 1
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
    #    中文数字按 1–99 读（「十一项未定」是 11，不是 1），读法在 lib-index-vs-body.py
    declared = lib_index_vs_body.declared_open_count(title)
    if declared is None:
        continue
    want, declared_phrase = declared
    if want is None:
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，里面的数认不出来",
            "条数写阿拉伯数字，或 1–99 的中文数字（「十一项未定」「两项未定」）。")
        continue
    t = index_of("未定项")
    if t is None:
        bad(f"标题声明了「{declared_phrase}」却没有「### 未定项」小节：{title[:40]}",
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
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，「### 未定项」小节里实际 {got} 项",
                "改正文标题里的条数，并同步 decisions.md 索引行；改完跑 .claude/gate.d/21-decision-items-sync.sh --write")
if not fail:
    print(f"  ✓ 决策标题与未定项列表相符，且两节没有互相串味（{decision_count} 条决策、核 {index_rows_checked} 行分项索引）")
sys.exit(1 if fail else 0)
PY
[[ $? -ne 0 ]] && fail=1

echo
echo "── 7. 决策索引表的状态列（分项计数）vs 正文实际分项数 ──"
# ⚠️ 2026-08-30 实测踩过：给 D21 加了两个未定项、改了正文标题（两项→四项），
# 而 decisions.md 的索引行还写「两项未定」。第 5 段只比「正文标题 vs 正文列表」，
# 索引页在它的视野之外 ⇒ 这类不一致此前无人拦。
# ⚠️ 本段**自己数正文**，不经过 gen-decision-items.py：写回索引页的是那个生成器，
# 21 阶段拿它的输出与索引页逐字比对 ⇒ 生成器数错时两边一起错，只有本段会红。
if python3 "$LIB_INDEX_VS_BODY" "$KB/decisions.md" $KB/decisions/*.md; then :; else fail=1; fi

echo
if [[ $fail -eq 0 ]]; then echo "  ✓ kb 形状检查通过（kb 文件 ${#kb_md_files[@]} 份、规则 ${#rule_md_files[@]} 份）"; else echo "  ✗ kb 形状检查未通过"; fi   # gate-lint:summary
exit $fail
