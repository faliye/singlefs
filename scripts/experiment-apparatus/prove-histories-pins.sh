#!/usr/bin/env bash
# run-condition: 无（不要 GPU、不要第二台）。
# 证 E206（发布固定点轮数的代价）、E207（B8严格判据下的步数）、E209（尖峰成因）三份历史用例文件末尾钉绝对值的测试会红：
# 历史用例不是 scripts/experiment-apparatus 里的 bin，mutate.sh 够不着它；这里在插了读数的副本（plain 构建，一个字节不插）里起它，逐条改坏变异表里 pin_ 起头的行，读那几条测试红不红。
#
# 用法：
#   prove-histories-pins.sh <e206|e207|e209> --work <共享根下的新目录>
#       先照对应的 e<号>_*_counters.py 的 --build plain 在 <工作目录>/plain 建副本，把历史用例（e207 还有量窗取法与紧凑盘、e209 还有脚本表）拷进副本的 tests/，
#       起钉绝对值的测试一遍（基线要全绿），再对变异表 scripts/mutation/tables/e<号>_*_histories.tsv 里每一条 pin_ 起头的行：改坏历史用例的那一处、起同一批测试、还原。
# 输出：name=pin_baseline result=green|red；name=pin_mutation name=<变异名> result=caught|missed|invalid；末行 计数：抓到 N、无效 M、没红 K。
# 退出码：0 基线绿、每一行都抓到；1 基线红，或有没红、无效的行；2 用法错、工作目录已存在或锚点不是恰好命中一次。
# 副本与编译目录由调用方（主 agent）在跑完之后删。
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
experiment=${1:-}
[ "$#" -ge 3 ] && [ "${2:-}" = "--work" ] || { echo "用法：prove-histories-pins.sh <e206|e207|e209> --work <新目录>" >&2; exit 2; }
work=$3
# 工作目录是这份脚本上一次建的（带 .prove-histories-pins-owner）就整个删了重建：任务服务撞内存上限照大一档重来时，上一次尝试留下的半成品不挡这一次；别的已存在目录照旧拒。
if [ -e "$work" ]; then
  [ -f "$work/.prove-histories-pins-owner" ] || { echo "工作目录 $work 已存在、不是这份脚本建的，换一个新的" >&2; exit 2; }
  rm -rf -- "${work:?}" || { echo "删不掉上一次留下的工作目录 $work" >&2; exit 2; }
fi
case "$experiment" in
  e206) counters="$here/e206_fixed_point_cost_counters.py"; histories_name=e206_fixed_point_cost_histories; table="$repo/scripts/mutation/tables/e206_fixed_point_cost_histories.tsv"
        filters=(the_six_mebibyte the_registered_points the_base_history) ;;
  e207) counters="$here/e207_strict_scan_steps_counters.py"; histories_name=e207_strict_scan_steps_histories; table="$repo/scripts/mutation/tables/e207_strict_scan_steps_histories.tsv"
        filters=(the_six_mebibyte the_registered_points the_base_history) ;;
  e209) counters="$here/e209_spike_cause_counters.py"; histories_name=e209_spike_cause_histories; table="$repo/scripts/mutation/tables/e209_spike_cause_histories.tsv"
        filters=(the_six_mebibyte the_training_point small_history_makes) ;;
  *) echo "用法：prove-histories-pins.sh <e206|e207|e209> --work <新目录>" >&2; exit 2 ;;
esac
mkdir -p "$work" || exit 2
: > "$work/.prove-histories-pins-owner" || exit 2
python3 -I "$counters" --build plain --source "$repo" --into "$work/plain" --diff-out "$work/plain.diff" || { echo "name=pin_baseline result=red reason=plain_copy_failed"; exit 1; }
tests="$work/plain/crates/singlefs-verification/tests"
mkdir -p "$tests" || exit 2
cp "$here/$histories_name.rs" "$tests/$histories_name.rs" || exit 2
if [ "$experiment" = e207 ]; then
  mkdir -p "$tests/e207_window_choices" "$tests/e207_compact_device" || exit 2
  cp "$here/e207_strict_scan_steps_window_choices.rs" "$tests/e207_window_choices/mod.rs" || exit 2
  cp "$here/e207_strict_scan_steps_compact_device.rs" "$tests/e207_compact_device/mod.rs" || exit 2
