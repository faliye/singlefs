#!/usr/bin/env python3
"""三方攻方腿（m3-close-unit-graph-wrapper-impls-r2）：同一棵树（仓根）上，两份 unit-graph（主工作区编的 today 与 verify-copy 编的）
各收一趟闭包，比收严一、二之后闭包变大变小多少。量的是：入口登记表 crates/singlefs-verification/src/judging_units_entries.rs 的九组
（照 gpu 特性那一种构建收：--cfg feature="gpu"），与几条按条复用的测试函数入口（.claude/gate.d/stage-inputs.tsv 里 random_histories 那六个）、
几个观察者入口（同一份登记表的观察者那几行，各自的测试目标）。都给 --toolchain-line host: x86_64-unknown-linux-gnu 与 --judgment-kind product-behavior，
不给依赖行；不带 GPU 判器组的「不沿它收」登记（unit-graph closure 不认那张表），量的是收严一、二本身。

用法：measure_closure_sizes.py --today-binary <路径> --verify-copy-binary <路径> --root <仓根> --work <运行目录>
每个量一组三行：name=<量> rule=<today|verify_copy> rc=<退出码> rust=<个> wgsl=<个> data=<个> refused=<处>（从标准错误那一句取）；
name=<量> rust_only_in_verify_copy=<个> rust_only_in_today=<个> top_files_only_in_verify_copy=<文件:个,…前六个> top_files_only_in_today=<同>。
两边的组成清单落在 <运行目录>/<量>.<规则>.txt。退出码：0 每一趟都退 0 或 3（3 是有拒算，照列）；2 有一趟退别的码或标准错误里找不到那一句。"""
import argparse
import collections
import os
import re
import signal
import subprocess
import sys

TOOLCHAIN_LINE = "host: x86_64-unknown-linux-gnu"
SECONDS = 1800
GPU = 'feature="gpu"'
GROUPS = [
    ("gpu_judge", "lib", True, [
        "crash_judge_gpu::JudgeKernelSources::of_the_judge",
        "crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs",
        "crash_judge_dispatch::GpuCrashJudge::on_card_compiling_these_kernel_sources",
        "crash_judge_dispatch::GpuCrashJudge::judge_one_dispatch",
        "crash_judge_gpu::gpu_verdict_items",
        "crash_judge_gpu::recovery_words_of_the_gpu_verdict",
        "crash_judge_gpu::cross_check_words_of_gpu_verdict",
        "crash_judge_gpu::scan_rebuild_outcome_of_the_gpu_verdict",
        "crash_judge_gpu::self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict",
        "crash_judge_conclusions::StateCountsOfABlock::count_the_gpu_bits",
        "crash_judge_conclusions::StateCountsOfABlock::into_count_vector",
        "crash_judge_conclusions::encoded_verdict_of_the_red_items",
        "crash_judge_tables::build_judge_tables_against_the_version_table_with_slices",
        "crash_judge_tables::build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions",
        "crash_judge_tables::JudgeTables::words",
        "crash_judge_dispatch::PackedJudgeTables::of"]),
    ("judgement_of_the_crash_images", "lib", True, [
        "crash::judge_crash_image",
        "crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu",
        "crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device",
        "crash_judge_conclusions::StateCountsOfABlock::count_the_cpu_judgement",
        "crash_judge_conclusions::StateCountsOfABlock::into_count_vector",
        "crash_judge_conclusions::encoded_verdict_of_the_red_items"]),
    ("judgement_of_the_given_states", "lib", True, [
        "crash_given_states::judge_the_given_states_on_the_cpu",
        "crash_given_states::count_vector_of_the_given_states",
        "crash_judge_conclusions::encoded_verdict_of_the_red_items"]),
    ("cpu_facts_verifier", "lib", True, [
        "crash_facts::verify_states_from_facts",
        "crash_facts::verify_states_from_facts_by_the_segment_expansions"]),
    ("gpu_facts_verifier", "lib", True, [
        "crash_verify_gpu::GpuFactsVerifier::new",
        "crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions",
        "crash_verify_gpu::GpuFactsVerifier::verify_states"]),
    ("recording", "lib", True, [
        "crash_facts::extract_flow_facts_with_slices",
        "crash_facts::FlowFacts::to_bytes",
        "crash_judge_tables::build_judge_tables_against_the_version_table_with_slices",
        "crash_judge_tables::build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions",
        "crash_judge_tables::JudgeTables::words",
        "crash_judge_dispatch::PackedJudgeTables::of",
        "crash_judge_dispatch::PackedJudgeTables::to_bytes",
        "crash_amplification::RecordedFlow::to_bytes"]),
    ("gpu_and_cpu_judge_pairing_comparison", "test:crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state", True,
     ["compare_the_judgements_state_by_state"]),
    ("gpu_and_cpu_facts_verifier_pairing_comparison", "test:crash_facts_gpu_verifier_agrees_with_the_cpu_reference", True,
     ["verifier_bits_agreeing_on_every_state"]),
    ("observer_conclusion_composition", "lib", True, [
        "crash_amplification::judge_the_images_after_the_observers_on_the_cpu",
        "crash_amplification::judge_the_images_after_the_observers_on_the_gpu",
        "crash_amplification::plan_the_judge_tables_of_the_images_after_the_observers",
        "crash_amplification::pool_after_the_observer"]),
]
RANDOM_HISTORIES = [
    "random_histories_end_only_in_known_red_forms_and_exercise_every_operation",
    "reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms",
    "rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms",
    "allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall",
    "unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall",
    "unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval",
]
OBSERVERS = [
    ("observer.publish_boundary", "test:crash_enumeration_record_spill_over_stream", "observe_the_publish_boundary"),
    ("observer.deletion_cut", "test:crash_enumeration_unlink_after_the_multi_record_publish_stream", "observe_the_deletion_cut_in_the_middle"),
    ("observer.stale_tail", "test:crash_enumeration_fixed_script_stream", "observe_the_stale_tail"),
    ("observer.first_file_after_the_crash", "test:crash_enumeration_formatted_pool_remounted_without_a_file_stream",
     "remount_and_publish_the_first_file_after_the_crash"),
    ("observer.floor_raise", "test:crash_enumeration_floor_raise_pushed_by_admission", "mount_writable_after_the_crash_of_the_floor_raise"),
    ("observer.instance_table_sweep", "test:crash_points_of_the_instance_table_sweep", "count_the_states_where_the_deletion_took_effect"),
    ("observer.mount_continuation", "test:crash_enumeration_multi_record_publish_stream",
     "common_crash_point_observers::observe_the_mount_continuation"),
]
SUMMARY = re.compile(r"闭包 (\d+) 个单元（Rust (\d+)、WGSL (\d+)、数据 (\d+)），拒算 (\d+) 处")


