#!/usr/bin/env python3
# admission: always 每一趟照此刻的导出目录与这份判法现算每一份输入的键、现查判定库（scripts/experiment-apparatus/long_run_item_verdicts.py）；键相同、库里判绿的复用
# run-condition: command python3
"""E170 第 2 次重跑第二段 Q5 那一半：Q5-e 第一项「两块盘各至少一条记录自证过的态数」（`research/prompts/e170-r2-prereg.md` 12.7）。

判定行里没有逐盘的记录数，这里照 v2-487 自己的步 2–8（`executor_core.recover`）把每一态重建一遍，读 `ring_scan.valid_records_by_device`：
池里每块盘都至少有一条按 487 格式自证过的记录，记一态。坏镜像照 `comparator_bad_image_judge.py` 的取法建完再叠补丁（补丁不算写）。
这只是路径计数（阴性结果与「没读到记录」分开），不判任何一行；用的就是被测那一条臂的读法，所以它只说明那条臂读到了，不说明读对了
（读对由第一段乙的 B11–B13 钉）。
跑法：python3 run_q5_record_path_counts.py --executor-directory <v2-487 目录> --states <名>:<导出目录>:<批,批> [...]
      --images <名>:<含 bad-*.tsv 的导出目录>[:<已知坏镜像目录>] [...] --output <新文件>
      python3 run_q5_record_path_counts.py --selftest
文件名以 run_ 起头，不进冻结。

按份判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每一个 --states / --images 一条判定，经 scripts/experiment-apparatus/long_run_item_verdicts.py 的
  judge_items 存进 prune-server 的 apparatus-items 族、种类 run_q5_record_path_counts。键 = 判法 spec_lines 的单元闭包（连同 EXECUTOR_DIRECTORIES 里各版执行器的模块；
  admission.py 的 compute_unit_key 算）、解释器版本、这一份的种类、名、批、有没有已知坏镜像、选的是哪一版执行器（目录名），与它读的导出目录里每个文件的内容（@files；路径不进键）；
  产物逐字节可比（compare=exact）。键相同、库里判绿的那一份直接取库里的行，不重建。产物照原样：各份的行按命令行的次序接起来，末行 name=done emitted=<行数>。
  判定库那几行打到标准错误；成功那句（标准错误末尾的 LONG_RUN_ITEMS 行）报「现判 N、复用 M、拒算 R」。退出码：0 成；1 复用哨兵红或键在跑的中途变了；2 用法错、产物已在；75 prune-server 不可用。
自证（--selftest）：经 long_run_item_verdicts.py 的复用格跑那六格（reuse-three-questions、key-independent-of-directory、reuse-sentinel 与哨兵的三种红），
  一趟两份 --states，假驱动是 LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE（判法换成假的）；只在临时 prune-server 上跑（客户端 selftest_service_problem 判，不在退 2），起法：
    python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_q5_record_path_counts.py --selftest
  弄坏开关：复用那一侧的分支都在 long_run_item_verdicts.py 里，开关是它文件头的 LONG_RUN_ITEMS_BREAK（打开后这里的自证红在它点名的同名格）；这一份自己不判红（只计数）。
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
STANDARD_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")
# E170 各版执行器的目录（--executor-directory 只认这几份之一）：显式列出来，函数单元库照这张表把几版的模块都收进闭包
EXPERIMENTS_DIRECTORY = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXECUTOR_DIRECTORIES = (os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates_journal_header_487"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor_journal_header_487"))
SCRIPT_KEY = "scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_q5_record_path_counts.py"
LIBRARY_KIND = "run_q5_record_path_counts"


def registered_executor_directory(executor_directory: str) -> str:
    """--executor-directory 给的那一份 → EXECUTOR_DIRECTORIES 里同一个目录；不是其中之一就退出、说清给哪几份。"""
    for directory in EXECUTOR_DIRECTORIES:
        if os.path.realpath(directory) == os.path.realpath(executor_directory):
            return directory
    raise SystemExit(f"  ✗ --executor-directory {executor_directory} 不是 E170 登记的执行器目录之一\n"
                     "  → 怎么办：给 EXECUTOR_DIRECTORIES 里的一份（scripts/experiment-apparatus/ 底下那五个 e170_* 执行器目录）")


def patches_of(columns: list[str], blob: bytes) -> list[tuple[int, int, bytes]]:
    patches = []
    for entry in columns[4].split(","):
        device, offset, start, length = (int(token) for token in entry.split(":"))
        assert start + length <= len(blob), f"行 {columns[0]} 的补丁越过末尾"
        patches.append((device, offset, blob[start:start + length]))
    return patches


def spec_lines(chosen: str, kind: str, specification: str) -> list[str]:
    """一份 --states（kind 是 states）或 --images（images）的计数行；chosen 是 EXECUTOR_DIRECTORIES 里的一项。"""
    for executor_candidate in EXECUTOR_DIRECTORIES:  # 插的是表里那一项本身：插进 sys.path 的目录就是表里写着的那几个之一
        if executor_candidate == chosen:
            sys.path.insert(0, executor_candidate)
    import crash_dump
    import executor_core
    import replay_prefix
    import spec_gaps
    lines: list[str] = []

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
        lines.append(f"name=q5e_records_on_every_device input={label} stream={stream} group={group} states={len(results)} "
                     f"every_device_has_a_self_verified_record={every} some_device_without={len(results) - every - none} no_device_with_a_record={none}")

    if kind == "states":
        label, dumps, batches = specification.split(":")
        for stream in STREAM_NAMES:
            context = executor_core.prepare_stream_context(crash_dump.load_stream_dump(os.path.join(dumps, stream)))
            for batch in batches.split(","):
                states = [state for state in context.dump.states if state.batch_number == int(batch)]
                tally(label, stream, f"batch_{batch}", [devices_with_a_record(context, state.landing_vector, ()) for state in states])
        return lines
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
    return lines


# ── 按份判定（文件头「按份判定」）──

def repository_root() -> str:
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))))


def item_verdicts_module():
    """scripts/experiment-apparatus/long_run_item_verdicts.py；这份脚本不在仓里（上一层没有它）时交 None。"""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), ".."))
    try:
        import long_run_item_verdicts
    except ModuleNotFoundError as error:
        if error.name != "long_run_item_verdicts":
            raise
        return None
    return long_run_item_verdicts


def item_request_lines(name: str, chosen: str, kind: str, specification: str) -> list[str]:
    """一份 --states / --images 的判定请求（long_run_item_verdicts.py 文件头「请求文件」）：路径只经 @files 进内容。"""
    if kind == "states":
        label, dumps, batches = specification.split(":")
        parameters, directories = [f"line:batches={batches}"], [os.path.join(dumps, stream) for stream in STREAM_NAMES]
    else:
        label, dumps, *known = specification.split(":")
        parameters = [f"line:known-bad={'yes' if known else 'no'}"]
        directories = [os.path.join(dumps, stream) for stream in STREAM_NAMES] + known[:1]
    request = "\t".join([name, f"{SCRIPT_KEY}::spec_lines", f"line:kind={kind}", f"line:label={label}", *parameters,
                         f"line:executor={os.path.basename(chosen)}", "line:compare=exact"])
    return [request] + [f"@files\t{name}\t{os.path.abspath(directory)}" for directory in directories]


def main(argument_list: list[str]) -> int:
    if argument_list == ["--selftest"]:
        return selftest()
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑 Q5-e 第一项")
    parser.add_argument("--executor-directory", required=True)
    parser.add_argument("--states", action="append", default=[])
    parser.add_argument("--images", action="append", default=[])
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    chosen = registered_executor_directory(arguments.executor_directory)
    specifications = [("states", text) for text in arguments.states] + [("images", text) for text in arguments.images]
    names = [f"{kind}-{re.sub(r'[^A-Za-z0-9._-]', '-', text.split(':', 1)[0])}" for kind, text in specifications]
    if len(set(names)) != len(names):
        print("  ✗ 几份 --states / --images 的名重了（一份一条判定，名是项名）", file=sys.stderr)
        print("  → 怎么办：每份给一个不同的名", file=sys.stderr)
        return 2
    output = open(arguments.output, "x", encoding="utf-8")
    by_name = dict(zip(names, specifications))
    verdicts = item_verdicts_module()
    finish_code = 0
    if verdicts is None:
        print(f"  · 拒算：{os.path.realpath(__file__)} 不在仓里（旁边没有 ../long_run_item_verdicts.py），照判、不复用、不入库", file=sys.stderr)
        results = {name: (0, "".join(line + "\n" for line in spec_lines(chosen, *by_name[name])).encode("utf-8")) for name in names}
    else:
        request_lines = [line for name in names for line in item_request_lines(name, chosen, *by_name[name])]

        def judge_item(name):
            return 0, "".join(line + "\n" for line in spec_lines(chosen, *by_name[name])).encode("utf-8"), True

        finish_code, results = verdicts.judge_items(SCRIPT_KEY, LIBRARY_KIND, repository_root(), "\n".join(request_lines) + "\n", judge_item,
                                                    printer=lambda text: print(text, file=sys.stderr))
    emitted = 0
    for name in names:
        text = results[name][1].decode("utf-8")
        emitted += text.count("\n")
        output.write(text)
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return finish_code


def selftest() -> int:
    """文件头「自证」：样板树里是这份脚本的拷贝，long_run_item_verdicts.py 与 .claude 是指回真仓的符号链接；导出目录是空的假目录（判法换成假的，不读它们）。"""
    verdicts = item_verdicts_module()
    if verdicts is None:
        print(f"  ✗ run_q5_record_path_counts 自证没跑：{os.path.realpath(__file__)} 旁边找不到 ../long_run_item_verdicts.py")
        print("     → 怎么办：在仓里的那一份上起自证（起法见文件头「自证」）")
        return 2
    problem = verdicts.prune_server_client.selftest_service_problem()
    if problem:
        print(f"  ✗ run_q5_record_path_counts 自证没跑：{problem}")
        print(f"     → 怎么办：{verdicts.prune_server_client.SELFTEST_SERVICE_REMEDY}")
        return 2
    work = tempfile.mkdtemp(prefix="run-q5-record-path-counts-selftest-")
    try:
        template = os.path.join(work, "template")
        os.makedirs(os.path.join(template, os.path.dirname(SCRIPT_KEY)))
        shutil.copyfile(os.path.realpath(__file__), os.path.join(template, SCRIPT_KEY))
        os.symlink(os.path.join(repository_root(), verdicts.SCRIPT_KEY), os.path.join(template, verdicts.SCRIPT_KEY))
        os.symlink(os.path.join(repository_root(), ".claude"), os.path.join(template, ".claude"))
        for label in ("alpha", "beta"):
            for stream in STREAM_NAMES:
                os.makedirs(os.path.join(work, "dumps", label, stream))
                with open(os.path.join(work, "dumps", label, stream, "manifest.tsv"), "w", encoding="utf-8") as handle:
                    handle.write(f"自证的假导出 {label} {stream}\n")
        executor = os.path.join("{tree}", os.path.dirname(SCRIPT_KEY))
        # 产物每一趟一个新名字（--output 已在就拒）：经 bash 拿进程号拼
        command = ["bash", "-c", f'exec python3 "$1" --executor-directory "$2" --states "alpha:$3/alpha:1" --states "beta:$3/beta:1" --output "$4/q5-$$.out"',
                   "run-q5-selftest", "{tree}/" + SCRIPT_KEY, executor, os.path.join(work, "dumps"), work]
        red = verdicts.reuse_cells(SCRIPT_KEY, template, ["states-alpha", "states-beta"], os.path.join(work, "reuse"), 0, command, fake_judge=True)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    print(f"run_q5_record_path_counts.py 自证：6 格，红 {red} 格")
    return 1 if red else 0


if __name__ == "__main__":
    PREFLIGHT_DIRECTORY = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", "..", ".claude", "scripts")
    if os.path.isdir(PREFLIGHT_DIRECTORY):   # 拷到仓外的副本旁边没有 .claude：准入判不了，照「拒算」那一路原样跑
        sys.path.insert(0, PREFLIGHT_DIRECTORY)
        from project_preflight import preflight
        preflight(__file__)
    sys.exit(main(sys.argv[1:]))
