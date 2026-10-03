"""把步 1–10 串成一态的判定（登记 5.2）。不读导出给的布局分类，不读实现判定。"""

from __future__ import annotations

import time
from dataclasses import dataclass

import spec_gaps
import witness_predicates
from crash_dump import SampledState, StreamDump
from crash_image import LANDED_TORN, PreparedWrite, SparseCrashImage, build_crash_image, copy_base_image, prepare_writes
from journal_ring import RingScan, scan_journal_ring
from replay_prefix import PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE, PATH_ANCHOR_UNREADABLE_NO_HEAD, ReplayOutcome, \
    UnitChecksumCache, replay_prefix
from root_ring import RootRingReading, SelfVerifiedRoot, VersionProjection, choose_root, read_root_ring
from system_configuration import SLOT_ONE_ROUTE_EVERY_SPACING, SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL, \
    SLOT_ONE_ROUTE_OWN_SLOT_ZERO, PoolRefusal, PoolSystemConfiguration, choose_system_configuration
from version_table import AllowedVersions, VersionTable, allowed_versions, build_version_table

VERDICT_IN = "in"
VERDICT_NOT_IN = "not_in"
VERDICT_SPEC_GAP = "spec_gap"


@dataclass
class Recovery:
    pool: PoolSystemConfiguration | None
    root_reading: RootRingReading | None
    chosen_root: SelfVerifiedRoot | None
    ring_scan: RingScan | None
    replay: ReplayOutcome | None
    landed: VersionProjection | None  # None = L(c) 无
    refusal_reason: str | None


def recover(image: SparseCrashImage, device_identities: list[int], probed_sector_bytes: int,
            cache: UnitChecksumCache, gaps: spec_gaps.GapsHitInOneState) -> Recovery:
    """步 2–8。按条款整池拒、或一条根都择不出时 L(c) = 无（登记第一节读法 4）。"""
    pool = choose_system_configuration(image, device_identities, gaps)
    if isinstance(pool, PoolRefusal):
        return Recovery(None, None, None, None, None, None, pool.reason)
    root_reading = read_root_ring(image, pool, probed_sector_bytes, gaps)
    chosen_root = choose_root(root_reading, pool, gaps)
    if chosen_root is None:
        return Recovery(pool, root_reading, None, None, None, None, "no_self_verified_root")
    ring_scan = scan_journal_ring(image, pool, gaps)
    if ring_scan.unknown_record_type_seen:
        # D23 已定项 1 A 条：未知类型 ⇒ 拒绝挂载。
        return Recovery(pool, root_reading, chosen_root, ring_scan, None, None, "unknown_journal_record_type")
    replay = replay_prefix(chosen_root, ring_scan, image, pool, cache, gaps)
    if isinstance(replay, PoolRefusal):
        return Recovery(pool, root_reading, chosen_root, ring_scan, None, None, replay.reason)
    return Recovery(pool, root_reading, chosen_root, ring_scan, replay, replay.landed, None)


@dataclass
class StreamContext:
    dump: StreamDump
    prepared_writes: list[PreparedWrite]
    base_pool: PoolSystemConfiguration
    version_table: VersionTable


def prepare_stream_context(dump: StreamDump) -> StreamContext:
    """版本 0 = 执行器在基镜像上自己恢复得出的那一版；根槽与系统配置槽的位置取基镜像所择系统配置的几何。"""
    prepared_writes = prepare_writes(dump)
    base_image = copy_base_image(dump)
    base_recovery = recover(base_image, dump.device_identities, dump.sector_bytes, UnitChecksumCache(),
                            spec_gaps.GapsHitInOneState())
    if base_recovery.pool is None or base_recovery.landed is None:
        raise BaseImageNotRecoverable(f"基镜像上恢复不出现行那一版：{base_recovery.refusal_reason}")
    table = build_version_table(dump, prepared_writes, base_recovery.pool, base_recovery.landed)
    return StreamContext(dump, prepared_writes, base_recovery.pool, table)


class BaseImageNotRecoverable(Exception):
    """基镜像上择不出系统配置或根：版本 0 与根槽几何都无从得出，这条流不判（驱动照打一行，不当结果）。"""


@dataclass(frozen=True)
class StateJudgement:
    state: SampledState
    digest_matches: bool
    strict_verdict: str
    provisional_verdict: str
    gaps_hit: list[str]
    recovery_summary: dict
    allowed: AllowedVersions
    landed_text: str
    elapsed_nanoseconds: int


def projection_text(projection: VersionProjection | None) -> str:
    return "none" if projection is None else projection.short_text()


