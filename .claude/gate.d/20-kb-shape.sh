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

hit=$(grep -rn '^### 未定$' $KB/*.md 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "小节标题写作「### 未定」"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "统一写「### 未定项」——按标题检索时两种写法只能命中一种。"
else ok "未定项小节标题统一"; fi

echo
echo "── 2. 跨小节的上下文指代（检索时会断掉）──"
# ⚠️ **已知假阳性：模式是子串匹配，会跨词边界命中。**
# 实测（2026-08-29）：「这条**上表**现相同」里的「上表」被判成按位置指代。
# 收紧模式要冒漏判的风险（真正的「上表」几乎总是紧跟标点或「里/中/的」），
# **现在的处置是接受这个假阳性并改措辞**——它逼出来的改写通常也更自足。
hit=$(grep -rnE '上表|下表|上面那|上面这|下面那|下面这|上一节|下一节|见上方|见下方' \
        $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null || true)
if [[ -n "$hit" ]]; then
  bad "kb 里出现按位置指代的写法"; printf '%s\n' "$hit" | sed 's/^/     /'
  howto "换成被指对象的名字（小节标题、表名、决策号），让这一条被单独取出时仍然成立。"
else ok "没有按位置指代的写法"; fi

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
echo "── 5. 决策标题：编号连续、带状态、未定项条数与列表相符 ──"
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
def bad(m):
    global fail; print(f"  ✗ {m}"); fail = 1

# 编号连续（每个文件一个标题，按文件名顺序读进来，所以直接比对）
nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', body, flags=re.M)]
if not nums:
    bad("一个 `## D<n>` 标题都没读到——这一段又变成恒绿了，先修检查再谈别的")
elif nums != list(range(1, len(nums) + 1)):
    bad(f"决策编号不连续或不从 D1 起：{nums}")

# 切成每条决策
starts = [m.start() for m in re.finditer(r'^## D\d+ ', body, flags=re.M)]
starts.append(len(body))
for a, b in zip(starts, starts[1:]):
    blk = body[a:b]
    title = blk.split("\n", 1)[0]
    if not re.search(r'——\s*(已定|半定|待定)', title):
        bad(f"标题缺状态（已定/半定/待定）：{title[:50]}")
    m = re.search(r'([0-9]+|[一二两三四五六七八九十])\s*[项条]未定', title)
    if not m:
        continue
    want = int(m.group(1)) if m.group(1).isdigit() else CN[m.group(1)]
    sec = re.search(r'^### 未定项$(.*?)(?=^### |\Z)', blk, flags=re.M | re.S)
    if not sec:
        bad(f"标题声明了「{m.group(0)}」却没有「### 未定项」小节：{title[:40]}")
        continue
    t = sec.group(1)
    items = re.findall(r'^\d+\.\s+(.*)$', t, flags=re.M)
    rows = [r for r in re.findall(r'^\|\s*\d+\s*\|.*$', t, flags=re.M)]
    if items:
        got = sum(1 for it in items if not re.search(r'——\s*已定', it))
    elif rows:
        got = sum(1 for r in rows if "**未定" in r)
    else:
        bad(f"「### 未定项」小节里既没有编号条目也没有表格：{title[:40]}")
        continue
    if got != want:
        bad(f"{title.split()[1]} 标题写「{m.group(0)}」，列表里实际 {got} 项")
if not fail:
    print("  ✓ 决策标题与未定项列表相符")
sys.exit(1 if fail else 0)
PY
[[ $? -ne 0 ]] && fail=1

echo
echo "── 6. 正文（文末历史版本之前）不写历史 ──"
# 正文只写现状，历史进文末（kb-discipline 第 7 条）。
hit=$(awk '/^## 历史版本/{exit} {print FILENAME":"FNR": "$0}' $KB/decisions.md \
        | grep -E '本决策此前|此前从未写|此前没有写过|此前没写过|曾经写作' || true)
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
