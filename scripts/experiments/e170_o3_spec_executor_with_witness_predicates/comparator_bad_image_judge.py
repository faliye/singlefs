"""E170 第 2 次重跑第二段：坏状态与坏镜像的判法（`research/prompts/e170-r2-prereg.md` 5.3 第三条），对 v1、v2 各跑一遍。

「判的那一半」拷自 v1 目录的 `comparator_q3_bad_states.py`（sha256 01d0cacb0eacd3c6537b6a9308f1bc1a7d46bb378d232565b8bc675acf222895）的
`judge_bad_states`，三种输入的读法照 E171 那一份 `scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py`
（sha256 ac16c7944155c991fed008e09a2d97831d699d931cd0b5d537388b7a6282b937）：每一张镜像按底态落地向量建镜像、叠补丁、照常判，
全走所选执行器目录里的 `executor_core.judge_state` 原样；唯一的改动是判的期间把 `executor_core.build_crash_image` 换成「原样建完、再叠补丁」，
判完换回去。A(c) 因此照底态向量算（原登记读法 3：补丁不算写）。判定行用那个执行器自己的 `executor_core.judgement_line`
（v2 的那一行带读法 4 加的字段），前面加 `input=`、`source=`、`label=`、`base_digest=`。

判的期间不打开 `bad-expected.tsv`：P、N、盲测各行与已知坏镜像判完之后，才读它的第 4 列（底态名）给没打补丁的底态取落地向量（同 E171 那一份）；
行名留给比对器在判完之后读。

跑法（经 capped.sh 与 run-with-memory-cap.sh）：
  python3 comparator_bad_image_judge.py --executor-directory <v1 或 v2 目录> --input-label I3|I4 --dumps <含 bad-*.tsv 的导出目录>
      [--known-bad-dir <known-bad-images 的输出>] --freeze-reference <带那一版冻结行的产物> --output <新文件>
文件名以 comparator 起头，不进 v2 冻结的 sha256。
"""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import dataclass

sys.dont_write_bytecode = True

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
STANDARD_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")  # 两条流的四种底态不打补丁（登记读法 5 的 I3、I4）
EXPECTED_COLUMN_OF_THE_BASE_LABEL = 3
# E170 各版执行器的目录（--executor-directory 只认这几份之一）：显式列出来，函数单元库照这张表把几版的模块都收进闭包
EXPERIMENTS_DIRECTORY = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXECUTOR_DIRECTORIES = (os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates_journal_header_487"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor_journal_header_487"))


def registered_executor_directory(executor_directory: str) -> str:
    """--executor-directory 给的那一份 → EXECUTOR_DIRECTORIES 里同一个目录；不是其中之一就退出、说清给哪几份。"""
    for directory in EXECUTOR_DIRECTORIES:
        if os.path.realpath(directory) == os.path.realpath(executor_directory):
            return directory
    raise SystemExit(f"  ✗ --executor-directory {executor_directory} 不是 E170 登记的执行器目录之一\n"
                     "  → 怎么办：给 EXECUTOR_DIRECTORIES 里的一份（scripts/experiments/ 底下那五个 e170_* 执行器目录）")


def import_executor(executor_directory: str):
    chosen = registered_executor_directory(executor_directory)
    for directory in EXECUTOR_DIRECTORIES:  # 插的是表里那一项本身（不是 chosen）：插进 sys.path 的目录就是表里写着的那几个之一
        if directory == chosen:
            sys.path.insert(0, directory)
    import crash_dump
    import executor_core
    import executor_main
    import replay_prefix
    return executor_core, crash_dump, executor_main, replay_prefix


@dataclass(frozen=True)
class ImageItem:
    row_id: int
    landing_vector: str
    base_digest: str
    patched_digest: str
    patches: tuple[tuple[int, int, bytes], ...]


def read_image_items(stream_directory: str, states_file: str, patches_file: str, read_tab_separated_rows) -> list[ImageItem]:
    """只读行号、底态落地向量、两个摘要、补丁；不带族名与期望（拷自 `comparator_q3_bad_states.read_bad_states`，文件名参数化）。"""
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


def judge_images(modules, context, items: list[ImageItem]) -> list[tuple[ImageItem, bool, object, str | None]]:
    """拷自 `comparator_q3_bad_states.judge_bad_states`：`build_crash_image` 换成「建完再叠补丁」，判完换回去。交回 (项, 底态摘要对不对, 判定, 出错)。"""
    executor_core, crash_dump, _, replay_prefix = modules
    original_builder = executor_core.build_crash_image
    current: dict = {"patches": (), "base_digest": None}

    def builder_with_patches(dump, prepared_writes, landing_vector):
        image = original_builder(dump, prepared_writes, landing_vector)
        current["base_digest"] = image.digest()
        for device, offset, payload in current["patches"]:
            image.write_bytes(device, offset, payload)
        return image

    results = []
    executor_core.build_crash_image = builder_with_patches
    try:
        for item in items:
            current["patches"] = item.patches
            current["base_digest"] = None
            state = crash_dump.SampledState(item.row_id, -1, -1, 0, item.landing_vector, item.patched_digest)
            try:
                judgement, failure = executor_core.judge_state(context, state, replay_prefix.UnitChecksumCache()), None
            except Exception as error:  # 判定行缺失：照打出错，不吞（登记第十一节 V4）
                judgement, failure = None, f"{type(error).__name__}:{str(error)[:200]}"
            results.append((item, current["base_digest"] == item.base_digest, judgement, failure))
    finally:
        executor_core.build_crash_image = original_builder
    return results


