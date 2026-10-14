#!/usr/bin/env bash
# Y6：集成副本复核产物里的计数，只读、只数。用法：summary_counts.sh <集成副本工作根>（含 verifyN.summary、verifyN-out/）
set -u
root="${1:?给集成副本工作根}"
for s in verify4 verify6 verify8 verify10; do
  f="$root/$s.summary"
  rows=$(grep -cE '^ *[0-9]+ ' "$f"); bad=$(grep -c '错（' "$f"); last=$(tail -n 1 "$f")
  echo "name=summary file=$s.summary rows=$rows bad_rows=$bad last_line=$last"
  grep -E '错（' "$f" | awk '{printf "name=summary_bad file=%s index=%s item=%s expected=%s\n", "'"$s"'", $1, $2, $4}'
done
# 同名门禁开关项序号（records 引的 156-158、158-162）
for pair in verify8:long-run-exempt verify10:long-run-exempt; do
  s="${pair%%:*}"
  echo "name=summary_code_tooling_exempt file=$s.summary indexes=$(grep -E 'brk-code-tooling-long-run-exempt' "$root/$s.summary" | awk '{printf "%s ", $1}')"
done
# verify7/9 弄坏开关日志数
echo "name=out_logs dir=verify7-out thread_default=$(ls "$root"/verify7-out/brk-thread-default-*.log | wc -l) long_run_exempt=$(ls "$root"/verify7-out/brk-long-run-*.log | wc -l)"
echo "name=out_logs dir=verify9-out long_run_exempt=$(ls "$root"/verify9-out/brk-*.log | wc -l)"
for f in "$root"/verify7-out/brk-*.log "$root"/verify9-out/brk-*.log; do
  echo "name=out_red file=${f#"$root"/} fail_marks=$(grep -c '✗' "$f")"
done
# verify11：两个新开关各红几格、四份自证行
echo "name=verify11 rdg-dup_fail_marks=$(grep -c '✗ 自检' "$root/verify11-out/rdg-dup.log") rdg-cb_fail_marks=$(grep -c '✗ 自检' "$root/verify11-out/rdg-cb.log")"
for f in rdg dp hostd detached; do echo "name=verify11_selftest file=$f.log line=$(grep -E '✓ ' "$root/verify11-out/$f.log" | tail -n 1 | cut -c1-70)"; done
echo "name=verify11_stage line=$(grep -E '点名的' "$root/verify11-out/stage-ct.log" | tail -n 1 | cut -c1-60)"
for v in verify7-out/cell.log verify9-out/cells.log verify11-out/cells.log; do
  echo "name=cell_line file=$v $(grep -E '扫了' "$root/$v" | sed -E 's/.*登记值 ([0-9]+)；扫了 ([0-9]+) 份.*可疑写法 ([0-9]+) 处.*（表 ([0-9]+) 行.*线程变量 ([0-9]+) 个.*/registered=\1 scanned=\2 suspicious=\3 table_rows=\4 own_variables=\5/')"
done
# 批 7、批 8 的真红：verify8/verify10 的 cells-true-tree 日志
echo "name=true_tree_red verify8_fail_lines=$(grep -c '^     第 .* 行的锚串' "$root/verify8-out/015-cells-true-tree.log") verify10_fail_lines=$(grep -cE '^     (\.claude|scripts)/.*:[0-9]+  available_threads' "$root/verify10-out/015-cells-true-tree.log")"
