# 附录二：门禁修复第二轮代码改动（跟踪文件用 `git -c core.quotepath=false diff HEAD -- <路径…>` 原样，加未跟踪文件全文与一份对照用未改文件全文；生成于 2026-09-23 15:43 UTC）

基准：HEAD `3b60f09`（2026-09-22 07:27:29 +0000）。这一批改动没有提交点，全部在工作区（未暂存）。

不进本附录（别的会话同时在改，主 agent 明确排除）：`.claude/gate.d/47-research-script-selftests.sh`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/72-agent-def-adversarial-review.sh`、`research/scripts/check-segment-registry.py`。

## 一、diff（跟踪文件相对 HEAD `3b60f09` 的工作区改动）

命令：

```
git -c core.quotepath=false diff HEAD -- \
  .claude/gate.d/10-kb-rot.sh .claude/gate.d/fixtures/10-kb-rot.sh/ \
  .claude/rules/implementation-workflow.md \
  .claude/gate.d/80-absolute-assertions.sh .claude/gate.d/fixtures/80-absolute-assertions.sh/ \
  .claude/gate.d/40-results-cited.sh .claude/gate.d/61-settled-same-file.sh \
  .claude/gate.d/88-quoted-result-lines.sh .claude/gate.d/92-layout-checker-sync.sh \
  .claude/gate.d/fixtures/40-results-cited.sh/ .claude/gate.d/fixtures/61-settled-same-file.sh/ \
  .claude/gate.d/fixtures/88-quoted-result-lines.sh/ \
  .claude/gate.d/30-decision-history.sh .claude/gate.d/66-abandoned-rounds.sh \
  .claude/gate.d/52-segment-registry.sh .claude/gate.d/31-blocking-verdict.sh \
  .claude/scripts/gen-decision-items.py
```

（这些文件 HEAD 以来的改动里还含这一批更早的修复，按主 agent 指示一并放入、不截。）

```diff
diff --git a/.claude/gate.d/10-kb-rot.sh b/.claude/gate.d/10-kb-rot.sh
index 448b4e5..000609f 100755
--- a/.claude/gate.d/10-kb-rot.sh
+++ b/.claude/gate.d/10-kb-rot.sh
@@ -1,5 +1,5 @@
 #!/usr/bin/env bash
-# gate-stage: kb 腐化
+# gate-stage: kb 腐化（1–2 实验号与决策号的引用都有定义；3 已跑实验被决策正文引用；4 不变量声明条数与表对得上、欠账表数得出条数）
 #
 # kb 腐化审计：查「一处改了、引用它的地方没跟着改」。
 #
@@ -9,10 +9,19 @@
 # 这类腐化对模型比对人更危险——检索会把陈旧的那一条**单独**端出来，
 # 既没有上下文也没有对照（singlefs-ai-sop/rules/kb-discipline.md 第 7 条）。
 #
-# 三类机械可判的：
-#   1. 实验状态与引用它的决策不同步（按 git 判：改状态的那个提交有没有同时动 decisions.md）
-#   2. 正文里写死的条数与实际条数对不上
-#   3. 引用了不存在的编号（doc-lint 已覆盖一部分，这里补实验号）
+# 四段，都是机械可判的：
+#   1–2. 引用了不存在的实验号 / 决策号（doc-lint 已覆盖一部分，这里补实验号）
+#   3. 已跑的实验没有任何决策正文引用它（结论悬空），除非正文写明备料在等谁
+#   4. 正文写死的条数与实际条数对不上；欠账表数不出条数
+# 「实验改成已跑、引用它的决策有没有同批回看」不在这里判：门禁 75 号 ⑤ 把实验页标题行的变动算作正文改了，
+# 要求同一批回看影响的决策表、写「改了」的那条决策文件在同一批里（三方判决 gate-fix-forks-r1 的 T1）。
+#
+# 每段的成功行都报检查了多少项；本该有对象却一个都没扫到的，判红
+# （.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
+#
+# 判别力：fixtures/10-kb-rot.sh/red 必须判红（悬空的实验号、结论悬空的已跑实验、
+# invariants.md 丢了登记标记、欠账表一行都数不出）；
+# green 必须判绿。
 set -uo pipefail
 # 门禁调用时把项目根作为 $1 传进来；单独跑时从脚本位置推。
 cd "${1:-$(dirname "$0")/../..}" || exit 2
