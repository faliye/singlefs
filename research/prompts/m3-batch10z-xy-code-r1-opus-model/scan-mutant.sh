#!/usr/bin/env bash
# K2（第二十五块）：scan-mutant.sh <名字> <工作目录>：两份源文件从冻结副本拷进 <工作目录>/scanmut/src-in，施加 scan-mutants.py 的那一个改坏，
# 重抽两个小程序（build-scan-proto.py）、编（经内存包装 4G、线程 4），在改过的那份源文件上（用例 include_str! 读的就是它自己）跑点名的用例，
# 另对 B 系列在 <工作目录>/scan-inputs 里点名的输入上跑同一条用例与切口。输出 <工作目录>/runs/scanmut-<名字>.log，打最后一行结局。
set -uo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree/crates/singlefs-checker-tier/src
name=${1:?}; WORK=${2:?}
CAP=$ROOT/scripts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/capped.sh
tag=${name//:/-}; log=$WORK/runs/scanmut-$tag.log
S=$WORK/scanmut; mkdir -p "$S/src-in" "$S/proto/src/bin"
cp "$FROZEN"/crash_amplification.rs "$FROZEN"/gpu_unit_checks.rs "$S/src-in/"
mapfile -t picked < <(python3 "$MODEL"/scan-mutants.py "$S/src-in" "$name") || true
[ "${#picked[@]}" = 2 ] || { echo "$name | 施加失败" | tee "$log"; exit 0; }
printf '[package]\nname = "atk-scan-mut"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' > "$S/proto/Cargo.toml"; : > "$S/proto/src/lib.rs"
if [ "${picked[0]}" = gpu_unit_checks.rs ]; then
  python3 "$MODEL"/build-scan-proto.py "$S/src-in/gpu_unit_checks.rs" "$S/proto/src/bin/scan.rs" probe=code_before_the_tests_module skip=a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty > "$log"
else
  python3 "$MODEL"/build-scan-proto.py "$S/src-in/crash_amplification.rs" "$S/proto/src/bin/scan.rs" probe=text_outside_the_tests_module skip=every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads skip=the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run skip=the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken > "$log"
fi
( cd "$S/proto" && bash "$CAP" 4G nice -n 19 bash "$THREADS" 4 cargo build --offline --release ) >> "$log" 2>&1 || { echo "$name | 编不过" | tee -a "$log"; exit 0; }
SRC="$S/src-in/${picked[0]}" bash "$CAP" 2G nice -n 19 "$S/proto/target/release/scan" "${picked[1]}" >> "$log" 2>&1
for input in ${ATK_SCAN_INPUTS:-}; do
  echo "== $input" >> "$log"
  SRC="$WORK/scan-inputs/$input.rs" bash "$CAP" 2G nice -n 19 "$S/proto/target/release/scan" "${picked[1]}" >> "$log" 2>&1
done
echo "$name | ${picked[0]} | $(grep -m1 "^${picked[1]}:" "$log" | cut -c1-200)"
