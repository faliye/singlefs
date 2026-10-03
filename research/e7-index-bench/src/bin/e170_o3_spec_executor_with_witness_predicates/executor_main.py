"""E170（O3规约执行器第一版） 执行器的驱动：读导出、按批判每一态、计时、打判定行与汇总行，最后打冻结的 sha256。
第 2 次重跑（v2，登记 5.2 改三）：批汇总另打 name=witness_summary；锚点另打 B1–B3（name=witness_anchor_assertion）；
每版的轮换写打 name=rotation_tail；冻结照 v1 的取法，不进冻结的前缀照登记 5.6 加 witness_reference、run_。

跑法（在本目录，经 run-with-memory-cap.sh 与 capped.sh）：
  python3 executor_main.py --dumps <research/target/e170-dumps> --batches 1,2,3 --workers <N> --output <新文件>
每态耗时在执行器进程里计（time.perf_counter_ns，从开始建这一态的镜像起，到判定行定稿止；登记第一节读法 7），
一批的合计用时从第一态派出前到最后一行写完止，由主进程计；读入导出文件的时间另报（name=load）。
"""

from __future__ import annotations

import argparse
import hashlib
import multiprocessing
import os
import statistics
import sys
import time

import crc32c_castagnoli
import on_disk_layout as layout
import spec_gaps
import witness_predicates
from crash_dump import load_stream_dump
from crash_image import build_crash_image
from executor_core import VERDICT_IN, VERDICT_NOT_IN, VERDICT_SPEC_GAP, BaseImageNotRecoverable, StreamContext, judge_state, \
    judgement_line, prepare_stream_context, recover
from replay_prefix import UnitChecksumCache
from version_table import allowed_versions

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
# 第 2 次重跑登记 5.6：冻结的文件 = 新目录里除 comparator*、mutation_driver*、witness_reference*、run_* 之外的全部（含 spec-gaps.tsv 与单测）
FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver", "witness_reference", "run_")
EXECUTOR_DIRECTORY = os.path.dirname(os.path.abspath(__file__))

contexts_by_stream: dict[str, StreamContext] = {}
worker_checksum_cache = UnitChecksumCache()


def judge_one_state_in_worker(task: tuple[str, int]) -> tuple[dict, str]:
    stream_name, state_position = task
    context = contexts_by_stream[stream_name]
    state = context.dump.states[state_position]
    try:
        judgement = judge_state(context, state, worker_checksum_cache)
    except Exception as error:  # 判定行缺失按登记第十一节 V4 处置；这里照打一行出错，不吞
        failure = f"{type(error).__name__}:{str(error)[:200]}".replace(" ", "_")
        line = (f"name=state stream={stream_name} state={state.state_number} layer0={state.layer_zero_sequence} "
                f"segment={state.segment_number} batch={state.batch_number} strict=error provisional=error error={failure} "
                f"worker={os.getpid()}")
        outcome = {"stream": stream_name, "state": state.state_number, "elapsed": 0, "worker": os.getpid(),
                   "strict": "error", "provisional": "error", "gaps": [], "digest_matches": False,
                   "segment": state.segment_number, "refusal": "-", "chosen_root_below_largest_slot_txg": 0,
                   "chosen_root_below_largest_self_verified_txg": 0, "slot_one_route_one_disks": 0, "slot_one_route_two_disks": 0,
                   "slot_one_route_three_disks": 0, "slot_one_route_three_two_or_more_collected_disks": 0,
                   "applied": 0, "anchor_unreadable": 0, "torn": 0,
                   "strict_before_witness": "error", "provisional_before_witness": "error", "witness_counter": "-",
                   "witness_branch": "error", "witness_compared": "-", "witness_true": "error", "ack_rotation_tail": "-",
                   "tail_check": "error", "failed_predicates": "-", "acknowledged_version": -1}
        return outcome, line
    line = judgement_line(stream_name, judgement, os.getpid())
    outcome = {"stream": stream_name, "state": state.state_number, "elapsed": judgement.elapsed_nanoseconds,
               "worker": os.getpid(), "strict": judgement.strict_verdict, "provisional": judgement.provisional_verdict,
               "gaps": judgement.gaps_hit, "digest_matches": judgement.digest_matches,
               "segment": state.segment_number} | judgement.recovery_summary
    return outcome, line


