#!/usr/bin/env bash
# 逐行把 crates/mutations.tsv 的变异套到仓副本上、只跑那一行点名的 harness 档测试（经内存包装 6G、线程 10）、再从冻结副本拷回原文件。
# 第二轮攻方 harness-rows.sh 的改写：cargo test 经 run-with-memory-cap.sh；点名 singlefs-checker-tier 的行不跑（重型）。
# 用法：harness-rows-r3.sh <仓副本根> <冻结副本根> <变异表> <出口目录> <行号…>
set -u
tree=$1; frozen=$2; table=$3; out=$4; shift 4
script_directory=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示给定内存上限十二G，变异行按其中六G跑"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的几个腿各跑各的"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1500
for row in "$@"; do
    line=$(awk -v wanted="$row" 'NR==wanted' "$table")
    relative_path=$(printf '%s' "$line" | cut -f2)
    cargo_arguments=$(printf '%s' "$line" | cut -f5)
    required_red=$(printf '%s' "$line" | cut -f6)
    name=$(printf '%s' "$line" | cut -f1 | cut -c1-60)
    case "$cargo_arguments" in
        *singlefs-checker-tier*) printf '%s\tSKIPPED_HEAVY\t%s\n' "$row" "$cargo_arguments" >> "$out/summary.tsv"; continue ;;
    esac
    if ! python3 "$script_directory/apply-row.py" "$table" "$row" "$tree" > "$out/apply-$row.txt"; then
        printf '%s\tANCHOR\t%s\n' "$row" "$(cat "$out/apply-$row.txt")" >> "$out/summary.tsv"; continue
    fi
    read -r -a argument_array <<< "$cargo_arguments"
    ( cd "$tree" && nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 10 cargo test --offline --release "${argument_array[@]}" ) > "$out/test-$row.log" 2>&1
    status=$?
    cp "$frozen/$relative_path" "$tree/$relative_path"
    failed_tests=$(grep -E '^test .* \.\.\. FAILED$' "$out/test-$row.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | tr '\n' ' ')
    passed_count=$(grep -cE '^test .* \.\.\. ok$' "$out/test-$row.log")
    if [[ "$required_red" == ANY ]]; then
        verdict=$([[ -n "$failed_tests" ]] && echo RED || echo GREEN)
    elif printf '%s' "$failed_tests" | grep -qF "$required_red"; then
        verdict=RED
    else
        verdict=GREEN
    fi
    if grep -qE '^error(\[E[0-9]+\])?: (could not compile|aborting)|^error\[E[0-9]+\]' "$out/test-$row.log"; then verdict=BUILD_ERROR; fi
    printf '%s\t%s\trc=%s\tpassed=%s\tfailed=%s\t%s\n' "$row" "$verdict" "$status" "$passed_count" "$failed_tests" "$name" >> "$out/summary.tsv"
done