@@ -28,87 +37,98 @@ howto() { printf '     → 怎么办： %s\n' "$1"; shift; for l in "$@"; do pri
 # 实测拆分之后这段检查有一段时间对决策文件里的引用完全失明。
 echo "══ kb 腐化审计 ══"
 echo
+# 扫的文件：kb 下全部 .md（递归）加项目规则。数组装、不靠 $(find …) 的分词，文件名里有空格也不散。
+kb_md_files=()
+while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
+  < <(find "$KB" -name '*.md' -print0 2>/dev/null)
+shopt -s nullglob
+rule_md_files=(.claude/rules/*.md)
+experiment_body_files=("$KB"/experiments/*.md)
+decision_body_files=("$KB"/decisions/*.md)
+shopt -u nullglob
+scanned_md_files=("${kb_md_files[@]}" "${rule_md_files[@]}")
+scanned_note="扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则"
+
 echo "── 1. 实验号引用是否都有定义 ──"
 missing=0
+experiment_refs=()
 # ⚠️ 不能用 \bE[0-9]+\b —— 它会把 URL 里的 E19253-01 当成实验号（实测踩过）。
-for e in $(grep -ohE '(^|[^A-Za-z0-9/-])E[0-9]{1,3}([^A-Za-z0-9-]|$)' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null \
-             | grep -oE 'E[0-9]{1,3}' | sort -u); do
-  grep -rqE "^## $e " "$KB/experiments" || { bad "$e 被引用但 experiments/ 下没有它"
+if [[ ${#scanned_md_files[@]} -gt 0 ]]; then
+  mapfile -t experiment_refs < <(grep -ohE '(^|[^A-Za-z0-9/-])E[0-9]{1,3}([^A-Za-z0-9-]|$)' "${scanned_md_files[@]}" 2>/dev/null \
+                                   | grep -oE 'E[0-9]{1,3}' | sort -u)
+fi
+for e in "${experiment_refs[@]}"; do
+  grep -rqE "^## $e " "$KB/experiments" 2>/dev/null || { bad "$e 被引用但 experiments/ 下没有它"
     howto "要么在 experiments/ 下给它建正文（\`## $e <简称>\` 起头），" \
           "要么把引用它的那处改成真实存在的实验号——编号引用悬空，检索到的人会自己补一个。"
     missing=1; }
 done
-[[ $missing -eq 0 ]] && ok "实验号引用全部有定义"
+if [[ ${#experiment_refs[@]} -eq 0 ]]; then
+  bad "一个实验号引用都没扫到（$scanned_note）——这一段没有对象可判"
+  howto "确认门禁是在仓库根上跑的（第一个参数是仓库根）、$KB 目录在；" \
+        "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
+elif [[ $missing -eq 0 ]]; then
+  ok "实验号引用全部有定义：${#experiment_refs[@]} 个不同的实验号（$scanned_note）"
+fi
 
 echo
 echo "── 2. 决策号引用是否都有定义 ──"
 missing=0
-for d in $(grep -ohE '(^|[^A-Za-z0-9/-])D[0-9]{1,3}([^A-Za-z0-9-]|$)' $(find "$KB" -name "*.md") .claude/rules/*.md 2>/dev/null \
-             | grep -oE 'D[0-9]{1,3}' | sort -u); do
-  grep -rqE "^## $d " "$KB/decisions"  || { bad "$d 被引用但 decisions/ 下没有它"
+decision_refs=()
+if [[ ${#scanned_md_files[@]} -gt 0 ]]; then
+  mapfile -t decision_refs < <(grep -ohE '(^|[^A-Za-z0-9/-])D[0-9]{1,3}([^A-Za-z0-9-]|$)' "${scanned_md_files[@]}" 2>/dev/null \
+                                 | grep -oE 'D[0-9]{1,3}' | sort -u)
+fi
+for d in "${decision_refs[@]}"; do
+  grep -rqE "^## $d " "$KB/decisions" 2>/dev/null || { bad "$d 被引用但 decisions/ 下没有它"
     howto "要么在 decisions/ 下给它建正文（\`## $d <简称>\` 起头），" \
           "要么把引用它的那处改成真实存在的决策号。"
     missing=1; }
 done
-[[ $missing -eq 0 ]] && ok "决策号引用全部有定义"
+if [[ ${#decision_refs[@]} -eq 0 ]]; then
+  bad "一个决策号引用都没扫到（$scanned_note）——这一段没有对象可判"
+  howto "确认门禁是在仓库根上跑的、$KB 目录在；" \
+        "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
+elif [[ $missing -eq 0 ]]; then
+  ok "决策号引用全部有定义：${#decision_refs[@]} 个不同的决策号（$scanned_note）"
+fi
 
-echo
-echo "── 3. 已跑的实验，引用它的决策有没有跟着改 ──"
-# 判据：该实验状态行最后一次变动的提交，有没有同时改 decisions.md。
-# 没有 ⇒ 至少要人看一眼。这不是「一定错」，是「一定没人对过」。
-stale=0
-while read -r line; do
-  e="${line%% *}"
-  grep -q "已跑" <<<"$line" || continue
-  # 引用它的决策
-  refs=$(grep -nE "\b$e\b" "$KB/decisions.md" | head -3 | cut -d: -f1 | paste -sd, -)
-  [[ -n "$refs" ]] || continue
-  c=$(git log -1 --format=%h -S"## $e " -- "$KB/experiments" "$KB/experiments.md" 2>/dev/null)
-  [[ -n "$c" ]] || continue
-  # ⚠️ **不许写成 `git show ... | grep -q`**：本脚本开头是 `set -uo pipefail`，
-  # 而 `grep -q` 一命中就退出 ⇒ 前段还没写完就吃 SIGPIPE ⇒ 管道整体 141
-  # ⇒ 一次**命中**被读成「没动过 decisions.md」。
-  # 实测（2026-09-10）：整轮门禁（cargo 构建压着机器）里 E109 那一格判红一次，
-  # 同一份输入空载连跑 23 次全绿——两次的提交号与相邻那格逐字相同，只有这条管道的退出码翻了。
-  # 出路是 `.claude/singlefs-ai-sop/rules/command-safety.md` 自己写的那条：
-  # 先把输出落到变量，判完退出码再处理。
-  stat_out=$(git show --stat --format= "$c" 2>/dev/null || true)
-  if grep -q "decisions.md" <<<"$stat_out"; then
-    ok "$e 已跑，其状态变动的提交 $c 同时动过 decisions.md"
-  else
-    bad "$e 已跑，但把它改成已跑的提交 $c **没有动 decisions.md**（decisions.md 第 $refs 行引用了它）"
-    howto "把这个实验的结论写回它支撑的那条决策，与状态变动同批提交；" \
-          "结论没改变任何决策的话，在实验正文里写明这一点（零结论也是结论）。"
-    stale=1
-  fi
-done < <(cat "$KB"/experiments/*.md | grep -E "^## E[0-9]+ " | sed 's/^## //')
-[[ $stale -eq 0 ]] && ok "已跑实验与引用它的决策都在同一个提交里动过"
+# 已跑实验的清单，第 3 段用：experiments/ 下每份正文的 `## E<n> ` 标题行里带「已跑」的。
+ran_experiment_lines=()
+if [[ ${#experiment_body_files[@]} -gt 0 ]]; then
+  mapfile -t ran_experiment_lines < <(cat "${experiment_body_files[@]}" | grep -E "^## E[0-9]+ " | sed 's/^## //' | grep "已跑")
+fi
 
 echo
-echo "── 4. 已跑的实验有没有决策引用 ──"
+echo "── 3. 已跑的实验有没有决策引用 ──"
 # 一个实验跑完、产出了决策相关的结果，却没有任何决策引用它 ⇒ 那个结果没落进任何判断。
 # 这不是「引用格式问题」，是**结论悬空**。
 # 出路二：实验正文里一行「**备料**：」，点名它等着的决策或欠账（编号带简称），而且那个编号在 kb 里真有。
 # 此前出路里写着这一句而检查并不认它，照做了也还是红（2026-09-15 E152（按里程碑对比六家文件系统的文件性能） 撞上）；
 # 点名一个不存在的编号照样红，否则「备料」两个字就成了免检章。
-orphan=0
-while read -r line; do
+orphan=0; cited=0; reserved=0
+decision_text_files=("${decision_body_files[@]}")
+[[ -f "$KB/decisions.md" ]] && decision_text_files+=("$KB/decisions.md")
+for line in "${ran_experiment_lines[@]}"; do
   e="${line%% *}"
-  grep -q "已跑" <<<"$line" || continue
-  n=$(cat "$KB/decisions.md" "$KB"/decisions/*.md | grep -cE "(^|[^A-Za-z0-9/-])$e([^A-Za-z0-9-]|$)")
-  [[ "$n" -gt 0 ]] && continue
-  body="$(grep -lE "^## $e " "$KB"/experiments/*.md 2>/dev/null | head -1)"
+  n=0
+  if [[ ${#decision_text_files[@]} -gt 0 ]]; then
+    n=$(cat "${decision_text_files[@]}" | grep -cE "(^|[^A-Za-z0-9/-])$e([^A-Za-z0-9-]|$)")
+  fi
+  if [[ "$n" -gt 0 ]]; then cited=$((cited+1)); continue; fi
+  body="$(grep -lE "^## $e " "${experiment_body_files[@]}" 2>/dev/null | head -1)"
   reserve=""
   [[ -n "$body" ]] && reserve="$(grep -m1 -E '^\*\*备料\*\*：' "$body")"
   waiting_for=""
   for token in $(grep -oE '(D|C)[0-9]{1,3}（' <<<"$reserve" | tr -d '（' | sort -u); do
     case "$token" in
-      D*) grep -rqE "^## $token " "$KB/decisions" && waiting_for="$waiting_for $token" ;;
-      C*) grep -qE "^\| $token \|" "$KB/checks-owed.md" && waiting_for="$waiting_for $token" ;;
+      D*) grep -rqE "^## $token " "$KB/decisions" 2>/dev/null && waiting_for="$waiting_for $token" ;;
+      C*) grep -qE "^\| $token \|" "$KB/checks-owed.md" 2>/dev/null && waiting_for="$waiting_for $token" ;;
     esac
   done
   if [[ -n "$waiting_for" ]]; then
     ok "$e 不被任何决策引用，正文写明是备料，等$waiting_for"
+    reserved=$((reserved+1))
     continue
   fi
   bad "$e 已跑，但决策正文一次都没引用它——它的结论悬空了"
@@ -116,38 +136,85 @@ while read -r line; do
         "确实谁也不支撑的话，在实验正文里写一行「**备料**：……」，点名它等着的决策或欠账（编号带简称，例：D23（journal 的角色与格式）），" \
         "那个编号要在 decisions/ 下有正文、或在 checks-owed.md 里有一行。"
   orphan=1
-done < <(cat "$KB"/experiments/*.md | grep -E "^## E[0-9]+ " | sed 's/^## //')
-[[ $orphan -eq 0 ]] && ok "每个已跑实验都被决策引用，或正文写明了备料在等谁"
+done
+if [[ ${#ran_experiment_lines[@]} -eq 0 ]]; then
+  bad "一个已跑实验都没读到（experiments/ 下 ${#experiment_body_files[@]} 份正文）——这一段没有对象可判"
+  howto "确认 $KB/experiments/ 在、正文首行形如 \`## E<n> 简称 —— 已跑\`；" \
+        "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
+elif [[ $orphan -eq 0 ]]; then
+  ok "每个已跑实验都被决策引用，或正文写明了备料在等谁：共 ${#ran_experiment_lines[@]} 个，被决策引用 $cited 个、写明备料 $reserved 个"
+fi
 
 echo
-echo "── 5. 正文写死的条数 vs 实际条数 ──"
-inv_actual=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$KB/invariants.md")
-# 表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列
-inv_retired=$(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$KB/invariants.md")
-inv_live=$(( inv_actual - inv_retired ))
-# 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句（2026-09-18 立）：历史版本里也有「现共 N 条在用」，
-# 那是当时的数、不跟着改，按文件序取第一处会取到历史里的那一句（实测：2026-09-18 取到 2026-09-14 那条的 66）。
-inv_claim=$(awk '/<!-- invariant-count -->/{found=1; next} found && /现共 [0-9]+ 条在用/{print; exit}' "$KB/invariants.md" | grep -oE '[0-9]+' | head -1)
-if [[ -z "$inv_claim" ]]; then
-  inv_claim=$(grep -oE '现共 [0-9]+ 条在用' "$KB/invariants.md" | head -1 | grep -oE '[0-9]+')
-fi
-if [[ -n "$inv_claim" && "$inv_claim" != "$inv_live" ]]; then
-  bad "invariants.md 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
-  howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
-        "两个数对不上时，读的人不知道该信哪一个。"
+echo "── 4. 正文写死的条数 vs 实际条数 ──"
+if [[ ! -f "$KB/invariants.md" ]]; then
+  bad "找不到 $KB/invariants.md，不变量条数无从核对"
+  howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
 else
-  ok "不变量条数一致：在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
+  inv_actual=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$KB/invariants.md")
+  # 表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列
+  inv_retired=$(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$KB/invariants.md")
+  inv_live=$(( inv_actual - inv_retired ))
+  # 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句（2026-09-18 立）：历史版本里也有「现共 N 条在用」，
+  # 那是当时的数、不跟着改，按文件序取第一处会取到历史里的那一句（实测：2026-09-18 取到 2026-09-14 那条的 66）。
+  # 标记丢了、或下一行读不出那句，都判红：退回去取文件里第一处「N 条在用」，取到的正是那句历史。
+  marker_line_number=$(grep -n -m1 -F '<!-- invariant-count -->' "$KB/invariants.md" | cut -d: -f1)
+  if [[ "$inv_actual" -eq 0 ]]; then
+    bad "invariants.md 里一行 \`| I-<章>.<号> \` 表行都没数到——不变量表的写法变了，或这份文件是空的"
+    howto "表行形如 \`| I-1.1 | 简称 | 陈述 | … |\`；写法真改了的话，这里与 36 号的正则一起改。"
+  elif [[ -z "$marker_line_number" ]]; then
+    bad "invariants.md 里没有 <!-- invariant-count --> 标记——当前条数的登记位丢了，读不出正文声称几条（表里在用 $inv_live 条）"
+    howto "在「现共 N 条在用」那句的上一行补回 <!-- invariant-count -->；" \
+          "不许让检查退回去取文件里第一处「N 条在用」：历史版本里那几句记的是当时的数。"
+  else
+    claim_line=$(sed -n "$((marker_line_number + 1))p" "$KB/invariants.md")
+    inv_claim=$(grep -oE '现共 [0-9]+ 条在用' <<<"$claim_line" | grep -oE '[0-9]+' | head -1)
+    if [[ -z "$inv_claim" ]]; then
+      bad "invariants.md 第 $((marker_line_number + 1)) 行（<!-- invariant-count --> 的下一行）读不出「现共 N 条在用」：${claim_line:0:60}"
+      howto "标记的下一行就写那句「现共 $inv_live 条在用（…）」，中间不空行；" \
+            "读不出声明就是没核过，不许当成一致。"
+    elif [[ "$inv_claim" != "$inv_live" ]]; then
+      bad "invariants.md 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
+      howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
+            "两个数对不上时，读的人不知道该信哪一个。"
+    else
+      ok "不变量条数一致：正文声称 $inv_claim 条在用，表里在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
+    fi
+  fi
 fi
 # 三段各数各的：欠着的那张表、「### 已还清」那张表、「## 历史版本」里的条目。
 # ⚠️ **分界线是「### 已还清」，不是「## 历史版本」**：已还清那张表住在历史版本**之前**，
 # 按历史版本切会把还清的全算进欠账里——2026-09-16 现查它报「欠 326、已还清 0」，
 # 真数是 297 / 29。一条报错数的检查与没有这条检查，在门禁输出里长得一模一样。
-read -r chk_actual chk_done < <(awk '
-  /^### 已还清/{sec=1; next}
-  /^## 历史版本/{sec=2; next}
-  /^\| C[0-9]+ /{ if(sec==0) a++; else if(sec==1) d++ }
-  END{print a+0, d+0}' "$KB/checks-owed.md")
-ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
+# 数不出来（文件不在、awk 出错、两张表一行都没数到）判红：两个空白或两个 0 印在成功行里，看着就像数过了。
+if [[ ! -f "$KB/checks-owed.md" ]]; then
+  bad "找不到 $KB/checks-owed.md，欠账条数取不到"
+  howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
+else
+  # 开着与已还清的切法用共用读法 lib-owed.py（67、92、96 号同一份），不在这里再抄一份 awk
+  chk_counts=""
+  if chk_counts=$(python3 - "$(cd "$(dirname "$0")" && pwd)/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
+import importlib.util, sys
+spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
+lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
+table = lib_owed.read_owed_table(sys.argv[2])
+print(len(table.open_names), len(table.paid_names), int(table.paid_heading_found))
+PY_OWED
+  ); then :; else chk_counts=""; fi
+  read -r chk_actual chk_done chk_heading <<<"$chk_counts"
+  if [[ -z "${chk_actual:-}" || -z "${chk_done:-}" ]]; then
+    bad "共用读法 lib-owed.py 没数出 checks-owed.md 的条数（输出「$chk_counts」）"
+    howto "单独跑一遍上面那段 python 看它报什么错；数不出来就是没核过，不许当成数过了。"
+  elif [[ $((chk_actual + chk_done)) -eq 0 ]]; then
+    bad "checks-owed.md 里一行 \`| C<n> \` 都没数到（欠着 0、已还清 0）——欠账表的写法变了，或这份文件是空的"
+    howto "欠账行形如 \`| C12 | 简称 | … |\`；写法真改了的话，改 .claude/gate.d/lib-owed.py 的正则，67、92、96 号与这里一起跟上。"
+  elif [[ "${chk_heading:-0}" != 1 ]]; then
+    bad "checks-owed.md 里认不出「已还清」标题——开着的与已还清的分不开，$chk_actual 条全被算成欠着"
+    howto "已还清那张表上面要有一行「### 已还清」；标题改过名的话，改 .claude/gate.d/lib-owed.py 认标题的那条正则。"
+  else
+    ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
+  fi
+fi
 
 echo
 if [[ $fail -ne 0 ]]; then
diff --git a/.claude/gate.d/30-decision-history.sh b/.claude/gate.d/30-decision-history.sh
index 4159c84..7944f73 100755
--- a/.claude/gate.d/30-decision-history.sh
+++ b/.claude/gate.d/30-decision-history.sh
@@ -27,21 +27,41 @@ bad() { printf '  ✗ %s\n' "$*"; }
 ok()  { printf '  ✓ %s\n' "$*"; }
 howto() { printf '     → %s\n' "$*"; }
 
-if git diff --quiet HEAD -- "${KB[@]}" 2>/dev/null; then
-  ok "决策正文与 HEAD 无差异，本阶段无对象可判"
-  exit 0
+# 不在 git 仓里没有「改了什么」可比：退 77（本次无对象可判）。老写法把 git 的报错丢进 /dev/null，
+# 每一道 git 都失败、行数读成 0，于是「只改了 0 行，按小改动放行」报绿。
+if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
+  echo "  ! 不在 git 仓库里，本阶段无对象可判"
+  exit 77
 fi
+# 在仓里时每一道 git 都要成功：失败了这一阶段什么都没比，判红，不许读成「没改」。
+git_failed() {
+  bad "git $1 跑不起来（退出码 $2），决策正文改了多少、变更史加了几条都没数出来"
+  howto "按上面 git 的报错修好仓库状态再跑（常见是还没有任何提交、HEAD 不存在：先提交一次）；"
+  howto "  git 失败时这一阶段什么都没比，不是通过。"
+  exit 1
+}
+diff_rc=0
+git diff --quiet HEAD -- "${KB[@]}" || diff_rc=$?
+case "$diff_rc" in
+  0) echo "  ! 决策正文与 HEAD 无差异，本阶段无对象可判"; exit 77 ;;
+  1) ;;
+  *) git_failed "diff --quiet HEAD" "$diff_rc" ;;
+esac
 
 # 决策正文改了多少行（增 + 删，各文件相加）
-changed=$(git diff HEAD --numstat -- "${KB[@]}" | awk '{n+=$1+$2} END{print n+0}')
+numstat="$(git diff HEAD --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"
+changed=$(awk '{n+=$1+$2} END{print n+0}' <<<"$numstat")
 # 本次 diff 往变更史里加了几条日期标题
 added=0
 if ((${#HIST[@]})); then
-  added=$(git diff HEAD -- "${HIST[@]}" | grep -c '^+### 20[0-9][0-9]-' || true)
+  hist_diff="$(git diff HEAD -- "${HIST[@]}")" || git_failed "diff（变更史）" "$?"
+  added=$(grep -c '^+### 20[0-9][0-9]-' <<<"$hist_diff" || true)
+  untracked="$(git -c core.quotepath=false ls-files --others --exclude-standard -- "${HIST[@]}")" || git_failed "ls-files --others" "$?"
   while IFS= read -r untracked_history; do
+    [[ -n "$untracked_history" ]] || continue
     untracked_count=$(grep -c '^### 20[0-9][0-9]-' "$untracked_history" || true)
     added=$((added + untracked_count))
-  done < <(git ls-files --others --exclude-standard -- "${HIST[@]}")
+  done <<<"$untracked"
 fi
 
 if [[ "$added" -gt 0 ]]; then
diff --git a/.claude/gate.d/31-blocking-verdict.sh b/.claude/gate.d/31-blocking-verdict.sh
index 509054d..b849e61 100755
--- a/.claude/gate.d/31-blocking-verdict.sh
+++ b/.claude/gate.d/31-blocking-verdict.sh
@@ -55,15 +55,17 @@
 # 与 21 阶段同一个权威解析器。本阶段自己定位登记行，所以另加一道**条数比对**——
 # 两侧对不上就说明定位漏了或多了，判红而不是安静地少查几条。
 set -uo pipefail
-cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
+# cd 失败就退 2：老写法 `cd … 2>/dev/null || true` 在参数指错时留在调用方的 cwd 里，判的是调用方所在的那个仓，还报绿
+cd "${1:-$(dirname "$0")/../..}" || exit 2
 DEC=.claude/kb/decisions
 # ⚠️ 生成器按**脚本自身的位置**取，不按 cwd——判别力样本会把 cwd 换成一个只放着
 # 样本决策文件的临时目录，那里没有 `.claude/scripts/`。按 cwd 取会「找不到生成器 ⇒ 跳过」，
 # 于是红样本安静地绿掉，而这条检查看起来一切正常。生成器自己 glob 的是 cwd 下的 kb，正合样本所需。
 GEN="$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd)/.claude/scripts/gen-decision-items.py"
-[[ -f "$GEN" ]] || GEN=.claude/scripts/gen-decision-items.py
-[[ -d "$DEC" ]] || { echo "  ✓ 没有 $DEC，无对象可判"; exit 0; }
-[[ -f "$GEN" ]] || { echo "  ! 找不到 $GEN，本阶段跳过"; exit 77; }
+# 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
+[[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，本阶段无对象可判"; exit 77; }
+# 生成器随仓走，不在就是被删了或挪了——退 1 不退 77，与 21 号依赖同一个生成器时的退法一致（三方判决 gate-fix-forks-r1 的 T7）
+[[ -f "$GEN" ]] || { echo "  ✗ 找不到生成器 $GEN"; echo "     → 生成器随门禁住在同一个仓的 .claude/scripts/ 下：它丢了这一阶段什么都判不了，从 git 里找回它"; exit 1; }
 
 python3 - "$DEC" "$GEN" <<'PY'
 import re, sys, glob, subprocess, os
@@ -165,6 +167,9 @@ if bad:
         print("               两把量的不是一个集合。")
     sys.exit(1)
 
+if not seen:
+    print("  ! 没有一条未定项，本阶段无对象可判")
+    sys.exit(77)
 print(f"  ✓ {len(seen)} 条未定项两把尺都判过："
       + "、".join(f"{name}" for name, _, _ in RULERS))
 PY
diff --git a/.claude/gate.d/40-results-cited.sh b/.claude/gate.d/40-results-cited.sh
index 1c386aa..c5acbeb 100755
--- a/.claude/gate.d/40-results-cited.sh
+++ b/.claude/gate.d/40-results-cited.sh
@@ -7,17 +7,25 @@
 # ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 有一次把 E20 从 2 档扩到 6 档、
 # 跑了三轮、原始输出 22 KB 落了盘，而 experiments.md 里那一节还是两点对比，
 # 决策侧一次都没引——报告只存在于对话里。
+#
+#   bash .claude/gate.d/40-results-cited.sh [项目根]
 set -uo pipefail
+ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
+cd "$ROOT" || exit 2
 # ⚠️ 2026-08-29 起实验正文拆到 `kb/experiments/` 下，索引只剩导航表。
 # 两侧都要扫：产物可能被任一实验正文点名。
 EXP_DIR=.claude/kb/experiments
-EXP_ALL="$(mktemp)"; trap 'rm -f "$EXP_ALL"' EXIT
+EXP_ALL="$(mktemp)"; UNCITED="$(mktemp)"; NAMED="$(mktemp)"
+# 中间文件全用 mktemp 并交给 trap：固定路径加 $$ 不会撞，但判红那一支 exit 之前只删了一份，另一份每红一次就在 /tmp 留一份
+trap 'rm -f "$EXP_ALL" "$UNCITED" "$NAMED"' EXIT
 cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
 EXP="$EXP_ALL"
 RES=research/results
 [[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，本阶段跳过"; exit 77; }
-[[ -d "$RES" ]] || { echo "  ✓ 没有 $RES 目录，无对象可判"; exit 0; }
+[[ -d "$RES" ]] || { echo "  ! 没有 $RES 目录，本阶段无对象可判"; exit 77; }
 
+# 三道都判完再退出，一次把问题说全；判的工具自己没跑成（awk 出错）才当场退出。
+failed=0
 missing=()
 while IFS= read -r f; do
   b="$(basename "$f")"
@@ -31,9 +39,10 @@ if ((${#missing[@]})); then
   for m in "${missing[@]}"; do echo "     $RES/$m"; done
   echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
   echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
-  exit 1
+  failed=1
+else
+  echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（$(find "$RES" -maxdepth 1 -name '*.out'|wc -l) 个文件）"
 fi
-echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（$(find "$RES" -maxdepth 1 -name '*.out'|wc -l) 个文件）"
 
 # ── 反方向：已跑的实验必须要么点名产物，要么显式说明产物没留 ──
 # 只查一个方向会漏掉「实验写了结论、但产物从没存在过」——那种情况下
@@ -55,31 +64,42 @@ awk '
   /原始输出未留存|输出未留存/{ excused=1 }
   END{ if (cur != "" && done && !cited && !excused) print cur
        for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr" }
-' "$EXP" > /tmp/.gate-uncited-$$ 2>/tmp/.gate-named-$$
-if [[ -s /tmp/.gate-uncited-$$ ]]; then
+' "$EXP" > "$UNCITED" 2>"$NAMED" || {
+  echo "  ✗ 判「已跑实验有没有点名产物」的 awk 没跑成（退出码 $?），这一格没判："
+  sed 's/^/     /' "$NAMED"   # gate-lint:detail
+  echo "     → 怎么办：看上面 awk 的报错修这一段；它没跑完时未点名的实验一个都不会被列出来，不许当成通过。"
+  exit 1
+}
+if [[ -s "$UNCITED" ]]; then
   echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
-  sed 's/^/     /' /tmp/.gate-uncited-$$
+  sed 's/^/     /' "$UNCITED"
   echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
   echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
-  rm -f /tmp/.gate-uncited-$$
-  exit 1
+  failed=1
 fi
-rm -f /tmp/.gate-uncited-$$
+rm -f "$UNCITED"
 
 # ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
 # 射程与门禁 88 号同一条（用户 2026-09-21 定）：只判**这次改动新增或改写的**点名行。
 # 早先写下的点名是历史，仅作参考；而别的会话正在跑、产物还没提交的实验，它的页也不该由这一次提交来判。
-# 拿不到 diff 基准就判全部（保守）。
+# 改动范围取共用脚本 research/scripts/changed-paths.sh（基准 gate_diff_base gate，新增行 gate_added_lines：
+# 未跟踪的实验页整份算新增——新写的页在 git add 之前，里面点名的产物也要判）。不在 git 仓里、或取不到改动范围，就判全部（保守）。
 # 少了这一道，上一道就退化成「正文里写个像文件名的串」：归档之后引用只剩文件名，
 # 而一个从没存在过的文件名与一份归档进历史的产物，在正文里长得一模一样。
-BASE="${GATE_BASE:-}"
-if [[ -z "$BASE" ]]; then
-  BASE="$(git rev-parse --verify --quiet refs/sop/gate-ok || git rev-parse --verify --quiet refs/singlefs/gate-ok || git rev-parse --verify --quiet "@{upstream}" || true)"
-fi
-touched=""
-if [[ -n "$BASE" ]]; then
-  touched="$(git diff --unified=0 "$BASE" -- .claude/kb/experiments.md "$EXP_DIR" 2>/dev/null \
-             | grep "^+" | grep -v "^+++" | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
+LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
+# shellcheck source=../../research/scripts/changed-paths.sh
+source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
+BASE="" touched="" scope_note="全部的行（不在 git 仓里，判全部）"
+if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
+  BASE="$(gate_diff_base gate)"
+  added_lines="$(mktemp)"
+  if gate_added_lines "$BASE" .claude/kb/experiments.md "$EXP_DIR" > "$added_lines"; then
+    touched="$(cut -f2- "$added_lines" | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
+    scope_note="这次改动（基准 $BASE）新增或改写的行"
+  else
+    BASE="" scope_note="全部的行（取不到基准 $(gate_diff_base gate) 起的新增行，判全部）"
+  fi
+  rm -f "$added_lines"
 fi
 missing=0; checked=0; skipped=0
 while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
@@ -94,19 +114,22 @@ while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都
   fi
   checked=$((checked+1))
   [[ -f "$RES/$name" ]] && continue
-  if [[ -n "$(git log --all --diff-filter=D --format=%h --name-only -- "*$name" 2>/dev/null | head -1)" ]]; then
+  if [[ -n "$(git -c core.quotepath=false log --all --diff-filter=D --format=%h --name-only -- "*$name" 2>/dev/null | head -1)" ]]; then
     continue
   fi
   echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
   missing=$((missing+1))
-done < <(sort -u /tmp/.gate-named-$$ 2>/dev/null)
-rm -f /tmp/.gate-named-$$
+done < <(sort -u "$NAMED" 2>/dev/null)
+rm -f "$NAMED"
 if [[ $missing -gt 0 ]]; then
   echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
   echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
   echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
   echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
-  exit 1
+  failed=1
+else
+  echo "  ✓ $scope_note点名的 $checked 份产物在树里或版本库历史里都找得到"
+  echo "     没判的 $skipped 份：早先就写在正文里的点名（历史参考），以及别的会话正在跑、产物还没提交的实验"
 fi
-echo "  ✓ 已跑的实验都点了名或写明了产物未留存；这次改动点名的 $checked 份产物在树里或版本库历史里都找得到"
-echo "     没判的 $skipped 份：早先就写在正文里的点名（历史参考），以及别的会话正在跑、产物还没提交的实验"
+((failed)) && exit 1
+echo "  ✓ 已跑的实验都点了名或写明了产物未留存"
diff --git a/.claude/gate.d/52-segment-registry.sh b/.claude/gate.d/52-segment-registry.sh
index eb8becc..444822e 100755
--- a/.claude/gate.d/52-segment-registry.sh
+++ b/.claude/gate.d/52-segment-registry.sh
@@ -13,17 +13,23 @@
 # 该脚本自己的 --selftest（改坏拷贝里的一个段序列数字，确认判红；未改动的拷贝确认判绿）
 # 由 47 号阶段（三方论证 research 脚本的自证）复跑，这里不重复跑一遍。
 #
-# ⚠️ 本阶段没有 .claude/gate.d/fixtures/ 判别力样本：它转发的检查依赖
-#   .claude/kb/layout/01-first-txn.md 与 research/results/ 下的真实产物，这两样在 89 号阶段
-#   （项目本地阶段自检）的隔离沙箱里不存在——与 87-replay.sh、47-research-script-selftests.sh
-#   同理，这两个阶段同样转发外部 research/scripts/ 下的真实脚本，也同样没有样本。
-#   89 号阶段会把本阶段显式列成「未自检」，这是如实反映、不是漏做
-#   （rules/show-me-test.md「门禁不许假装通过」）。
+# ⚠️ 脚本按**本阶段自己的位置**取仓里的那一份，不按 cwd 取；`--root` 指被判的仓。判别力样本会把 cwd 换成样本目录，
+#   按 cwd 取就成了样本里那份拷贝：仓里的脚本退化了（例如 main() 不再传退出码），样本照样判对，这一道与 47 号的
+#   --selftest 一起放过（三方判决 research/prompts/gate-fix-forks-r1-main-verification.md 的 T6）。同一个坑见 31 号头部。
+# 样本：fixtures/52-segment-registry.sh 只放脚本读的三样合成输入——layout 表的「八、」一节（一行表、一句 path 声明、
+#   一句整条流）、replay.sh 里 E142 那一行、产物里两行 name=segments。red 把表里的段序列写成与产物不一致，判红；green 一致，判绿。
+#   钉活代码的那几句与真产物的解析由脚本自己的 --selftest（47 号跑）拿真文件测，样本不再测一遍。
 #
 #   bash .claude/gate.d/52-segment-registry.sh [仓根]
 set -uo pipefail
 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
 cd "$ROOT" 2>/dev/null || exit 2
-[[ -f research/scripts/check-segment-registry.py ]] || { echo "  ! 找不到 research/scripts/check-segment-registry.py，本阶段跳过"; exit 77; }
+SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/check-segment-registry.py"
+# 脚本随仓走，不在就是被删了或挪了——退 1 不退 77，与 57、70、73、77 号同一条（三方判决 gate-fix-forks-r1 的 T7）
+[[ -f "$SCRIPT" ]] || {
+  echo "  ✗ 找不到 $SCRIPT"
+  echo "     → 怎么办：它随仓走（research/scripts/ 下），不在就是被删了或挪了：从 git 找回来，挪了就改这一行的路径。"
+  exit 1
+}
 
-python3 research/scripts/check-segment-registry.py --root "$ROOT" || exit 1
+python3 "$SCRIPT" --root "$ROOT" || exit 1
diff --git a/.claude/gate.d/61-settled-same-file.sh b/.claude/gate.d/61-settled-same-file.sh
index 31f331b..eda7f9e 100755
--- a/.claude/gate.d/61-settled-same-file.sh
+++ b/.claude/gate.d/61-settled-same-file.sh
@@ -1,5 +1,5 @@
 #!/usr/bin/env bash
-# gate-stage: 定了新东西之后有没有回头看同文件的未定项
+# gate-stage: 状态一致性：定了新东西之后有没有回头看同文件的未定项
 #
 # 还 checks-owed.md C36（未定项检查的三个盲区）的前两条。
 #
@@ -26,19 +26,24 @@
 set -uo pipefail
 DEC=.claude/kb/decisions
 IDX=.claude/kb/decisions.md
-[[ -d "$DEC" ]] || { echo "  ✓ 没有 $DEC，无对象可判"; exit 0; }
+# 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
+[[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，本阶段无对象可判"; exit 77; }
 git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，本阶段跳过"; exit 77; }
 
-# diff 基准：与共享门禁的 Show me test 同一套口径
-BASE="${GATE_BASE:-}"
-if [[ -z "$BASE" ]]; then
-  for def in master main; do
-    if git rev-parse --verify -q "$def" >/dev/null; then
-      BASE="$(git merge-base HEAD "$def" 2>/dev/null)" && break
-    fi
-  done
-fi
-[[ -n "$BASE" ]] || BASE=HEAD
+# 改动范围取共用脚本 research/scripts/changed-paths.sh（门禁 64 号判阶段里不另算一份）：
+# 基准是 gate_diff_base gate，名单带未跟踪文件——新写的决策文件在 git add 之前也算这次改动。
+LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
+# shellcheck source=../../research/scripts/changed-paths.sh
+source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
+BASE="$(gate_diff_base gate)"
+
+# 本次改动碰过的决策文件。名单先落到变量、判过退出码再读：git 失败时名单是空的，会被读成「本次没有新增已定小节」。
+all_changed="$(gate_changed_paths "$BASE" untracked)" || {
+  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $BASE），这次改了哪些决策文件没取到"
+  echo "     → 按上面 git 的报错修好仓库状态（基准 $BASE 要存在；仓里还没有提交就先提交一次）再跑；git 失败时这一阶段什么都没比，不是通过。"
+  exit 1
+}
+changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"
 
 # 本次 diff 里**新增**了「已定」小节标题的决策文件
 settled_files=()
@@ -51,26 +56,41 @@ while IFS= read -r f; do
   # 合成仓双向验的时候当场红——**这就是「新增的检查必须先证明它会红」拦下来的那一次**。
   # 同 10-kb-rot.sh 那条：pipefail + `grep -q` 提前退出 ⇒ 前段 SIGPIPE ⇒ 命中被读成没命中。
   # `git diff` 的输出可以很大，这里比那条更容易撞上。
-  diff_out=$(git diff "$BASE" -- "$f" || true)
-  if grep -qE '^\+#{2,4} .*—— 已定|^\+#{2,4} 已定[（(]' <<<"$diff_out"; then
+  # 新增行按共用脚本取（未跟踪的文件整份算新增）；取不到就判红，不当「没有新增」
+  added_out="$(gate_added_lines "$BASE" "$f")" || {
+    echo "  ✗ 取不到 $f 这次新增了哪些行（基准 $BASE）"
+    echo "     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一阶段什么都没比，不是通过。"
+    exit 1
+  }
+  added_text="$(cut -f2- <<<"$added_out")"
+  if grep -qE '^#{2,4} .*—— 已定|^#{2,4} 已定[（(]' <<<"$added_text"; then
     settled_files+=("$f")
   fi
-done < <(git -c core.quotepath=false diff --name-only "$BASE" -- "$DEC" 2>/dev/null)
+done <<<"$changed_names"
 
 if ((${#settled_files[@]} == 0)); then
-  echo "  ✓ 本次 diff 没有新增「已定」小节，本阶段无对象可判"
-  exit 0
+  # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
+  echo "  ! 本次 diff 没有新增「已定」小节，本阶段无对象可判"
+  exit 77
 fi
 
 flagged=0
+open_checked=0      # 查过的未定项条数（新增了已定小节的那几份里）
+pending_checked=0   # 查过的索引页「待议」节数
 
 # ── ① 同文件里还开着、而本次一个都没碰的未定项 ──────────────────
 for f in "${settled_files[@]}"; do
   # 本次 diff 在这个文件里碰过的行号（新文件侧）
-  touched="$(git diff -U0 "$BASE" -- "$f" \
-            | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}')"
+  # 未跟踪的新文件整份都是这次写的：每一行都算碰过
+  if git -c core.quotepath=false ls-files --error-unmatch -- "$f" >/dev/null 2>&1; then
+    touched="$(git diff -U0 "$BASE" -- "$f" \
+              | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}')"
+  else
+    touched="$(seq 1 "$(wc -l < "$f")")"
+  fi
   while IFS=: read -r ln text; do
     [[ -n "$ln" ]] || continue
+    open_checked=$((open_checked + 1))
     end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^### /||/^## /){print NR-1; exit}' "$f")
     [[ -n "$end" ]] || end=$((ln+20))
     # 这个条目块里有没有任何一行在本次 diff 里被碰过
@@ -96,11 +116,11 @@ done
 if [[ -f "$IDX" ]]; then
   idx_touched=0
   # 同上：`grep -q .` 在第一行就退出，前段 SIGPIPE 会让「动过」被读成「没动过」。
-  idx_names=$(git -c core.quotepath=false diff --name-only "$BASE" -- "$IDX" 2>/dev/null || true)
-  [[ -n "$idx_names" ]] && idx_touched=1
+  grep -qxF "$IDX" <<<"$all_changed" && idx_touched=1
   while IFS=: read -r ln text; do
     [[ -n "$ln" ]] || continue
     grep -qE '已回收|已收摊|已并入' <<<"$text" && continue
+    pending_checked=$((pending_checked + 1))
     (( idx_touched )) && continue
     echo "  ✗ $(basename "$IDX"):$ln 本次有决策定案，而这一节「待议」一个字都没动"
     echo "     ⇒ 复核它是不是被这次定案实质回答/否决了。原文：${text:0:60}"
@@ -114,4 +134,4 @@ if ((flagged)); then
   echo "               后一种做法本身就是这条检查要的东西——它要的是一次回头看，不是一次沉默。"
   exit 1
 fi
-echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过"
+echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过（新增已定小节的决策 ${#settled_files[@]} 份，查了 $open_checked 条未定项、$pending_checked 节待议）"
diff --git a/.claude/gate.d/66-abandoned-rounds.sh b/.claude/gate.d/66-abandoned-rounds.sh
index a1ed5bb..9c56b64 100755
--- a/.claude/gate.d/66-abandoned-rounds.sh
+++ b/.claude/gate.d/66-abandoned-rounds.sh
@@ -61,7 +61,7 @@ legacy_leg_form = re.compile(r"^(?P<prefix>[^_].*?)(?:(?:-(?:forward|reverse|att
 # 而登记表本身是归档规则明令保留的（门禁 66 号的输入），两边对不上不是登记错了，是这道检查只看了树。
 archived_names = set()
 try:
-    log = subprocess.run(["git", "log", "--all", "--diff-filter=D", "--format=", "--name-only",
+    log = subprocess.run(["git", "-c", "core.quotepath=false", "log", "--all", "--diff-filter=D", "--format=", "--name-only",
                           "--", prompts_dir], capture_output=True, text=True, check=True).stdout
     for entry in log.split("\n"):
         entry = entry.strip()
diff --git a/.claude/gate.d/80-absolute-assertions.sh b/.claude/gate.d/80-absolute-assertions.sh
index 252ca87..ee1ce1f 100755
--- a/.claude/gate.d/80-absolute-assertions.sh
+++ b/.claude/gate.d/80-absolute-assertions.sh
@@ -1,5 +1,5 @@
 #!/usr/bin/env bash
-# gate-stage: 每个实验都要有钉绝对值的断言
+# gate-stage: 每个实验二进制都要有钉绝对值的断言（research/e7-index-bench/src/bin 下全部，别处 src/bin 下以 e<数字>_ 开头的；其余成功行逐个列名）
 #
 # 还 test-discipline.md 的这一条：
 # 「只让多条臂互相比，测不出『所有臂一起错』……每一条互比断言旁边，
@@ -14,17 +14,41 @@
 # 的跨语言比值——两个都是承重结论，而它们量出来的那个数没有任何东西钉。
 #
 # 判别力已证（2026-08-29）：把 e9 的四条绝对值断言注释掉 ⇒ 本阶段判红。
+#
+# 射程：research/e7-index-bench/src/bin 下的每一份，加上别处 `crates/*/src/bin`、`research/*/src/bin` 下文件名以 `e<数字>_` 开头的
+# （实验编号的写法，`.claude/abbreviations` 登记的 `e<数字>`）——按「是不是实验」认，不按住在哪个目录认：
+# 住在 crates/ 下的实验与 research/ 下的同规矩。别处不以 `e<数字>_` 开头的是装置工具（例：拿设备日志逐项比 ground truth 的），
+# 不判，成功行逐个列名，清单现算；research/prompts/ 下腿的模型是冻结证据，不算实验二进制，不列。
+# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T4。
+# 样本：fixtures/80-absolute-assertions.sh/red 放一份只比相对值的（e7 目录）与一份住在 crates/ 下、以 e<数字>_ 开头、只比相对值的，
+# 两份都要点名判红；green 的两份都钉了绝对值、判绿，另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。
+#
+#   bash .claude/gate.d/80-absolute-assertions.sh [项目根]
 set -uo pipefail
+ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
+cd "$ROOT" 2>/dev/null || exit 2
 BINS=research/e7-index-bench/src/bin
-[[ -d "$BINS" ]] || { echo "  ✓ 没有 $BINS，无对象可判"; exit 0; }
+[[ -d "$BINS" ]] || { echo "  ! 没有 $BINS，本阶段无对象可判"; exit 77; }
 
-bad=0; n=0
+judged=() uncovered=()
 for f in "$BINS"/*.rs; do
+  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
+done
+in_bins=${#judged[@]}
+for directory in crates/*/src/bin research/*/src/bin; do
+  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
+  for f in "$directory"/*.rs; do
+    [[ -f "$f" ]] || continue
+    if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
+  done
+done
+bad=0; n=0
+for f in "${judged[@]}"; do
   n=$((n+1))
   # 绝对值断言：与数字字面量比死。两种形态都认。
   c=$(grep -cE 'assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[<>=]=? *-?[0-9][0-9_]*(\.[0-9]+)?[,)]' "$f")
   if (( c == 0 )); then
-    echo "  ✗ $(basename "$f") 一条绝对值断言都没有"
+    echo "  ✗ $f 一条绝对值断言都没有"
     bad=$((bad+1))
   fi
 done
@@ -35,4 +59,7 @@ if ((bad)); then
   echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
   exit 1
 fi
-echo "  ✓ $n 个实验各自至少有一条绝对值断言"
+((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，本阶段无对象可判"; exit 77; }
+echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
+echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
+for f in "${uncovered[@]}"; do echo "      $f"; done
diff --git a/.claude/gate.d/88-quoted-result-lines.sh b/.claude/gate.d/88-quoted-result-lines.sh
index 763a509..17846d3 100755
--- a/.claude/gate.d/88-quoted-result-lines.sh
+++ b/.claude/gate.d/88-quoted-result-lines.sh
@@ -1,5 +1,5 @@
 #!/usr/bin/env bash
-# gate-stage: kb 正文里整行抄的产物行，产物里逐字找得到
+# gate-stage: kb 与 research 正文里整行抄的产物行（只认去掉首尾空白后以 `E7RESULT ` 开头的行），产物里逐字找得到
 #
 # 判据：kb 正文（「## 历史版本」之前；*-history.md 与 decisions-history/ 不算）里去掉首尾空白后
 # 以 `E7RESULT ` 开头的行，必须在 research/results/*.out 的某一行里逐字出现。
@@ -25,11 +25,22 @@ ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
 cd "$ROOT" 2>/dev/null || exit 2
 [[ -d .claude/kb ]] || { echo "  ! 找不到 .claude/kb，本阶段无对象可判"; exit 77; }
 
-BASE="${GATE_BASE:-}"
-if [[ -z "$BASE" ]]; then
-  BASE="$(git rev-parse --verify --quiet refs/sop/gate-ok || git rev-parse --verify --quiet refs/singlefs/gate-ok || git rev-parse --verify --quiet '@{upstream}' || true)"
+# 改动范围取共用脚本 research/scripts/changed-paths.sh（门禁 64 号判阶段里不另算一份）：基准 gate_diff_base gate，
+# 新增行 gate_added_lines——未跟踪的文件整份算新增，新写的 kb 页在 git add 之前抄的产物行也要判。
+LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
+# shellcheck source=../../research/scripts/changed-paths.sh
+source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
+ADDED_LINES="$(mktemp)"
+trap 'rm -f "$ADDED_LINES"' EXIT
+SCOPE_BASE=""
+if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
+  base="$(gate_diff_base gate)"
+  # research/ 只取 .md：未跟踪的产物可能很大，整份读一遍白费
+  if gate_added_lines "$base" .claude/kb ':(glob)research/**/*.md' > "$ADDED_LINES"; then
+    SCOPE_BASE="$base"
+  fi
 fi
-export GATE_BASE_RESOLVED="$BASE"
+export SCOPE_BASE ADDED_LINES
 
 python3 - <<'PY'
 import glob, os, subprocess, sys
@@ -51,28 +62,19 @@ for path in kb:
         stripped = line.strip()
         if stripped.startswith('E7RESULT '):
             quoted.append((path, number, stripped))
-# 只判这次改动新增或改写的行：拿 diff 基准算出每份 kb 文件新增的 E7RESULT 行，
-# 基准取不到就退回全量判（保守，宁可多判不可漏判）。
-base = os.environ.get('GATE_BASE_RESOLVED', '').strip()
+# 只判这次改动新增或改写的行：共用脚本交来的新增行（路径<TAB>行文）里挑 E7RESULT 行，
+# 不在 git 仓里或取不到新增行（SCOPE_BASE 为空）就退回全量判（保守，宁可多判不可漏判）。
 scope = '全量'
-if base:
+if os.environ.get('SCOPE_BASE', '').strip():
     added = set()
-    for path in kb:
-        try:
-            diff = subprocess.run(['git', 'diff', '--unified=0', base, '--', path],
-                                  capture_output=True, text=True, check=True).stdout
-        except subprocess.CalledProcessError:
-            added = None
-            break
-        for line in diff.split('\n'):
-            if line.startswith('+') and not line.startswith('+++'):
-                stripped = line[1:].strip()
-                if stripped.startswith('E7RESULT '):
-                    added.add((path, stripped))
-    if added is not None:
-        quoted = [(path, number, stripped) for path, number, stripped in quoted
-                  if (path, stripped) in added]
-        scope = '这次改动新增或改写的'
+    with open(os.environ['ADDED_LINES'], encoding='utf-8', errors='replace') as added_handle:
+        for record in added_handle.read().split('\n'):
+            added_path, separator, added_text = record.partition('\t')
+            if separator and added_text.strip().startswith('E7RESULT '):
+                added.add((added_path, added_text.strip()))
+    quoted = [(path, number, stripped) for path, number, stripped in quoted
+              if (path, stripped) in added]
+    scope = '这次改动新增或改写的'
 
 if not quoted:
     print('  ! %s kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判' % scope)
@@ -94,7 +96,7 @@ archived_count = 0
 def archived_product_lines():
     global archived_count
     lines = set()
-    log = subprocess.run(['git', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only',
+    log = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only',
                           '--', 'research/results'], capture_output=True, text=True).stdout
     commit = None
     for entry in log.split('\n'):
diff --git a/.claude/gate.d/92-layout-checker-sync.sh b/.claude/gate.d/92-layout-checker-sync.sh
index eac8aa4..ae1a4ea 100755
--- a/.claude/gate.d/92-layout-checker-sync.sh
+++ b/.claude/gate.d/92-layout-checker-sync.sh
@@ -19,29 +19,63 @@
 # `format-spec/<组件>.toml` 求，而那份 toml 今天一个都不存在（C119（冻结组件没有 spec 文件））。
 # 在它出现之前，这一道拿「常量名 → 值的集合」当变更探测器——哈希本来就只是变更探测器，
 # 而一次 diff 也是。spec toml 有了就把抽取源换过去，四条判据不变。
-# 常量从两处抽：`.rs` 里的 `pub const 名字: 类型 = 值;`，`.md` 里的 `<!-- format-const: 名字 = 值 … -->` 标记。
+# 常量从两处抽：`.rs` 里顶格的 `pub const 名字: 类型 = 值;`，`.md` 里的 `<!-- format-const: 名字 = 值 … -->` 标记。
+# 两种都按 `lib-format-const.py` 读（27、39 号用的是同一份）：标记按文法读不出来的、同一份格式定义里
+# 同一个名字登记了不止一次的，这一道判红——前者在变更探测里看不见，后者两个值里改了哪个说不清。
+# `.rs` 的值按空白归一后的原文比（value_reading="normalized_text"），不像 27 号那样要求整数字面量：
+# 格式常量模块里有 `DATA_UNIT_HEADER_BYTES + …` 这类算出来的常量，这一道只问它变没变。
 #
-# 改动范围与 56 号同一条：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，
-# 都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算。
+# 改动范围（基准与路径集合）都取共用脚本 research/scripts/changed-paths.sh：gate_diff_base gate 与
+# gate_changed_paths 带未跟踪文件，不在这里另算一份（门禁 64 号判）。git 调用一律带 `-c core.quotepath=false`：
+# 默认的 quoting 把中文路径打成八进制引号串，与布局清单里的路径逐字比对不上，checker 明明跟了也判「没碰」。
 #
-# 判别力：fixtures/92-layout-checker-sync.sh/red 是一个改了格式常量、没碰 checker 的小仓，必须判红；
-# green 是同一处改动加上 checker 跟着改，必须判绿。
+# 滞后登记表指的欠账号开没开着，按 `lib-owed.py` 读 checks-owed.md（67、96 号用的是同一份）：
+# 「### 已还清」整行标题之前的是开着的；认不出那个标题就判红，不对着一张认不出的表判。
+#
+# 滞后表、标记与第 ④ 条三样都判完再退出，一次把问题说全。
+#
+# 判别力：fixtures/92-layout-checker-sync.sh/red 是一个改了格式常量、没碰 checker 的小仓，
+# 另带一份多写了键又重复登记的格式定义、一张挂在认不出的欠账表上的滞后表，必须判红；
+# green 是同一处改动加上 checker 跟着改（checker 路径是中文文件名），另带一张滞后表，挂的欠账号
+# 排在一行正文提到「### 已还清」的开着的账后面，必须判绿。
 #
 #   bash .claude/gate.d/92-layout-checker-sync.sh [项目根]
 set -uo pipefail
 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
+LIBRARY_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
 cd "$ROOT" 2>/dev/null || exit 2
 git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
-base="HEAD"
-if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
-  base="$GATE_BASE"
-elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
-  base="$(git merge-base HEAD '@{upstream}')"
-fi
-python3 - "$base" <<'PY'
-import os, re, subprocess, sys
-
-base = sys.argv[1]
+# 基准取法与 56、68、69、75、97 号同一份：research/scripts/changed-paths.sh 的 gate 取法
+LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
+# shellcheck source=../../research/scripts/changed-paths.sh
+source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
+base="$(gate_diff_base gate)"
+CHANGED_PATHS="$(mktemp)"
+trap 'rm -f "$CHANGED_PATHS"' EXIT
+# 路径集合先落到文件、判过退出码再交给 python：git 失败时集合静默为空，会被读成「这次什么都没碰」
+gate_changed_paths "$base" untracked > "$CHANGED_PATHS" || {
+  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base，gate_changed_paths 退出码 $?）"
+  echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到改动范围时第 ④ 条什么都没比，不是通过。"
+  exit 1
+}
+python3 - "$base" "$LIBRARY_DIRECTORY" "$CHANGED_PATHS" <<'PY'
+import importlib.util, os, re, subprocess, sys
+
+base, library_directory, changed_paths_file = sys.argv[1], sys.argv[2], sys.argv[3]
+
+def load_library(module_name, file_name):
+    library_spec = importlib.util.spec_from_file_location(module_name, os.path.join(library_directory, file_name))
+    library = importlib.util.module_from_spec(library_spec)
+    library_spec.loader.exec_module(library)
+    return library
+
+format_const = load_library("format_const", "lib-format-const.py")
+owed_library = load_library("owed", "lib-owed.py")
+# 27 号读 .rs 的值要整数字面量（integer_literal）；这一道沿用只做空白归一的旧口径，
+# 两道该不该统一成一种还没定，统一时改这一个名字。
+RUST_VALUE_READING = "normalized_text"
+GIT = ["git", "-c", "core.quotepath=false"]
+
 manifest_path = ".claude/gate.d/layouts.tsv"
 lag_path = ".claude/gate.d/layouts-checker-lag.tsv"
 registry_path = ".claude/kb/decisions/15-格式冻结政策.md"
@@ -125,28 +159,32 @@ if unregistered:
     sys.exit(1)
 
 # ④ 格式常量集合变了，checker 判定路径要跟
-changed = set()
-for args in (["diff", "--name-only", base, "--"],
-             ["diff", "--name-only", "--cached", "--"],
-             ["ls-files", "--others", "--exclude-standard", "--"]):
-    result = subprocess.run(["git", *args], capture_output=True, text=True)
-    changed.update(name for name in result.stdout.split("\n") if name.strip())
+with open(changed_paths_file, encoding="utf-8") as changed_paths_handle:
+    changed = {name for name in changed_paths_handle.read().split("\n") if name.strip()}
 
-RUST_CONST = re.compile(r"^pub const\s+(\w+)\s*:[^=]+=\s*(.+?);", re.M | re.S)
-MARK_CONST = re.compile(r"<!--\s*format-const:\s*(\w+)\s*=\s*(-?\d+)")
+marker_problems = []
 
-def constants_in(text, path):
+def constants_in(text, path, record_problems):
+    """常量名 → 值的原文；record_problems 为真时把读不出来的标记与重复登记记进 marker_problems。"""
     found = {}
     if path.endswith(".rs"):
-        for name, value in RUST_CONST.findall(text):
-            found[name] = re.sub(r"\s+", " ", value).strip()
-    else:
-        for name, value in MARK_CONST.findall(text):
-            found[name] = value
+        for declaration in format_const.read_rust_consts(text, value_reading=RUST_VALUE_READING,
+                                                         only_top_level_public=True):
+            found[declaration.name] = declaration.value
+        return found
+    parsed = format_const.parse_marks(text)
+    for mark in parsed.marks:
+        found[mark.name] = mark.value_text
+    if record_problems:
+        for unparsable in parsed.unparsable:
+            marker_problems.append(f"{path}:{unparsable.line_number}  标记按文法读不出来：「{unparsable.excerpt}」")
+        for duplicate in parsed.duplicates:
+            marker_problems.append(f"{path}  {duplicate.name} 在这一份里登记了 {len(duplicate.line_numbers)} 次"
+                                   f"（第 {'、'.join(str(line_number) for line_number in duplicate.line_numbers)} 行）")
     return found
 
 def baseline_text(path):
-    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
+    result = subprocess.run([*GIT, "show", f"{base}:{path}"], capture_output=True, text=True)
     return result.stdout if result.returncode == 0 else ""
 
 lag = {}
@@ -162,26 +200,30 @@ if os.path.isfile(lag_path):
             ])
         lag[fields[0].strip()] = (fields[1].strip(), line_number)
 
