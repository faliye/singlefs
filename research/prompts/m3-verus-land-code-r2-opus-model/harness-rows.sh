#!/usr/bin/env bash
# 逐行把 crates 变异套到仓副本上、只跑那一行点名的 harness 档测试、再从冻结副本拷回原文件。
# 环境变量 CARGO_PROFILE_FLAG（例 --release）原样加在 cargo test 之后。
# 用法：harness-rows.sh <仓副本根> <冻结副本根> <变异表> <出口目录> <行号…>
# 变异表第 5 段是 cargo test 的参数、第 6 段是必须红的测试名（与 crates/mutations.tsv 同格式）；
# 第 5 段写 ALL_HARNESS 时跑整个 singlefs-harness 包，第 6 段写 ANY 时任一测试红都算红。
# 点名 singlefs-checker-tier 的行一律不跑（重型），记 SKIPPED_HEAVY。每行一份判定行写进 <出口目录>/summary.tsv。
set -u
tree=$1; frozen=$2; table=$3; out=$4; shift 4
script_directory=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
for row in "$@"; do
    line=$(awk -v wanted="$row" 'NR==wanted' "$table")
    relative_path=$(printf '%s' "$line" | cut -f2)
    cargo_arguments=$(printf '%s' "$line" | cut -f5)
    required_red=$(printf '%s' "$line" | cut -f6)
    case "$cargo_arguments" in
        *singlefs-checker-tier*) printf '%s\tSKIPPED_HEAVY\t%s\n' "$row" "$cargo_arguments" >> "$out/summary.tsv"; continue ;;
    esac
    if ! python3 "$script_directory/apply-row.py" "$table" "$row" "$tree" > "$out/apply-$row.txt"; then
        printf '%s\tANCHOR\t%s\n' "$row" "$(cat "$out/apply-$row.txt")" >> "$out/summary.tsv"; continue
    fi
    if [[ "$cargo_arguments" == ALL_HARNESS ]]; then
        read -r -a argument_array <<< "-p singlefs-harness --no-fail-fast"
    else
        read -r -a argument_array <<< "$cargo_arguments"
    fi
    started=$(date +%s)
    ( cd "$tree" && cargo test --offline ${CARGO_PROFILE_FLAG:-} "${argument_array[@]}" ) > "$out/test-$row.log" 2>&1
    status=$?
    elapsed=$(( $(date +%s) - started ))
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
    printf '%s\t%s\trc=%s\tpassed=%s\tseconds=%s\tfailed=%s\n' "$row" "$verdict" "$status" "$passed_count" "$elapsed" "$failed_tests" >> "$out/summary.tsv"
done
