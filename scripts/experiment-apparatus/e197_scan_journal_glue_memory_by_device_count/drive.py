#!/usr/bin/env python3
# admission: always 每一次调都在此刻的仓与装置二进制上现跑，上一次的产物不替这一次作保
# run-condition: command python3
# task-server-shards: segment
"""E197（扫环胶水内存随盘数）驱动：按跑前登记 research/prompts/e197-preregistration.md 第五、六、十一、十四节与第十二节修订一，
一格一次服务尝试（分片的一片）、不套任何包装起装置二进制；判的峰值是统一任务服务记下的那一次尝试的 memory_peak，
由主 agent 在每件有结局之后读 `task-server/task status <父任务号> --json` 存进 research/results/，判归同目录的 report.py。

用法（在仓根跑；装置二进制只编一次：compile-then-swap 编出来的 <草稿目录>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count，
在共享工作根下，两台照同一个绝对路径读得到）：
  drive.py list-cells <段> --round <r> --group <组>        逐行列这一组的格，不起任何进程
  drive.py checks --binary <二进制> --out <产物.out>         不分片：S1（两段 sha256）、S4（读缓存那一段 sha256）、装置 --checks、--diff-set
  drive.py compare --binary <二进制> --out <产物.out>        不分片：五份胶水臂在生产格上与 scan_journal 逐键逐值比，52 格串行，不量峰值
  drive.py segment <段> --round <r> --group <组> --binary <二进制> --out <产物.out> [--cells <臂:盘数,…>]
        由统一任务服务分片（文件头 task-server-shards 那一行，第一个参数是 segment 才分）：
        计划一步（TASK_SERVER_SHARD_PHASE=plan）把这一组的格（至多 8 格）写进 plan.json 与 cells.json，把 <二进制> 拷进计划目录；
        每片（shard）把计划目录里那份二进制再拷进自己的产物目录、从这份拷贝起这一格（页缓存记在这一片自己的 scope 上），
        结果行另带 task= attempt= shard_host= binary_sha256=；合并一步（merge）按片号次序把各片结果写进 <产物.out>（排他新建，已在就拒）。
        不经服务直接起（环境里没有 TASK_SERVER_SHARD_PHASE）退 2。
        段：k64-full、k64-half、k1-full。组：glue-1、glue-2、glue-3（空跑、四个胶水臂、k64-full 另加 measurement_pc，按次序每组 8 格），
        read-stage-cache（丙那几格：read_stage_cache 每段 4 格，k64-half 另加 read_stage_cache_keeps_zeros_pc 4 格）。
        --round 1–5 是初测轮的重复号；--round 6–10 是 V2 复测轮，组写 rerun-<n>，只跑 --cells 点名的格（list-reruns 打出来的）。
  drive.py list-submissions --binary <二进制> --date <YYYY-MM-DD> [--batch <n>]
        逐行打出要交的 task-server/task submit 整条（不交、不跑）：批 0 是 checks 与 compare，批 1–10 各 6 件 segment
        （段 k64-full、k64-half、k1-full 依次，每段重复号 1–5，每个重复号四组）；不带 --batch 打全部。
  drive.py list-reruns --binary <二进制> --date <YYYY-MM-DD> --artifacts <产物.out…> --status <状态存档.json…>
        照 report.py 的 V2 判法（初测轮 5 次散布 > 2%）打出复测轮（--round 6–10）要交的整条
  drive.py --selftest                                       假的二进制、假的服务环境上的自证（不起装置、不起服务）：各项过了之后接着逐个打开弄坏开关
                                                            起自己的 --selftest，要求每个开关都红在点名的那一项（开关生效的那一趟只跑各项）
退出码：0 做完；1 自证红、有格没跑成、片没跑成或产物文件已在；2 用法错或不经服务起 segment；3 停机条款 S1 / S4（实现不是登记时那一份）。
不写内存上限、线程与挑台参数：上限、哪台、撞上限放大都由服务定；这份脚本也不起 task-server/task（峰值由主 agent 存档，report.py 读存档）。

弄坏开关（只给自证用；环境变量 E197_DRIVE_BREAK=<一个值>，不认识的值直接拒绝）：
  E197_DRIVE_BREAK=shard-result-untagged        片的结果行不带 task= attempt= shard_host= binary_sha256=
      自证项 「✗ 自检 a-shard-runs-its-own-binary-copy-and-tags-the-result-line：」
  E197_DRIVE_BREAK=shard-runs-the-plan-binary   片直接起计划目录里那份二进制，不拷进自己的产物目录
      自证项 「✗ 自检 a-shard-runs-its-own-binary-copy-and-tags-the-result-line：」
  E197_DRIVE_BREAK=failed-cell-passes           装置退非 0 或结果行不是恰好一行，片照样退 0
      自证项 「✗ 自检 a-failing-cell-fails-the-shard：」
  E197_DRIVE_BREAK=merge-not-in-shard-order     合并照 outputs.tsv 的行序写，不按片号排
      自证项 「✗ 自检 merge-writes-the-shards-in-order-and-refuses-an-existing-file：」
  E197_DRIVE_BREAK=merge-overwrites             合并覆盖已在的产物文件
      自证项 「✗ 自检 merge-writes-the-shards-in-order-and-refuses-an-existing-file：」
  E197_DRIVE_BREAK=s1-always-same               S1、S4 的核对恒报 same
      自证项 「✗ 自检 s1-tells-a-moved-function-from-a-changed-one：」
  E197_DRIVE_BREAK=merge-reads-unchecked-inputs 合并那一步不核计划与各片产物表是不是在的文件，直接读（没设或指到目录时抛 Traceback）
      自证项 「✗ 自检 merge-without-the-service-tables-explains-and-exits-one：」
"""
import contextlib
import hashlib
import io
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True   # 不往仓里写 __pycache__
REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPOSITORY_ROOT / "scripts" / "hosts"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import heavy_threads  # noqa: E402
import report  # noqa: E402

BREAK_SWITCHES = ("shard-result-untagged", "shard-runs-the-plan-binary", "failed-cell-passes", "merge-not-in-shard-order",
                  "merge-overwrites", "s1-always-same", "merge-reads-unchecked-inputs")