+failures = []   # 每项是一段要打印的拒绝：(摘要, 明细, 出路)
+
 if lag:
+    owed = owed_library.read_owed_table(owed_path)
     if not os.path.isfile(owed_path):
-        fail(f"滞后登记表有 {len(lag)} 行，而找不到欠账表 {owed_path}", [
+        failures.append((f"滞后登记表有 {len(lag)} 行，而找不到欠账表 {owed_path}", [], [
             "怎么办：欠账表挪了位置就同步改这个阶段里的路径；没有欠账表就核不了滞后登记指的账开没开着，",
             "          而一条指向空处的滞后登记，与 checker 真的跟上了在这一道的输出里一模一样。",
-        ])
-    owed_text = open(owed_path, encoding="utf-8").read()
-    head = owed_text.split("### 已还清")[0]
-    owed_open = set(re.findall(r"^\|\s*(C\d+)\s*\|", head, re.M))
-    dangling = [f"{name}：{number}（第 {line_number} 行）"
-                for name, (number, line_number) in sorted(lag.items())
-                if number not in owed_open]
-    if dangling:
-        print(f"  ✗ 滞后登记表里 {len(dangling)} 行指的欠账编号不在 checks-owed.md 欠着那张表里：")  # gate-lint:summary
-        for entry in dangling:
-            print(f"      {entry}")  # gate-lint:detail
-        print(f"     → 怎么办：登记一条滞后，等于承认这个格式常量今天 checker 判不了，那笔账要有人排期。")
-        print(f"               去 {owed_path} 立一条欠账（写清拦什么、怎么拦会红、缺什么前置），把它的编号写回这一行；")
-        print("               那笔账已经还清了就把这一行删掉——checker 跟上了就不该再登记滞后。")
-        sys.exit(1)
+        ]))
+    elif not owed.paid_heading_found:
+        failures.append((f"滞后登记表有 {len(lag)} 行，而 {owed_path} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [], [
+            "怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
+            "          别让这一道对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。",
+        ]))
+    else:
+        dangling = [f"{name}：{number}（第 {line_number} 行）"
+                    for name, (number, line_number) in sorted(lag.items())
+                    if number not in owed.open_names]
+        if dangling:
+            failures.append((f"滞后登记表里 {len(dangling)} 行指的欠账编号不在 checks-owed.md 欠着那张表里：", dangling, [
+                "怎么办：登记一条滞后，等于承认这个格式常量今天 checker 判不了，那笔账要有人排期。",
+                f"          去 {owed_path} 立一条欠账（写清拦什么、怎么拦会红、缺什么前置），把它的编号写回这一行；",
+                "          那笔账已经还清了就把这一行删掉——checker 跟上了就不该再登记滞后。",
+            ]))
 
 # 键是（格式定义路径, 常量名），不是光一个常量名：同一个常量在 kb 字段表与常量模块里各登记一次
 # （门禁 27 号绑住这两处），按名字合并时后读到的那一份会把前一份盖掉，那一侧的改值就此看不见。