def slot_one_trace_text(pool: PoolSystemConfiguration | None) -> str:
    """逐盘的轨迹：槽 1 走 ① ② ③ 哪一步（r1 r2 r3；③ 带收到几槽 c<n>），这块盘最后择定的系统配置槽是几号槽、在哪个偏移、世代号多少。"""
    if pool is None:
        return "-"
    parts = []
    for device_identity in sorted(pool.slot_one_reading_by_device):
        reading = pool.slot_one_reading_by_device[device_identity]
        chosen = pool.chosen_slot_by_device.get(device_identity)
        chosen_text = "none" if chosen is None else f"s{chosen.slot_number}@{chosen.byte_offset}g{chosen.unsigned('slot_generation')}"
        collected_text = f"c{len(reading.collected_offsets)}" if reading.route == SLOT_ONE_ROUTE_EVERY_SPACING else ""
        parts.append(f"d{device_identity}:r{reading.route}{collected_text}:{chosen_text}")
    return ",".join(parts) or "-"


def count_disks_by_slot_one_route(pool: PoolSystemConfiguration | None, route: str) -> int:
    return 0 if pool is None else sum(reading.route == route for reading in pool.slot_one_reading_by_device.values())


def count_disks_where_every_spacing_collected_two_or_more_slots(pool: PoolSystemConfiguration | None) -> int:
    return 0 if pool is None else sum(reading.route == SLOT_ONE_ROUTE_EVERY_SPACING and len(reading.collected_offsets) >= 2
                                      for reading in pool.slot_one_reading_by_device.values())


def verdicts_after_the_witness_predicates(provisional_before_witness: str, strict_before_witness: str,
                                          evaluation: witness_predicates.WitnessEvaluation) -> tuple[str, str]:
    """改二（E170 第 2 次重跑登记读法 4）：前判定「在」且 W 真 ⇒「不在」；前判定「在」且 T 假 ⇒「不在」；其余照前判定。
    严格跑法前判定是「规约欠一句」的照旧（v1 的欠一句表 G01–G22；W、T 这一次没有新记的欠一句）。返回 (暂代, 严格)。"""
    predicates_fail = evaluation.witness_true is True or evaluation.tail_check == witness_predicates.TAIL_CHECK_FALSE

    def after(verdict_before: str) -> str:
        return VERDICT_NOT_IN if verdict_before == VERDICT_IN and predicates_fail else verdict_before

    return after(provisional_before_witness), after(strict_before_witness)


