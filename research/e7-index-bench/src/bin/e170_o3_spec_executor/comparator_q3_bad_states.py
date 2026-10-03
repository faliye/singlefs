"""E170（O3规约执行器第一版） 第三段：冻结的执行器判坏状态（登记 5.5），再由比对器对期望（问题单 Q3）。

判的那一半只调冻结的执行器：每行按底态落地向量建镜像、择系统配置、择根、扫环、取前缀、算 A(c)、判在 / 不在，全走
executor_core.judge_state 原样；唯一的改动是判坏状态期间把 executor_core 模块里 build_crash_image 这个名字换成「原样建完、再叠上
bad-patches.bin 里这一行的补丁」，判完换回去。A(c) 因此照底态向量算（登记第一节读法 3：补丁不算写）。judge_state 里那一句镜像摘要比对
比的是补丁之后的摘要（导出装置从 CrashImage 算的那一份），另核补丁之前的摘要与导出写的底态摘要相同（登记第十一节 S2 同形）。
判的那一半不打开 bad-expected.tsv 与 bad-impl-verdicts.tsv；判定行全部写出之后，比对那一半才打开它们（登记 5.5「期望答案…只给比对器读」）。

跑法（在本目录，经 run-with-memory-cap.sh 与 capped.sh）：
  python3 comparator_q3_bad_states.py --dumps <导出目录> --frozen-verdicts <第二段产物> --output <新文件>
文件名以 comparator 起头，不进冻结的 sha256（executor_main.FILES_OUTSIDE_THE_FREEZE）。
"""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import dataclass

import executor_core
import executor_main
from comparator_result_lines import ResultLines, field
from crash_dump import SampledState, load_stream_dump, read_tab_separated_rows
from executor_core import VERDICT_IN, VERDICT_NOT_IN, VERDICT_SPEC_GAP, StateJudgement, StreamContext, judge_state, \
    prepare_stream_context, projection_text
from replay_prefix import UnitChecksumCache
from root_ring import VersionProjection

BAD_STATES_FILE = "bad-states.tsv"
BAD_PATCHES_FILE = "bad-patches.bin"
BAD_EXPECTED_FILE = "bad-expected.tsv"
BAD_IMPLEMENTATION_VERDICTS_FILE = "bad-impl-verdicts.tsv"
BAD_SEED_FILE = "bad-seed.tsv"
FAMILY_POSITIVE = "P"
FAMILY_NEGATIVE = "N"
FAMILY_BLIND = "B"
EXPECTED_LANDING_NOTHING = "none"
EXPECTED_LANDING_OUTSIDE = "outside_version_table"


@dataclass(frozen=True)
class BadStateRow:
    row_id: int
    landing_vector: str
    base_digest: str
    patched_digest: str
    patches: tuple[tuple[int, int, bytes], ...]  # (盘身份, 盘内偏移, 字节)


def read_bad_states(stream_directory: str) -> list[BadStateRow]:
    """执行器那一侧唯一读的两个坏状态文件：行号、底态落地向量、两个摘要、补丁（不带族名与期望）。"""
    with open(os.path.join(stream_directory, BAD_PATCHES_FILE), "rb") as handle:
        blob = handle.read()
    rows = []
    for columns in read_tab_separated_rows(os.path.join(stream_directory, BAD_STATES_FILE)):
        row_id, landing_vector, base_digest, patched_digest, patch_list = columns[:5]
        patches = []
        for entry in patch_list.split(","):
            device, offset, start, length = (int(token) for token in entry.split(":"))
            assert start + length <= len(blob), f"行 {row_id} 的补丁越过 bad-patches.bin 末尾"
            patches.append((device, offset, blob[start:start + length]))
        rows.append(BadStateRow(int(row_id), landing_vector, base_digest.lower(), patched_digest.lower(), tuple(patches)))
    return rows


@dataclass(frozen=True)
class BadJudgement:
    stream: str
    row_id: int
    base_digest_matches: bool
    judgement: StateJudgement | None
    landed_projection: VersionProjection | None  # 判定行里只有投影的短文本（不带两条指针），比对落点要整个投影
    failure: str | None