@@ -189,12 +231,12 @@ drifted, empty_sources, checked_constants, excused, followed = [], [], 0, [], []
 for row in rows:
     current, baseline = {}, {}
     for path in row["format"]:
-        found = constants_in(open(path, encoding="utf-8").read(), path)
+        found = constants_in(open(path, encoding="utf-8").read(), path, record_problems=True)
         if not found:
             empty_sources.append(f'{row["name"]}：{path}')
         for name, value in found.items():
             current[(path, name)] = value
-        for name, value in constants_in(baseline_text(path), path).items():
+        for name, value in constants_in(baseline_text(path), path, record_problems=False).items():
             baseline[(path, name)] = value
     checked_constants += len(current)
     changes = []
@@ -216,17 +258,33 @@ for row in rows:
     if unexcused:
         drifted.append((row, unexcused))
 
+if marker_problems:
+    failures.append((f"格式定义里 {len(marker_problems)} 处 format-const 标记读不出来或重复登记，变更探测对它们不作数：", marker_problems, [
+        "怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；",
+        "          它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」；",
+        "          重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。",
+    ]))
+
 if drifted:
     total = sum(len(entries) for _, entries in drifted)
-    print(f"  ✗ {total} 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 {base}）：")  # gate-lint:summary
+    details = []
     for row, entries in drifted:
