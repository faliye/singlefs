#!/usr/bin/env bash
# run_mutant.sh <仓根> <冻结副本的拷贝（改后，不动）> <变异副本> <编译目录> <变异表> <名字|baseline> <阶段：build|run> <输出目录>
# build：变异副本的 transaction.rs、mounted_session.rs 先从改后拷贝恢复，再按名字打一条变异（baseline 不打），debug 编两个测试目标。
# run：直接执行编出来的两个测试二进制（publish_lookup_by_identity 连 ignored 一起跑；snapshot_release_and_destroy_through_the_deadlist 全跑），
#      输出写进 <输出目录>/<名字>-<目标>.log，打一行汇总。线程照 capped.sh 31，内存照 run-with-memory-cap.sh 12G。
#      环境变量 RUN_TARGETS（空格分隔）只跑点名的测试目标，默认两个都跑；RUN_FILTER 是交给测试二进制的过滤串（默认不过滤）。
set -uo pipefail
repo=$1 after=$2 mutant=$3 target=$4 table=$5 name=$6 phase=$7 out=$8
model=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
capped="$repo/scripts/hosts/capped.sh" cap="$repo/scripts/hosts/run-with-memory-cap.sh"
mkdir -p "$out"
case "$phase" in
  build)
    for file in transaction.rs mounted_session.rs; do
      cp "$after/crates/singlefs-core/src/$file" "$mutant/crates/singlefs-core/src/$file"
      touch "$mutant/crates/singlefs-core/src/$file"
    done
    if [[ "$name" != baseline ]]; then python3 "$model/tools/apply_mutant.py" "$table" "$name" "$mutant" || exit 1; fi
    (cd "$mutant" && CARGO_TARGET_DIR="$target" nice -n 19 bash "$cap" 12G bash "$capped" 31 cargo test --offline -p singlefs-harness \
      --test publish_lookup_by_identity --test snapshot_release_and_destroy_through_the_deadlist --no-run) > "$out/$name-build.log" 2>&1
    rc=$?
    echo "name=build mutant=$name rc=$rc $(grep -E '^(error|warning: unused)' "$out/$name-build.log" | sort | uniq -c | tr '\n' ' ')"
    exit $rc ;;
  run)
    for test in ${RUN_TARGETS:-publish_lookup_by_identity snapshot_release_and_destroy_through_the_deadlist}; do
      binary=$(ls -t "$target"/debug/deps/"$test"-* | grep -v '\.d$' | head -1)
      extra=(); [[ $test == publish_lookup_by_identity ]] && extra=(--include-ignored)
      (cd "$mutant/crates/singlefs-harness" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 $test; 范围=1 个测试目标; 为什么=攻方腿副本上的变异 $name 看点名用例红不红" \
        nice -n 19 bash "$capped" 31 bash "$cap" 12G "$binary" "${extra[@]}" --nocapture ${RUN_FILTER:-}) > "$out/$name-$test.log" 2>&1
      rc=$?
      echo "name=run mutant=$name test=$test rc=$rc $(grep -E '^test result' "$out/$name-$test.log") failed=[$(grep -E '^test .* FAILED$' "$out/$name-$test.log" | sed 's/^test //; s/ \.\.\. FAILED$//' | LC_ALL=C sort | tr '\n' ' ')] $(grep -oE 'growth_in_percent=[0-9]+' "$out/$name-$test.log" | tr '\n' ' ')"
    done ;;
esac
