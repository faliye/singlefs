#!/usr/bin/env bash
# C439 第二轮辩方腿 K6：第一轮判决里出局的丙、戊、乙（门禁写、退 merge-base）、丁（按树挂）各自被打中的形，
# 与「改成什么之后它就是庚二或 wu_cf」的那一版，在同一段历史、同一个续作上各跑一遍。
# 用法：bash k6-examples.sh <冻结副本> <工作目录（新建）>
# 每个例子打出：=== example=<名> ...、缩进的整段日志、result 行（judged_ever=no 即「那一行一次都没进判的行」，条款 2 的字面）。
# 末尾 name=k6row 一行一例，便于对表。引擎是同目录的 c439-model.sh（派生自 c439-r1-opus-model，加了 r2 标记的几处）。
set -uo pipefail
FROZEN="$1"; OUT="$2"; mkdir -p "$OUT" || exit 2
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROWS="$OUT/rows.txt"; : > "$ROWS"
run() {   # run <编号> <候选> <历史> <续作>
  local id="$1" cand="$2" hist="$3" cont="$4" result
  result="$(bash "$MODEL_DIR/c439-model.sh" "$FROZEN" "$OUT/$id" "$cand" "$hist" "$cont" | tail -1)"
  echo "=== example=$id cand=$cand hist=$hist cont=$cont"
  sed 's/^/    /' "$OUT/$id/log"
  echo "$result"
  echo "name=k6row id=$id cand=$cand hist=$hist cont=$cont judged_ever=$(sed -n 's/.* judged_ever=\([a-z]*\) .*/\1/p' <<<"$result")" >> "$ROWS"
}
# ── 丙（bing）：交错三形与人给 GATE_BASE 一形；修成庚二（geng2）、丁的两个修后变体、乙的修后变体各跑一遍 ──
for c in bing geng2 ding_tree_fixed ding_lazy_fixed; do
  run "$c-race-unstage"      "$c" h9_aba          S
  run "$c-race-stash"        "$c" h9_stash        S
  run "$c-race-commit-undo"  "$c" h9_commit_undo  S
  run "$c-override"          "$c" h15_override    kS
done
# ── 丙修成庚二要补两处，缺一处都不行：geng 只补「里层判 write-tree 的树」，bing_guard 只补「人给 GATE_BASE 不前移」 ──
run geng-race-unstage        geng        h9_aba          S
run geng-override            geng        h15_override    kS
run bing_guard-race-unstage  bing_guard  h9_aba          S
run bing_guard-override      bing_guard  h15_override    kS
# ── 丁（按树挂）：原样 ding_tree 在两个形上被打中 ──
run ding_tree-race-unstage   ding_tree h9_aba         kS
run ding_tree-override       ding_tree h15_override   kS
# ── 乙（门禁写、退 merge-base）：记号挂在 HEAD 上的三形；修成「记号挂被判的树」= 庚二，另跑乙的「只在干净轮写」变体 ──
for c in yi_gate_mb yi_clean geng2 ding_tree_fixed; do
  run "$c-stamp-S"       "$c" h6_stamp_S       S
  run "$c-stamp-N"       "$c" h6_stamp_N       S
  run "$c-stamp-revert"  "$c" h6_stamp_revert  S
done
run yi_clean-race-unstage    yi_clean h9_aba        S
# ── 戊：集合键、比 HEAD 的各变体与 wu_cf ──
for c in wu_set wu_count wu_fixed wu_cf; do
  run "$c-duplicate"   "$c" h11_dup          S
  run "$c-grow"        "$c" h12_exempt_grow  S
  run "$c-move"        "$c" h17_wu_move      S
  run "$c-copy-other"  "$c" h18_wu_copy_other_page S
done
run geng2-move geng2 h17_wu_move S
run today-move today h17_wu_move S
# ── 庚二的射程：只改带 --staged 的一条路时，只跑不带 --staged 的整轮留下的形 ──
run geng2_s-n-only-bypass   geng2_s h2_noverify N
run geng2-n-only-bypass     geng2   h2_noverify N
run geng2_s-n-only-c439     geng2_s h1_c439     NN
run geng2-n-only-c439       geng2   h1_c439     NN
# ── 对照：错引文暂存着没提交，下一次 gate-staged 每个候选都该判到 ──
for c in today bing geng2 ding_tree_fixed ding_lazy_fixed yi_clean wu_cf wu_set geng2_s; do run "control-$c" "$c" h0_control S; done
echo "name=k6 count=$(wc -l < "$ROWS")"
cat "$ROWS"
