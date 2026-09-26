#!/usr/bin/env bash
# gate-stage: kb 腐化（1–2 实验号与决策号的引用都有定义；3 不变量声明条数与表对得上、欠账表数得出条数；4 治理文档里的门禁号、路径与「小节」指得到）
# gate-similar: link-targets.py 它只解析 ](相对路径) 链接与「第 N 节」，不看反引号里的路径、「门禁 N 号」与「小节」名；这三类与 1–2 段同是「引用了不存在的东西」，并在这里
#
# kb 腐化审计：查「一处改了、引用它的地方没跟着改」。
#
# 这是一个**项目本地门禁阶段**：共享 gate.sh 会扫 .claude/gate.d/*.sh 逐个当阶段跑。
# 单独跑也可以： bash .claude/gate.d/10-kb-rot.sh
#
# 这类腐化对模型比对人更危险——检索会把陈旧的那一条**单独**端出来，
# 既没有上下文也没有对照（singlefs-ai-sop/rules/kb-discipline.md 第 7 条）。
#
# 四段，都是机械可判的：
#   1–2. 引用了不存在的实验号 / 决策号（doc-lint 已覆盖一部分，这里补实验号）
#   3. 正文写死的条数与实际条数对不上；欠账表数不出条数
#   4. 治理文档（CLAUDE.md、main-agent、agent-common、agents、rules、skills）里「门禁 N 号」没有对应阶段、
#      反引号里的仓内路径不存在、`文件「小节」` 在那份文件里找不到——判据与够不着的写法在 lib-governance-refs.py 文件头
# 实验与决策之间的两件事不在这里判，都归门禁 75 号（三方判决 gate-fix-forks-r1、r2 的 T1）：
#   「实验改成已跑、引用它的决策有没有同批回看」归 ⑤——表里改过一行不够，正文引了它的每条决策都要回看；
#   「已跑的实验有没有对应的决策」归 ⑨——表里至少一行支撑、推翻或备料，标题写了作废或退役的不判。
#
# 每段的成功行都报检查了多少项；本该有对象却一个都没扫到的，判红
# （.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
#
# 判别力：fixtures/10-kb-rot.sh/red 必须判红（悬空的实验号、invariants.md 丢了登记标记、欠账表一行都数不出、第 4 段的悬空门禁号、路径与小节）；
# green 必须判绿。
set -uo pipefail
# 门禁调用时把项目根作为 $1 传进来；单独跑时从脚本位置推。共用库按脚本自己的目录找，在 cd 之前取成绝对路径。
GATE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
cd "${1:-$GATE_DIRECTORY/../..}" || exit 2
KB=.claude/kb
fail=0
say() { printf '  %s\n' "$*"; }
bad() { printf '  ✗ %s\n' "$*"; fail=1; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → 怎么办： %s\n' "$1"; shift; for l in "$@"; do printf '                %s\n' "$l"; done; }

# ⚠️ **必须递归扫**：2026-08-29 决策与实验都拆进了子目录，
# 而引用大半住在那里。用 $KB/*.md 单层通配会静默漏掉它们——
# 实测拆分之后这段检查有一段时间对决策文件里的引用完全失明。
echo "══ kb 腐化审计 ══"
echo
# 扫的文件：kb 下全部 .md（递归）加项目规则。数组装、不靠 $(find …) 的分词，文件名里有空格也不散。
kb_md_files=()
while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
  < <(find "$KB" -name '*.md' -print0 2>/dev/null)
shopt -s nullglob
rule_md_files=(.claude/rules/*.md)
shopt -u nullglob
scanned_md_files=("${kb_md_files[@]}" "${rule_md_files[@]}")
scanned_note="扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则"

echo "── 1. 实验号引用是否都有定义 ──"
missing=0
experiment_refs=()
# ⚠️ 不能用 \bE[0-9]+\b —— 它会把 URL 里的 E19253-01 当成实验号（实测踩过）。
if [[ ${#scanned_md_files[@]} -gt 0 ]]; then
  mapfile -t experiment_refs < <(grep -ohE '(^|[^A-Za-z0-9/-])E[0-9]{1,3}([^A-Za-z0-9-]|$)' "${scanned_md_files[@]}" 2>/dev/null \
                                   | grep -oE 'E[0-9]{1,3}' | sort -u)
fi
for e in "${experiment_refs[@]}"; do
  grep -rqE "^## $e " "$KB/experiments" 2>/dev/null || { bad "$e 被引用但 experiments/ 下没有它"
    howto "要么在 experiments/ 下给它建正文（\`## $e <简称>\` 起头），" \
          "要么把引用它的那处改成真实存在的实验号——编号引用悬空，检索到的人会自己补一个。"
    missing=1; }
done
if [[ ${#experiment_refs[@]} -eq 0 ]]; then
  bad "一个实验号引用都没扫到（$scanned_note）——这一段没有对象可判"
  howto "确认门禁是在仓库根上跑的（第一个参数是仓库根）、$KB 目录在；" \
        "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
elif [[ $missing -eq 0 ]]; then
  ok "实验号引用全部有定义：${#experiment_refs[@]} 个不同的实验号（$scanned_note）"
fi

echo
echo "── 2. 决策号引用是否都有定义 ──"
missing=0
decision_refs=()
if [[ ${#scanned_md_files[@]} -gt 0 ]]; then
  mapfile -t decision_refs < <(grep -ohE '(^|[^A-Za-z0-9/-])D[0-9]{1,3}([^A-Za-z0-9-]|$)' "${scanned_md_files[@]}" 2>/dev/null \
                                 | grep -oE 'D[0-9]{1,3}' | sort -u)
fi
for d in "${decision_refs[@]}"; do
  grep -rqE "^## $d " "$KB/decisions" 2>/dev/null || { bad "$d 被引用但 decisions/ 下没有它"
    howto "要么在 decisions/ 下给它建正文（\`## $d <简称>\` 起头），" \
          "要么把引用它的那处改成真实存在的决策号。"
    missing=1; }
done
if [[ ${#decision_refs[@]} -eq 0 ]]; then
  bad "一个决策号引用都没扫到（$scanned_note）——这一段没有对象可判"
  howto "确认门禁是在仓库根上跑的、$KB 目录在；" \
        "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
elif [[ $missing -eq 0 ]]; then
  ok "决策号引用全部有定义：${#decision_refs[@]} 个不同的决策号（$scanned_note）"
fi

echo
echo "── 3. 正文写死的条数 vs 实际条数 ──"
if [[ ! -f "$KB/invariants.md" ]]; then
  bad "找不到 $KB/invariants.md，不变量条数无从核对"
  howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
else
  inv_actual=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$KB/invariants.md")
  # 表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列
  inv_retired=$(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$KB/invariants.md")
  inv_live=$(( inv_actual - inv_retired ))
  # 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句（2026-09-18 立）：历史版本里也有「现共 N 条在用」，
  # 那是当时的数、不跟着改，按文件序取第一处会取到历史里的那一句（实测：2026-09-18 取到 2026-09-14 那条的 66）。
  # 标记丢了、或下一行读不出那句，都判红：退回去取文件里第一处「N 条在用」，取到的正是那句历史。
  marker_line_number=$(grep -n -m1 -F '<!-- invariant-count -->' "$KB/invariants.md" | cut -d: -f1)
  if [[ "$inv_actual" -eq 0 ]]; then
    bad "invariants.md 里一行 \`| I-<章>.<号> \` 表行都没数到——不变量表的写法变了，或这份文件是空的"
    howto "表行形如 \`| I-1.1 | 简称 | 陈述 | … |\`；写法真改了的话，这里与 36 号的正则一起改。"
  elif [[ -z "$marker_line_number" ]]; then
    bad "invariants.md 里没有 <!-- invariant-count --> 标记——当前条数的登记位丢了，读不出正文声称几条（表里在用 $inv_live 条）"
    howto "在「现共 N 条在用」那句的上一行补回 <!-- invariant-count -->；" \
          "不许让检查退回去取文件里第一处「N 条在用」：历史版本里那几句记的是当时的数。"
  else
    claim_line=$(sed -n "$((marker_line_number + 1))p" "$KB/invariants.md")
    inv_claim=$(grep -oE '现共 [0-9]+ 条在用' <<<"$claim_line" | grep -oE '[0-9]+' | head -1)
    if [[ -z "$inv_claim" ]]; then
      bad "invariants.md 第 $((marker_line_number + 1)) 行（<!-- invariant-count --> 的下一行）读不出「现共 N 条在用」：${claim_line:0:60}"
      howto "标记的下一行就写那句「现共 $inv_live 条在用（…）」，中间不空行；" \
            "读不出声明就是没核过，不许当成一致。"
    elif [[ "$inv_claim" != "$inv_live" ]]; then
      bad "invariants.md 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
      howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
            "两个数对不上时，读的人不知道该信哪一个。"
    else
      ok "不变量条数一致：正文声称 $inv_claim 条在用，表里在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
    fi
  fi
fi
# 三段各数各的：欠着的那张表、「### 已还清」那张表、「## 历史版本」里的条目。
# ⚠️ **分界线是「### 已还清」，不是「## 历史版本」**：已还清那张表住在历史版本**之前**，
# 按历史版本切会把还清的全算进欠账里——2026-09-16 现查它报「欠 326、已还清 0」，
# 真数是 297 / 29。一条报错数的检查与没有这条检查，在门禁输出里长得一模一样。
# 数不出来（文件不在、awk 出错、两张表一行都没数到）判红：两个空白或两个 0 印在成功行里，看着就像数过了。
if [[ ! -f "$KB/checks-owed.md" ]]; then
  bad "找不到 $KB/checks-owed.md，欠账条数取不到"
  howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
else
  # 开着与已还清的切法用共用读法 lib-owed.py（67、92、96 号同一份），不在这里再抄一份 awk
  chk_counts=""
  if chk_counts=$(python3 - "$GATE_DIRECTORY/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
table = lib_owed.read_owed_table(sys.argv[2])
print(len(table.open_names), len(table.paid_names), int(table.paid_heading_found))
PY_OWED
  ); then :; else chk_counts=""; fi
  read -r chk_actual chk_done chk_heading <<<"$chk_counts"
  if [[ -z "${chk_actual:-}" || -z "${chk_done:-}" ]]; then
    bad "共用读法 lib-owed.py 没数出 checks-owed.md 的条数（输出「$chk_counts」）"
    howto "单独跑一遍上面那段 python 看它报什么错；数不出来就是没核过，不许当成数过了。"
  elif [[ $((chk_actual + chk_done)) -eq 0 ]]; then
    bad "checks-owed.md 里一行 \`| C<n> \` 都没数到（欠着 0、已还清 0）——欠账表的写法变了，或这份文件是空的"
    howto "欠账行形如 \`| C12 | 简称 | … |\`；写法真改了的话，改 .claude/gate.d/lib-owed.py 的正则，67、92、96 号与这里一起跟上。"
  elif [[ "${chk_heading:-0}" != 1 ]]; then
    bad "checks-owed.md 里认不出「已还清」标题——开着的与已还清的分不开，$chk_actual 条全被算成欠着"
    howto "已还清那张表上面要有一行「### 已还清」；标题改过名的话，改 .claude/gate.d/lib-owed.py 认标题的那条正则。"
  else
    ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
  fi
fi

echo "── 4. 治理文档里的指向 ──"
governance_rc=0
python3 "$GATE_DIRECTORY/lib-governance-refs.py" || governance_rc=$?
if [[ $governance_rc -eq 0 ]]; then
  ok "治理文档里的门禁号、路径与「小节」都指得到"
elif [[ $governance_rc -eq 3 ]]; then
  bad "一份治理文档都没扫到——这一段没有对象可判"
  howto "确认门禁是在仓库根上跑的；治理文档搬了家的话，改 lib-governance-refs.py 的 CARRIER_PATTERNS。"
elif [[ $governance_rc -ne 1 ]]; then
  bad "lib-governance-refs.py 没跑成（退出码 $governance_rc）——这一段没判"
  howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看它报什么错；没跑成就是没判，不许当成判过了。"
else
  bad "治理文档里有指不到的指向（上面逐条列出）"
  howto "门禁号：阶段被删或收归上游的，改成共享 gate.sh 里那一道的名字（「链接指向」这类）或现存的号；" \
        "路径：搬了家的改成新路径，已删的改指现存的做法，已归档的写裸文件名并指到取法（.claude/agent-common.md「找不到历史实验的数据」那一条）；" \
        "小节：按那份文件今天的标题改。改 agent 定义与共用约束照走门禁 72 号那一条。"
fi

echo
if [[ $fail -ne 0 ]]; then
  echo "  ✗ kb 腐化审计发现问题"
  echo "     → 怎么办：上面每一条都要处理掉，二选一——按现状改正文，"
  echo "               或者在那一条旁边写明为什么现在不用改（带日期和依据）。"
  echo "               放着不动会让下一轮再审一遍同样的东西。"
  exit 1
fi
echo "  ✓ kb 腐化审计通过"