def percentile(sorted_values: list[int], fraction: float) -> int:
    if not sorted_values:
        return 0
    index = min(len(sorted_values) - 1, max(0, int(round(fraction * (len(sorted_values) - 1)))))
    return sorted_values[index]


def run_batch(batch_number: int, worker_count: int, emit) -> list[tuple]:
    tasks = [(stream_name, position) for stream_name, context in contexts_by_stream.items()
             for position, state in enumerate(context.dump.states) if state.batch_number == batch_number]
    results = []
    started = time.perf_counter_ns()
    if worker_count == 1:
        for task in tasks:
            outcome, line = judge_one_state_in_worker(task)
            emit(line)
            results.append(outcome)
    else:
        with multiprocessing.get_context("fork").Pool(worker_count) as pool:
            for outcome, line in pool.imap_unordered(judge_one_state_in_worker, tasks, chunksize=1):
                emit(line)
                results.append(outcome)
    total = time.perf_counter_ns() - started
    per_worker_sum: dict[int, int] = {}
    for outcome in results:
        per_worker_sum[outcome["worker"]] = per_worker_sum.get(outcome["worker"], 0) + outcome["elapsed"]
    largest_worker_sum = max(per_worker_sum.values(), default=0)
    contains = total >= largest_worker_sum  # 登记 5.1：合计用时要包住各态耗时之和（多进程时包住各工作进程之和的最大值）
    emit(f"name=timing batch={batch_number} workers={worker_count} states={len(results)} total_ns={total} "
         f"sum_state_ns={sum(outcome['elapsed'] for outcome in results)} largest_worker_sum_ns={largest_worker_sum} "
         f"worker_processes={len(per_worker_sum)} contains_self_check={'pass' if contains else 'fail'}")
    for stream_name in contexts_by_stream:
        times = sorted(outcome["elapsed"] for outcome in results if outcome["stream"] == stream_name)
        emit(f"name=state_time batch={batch_number} workers={worker_count} stream={stream_name} states={len(times)} "
             f"median_ns={int(statistics.median(times)) if times else 0} p95_ns={percentile(times, 0.95)} "
             f"max_ns={times[-1] if times else 0} mean_ns={int(statistics.mean(times)) if times else 0}")
    return results


