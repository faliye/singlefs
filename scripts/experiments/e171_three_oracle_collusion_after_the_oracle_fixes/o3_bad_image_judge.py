"""E171 Q2 的 O3 一侧：冻结的执行器判坏镜像（登记 5.4、5.5）。

「判的那一半」拷自 `scripts/experiments/e170_second_run_o3_spec_executor/comparator_q3_bad_states.py`（登记 5.3「执行器不改」：拷进本实验目录、
改名，不改原文件；拷来的那份记 sha256，见产物里的 `name=copy_of` 行）：每一张镜像按底态落地向量建镜像、叠上补丁、择系统配置、择根、扫环、取前缀、算 A(c)、判在 / 不在，
全走冻结的 `executor_core.judge_state` 原样；唯一的改动是判的期间把 `executor_core` 模块里 `build_crash_image` 这个名字换成「原样建完、再叠补丁」。
A(c) 因此照底态落地向量算（补丁不算写）。拷的只有判的那一半：不打开 `bad-expected.tsv`（判完两族补丁镜像之后，才读它的族名、行名与底态名来给没打补丁的底态起名），
也不读实现判定文件。

三种输入：
- `e170`：E170 导出装置 `bad-states` 造的 P、N、盲测各行（`bad-states.tsv` 与 `bad-patches.bin`，每条流一份）；
- `known_bad`：装置 `known-bad-images` 写的 41 条已知坏镜像（`known-bad-states.tsv` 与 `known-bad-patches.bin`，格式同上，只在流一全持久态上）；
- `base`：不打补丁的底态（Q2-阴性，每条流 F、Mk、Mk⁺、Mk⁻ 各一张）。

跑法（经 run-with-memory-cap.sh 与 capped.sh）：
  python3 o3_bad_image_judge.py judge --e170-dumps <含 bad-*.tsv 的导出目录> --known-bad-dir <装置 known-bad-images 的输出> --output <新文件>
  python3 o3_bad_image_judge.py control --dumps <一趟的导出目录> --e170-dumps <含 bad-*.tsv 的导出目录> --row-name P1-F --output <新文件>
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
from dataclasses import dataclass

import frozen_o3
from result_lines import ResultLines, field

sys.dont_write_bytecode = True
BAD_STATES_FILE = "bad-states.tsv"
BAD_PATCHES_FILE = "bad-patches.bin"
BAD_EXPECTED_FILE = "bad-expected.tsv"
KNOWN_BAD_STATES_FILE = "known-bad-states.tsv"
KNOWN_BAD_PATCHES_FILE = "known-bad-patches.bin"
STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
SOURCE_E170, SOURCE_KNOWN_BAD, SOURCE_BASE = "e170", "known_bad", "base"
COPIED_FROM = "comparator_q3_bad_states.py"
EXPECTED_COLUMN_OF_THE_BASE_LABEL = 3
STANDARD_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")  # 登记 5.6：两条流的四种底态不打补丁（8 张）；盲测族自己的 Mk@k 底态不进阴性对照


def import_frozen_modules(repo_root: str):
    """冻结目录放进 `sys.path` 第一位再导入（目录里的名字是执行器自己的 `executor_core`、`crash_dump` 等）。交回 (executor_core, crash_dump, replay_prefix)。"""
    directory = os.path.join(repo_root, frozen_o3.FROZEN_DIRECTORY_RELATIVE)
    sys.path.insert(0, directory)
    import crash_dump
    import executor_core
    import replay_prefix
    return executor_core, crash_dump, replay_prefix


@dataclass(frozen=True)
class ImageItem:
    """一张要判的镜像：行号、底态落地向量、底态摘要、补丁之后的摘要、补丁 (盘身份, 盘内偏移, 字节)。没打补丁的底态补丁为空、两个摘要相同。"""

    row_id: int
    landing_vector: str
    base_digest: str
    patched_digest: str
    patches: tuple[tuple[int, int, bytes], ...]


@dataclass(frozen=True)
class ImageJudgement:
    stream: str
    row_id: int
    base_digest_matches: bool
    judgement: object | None  # executor_core.StateJudgement 或判定抛出异常时的 None
    landed_projection: object | None
    failure: str | None


def read_image_items(stream_directory: str, states_file: str, patches_file: str, read_tab_separated_rows) -> list[ImageItem]:
    """只读行号、底态落地向量、两个摘要、补丁；不带族名与期望（拷自原文件的 `read_bad_states`，文件名参数化）。"""
    with open(os.path.join(stream_directory, patches_file), "rb") as handle:
        blob = handle.read()
    items = []
    for columns in read_tab_separated_rows(os.path.join(stream_directory, states_file)):
        row_id, landing_vector, base_digest, patched_digest, patch_list = columns[:5]
        patches = []
        for entry in patch_list.split(","):
            device, offset, start, length = (int(token) for token in entry.split(":"))
            assert start + length <= len(blob), f"行 {row_id} 的补丁越过 {patches_file} 末尾"
            patches.append((device, offset, blob[start:start + length]))
        items.append(ImageItem(int(row_id), landing_vector, base_digest.lower(), patched_digest.lower(), tuple(patches)))
    return items


def judge_images(modules, context, items: list[ImageItem]) -> list[ImageJudgement]:
    """拷自原文件的 `judge_bad_states`：`build_crash_image` 换成「建完再叠补丁」，`recover` 只包一层记下它交回的落点投影；判完两个名字都换回去。"""
    executor_core, crash_dump, replay_prefix = modules
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
        for item in items:
            current["patches"] = item.patches
            current["base_digest"] = None
            current["landed"] = None
            state = crash_dump.SampledState(item.row_id, -1, -1, 0, item.landing_vector, item.patched_digest)
            try:
                judgement, failure = executor_core.judge_state(context, state, replay_prefix.UnitChecksumCache()), None
            except Exception as error:  # 判定行缺失：照打出错，不吞（执行器一侧同样）
                judgement, failure = None, f"{type(error).__name__}:{str(error)[:200]}"
            results.append(ImageJudgement(context.dump.stream_name, item.row_id, current["base_digest"] == item.base_digest,
                                          judgement, current["landed"], failure))
    finally:
        executor_core.build_crash_image = original_builder
        executor_core.recover = original_recover
    return results


def judgement_line(modules, source: str, label: str, judged: ImageJudgement) -> str:
    if judged.judgement is None:
        return (f"name=image_judgement source={source} stream={judged.stream} row={judged.row_id} label={field(label)} strict=error "
                f"provisional=error error={field(judged.failure)}")
    judgement = judged.judgement
    summary = judgement.recovery_summary
    return (f"name=image_judgement source={source} stream={judged.stream} row={judged.row_id} label={field(label)} "
            f"patched_digest={'match' if judgement.digest_matches else 'mismatch'} base_digest={'match' if judged.base_digest_matches else 'mismatch'} "
            f"strict={judgement.strict_verdict} provisional={judgement.provisional_verdict} gaps={','.join(judgement.gaps_hit) or '-'} "
            f"landed={judgement.landed_text} refusal={field(summary['refusal'])} chosen_root={summary['chosen_root']} applied={summary['applied']}")


def base_labels(stream_directory: str, read_tab_separated_rows) -> dict[str, tuple[int, str]]:
    """底态名 → (某一行的号, 底态落地向量)：`bad-expected.tsv` 的第 1 列（行号）与第 4 列（底态名）；同一个底态名取行号最小的那一行的落地向量。
    判完补丁镜像之后才读这个文件，只取底态名，不取期望判定。"""
    vectors = {columns[0]: columns[1] for columns in read_tab_separated_rows(os.path.join(stream_directory, BAD_STATES_FILE))}
    labels: dict[str, tuple[int, str]] = {}
    for columns in read_tab_separated_rows(os.path.join(stream_directory, BAD_EXPECTED_FILE)):
        label = columns[EXPECTED_COLUMN_OF_THE_BASE_LABEL]
        row_id = int(columns[0])
        if label not in labels or row_id < labels[label][0]:
            labels[label] = (row_id, vectors[columns[0]])
    return labels


def run_judge(arguments: argparse.Namespace) -> int:
    modules = import_frozen_modules(arguments.repo_root)
    executor_core, crash_dump, _ = modules
    lines = ResultLines(arguments.output)
    source_path = os.path.join(arguments.repo_root, "scripts/experiments/e170_second_run_o3_spec_executor", COPIED_FROM)
    lines.result(f"name=config mode=judge copy_of={COPIED_FROM} copied_from_sha256={frozen_o3.sha256_of_file(source_path)} "
                 f"this_file_sha256={frozen_o3.sha256_of_file(os.path.abspath(__file__))} e170_dumps={field(os.path.basename(arguments.e170_dumps.rstrip('/')))} "
                 f"known_bad={field(os.path.basename(arguments.known_bad_dir.rstrip('/')))}")
    differences = frozen_o3.freeze_differences(arguments.repo_root)
    lines.result(f"name=freeze_check before=true differences={len(differences)} detail={field(','.join(differences) or 'none')}")
    contexts = {}
    for stream in STREAM_NAMES:
        context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(os.path.join(arguments.e170_dumps, stream)))
        contexts[stream] = context
        lines.result(f"name=dump_check stream={stream} count_mismatches={len(context.dump.count_mismatches)} "
                     f"checksum_mismatches={len(context.dump.checksum_mismatches)} versions={len(context.version_table.versions)}")
    judged_counts: dict[str, int] = {}
    for stream in STREAM_NAMES:
        items = read_image_items(os.path.join(arguments.e170_dumps, stream), BAD_STATES_FILE, BAD_PATCHES_FILE, crash_dump.read_tab_separated_rows)
        for judged in judge_images(modules, contexts[stream], items):
            lines.result(judgement_line(modules, SOURCE_E170, "-", judged))
            judged_counts[SOURCE_E170] = judged_counts.get(SOURCE_E170, 0) + 1
    known_items = read_image_items(arguments.known_bad_dir, KNOWN_BAD_STATES_FILE, KNOWN_BAD_PATCHES_FILE, crash_dump.read_tab_separated_rows)
    for judged in judge_images(modules, contexts[STREAM_NAMES[0]], known_items):
        lines.result(judgement_line(modules, SOURCE_KNOWN_BAD, "-", judged))
        judged_counts[SOURCE_KNOWN_BAD] = judged_counts.get(SOURCE_KNOWN_BAD, 0) + 1
    for stream in STREAM_NAMES:
        stream_directory = os.path.join(arguments.e170_dumps, stream)
        bases = base_labels(stream_directory, crash_dump.read_tab_separated_rows)
        states = {int(columns[0]): columns for columns in crash_dump.read_tab_separated_rows(os.path.join(stream_directory, BAD_STATES_FILE))}
        for label in STANDARD_BASE_LABELS:
            if label not in bases:
                continue
            row_id, vector = bases[label]
            base_digest = states[row_id][2].lower()
            item = ImageItem(row_id, vector, base_digest, base_digest, ())
            for judged in judge_images(modules, contexts[stream], [item]):
                lines.result(judgement_line(modules, SOURCE_BASE, label, judged))
                judged_counts[SOURCE_BASE] = judged_counts.get(SOURCE_BASE, 0) + 1
    after = frozen_o3.freeze_differences(arguments.repo_root)
    lines.result(f"name=freeze_check before=false differences={len(after)} detail={field(','.join(after) or 'none')}")
    lines.result("name=verdict " + " ".join(f"judged_{source}={judged_counts.get(source, 0)}" for source in (SOURCE_E170, SOURCE_KNOWN_BAD, SOURCE_BASE))
                 + f" freeze_differences_before={len(differences)} freeze_differences_after={len(after)}")
    lines.finish()
    return 0


def run_control(arguments: argparse.Namespace) -> int:
    """O3 阳性对照（登记 5.6）：这一趟的流一导出上，全持久态 F 叠 P1 补丁交冻结的执行器，要判「不在」。P1 补丁与它的行号从 E170 `bad-states` 的导出里取
    （行名取 `bad-expected.tsv` 第 2 列；那一列只用来找行号，不读期望判定）。这一趟的 F 摘要与补丁的底态摘要对不上时，对照记 `base_digest=mismatch`、不算红。"""
    modules = import_frozen_modules(arguments.repo_root)
    executor_core, crash_dump, _ = modules
    lines = ResultLines(arguments.output)
    stream = STREAM_NAMES[0]
    context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(os.path.join(arguments.dumps, stream)))
    patch_directory = os.path.join(arguments.e170_dumps, stream)
    names = {columns[1]: int(columns[0]) for columns in crash_dump.read_tab_separated_rows(os.path.join(patch_directory, BAD_EXPECTED_FILE))}
    row_id = names[arguments.row_name]
    items = [item for item in read_image_items(patch_directory, BAD_STATES_FILE, BAD_PATCHES_FILE, crash_dump.read_tab_separated_rows) if item.row_id == row_id]
    judged = judge_images(modules, context, items)[0]
    lines.result(f"name=o3_control dumps={field(os.path.basename(arguments.dumps.rstrip('/')))} patch_row={field(arguments.row_name)} stream={stream}")
    lines.result(judgement_line(modules, "control", arguments.row_name, judged))
    strict = "error" if judged.judgement is None else judged.judgement.strict_verdict
    provisional = "error" if judged.judgement is None else judged.judgement.provisional_verdict
    lines.result(f"name=o3_control_verdict strict={strict} provisional={provisional} base_digest_matches={str(judged.base_digest_matches).lower()} "
                 f"red_strict={str(strict == 'not_in' and judged.base_digest_matches).lower()}")
    lines.finish()
    return 0


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 Q2 的 O3 一侧")
    parser.add_argument("mode", choices=["judge", "control"])
    parser.add_argument("--repo-root", default=os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")))
    parser.add_argument("--e170-dumps", required=True)
    parser.add_argument("--known-bad-dir")
    parser.add_argument("--dumps")
    parser.add_argument("--row-name", default="P1-F")
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    return run_judge(arguments) if arguments.mode == "judge" else run_control(arguments)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
