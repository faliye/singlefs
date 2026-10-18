#!/usr/bin/env bash
# 放开用户动作扫一遍：一段历史（固定前缀与故障）× 每个候选 × 续作序列（字母表里长度 0..最长 的全部序列），并行跑 c439-model.sh。
# 用法：bash sweep.sh <冻结副本> <输出目录（新建）> <历史> <最长> <并行数> [<字母表，默认 NScko>] [<候选…>]
# 输出：<输出目录>/results.txt，一行一个 result（c439-model.sh 的最后一行），每行末尾补 rc=<退出码>。
set -uo pipefail
FROZEN="$1"; OUT="$2"; HIST="$3"; MAXLEN="$4"; JOBS="$5"; ALPHABET="${6:-NScko}"; shift 6 2>/dev/null || shift $#
CANDIDATES=("$@")
((${#CANDIDATES[@]})) || CANDIDATES=(today jia yi_shared yi_session yi_gate_mb yi_gate_h1 bing ding_a ding_rw ding_tree wu_set wu_count wu_fixed)
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$OUT/runs"
python3 - "$ALPHABET" "$MAXLEN" "${CANDIDATES[@]}" > "$OUT/jobs.txt" <<'PY'
import itertools, sys
alphabet, maxlen, candidates = sys.argv[1], int(sys.argv[2]), sys.argv[3:]
for cand in candidates:
    for length in range(maxlen + 1):
        for seq in itertools.product(alphabet, repeat=length):
            print(cand, ''.join(seq) or '-')
PY
export FROZEN OUT HIST MODEL_DIR
xargs -P "$JOBS" -L 1 bash -c 'set -o pipefail
  cand="$1"; cont="$2"; [[ "$cont" == - ]] && cont=""
  line="$(bash "$MODEL_DIR/c439-model.sh" "$FROZEN" "$OUT/runs/$cand.${cont:-none}" "$cand" "$HIST" "$cont" | tail -1)"; rc=$?
  printf "%s rc=%s\n" "$line" "$rc"
' _ < "$OUT/jobs.txt" > "$OUT/results.txt"
echo "jobs=$(wc -l < "$OUT/jobs.txt") results=$(wc -l < "$OUT/results.txt")"