def requests():
    for name, target, with_gpu, entries in GROUPS:
        yield name, target, with_gpu, [f"singlefs_verification::{entry}" for entry in entries]
    for function in RANDOM_HISTORIES:
        yield f"random_histories.{function}", "test:random_histories", False, [f"singlefs_verification::{function}"]
    for name, target, function in OBSERVERS:
        yield name, target, False, [f"singlefs_verification::{function}"]


def run_in_its_own_group(argv):
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        output, errors = process.communicate(timeout=SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        return None, "", f"超时（{SECONDS} 秒）"
    try:
        os.killpg(process.pid, 0)
    except ProcessLookupError:
        pass
    else:
        os.killpg(process.pid, signal.SIGKILL)
        return None, output, "跑完之后进程组里还有进程，已整组 SIGKILL"
    return process.returncode, output, errors


def rust_members(output):
    return {tuple(line.split("\t")[:3]) for line in output.splitlines()
            if len(line.split("\t")) >= 3 and re.search(r"\.rs:\d+-\d+$", line.split("\t")[2])}


def top_files(members):
    counts = collections.Counter(member[2].rsplit(":", 1)[0] for member in members)
    return ",".join(f"{path}:{count}" for path, count in counts.most_common(6)) or "-"


def main():
    parser = argparse.ArgumentParser()
    for option in ("--today-binary", "--verify-copy-binary", "--root", "--work"):
        parser.add_argument(option, required=True)
    arguments = parser.parse_args()
    os.makedirs(arguments.work, exist_ok=True)
    broken = 0
    for name, target, with_gpu, entries in requests():
        members_of = {}
        for rule, binary in (("today", arguments.today_binary), ("verify_copy", arguments.verify_copy_binary)):
            argv = [binary, "closure", arguments.root, *entries]
            if target != "lib":
                argv += ["--target", target]
            if with_gpu:
                argv += ["--cfg", GPU]
            argv += ["--toolchain-line", TOOLCHAIN_LINE, "--judgment-kind", "product-behavior"]
            code, output, errors = run_in_its_own_group(argv)
            with open(os.path.join(arguments.work, f"{name}.{rule}.txt"), "w", encoding="utf-8") as listing:
                listing.write(output)
            found = SUMMARY.search(errors or "")
            if code not in (0, 3) or found is None:
                broken += 1
                print(f"name={name} rule={rule} rc={code} error={' '.join((errors or '').split())[-300:]}")
                continue
            total, rust, wgsl, data, refused = found.groups()
            print(f"name={name} rule={rule} rc={code} rust={rust} wgsl={wgsl} data={data} refused={refused}")
            members_of[rule] = rust_members(output)
        if len(members_of) == 2:
            only_new = members_of["verify_copy"] - members_of["today"]
            only_old = members_of["today"] - members_of["verify_copy"]
            print(f"name={name} rust_only_in_verify_copy={len(only_new)} rust_only_in_today={len(only_old)} "
                  f"top_files_only_in_verify_copy={top_files(only_new)} top_files_only_in_today={top_files(only_old)}")
    print(f"name=summary measured={sum(1 for _ in requests())} broken_runs={broken}")
    return 2 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
