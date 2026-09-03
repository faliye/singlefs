#!/usr/bin/env bash
# gate-stage: kb 腐化
#
# kb 腐化审计：查「一处改了、引用它的地方没跟着改」。
#
# 这是一个**项目本地门禁阶段**：共享 gate.sh 会扫 .claude/gate.d/*.sh 逐个当阶段跑。
# 单独跑也可以： bash .claude/gate.d/10-kb-rot.sh
#
# 这类腐化对模型比对人更危险——检索会把陈旧的那一条**单独**端出来，
# 既没有上下文也没有对照（singlefs-ai-sop/rules/kb-discipline.md 第 7 条）。
#
# 三类机械可判的：
#   1. 实验状态与引用它的决策不同步（按 git 判：改状态的那个提交有没有同时动 decisions.md）
#   2. 正文里写死的条数与实际条数对不上
#   3. 引用了不存在的编号（doc-lint 已覆盖一部分，这里补实验号）
set -uo pipefail
# 门禁调用时把项目根作为 $1 传进来；单独跑时从脚本位置推。
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=.claude/kb
fail=0
say() { printf '  %s\n' "$*"; }
bad() { printf '  ✗ %s\n' "$*"; fail=1; }
ok()  { printf '  ✓ %s\n' "$*"; }

# ⚠️ **必须递归扫**：2026-08-29 决策与实验都拆进了子目录，
# 而引用大半住在那里。用 $KB/*.md 单层通配会静默漏掉它们——
# 实测拆分之后这段检查有一段时间对决策文件里的引用完全失明。
echo "══ kb 腐化审计 ══"
echo
echo "── 1. 实验号引用是否都有定义 ──"
missing=0
# ⚠️ 不能用 \bE[0-9]+\b —— 它会把 URL 里的 E19253-01 当成实验号（实测踩过）。
for e in $(grep -ohE '(^|[^A-Za-z0-9/-])E[0-9]{1,3}([^A-Za-z0-9-]|$)' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null \
             | grep -oE 'E[0-9]{1,3}' | sort -u); do
  grep -rqE "^## $e " "$KB/experiments" || { bad "$e 被引用但 experiments/ 下没有它"; missing=1; }
done
[[ $missing -eq 0 ]] && ok "实验号引用全部有定义"

echo
echo "── 2. 决策号引用是否都有定义 ──"
missing=0
for d in $(grep -ohE '(^|[^A-Za-z0-9/-])D[0-9]{1,3}([^A-Za-z0-9-]|$)' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null \
             | grep -oE 'D[0-9]{1,3}' | sort -u); do
  grep -rqE "^## $d " "$KB/decisions"  || { bad "$d 被引用但 decisions/ 下没有它"; missing=1; }
done
[[ $missing -eq 0 ]] && ok "决策号引用全部有定义"

echo
echo "── 3. 已跑的实验，引用它的决策有没有跟着改 ──"
# 判据：该实验状态行最后一次变动的提交，有没有同时改 decisions.md。
# 没有 ⇒ 至少要人看一眼。这不是「一定错」，是「一定没人对过」。
stale=0
while read -r line; do
  e="${line%% *}"
  grep -q "已跑" <<<"$line" || continue
  # 引用它的决策
  refs=$(grep -nE "\b$e\b" "$KB/decisions.md" | head -3 | cut -d: -f1 | paste -sd, -)
  [[ -n "$refs" ]] || continue
  c=$(git log -1 --format=%h -S"## $e " -- "$KB/experiments" "$KB/experiments.md" 2>/dev/null)
  [[ -n "$c" ]] || continue
  if git show --stat --format= "$c" 2>/dev/null | grep -q "decisions.md"; then
    ok "$e 已跑，其状态变动的提交 $c 同时动过 decisions.md"
  else
    bad "$e 已跑，但把它改成已跑的提交 $c **没有动 decisions.md**（decisions.md 第 $refs 行引用了它）"
    stale=1
  fi
done < <(cat "$KB"/experiments/*.md | grep -E "^## E[0-9]+ " | sed 's/^## //')
[[ $stale -eq 0 ]] && ok "已跑实验与引用它的决策都在同一个提交里动过"

echo
echo "── 4. 已跑的实验有没有决策引用 ──"
# 一个实验跑完、产出了决策相关的结果，却没有任何决策引用它 ⇒ 那个结果没落进任何判断。
# 这不是「引用格式问题」，是**结论悬空**。
orphan=0
while read -r line; do
  e="${line%% *}"
  grep -q "已跑" <<<"$line" || continue
  n=$(cat "$KB/decisions.md" "$KB"/decisions/*.md | grep -cE "(^|[^A-Za-z0-9/-])$e([^A-Za-z0-9-]|$)")
  if [[ "$n" -eq 0 ]]; then
    bad "$e 已跑，但决策正文一次都没引用它——它的结论悬空了"
    orphan=1
  fi
done < <(cat "$KB"/experiments/*.md | grep -E "^## E[0-9]+ " | sed 's/^## //')
[[ $orphan -eq 0 ]] && ok "每个已跑实验都至少被一条决策引用"

echo
echo "── 5. 正文写死的条数 vs 实际条数 ──"
inv_actual=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$KB/invariants.md")
# 表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列
inv_retired=$(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$KB/invariants.md")
inv_live=$(( inv_actual - inv_retired ))
# 历史版本是新的在前 ⇒ 取**第一处**。取 tail 会拿到最老那条，永远判红（实测踩过）。
inv_claim=$(grep -oE '现共 [0-9]+ 条在用' "$KB/invariants.md" | head -1 | grep -oE '[0-9]+')
if [[ -n "$inv_claim" && "$inv_claim" != "$inv_live" ]]; then
  bad "invariants.md 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
else
  ok "不变量条数一致：在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
fi
# 历史版本里的「已还清」表也是 `| C<n> |` 行 —— 一起数会把还清的算进欠账里。
chk_actual=$(sed '/^## 历史版本/,$d' "$KB/checks-owed.md" | grep -cE '^\| C[0-9]+ ')
chk_done=$(sed -n '/^## 历史版本/,$p' "$KB/checks-owed.md" | grep -cE '^\| C[0-9]+ ')
ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"

echo
if [[ $fail -ne 0 ]]; then
  echo "  ✗ kb 腐化审计发现问题——上面每一条都要么改、要么写明为什么不用改"
  exit 1
fi
echo "  ✓ kb 腐化审计通过"