def judge_state(context: StreamContext, state: SampledState, cache: UnitChecksumCache) -> StateJudgement:
    started = time.perf_counter_ns()
    gaps = spec_gaps.GapsHitInOneState()
    image = build_crash_image(context.dump, context.prepared_writes, state.landing_vector)  # 步 1
    digest_matches = image.digest() == state.image_digest  # 登记第十一节 S2
    recovery = recover(image, context.dump.device_identities, context.dump.sector_bytes, cache, gaps)
    allowed = allowed_versions(context.version_table, state.landing_vector)  # 步 9
    # 步 10：L(c) ∈ A(c) 判「在」；L(c) ∉ A(c) 或 L(c) = 无判「不在」（登记第一节读法 5）。这是 v1 的判定，下称前判定。
    landed_is_allowed = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)
    provisional_before_witness = VERDICT_IN if landed_is_allowed else VERDICT_NOT_IN
    strict_before_witness = VERDICT_SPEC_GAP if gaps.identifiers else provisional_before_witness
    # 改二：前判定照旧算完再过 W、T（witness_predicates.py，登记读法 2、3、4），得终判定。
    evaluation = witness_predicates.evaluate_witness_predicates(context, image, recovery, allowed)
    provisional, strict = verdicts_after_the_witness_predicates(provisional_before_witness, strict_before_witness, evaluation)
    replay = recovery.replay
    chosen = recovery.chosen_root
    summary = {
        "refusal": recovery.refusal_reason or "-",
        "chosen_root": "none" if chosen is None else
        f"({chosen.unsigned('instance')},{chosen.unsigned('checkpoint_txg')})@r{chosen.location.region}s{chosen.location.slot_in_region}",
        "chosen_root_below_largest_slot_txg": int(chosen is not None and recovery.root_reading is not None
                                                  and recovery.root_reading.largest_txg_in_any_nonzero_root_slot is not None
                                                  and chosen.unsigned("checkpoint_txg") < recovery.root_reading.largest_txg_in_any_nonzero_root_slot),
        "applied": 0 if replay is None else len(replay.applied_records),
        "last_applied_counter": "-" if replay is None or not replay.applied_records else replay.applied_records[-1].unsigned("jsn_counter"),
        "stop": "-" if replay is None else replay.stop_reason,
        "path": "-" if replay is None else replay.chain_path,
        "anchor_unreadable": int(replay is not None and replay.chain_path in (PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE,
                                                                             PATH_ANCHOR_UNREADABLE_NO_HEAD)),
        "chosen_root_below_largest_self_verified_txg": int(
            chosen is not None and recovery.root_reading is not None and bool(recovery.root_reading.self_verified_roots)
            and chosen.unsigned("checkpoint_txg") < max(root.unsigned("checkpoint_txg") for root in recovery.root_reading.self_verified_roots)),
        "slot_one": slot_one_trace_text(recovery.pool),
        "slot_one_route_one_disks": count_disks_by_slot_one_route(recovery.pool, SLOT_ONE_ROUTE_OWN_SLOT_ZERO),
        "slot_one_route_two_disks": count_disks_by_slot_one_route(recovery.pool, SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL),
        "slot_one_route_three_disks": count_disks_by_slot_one_route(recovery.pool, SLOT_ONE_ROUTE_EVERY_SPACING),
        "slot_one_route_three_two_or_more_collected_disks": count_disks_where_every_spacing_collected_two_or_more_slots(recovery.pool),
        "torn": int(LANDED_TORN in state.landing_vector),
        # 改二：登记读法 4 要判定行另加的字段（v1 的字段与字段名一个不改；strict / provisional 写终判定）
        "strict_before_witness": strict_before_witness,
        "provisional_before_witness": provisional_before_witness,
        "witness_counter": "-" if evaluation.witness_counter is None else evaluation.witness_counter,
        "witness_branch": evaluation.witness_branch,
        "witness_compared": evaluation.witness_compared,
        "witness_true": "not_evaluated" if evaluation.witness_true is None else str(evaluation.witness_true).lower(),
        "ack_rotation_tail": "-" if evaluation.ack_rotation_tail is None else evaluation.ack_rotation_tail,
        "tail_check": evaluation.tail_check,
        "failed_predicates": evaluation.failed_predicates,
        "acknowledged_version": allowed.acknowledged_version_index,
    }
    elapsed = time.perf_counter_ns() - started
    return StateJudgement(state, digest_matches, strict, provisional, gaps.sorted_identifiers(), summary, allowed,
                          projection_text(recovery.landed), elapsed)


def judgement_line(stream_name: str, judgement: StateJudgement, worker_identity: int) -> str:
    state = judgement.state
    summary = judgement.recovery_summary
    allowed_text = "[" + ",".join(projection_text(version.projection) for version in judgement.allowed.allowed) + "]"
    return (f"name=state stream={stream_name} state={state.state_number} layer0={state.layer_zero_sequence} "
            f"segment={state.segment_number} batch={state.batch_number} "
            f"digest={'match' if judgement.digest_matches else 'mismatch'} strict={judgement.strict_verdict} "
            f"provisional={judgement.provisional_verdict} gaps={','.join(judgement.gaps_hit) or '-'} "
            f"landed={judgement.landed_text} refusal={summary['refusal']} allowed={allowed_text} "
            f"m={judgement.allowed.largest_landed_write_stream_index} ack={judgement.allowed.acknowledged_version_index} "
            f"begun={judgement.allowed.begun_version_index} chosen_root={summary['chosen_root']} "
            f"chosen_below_largest={summary['chosen_root_below_largest_slot_txg']} "
            f"chosen_below_largest_self_verified={summary['chosen_root_below_largest_self_verified_txg']} "
            f"slot_one={summary['slot_one']} applied={summary['applied']} "
            f"last_counter={summary['last_applied_counter']} stop={summary['stop']} path={summary['path']} "
            f"anchor_unreadable={summary['anchor_unreadable']} torn={summary['torn']} "
            f"strict_before_witness={summary['strict_before_witness']} provisional_before_witness={summary['provisional_before_witness']} "
            f"witness_counter={summary['witness_counter']} witness_branch={summary['witness_branch']} "
            f"witness_compared={summary['witness_compared']} witness_true={summary['witness_true']} "
            f"ack_rotation_tail={summary['ack_rotation_tail']} tail_check={summary['tail_check']} "
            f"failed_predicates={summary['failed_predicates']} "
            f"time_ns={judgement.elapsed_nanoseconds} worker={worker_identity}")