def summarise_verdicts(batch_number: int, results: list[tuple], emit) -> None:
    for stream_name in contexts_by_stream:
        stream_results = [outcome for outcome in results if outcome["stream"] == stream_name]
        strict_gap_states = [outcome for outcome in stream_results if outcome["strict"] == VERDICT_SPEC_GAP]
        gap_identifiers = sorted({identifier for outcome in strict_gap_states for identifier in outcome["gaps"]})
        states_by_gap = {identifier: sum(identifier in outcome["gaps"] for outcome in stream_results) for identifier in gap_identifiers}
        failed = [outcome for outcome in stream_results if outcome["provisional"] == "error"]
        given = len(stream_results) - len(strict_gap_states) - len(failed)
        not_in = [outcome for outcome in stream_results if outcome["provisional"] == VERDICT_NOT_IN]
        emit(f"name=verdict batch={batch_number} stream={stream_name} states={len(stream_results)} "
             f"strict_spec_gap_states={len(strict_gap_states)} strict_spec_gap_identifiers={','.join(gap_identifiers) or 'zero'} "
             f"strict_given_states={given} provisional_in={sum(outcome['provisional'] == VERDICT_IN for outcome in stream_results)} "
             f"provisional_not_in={len(not_in)} judgement_failures={len(failed)} "
             f"digest_mismatches={sum(not outcome['digest_matches'] for outcome in stream_results if outcome['provisional'] != 'error')}")
        emit(f"name=gap_hits batch={batch_number} stream={stream_name} "
             + " ".join(f"{identifier}={count}" for identifier, count in states_by_gap.items()))
        # Q2-e：执行器路径计数（阴性结果与「代码没跑到」分开）
        emit(f"name=path_counts batch={batch_number} stream={stream_name} replayed_at_least_one={sum(outcome['applied'] >= 1 for outcome in stream_results)} "
             f"chosen_root_below_largest_slot_txg={sum(outcome['chosen_root_below_largest_slot_txg'] for outcome in stream_results)} "
             f"chosen_root_below_largest_self_verified_txg={sum(outcome['chosen_root_below_largest_self_verified_txg'] for outcome in stream_results)} "
             f"slot_one_route_one_disk_states={sum(outcome['slot_one_route_one_disks'] for outcome in stream_results)} "
             f"slot_one_route_two_disk_states={sum(outcome['slot_one_route_two_disks'] for outcome in stream_results)} "
             f"slot_one_route_three_disk_states={sum(outcome['slot_one_route_three_disks'] for outcome in stream_results)} "
             f"slot_one_route_three_two_or_more_collected={sum(outcome['slot_one_route_three_two_or_more_collected_disks'] for outcome in stream_results)} "
             f"anchor_unreadable_path={sum(outcome['anchor_unreadable'] for outcome in stream_results)} "
             f"provisional_not_in={len(not_in)} with_torn_write={sum(outcome['torn'] for outcome in stream_results)} "
             f"pool_refused={sum(outcome['refusal'] != '-' for outcome in stream_results)}")
        # 改三（第 2 次重跑登记 5.2）：W 真、T 假、T 不判、failed_predicates 各几态；前判定与终判定各几态
        emit(f"name=witness_summary batch={batch_number} stream={stream_name} states={len(stream_results)} "
             f"strict_before_in={sum(outcome['strict_before_witness'] == VERDICT_IN for outcome in stream_results)} "
             f"strict_before_not_in={sum(outcome['strict_before_witness'] == VERDICT_NOT_IN for outcome in stream_results)} "
             f"strict_before_spec_gap={sum(outcome['strict_before_witness'] == VERDICT_SPEC_GAP for outcome in stream_results)} "
             f"strict_in={sum(outcome['strict'] == VERDICT_IN for outcome in stream_results)} "
             f"strict_not_in={sum(outcome['strict'] == VERDICT_NOT_IN for outcome in stream_results)} "
             f"witness_not_evaluated={sum(outcome['witness_branch'] == 'not_evaluated' for outcome in stream_results)} "
             + " ".join(f"witness_branch_{branch}={sum(outcome['witness_branch'] == branch for outcome in stream_results)}"
                        for branch in ('nothing', 'last_record', 'record_at_counter', 'both_unreadable')) + " "
             f"witness_true={sum(outcome['witness_true'] == 'true' for outcome in stream_results)} "
             f"tail_check_true={sum(outcome['tail_check'] == 'true' for outcome in stream_results)} "
             f"tail_check_false={sum(outcome['tail_check'] == 'false' for outcome in stream_results)} "
             f"tail_check_not_evaluated={sum(outcome['tail_check'] == 'not_evaluated' for outcome in stream_results)} "
             + " ".join(f"failed_{label}={sum(outcome['failed_predicates'] == value for outcome in stream_results)}"
                        for label, value in (('W', 'W'), ('T', 'T'), ('W_and_T', 'W,T'))) + " "
             f"strict_in_to_not_in={sum(outcome['strict_before_witness'] == VERDICT_IN and outcome['strict'] == VERDICT_NOT_IN for outcome in stream_results)} "
             f"before_not_in_with_failed_predicates={sum(outcome['provisional_before_witness'] == VERDICT_NOT_IN and outcome['failed_predicates'] not in ('-',) for outcome in stream_results)}")
        segments = sorted({outcome["segment"] for outcome in stream_results})
        gap_states_by_segment = [sum(outcome["segment"] == segment for outcome in strict_gap_states) for segment in segments]
        emit(f"name=gap_trajectory batch={batch_number} stream={stream_name} segments={len(segments)} "
             f"peak={max(gap_states_by_segment, default=0)} positive_segments={sum(count > 0 for count in gap_states_by_segment)} "
             f"last_segment_value={gap_states_by_segment[-1] if gap_states_by_segment else 0}")
        not_in_by_segment = [sum(outcome["segment"] == segment for outcome in not_in) for segment in segments]
        emit(f"name=not_in_trajectory batch={batch_number} stream={stream_name} segments={len(segments)} "
             f"peak={max(not_in_by_segment, default=0)} positive_segments={sum(count > 0 for count in not_in_by_segment)} "
             f"last_segment_value={not_in_by_segment[-1] if not_in_by_segment else 0}")