fi
if [ "$experiment" = e209 ]; then
  mkdir -p "$tests/e209_scripts" || exit 2
  cp "$here/e209_spike_cause_scripts.rs" "$tests/e209_scripts/mod.rs" || exit 2
fi
cp "$tests/$histories_name.rs" "$work/pristine.rs" || exit 2
cd "$work/plain" || exit 2
E206_ARM=p1 E207_ARM=head E209_ARM=head cargo test --release -p singlefs-verification --test "$histories_name" -- "${filters[@]}" > "$work/baseline.out" 2>&1
baseline_code=$?
if [ "$baseline_code" -ne 0 ]; then
  echo "name=pin_baseline result=red exit=$baseline_code output=$work/baseline.out"
  tail -n 30 "$work/baseline.out"
  exit 1
fi
echo "name=pin_baseline result=green output=$work/baseline.out"
caught=0 invalid=0 missed=0
while IFS=$'\t' read -r mutation_name original replacement; do
  case "$mutation_name" in pin_*) ;; *) continue ;; esac
  python3 -I - "$tests/$histories_name.rs" "$original" "$replacement" <<'PYTHON'
import sys
path, original, replacement = sys.argv[1], sys.argv[2].replace("\\n", "\n"), sys.argv[3].replace("\\n", "\n")
text = open(path, encoding="utf-8").read()
if text.count(original) != 1:
    print("anchor_hits=%d" % text.count(original))
    sys.exit(2)
open(path, "w", encoding="utf-8").write(text.replace(original, replacement, 1))
PYTHON
  anchor_code=$?
  if [ "$anchor_code" -ne 0 ]; then
    echo "name=pin_mutation name=$mutation_name result=invalid reason=anchor_not_found_exactly_once"
    invalid=$((invalid + 1))
    cp "$work/pristine.rs" "$tests/$histories_name.rs"
    continue
  fi
  E206_ARM=p1 E207_ARM=head E209_ARM=head cargo test --release -p singlefs-verification --test "$histories_name" -- "${filters[@]}" > "$work/$mutation_name.out" 2>&1
  code=$?
  cp "$work/pristine.rs" "$tests/$histories_name.rs"
  if [ "$code" -eq 0 ]; then
    echo "name=pin_mutation name=$mutation_name result=missed output=$work/$mutation_name.out"
    missed=$((missed + 1))
  elif grep -q 'could not compile\|^error\[E' "$work/$mutation_name.out" || ! grep -q '^test result: FAILED' "$work/$mutation_name.out"; then
    echo "name=pin_mutation name=$mutation_name result=invalid reason=does_not_compile output=$work/$mutation_name.out"
    invalid=$((invalid + 1))
  else
    echo "name=pin_mutation name=$mutation_name result=caught failed_tests=$(grep -c '^test .* FAILED$' "$work/$mutation_name.out")"
    caught=$((caught + 1))
  fi
done < "$table"
cmp -s "$work/pristine.rs" "$tests/$histories_name.rs" && echo "已还原" || echo "还原失败"
# 仓副本与它的编译目录用完就删（各行的输出、基线输出与 pristine.rs 留在工作目录里）
cd "$work" || exit 2
plain_size=$(du -sh -- "$work/plain" | cut -f1)
rm -rf -- "${work:?}/plain" && echo "删了仓副本 $work/plain（$plain_size）"
echo "计数：抓到 $caught、无效 $invalid、没红 $missed"
[ "$invalid" -eq 0 ] && [ "$missed" -eq 0 ]