def judge_bad_states(context: StreamContext, rows: list[BadStateRow]) -> list[BadJudgement]:
    """build_crash_image 换成「建完再叠补丁」；recover 只包一层记下它交回的落点投影（不改它的输入与输出）。判完两个名字都换回去。"""
    original_builder = executor_core.build_crash_image
    original_recover = executor_core.recover
    current: dict = {"patches": (), "base_digest": None, "landed": None}

    def builder_with_patches(dump, prepared_writes, landing_vector):
        image = original_builder(dump, prepared_writes, landing_vector)
        current["base_digest"] = image.digest()
        for device, offset, payload in current["patches"]:
            image.write_bytes(device, offset, payload)
        return image

    def recover_and_note_the_landing(*arguments):
        recovery = original_recover(*arguments)
        current["landed"] = recovery.landed
        return recovery

    results = []
    executor_core.build_crash_image = builder_with_patches
    executor_core.recover = recover_and_note_the_landing
    try:
        for row in rows:
            current["patches"] = row.patches
            current["base_digest"] = None
            current["landed"] = None
            state = SampledState(row.row_id, -1, -1, 0, row.landing_vector, row.patched_digest)
            try:
                judgement, failure = judge_state(context, state, UnitChecksumCache()), None
            except Exception as error:  # 判定行缺失按登记第十一节 V4 处置：照打出错，不吞
                judgement, failure = None, f"{type(error).__name__}:{str(error)[:200]}"
            results.append(BadJudgement(context.dump.stream_name, row.row_id, current["base_digest"] == row.base_digest,
                                        judgement, current["landed"], failure))
    finally:
        executor_core.build_crash_image = original_builder
        executor_core.recover = original_recover
    return results


def bad_judgement_line(bad: BadJudgement) -> str:
    if bad.judgement is None:
        return f"name=bad_judgement stream={bad.stream} row={bad.row_id} strict=error provisional=error error={field(bad.failure)}"
    judgement = bad.judgement
    summary = judgement.recovery_summary
    allowed_text = "[" + ",".join(projection_text(version.projection) for version in judgement.allowed.allowed) + "]"
    return (f"name=bad_judgement stream={bad.stream} row={bad.row_id} "
            f"patched_digest={'match' if judgement.digest_matches else 'mismatch'} "
            f"base_digest={'match' if bad.base_digest_matches else 'mismatch'} strict={judgement.strict_verdict} "
            f"provisional={judgement.provisional_verdict} gaps={','.join(judgement.gaps_hit) or '-'} "
            f"landed={judgement.landed_text} refusal={field(summary['refusal'])} allowed={allowed_text} "
            f"m={judgement.allowed.largest_landed_write_stream_index} ack={judgement.allowed.acknowledged_version_index} "
            f"begun={judgement.allowed.begun_version_index} chosen_root={summary['chosen_root']} applied={summary['applied']} "
            f"last_counter={summary['last_applied_counter']} stop={summary['stop']} path={summary['path']}")


@dataclass(frozen=True)
class ExpectedRow:
    row_id: int
    name: str
    family: str
    base_label: str
    base_call: str
    verdict: str
    landing: str
    layer_zero_match: str


def read_expected(stream_directory: str) -> dict[int, ExpectedRow]:
    return {int(columns[0]): ExpectedRow(int(columns[0]), *columns[1:8])
            for columns in read_tab_separated_rows(os.path.join(stream_directory, BAD_EXPECTED_FILE))}


def read_implementation_both_pass(stream_directory: str) -> dict[int, str]:
    """bad-impl-verdicts.tsv 最后一列：实现两判都放行（池级 checker 一条 Violated 都没有，记录核对器两项都为假；登记第一节读法 6）。"""
    return {int(columns[0]): columns[9]
            for columns in read_tab_separated_rows(os.path.join(stream_directory, BAD_IMPLEMENTATION_VERDICTS_FILE))}


@dataclass(frozen=True)
class RowComparison:
    stream: str
    expected: ExpectedRow
    strict: str
    provisional: str
    landed: str
    verdict_matches: bool
    landing_matches: bool
    implementation_both_pass: str


def compare_row(bad: BadJudgement, expected: ExpectedRow, implementation_both_pass: str,
                versions: list[VersionProjection]) -> RowComparison:
    """期望落点三种：none（L = 无）、outside_version_table（L 有、不等于版本表里任何一版的整个投影，两条指针也比）、
    某一版的投影短文本（导出装置写的；在版本表里按短文本找到那一版——(实例代号, txg) 在表里唯一——再比整个投影）。
    版本表取执行器自己建的那一份（第二段 A5、A6 锚点核过它）。"""
    strict = "error" if bad.judgement is None else bad.judgement.strict_verdict
    provisional = "error" if bad.judgement is None else bad.judgement.provisional_verdict
    landed = "error" if bad.judgement is None else bad.judgement.landed_text
    if bad.judgement is None:
        landing_matches = False
    elif expected.landing == EXPECTED_LANDING_NOTHING:
        landing_matches = bad.landed_projection is None
    elif expected.landing == EXPECTED_LANDING_OUTSIDE:
        landing_matches = bad.landed_projection is not None and bad.landed_projection not in versions
    else:
        named = [version for version in versions if projection_text(version) == expected.landing]
        landing_matches = len(named) == 1 and bad.landed_projection == named[0]
    return RowComparison(bad.stream, expected, strict, provisional, landed, provisional == expected.verdict,
                         landing_matches, implementation_both_pass)