def version_pair(projection) -> tuple[int, int]:
    return (projection.instance, projection.checkpoint_txg)


def anchor_assertions(stream_name: str, context: StreamContext, emit) -> None:
    """登记第七节 A1–A7（流一全持久态）、A6（两条流）、A11（流二全持久态）。不符走第十节 F2，不作废、不改锚点。"""
    versions = [version_pair(version.projection) for version in context.version_table.versions]
    adjacency_holds = all((later[0] == earlier[0] and later[1] == earlier[1] + 1) or (later[0] != earlier[0] and later[1] >= earlier[1] + 1)
                          for earlier, later in zip(versions, versions[1:]))
    emit(f"name=anchor_assertion id=A6 stream={stream_name} holds={str(adjacency_holds).lower()} versions={len(versions)}")
    all_persisted = "2" * len(context.prepared_writes)
    if not any(state.landing_vector == all_persisted for state in context.dump.states):
        emit(f"name=anchor_assertion id=all_persisted_state stream={stream_name} holds=not_run detail=no_sampled_all_persisted_state")
        return
    image = build_crash_image(context.dump, context.prepared_writes, all_persisted)
    recovery = recover(image, context.dump.device_identities, context.dump.sector_bytes, UnitChecksumCache(), spec_gaps.GapsHitInOneState())
    allowed = [version_pair(version.projection) for version in allowed_versions(context.version_table, all_persisted).allowed]
    landed = None if recovery.landed is None else version_pair(recovery.landed)
    if stream_name == STREAM_NAMES[1]:
        flags = None if recovery.chosen_root is None else recovery.chosen_root.unsigned("flags")
        emit(f"name=anchor_assertion id=A11 stream={stream_name} holds={str(flags is not None and flags & 1 == 1).lower()} "
             f"chosen_root_flags={flags} landed={landed}")
        return
    if stream_name != STREAM_NAMES[0]:
        return
    # A1（第 1 次重跑登记第七节改正）：A(F) 恰两个元素 (1, 2)、(1, 3)——读法 3：末次调用即新池新建文件那次发布，返回点大于 m(F)，
    # ack = 暖机那次调用的最后一版 (1, 2)；原登记写「恰一个元素」与读法 3 不符，这次改正。
    emit(f"name=anchor_assertion id=A1 stream={stream_name} holds={str(landed == (1, 3) and allowed == [(1, 2), (1, 3)]).lower()} "
         f"landed={landed} allowed={allowed} landed_part={str(landed == (1, 3)).lower()} two_element_part={str(allowed == [(1, 2), (1, 3)]).lower()}")
    roots = sorted((root.location.region, root.location.slot_in_region, root.location.device_identity, root.unsigned("instance"),
                    root.unsigned("checkpoint_txg"), root.location.byte_offset) for root in recovery.root_reading.self_verified_roots)
    expected_roots = [(0, 0, 0, 0, 0), (0, 1, 0, 1, 3), (1, 0, 1, 1, 1), (2, 0, 0, 1, 2)]
    txg_three_offset = [root[5] for root in roots if root[4] == 3]
    emit(f"name=anchor_assertion id=A2 stream={stream_name} holds={str([root[:5] for root in roots] == expected_roots and txg_three_offset == [(1 << 20) + 4096]).lower()} "
         f"roots={roots}")
    ring = {device: [(record.ring_slot_index * 4096, record.jsn, record.unsigned("checkpoint_txg")) for record in records]
            for device, records in recovery.ring_scan.valid_records_by_device.items()}
    expected_ring = [(0, (1, 1), 1), (4096, (1, 2), 2), (8192, (1, 3), 3)]
    emit(f"name=anchor_assertion id=A3 stream={stream_name} holds={str(all(records == expected_ring for records in ring.values()) and len(ring) == 2).lower()} "
         f"ring={ring}")
    chosen = {device: (slot.unsigned("slot_generation"), slot.slot_number, slot.unsigned("journal_tail"), slot.unsigned("journal_instance"))
              for device, slot in recovery.pool.chosen_slot_by_device.items()}
    emit(f"name=anchor_assertion id=A4 stream={stream_name} holds={str(len(chosen) == 2 and all(value == (5, 1, 3, 1) for value in chosen.values())).lower()} "
         f"chosen={chosen}")
    emit(f"name=anchor_assertion id=A5 stream={stream_name} holds={str(versions == [(0, 0), (1, 1), (1, 2), (1, 3)]).lower()} versions={versions}")
    slot_one_routes = {device: (reading.route, reading.spacing_read, reading.chosen_offset)
                       for device, reading in recovery.pool.slot_one_reading_by_device.items()}
    emit(f"name=anchor_assertion id=A13 stream={stream_name} holds={str(len(slot_one_routes) == 2 and all(value == ('1', 4096, 4096) for value in slot_one_routes.values())).lower()} "
         f"slot_one_routes={slot_one_routes}")
    magic_texts = {"system_configuration": layout.SYSTEM_CONFIGURATION_MAGIC.hex(), "root_record": layout.ROOT_RECORD_MAGIC.hex(),
                   "journal_record": layout.JOURNAL_RECORD_MAGIC.hex()}
    chosen_slot_magics = [slot.raw[:4].hex() for slot in recovery.pool.chosen_slot_by_device.values()]
    root_magics = [root.raw[:4].hex() for root in recovery.root_reading.self_verified_roots]
    record_magics = [record.raw[:4].hex() for records in recovery.ring_scan.valid_records_by_device.values() for record in records]
    emit(f"name=anchor_assertion id=A12 stream={stream_name} "
         f"holds={str(magic_texts == {'system_configuration': '53465342', 'root_record': '53465352', 'journal_record': '5346534a'} and len(chosen_slot_magics) == 2 and set(chosen_slot_magics) == {'53465342'} and len(root_magics) == 4 and set(root_magics) == {'53465352'} and len(record_magics) == 6 and set(record_magics) == {'5346534a'}).lower()} "
         f"magic_values={magic_texts} chosen_slots={len(chosen_slot_magics)} roots={len(root_magics)} records={len(record_magics)}")
    record = recovery.ring_scan.records_by_counter.get(3)
    fields = None if record is None else (record.unsigned("named_item_count"), record.unsigned("transaction_number"),
                                          record.unsigned("commit_marker"), record.unsigned("ordinal_within_publish"), int(record.is_last_of_publish))
    emit(f"name=anchor_assertion id=A7 stream={stream_name} holds={str(record is not None and record.jsn == (1, 3) and fields == (12, 1, 1, 1, 1)).lower()} "
         f"record_fields={fields}")
    witness_anchor_assertion_b1(stream_name, context, all_persisted, emit)