-        print(f'      {row["name"]}（checker 判定路径：{", ".join(row["checker"])}）')  # gate-lint:detail
-        for entry in entries:
-            print(f"        {entry}")  # gate-lint:detail
-    print("     → 怎么办：两条出路。① 在同一次改动里改这套布局的 checker 判定路径，让它按新格式判——")
-    print("               格式走了一步而 checker 停在旧口径时，它会拿旧宽度去解新字节，而且全绿；")
-    print(f"               ② checker 今天确实判不了它，就把常量名登记进 {lag_path}：")
-    print("               三列写常量名、一条开着的欠账编号、为什么今天不判。账在册才有人排期。")
+        details.append(f'{row["name"]}（checker 判定路径：{", ".join(row["checker"])}）')
+        details.extend(f"  {entry}" for entry in entries)
+    failures.append((f"{total} 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 {base}）：", details, [
+        "怎么办：两条出路。① 在同一次改动里改这套布局的 checker 判定路径，让它按新格式判——",
+        "          格式走了一步而 checker 停在旧口径时，它会拿旧宽度去解新字节，而且全绿；",
+        f"          ② checker 今天确实判不了它，就把常量名登记进 {lag_path}：",
+        "          三列写常量名、一条开着的欠账编号、为什么今天不判。账在册才有人排期。",
+    ]))
+
+if failures:
+    for summary, details, steps in failures:
+        print(f"  ✗ {summary}")  # gate-lint:summary
+        for detail in details:
+            print(f"      {detail}")  # gate-lint:detail
+        for step in steps:
+            print(f"     → {step}" if step.startswith("怎么办") else f"     {step}")
     sys.exit(1)
 
 paths_total = sum(len(row["format"]) + len(row["checker"]) for row in rows)
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/kb/invariants.md b/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/kb/invariants.md
index da50377..997f1a8 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/kb/invariants.md
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/kb/invariants.md
@@ -2,6 +2,7 @@
 
 | I-92.1 | 绿样条 | 样例陈述 |
 
+<!-- invariant-count -->
 现共 1 条在用。
 
 ## 历史版本
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect b/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
index 316794c..2dce7c4 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
@@ -2,3 +2,5 @@ exit=0
 want=kb 腐化审计通过
 want=E342 不被任何决策引用，正文写明是备料，等 D340
 want=欠检查 1 条、已还清 1 条
+want=不变量条数一致：正文声称 1 条在用，表里在用 1 条
+want=共 2 个，被决策引用 1 个、写明备料 1 个
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/checks-owed.md b/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/checks-owed.md
index 8691145..100af24 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/checks-owed.md
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/checks-owed.md
@@ -1,6 +1,4 @@
-<!-- doc-lint:registry name-col=2 -->
-
-| C93 | 红样账 | 欠着 |
+欠账表两张都空着：这一份样本故意一行欠账行都不放，第 5 段数到 0 条必须判红。
 
 ## 历史版本
 
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/decisions/341-样例.md b/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/decisions/341-样例.md
index 6e562c4..9590945 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/decisions/341-样例.md
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/kb/decisions/341-样例.md
@@ -1,6 +1,6 @@
 ## D341 红样决策 —— 已定
 
-依据 E341（红样实验）的实测。
+依据 E341（红样实验）与 E344（红样同批漏改实验）的实测。
 
 ## 历史版本
 
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/red/expect b/.claude/gate.d/fixtures/10-kb-rot.sh/red/expect
index 9e9c644..d71ad5f 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/red/expect
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/red/expect
@@ -1,3 +1,5 @@
 exit=1
 want=E999 被引用但 experiments/ 下没有它
 want=E343 已跑，但决策正文一次都没引用它
+want=invariants.md 里没有 <!-- invariant-count --> 标记
+want=checks-owed.md 里一行 `| C<n> ` 都没数到（欠着 0、已还清 0）
diff --git a/.claude/gate.d/fixtures/40-results-cited.sh/green/.claude/kb/experiments.md b/.claude/gate.d/fixtures/40-results-cited.sh/green/.claude/kb/experiments.md
index dce4574..9f8baf8 100644
--- a/.claude/gate.d/fixtures/40-results-cited.sh/green/.claude/kb/experiments.md
+++ b/.claude/gate.d/fixtures/40-results-cited.sh/green/.claude/kb/experiments.md
@@ -1,5 +1,5 @@
 # 待做实验
 
-产物 `research/results/e99-orphan-2026-01-01.out` 已点名。
+产物 `research/results/e99-orphan-2026-08-29.out` 已点名。
 
 ## 历史版本
diff --git a/.claude/gate.d/fixtures/40-results-cited.sh/green/research/results/e99-orphan-2026-01-01.out b/.claude/gate.d/fixtures/40-results-cited.sh/green/research/results/e99-orphan-2026-01-01.out
deleted file mode 100644
index 580d588..0000000
--- a/.claude/gate.d/fixtures/40-results-cited.sh/green/research/results/e99-orphan-2026-01-01.out
+++ /dev/null
@@ -1 +0,0 @@
-E7RESULT name=done emitted=1
diff --git a/.claude/gate.d/fixtures/40-results-cited.sh/red/expect b/.claude/gate.d/fixtures/40-results-cited.sh/red/expect
index b262f4e..e60de6b 100644
--- a/.claude/gate.d/fixtures/40-results-cited.sh/red/expect
+++ b/.claude/gate.d/fixtures/40-results-cited.sh/red/expect
@@ -1,2 +1,4 @@
 exit=1
 want=没被 experiments.md 点名
+want=E98 点名 e98-nowhere-2026-09-24.out —— 树里没有，git 历史里也没有
+want=1 份被点名的产物既不在 research/results 下、也不在版本库历史里
diff --git a/.claude/gate.d/fixtures/40-results-cited.sh/red/research/results/e99-orphan-2026-01-01.out b/.claude/gate.d/fixtures/40-results-cited.sh/red/research/results/e99-orphan-2026-01-01.out
deleted file mode 100644
index 580d588..0000000
--- a/.claude/gate.d/fixtures/40-results-cited.sh/red/research/results/e99-orphan-2026-01-01.out
+++ /dev/null
@@ -1 +0,0 @@
-E7RESULT name=done emitted=1
diff --git a/.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh b/.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh
index 5c925d1..433af62 100644
--- a/.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh
+++ b/.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh
@@ -3,5 +3,5 @@
 set -e
 export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
 git init -q -b master . && git add -A && git commit -qm base
-printf '\n### 已定项 2 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/100-样本.md
-sed -i 's/两条出路都还没选/两条出路都还没选（2026-01-02 复核过，仍然开着）/' .claude/kb/decisions/100-样本.md
+printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/100-样本.md
+sed -i 's/两条出路都还没选/两条出路都还没选（2026-09-01 复核过，仍然开着）/' .claude/kb/decisions/100-样本.md
diff --git a/.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh b/.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh
index fe469a9..6fc46fd 100644
--- a/.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh
+++ b/.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh
@@ -1,7 +1,11 @@
 #!/usr/bin/env bash
-# 先提交一份带未定项的决策，再在**工作区**加一个「已定」小节而不碰那条未定项
-# ⇒ 本该判红：定了新东西却没回头看同文件还开着的那条。
+# 先提交一份带未定项的决策、把上游设在这个提交，再**另提交一次、不推**：加一个「已定」小节而不碰那条未定项；
+# 工作区另改一个无关文件 ⇒ 本该判红：定了新东西却没回头看同文件还开着的那条。
+# 定案已经提交在本地而没推，正是「在默认分支上 merge-base 就是 HEAD」那种基准会漏掉的形状（三方判决 gate-fix-forks-r1 的 T5）。
 set -e
 export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
 git init -q -b master . && git add -A && git commit -qm base
-printf '\n### 已定项 2 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md
+git branch 上游 && git branch -q --set-upstream-to=上游 master
+printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md
+git add -A && git commit -qm '定案，没推'
+printf '无关的改动\n' > 无关.md
diff --git a/.claude/gate.d/fixtures/80-absolute-assertions.sh/green/expect b/.claude/gate.d/fixtures/80-absolute-assertions.sh/green/expect
index 9200c2a..1d10d1d 100644
--- a/.claude/gate.d/fixtures/80-absolute-assertions.sh/green/expect
+++ b/.claude/gate.d/fixtures/80-absolute-assertions.sh/green/expect
@@ -1 +1,4 @@
 exit=0
+want=✓ 2 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 1 份，别处 src/bin 下以 e<数字>_ 开头的 1 份）
+want=没判的 1 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）
+want=crates/demo/src/bin/probe.rs
diff --git a/.claude/gate.d/fixtures/80-absolute-assertions.sh/red/expect b/.claude/gate.d/fixtures/80-absolute-assertions.sh/red/expect
index ad7ee0a..a9fc978 100644
--- a/.claude/gate.d/fixtures/80-absolute-assertions.sh/red/expect
+++ b/.claude/gate.d/fixtures/80-absolute-assertions.sh/red/expect
@@ -1,2 +1,3 @@
 exit=1
-want=一条绝对值断言都没有
+want=e1_relative.rs 一条绝对值断言都没有
+want=crates/demo/src/bin/e2_crates_relative.rs 一条绝对值断言都没有
diff --git a/.claude/gate.d/fixtures/88-quoted-result-lines.sh/green/research/results/e99-sample-2026-01-01.out b/.claude/gate.d/fixtures/88-quoted-result-lines.sh/green/research/results/e99-sample-2026-01-01.out
deleted file mode 100644
index efcfa9e..0000000
--- a/.claude/gate.d/fixtures/88-quoted-result-lines.sh/green/research/results/e99-sample-2026-01-01.out
+++ /dev/null
@@ -1,2 +0,0 @@
-E7RESULT name=verdict arm=a count=3
-E7RESULT name=done emitted=2
diff --git a/.claude/gate.d/fixtures/88-quoted-result-lines.sh/red/research/results/e99-sample-2026-01-01.out b/.claude/gate.d/fixtures/88-quoted-result-lines.sh/red/research/results/e99-sample-2026-01-01.out
deleted file mode 100644
index efcfa9e..0000000
--- a/.claude/gate.d/fixtures/88-quoted-result-lines.sh/red/research/results/e99-sample-2026-01-01.out
+++ /dev/null
@@ -1,2 +0,0 @@
-E7RESULT name=verdict arm=a count=3
-E7RESULT name=done emitted=2
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 9f8c1d5..e59f3f9 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -19,6 +19,8 @@
 
 **改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。
 
+**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先做成一个必须报非 0 的模型，再改定义。
+
 ## 代码轮派腿之前记一份开工快照
 
 派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。
diff --git a/.claude/scripts/gen-decision-items.py b/.claude/scripts/gen-decision-items.py
index 458a9e9..2c3a70f 100644
--- a/.claude/scripts/gen-decision-items.py
+++ b/.claude/scripts/gen-decision-items.py
@@ -34,8 +34,14 @@ def clip(text, n):
     # 截断还可能在编号后面切掉它的简称，留下一个**裸引用**——doc-lint 同样判红，
     # 而它判得对：一个只剩符号的编号，含义可以被悄悄改掉而没有一个字看起来别扭
     # （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条）。把这种尾巴一并去掉。
+    # ⚠️ **只在真截断过时剥**（按长度截了，或上面剥过半个括注）：没截断的名字结尾那串是原文自己写的，剥掉就丢了原文。
+    # 编号按 doc-lint 认编号的同一个形状剥：整词（`[A-Z]+-?数字(.数字)*`），前一个字符不是字母、数字、`.`、`_`、`-`。
+    # 不带左边界时，截断正好停在「SHA256」「RAID5」之后会从词中间咬走「A256」「D5」，剩下半截「SH」「RAI」；
+    # 带了左边界，截断尾巴上的领域词要么整个留着、要么整个剥掉，不会剩半截（三方判决 gate-fix-forks-r1 的 T8）。
+    if t == text:
+        return t.rstrip()
     while True:
-        t2 = re.sub(r'[A-Z]-?\d+(?:\.\d+)?\s*$', '', t)
+        t2 = re.sub(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', '', t)
         t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
         if t2 == t:
             break
@@ -71,8 +77,8 @@ def harvest(sec):
     return re.findall(r'^(\d+)\.\s+(.*)$', top, re.M)
 
 
-def items_of(body):
-    """返回 [(编号, 名字, 状态)]，两节合起来按编号排。"""
+def items_of(body, where):
+    """返回 [(编号, 名字, 状态)]，两节合起来按编号排。where 是「文件（决策号）」，只用在报错里。"""
     res = []
     for head, st in (('已定项', '已定'), ('未定项', '**未定**')):
         m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, re.M | re.S)
@@ -85,7 +91,7 @@ def items_of(body):
         # 于是那一节没写索引表这件事在索引页上躺了很久没人看见。
         if not got:
             raise SystemExit(
-                f"  ✗ 「### {head}」一节里取不到编号项\n"
+                f"  ✗ {where} 的「### {head}」一节里取不到编号项\n"
                 f"     → 该节要有索引表（`| # | 分项 | 状态 |`）或编号列表，"
                 f"每条分项一行、编号连号不重排")
         for n, txt in got:
@@ -103,11 +109,16 @@ for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
     title = s.split('\n', 1)[0]
     # 破折号前有没有空格两种都有（D14 没有），不能只认一种
     mm = re.match(r'## (D\d+) (.+?)\s*—— *(.+)$', title)
+    # 首行读不出就报错，不许跳过：跳过的那条决策从分项清单里整条消失，21 号只会报一句
+    # 「索引表里有行而 decisions/ 下没有它的正文」，31 号则一条未定项都不替它查。
     if not mm:
-        continue
+        raise SystemExit(
+            f"  ✗ {f} 首行读不出 `## D<n> 简称 —— 状态`：{title[:40]!r}\n"
+            f"     → 把首行写成 `## D3 空间分配 —— 半定（…）`，首行之前不许有空行或 BOM；"
+            f"decisions/ 下只放决策正文")
     num, name, status = mm.group(1), mm.group(2).strip(), mm.group(3).strip()
     body = s.split('\n## 历史版本')[0]
-    its = items_of(body)
+    its = items_of(body, f"{f}（{num}）")
     open_n = sum(1 for _, _, st in its if '未定' in st)
     kind = re.match(r'(已定|半定|待定)', status)
     kind = kind.group(1) if kind else status[:6]
```

## 二、未跟踪文件全文（工作区未跟踪；`git ls-files --others --exclude-standard -- <目录>` 现查所得，逐份列全，未截）

### `research/scripts/changed-paths.sh`（175 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 门禁阶段共用的「这次改动碰了哪些路径」取法（中文路径不转义、未跟踪可开关、基准的回退次序）。
#
# 这是门禁阶段共用的「这次改动碰了哪些路径、新增了哪些行」取法，阶段 source 它，不各抄一份（门禁 64 号判阶段里没有第二份）：
# 六份拷贝已经分叉过——一半的 git 调用漏了 -c core.quotepath=false（中文路径被转成带引号的八进制串，
# 与仓里的路径逐字对不上），97 号借「与 56 号同一条」时又省掉了未跟踪文件。
#
# 两个函数：
#   gate_diff_base <取法>
#     gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。
#           一轮可以跨几个提交，问「这一轮碰了哪些」的阶段用它（56、68、69、97 号）。
#     head：HEAD。问「这一次提交要带哪些」的阶段用它（11 号；为什么不用 GATE_BASE 写在 11 号头部）。
#   gate_changed_paths <基准> <untracked | tracked-only> [<diff-filter>]
#     一行一条、排序去重：基准到工作区的 diff、HEAD 到暂存区的 diff；第二个参数是 untracked 时再加未跟踪文件。
#     给了 diff-filter（例 A）只在两个 diff 上加；未跟踪文件只在 diff-filter 为空或含 A 时加（它们本来就是新增）。
#     git 调用一律带 -c core.quotepath=false。路径里带制表符、换行或引号的，git 照样转义，这一条管不到。
#   gate_added_lines <基准> <路径>…
#     这次改动新增的行，一行一条「路径<TAB>行文」：跟踪的文件取基准到工作区的 diff 里以 + 开头的行，
#     **未跟踪的文件整份都算新增**（`git diff` 看不到它们，新写的 kb 页在 git add 之前整页会被当成「早先写下的」跳过）。
#     路径可以是目录。git 失败时退出码非 0、不输出半截结果，调用方按「拿不到改动范围」处理（通常退回全量判）。
#
# 阶段里的用法：
#   LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
#   source "$LIB_CHANGED_PATHS" || { echo "  ✗ …"; echo "     → …"; exit 1; }
#   base="$(gate_diff_base gate)"
#   changed="$(gate_changed_paths "$base" untracked)"
#
# 住在 research/scripts/ 而不是 .claude/gate.d/：共享门禁（gate.sh）把 gate.d 下每个 *.sh 都当阶段跑，
# 阶段调用的共用 shell 脚本照 stage-must-run.sh、change-touches-crates.sh 的惯例放这里；自证由门禁 47 号跑。
# `--selftest` 在临时 git 仓里自证：三种来源（工作区改、暂存新增、未跟踪）各放一个中文路径，逐字核两个函数的输出。
# 判别力：LIB_CHANGED_PATHS_BREAK=quotepath 让取法漏掉 core.quotepath=false、=untracked 让它丢掉未跟踪文件、
# =fallback 让 gate 取法不认 GATE_BASE、=empty-merge-base 让 merge-base 取不到时交出空串、
# =untracked-lines 让新增行丢掉未跟踪文件、=swallow 让路径取法吞掉 git 的失败，六种自证都必须判红。
#
#   bash research/scripts/changed-paths.sh --selftest      # 自证
#   LIB_CHANGED_PATHS_BREAK=quotepath bash research/scripts/changed-paths.sh --selftest   # 必须判红

gate_diff_base() {
  local mode="$1" merge_base
  case "$mode" in
    head)
      echo "HEAD"
      ;;
    gate)
      if [[ "${LIB_CHANGED_PATHS_BREAK:-}" != fallback && -n "${GATE_BASE:-}" ]] \
         && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
        echo "$GATE_BASE"
      elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1 \
           && { merge_base="$(git merge-base HEAD '@{upstream}' 2>/dev/null)" || [[ "${LIB_CHANGED_PATHS_BREAK:-}" == empty-merge-base ]]; }; then
        echo "$merge_base"
      else
        echo "HEAD"
      fi
      ;;
    *)
      echo "gate_diff_base：取法只认 gate 或 head，收到「$mode」" >&2
      return 2
      ;;
  esac
}

gate_changed_paths() {
  local base="$1" untracked_mode="$2" diff_filter="${3:-}"
  local quote_option=(-c core.quotepath=false) filter_option=()
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == quotepath ]] && quote_option=()
  [[ -n "$diff_filter" ]] && filter_option=("--diff-filter=$diff_filter")
  case "$untracked_mode" in
    untracked|tracked-only) ;;
    *)
      echo "gate_changed_paths：第二个参数只认 untracked 或 tracked-only，收到「$untracked_mode」" >&2
      return 2
      ;;
  esac
  local include_untracked=0
  if [[ "$untracked_mode" == untracked && "${LIB_CHANGED_PATHS_BREAK:-}" != untracked ]] \
     && [[ -z "$diff_filter" || "$diff_filter" == *A* ]]; then
    include_untracked=1
  fi
  # 三次调用逐条判退出码、先落到变量再排序：写成 `{ …; …; …; } | sort -u` 时花括号组只交最后一条的退出码，
  # 基准写错、第一条 git diff 失败，集合就静默少了一截。
  local listed
  listed="$(
    git "${quote_option[@]}" diff --name-only "${filter_option[@]}" "$base" -- || [[ "${LIB_CHANGED_PATHS_BREAK:-}" == swallow ]] || exit 1
    git "${quote_option[@]}" diff --name-only --cached "${filter_option[@]}" -- || exit 1
    if ((include_untracked)); then git "${quote_option[@]}" ls-files --others --exclude-standard -- || exit 1; fi
  )" || return 1
  [[ -z "$listed" ]] || sort -u <<<"$listed"
}

gate_added_lines() {
  local base="$1"; shift
  local quote_option=(-c core.quotepath=false) diff_output untracked_list untracked_path
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == quotepath ]] && quote_option=()
  # 前缀写死：diff.noprefix 之类的配置会让 +++ 行不带 b/，路径就解析错了。
  diff_output="$(git "${quote_option[@]}" diff --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1
  untracked_list="$(git "${quote_option[@]}" ls-files -z --others --exclude-standard -- "$@" | tr '\0' '\n')" || return 1
  awk '
    /^\+\+\+ / { path = substr($0, 5); if (path == "/dev/null") path = ""; else sub(/^b\//, "", path); next }
    /^\+/ && path != "" { print path "\t" substr($0, 2) }' <<<"$diff_output"
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == untracked-lines ]] && return 0
  while IFS= read -r untracked_path; do
    [[ -n "$untracked_path" && -f "$untracked_path" ]] || continue
    UNTRACKED_PATH="$untracked_path" awk '{ print ENVIRON["UNTRACKED_PATH"] "\t" $0 }' "$untracked_path"
  done <<<"$untracked_list"
}

# ── 直接跑（不是 source）时：只认 --selftest，在临时仓里自证 ─────────────────
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  [[ "${1:-}" == "--selftest" ]] || { echo "  ✗ 用法：bash research/scripts/changed-paths.sh --selftest（平时由门禁阶段 source 它）"; echo "     → 怎么办：自证就带 --selftest；要取改动路径，在阶段里 source 它再调 gate_diff_base / gate_changed_paths。"; exit 2; }
  selftest_directory="$(mktemp -d)"
  trap 'rm -rf "${selftest_directory:?}"' EXIT
  selftest_output="$(
    cd "$selftest_directory" || exit 2
    # 与使用者的 git 配置隔开：全局配了 core.quotepath=false 的机器上，漏掉那个选项的取法也照样对，自证就分不出来。
    export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
    export GIT_AUTHOR_NAME=selftest GIT_AUTHOR_EMAIL=selftest@example.invalid
    export GIT_COMMITTER_NAME=selftest GIT_COMMITTER_EMAIL=selftest@example.invalid
    unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE
    git init -q -b master . || exit 2
    printf '第一版\n' > 已跟踪.md
    git add -A && git commit -qm 基准 || exit 2
    first_commit="$(git rev-parse HEAD)"
    printf 'x\n' > 第二个提交.md
    git add -A && git commit -qm 第二个提交 || exit 2
    second_commit="$(git rev-parse HEAD)"
    # 上游指第二个提交：merge-base 与 GATE_BASE 取的第一个提交不同，回退那一档错了才分得出来。
    git branch 上游 "$second_commit" && git branch -q --set-upstream-to=上游 master || exit 2
    printf '第二版\n' > 已跟踪.md
    printf '新\n' > 暂存新增.md && git add 暂存新增.md
    printf '未跟踪\n' > 未跟踪.md
    check() {
      local label="$1" expected="$2" actual="$3"
      if [[ "$actual" == "$expected" ]]; then
        echo "PASS $label"
      else
        echo "FAIL $label：期望「${expected//$'\n'/ | }」，实测「${actual//$'\n'/ | }」"
      fi
    }
    check "未跟踪也算、中文路径不转义" "$(printf '%s\n' 已跟踪.md 暂存新增.md 未跟踪.md 第二个提交.md | sort -u)" \
          "$(gate_changed_paths "$first_commit" untracked)"
    check "tracked-only 不带未跟踪" "$(printf '%s\n' 已跟踪.md 暂存新增.md | sort -u)" \
          "$(gate_changed_paths HEAD tracked-only)"
    check "diff-filter=A 只要新增、未跟踪算新增" "$(printf '%s\n' 暂存新增.md 未跟踪.md | sort -u)" \
          "$(gate_changed_paths HEAD untracked A)"
    check "新增行：改过的行、暂存新增的整份、未跟踪的整份，中文路径不转义" \
          "$(printf '%s\t%s\n' 已跟踪.md 第二版 暂存新增.md 新 未跟踪.md 未跟踪 | sort)" "$(gate_added_lines HEAD . | sort)"
    check "新增行只看给的路径、删掉的旧行不算" "$(printf '%s\t%s\n' 已跟踪.md 第二版 第二个提交.md x | sort)" \
          "$(gate_added_lines "$first_commit" 第二个提交.md 已跟踪.md | sort)"
    check "基准解析不到：两个取法都退非 0，不交半截结果" "1 1 空" \
          "$(gate_changed_paths 没有这个提交 untracked >/dev/null 2>&1; a=$?; gate_added_lines 没有这个提交 . >/dev/null 2>&1; b=$?; \
             out="$(gate_changed_paths 没有这个提交 untracked 2>/dev/null)"; echo "$((a != 0)) $((b != 0)) ${out:-空}")"
    check "head 取法是 HEAD" "HEAD" "$(gate_diff_base head)"
    check "GATE_BASE 是个提交就用它" "$first_commit" "$(GATE_BASE="$first_commit" gate_diff_base gate)"
    check "GATE_BASE 不是提交就回退到上游的 merge-base" "$second_commit" "$(GATE_BASE=没有这个提交 gate_diff_base gate)"
    # 上游是一条没有共同祖先的分支：merge-base 取不到，要回退到 HEAD，不许拿空串当基准（空基准让 git diff 报错、改动范围静默变空）。
    orphan_commit="$(git commit-tree "$(git mktree </dev/null)" -m 孤儿)" && git branch 孤儿 "$orphan_commit" \
      && git branch -q --set-upstream-to=孤儿 master || exit 2
    check "上游与 HEAD 没有共同祖先就回退到 HEAD" "HEAD" "$(gate_diff_base gate)"
    git branch -q --unset-upstream
    check "没有上游就回退到 HEAD" "HEAD" "$(gate_diff_base gate)"
  )"
  check_count="$(grep -c '^PASS \|^FAIL ' <<<"$selftest_output")"
  failures="$(grep '^FAIL ' <<<"$selftest_output")"
  if [[ -n "$failures" || "$check_count" -ne 11 ]]; then
    echo "  ✗ 共用库 research/scripts/changed-paths.sh 的自证没过（查了 $check_count 项，应当 11 项）："   # gate-lint:summary
    printf '%s\n' "$failures" | sed 's/^FAIL /      /'   # gate-lint:detail
    printf '%s\n' "$selftest_output" | grep -v '^PASS \|^FAIL ' | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：看 gate_changed_paths / gate_added_lines 的 git 调用有没有带 -c core.quotepath=false、未跟踪文件那一支有没有加 ls-files --others，"
    echo "               gate_diff_base 的回退次序是不是 GATE_BASE → @{upstream} 的 merge-base → HEAD；门禁 11、56、68、69、97 号都靠它取改动范围。"
    exit 1
  fi
  echo "  ✓ 共用库 research/scripts/changed-paths.sh 自证通过（$check_count 项：三种来源的中文路径逐字不转义、未跟踪可开关、diff-filter=A、新增行含未跟踪整份且只看给的路径、基准解析不到退非 0、基准回退四档）"
  exit 0
fi
```

### `.claude/gate.d/64-change-range-single-source.sh`（112 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# gate-stage: 改动范围只许一份取法：阶段不自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
#
# 判据（射程是 `.claude/gate.d/` 顶层的 *.sh 与 *.py，本阶段自己除外）。两条，任一条不成立判红：
#   ① 算 diff 基准只许在 research/scripts/changed-paths.sh 里：代码行里出现 merge-base、@{upstream}、
#      gate-ok、读 GATE_BASE 或 GATE_DIFF_BASE 的，判红。阶段要改动范围就 source 那份共用脚本、调 gate_diff_base。
#   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false（配置键不分大小写）：
#      同一行写了它；或者这一行用的变量 / 列表在同一个文件里带着它定义（例 `GIT = ["git", "-c", "core.quotepath=false"]`）；
#      或者调的是同一个文件里定义的包装函数，函数头往下三行内带着它（例 75 号的 `def git(*args)`）；或者这一行带 -z（git 在 -z 下不转义路径）。
#      中文路径不带这个选项会被转成带引号的八进制串，与仓里的路径逐字对不上，而 git 不报错。
#   「代码行」：去掉前导空白后不以 #、echo、printf、print(、bad、howto、die、引号开头的行——出路文字与注释里提到这些词不算。
#
# 为什么：改动范围的取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath；61 号的基准在默认分支上
# 就是 HEAD，定案分两次提交、第一次没推，它就退 77 不判）。收完之后没有东西拦下一份拷贝。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T5。
#
# 没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下的脚本不是门禁阶段，成功行把其中碰到这两条的逐个列名，清单现算。
# 样本：fixtures/64-change-range-single-source.sh/red 放一个自己取 merge-base 的阶段与一个不带 quotepath 的 ls-files，判红；
# green 放一个 source 共用脚本的阶段、一个在同一行带 quotepath 的、一个用带 quotepath 的列表的，判绿。
#
#   bash .claude/gate.d/64-change-range-single-source.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，本阶段无对象可判"; exit 77; }
python3 - "$(basename "$0")" <<'PY'
import glob, os, re, sys

