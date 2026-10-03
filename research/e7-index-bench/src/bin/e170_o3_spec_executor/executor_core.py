"""把步 1–10 串成一态的判定（登记 5.2）。不读导出给的布局分类，不读实现判定。"""

from __future__ import annotations

import time
from dataclasses import dataclass

import spec_gaps
from crash_dump import SampledState, StreamDump
from crash_image import LANDED_TORN, PreparedWrite, SparseCrashImage, build_crash_image, copy_base_image, prepare_writes
from journal_ring import RingScan, scan_journal_ring
from replay_prefix import PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE, PATH_ANCHOR_UNREADABLE_NO_HEAD, ReplayOutcome, \
    UnitChecksumCache, replay_prefix
from root_ring import RootRingReading, SelfVerifiedRoot, VersionProjection, choose_root, read_root_ring
from system_configuration import PoolRefusal, PoolSystemConfiguration, choose_system_configuration
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


def judge_state(context: StreamContext, state: SampledState, cache: UnitChecksumCache) -> StateJudgement:
    started = time.perf_counter_ns()
    gaps = spec_gaps.GapsHitInOneState()
    image = build_crash_image(context.dump, context.prepared_writes, state.landing_vector)  # 步 1
    digest_matches = image.digest() == state.image_digest  # 登记第十一节 S2
    recovery = recover(image, context.dump.device_identities, context.dump.sector_bytes, cache, gaps)
    allowed = allowed_versions(context.version_table, state.landing_vector)  # 步 9
    # 步 10：L(c) ∈ A(c) 判「在」；L(c) ∉ A(c) 或 L(c) = 无判「不在」（登记第一节读法 5）。
    landed_is_allowed = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)
    provisional = VERDICT_IN if landed_is_allowed else VERDICT_NOT_IN
    strict = VERDICT_SPEC_GAP if gaps.identifiers else provisional
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
        "torn": int(LANDED_TORN in state.landing_vector),
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
            f"chosen_below_largest={summary['chosen_root_below_largest_slot_txg']} applied={summary['applied']} "
            f"last_counter={summary['last_applied_counter']} stop={summary['stop']} path={summary['path']} "
            f"anchor_unreadable={summary['anchor_unreadable']} torn={summary['torn']} "
            f"time_ns={judgement.elapsed_nanoseconds} worker={worker_identity}")