def witness_anchor_assertion_b1(stream_name: str, context: StreamContext, all_persisted: str, emit) -> None:
    """第 2 次重跑登记第七节 B1：流一 F 态 landed (1,3)、witness_counter 3、last_record、比的计数器 3、W 假；ack 是 (1,2) 那一版、
    ack_rotation_tail 2、tail_check true；终判定 = 前判定 = 在。依据 layout 第 152 行 tail 取值、D23 已定项 14 计数那一支。
    不符走第十节 F2（两边都查，不作废、不改锚点）。打成 name=witness_anchor_assertion，与 v1 的 anchor_assertion 行分开。"""
    state = next(state for state in context.dump.states if state.landing_vector == all_persisted)
    judgement = judge_state(context, state, UnitChecksumCache())
    summary = judgement.recovery_summary
    acknowledged = context.version_table.versions[summary["acknowledged_version"]].projection
    observed = {"landed": judgement.landed_text.split(",wm=")[0] + ")", "witness_counter": summary["witness_counter"],
                "witness_branch": summary["witness_branch"], "witness_compared": summary["witness_compared"],
                "witness_true": summary["witness_true"], "ack": (acknowledged.instance, acknowledged.checkpoint_txg),
                "ack_rotation_tail": summary["ack_rotation_tail"], "tail_check": summary["tail_check"],
                "provisional_before": summary["provisional_before_witness"], "provisional": judgement.provisional_verdict,
                "strict_before": summary["strict_before_witness"], "strict": judgement.strict_verdict}
    expected = {"landed": "(1,3)", "witness_counter": 3, "witness_branch": "last_record", "witness_compared": "3",
                "witness_true": "false", "ack": (1, 2), "ack_rotation_tail": 2, "tail_check": "true",
                "provisional_before": VERDICT_IN, "provisional": VERDICT_IN, "strict_before": VERDICT_IN, "strict": VERDICT_IN}
    differing = [key for key in expected if observed[key] != expected[key]]
    emit(f"name=witness_anchor_assertion id=B1 stream={stream_name} holds={str(not differing).lower()} "
         f"differing={','.join(differing) or '-'} observed={str(observed).replace(' ', '')}")


