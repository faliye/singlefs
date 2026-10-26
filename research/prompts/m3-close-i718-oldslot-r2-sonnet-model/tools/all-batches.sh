#!/usr/bin/env bash
# 辩方腿各格全跑：S = 4、5、8、16 各一批，批内各格并行（每格一个单线程进程，225 秒封顶），批与批顺序。
# 用法：all-batches.sh <模型目录> <测试二进制> <输出目录>      整条经 scripts/hosts/run-with-memory-cap.sh 跑。
# RERUN_QUICK=1：只跑 S=4 的格 Y 与格 F（验装置能起，约一分钟）。
set -u
MODEL="$1"; BIN="$2"; OUT="$3"
mkdir -p "$OUT"
cells() { bash "$MODEL/tools/run-cells.sh" "$@"; }
if [[ "${RERUN_QUICK:-}" == 1 ]]; then
  cells 4 "$BIN" "$OUT" cell_y cell_f
  exit 0
fi
# S = 4：不少取点
{ cells 4 "$BIN" "$OUT" cell_a_sweep cell_h1 cell_h2 cell_f cell_f2_pinned_slot_whose_failure_was_in_the_first_revolution cell_g cell_r cell_y > "$OUT/batch-s4.txt" 2>&1; } &
{ I718_ROWS_FROM=318 cells 4 "$BIN" "$OUT" cell_w_journal_ring_wrapped > "$OUT/batch-s4-w.txt" 2>&1; } &
for base in 12 13 16 23; do
  { TAG=b$base I718_B_BASE=$base cells 4 "$BIN" "$OUT" cell_b > "$OUT/batch-s4-b$base.txt" 2>&1; } &
done
wait
# S = 5：不少取点
{ cells 5 "$BIN" "$OUT" cell_a_sweep cell_h1 cell_h2 cell_f cell_f2_pinned_slot_whose_failure_was_in_the_first_revolution cell_g cell_r cell_y > "$OUT/batch-s5.txt" 2>&1; } &
for base in 15 29; do
  { TAG=b$base I718_B_BASE=$base cells 5 "$BIN" "$OUT" cell_b > "$OUT/batch-s5-b$base.txt" 2>&1; } &
done
wait
# S = 8：少取点
{ I718_FEW=1 cells 8 "$BIN" "$OUT" cell_a_sweep cell_h1 cell_h2 cell_f cell_f2_pinned_slot_whose_failure_was_in_the_first_revolution cell_g cell_r cell_y > "$OUT/batch-s8.txt" 2>&1; } &
for base in 24 47; do
  { TAG=b$base I718_FEW=1 I718_B_BASE=$base cells 8 "$BIN" "$OUT" cell_b > "$OUT/batch-s8-b$base.txt" 2>&1; } &
done
wait
# S = 16：少取点；格 A、F、G、R 不扫描，另用 cell_a_scan_only 扫一个点
{ I718_FEW=1 cells 16 "$BIN" "$OUT" cell_h1 cell_h2 cell_y > "$OUT/batch-s16.txt" 2>&1; } &
{ I718_FEW=1 I718_NOSCAN=1 cells 16 "$BIN" "$OUT" cell_a_sweep cell_f cell_f2_pinned_slot_whose_failure_was_in_the_first_revolution cell_g cell_r > "$OUT/batch-s16-b.txt" 2>&1; } &
{ TAG=scan I718_FEW=1 I718_NOROWS=1 cells 16 "$BIN" "$OUT" cell_a_scan_only > "$OUT/batch-s16-scan.txt" 2>&1; } &
for base in 48 95; do
  { TAG=b$base I718_FEW=1 I718_B_BASE=$base I718_B_REPEATS=2 cells 16 "$BIN" "$OUT" cell_b > "$OUT/batch-s16-b$base.txt" 2>&1; } &
done
wait
cat "$OUT"/batch-*.txt
