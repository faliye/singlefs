#!/usr/bin/env bash
# 用法：peer-batch.sh <树根> <后置分类原型二进制> <给定状态原型二进制或 none> <限时秒> <作业…>
# 先把给定状态原型的四个测试各起一个进程（不是 none 时），再起 run-jobs.sh 跑后置分类原型的各片，最后一个不带参数的 wait 等齐。
# 外层已经在内存包装里（multi-host-run.sh --peer 经副本里的 run-with-memory-cap.sh 起这条），所以每件不再各套一层。
set -uo pipefail
root="$1" classifier="$2" given="$3" limit="$4"; shift 4
out="$root/out"
mkdir -p "$out"
if [[ "$given" != none ]]; then
  for test_name in given_states_of_the_rollback_to_the_oldest_ring_root_torn given_states_of_the_rollback_to_the_oldest_ring_root_zeroed \
    given_states_of_the_rollback_to_the_oldest_ring_root_refused_before_writing given_states_of_the_plain_overwrite_zeroing_the_oldest_ring_root; do
    {
      SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 i718_r3_given_states 的 $test_name; 范围=1 个测试; 为什么=第三轮辩方腿给定状态原型" \
        timeout "$limit" nice -n 19 "$given" "$test_name" --nocapture --test-threads=1 > "$out/given-$test_name.txt" 2> "$out/given-$test_name.err"
      echo "$?" > "$out/given-$test_name.exit"
    } &
  done
fi
RUN_JOBS_DIRECT=1 bash "$root/tools/run-jobs.sh" "$root" "$classifier" "$out" "$limit" 3G "$@"
wait
for exit_file in "$out"/given-*.exit; do
  if [[ -e "$exit_file" ]]; then echo "given $(basename "$exit_file" .exit) exit=$(cat "$exit_file")"; fi
done