def landed_at_least_once(landing_vector: str, write_ordinals: tuple[int, ...]) -> bool:
    return any(landing_vector[write_ordinal] == "2" for write_ordinal in write_ordinals)


def witness_anchor_assertions_b2_b3(stream_name: str, context: StreamContext, outcomes: list[dict], emit) -> None:
    """第 2 次重跑登记第七节 B2、B3（流一批 1、批 2）。版本下标照 A5 钉的流一版本表：1、2 是暖机两次，3 是新池新建文件。
    B2：m(c) 不早于暖机那次调用的返回点（ack ≥ 2）、新池新建文件那次发布的轮换写一份都没落成 2 的每一态：witness_counter 2、ack_rotation_tail 2、tail_check true。
    B3：暖机两次发布的轮换写一份都没落成 2 的每一态：witness_counter 0、nothing、W 假。个数照报（0 照第六节 Q3-e 记）。"""
    versions = [(version.projection.instance, version.projection.checkpoint_txg) for version in context.version_table.versions]
    if versions != [(0, 0), (1, 1), (1, 2), (1, 3)]:
        emit(f"name=witness_anchor_assertion id=B2 stream={stream_name} holds=not_run detail=version_table_is_not_the_one_A5_pins")
        emit(f"name=witness_anchor_assertion id=B3 stream={stream_name} holds=not_run detail=version_table_is_not_the_one_A5_pins")
        return
    rotation_ordinals = {version_index: witness_predicates.rotation_write_ordinals_of_version(
        context.version_table, context.prepared_writes, context.base_pool, version_index) for version_index in (1, 2, 3)}
    landing_by_state = {state.state_number: state.landing_vector for state in context.dump.states}
    b2_states = [outcome for outcome in outcomes if outcome["acknowledged_version"] >= 2
                 and not landed_at_least_once(landing_by_state[outcome["state"]], rotation_ordinals[3])]
    b2_failing = [outcome["state"] for outcome in b2_states if (outcome["witness_counter"], outcome["ack_rotation_tail"], outcome["tail_check"]) != (2, 2, "true")]
    emit(f"name=witness_anchor_assertion id=B2 stream={stream_name} holds={str(not b2_failing).lower()} states={len(b2_states)} "
         f"failing_states={','.join(map(str, b2_failing)) or '-'} rotation_write_ordinals_of_version_3={list(rotation_ordinals[3])}")
    b3_states = [outcome for outcome in outcomes
                 if not landed_at_least_once(landing_by_state[outcome["state"]], rotation_ordinals[1] + rotation_ordinals[2])]
    b3_failing = [outcome["state"] for outcome in b3_states if (outcome["witness_counter"], outcome["witness_branch"], outcome["witness_true"]) != (0, "nothing", "false")]
    emit(f"name=witness_anchor_assertion id=B3 stream={stream_name} holds={str(not b3_failing).lower()} states={len(b3_states)} "
         f"failing_states={','.join(map(str, b3_failing)) or '-'} not_evaluated_states={sum(outcome['witness_branch'] == 'not_evaluated' for outcome in b3_states)} "
         f"rotation_write_ordinals_of_versions_1_2={list(rotation_ordinals[1] + rotation_ordinals[2])}")


