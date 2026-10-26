#!/usr/bin/env bash
# 用法：run-jobs.sh <wrapper-root> <测试二进制> <输出目录> <单件限时秒> <内存上限> <作业…>
# 作业写成 S:测试名:k/n（S = 每区槽数，k/n = I718_SHARD）。每件起一个进程并行，各件的标准输出写 <输出目录>/<S>-<测试名>-<k>of<n>.txt，
# 标准错误（含内存包装的提示行）写同名 .err，退出码写同名 .exit；最后一个不带参数的 wait 等齐，按作业次序数 .exit 文件。
# RUN_JOBS_DIRECT=1：每件不再各套一层内存包装（整条脚本已经在外层的 run-with-memory-cap.sh 里，第二台上就是这么跑的）。
set -uo pipefail
wrapper_root="$1" binary="$2" out="$3" limit="$4" cap="$5"; shift 5
mkdir -p "$out"
jobs_count=$#
for job in "$@"; do
  s="${job%%:*}"; rest="${job#*:}"; test_name="${rest%%:*}"; shard="${rest#*:}"
  k="${shard%%/*}"; n="${shard##*/}"
  base="$out/$s-$test_name-${k}of${n}"
  rm -f "$base.txt" "$base.err" "$base.exit"
  {
    export I718_S="$s" I718_SHARD="$shard"
    export SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 i718_r3_independent 的 $test_name; 范围=1 个测试的 $shard 片; 为什么=第三轮辩方腿后置分类原型 S=$s"
    if [[ "${RUN_JOBS_DIRECT:-}" == 1 ]]; then
      timeout "$limit" nice -n 19 "$binary" "$test_name" --nocapture --test-threads=1 > "$base.txt" 2> "$base.err"
    else
      timeout "$limit" nice -n 19 bash "$wrapper_root/scripts/hosts/run-with-memory-cap.sh" "$cap" "$binary" "$test_name" --nocapture --test-threads=1 > "$base.txt" 2> "$base.err"
    fi
    echo "$?" > "$base.exit"
  } &
done
wait
found=$(ls "$out"/*.exit 2>/dev/null | wc -l)
echo "jobs=$jobs_count exit_files_in_dir=$found"
for job in "$@"; do
  s="${job%%:*}"; rest="${job#*:}"; test_name="${rest%%:*}"; shard="${rest#*:}"
  k="${shard%%/*}"; n="${shard##*/}"
  base="$out/$s-$test_name-${k}of${n}"
  echo "job=$job exit=$(cat "$base.exit" 2>/dev/null || echo missing) rows=$(grep -c 'ROW cell' "$base.txt" 2>/dev/null) scans=$(grep -c '^SCAN' "$base.txt" 2>/dev/null)"
done