BREAK_TO_CASE = {
    "shard-result-untagged": "a-shard-runs-its-own-binary-copy-and-tags-the-result-line",
    "shard-runs-the-plan-binary": "a-shard-runs-its-own-binary-copy-and-tags-the-result-line",
    "failed-cell-passes": "a-failing-cell-fails-the-shard",
    "merge-not-in-shard-order": "merge-writes-the-shards-in-order-and-refuses-an-existing-file",
    "merge-overwrites": "merge-writes-the-shards-in-order-and-refuses-an-existing-file",
    "s1-always-same": "s1-tells-a-moved-function-from-a-changed-one",
    "merge-reads-unchecked-inputs": "merge-without-the-service-tables-explains-and-exits-one",
}
BINARY_NAME = "e197_scan_journal_glue_memory_by_device_count"
THIS_SCRIPT = "scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py"
SEGMENTS = {"k64-full": (64, "full"), "k64-half": (64, "half"), "k1-full": (1, "full")}
GLUE_GROUPS = ("glue-1", "glue-2", "glue-3")
READ_STAGE_CACHE_GROUP = "read-stage-cache"
RERUN_GROUP_PREFIX = "rerun-"
MAXIMUM_CELLS_PER_GROUP = 8          # 服务的 MAXIMUM_SHARDS（task-server/task_server_lib.py），一片一格
FIRST_ROUNDS = range(1, report.REPEATS_PER_ROUND + 1)
SECOND_ROUNDS = range(report.REPEATS_PER_ROUND + 1, 2 * report.REPEATS_PER_ROUND + 1)
SEGMENT_TASKS_PER_BATCH = 6          # 一批交 6 件分片任务，前一批大半有了结局再交下一批
COMPARE_GLUE_ARMS = ("baseline_pc", "split", "split_pc")
# 登记第十一节 S1 与修订一 S4：登记时的实现。(文件, 起行, 止行, sha256, 那一行的起头, 那一行在这一段里的行偏移)
IMPLEMENTATION_RANGES = (
    ("crates/singlefs-core/src/recovery.rs", 4431, 4484,
     "0ca338aaafcfff2796656134c9246295e33d58503f439d61588a9cb394697c55", "pub fn scan_journal(", 5),
    ("crates/singlefs-replay-prefix/src/lib.rs", 4029, 4058,
     "60e0648acc3f3a3a730b3c1731781cf0eda6e28068a56b6e0aad8d58dc1d2d41", "pub fn first_copy_of_each_record_key(", 0),
    ("crates/singlefs-core/src/mount.rs", 8446, 8500,
     "58f00a3e70e681cf18cf672e29707b9bee2fe2655bb2ea996a4072f72a5d3f8c", "struct ReadStageCache<", 4),
)


def broken(name):
    """弄坏开关生效了吗；不认识的值直接拒绝（免得拼错的开关被当成没开）。"""
    value = os.environ.get("E197_DRIVE_BREAK", "")
    if value and value not in BREAK_SWITCHES:
        sys.exit(f"不认识的 E197_DRIVE_BREAK 值：{value}")
    return value == name


class Cell:
    def __init__(self, arm, devices, fill, named, repeat, compare=False):
        self.arm, self.devices, self.fill, self.named, self.repeat, self.compare = arm, devices, fill, named, repeat, compare

    def key(self):
        prefix = "e197/compare" if self.compare else "e197"
        return f"{prefix}/{self.arm}/{self.devices}/{self.fill}/{self.named}/{self.repeat}"

    def arguments(self):
        words = ["--arm", self.arm, "--devices", str(self.devices), "--fill", self.fill, "--named", str(self.named), "--repeat", str(self.repeat)]
        return words + ["--compare"] if self.compare else words

    def as_record(self):
        return {"arm": self.arm, "devices": self.devices, "fill": self.fill, "named": self.named, "repeat": self.repeat}

    @staticmethod
    def from_record(record):
        return Cell(record["arm"], int(record["devices"]), record["fill"], int(record["named"]), int(record["repeat"]))


def glue_cells(segment, repeat):
    """原有的格（登记第五节「分段」）：空跑与四个胶水臂各盘数 1–4，k64-full 另加 measurement_pc（1 盘）；空跑排在最前。"""
    named, fill = SEGMENTS[segment]
    cells = [Cell(arm, devices, fill, named, repeat) for arm in ("empty",) + report.GLUE_ARMS for devices in report.DEVICE_COUNTS]
    if segment == "k64-full":
        cells.append(Cell("measurement_pc", 1, fill, named, repeat))
    return cells


def read_stage_cache_cells(segment, repeat):
    """丙那几格（登记修订一第五节）：read_stage_cache 盘数 1–4；阳性对照只在 K = 64 半满上跑。"""
    named, fill = SEGMENTS[segment]
    arms = ("read_stage_cache",) + (("read_stage_cache_keeps_zeros_pc",) if segment == "k64-half" else ())
    return [Cell(arm, devices, fill, named, repeat) for arm in arms for devices in report.DEVICE_COUNTS]


def parse_cell_list(text, segment, repeat):
    """--cells 写的 <臂:盘数,…> → 格；写错交 None。"""
    named, fill = SEGMENTS[segment]
    cells = []
    for item in (text or "").split(","):
        arm, _, devices = item.partition(":")
        if arm not in report.MEASURED_ARMS or not devices.isdigit() or int(devices) not in report.DEVICE_COUNTS:
            return None
        cells.append(Cell(arm, int(devices), fill, named, repeat))
    return cells or None


def cells_of_group(segment, repeat, group, cell_list=None):
    """一件分片任务的格（至多 MAXIMUM_CELLS_PER_GROUP 格）；参数对不上交 None。"""
    if segment not in SEGMENTS:
        return None
    if repeat in FIRST_ROUNDS and cell_list is None:
        if group in GLUE_GROUPS:
            start = GLUE_GROUPS.index(group) * MAXIMUM_CELLS_PER_GROUP
            return glue_cells(segment, repeat)[start:start + MAXIMUM_CELLS_PER_GROUP]
        if group == READ_STAGE_CACHE_GROUP:
            return read_stage_cache_cells(segment, repeat)
        return None
    if repeat in SECOND_ROUNDS and group.startswith(RERUN_GROUP_PREFIX) and cell_list is not None:
        cells = parse_cell_list(cell_list, segment, repeat)
        return cells if cells and len(cells) <= MAXIMUM_CELLS_PER_GROUP else None
    return None


def every_first_round_group():
    """(段, 重复号, 组)：段依次 k64-full、k64-half、k1-full（登记第五节「分段」的次序），每段重复号 1–5，每个重复号四组。"""
    return [(segment, repeat, group) for segment in SEGMENTS for repeat in FIRST_ROUNDS for group in GLUE_GROUPS + (READ_STAGE_CACHE_GROUP,)]


def compare_cells():
    """compare：原三份胶水臂 × 三个几何 × 盘数 1–4（36 格），read_stage_cache 三个几何 × 盘数 1–4（12 格），阳性对照 K = 64 半满 × 盘数 1–4（4 格）。"""
    cells = [Cell(arm, devices, fill, named, 1, compare=True) for arm in COMPARE_GLUE_ARMS for named, fill in SEGMENTS.values()
             for devices in report.DEVICE_COUNTS]
    cells += [Cell("read_stage_cache", devices, fill, named, 1, compare=True) for named, fill in SEGMENTS.values() for devices in report.DEVICE_COUNTS]
    named, fill = SEGMENTS["k64-half"]
    cells += [Cell("read_stage_cache_keeps_zeros_pc", devices, fill, named, 1, compare=True) for devices in report.DEVICE_COUNTS]
    return cells


