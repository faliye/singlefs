#!/usr/bin/env bash
# 报告里点名的每一个打中与对照，各从头造一次仓、打出整段日志与 result 行（提交时刻写死，输出逐字可复跑）。
# 用法：bash examples.sh <冻结副本> <工作目录（新建）>
set -uo pipefail
FROZEN="$1"; OUT="$2"; mkdir -p "$OUT" || exit 2
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
n=0
run() {   # run <编号> <候选> <历史> <续作>
  local id="$1" cand="$2" hist="$3" cont="$4" result
  n=$((n + 1))
  result="$(bash "$MODEL_DIR/c439-model.sh" "$FROZEN" "$OUT/$id" "$cand" "$hist" "$cont" | tail -1)"
  echo "=== example=$id cand=$cand hist=$hist cont=$cont"
  sed 's/^/    /' "$OUT/$id/log"
  echo "$result"
}
# 今天的取法（H1）
run today-c439-original      today      h1_c439          cS
run today-bypass             today      h2_noverify      cS
run today-bypass-pathN       today      h2_noverify      N
run today-split              today      h3_split         cS
run today-other-session      today      h4_other         cS
run today-pushed-pathN       today      h5_pushed        N
run today-pushed-commit      today      h5_pushed        cpS
run today-unpushed-caught    today      h2u_unpushed     S
# 候选（H2）
run jia-bypass               jia        h2_noverify      S
run yi-shared-overwrite      yi_shared  h13_shared_open  S
run yi-session-reopen        yi_session h4_other         oS
run yi-gate-h1-amend         yi_gate_h1 h8_amend         cS
run yi-gate-h1-revert        yi_gate_h1 h6_stamp_revert  S
run yi-gate-mb-stamp-S       yi_gate_mb h6_stamp_S       S
run yi-gate-mb-stamp-N       yi_gate_mb h6_stamp_N       S
run yi-gate-mb-stamp-revert  yi_gate_mb h6_stamp_revert  S
run yi-gate-mb-amend-caught  yi_gate_mb h8_amend         cS
run ding-a-stamp-S           ding_a     h6_stamp_S       S
run ding-rw-amend            ding_rw    h8_amend         S
run bing-race-unstage        bing       h9_aba           S
run bing-race-stash          bing       h9_stash         S
run bing-race-commit-undo    bing       h9_commit_undo   S
run bing-override            bing       h15_override     S
run bing-stamp-caught        bing       h6_stamp_S       S
run ding-tree-race           ding_tree  h9_aba           kS
run ding-tree-override       ding_tree  h15_override     kS
run wu-set-duplicate         wu_set     h11_dup          S
run wu-set-grow              wu_set     h12_exempt_grow  S
run wu-count-grow            wu_count   h12_exempt_grow  S
run wu-fixed-duplicate       wu_fixed   h11_dup          S
run wu-cf-duplicate-caught   wu_cf      h11_dup          S
run wu-cf-grow-caught        wu_cf      h12_exempt_grow  S
# 攻方腿自己提的两步改法（被攻过零轮）
run geng-race-caught         geng       h9_aba           S
run geng-override            geng       h15_override     S
run geng2-override-caught    geng2      h15_override     S
run geng2-stamp-caught       geng2      h6_stamp_S       S
# 对照：错引文暂存着，下一次 gate-staged 每个候选都判到
for cand in today jia yi_shared yi_session yi_gate_h1 yi_gate_mb bing ding_a ding_rw ding_tree wu_set wu_count wu_fixed wu_cf geng geng2; do
  run "control-$cand" "$cand" h0_control S
done
echo "name=examples count=$n"