def judge_and_compare(dumps: str, emit_judgement=None) -> tuple[list[RowComparison], dict[str, StreamContext], list[BadJudgement]]:
    """两条流：先全部判完（判定行交 emit_judgement），再打开期望与实现判定逐行比。变异驱动的检查也调这一个函数。"""
    contexts: dict[str, StreamContext] = {}
    judgements_by_stream: dict[str, list[BadJudgement]] = {}
    for stream_name in executor_main.STREAM_NAMES:
        context = prepare_stream_context(load_stream_dump(os.path.join(dumps, stream_name)))
        contexts[stream_name] = context
        judgements_by_stream[stream_name] = judge_bad_states(context, read_bad_states(os.path.join(dumps, stream_name)))
        if emit_judgement is not None:
            for bad in judgements_by_stream[stream_name]:
                emit_judgement(bad)
    comparisons = []
    for stream_name, judgements in judgements_by_stream.items():
        directory = os.path.join(dumps, stream_name)
        expected_rows = read_expected(directory)
        both_pass = read_implementation_both_pass(directory)
        assert sorted(expected_rows) == [bad.row_id for bad in judgements], f"{stream_name}：期望表与坏状态表的行号对不上（V4）"
        versions = [version.projection for version in contexts[stream_name].version_table.versions]
        for bad in judgements:
            comparisons.append(compare_row(bad, expected_rows[bad.row_id], both_pass[bad.row_id], versions))
    return comparisons, contexts, [bad for judgements in judgements_by_stream.values() for bad in judgements]


def comparison_line(comparison: RowComparison) -> str:
    expected = comparison.expected
    return (f"name=bad_comparison stream={comparison.stream} row={expected.row_id} name={expected.name} family={expected.family} "
            f"base={expected.base_label} layer0_match={expected.layer_zero_match} expected={expected.verdict} "
            f"provisional={comparison.provisional} strict={comparison.strict} verdict_matches={str(comparison.verdict_matches).lower()} "
            f"expected_landing={expected.landing} landed={comparison.landed} "
            f"landing_matches={str(comparison.landing_matches).lower()} impl_both_pass={comparison.implementation_both_pass}")


def count_lines(comparisons: list[RowComparison], emit) -> dict[str, int]:
    """登记第六节 Q3-a…Q3-f 与第八节 G5（底态 F 对 Mk）。每个量一行，逐条流与两条流合计各报。"""
    totals: dict[str, int] = {}
    for scope in list(executor_main.STREAM_NAMES) + ["both"]:
        rows = [comparison for comparison in comparisons if scope in ("both", comparison.stream)]
        positive = [row for row in rows if row.expected.family == FAMILY_POSITIVE]
        blind = [row for row in rows if row.expected.family == FAMILY_BLIND]
        negative = [row for row in rows if row.expected.family == FAMILY_NEGATIVE]
        counts = {
            "q3a_positive_rows": len(positive),
            "q3a_positive_judged_in": sum(row.provisional == VERDICT_IN for row in positive),
            "q3a_positive_judged_in_on_base_f": sum(row.provisional == VERDICT_IN for row in positive if row.expected.base_label == "F"),
            "q3a_positive_judged_in_on_base_mk": sum(row.provisional == VERDICT_IN for row in positive if row.expected.base_label != "F"),
            "q3b_blind_rows": len(blind),
            "q3b_blind_judged_in": sum(row.provisional == VERDICT_IN for row in blind),
            "q3c_strict_gap_without_provisional_verdict": sum(row.strict == VERDICT_SPEC_GAP and row.provisional not in (VERDICT_IN, VERDICT_NOT_IN)
                                                               for row in positive + blind),
            "strict_spec_gap_rows": sum(row.strict == VERDICT_SPEC_GAP for row in rows),
            "judgement_failures": sum(row.provisional == "error" for row in rows),
            "q3d_positive_landing_differs": sum(not row.landing_matches for row in positive),
            "q3e_negative_rows": len(negative),
            "q3e_negative_judged_not_in": sum(row.provisional == VERDICT_NOT_IN for row in negative),
            "q3e_negative_landing_differs": sum(not row.landing_matches for row in negative),
            "q3f_impl_both_pass_on_positive_and_blind": sum(row.implementation_both_pass == "true" for row in positive + blind),
            "impl_both_pass_on_negative": sum(row.implementation_both_pass == "true" for row in negative),
        }
        emit(f"name=q3_counts scope={scope} " + " ".join(f"{key}={value}" for key, value in counts.items()))
        if scope == "both":
            totals = counts
    return totals


