#!/usr/bin/env bash
# 辩方腿的批量扫描：给定历史名单，对 16 个候选各扫一遍长度 0..<最长> 的全部续作（字母表 NSckoG，有上游的历史另加 p）。
# 用法：bash sweep-all.sh <冻结副本> <输出目录> <最长> <历史>…
# 输出：<输出目录>/<历史>/results.txt；全部历史跑完后用 `python3 summarize.py <输出目录>/*/results.txt` 合表。
set -uo pipefail
FROZEN="$1"; OUT="$2"; MAXLEN="$3"; shift 3
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CANDS="today bing bing_guard geng geng2 geng2_nc geng2_s ding_tree ding_tree_fixed ding_lazy_fixed yi_gate_mb yi_clean wu_set wu_count wu_fixed wu_cf"
for h in "$@"; do
  a=NSckoG; [[ "$h" == h5_pushed || "$h" == h2u_unpushed ]] && a=NSckoGp
  # shellcheck disable=SC2086
  bash "$MODEL_DIR/sweep.sh" "$FROZEN" "$OUT/$h" "$h" "$MAXLEN" "${SINGLEFS_THREAD_CAP:-4}" "$a" $CANDS | sed "s/^/hist=$h /"
done