own_name = sys.argv[1]
BASE_PATTERN = re.compile(r'merge-base|@\{upstream\}|gate-ok|\bGATE_BASE\b|\bGATE_DIFF_BASE\b')
LIST_PATTERN = re.compile(r'--name-only|--name-status|\bls-files\b')
TEXT_PREFIX = re.compile(r'''^(#|echo\b|printf\b|print\(|bad\b|howto\b|die\b|["'])''')
QUOTEPATH = re.compile(r'quotepath', re.I)
QUOTE_VARIABLE = re.compile(r'^\s*([A-Za-z_][A-Za-z_0-9]*)\s*\+?=.*quotepath', re.I)
WRAPPER_HEAD = re.compile(r'^\s*(?:def\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(|(?:function\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*\(\)\s*\{?)')
NO_QUOTING = re.compile(r'(?<![\w-])-z\b|[\'"]-z[\'"]')
WRAPPER_REACH = 3

def code_lines(path):
    with open(path, encoding='utf-8', errors='replace') as handle:
        for number, line in enumerate(handle, 1):
            stripped = line.strip()
            if not stripped or TEXT_PREFIX.match(stripped):
                continue
            yield number, line.rstrip('\n')

def quoting_names(path):
    """同一个文件里带着 quotepath 定义的变量、列表与包装函数的名字。"""
    with open(path, encoding='utf-8', errors='replace') as handle:
        all_lines = handle.read().split('\n')
    names = set()
    for index, line in enumerate(all_lines):
        variable = QUOTE_VARIABLE.match(line)
        if variable:
            names.add(variable.group(1))
        wrapper = WRAPPER_HEAD.match(line)
        if wrapper and any(QUOTEPATH.search(body) for body in all_lines[index:index + 1 + WRAPPER_REACH]):
            names.add(wrapper.group(1) or wrapper.group(2))
    return names

def judge(path):
    """返回 (算基准的行, 不带 quotepath 的列路径行)。"""
    lines = list(code_lines(path))
    names = quoting_names(path)
    base_hits, list_hits = [], []
    for number, line in lines:
        if BASE_PATTERN.search(line):
            base_hits.append((number, line.strip()))
        if LIST_PATTERN.search(line) and not QUOTEPATH.search(line) and not NO_QUOTING.search(line) \
                and not any(re.search(r'(\$\{?|\b)' + re.escape(name) + r'\b', line) for name in names):
            list_hits.append((number, line.strip()))
    return base_hits, list_hits

scanned = sorted(path for pattern in ('.claude/gate.d/*.sh', '.claude/gate.d/*.py')
                 for path in glob.glob(pattern) if os.path.basename(path) != own_name)
base_bad, list_bad = [], []
for path in scanned:
    base_hits, list_hits = judge(path)
    base_bad += [(path, number, line) for number, line in base_hits]
    list_bad += [(path, number, line) for number, line in list_hits]

failed = False
if base_bad:
    failed = True
    print(f'  ✗ {len(base_bad)} 行在阶段里自己算 diff 基准（改动范围只许一份取法：research/scripts/changed-paths.sh）：')
    for path, number, line in base_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：阶段里 source research/scripts/changed-paths.sh，基准写 base="$(gate_diff_base gate)"（问「这一次提交带哪些」的写 head），')
    print('               路径写 gate_changed_paths、新增行写 gate_added_lines；共用脚本缺哪种取法就往它里面加，连同它的 --selftest 一起改。')
if list_bad:
    failed = True
    print(f'  ✗ {len(list_bad)} 行列路径的 git 调用没带 -c core.quotepath=false：')
    for path, number, line in list_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：在那一行的 git 后面加 -c core.quotepath=false（python 里是 ["git", "-c", "core.quotepath=false", …]），')
    print('               或者把带它的调用前缀收进一个变量 / 列表再用；不带它时中文路径变成带引号的八进制串，与仓里的路径逐字对不上。')
if failed:
    sys.exit(1)

outside = []
for pattern in ('research/scripts/*.sh', 'research/scripts/*.py', '.claude/scripts/*', '.claude/hooks/*'):
    for path in sorted(glob.glob(pattern)):
        if os.path.isfile(path) and path != 'research/scripts/changed-paths.sh':
            base_hits, list_hits = judge(path)
            if base_hits or list_hits:
                outside.append(f'{path}（算基准 {len(base_hits)} 行、不带 quotepath 的列路径 {len(list_hits)} 行）')
print(f'  ✓ 查了 .claude/gate.d/ 下 {len(scanned)} 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false')
if outside:
    print(f'     没扫的 {len(outside)} 份（不是门禁阶段，射程之外）：' + '；'.join(outside))
else:
    print('     没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下没有碰到这两条的脚本')
PY
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/.claude/gate.d/50-样本走共用脚本.sh`（6 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 绿样本：基准从共用脚本取；出路文字里提到 merge-base、GATE_BASE 不算
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
base="$(gate_diff_base gate)"
echo "  → 基准不对就看 GATE_BASE 与 merge-base"
gate_changed_paths "$base" untracked
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/.claude/gate.d/51-样本同一行带选项.sh`（3 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 绿样本：同一行带着选项，配置键大小写不同也认
git -c core.quotePath=false log --all --diff-filter=D --format= --name-only -- research/results
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/.claude/gate.d/52-样本带选项的列表.py`（4 行，工作区未跟踪）

```py
# 绿样本：调用前缀收进一个带着选项的列表
import subprocess
GIT = ["git", "-c", "core.quotepath=false"]
names = subprocess.run(GIT + ["ls-files", "--others", "--exclude-standard"], capture_output=True, text=True).stdout
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/.claude/gate.d/53-样本包装函数.py`（5 行，工作区未跟踪）

```py
# 绿样本：同一个文件里的包装函数带着选项
import subprocess
def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, text=True)
tracked = git('ls-files', '--error-unmatch', 'a.md').returncode == 0
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/.claude/gate.d/54-样本零分隔.sh`（3 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 绿样本：-z 下 git 不转义路径
git ls-files -z --cached --others --exclude-standard | sort -z -u
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/green/expect`（2 行，工作区未跟踪）

```text
exit=0
want=查了 .claude/gate.d/ 下 5 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/red/.claude/gate.d/50-样本自己算基准.sh`（5 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 红样本：阶段自己取 merge-base，不走共用脚本
base="$(git merge-base HEAD '@{upstream}')"
echo "  → 出路文字里提到 merge-base 不算"
git -c core.quotepath=false diff --name-only "$base" --
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/red/.claude/gate.d/51-样本列路径不带选项.sh`（3 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 红样本：列路径的 git 调用没带 quotepath，中文路径会被转义
git ls-files --others --exclude-standard -- .claude/kb
```

### `.claude/gate.d/fixtures/64-change-range-single-source.sh/red/expect`（5 行，工作区未跟踪）

```text
exit=1
want=1 行在阶段里自己算 diff 基准
want=.claude/gate.d/50-样本自己算基准.sh:3
want=1 行列路径的 git 调用没带 -c core.quotepath=false
want=.claude/gate.d/51-样本列路径不带选项.sh:3
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/green/.claude/kb/layout/01-first-txn.md`（11 行，工作区未跟踪）

```md
## 八、合成登记表

产物的 `name=segments` 两行（`path=a / b`）。

| 路径 | 段序列 | 出处 | 层 0 |
|---|---|---|---|
| 甲路径 | `2+1`，3 次操作、5 个崩溃状态，种类 `[x×2]\|[y]` | 产物 `name=segments path=a` | 无 |

⚠️ 整条流 `1+1`、3 个状态（种类 `[x]|[y]`）。

## 九、下一节
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/green/expect`（2 行，工作区未跟踪）

```text
exit=0
want=比对了 2 处登记（表格 1 行 + 整条流提示 1 处
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/green/research/results/syn.out`（2 行，工作区未跟踪）

```text
E7RESULT name=segments path=a operations=3 segments=2+1 closed_form=5 kinds=[x×2]|[y]
E7RESULT name=segments path=b operations=2 segments=1+1 closed_form=3 kinds=[x]|[y]
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/green/research/scripts/replay.sh`（1 行，工作区未跟踪）

```sh
E142|@x||syn.out|exact
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/red/.claude/kb/layout/01-first-txn.md`（11 行，工作区未跟踪）

```md
## 八、合成登记表

产物的 `name=segments` 两行（`path=a / b`）。

| 路径 | 段序列 | 出处 | 层 0 |
|---|---|---|---|
| 甲路径 | `3+1`，3 次操作、5 个崩溃状态，种类 `[x×2]\|[y]` | 产物 `name=segments path=a` | 无 |

⚠️ 整条流 `1+1`、3 个状态（种类 `[x]|[y]`）。

## 九、下一节
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/red/expect`（3 行，工作区未跟踪）

```text
exit=1
want=对不上，共 1 处
want=甲路径（path=a）：kb 写的段序列是 `3+1`，产物里是 `2+1`
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/red/research/results/syn.out`（2 行，工作区未跟踪）

```text
E7RESULT name=segments path=a operations=3 segments=2+1 closed_form=5 kinds=[x×2]|[y]
E7RESULT name=segments path=b operations=2 segments=1+1 closed_form=3 kinds=[x]|[y]
```

### `.claude/gate.d/fixtures/52-segment-registry.sh/red/research/scripts/replay.sh`（1 行，工作区未跟踪）

```sh
E142|@x||syn.out|exact
```

### `.claude/gate.d/fixtures/40-results-cited.sh/red/setup.sh`（9 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 索引与产物进基准提交、上游设在那里；新写的实验页停在**未跟踪**状态，它点名的产物树里与历史里都没有。
# git diff 看不见未跟踪的页，只有共用脚本把它整份算新增，第三道才判得到（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add .claude/kb/experiments.md research
git commit -qm '索引与产物'
git branch 上游 && git branch -q --set-upstream-to=上游 master
```

### `.claude/gate.d/fixtures/40-results-cited.sh/red/.claude/kb/experiments/98-新页样本.md`（5 行，工作区未跟踪）

```md
## E98 新页样本 —— 已跑

口径：原始输出 e98-nowhere-2026-09-24.out。

## 历史版本
```

### `.claude/gate.d/fixtures/88-quoted-result-lines.sh/red/setup.sh`（10 行，工作区未跟踪）

```sh
#!/usr/bin/env bash
# 产物先进基准提交，抄错了的那一页停在**未跟踪**状态：新写的 kb 页在 git add 之前，
# git diff 看不见它，只有共用脚本把未跟踪文件整份算新增才判得到（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add research
git commit -qm '产物'
# 上游设在基准提交：旧的取法（先认 gate-ok、再认上游的 tip）在这里取得到基准，于是只看 git diff、漏掉未跟踪的那一页
git branch 上游 && git branch -q --set-upstream-to=上游 master
```

## 三、对照用、未改动的文件全文（这一轮没有改动，供比对用）

### `.claude/gate.d/75-decision-experiment-links.sh`（457 行，工作区当前内容；未在本轮改动）

```sh
#!/usr/bin/env bash
# gate-stage: 决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（GATE_BASE / @{upstream} 的 merge-base / HEAD 起，工作区、暂存区、未跟踪文件都算）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物变了，
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块；索引表的定案格不带日期、不超过 100 字。
#   ⑧ 有实验才有决策（用户 2026-09-19）：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。
#   ⑨ 实验必须对应决策（用户 2026-09-19）：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题状态里写着「作废」或「退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 为什么：2026-09-19 用户指出决策知识腐烂、分项膨胀、决策与实验关联不够，要求「做实验或者有结论后要回去看决策
# 是不是受到了影响」。当天现量：154 个实验页里 296 对「实验引了决策、决策没引回来」，决策里没有一处统一的依据栏；
# 一个实验重跑、结论变了之后，没有任何东西逼人回去看它撑着的那几条决策。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
#   bash .claude/gate.d/75-decision-experiment-links.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then
  echo "  ✗ 找不到 .claude/kb/experiments 或 .claude/kb/decisions"
  echo "  → 怎么办：kb 挪了位置就同步改这个阶段里的路径。"
  exit 1
fi
base="HEAD"
in_git=0
changed=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1 && git rev-parse --verify -q HEAD >/dev/null 2>&1; then
  in_git=1
  # 改动范围与 56、68、69、97 号同一份取法：research/scripts/changed-paths.sh 的 gate 取法，未跟踪也算
  LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
  changed="$(gate_changed_paths "$base" untracked)"
fi
python3 - "$ROOT" "$base" "$in_git" <(printf '%s\n' "$changed") <<'PY'
import glob, os, re, subprocess, sys

root, base, in_git = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
changed = {line for line in open(sys.argv[4], encoding='utf-8').read().split('\n') if line}
kb = os.path.join(root, '.claude', 'kb')
RELATIONS = ('支撑', '推翻', '备料', '不影响')
BASIS_RELATIONS = ('支撑', '推翻')
E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
D_REF = re.compile(r'(?<![A-Za-z0-9])D(\d+)（')
TABLE_HEAD = re.compile(r'^\|\s*决策分项\s*\|\s*关系\s*\|\s*回看\s*\|\s*$')
REVIEW = re.compile(r'^(20\d\d-\d\d-\d\d)\s*(改了|不受影响：\s*\S.{3,})')
DATE_HEAD = re.compile(r'^### (20\d\d-\d\d-\d\d)')


def read(path):
    return open(path, encoding='utf-8').read()


def history_start(lines):
    for index, line in enumerate(lines):
        if re.match(r'^## 历史版本\s*$', line):
            return index
    return len(lines)


def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def split_cells(line):
    return [cell.strip() for cell in re.sub(r'\\\|', '\x00', line.strip()).strip('|').split('|')]


def parse_decision_ref(cell):
    """「D28（挂载期承诺量） 已定项 2」→ (28, ('已定项', 2))；只写决策 → (28, None)；认不出 → None。"""
    head = re.match(r'^\**D(\d+)（', cell)
    if not head:
        return None
    item = re.search(r'(已定项|未定项)\s*(\d+)\s*\**$', cell)
    return int(head.group(1)), ((item.group(1), int(item.group(2))) if item else None)


def section_range(lines, heading_pattern, stop_level):
    """标题行号与它那一节的结束行号（下一个级别 ≤ stop_level 的标题之前），找不到返回 None。"""
    for index, line in enumerate(lines):
        if re.match(heading_pattern, line):
            for end in range(index + 1, len(lines)):
                level = re.match(r'^(#{1,6}) ', lines[end])
                if level and len(level.group(1)) <= stop_level:
                    return index, end
            return index, len(lines)
    return None


def table_rows(lines, start, end):
    """一节里那张影响的决策表的行：[(行号, 单元格)]；没有表头返回 None。"""
    rows, seen_head = [], False
    for index in range(start + 1, end):
        line = lines[index]
        if TABLE_HEAD.match(line.strip()):
            seen_head = True
            continue
        if seen_head and re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', line.strip()):
            continue
        if seen_head and line.strip().startswith('|'):
            rows.append((index, split_cells(line)))
        elif seen_head:
            break
    return rows if seen_head else None


problems = []   # (类别, 说明)


def bad(kind, text):
    problems.append((kind, text))


