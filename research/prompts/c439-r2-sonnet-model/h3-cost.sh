#!/usr/bin/env bash
# C439 第二轮辩方腿 K6 的 H3 一格：第一轮判决第二节 H3 那一格两种代价口径（退出暂存口径、留在暂存口径），在庚二（geng2）下各量一遍，今天（today）、丙（bing）、乙（门禁写 yi_gate_mb）、丁（ding_tree_fixed）作对照。
#   退出暂存口径  h16_unstaged_other：别的会话先把那行退出暂存再跑绿——写行的会话暂存好行并跑绿（行进窗口一次），另一个会话把它退出暂存、在不含它的暂存区上跑绿，写行的会话重新暂存它。
#   留在暂存口径  h14_rejudge_cost  ：那行留在暂存里被判过——另一个会话暂存好行、gate-staged 跑绿、提交。
#   附            h19_index_moves   ：写行的会话跑的当中，另一个会话往暂存区放了别的东西（庚二前移前比不比暂存树）。
# 用法：bash h3-cost.sh <冻结副本> <工作目录（新建）>
# 打出：一、固定续作 S / SS / kSS 的逐趟日志与 result 行（原样）；二、字母表 Sck、长度 0..4 的全部守规矩续作上，被追踪的那行在前缀之后进窗口的次数（重判次数）的合计 / 最大 / 至少一次的续作条数。
set -uo pipefail
FROZEN="$1"; OUT="$2"; mkdir -p "$OUT" || exit 2
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CANDS="today bing geng2 geng2_nc yi_gate_mb ding_tree_fixed"
echo "== 一、固定续作的逐趟日志（原样）"
for h in h16_unstaged_other h14_rejudge_cost; do
  for c in today geng2; do
    for k in S SS; do
      result="$(bash "$MODEL_DIR/c439-model.sh" "$FROZEN" "$OUT/fixed/$h.$c.$k" "$c" "$h" "$k" | tail -1)"
      echo "=== hist=$h cand=$c cont=$k"
      grep -E '^(run=|另一个会话把|写行的会话重新|commit|  staged-green)' "$OUT/fixed/$h.$c.$k/log" | sed 's/^/    /'
      echo "$result"
    done
  done
done
for c in geng2 geng2_nc; do
  k=SS; h=h19_index_moves
  result="$(bash "$MODEL_DIR/c439-model.sh" "$FROZEN" "$OUT/fixed/$h.$c.$k" "$c" "$h" "$k" | tail -1)"
  echo "=== hist=$h cand=$c cont=$k"
  grep -E '^(run=|（交错）|commit|  staged-green)' "$OUT/fixed/$h.$c.$k/log" | sed 's/^/    /'
  echo "$result"
done
echo "== 二、字母表 Sck、长度 0..4 的守规矩续作上的重判次数（每格：合计/单条续作最大/至少重判一次的续作条数/守规矩续作条数）"
for h in h16_unstaged_other h14_rejudge_cost h19_index_moves; do
  # shellcheck disable=SC2086
  bash "$MODEL_DIR/sweep.sh" "$FROZEN" "$OUT/sweep/$h" "$h" 4 "${SINGLEFS_THREAD_CAP:-4}" Sck $CANDS > /dev/null
done
python3 - "$OUT"/sweep/*/results.txt <<'PY'
import collections, sys
rows = collections.defaultdict(list)
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        if line.startswith('result '):
            f = dict(p.split('=', 1) for p in line.split()[1:] if '=' in p); rows[(f['hist'], f['cand'])].append(f)
def disciplined(c):   # 每次提交（c、k）紧跟在一次门禁（S）之后；前缀不以门禁收尾，开头的提交不算守规矩
    return c != 'none' and 'S' in c and all(c[i] not in 'ck' or (i > 0 and c[i-1] == 'S') for i in range(len(c)))
for hist in sorted({h for h, _ in rows}):
    cells = []
    for cand in "today bing geng2 geng2_nc yi_gate_mb ding_tree_fixed".split():
        d = [int(f['after_count']) for f in rows[(hist, cand)] if disciplined(f['cont'])]
        cells.append(f"{cand}={sum(d)}/{max(d)}/{sum(1 for x in d if x)}/{len(d)}")
    print(f"name=h3cost hist={hist} " + ' '.join(cells))
PY