def rotation_lines(stream_name: str, context: StreamContext, emit) -> None:
    """每一版那次发布的系统配置轮换写（登记读法 3 的 τ）与那一版末条记录的计数器（Q3-f 附带）。"""
    for rotation in witness_predicates.rotations_of_every_version(context):
        emit(f"name=rotation_tail stream={stream_name} version={rotation.version_index} "
             f"rotation_write_ordinals={','.join(map(str, rotation.rotation_write_ordinals)) or '-'} "
             f"self_verified_rotation_tails={','.join(map(str, rotation.self_verified_rotation_tails)) or '-'} "
             f"tau={'-' if rotation.rotation_tail is None else rotation.rotation_tail} "
             f"last_record_counter={'-' if rotation.last_record_counter is None else rotation.last_record_counter} "
             f"tails_differ_across_writes={str(len(set(rotation.self_verified_rotation_tails)) > 1).lower()}")


def torn_reading_disagreements(context: StreamContext) -> tuple[int, int, int]:
    """Q2-g（附带）：执行器照 D13 已定项 4 自己认的三态写（固定结构区、长于一个扇区、罩住的范围里原来有东西）与撕裂字节
    （新旧不同那一截前一半新、后一半旧，一半取下整）与 torn.tsv 逐条比。返回 (执行器认的条数, torn.tsv 的条数, 不同的条数)。"""
    image = build_crash_image(context.dump, context.prepared_writes, "0" * len(context.prepared_writes))
    unit_area_start = context.base_pool.unsigned("unit_area_start_slot") * 16384
    executor_torn = {}
    for prepared in context.prepared_writes:
        old = image.read_bytes(prepared.device_identity, prepared.byte_offset, prepared.byte_count)
        if prepared.byte_offset < unit_area_start and prepared.byte_count > 512 and any(old):
            differing = [index for index in range(prepared.byte_count) if old[index] != prepared.new_content[index]]
            if differing:
                split = differing[0] + (differing[-1] - differing[0] + 1) // 2
                executor_torn[prepared.write_ordinal] = prepared.new_content[:split] + old[split:]
            else:
                executor_torn[prepared.write_ordinal] = prepared.new_content
        image.write_bytes(prepared.device_identity, prepared.byte_offset, prepared.new_content)
    dumped = context.dump.torn_payload_by_write_ordinal
    differing_entries = sum(1 for ordinal in set(executor_torn) | set(dumped) if executor_torn.get(ordinal) != dumped.get(ordinal))
    return len(executor_torn), len(dumped), differing_entries


