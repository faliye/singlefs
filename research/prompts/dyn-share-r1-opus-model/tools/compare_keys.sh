#!/usr/bin/env bash
# 对 copy-base 与各变体树，用主工作区编好的 unit-graph 算四组判法单元的键，打一张表：变体、组、键是否与 copy-base 相同。
set -u
repo="$1"; work="$2"
graph="$repo/target/debug/unit-graph"
P=singlefs_checker_tier
declare -A entries
entries[judgement_of_the_crash_images]="$P::crash::judge_crash_image $P::crash_judge_conclusions::StateCountsOfABlock::count_the_cpu_judgement $P::crash_judge_conclusions::StateCountsOfABlock::into_count_vector $P::crash_judge_conclusions::encoded_verdict_of_the_red_items"
entries[gpu_judge]="$P::crash_judge_gpu::JudgeKernelSources::of_the_judge $P::crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs $P::crash_judge_dispatch::GpuCrashJudge::on_card_compiling_these_kernel_sources $P::crash_judge_dispatch::GpuCrashJudge::judge_states $P::crash_judge_gpu::gpu_verdict_items $P::crash_judge_gpu::recovery_words_of_the_gpu_verdict $P::crash_judge_gpu::cross_check_words_of_gpu_verdict $P::crash_judge_conclusions::StateCountsOfABlock::count_the_gpu_bits $P::crash_judge_conclusions::StateCountsOfABlock::into_count_vector $P::crash_judge_conclusions::encoded_verdict_of_the_red_items"
entries[recording]="$P::crash_facts::extract_flow_facts_with_slices $P::crash_facts::FlowFacts::to_bytes $P::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices $P::crash_judge_tables::build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions $P::crash_judge_tables::JudgeTables::words $P::crash_judge_dispatch::PackedJudgeTables::of $P::crash_judge_dispatch::PackedJudgeTables::to_bytes $P::crash_amplification::RecordedFlow::to_bytes"
entries[observer_conclusion_composition]="$P::crash_amplification::judge_the_images_after_the_observers_on_the_cpu $P::crash_amplification::judge_the_images_after_the_observers_on_the_gpu $P::crash_amplification::plan_the_judge_tables_of_the_images_after_the_observers $P::crash_amplification::pool_after_the_observer"
groups="judgement_of_the_crash_images gpu_judge recording observer_conclusion_composition"
key_of() {
  local tree="$1" group="$2"
  # shellcheck disable=SC2086
  "$graph" key "$tree" ${entries[$group]} --target lib --cfg 'feature="verdict-store"' --cfg 'feature="gpu"' --cfg 'feature="default"' --toolchain-of-the-build dev --lock-file "$tree/Cargo.lock" --threads 8 2>/dev/null | awk '$1=="key"{print $2}'
}
declare -A base
for group in $groups; do base[$group]="$(key_of "$work/copy-base" "$group")"; echo "copy-base	$group	${base[$group]}"; done
for tree in "$work"/variant-*; do
  name="${tree##*/variant-}"
  for group in $groups; do
    key="$(key_of "$tree" "$group")"
    if [[ -z "$key" ]]; then verdict="没算出"; elif [[ "$key" == "${base[$group]}" ]]; then verdict="相同"; else verdict="变了"; fi
    echo "$name	$group	$verdict	$key"
  done
done