def without_whitespace(value: object) -> str:
    return "".join("_" if character.isspace() else character for character in str(value)) or "-"


def image_line(modules, input_label: str, source: str, label: str, stream: str, result) -> str:
    executor_core = modules[0]
    item, base_digest_matches, judgement, failure = result
    prefix = (f"name=image_judgement input={input_label} source={source} label={without_whitespace(label)} "
              f"base_digest={'match' if base_digest_matches else 'mismatch'}")
    if judgement is None:
        return f"{prefix} stream={stream} state={item.row_id} strict=error provisional=error error={without_whitespace(failure)}"
    return f"{prefix} " + executor_core.judgement_line(stream, judgement, 0).removeprefix("name=state ")


def freeze_lines_now(executor_main) -> list[str]:
    captured: list[str] = []
    executor_main.freeze_lines(captured.append)
    return captured


def frozen_lines_of(reference_path: str) -> list[str]:
    with open(reference_path, encoding="utf-8") as handle:
        return [line.rstrip("\n").removeprefix("E7RESULT ") for line in handle
                if line.removeprefix("E7RESULT ").startswith(("name=freeze ", "name=freeze_seed "))]


def base_items(stream_directory: str, read_tab_separated_rows) -> list[tuple[str, ImageItem]]:
    """判完补丁镜像之后才读 `bad-expected.tsv` 的第 4 列（底态名）：同一个底态名取行号最小那一行的落地向量与底态摘要（同 E171 那一份）。"""
    states = {int(columns[0]): columns for columns in read_tab_separated_rows(os.path.join(stream_directory, "bad-states.tsv"))}
    first_row_by_label: dict[str, int] = {}
    for columns in read_tab_separated_rows(os.path.join(stream_directory, "bad-expected.tsv")):
        label, row_id = columns[EXPECTED_COLUMN_OF_THE_BASE_LABEL], int(columns[0])
        if label not in first_row_by_label or row_id < first_row_by_label[label]:
            first_row_by_label[label] = row_id
    items = []
    for label in STANDARD_BASE_LABELS:
        if label in first_row_by_label:
            columns = states[first_row_by_label[label]]
            items.append((label, ImageItem(first_row_by_label[label], columns[1], columns[2].lower(), columns[2].lower(), ())))
    return items


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第二段：坏状态与坏镜像的判法（v1、v2 各一遍）")
    parser.add_argument("--executor-directory", required=True)
    parser.add_argument("--input-label", required=True)
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--known-bad-dir")
    parser.add_argument("--freeze-reference", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    modules = import_executor(arguments.executor_directory)
    executor_core, crash_dump, executor_main, _ = modules
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    frozen = frozen_lines_of(arguments.freeze_reference)
    before = freeze_lines_now(executor_main)
    emit(f"name=config input={arguments.input_label} executor={os.path.basename(os.path.abspath(arguments.executor_directory))} "
         f"dumps={arguments.dumps} known_bad_dir={arguments.known_bad_dir or '-'} freeze_reference={os.path.basename(arguments.freeze_reference)}")
    emit(f"name=freeze_check when=start identical={str(before == frozen and len(frozen) > 0).lower()} files_now={len(before) - 1} reference_lines={len(frozen)}")
    contexts = {}
    for stream in STREAM_NAMES:
        context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(os.path.join(arguments.dumps, stream)))
        contexts[stream] = context
        emit(f"name=dump_check stream={stream} count_mismatches={len(context.dump.count_mismatches)} "
             f"checksum_mismatches={len(context.dump.checksum_mismatches)} versions={len(context.version_table.versions)}")
        for version in context.version_table.versions:
            emit(f"name=version stream={stream} index={version.version_index} projection={version.projection.short_text()} "
                 f"root_write={version.root_write_stream_index} return_point={version.return_point} call={version.call_label}")
    counts = {"e170": 0, "known_bad": 0, "base": 0}
    for stream in STREAM_NAMES:
        items = read_image_items(os.path.join(arguments.dumps, stream), "bad-states.tsv", "bad-patches.bin", crash_dump.read_tab_separated_rows)
        for result in judge_images(modules, contexts[stream], items):
            emit(image_line(modules, arguments.input_label, "e170", "-", stream, result))
            counts["e170"] += 1
    if arguments.known_bad_dir:
        items = read_image_items(arguments.known_bad_dir, "known-bad-states.tsv", "known-bad-patches.bin", crash_dump.read_tab_separated_rows)
        for result in judge_images(modules, contexts[STREAM_NAMES[0]], items):
            emit(image_line(modules, arguments.input_label, "known_bad", "-", STREAM_NAMES[0], result))
            counts["known_bad"] += 1
    for stream in STREAM_NAMES:
        for label, item in base_items(os.path.join(arguments.dumps, stream), crash_dump.read_tab_separated_rows):
            for result in judge_images(modules, contexts[stream], [item]):
                emit(image_line(modules, arguments.input_label, "base", label, stream, result))
                counts["base"] += 1
    after = freeze_lines_now(executor_main)
    emit(f"name=freeze_check when=end identical={str(after == frozen and len(frozen) > 0).lower()} files_now={len(after) - 1} reference_lines={len(frozen)}")
    emit("name=judged " + " ".join(f"{source}={count}" for source, count in counts.items()))
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