def freeze_lines(emit) -> None:
    """登记 5.6 冻结：执行器目录里的源文件（不含比对器与变异驱动）逐个算 sha256；种子 = 按路径排序逐个喂进 sha256 的摘要前 16 个十六进制字符（5.5 盲测族）。"""
    seed_hasher = hashlib.sha256()
    for file_name in sorted(os.listdir(EXECUTOR_DIRECTORY)):
        path = os.path.join(EXECUTOR_DIRECTORY, file_name)
        if not os.path.isfile(path) or file_name.startswith(FILES_OUTSIDE_THE_FREEZE) or file_name.endswith(".pyc"):
            continue
        with open(path, "rb") as handle:
            content = handle.read()
        seed_hasher.update(content)
        emit(f"name=freeze file={file_name} sha256={hashlib.sha256(content).hexdigest()}")
    emit(f"name=freeze_seed sha256_prefix16={seed_hasher.hexdigest()[:16]}")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 O3 规约执行器：判每个取样崩溃状态落不落在规约允许的集合里")
    parser.add_argument("--dumps", required=True, help="导出目录（每条流一个子目录，登记 5.3）")
    parser.add_argument("--streams", default=",".join(STREAM_NAMES))
    parser.add_argument("--batches", default="1,2,3")
    parser.add_argument("--workers", type=int, required=True, help="工作进程数，不超过派发提示的线程上限")
    parser.add_argument("--output", required=True, help="产物路径；已存在就拒绝（不覆盖）")
    parser.add_argument("--skip-torn-reading", action="store_true", help="不算 Q2-g（附带，够判后不跑）")
    arguments = parser.parse_args(argument_list)
    if os.path.exists(arguments.output):
        print(f"产物 {arguments.output} 已存在，不覆盖", file=sys.stderr)
        return 2
    with open(arguments.output, "x", encoding="utf-8", buffering=1) as output:
        def emit(line: str) -> None:
            output.write(line + "\n")

        emit(f"name=config dumps={arguments.dumps} streams={arguments.streams} batches={arguments.batches} "
             f"workers={arguments.workers} python={sys.version.split()[0]} crc32c={crc32c_castagnoli.LIBRARY_IMPLEMENTATION_NAME} "
             f"crc32c_self_check={'pass' if not crc32c_castagnoli.LOAD_TIME_DISAGREEMENTS else 'fail'}")
        for stream_name in arguments.streams.split(","):
            directory = os.path.join(arguments.dumps, stream_name)
            started = time.perf_counter_ns()
            dump = load_stream_dump(directory)
            loaded = time.perf_counter_ns()
            try:
                context = prepare_stream_context(dump)
            except BaseImageNotRecoverable as error:
                emit(f"name=base_recovery stream={stream_name} recovered=false detail={str(error).replace(' ', '_')}")
                continue
            prepared = time.perf_counter_ns()
            contexts_by_stream[stream_name] = context
            emit(f"name=load stream={stream_name} load_ns={loaded - started} prepare_ns={prepared - loaded} "
                 f"ops={len(dump.operations)} writes={len(dump.write_operation_indices)} calls={len(dump.calls)} "
                 f"sampled_states={len(dump.states)} torn_writes={len(dump.torn_payload_by_write_ordinal)}")
            emit(f"name=dump_check stream={stream_name} count_mismatches={len(dump.count_mismatches)} "
                 f"checksum_mismatches={len(dump.checksum_mismatches)} version_table_anomalies={len(context.version_table.anomalies)}")
            for mismatch in dump.count_mismatches + dump.checksum_mismatches + context.version_table.anomalies:
                emit(f"name=dump_problem stream={stream_name} detail={mismatch.replace(' ', '_')}")
            for version in context.version_table.versions:
                emit(f"name=version stream={stream_name} index={version.version_index} projection={version.projection.short_text()} "
                     f"root_write={version.root_write_stream_index} return_point={version.return_point} call={version.call_label}")
            anchor_assertions(stream_name, context, emit)
            rotation_lines(stream_name, context, emit)
            if not arguments.skip_torn_reading:
                executor_count, dumped_count, differing = torn_reading_disagreements(context)
                emit(f"name=torn_reading stream={stream_name} executor_third_state_writes={executor_count} "
                     f"dumped_third_state_writes={dumped_count} differing_entries={differing}")
        first_stream_outcomes_of_batches_one_and_two = []
        for batch_number in (int(token) for token in arguments.batches.split(",")):
            results = run_batch(batch_number, arguments.workers, emit)
            summarise_verdicts(batch_number, results, emit)
            if batch_number in (1, 2):
                first_stream_outcomes_of_batches_one_and_two += [outcome for outcome in results if outcome["stream"] == STREAM_NAMES[0]]
        if STREAM_NAMES[0] in contexts_by_stream:
            witness_anchor_assertions_b2_b3(STREAM_NAMES[0], contexts_by_stream[STREAM_NAMES[0]], first_stream_outcomes_of_batches_one_and_two, emit)
        freeze_lines(emit)
        emit("name=complete")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