def result_lines_of(standard_output):
    return [line for line in standard_output.splitlines() if line.startswith("E7RESULT ")]


def subprocess_runner(command):
    """(退出码, 标准输出, 标准错误)。"""
    finished = subprocess.run(command, cwd=REPOSITORY_ROOT, capture_output=True, text=True, check=False)
    return finished.returncode, finished.stdout, finished.stderr


def sha256_of_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def copy_binary(source, directory):
    """把二进制拷进 directory（先写同目录临时文件再改名，重试时盖掉上一次的），交拷贝的路径与它的 sha256。"""
    target = Path(directory, BINARY_NAME)
    temporary = Path(directory, BINARY_NAME + ".partial")
    shutil.copyfile(source, temporary)
    os.chmod(temporary, stat.S_IRWXU | stat.S_IRGRP | stat.S_IXGRP | stat.S_IROTH | stat.S_IXOTH)
    os.replace(temporary, target)
    return target, sha256_of_file(target)


def write_new_file(path, lines):
    """产物排他新建：已在就拒（交 False，不动它）；整块一次写进去并刷盘。"""
    mode = "w" if broken("merge-overwrites") else "x"
    try:
        with open(path, mode, encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
            handle.flush()
            os.fsync(handle.fileno())
    except FileExistsError:
        return False
    return True


def write_json_atomically(path, value):
    temporary = f"{path}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False, sort_keys=True)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def registered_heavy_threads():
    """配置行记的是每台默认重活线程数的登记值（不是本机核数）；取不了就带出路退出，不退回按核数算。"""
    try:
        return heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        sys.exit(f"取不了每台默认重活线程数：{error}（→ 怎么办：{error.howto}）")


# ---- 停机条款 S1、S4：实现还是登记时那一份 ----

def sha256_of_lines(lines, start, end):
    return hashlib.sha256(("\n".join(lines[start - 1:end]) + "\n").encode("utf-8")).hexdigest()


def implementation_line(root, entry):
    """一份文件的核对行：同一段行号上 sha256 相同记 same；挪了行号而正文没变（按那一行的起头找到新位置、取同样长的一段）记 moved_same；否则 changed。"""
    path, start, end, expected, signature, offset = entry
    if broken("s1-always-same"):
        return f"E7RESULT name=e197_s1 file={path} status=same"
    lines = Path(root, path).read_text(encoding="utf-8").split("\n")
    found = sha256_of_lines(lines, start, end)
    status, shown_start = ("same", start) if found == expected else ("changed", start)
    if status == "changed":
        positions = [number for number, text in enumerate(lines, 1) if text.startswith(signature)]
        if len(positions) == 1:
            moved_start = positions[0] - offset
            moved = sha256_of_lines(lines, moved_start, moved_start + (end - start))
            if moved == expected:
                status, shown_start, found = "moved_same", moved_start, moved
    return (f"E7RESULT name=e197_s1 file={path} registered_start={start} registered_end={end} found_start={shown_start} "
            f"sha256={found} expected={expected} status={status}")


def implementation_lines(root):
    return [implementation_line(root, entry) for entry in IMPLEMENTATION_RANGES]


def implementation_is_the_registered_one(lines):
    return all(line.endswith("status=same") or line.endswith("status=moved_same") for line in lines)


def config_line(step, binary_sha256, **fields):
    extra = " ".join(f"{name}={value}" for name, value in fields.items())
    return (f"E7RESULT name=e197_config step={step} binary_sha256={binary_sha256} threads_per_cell=1 "
            f"registered_heavy_threads={registered_heavy_threads()} peak_source=task_server_attempt_memory_peak {extra}").rstrip()


def done_line(step, cells, **fields):
    extra = " ".join(f"{name}={value}" for name, value in fields.items())
    return f"E7RESULT name=e197_segment_done step={step} cells={cells} {extra}".rstrip()


def absolute_output(path):
    return path if os.path.isabs(path) else os.path.join(REPOSITORY_ROOT, path)


def run_checks(binary, artifact_path, root=REPOSITORY_ROOT, runner=subprocess_runner):
    """S1、S4 先核，实现变了就停（退 3，不跑装置）；没变再直接起装置的 --checks 与 --diff-set（不套包装）。产物排他新建。"""
    output = absolute_output(artifact_path)
    if os.path.exists(output):
        print(f"产物 {artifact_path} 已在，不覆盖（→ 怎么办：换一个新文件名，按跑的那天的日期另存）", file=sys.stderr)
        return 1
    digest = sha256_of_file(binary)
    implementation = implementation_lines(root)
    if not implementation_is_the_registered_one(implementation):
        write_new_file(output, [config_line("checks", digest)] + implementation)
        print("停机条款 S1 / S4：实现不是登记时那一份，停；抄本（baseline_pc、split、读缓存）要按新实现重写", file=sys.stderr)
        return 3
    lines = [config_line("checks", digest)] + implementation
    failed = 0
    for mode in ("--checks", "--diff-set"):
        exit_code, standard_output, _ = runner([str(binary), mode])
        lines += result_lines_of(standard_output)
        lines.append(f"E7RESULT name=e197_run mode={mode} exit={exit_code}")
        failed += exit_code != 0
    lines.append(done_line("checks", 2))
    if not write_new_file(output, lines):
        print(f"产物 {artifact_path} 在跑的时候被别人建了，不覆盖", file=sys.stderr)
        return 1
    return 1 if failed else 0


def run_compare(binary, artifact_path, runner=subprocess_runner):
    """compare：52 格串行，每格一个装置进程（带 --compare），不量峰值；没跑成的格写 name=e197_cell_lost，不当相等。产物排他新建。"""
    output = absolute_output(artifact_path)
    if os.path.exists(output):
        print(f"产物 {artifact_path} 已在，不覆盖（→ 怎么办：换一个新文件名）", file=sys.stderr)
        return 1
    cells = compare_cells()
    lines, lost = [config_line("compare", sha256_of_file(binary), cells=len(cells))], 0
    for cell in cells:
        exit_code, standard_output, _ = runner([str(binary)] + cell.arguments())
        own = [line for line in result_lines_of(standard_output) if line.startswith("E7RESULT name=e197_compare ")]
        if exit_code == 0 and len(own) == 1:
            lines += own
        else:
            lost += 1
            lines.append(f"E7RESULT name=e197_cell_lost key={cell.key()} exit={exit_code} result_lines={len(own)}")
    lines.append(done_line("compare", len(cells), lost=lost))
    if not write_new_file(output, lines):
        print(f"产物 {artifact_path} 在跑的时候被别人建了，不覆盖", file=sys.stderr)
        return 1
    print(f"compare：{len(cells)} 格，没跑成 {lost} 格")
    return 1 if lost else 0


# ---- segment：服务分片的三步（task-server/task_server_lib.py 文件头「分片」）----

PHASE_VARIABLE = "TASK_SERVER_SHARD_PHASE"
SHARD_VARIABLE = "TASK_SERVER_SHARD"
PLAN_VARIABLE = "TASK_SERVER_SHARD_PLAN_DIRECTORY"
OUTPUT_VARIABLE = "TASK_SERVER_SHARD_OUTPUT_DIRECTORY"
OUTPUTS_VARIABLE = "TASK_SERVER_SHARD_OUTPUTS"
HOST_VARIABLE = "TASK_SERVER_SHARD_HOST"
CELLS_FILE = "cells.json"
RESULT_FILE = "result.out"


def plan_phase(cells, binary, plan_directory, segment, repeat, group):
    """计划：二进制拷进计划目录（共享工作根下，两台同一个路径），格写进 cells.json，一格一片写进 plan.json。"""
    _, digest = copy_binary(binary, plan_directory)
    write_json_atomically(Path(plan_directory, CELLS_FILE), {"cells": [cell.as_record() for cell in cells], "binary_sha256": digest,
                                                              "segment": segment, "round": repeat, "group": group})
    write_json_atomically(Path(plan_directory, "plan.json"), {"shards": [{"label": cell.key()} for cell in cells],
                                                              "note": f"E197 {segment} 第 {repeat} 轮 {group}：一片一格"})
    print(f"计划：{segment} 第 {repeat} 轮 {group}，{len(cells)} 片，二进制 sha256 {digest}")
    return 0


def shard_phase(environment, runner=subprocess_runner):
    """一片：计划目录里那份二进制拷进这一片的产物目录、从拷贝起这一格；结果行带上服务给的任务号、第几次、哪台与二进制的 sha256，写进 result.out。
    装置退非 0、结果行不是恰好一行、读不到服务给的环境变量，这一片退 1（读不到不当 0）。"""
    position = environment.get(SHARD_VARIABLE, "")
    index, _, count = position.partition("/")
    plan_directory, output_directory = environment.get(PLAN_VARIABLE, ""), environment.get(OUTPUT_VARIABLE, "")
    task, attempt, host = environment.get("TASK_SERVER_TASK", ""), environment.get("TASK_SERVER_ATTEMPT", ""), environment.get(HOST_VARIABLE, "")
    if not (index.isdigit() and count.isdigit() and plan_directory and output_directory and task and attempt and host):
        print(f"  ✗ 片读不全服务给的环境（{SHARD_VARIABLE}={position!r}、任务号 {task!r}、第几次 {attempt!r}、台 {host!r}）", file=sys.stderr)  # gate-lint:detail
        return 1
    plan = json.loads(Path(plan_directory, CELLS_FILE).read_text(encoding="utf-8"))
    if int(count) != len(plan["cells"]) or int(index) >= len(plan["cells"]):
        print(f"  ✗ 片号 {position} 与计划里的 {len(plan['cells'])} 格对不上", file=sys.stderr)  # gate-lint:detail
        return 1
    cell = Cell.from_record(plan["cells"][int(index)])
    if broken("shard-runs-the-plan-binary"):
        binary, digest = Path(plan_directory, BINARY_NAME), plan["binary_sha256"]
    else:
        binary, digest = copy_binary(Path(plan_directory, BINARY_NAME), output_directory)
    exit_code, standard_output, standard_error = runner([str(binary)] + cell.arguments())
    lines = result_lines_of(standard_output)
    own = [line for line in lines if line.startswith("E7RESULT name=e197 ")]
    succeeded = exit_code == 0 and len(own) == 1
    tags = "" if broken("shard-result-untagged") else f" task={task} attempt={attempt} shard_host={host} binary_sha256={digest} shard={position}"
    written = [line for line in lines if line not in own] + [line + tags for line in own]
    if not succeeded:
        written.append(f"E7RESULT name=e197_cell_lost key={cell.key()} exit={exit_code} result_lines={len(own)} task={task} attempt={attempt}")
    Path(output_directory, RESULT_FILE).write_text("\n".join(written) + "\n", encoding="utf-8")
    if binary.parent == Path(output_directory):
        binary.unlink()   # 页缓存已经记在这一片的 scope 上；拷贝用完删掉，不占共享根
    if not succeeded and not broken("failed-cell-passes"):
        print(f"  ✗ 格 {cell.key()} 没跑成：装置退 {exit_code}、结果行 {len(own)} 行；装置标准错误末尾：{standard_error[-2000:]}", file=sys.stderr)  # gate-lint:detail
        return 1
    print(f"片 {position}：{cell.key()}，退 {exit_code}")
    return 0


def merge_inputs_problem(environment):
    """合并要读的两样（计划目录里的 cells.json、各片产物表）都得是在的文件：没设、指到目录或不在的，交一句说明，不往下读（照 mutation-shard-run.sh 合并那一步的写法）。"""
    if broken("merge-reads-unchecked-inputs"):
        return None
    plan_directory, outputs = environment.get(PLAN_VARIABLE, ""), environment.get(OUTPUTS_VARIABLE, "")
    if not plan_directory or not Path(plan_directory, CELLS_FILE).is_file():
        return f"合并那一步读不到计划（{PLAN_VARIABLE}={plan_directory or '没设'}，里面的 {CELLS_FILE} 不是在的文件）"
    if not outputs or not Path(outputs).is_file():
        return f"合并那一步没给在的各片产物表（{OUTPUTS_VARIABLE}={outputs or '没设'}）"
    return None


def merge_phase(environment, artifact_path):
    """合并：按片号次序把各片 result.out 里的结果行写进这一组自己的产物文件（排他新建，已在就拒）。"""
    problem = merge_inputs_problem(environment)
    if problem:
        print(f"  ✗ {problem}（→ 怎么办：合并这一步只由 task-server 起，环境里要有 {PLAN_VARIABLE} 与 {OUTPUTS_VARIABLE}；"
              f"产物 {artifact_path} 还没写，把这一件的 task-server/task submit 整条原样重交）", file=sys.stderr)  # gate-lint:detail
        return 1
    plan = json.loads(Path(environment.get(PLAN_VARIABLE, ""), CELLS_FILE).read_text(encoding="utf-8"))
    rows = [line.split("\t") for line in Path(environment.get(OUTPUTS_VARIABLE, "")).read_text(encoding="utf-8").splitlines() if line.strip()]
    if not broken("merge-not-in-shard-order"):
        rows.sort(key=lambda row: int(row[0]))
    if len(rows) != len(plan["cells"]) or any(len(row) != 3 for row in rows):
        print(f"  ✗ outputs.tsv 有 {len(rows)} 行，计划里 {len(plan['cells'])} 格", file=sys.stderr)  # gate-lint:detail
        return 1
    fields = {"segment": plan["segment"], "round": plan["round"], "group": plan["group"]}
    lines = [config_line("segment", plan["binary_sha256"], shards=len(rows), **fields)]
    for index, host, directory in rows:
        result = Path(directory, RESULT_FILE)
        if not result.exists():
            print(f"  ✗ 第 {index} 片（{host}）的 {result} 不在", file=sys.stderr)  # gate-lint:detail
            return 1
        lines += [line for line in result.read_text(encoding="utf-8").splitlines() if line.startswith("E7RESULT ")]
    lines.append(done_line("segment", len(rows), **fields))
    if not write_new_file(absolute_output(artifact_path), lines):
        print(f"  ✗ 产物 {artifact_path} 已在，不覆盖（→ 怎么办：这一组已经合并过；要重跑就换一个新文件名重交）", file=sys.stderr)  # gate-lint:detail
        return 1
    print(f"合并：{len(rows)} 片写进 {artifact_path}")
    return 0


def run_segment(segment, repeat, group, cell_list, binary, artifact_path, environment, runner=subprocess_runner):
    cells = cells_of_group(segment, repeat, group, cell_list)
    if cells is None:
        print(f"段 {segment}、第 {repeat} 轮、组 {group}、--cells {cell_list!r} 对不上（第 1–5 轮组写 glue-1..3 或 read-stage-cache、不带 --cells；"
              f"第 6–10 轮组写 rerun-<n>、带 --cells，至多 {MAXIMUM_CELLS_PER_GROUP} 格）", file=sys.stderr)
        return 2
    phase = environment.get(PHASE_VARIABLE, "")
    if phase == "plan":
        return plan_phase(cells, binary, environment.get(PLAN_VARIABLE, ""), segment, repeat, group)
    if phase == "shard":
        return shard_phase(environment, runner)
    if phase == "merge":
        return merge_phase(environment, artifact_path)
    print("segment 只经统一任务服务分片跑（环境里没有 TASK_SERVER_SHARD_PHASE）：照 list-submissions 打出的整条交 task-server/task submit", file=sys.stderr)
    return 2


# ---- 打出要交的整条（不交、不跑）----

def submission(words):
    return "task-server/task submit -- python3 " + " ".join([THIS_SCRIPT] + [str(word) for word in words])


def segment_output_name(date, segment, repeat, group):
    return f"research/results/e197-{date}-{segment}-r{repeat}-{group}.out"


def segment_submission(binary, date, segment, repeat, group, cell_list=None):
    words = ["segment", segment, "--round", repeat, "--group", group, "--binary", binary, "--out", segment_output_name(date, segment, repeat, group)]
    return submission(words + (["--cells", cell_list] if cell_list else []))


def submissions_by_batch(binary, date):
    """{批号: [整条…]}：批 0 是 checks 与 compare（不分片）；批 1 起每批 SEGMENT_TASKS_PER_BATCH 件 segment。"""
    batches = {0: [submission(["checks", "--binary", binary, "--out", f"research/results/e197-{date}-checks.out"]),
                   submission(["compare", "--binary", binary, "--out", f"research/results/e197-{date}-compare.out"])]}
    groups = every_first_round_group()
    for number, start in enumerate(range(0, len(groups), SEGMENT_TASKS_PER_BATCH), 1):
        batches[number] = [segment_submission(binary, date, *group) for group in groups[start:start + SEGMENT_TASKS_PER_BATCH]]
    return batches


def segment_of_geometry(named, fill):
    return next(segment for segment, geometry in SEGMENTS.items() if geometry == (named, fill))


def rerun_submissions(binary, date, unstable_cells):
    """V2 复测轮：report.cells_needing_a_rerun 点名的格按段归拢，每 MAXIMUM_CELLS_PER_GROUP 格一组，每组重复号 6–10 各交一件。"""
    by_segment = {}
    for arm, devices, fill, named in unstable_cells:
        by_segment.setdefault(segment_of_geometry(named, fill), []).append(f"{arm}:{devices}")
    lines = []
    for segment, items in by_segment.items():
        for number, start in enumerate(range(0, len(items), MAXIMUM_CELLS_PER_GROUP), 1):
            cell_list = ",".join(items[start:start + MAXIMUM_CELLS_PER_GROUP])
            lines += [segment_submission(binary, date, segment, repeat, f"{RERUN_GROUP_PREFIX}{number}", cell_list) for repeat in SECOND_ROUNDS]
    return lines


def option(arguments, name, default=None):
    if name in arguments:
        index = arguments.index(name)
        return arguments[index + 1] if index + 1 < len(arguments) else None
    return default


def options_after(arguments, name):
    """name 后面到下一个 -- 起头的词之前的全部值。"""
    if name not in arguments:
        return []
    values = []
    for word in arguments[arguments.index(name) + 1:]:
        if word.startswith("--"):
            break
        values.append(word)
    return values


def main(arguments, environment=None):
    environment = os.environ if environment is None else environment
    if arguments[:1] == ["--selftest"]:
        return selftest()
    binary, artifact_path, date = option(arguments, "--binary"), option(arguments, "--out"), option(arguments, "--date")
    repeat_text, group = option(arguments, "--round", ""), option(arguments, "--group", "")
    repeat = int(repeat_text) if (repeat_text or "").isdigit() else None
    if arguments[:1] == ["list-cells"] and len(arguments) >= 2 and repeat is not None:
        cells = cells_of_group(arguments[1], repeat, group, option(arguments, "--cells"))
        if cells is not None:
            for cell in cells:
                print(cell.key())
            return 0
    if arguments[:1] == ["checks"] and binary and artifact_path:
        return run_checks(binary, artifact_path)
    if arguments[:1] == ["compare"] and binary and artifact_path:
        return run_compare(binary, artifact_path)
    if arguments[:1] == ["segment"] and len(arguments) >= 2 and repeat is not None and binary and artifact_path:
        return run_segment(arguments[1], repeat, group, option(arguments, "--cells"), binary, artifact_path, environment)
    if arguments[:1] == ["list-submissions"] and binary and date:
        batches = submissions_by_batch(binary, date)
        wanted = option(arguments, "--batch")
        if wanted is None or (wanted.isdigit() and int(wanted) in batches):
            for number, lines in batches.items():
                if wanted is None or int(wanted) == number:
                    print(f"# 第 {number} 批（{len(lines)} 件）")
                    for line in lines:
                        print(line)
            return 0
    if arguments[:1] == ["list-reruns"] and binary and date and options_after(arguments, "--artifacts") and options_after(arguments, "--status"):
        loaded = report.load(options_after(arguments, "--artifacts"), options_after(arguments, "--status"))
        lines = rerun_submissions(binary, date, report.cells_needing_a_rerun(loaded))
        print(f"# 第二轮（重复号 6–10）：{len(lines)} 件")
        for line in lines:
            print(line)
        return 0
    print(__doc__, file=sys.stderr)
    return 2


# ---- 自证：假的装置二进制（一份 python 脚本）、假的服务环境 ----

FAKE_BINARY_SOURCE = """#!{python}
import os
import sys
words = sys.argv[1:]
values = dict(zip(words[::2], words[1::2]))
print("E7RESULT name=fake_binary path=" + os.path.realpath(sys.argv[0]))
if "--compare" in words:
    print("E7RESULT name=e197_compare arm={{}} devices={{}} fill={{}} named={{}} equal=true records=1".format(
        values["--arm"], values["--devices"], values["--fill"], values["--named"]))
elif words == ["--checks"]:
    print("E7RESULT name=e197_checks fake=yes")
elif words == ["--diff-set"]:
    print("E7RESULT name=e197_diff case=D1 arm=split equal=true reference_records=1")
else:
    print("E7RESULT name=e197 arm={{}} devices={{}} fill={{}} named={{}} repeat={{}}".format(
        values["--arm"], values["--devices"], values["--fill"], values["--named"], values["--repeat"]))
sys.exit(int(os.environ.get("E197_FAKE_BINARY_EXIT", "0")))
"""


def make_fake_binary(directory):
    path = Path(directory, "source-binary")
    path.write_text(FAKE_BINARY_SOURCE.format(python=sys.executable), encoding="utf-8")
    os.chmod(path, stat.S_IRWXU)
    return path


def fake_runner(exit_code=0):
    def runner(command):
        finished = subprocess.run(command, capture_output=True, text=True, check=False,
                                  env=dict(os.environ, E197_FAKE_BINARY_EXIT=str(exit_code)))
        return finished.returncode, finished.stdout, finished.stderr
    return runner


def planned(directory, segment, repeat, group):
    """在 directory 里演一遍计划那一步，交 (假二进制, 计划目录)。"""
    source = make_fake_binary(directory)
    plan = Path(directory, "plan")
    plan.mkdir()
    code = run_segment(segment, repeat, group, None, str(source), "unused.out", {PHASE_VARIABLE: "plan", PLAN_VARIABLE: str(plan)})
    check(code == 0, f"计划那一步退 {code}")
    return source, plan


def shard_environment(plan, output, position, **changes):
    environment = {PHASE_VARIABLE: "shard", SHARD_VARIABLE: position, PLAN_VARIABLE: str(plan), OUTPUT_VARIABLE: str(output),
                   "TASK_SERVER_TASK": "4711", "TASK_SERVER_ATTEMPT": "2", HOST_VARIABLE: "test-host-address"}
    environment.update(changes)
    return {name: value for name, value in environment.items() if value is not None}


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def case_cell_counts_match_the_registration():
    sizes = {(segment, group): len(cells_of_group(segment, 1, group)) for segment in SEGMENTS for group in GLUE_GROUPS + (READ_STAGE_CACHE_GROUP,)}
    expected = {}
    for segment, glue_sizes, cache_size in (("k64-full", (8, 8, 5), 4), ("k64-half", (8, 8, 4), 8), ("k1-full", (8, 8, 4), 4)):
        expected.update({(segment, group): size for group, size in zip(GLUE_GROUPS, glue_sizes)})
        expected[(segment, READ_STAGE_CACHE_GROUP)] = cache_size
    check(sizes == expected, sizes)
    every = [cell.key() for segment, repeat, group in every_first_round_group() for cell in cells_of_group(segment, repeat, group)]
    check(len(every_first_round_group()) == 60 and len(every) == 385 and len(set(every)) == 385, (len(every_first_round_group()), len(every)))
    check(sum(1 for key in every if key.split("/")[1].startswith("read_stage_cache")) == 80, "丙 80 格")
    check(sum(1 for key in every if "/measurement_pc/" in key) == 5 and cells_of_group("k64-full", 1, "glue-1")[0].arm == "empty", "空跑排前、measurement_pc 5 格")
    compare = compare_cells()
    check(len(compare) == 52 and len({cell.key() for cell in compare}) == 52 and all(cell.arguments()[-1] == "--compare" for cell in compare), len(compare))
    check(cells_of_group("k64-full", 6, "glue-1") is None and cells_of_group("k64-full", 1, "rerun-1", "baseline:2") is None, "轮与组对不上要拒")
    rerun = cells_of_group("k1-full", 7, "rerun-1", "baseline:2,empty:2")
    check([cell.key() for cell in rerun] == ["e197/baseline/2/full/1/7", "e197/empty/2/full/1/7"], [cell.key() for cell in rerun])
    check(cells_of_group("k1-full", 7, "rerun-1", "nope:2") is None and cells_of_group("k1-full", 7, "rerun-1", ",".join(["baseline:1"] * 9)) is None, "认不得的臂、多于 8 格要拒")


def case_the_plan_copies_the_binary_and_writes_one_shard_per_cell():
    with tempfile.TemporaryDirectory() as directory:
        source, plan = planned(directory, "k64-half", 2, READ_STAGE_CACHE_GROUP)
        written = json.loads(Path(plan, "plan.json").read_text(encoding="utf-8"))
        wanted = [cell.key() for cell in cells_of_group("k64-half", 2, READ_STAGE_CACHE_GROUP)]
        check(set(written) == {"shards", "note"} and [shard["label"] for shard in written["shards"]] == wanted
              and all(set(shard) == {"label"} for shard in written["shards"]), written)
        recorded = json.loads(Path(plan, CELLS_FILE).read_text(encoding="utf-8"))
        check(recorded["binary_sha256"] == sha256_of_file(source) == sha256_of_file(Path(plan, BINARY_NAME)) and len(recorded["cells"]) == 8, recorded)


def case_a_shard_runs_its_own_binary_copy_and_tags_the_result_line():
    with tempfile.TemporaryDirectory() as directory:
        source, plan = planned(directory, "k1-full", 3, "glue-2")
        output = Path(directory, "output-1-attempt-2")
        output.mkdir()
        code = run_segment("k1-full", 3, "glue-2", None, str(source), "unused.out", shard_environment(plan, output, "1/8"), fake_runner())
        check(code == 0, f"片退 {code}")
        lines = Path(output, RESULT_FILE).read_text(encoding="utf-8").splitlines()
        cell = cells_of_group("k1-full", 3, "glue-2")[1]
        own = [line for line in lines if line.startswith("E7RESULT name=e197 ")]
        check(len(own) == 1 and f"arm={cell.arm} devices={cell.devices} fill=full named=1 repeat=3" in own[0], own)
        tags = f" task=4711 attempt=2 shard_host=test-host-address binary_sha256={sha256_of_file(source)} shard=1/8"
        check(own[0].endswith(tags), f"结果行要以「{tags}」收尾：{own[0]}")
        started = next(line for line in lines if "name=fake_binary " in line)
        check(started.endswith(f"path={os.path.realpath(Path(output, BINARY_NAME))}"), f"片要起自己产物目录里那份拷贝：{started}")
        check(not Path(output, BINARY_NAME).exists(), "片跑完删掉自己的二进制拷贝")


def case_a_failing_cell_fails_the_shard():
    with tempfile.TemporaryDirectory() as directory:
        source, plan = planned(directory, "k1-full", 3, "glue-1")
        output = Path(directory, "output-0-attempt-1")
        output.mkdir()
        environment = shard_environment(plan, output, "0/8")
        code = run_segment("k1-full", 3, "glue-1", None, str(source), "unused.out", environment, fake_runner(exit_code=1))
        check(code == 1, f"装置退 1 时片要退 1，退了 {code}")
        check("name=e197_cell_lost key=e197/empty/1/full/1/3 exit=1" in Path(output, RESULT_FILE).read_text(encoding="utf-8"), "没跑成的格写 cell_lost 行")
        for missing in ("TASK_SERVER_ATTEMPT", "TASK_SERVER_TASK", HOST_VARIABLE):
            code = run_segment("k1-full", 3, "glue-1", None, str(source), "unused.out", shard_environment(plan, output, "0/8", **{missing: None}), fake_runner())
            check(code == 1, f"读不到 {missing} 时片要退 1（读不到不当 0），退了 {code}")
        code = run_segment("k1-full", 3, "glue-1", None, str(source), "unused.out", shard_environment(plan, output, "0/7"), fake_runner())
        check(code == 1, f"片数与计划对不上要退 1，退了 {code}")


def case_merge_writes_the_shards_in_order_and_refuses_an_existing_file():
    with tempfile.TemporaryDirectory() as directory:
        source, plan = planned(directory, "k64-full", 1, "glue-3")
        rows = []
        for index in range(5):
            output = Path(directory, f"output-{index}-attempt-1")
            output.mkdir()
            Path(output, RESULT_FILE).write_text(f"E7RESULT name=e197 shard_order={index}\n", encoding="utf-8")
            rows.append(f"{index}\t{'peer' if index % 2 else 'local'}\t{output}")
        outputs = Path(directory, "outputs.tsv")
        outputs.write_text("".join(row + "\n" for row in reversed(rows)), encoding="utf-8")
        environment = {PHASE_VARIABLE: "merge", PLAN_VARIABLE: str(plan), OUTPUTS_VARIABLE: str(outputs)}
        artifact = Path(directory, "merged.out")
        code = run_segment("k64-full", 1, "glue-3", None, str(source), str(artifact), environment)
        check(code == 0, f"合并退 {code}")
        text = artifact.read_text(encoding="utf-8")
        orders = [int(line.split("shard_order=")[1]) for line in text.splitlines() if "shard_order=" in line]
        check(orders == [0, 1, 2, 3, 4], f"要按片号次序写：{orders}")
        check(text.startswith("E7RESULT name=e197_config step=segment ")
              and "E7RESULT name=e197_segment_done step=segment cells=5 segment=k64-full round=1 group=glue-3" in text, text)
        code = run_segment("k64-full", 1, "glue-3", None, str(source), str(artifact), environment)
        check(code == 1 and artifact.read_text(encoding="utf-8") == text, f"产物已在时合并要退 1、不动它，退了 {code}")
        Path(directory, "output-2-attempt-1", RESULT_FILE).unlink()
        check(run_segment("k64-full", 1, "glue-3", None, str(source), str(Path(directory, "other.out")), environment) == 1
              and not Path(directory, "other.out").exists(), "有一片的 result.out 不在要退 1、不写产物")


def case_merge_without_the_service_tables_explains_and_exits_one():
    """合并那一步在各片产物表没设、是空串、指到目录或不在，或计划目录没设时：说明里点名那个变量与出路，退 1，不抛 Traceback、不写产物。"""
    with tempfile.TemporaryDirectory() as directory:
        source, plan = planned(directory, "k64-full", 1, "glue-3")
        artifact = Path(directory, "merged.out")
        cases = (("各片产物表没设", {PLAN_VARIABLE: str(plan)}, OUTPUTS_VARIABLE),
                 ("各片产物表是空串", {PLAN_VARIABLE: str(plan), OUTPUTS_VARIABLE: ""}, OUTPUTS_VARIABLE),
                 ("各片产物表指到目录", {PLAN_VARIABLE: str(plan), OUTPUTS_VARIABLE: directory}, OUTPUTS_VARIABLE),
                 ("各片产物表指到不在的文件", {PLAN_VARIABLE: str(plan), OUTPUTS_VARIABLE: str(Path(directory, "no-such.tsv"))}, OUTPUTS_VARIABLE),
                 ("计划目录没设", {OUTPUTS_VARIABLE: str(Path(directory, "no-such.tsv"))}, PLAN_VARIABLE))
        for label, variables, named in cases:
            captured = io.StringIO()
            try:
                with contextlib.redirect_stderr(captured):
                    code = run_segment("k64-full", 1, "glue-3", None, str(source), str(artifact), dict(variables, **{PHASE_VARIABLE: "merge"}))
            except Exception as error:   # 弄坏开关下读不到就抛：要给说明后退 1，不许抛
                raise AssertionError(f"{label}时合并抛了 {type(error).__name__}，要给说明后退 1") from None
            text = captured.getvalue()
            check(code == 1 and named in text and "怎么办" in text and "Traceback" not in text and not artifact.exists(),
                  f"{label}：退 {code}，说明 {text!r}，产物{'在' if artifact.exists() else '不在'}")


def case_segment_outside_the_service_is_refused():
    check(run_segment("k1-full", 1, "glue-1", None, "/b", "x.out", {}) == 2, "环境里没有 TASK_SERVER_SHARD_PHASE 要退 2")
    check(run_segment("k1-full", 1, "glue-9", None, "/b", "x.out", {PHASE_VARIABLE: "plan"}) == 2, "认不得的组要退 2")


def case_s1_tells_a_moved_function_from_a_changed_one():
    with tempfile.TemporaryDirectory() as directory:
        body = [f"// 行 {number}" for number in range(1, 7)] + ["pub fn scan_demo(", "    body();", "}"]
        path = Path(directory, "demo.rs")
        path.write_text("\n".join(body) + "\n", encoding="utf-8")
        expected = sha256_of_lines(body, 4, 9)
        entry = ("demo.rs", 4, 9, expected, "pub fn scan_demo(", 3)
        check(implementation_line(directory, entry).endswith("status=same"), implementation_line(directory, entry))
        path.write_text("\n".join(["// 新增", "// 新增"] + body) + "\n", encoding="utf-8")
        moved = implementation_line(directory, entry)
        check(moved.endswith("status=moved_same") and "found_start=6" in moved, moved)
        changed_body = list(body)
        changed_body[7] = "    other_body();"
        path.write_text("\n".join(changed_body) + "\n", encoding="utf-8")
        check(implementation_line(directory, entry).endswith("status=changed"), implementation_line(directory, entry))
        check(not implementation_is_the_registered_one([implementation_line(directory, entry)]), "changed 要停")


def case_the_registered_implementation_ranges_hold_today():
    lines = implementation_lines(REPOSITORY_ROOT)
    check(len(lines) == 3 and implementation_is_the_registered_one(lines), lines)


def case_checks_and_compare_write_new_files_only():
    with tempfile.TemporaryDirectory() as directory:
        source = make_fake_binary(directory)
        artifact = Path(directory, "checks.out")
        check(run_checks(source, str(artifact), REPOSITORY_ROOT, fake_runner()) == 0, "checks 退 0")
        text = artifact.read_text(encoding="utf-8")
        check(text.count("name=e197_s1 ") == 3 and "mode=--checks exit=0" in text and "mode=--diff-set exit=0" in text
              and "name=e197_segment_done step=checks cells=2" in text and f"binary_sha256={sha256_of_file(source)}" in text, text)
        check(run_checks(source, str(artifact), REPOSITORY_ROOT, fake_runner()) == 1 and artifact.read_text(encoding="utf-8") == text, "已在的产物不覆盖")
        compare = Path(directory, "compare.out")
        check(run_compare(source, str(compare), fake_runner()) == 0 and compare.read_text(encoding="utf-8").count("name=e197_compare ") == 52, "compare 52 行")
        failing = Path(directory, "compare-failing.out")
        check(run_compare(source, str(failing), fake_runner(exit_code=1)) == 1
              and failing.read_text(encoding="utf-8").count("name=e197_cell_lost ") == 52, "没跑成的格报 cell_lost、不当相等")


def case_list_submissions_cover_every_task_once():
    batches = submissions_by_batch("/w/bin", "2026-11-02")
    check(len(batches) == 11 and len(batches[0]) == 2 and all(1 <= len(batches[number]) <= SEGMENT_TASKS_PER_BATCH for number in batches if number),
          {number: len(lines) for number, lines in batches.items()})
    segment_lines = [line for number in batches if number for line in batches[number]]
    every = batches[0] + segment_lines
    check(len(segment_lines) == 60 == len(set(segment_lines)), len(segment_lines))
    outputs = [line.split(" --out ")[1].split()[0] for line in every]
    check(len(set(outputs)) == 62 and all(name.startswith("research/results/e197-2026-11-02-") for name in outputs), outputs[:3])
    forbidden = ("run-with-memory-cap", "capped.sh", "--memory", "--threads", "--peer", "SINGLEFS_THREAD_CAP", "--target-dir", "CARGO_TARGET_DIR")
    check(not [line for line in every for word in forbidden if word in line], "交给服务的整条不带包装、上限、线程、挑台与编译目录")
    check(all(line.startswith(f"task-server/task submit -- python3 {THIS_SCRIPT} ") for line in every), every[0])
    reruns = rerun_submissions("/w/bin", "2026-11-02", [("baseline", 2, "full", 1), ("empty", 2, "full", 1)])
    check(len(reruns) == 5 and all(" --group rerun-1 " in line and line.endswith("--cells baseline:2,empty:2") for line in reruns)
          and " --round 6 " in reruns[0] and " --round 10 " in reruns[-1], reruns)


SELFTEST_CASES = (
    ("cell-counts-match-the-registration", case_cell_counts_match_the_registration),
    ("the-plan-copies-the-binary-and-writes-one-shard-per-cell", case_the_plan_copies_the_binary_and_writes_one_shard_per_cell),
    ("a-shard-runs-its-own-binary-copy-and-tags-the-result-line", case_a_shard_runs_its_own_binary_copy_and_tags_the_result_line),
    ("a-failing-cell-fails-the-shard", case_a_failing_cell_fails_the_shard),
    ("merge-writes-the-shards-in-order-and-refuses-an-existing-file", case_merge_writes_the_shards_in_order_and_refuses_an_existing_file),
    ("merge-without-the-service-tables-explains-and-exits-one", case_merge_without_the_service_tables_explains_and_exits_one),
    ("segment-outside-the-service-is-refused", case_segment_outside_the_service_is_refused),
    ("s1-tells-a-moved-function-from-a-changed-one", case_s1_tells_a_moved_function_from_a_changed_one),
    ("the-registered-implementation-ranges-hold-today", case_the_registered_implementation_ranges_hold_today),
    ("checks-and-compare-write-new-files-only", case_checks_and_compare_write_new_files_only),
    ("list-submissions-cover-every-task-once", case_list_submissions_cover_every_task_once),
)


def selftest():
    failed = 0
    for label, case in SELFTEST_CASES:
        try:
            case()
            print(f"  ✓ 自检 {label}")
        except AssertionError as error:
            failed += 1
            print(f"  ✗ 自检 {label}：{error}")  # gate-lint:detail
    if failed:
        print(f"自证红：{failed} 项没过，共 {len(SELFTEST_CASES)} 项（→ 怎么办：按上面 ✗ 行的项名改 drive.py 里对应的判法，E197_DRIVE_BREAK 设着的话本来就该红）")
        return 1
    print(f"自证通过：{len(SELFTEST_CASES)} 项")
    if any(os.environ.get(name) for name in BREAK_VARIABLES):
        return 0   # 弄坏开关生效的这一趟（开关轮起的子进程）只跑各项，不再套开关轮
    return selftest_breaks()


def selftest_breaks():
    """逐个打开弄坏开关起自己的 --selftest，要求退 1 且红在点名的那一项。"""
    bad = 0
    for switch in BREAK_SWITCHES:
        result = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "--selftest"], env=dict(os.environ, E197_DRIVE_BREAK=switch),
                                capture_output=True, text=True, check=False)
        target = BREAK_TO_CASE[switch]
        if result.returncode == 1 and f"✗ 自检 {target}：" in result.stdout:
            print(f"  ✓ 弄坏开关 {switch} 红在 {target}")
        else:
            bad += 1
            print(f"  ✗ 弄坏开关 {switch} 没红在 {target}（退出码 {result.returncode}）")  # gate-lint:detail
    print(f"弄坏开关自证{'通过' if not bad else '红'}：{len(BREAK_SWITCHES)} 个开关，{bad} 个没红在点名的项"
          + ("（→ 怎么办：对着没红的开关核 drive.py 里那条判法有没有被开关真的弄坏、BREAK_TO_CASE 点名的项对不对）" if bad else ""))
    return 1 if bad else 0


BREAK_VARIABLES = ("E197_DRIVE_BREAK", "E197_REPORT_BREAK")


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在 --selftest 里认（文件头「弄坏开关」；E197_REPORT_BREAK 是 report.py 的开关，本脚本 import 它，同样只在自证里认）：
    命令行的第一个参数不是 --selftest 而 BREAK_VARIABLES 里有一个非空（不管值是什么），退 2 并点名它只在自证里认；在参数解析之前判。"""
    for name in BREAK_VARIABLES:
        if os.environ.get(name) and arguments[:1] != ["--selftest"]:
            print(f"  ✗ {name} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
            print(f"     → 怎么办：去掉 {name} 再跑；要证红就跑 {name}=<项> python3 {THIS_SCRIPT} --selftest", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    sys.exit(main(sys.argv[1:]))
