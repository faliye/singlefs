#!/usr/bin/env bash
# 用法：run-mutation-campaign.sh <副本根> <日志目录> <表.tsv> <行名或 @行号>…
# 每一项：套上（几项用 + 连起来就叠着套）→ 跑 singlefs-harness 里 13 个测试目标与 singlefs-core、singlefs-replay-prefix 两个库的测试（--no-fail-fast，
# 含 singlefs-core 里挂上的对拍 opus_before_and_after…，OPUS_DIFF_CASES 默认 3000）→ 换回。每项一份日志，汇总行写进 <日志目录>/campaign.txt。
# 不跑 checker 档。外面由调用方套 run-with-memory-cap.sh 与 capped.sh。
set -u
tree=$1; logs=$2; table=$3; shift 3
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
export OPUS_DIFF_CASES=${OPUS_DIFF_CASES:-3000} OPUS_DIFF_SEED=${OPUS_DIFF_SEED:-7}
mkdir -p "$logs"
for item in "$@"; do
    label=${item//[@+]/_}
    applied=yes
    IFS='+' read -r -a parts <<< "$item"
    for part in "${parts[@]}"; do
        python3 "$here/apply-mutation.py" "$tree" "$table" "$part" || applied=no
    done
    if [ "$applied" = no ]; then
        echo "$item 套不上" >> "$logs/campaign.txt"
        python3 "$here/apply-mutation.py" "$tree" --restore
        continue
    fi
    start=$(date +%s)
    # 全量 harness 在 8G 里撞顶（基线那一次退 250），挂钟 1539 秒：只跑点名核的变异行引到的、与读 journal 前缀相关的 13 个测试目标与两个库。
    targets=()
    for target in journal_last_record_flag_of_a_multi_record_publish writable_mount_takes_a_second_instance multi_unit_file_write_and_read \
        row_publish_record_without_its_root core_review_geometry_back_chain_and_empty_inode core_review_tree_table_duplicates_and_slot_one_search \
        recovery_reads_the_created_file_after_reopen whole_device_loss_after_warm_up warm_up_journal_counter fault_injection_fast_tier \
        overwrite_in_one_instance sequential_write_split_into_one_unit_transactions writable_mount_of_a_formatted_pool; do
        targets+=(--test "$target")
    done
    (cd "$tree" && cargo test --offline -p singlefs-harness "${targets[@]}" --no-fail-fast -- --nocapture ;
        cargo test --offline -p singlefs-core -p singlefs-replay-prefix --lib --no-fail-fast -- --nocapture) > "$logs/$label.log" 2>&1
    status=$?
    seconds=$(( $(date +%s) - start ))
    failed=$(grep -E '^test .* \.\.\. FAILED$' "$logs/$label.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort -u | tr '\n' ' ')
    results=$(grep -c '^test result' "$logs/$label.log")
    passed=$(grep '^test result' "$logs/$label.log" | awk '{p += $4} END {print p + 0}')
    differential=$(grep -o 'opus-diff seed=[0-9]* cases=[0-9]* disagreements=[0-9]*' "$logs/$label.log" | head -1)
    compile_error=$(grep -c '^error' "$logs/$label.log")
    echo "$item rc=$status seconds=$seconds result_lines=$results passed=$passed compile_errors=$compile_error [$differential] failed: $failed" >> "$logs/$label.summary"
    cat "$logs/$label.summary" >> "$logs/campaign.txt"
    python3 "$here/apply-mutation.py" "$tree" --restore
done
