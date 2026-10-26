#!/usr/bin/env bash
# 并行跑一个 S 下的各格（每格一个单线程进程），每格 225 秒封顶；输出写 $OUT/S<S>-<格>.out，打每格退出码。
# 用法：run-cells.sh <S> <测试二进制> <输出目录> <格名…>
set -u
TAG="${TAG:-}"
S="$1"; BIN="$2"; OUT="$3"; shift 3
mkdir -p "$OUT"
rm -f "$OUT"/S"$S""$TAG"-*.exit
for cell in "$@"; do
  { I718_S="$S" timeout 225 nice -n 19 "$BIN" --test-threads 1 --nocapture "$cell" > "$OUT/S$S$TAG-$cell.out" 2>&1; echo "$?" > "$OUT/S$S$TAG-$cell.exit"; } &
done
wait
for cell in "$@"; do echo "S=$S cell=$cell exit=$(cat "$OUT/S$S$TAG-$cell.exit" 2>/dev/null) rows=$(grep -c '^ROW' "$OUT/S$S$TAG-$cell.out") scans=$(grep -c '^SCAN ' "$OUT/S$S$TAG-$cell.out")"; done