# ── 决策：分项与各分项的依据段 ──
decisions = {}
for path in sorted(glob.glob(os.path.join(kb, 'decisions', '*.md'))):
    lines = read(path).split('\n')
    body = lines[:history_start(lines)]
    head = next((re.match(r'^## D(\d+) ', line) for line in body if re.match(r'^## D(\d+) ', line)), None)
    if not head:
        continue
    number = int(head.group(1))
    items, basis, shapes, index_cells = set(), {}, [], []
    for index, line in enumerate(body):
        item = re.match(r'^(#{3,4}) (已定项|未定项) (\d+)', line)
        if not item:
            continue
        key = (item.group(2), int(item.group(3)))
        items.add(key)
        cited, in_basis, labels, basis_text = set(), False, set(), ''
        for inner in body[index + 1:]:
            level = re.match(r'^(#{1,6}) ', inner)
            if level and len(level.group(1)) <= len(item.group(1)):
                break
            label = re.match(r'^\*\*(定案|射程|依据|欠)\*\*：', inner)
            if label:
                labels.add(label.group(1))
            if inner.startswith('**依据**'):
                in_basis = True
            elif re.match(r'^\*\*[^*]{1,12}\*\*：', inner) or level:
                in_basis = False
            if in_basis:
                cited |= {int(e) for e in E_REF.findall(inner)}
                basis_text += inner + '\n'
        basis[key] = cited
        shapes.append((key, line, labels, cited, basis_text))
    for section in ('已定项', '未定项'):
        found = section_range(body, r'^### %s\s*$' % section, 3)
        if found:
            for line in body[found[0] + 1:found[1]]:
                if line.startswith('#### '):
                    break
                row = re.match(r'^\|\s*(\d+)\s*\|', line)
                if row:
                    items.add((section, int(row.group(1))))
                    index_cells.append((section, int(row.group(1)), split_cells(line)[-1]))
                # 没瘦身的决策有的用编号列表列分项（`9. **名字**…`），20 号数分项时也认这种写法
                listed = re.match(r'^(\d+)\.\s', line)
                if listed:
                    items.add((section, int(listed.group(1))))
    title = next(line for line in body if re.match(r'^## D(\d+) ', line))
    decisions[number] = {'path': os.path.relpath(path, root), 'items': items, 'basis': basis,
                         'shapes': shapes, 'index_cells': index_cells, 'title': title}

# ── 待回填清单 ──
pending_path = os.path.join(root, '.claude', 'decision-links-pending')
pending_rel = '.claude/decision-links-pending'
pending = set()
pending_lines = read(pending_path).split('\n') if os.path.exists(pending_path) else []
for number, line in enumerate(pending_lines, 1):
    if not line.strip() or line.lstrip().startswith('#'):
        continue
    entry = re.match(r'^(E|D)(\d+)\s+#\s*(\S.{3,})$', line.strip())
    if not entry:
        bad('待回填', f'{pending_rel}:{number}：写成「E<号>  # 理由」或「D<号>  # 理由」，理由至少四个字——「{line.strip()[:30]}」')
        continue
    pending.add((entry.group(1), int(entry.group(2))))
if in_git and os.path.exists(pending_path):
    previous = git('show', f'{base}:{pending_rel}')
    if previous.returncode == 0:
        before = set()
        for line in previous.stdout.split('\n'):
            entry = re.match(r'^(E|D)(\d+)\s', line.strip())
            if entry:
                before.add((entry.group(1), int(entry.group(2))))
        for kind, number in sorted(pending - before):
            bad('待回填', f'{pending_rel} 新加了 {kind}{number}：清单只减不增，新写的实验页与决策当场登记，不进清单')

# ── 瘦身形态与「有实验才有决策」（待回填清单之外的决策） ──
DATE = re.compile(r'20\d\d-\d\d-\d\d')
slim_items = no_experiment_items = 0
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    where = info['path']
    # 半定 / 待定的决策，20 号要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二三四五六七八九十]+[项条]未定）)?\s*$', info['title']):
        bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
    for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
        if DATE.search(heading):
            bad('瘦身形态', f'{where}：D{decision} {kind} {item_number} 的标题带日期，日期与来历进变更史')
        if kind != '已定项':
            continue
        slim_items += 1
        for label in ('定案', '射程', '依据', '欠'):
            if label not in labels:
                bad('瘦身形态', f'{where}：D{decision} 已定项 {item_number} 缺「**{label}**：」一块')
        if '依据' in labels and not cited:
            if re.search(r'无实验：\s*\S.{7,}', basis_text):
                no_experiment_items += 1
            else:
                bad('没有实验', f'{where}：D{decision} 已定项 {item_number} 的依据一个实验都没引——有实验才有决策；实在没有可量的，写「无实验：理由」')
    for kind, item_number, cell in info['index_cells']:
        # 「**状态：已定。**」是 20 号要的规范标记（检索端出来的是这一行），不算在一句话的字数里；
        # 31 号要写在未定项登记行里的两句规范标记（「改第一个事务的字节：…」与「动不动格式：…」）同理不算——
        # 31 号明写它要在登记行里（写在表下那一段里它定位不到），而它必须带日期，与这一条的「不带日期、
        # 不超过 100 字」直接相撞：2026-09-20 给 D2 未定项 21 与 D26 未定项 8 / 9 / 10 补上字节判决之后四行一起红。
        # 两句都剥：31 号是两把尺，一句量「这一版写不写出不同字节」、一句量「将来动不动格式」，
        # 缺哪一句它都判红，而两句都必须带日期 ⇒ 两句都不算进这一条的「不带日期、不超过 100 字」。
        without_marks = cell
        for mark in ('改第一个事务的字节', '动不动格式'):
            without_marks = re.sub(mark + r'：[^（(]*[（(][^）)]*[）)]', '', without_marks)
        plain = re.sub(r'\*\*|`', '', re.sub(r'\*\*状态：[^*]*\*\*', '', without_marks)).strip()
        if DATE.search(plain) or len(plain) > 100:
            bad('瘦身形态', f'{where}：D{decision} {kind}索引表第 {item_number} 行的定案格要一句话、不带日期、不超过 100 字（现在 {len(plain)} 字）')

# ── 实验最近一次变动的日期：页内历史节 + experiments-history.md ──
central_dates = {}
central_path = os.path.join(kb, 'experiments-history.md')
if os.path.exists(central_path):
    lines = read(central_path).split('\n')
    start = history_start(lines)
    current_date, heading_refs, body_refs = None, set(), set()

    def flush():
        refs = heading_refs or body_refs
        for experiment in refs:
            if current_date and current_date > central_dates.get(experiment, ''):
                central_dates[experiment] = current_date

    for line in lines[start:]:
        dated = DATE_HEAD.match(line)
        if dated or line.startswith('### ') or line.startswith('## '):
            flush()
            current_date = dated.group(1) if dated else None
            heading_refs = {int(e) for e in E_REF.findall(line)} if dated else set()
            body_refs = set()
            continue
        body_refs |= {int(e) for e in E_REF.findall(line)}
    flush()

# ── 实验页 ──
experiments, rows_total = {}, 0
for path in sorted(glob.glob(os.path.join(kb, 'experiments', '*.md'))):
    rel = os.path.relpath(path, root)
    lines = read(path).split('\n')
    head_line = next((line for line in lines if re.match(r'^## E(\d+) ', line)), None)
    if not head_line:
        continue
    number = int(re.match(r'^## E(\d+) ', head_line).group(1))
    voided = re.search(r'作废|退役', head_line) is not None
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    rows = table_rows(lines, section[0], section[1]) if section else None
    outside = [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]
    mentioned = {int(d) for d in D_REF.findall('\n'.join(outside))}
    local_dates = [m.group(1) for m in (DATE_HEAD.match(line) for line in lines[hist:]) if m]
    latest = max(local_dates + ([central_dates[number]] if number in central_dates else []), default='')
    is_pending = ('E', number) in pending
    experiments[number] = {'rel': rel, 'lines': lines, 'hist': hist, 'section': section, 'rows': rows or []}
    if rows is None and not is_pending:
        bad('没有表', f'{rel}：E{number} 没有「### 影响的决策」表（表头 | 决策分项 | 关系 | 回看 |）')
        continue
    covered = set()
    for index, cells in rows or []:
        rows_total += 1
        where = f'{rel}:{index + 1}'
        if len(cells) != 3:
            bad('行形状', f'{where}：影响的决策表一行要三格，这一行 {len(cells)} 格')
            continue
        ref = parse_decision_ref(cells[0])
        if not ref:
            bad('行形状', f'{where}：第一格要写「D<号>（简称）」或「D<号>（简称） 已定项 k」——「{cells[0][:30]}」')
            continue
        decision, item = ref
        covered.add(decision)
        if decision not in decisions:
            bad('指不到', f'{where}：D{decision} 在 decisions/ 下没有正文')
            continue
        if item and item not in decisions[decision]['items']:
            bad('指不到', f'{where}：D{decision} 没有{item[0]} {item[1]}')
        relation = cells[1].strip('* ')
        if relation not in RELATIONS:
            bad('行形状', f'{where}：关系写「{relation[:12]}」，只认 支撑 / 推翻 / 备料 / 不影响')
            continue
        if relation in BASIS_RELATIONS and item is None and decisions[decision]['items']:
            bad('行形状', f'{where}：关系是{relation}就要写到分项（D{decision} 有分项）')
        review = REVIEW.match(cells[2].strip('* '))
        if not review:
            bad('行形状', f'{where}：回看格写「YYYY-MM-DD 改了」或「YYYY-MM-DD 不受影响：理由」——「{cells[2][:30]}」')
            continue
        if latest and review.group(1) < latest:
            bad('回看过期', f'{where}：E{number} 最近一次变动在 {latest}，对 D{decision} 的回看停在 {review.group(1)}')
        if relation in BASIS_RELATIONS and item and ('D', decision) not in pending \
                and number not in decisions[decision]['basis'].get(item, set()):
            bad('不对称', f'{where}：E{number} 说它{relation} D{decision} {item[0]} {item[1]}，而那条分项的「**依据**」段没引 E{number}')
    if not is_pending and not voided and rows is not None \
            and not any(len(cells) == 3 and cells[1].strip('* ') in ('支撑', '推翻', '备料') for _, cells in rows):
        bad('没对应决策', f'{rel}：E{number} 的影响的决策表里没有一行是支撑 / 推翻 / 备料——实验必须对应决策（结论作废或退役的实验在标题里写明，就不判这一条）')
    if not is_pending:
        for decision in sorted(mentioned - covered):
            bad('没登记', f'{rel}：E{number} 正文提到 D{decision}，影响的决策表里没有它那一行')

# ── 决策这一侧：依据引的实验，实验页里要有支撑或推翻那一行 ──
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    for item, cited in sorted(info['basis'].items()):
        for experiment in sorted(cited):
            page = experiments.get(experiment)
            if not page:
                bad('指不到', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，kb 里没有这个实验页")
                continue
            ok = any(parse_decision_ref(cells[0]) == (decision, item) and cells[1].strip('* ') in BASIS_RELATIONS
                     for _, cells in page['rows'] if len(cells) == 3)
            if not ok:
                bad('不对称', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，{page['rel']} 的影响的决策表里没有「支撑 / 推翻」这一行")
for kind, number in sorted(pending):
    exists = number in (experiments if kind == 'E' else decisions)
    if not exists:
        bad('待回填', f'{pending_rel} 里的 {kind}{number} 在 kb 里没有正文')

# ── 按这次改动 ──
triggered = 0


def added_lines(rel):
    """这次改动里 rel 新加的行：[(新文件里的行号, 内容)]；未跟踪的文件整份都算。"""
    if git('ls-files', '--error-unmatch', rel).returncode != 0:
        return [(index, line) for index, line in enumerate(read(os.path.join(root, rel)).split('\n'))]
    diff = git('diff', '-U0', base, '--', rel).stdout
    result, new_line = [], 0
    for line in diff.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', line)
        if hunk:
            new_line = int(hunk.group(1)) - 1
            count = int(hunk.group(2)) if hunk.group(2) is not None else 1
            if count == 0:
                result.append((new_line, None))   # 纯删除：记它在新文件里的位置
            continue
        if line.startswith('+') and not line.startswith('+++'):
            result.append((new_line, line[1:]))
            new_line += 1
    return result


def check_changed_rows(rel, row_lines):
    for text in row_lines:
        cells = split_cells(text)
        if len(cells) != 3:
            continue
        ref = parse_decision_ref(cells[0])
        review = REVIEW.match(cells[2].strip('* '))
        if ref and review and review.group(2) == '改了' and ref[0] in decisions \
                and decisions[ref[0]]['path'] not in changed:
            bad('改了没改', f"{rel}：这次写了「改了」D{ref[0]}，而 {decisions[ref[0]]['path']} 不在这次改动里")


if in_git:
    result_hits = {}
    for rel in changed:
        match = re.match(r'^research/results/e(\d+)[^0-9]', rel)
        if match:
            result_hits.setdefault(int(match.group(1)), rel)
    for number, page in sorted(experiments.items()):
        rel = page['rel']
        body_changed = False
        added = added_lines(rel) if rel in changed and os.path.exists(os.path.join(root, rel)) else []
        section = page['section']
        for line_number, _ in added:
            inside_table = section and section[0] <= line_number < section[1]
            if line_number < page['hist'] and not inside_table:
                body_changed = True
        table_touched = [text for line_number, text in added
                         if text is not None and section and section[0] <= line_number < section[1]
                         and text.strip().startswith('|') and not TABLE_HEAD.match(text.strip())
                         and not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', text.strip())]
        check_changed_rows(rel, table_touched)
        reasons = []
        if body_changed:
            reasons.append('正文改了')
        if number in result_hits:
            reasons.append(f'产物 {result_hits[number]} 变了')
        if reasons:
            triggered += 1
            if not table_touched:
                bad('没回看', f"{rel}：这次改动里 E{number} {'、'.join(reasons)}，影响的决策表一行都没回看")
    for rel in sorted(changed):
        if not re.match(r'^research/prompts/[^/]+-main-verification\.md$', rel) or not os.path.exists(os.path.join(root, rel)):
            continue
        if git('cat-file', '-e', f'{base}:{rel}').returncode == 0:
            continue
        triggered += 1
        lines = read(os.path.join(root, rel)).split('\n')
        section = section_range(lines, r'^## 回看决策\s*$', 2)
        if not section:
            bad('没回看', f'{rel}：新写的三方判决没有「## 回看决策」一节')
            continue
        rows = table_rows(lines, section[0], section[1])
        waived = any(re.match(r'^不涉及决策：\s*\S.{3,}', line.strip()) for line in lines[section[0] + 1:section[1]])
        if not rows and not waived:
            bad('没回看', f'{rel}：「## 回看决策」里既没有表行，也没有一行「不涉及决策：理由」')
            continue
        for index, cells in rows or []:
            ref = parse_decision_ref(cells[0]) if len(cells) == 3 else None
            if not ref or not REVIEW.match(cells[2].strip('* ')):
                bad('行形状', f'{rel}:{index + 1}：回看决策表一行写成 | D<号>（简称） [已定项 k] | 关系 | YYYY-MM-DD 改了 / 不受影响：理由 |')
        check_changed_rows(rel, [lines[index] for index, _ in rows or []])

pending_experiments = sum(1 for kind, _ in pending if kind == 'E')
pending_decisions = sum(1 for kind, _ in pending if kind == 'D')
summary = (f'实验页 {len(experiments)} 个（待回填 {pending_experiments}）、决策 {len(decisions)} 条（待回填 {pending_decisions}）、'
           f'已瘦身决策的已定项 {slim_items} 个（其中写「无实验」的 {no_experiment_items} 个）、'
           f'表行 {rows_total} 行、这次改动触发回看 {triggered} 处')
if problems:
    kinds = {}
    for kind, _ in problems:
        kinds[kind] = kinds.get(kind, 0) + 1
    print(f"  ✗ 决策与实验的登记有 {len(problems)} 处问题（{'、'.join(f'{k} {v}' for k, v in kinds.items())}）；{summary}：")  # gate-lint:summary
    for kind, text in problems:
        print(f'      [{kind}] {text}')  # gate-lint:detail
    print('  → 怎么办：格式与判据见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。')
    print('            实验出了新结论、换了产物，就逐行回看它影响的决策，回看格写当天日期与「改了」或「不受影响：理由」；')
    print('            支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。')
    sys.exit(1)
print(f'  ✓ 决策与实验双向登记对得上、回看不过期（{summary}）')
PY
```

