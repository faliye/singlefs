"""E170 第 2 次重跑第二段 Q5 那一半：Q5-e 第一项「两块盘各至少一条记录自证过的态数」（`research/prompts/e170-r2-prereg.md` 12.7）。

判定行里没有逐盘的记录数，这里照 v2-487 自己的步 2–8（`executor_core.recover`）把每一态重建一遍，读 `ring_scan.valid_records_by_device`：
池里每块盘都至少有一条按 487 格式自证过的记录，记一态。坏镜像照 `comparator_bad_image_judge.py` 的取法建完再叠补丁（补丁不算写）。
这只是路径计数（阴性结果与「没读到记录」分开），不判任何一行；用的就是被测那一条臂的读法，所以它只说明那条臂读到了，不说明读对了
（读对由第一段乙的 B11–B13 钉）。
跑法：python3 run_q5_record_path_counts.py --executor-directory <v2-487 目录> --states <名>:<导出目录>:<批,批> [...]
      --images <名>:<含 bad-*.tsv 的导出目录>[:<已知坏镜像目录>] [...] --output <新文件>
文件名以 run_ 起头，不进冻结。
"""

from __future__ import annotations

import argparse
import os
import sys

sys.dont_write_bytecode = True

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
STANDARD_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")


def patches_of(columns: list[str], blob: bytes) -> list[tuple[int, int, bytes]]:
    patches = []
    for entry in columns[4].split(","):
        device, offset, start, length = (int(token) for token in entry.split(":"))
        assert start + length <= len(blob), f"行 {columns[0]} 的补丁越过末尾"
        patches.append((device, offset, blob[start:start + length]))
    return patches


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑 Q5-e 第一项")
    parser.add_argument("--executor-directory", required=True)
    parser.add_argument("--states", action="append", default=[])
    parser.add_argument("--images", action="append", default=[])
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    sys.path.insert(0, os.path.abspath(arguments.executor_directory))
    import crash_dump
    import executor_core
    import replay_prefix
    import spec_gaps
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    def devices_with_a_record(context, landing_vector: str, patches) -> tuple[int, int]:
        image = executor_core.build_crash_image(context.dump, context.prepared_writes, landing_vector)
        for device, offset, payload in patches:
            image.write_bytes(device, offset, payload)
        recovery = executor_core.recover(image, context.dump.device_identities, context.dump.sector_bytes,
                                         replay_prefix.UnitChecksumCache(), spec_gaps.GapsHitInOneState())
        devices = image.device_identities()
        if recovery.ring_scan is None:
            return 0, len(devices)
        return sum(1 for device in devices if recovery.ring_scan.valid_records_by_device.get(device)), len(devices)

    def tally(label: str, stream: str, group: str, results: list[tuple[int, int]]) -> None:
        every = sum(1 for with_record, total in results if with_record == total and total > 0)
        none = sum(1 for with_record, _ in results if with_record == 0)
        emit(f"name=q5e_records_on_every_device input={label} stream={stream} group={group} states={len(results)} "
             f"every_device_has_a_self_verified_record={every} some_device_without={len(results) - every - none} no_device_with_a_record={none}")

    for specification in arguments.states:
        label, dumps, batches = specification.split(":")
        for stream in STREAM_NAMES:
            context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(os.path.join(dumps, stream)))
            for batch in batches.split(","):
                states = [state for state in context.dump.states if state.batch_number == int(batch)]
                tally(label, stream, f"batch_{batch}", [devices_with_a_record(context, state.landing_vector, ()) for state in states])
    for specification in arguments.images:
        label, dumps, *known = specification.split(":")
        contexts = {}
        for stream in STREAM_NAMES:
            directory = os.path.join(dumps, stream)
            context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(directory))
            contexts[stream] = context
            with open(os.path.join(directory, "bad-patches.bin"), "rb") as handle:
                blob = handle.read()
            rows = crash_dump.read_tab_separated_rows(os.path.join(directory, "bad-states.tsv"))
            tally(label, stream, "e170_rows", [devices_with_a_record(context, columns[1], patches_of(columns, blob)) for columns in rows])
            first_row_by_label: dict[str, list[str]] = {}
            states_by_row = {columns[0]: columns for columns in rows}
            for columns in crash_dump.read_tab_separated_rows(os.path.join(directory, "bad-expected.tsv")):
                if columns[3] in STANDARD_BASE_LABELS and columns[3] not in first_row_by_label:
                    first_row_by_label[columns[3]] = states_by_row[columns[0]]
            tally(label, stream, "base", [devices_with_a_record(context, columns[1], ()) for columns in first_row_by_label.values()])
        if known:
            with open(os.path.join(known[0], "known-bad-patches.bin"), "rb") as handle:
                blob = handle.read()
            rows = crash_dump.read_tab_separated_rows(os.path.join(known[0], "known-bad-states.tsv"))
            tally(label, STREAM_NAMES[0], "known_bad", [devices_with_a_record(contexts[STREAM_NAMES[0]], columns[1], patches_of(columns, blob))
                                                        for columns in rows])
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