def freeze_lines_now() -> list[str]:
    """executor_main.freeze_lines 原样算一遍此刻执行器目录的冻结行（不含 comparator*、mutation_driver*）。"""
    captured: list[str] = []
    executor_main.freeze_lines(captured.append)
    return captured


def frozen_lines_of_segment_two(frozen_verdicts: str) -> list[str]:
    with open(frozen_verdicts, encoding="utf-8") as handle:
        return [line.rstrip("\n") for line in handle if line.startswith(("name=freeze ", "name=freeze_seed "))]


def seed_of_the_export(dumps: str, stream_name: str) -> str:
    rows = dict((columns[0], columns[1]) for columns in
                (line.rstrip("\n").split("\t") for line in open(os.path.join(dumps, stream_name, BAD_SEED_FILE), encoding="utf-8")))
    return rows["seed_hex"]


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第三段：冻结的执行器判坏状态，比对器对期望（Q3）")
    parser.add_argument("--dumps", required=True, help="导出目录（每条流一个子目录，里面已有 bad-*.tsv）")
    parser.add_argument("--frozen-verdicts", required=True, help="第二段产物（取其中 name=freeze 行核冻结没变）")
    parser.add_argument("--output", required=True, help="产物路径；已存在就拒绝")
    arguments = parser.parse_args(argument_list)
    lines = ResultLines(arguments.output)
    lines.result(f"name=config segment=s3 mode=bad_state_judgement streams={','.join(executor_main.STREAM_NAMES)} "
                 f"python={sys.version.split()[0]} patch_hook=executor_core.build_crash_image")
    now = freeze_lines_now()
    frozen = frozen_lines_of_segment_two(arguments.frozen_verdicts)
    seed_now = now[-1].split("sha256_prefix16=")[1]
    lines.result(f"name=freeze_check files={len(now) - 1} identical_to_segment_two={str(now == frozen).lower()} "
                 f"segment_two_lines={len(frozen)} seed_now={seed_now}")
    for stream_name in executor_main.STREAM_NAMES:
        exported_seed = seed_of_the_export(arguments.dumps, stream_name)
        lines.result(f"name=seed_check stream={stream_name} export_seed={exported_seed} "
                     f"matches_the_frozen_executor={str(exported_seed == seed_now).lower()}")
    comparisons, contexts, judgements = judge_and_compare(arguments.dumps, lambda bad: lines.result(bad_judgement_line(bad)))
    for stream_name, context in contexts.items():
        lines.result(f"name=dump_check stream={stream_name} count_mismatches={len(context.dump.count_mismatches)} "
                     f"checksum_mismatches={len(context.dump.checksum_mismatches)} versions={len(context.version_table.versions)}")
    for comparison in comparisons:
        lines.result(comparison_line(comparison))
    totals = count_lines(comparisons, lines.result)
    patched_mismatches = sum(bad.judgement is not None and not bad.judgement.digest_matches for bad in judgements)
    base_mismatches = sum(not bad.base_digest_matches for bad in judgements)
    lines.result(f"name=verdict bad_rows={len(judgements)} patched_digest_mismatches={patched_mismatches} "
                 f"base_digest_mismatches={base_mismatches} judgement_failures={totals['judgement_failures']} "
                 f"freeze_identical_to_segment_two={str(now == frozen).lower()} "
                 f"q3a_positive_judged_in={totals['q3a_positive_judged_in']} q3b_blind_judged_in={totals['q3b_blind_judged_in']} "
                 f"q3c_strict_gap_without_provisional_verdict={totals['q3c_strict_gap_without_provisional_verdict']} "
                 f"q3e_negative_judged_not_in={totals['q3e_negative_judged_not_in']} "
                 f"q3f_impl_both_pass_on_positive_and_blind={totals['q3f_impl_both_pass_on_positive_and_blind']}")
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
