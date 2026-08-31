#!/usr/bin/env bash
# E56 的跑批。三组配置，一次跑完：
#   1. 测量：5 个种子 × 缓存 66 个节点（约占树 6.4%）
#   2. 自检：把两条攒批臂的核心判据换成常量（缓冲=1），1 个种子 —— 结构性检查，不是测量
#   3. 缓存敏感性：缓存 264 / 1057 个节点，1 个种子
#      ⚠️ 第 3 组是 2026-08-31 两条独立论证腿都指出来的那个混淆：
#      点查那一列在高 ε 上贵起来，是「内部节点数逼近缓存容量」造成的，
#      不是消息缓冲本身让单次查询变贵。不扫缓存就分不开这两件事。
#
#   bash research/scripts/e56-sweep.sh <输出文件>
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 每一轮的收尾行必须是 `name=done emitted=N`，且 N 必须等于该轮实际 E7RESULT 行数；
#   3. 任一轮退出码非 0 或条数对不上，整脚本判红，不许当成「跑过了」。
set -uo pipefail
cd "$(dirname "$0")/.."
OUT="${1:?用法：e56-sweep.sh <输出文件>}"
BIN=./target/release/e56_epsilon
[[ -x "$BIN" ]] || { echo "先构建：cargo build --release --bin e56_epsilon"; exit 2; }

# 镜像一律落临时目录，不许进仓（command-safety.md）
WORK="${E56_WORK:-${TMPDIR:-/tmp}/singlefs-e56}"
mkdir -p "$WORK"
IMG="$WORK/e56.img"
truncate -s 512M "$IMG"   # 8192 叶 × ε=0.95 的几何要约 143 MB

# 模式 种子 缓存节点数 叶数
#
# 第 3 组（叶数 4096 / 8192）是 2026-08-31 两条独立论证腿都点名的那个外推威胁：
# 「树只有 1024 个叶、缓存占 6.4%」。缓存预算不变而树长大 4 倍 / 8 倍，
# 缓存占比从 6.4% 掉到 1.6% / 0.8% —— 收益站不站得住，量出来而不是让步。
CONFIGS="
none 101 66 1024
none 202 66 1024
none 303 66 1024
none 404 66 1024
none 505 66 1024
none 101 66 8192
none 202 66 8192
selfcheck 101 66 1024
none 101 264 1024
none 101 1057 1024
none 101 66 4096
"

rm -f "$OUT"                      # 闸 1：跑前删本轮输出
: > "$OUT"
fail=0
while read -r mode seed cache nleaf; do
  [[ -z "$mode" ]] && continue
  tmp="$WORK/round-$mode-$seed-c$cache-l$nleaf.out"
  rm -f "$tmp"
  echo "=== mode=$mode seed=$seed cache=$cache nleaf=$nleaf ===" >> "$OUT"
  if ! "$BIN" "$IMG" "$seed" "$mode" "$cache" "$nleaf" > "$tmp" 2>"$tmp.err"; then
    echo "!! mode=$mode seed=$seed cache=$cache nleaf=$nleaf 退出码非 0" >> "$OUT"; cat "$tmp.err" >> "$OUT"; fail=1; continue
  fi
  got=$(grep -c '^E7RESULT ' "$tmp")
  want=$(sed -n 's/^E7RESULT name=done emitted=\([0-9]*\)$/\1/p' "$tmp" | tail -1)
  if [[ -z "$want" || "$got" != "$want" ]]; then
    echo "!! mode=$mode seed=$seed cache=$cache nleaf=$nleaf 条数对不上：抓到 $got，收尾行说 ${want:-无}" >> "$OUT"; fail=1
  fi
  cat "$tmp" >> "$OUT"
done <<< "$CONFIGS"
if (( fail )); then echo "E56 跑批判红：有轮次失败或条数对不上，见 $OUT" >&2; exit 1; fi
echo "E56 跑批完成：$OUT"
