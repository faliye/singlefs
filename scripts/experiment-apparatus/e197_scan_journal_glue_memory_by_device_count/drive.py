#!/usr/bin/env python3
# admission: always 每一次调都在此刻的仓与装置二进制上现跑，上一次的产物不替这一次作保
# run-condition: command bash python3
"""E197（扫环胶水内存随盘数）驱动：按跑前登记 research/prompts/e197-preregistration.md 第五、六、十一、十四节，
一格一个进程、经内存包装起装置二进制，把结果行与包装写的 cgroup 峰值合成产物；判归同目录的 report.py。

用法（在仓根跑，装置二进制是 compile-then-swap 编出来的 <草稿目录>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count）：
  drive.py list-cells <段>                                  逐行列这一段的格，不起任何进程
  drive.py checks --binary <二进制> --out <产物.out>         停机条款 S1（两段 sha256）、装置 --checks、差分集 D（--diff-set）
  drive.py segment <段> --binary <二进制> --out <产物.out> --peaks <峰值表.tsv> [--jobs 5] [--cap 8G]
                                                            段：compare（三份胶水臂在生产格上与 scan_journal 逐键逐值比，36 格，不量峰值）、
                                                            k64-full（含 measurement_pc）、k64-half、k1-full；写完一段追加一行 name=e197_segment_done，
                                                            再起同一条命令时已写过收尾行的段跳过（续跑）
  drive.py rerun-unstable --binary … --out … --peaks …       report.py --unstable 点名的格（第一轮 5 次散布 > 2%）再跑重复号 6–10 的一轮
  drive.py --selftest                                       假的包装与假的二进制上的自证（不起装置、不起内存包装）
  drive.py --selftest-breaks                                逐个打开弄坏开关起自己的 --selftest，要求每个开关都红在点名的那一项
退出码：0 做完；1 自证红或有格丢了；2 用法错；3 停机条款 S1（实现不是登记时那一份）。
峰值表、每格的包装退出码与重试都写进产物：V1（包装退 250 的那一格把上限改 12G 重跑，退 245、251–254 原样重跑，至多 3 次）。

弄坏开关（只给自证用；环境变量 E197_DRIVE_BREAK=<一个值>，不认识的值直接拒绝）：
  E197_DRIVE_BREAK=cap-not-raised          包装退 250 之后不改上限，原样重跑
      自证项 「✗ 自检 a-cap-hit-is-rerun-at-twelve-gibibytes：」
  E197_DRIVE_BREAK=lost-cell-dropped       丢了的格不写 name=e197_cell_lost 行，悄悄少一格
      自证项 「✗ 自检 a-cell-that-keeps-failing-is-reported-lost-not-zero：」
  E197_DRIVE_BREAK=s1-always-same          S1 的核对恒报 same
      自证项 「✗ 自检 s1-tells-a-moved-function-from-a-changed-one：」
  E197_DRIVE_BREAK=segment-not-skipped     已写过收尾行的段照样重跑
      自证项 「✗ 自检 a-finished-segment-is-skipped-on-the-second-call：」
  E197_DRIVE_BREAK=missing-peak-as-zero    峰值表里没有这一格的行时当 0 收下
      自证项 「✗ 自检 a-missing-peak-row-is-a-failed-attempt：」
"""
import concurrent.futures
import hashlib
import os
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True   # 不往仓里写 __pycache__
REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPOSITORY_ROOT / "scripts" / "hosts"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import heavy_threads  # noqa: E402
import memory_peaks_lib  # noqa: E402
import report  # noqa: E402

BREAK_SWITCHES = ("cap-not-raised", "lost-cell-dropped", "s1-always-same", "segment-not-skipped", "missing-peak-as-zero")
BREAK_TO_CASE = {
    "cap-not-raised": "a-cap-hit-is-rerun-at-twelve-gibibytes",
    "lost-cell-dropped": "a-cell-that-keeps-failing-is-reported-lost-not-zero",
    "s1-always-same": "s1-tells-a-moved-function-from-a-changed-one",
    "segment-not-skipped": "a-finished-segment-is-skipped-on-the-second-call",
    "missing-peak-as-zero": "a-missing-peak-row-is-a-failed-attempt",
}
WRAPPER = "scripts/hosts/run-with-memory-cap.sh"
DEFAULT_CAP = "8G"
RAISED_CAP = "12G"
MAX_ATTEMPTS = 3
RETRY_ON = (245, 251, 252, 253, 254)
SEGMENTS = {"k64-full": (64, "full"), "k64-half": (64, "half"), "k1-full": (1, "full")}
COMPARE_SEGMENT = "compare"
COMPARE_ARMS = ("baseline_pc", "split", "split_pc")
# 登记第十一节 S1：登记时的实现。(文件, 起行, 止行, sha256, 函数签名起头, 签名在这一段里的行偏移)
IMPLEMENTATION_RANGES = (
    ("crates/singlefs-core/src/recovery.rs", 4431, 4484,
     "0ca338aaafcfff2796656134c9246295e33d58503f439d61588a9cb394697c55", "pub fn scan_journal(", 5),
    ("crates/singlefs-replay-prefix/src/lib.rs", 4029, 4058,
     "60e0648acc3f3a3a730b3c1731781cf0eda6e28068a56b6e0aad8d58dc1d2d41", "pub fn first_copy_of_each_record_key(", 0),
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


def cells_of(segment, first_repeat=1):
    """一段的格：空跑、四个扫环臂各 盘数 1–4 × 5 次；k64-full 另加 measurement_pc（1 盘、5 次）。空跑排在最前。
    compare 段：三份胶水臂 × 盘数 1–4 × 三个几何，各一格（登记第十一节 S2：每一格上的表与 scan_journal 逐键逐值比）。"""
    if segment == COMPARE_SEGMENT:
        return [Cell(arm, devices, fill, named, 1, compare=True) for arm in COMPARE_ARMS for named, fill in SEGMENTS.values()
                for devices in report.DEVICE_COUNTS]
    named, fill = SEGMENTS[segment]
    arms = ("empty",) + report.SCANNING_ARMS
    repeats = range(first_repeat, first_repeat + report.REPEATS_PER_ROUND)
    cells = [Cell(arm, devices, fill, named, repeat) for arm in arms for devices in report.DEVICE_COUNTS for repeat in repeats]
    if segment == "k64-full":
        cells += [Cell("measurement_pc", 1, fill, named, repeat) for repeat in repeats]
    return cells


def command_of(binary, cell, cap):
    return ["bash", WRAPPER, cap, str(binary)] + cell.arguments()


def environment_of(cell, peaks_path):
    return dict(os.environ, RUN_WITH_MEMORY_CAP_PEAKS=str(peaks_path), RUN_WITH_MEMORY_CAP_KEY=cell.key(),
                RUN_WITH_MEMORY_CAP_HOST_LOOKUP="off")


def result_lines_of(standard_output):
    return [line for line in standard_output.splitlines() if line.startswith("E7RESULT ")]


def subprocess_runner(command, environment):
    """(退出码, 标准输出, 标准错误)。"""
    finished = subprocess.run(command, env=environment, cwd=REPOSITORY_ROOT, capture_output=True, text=True, check=False)
    return finished.returncode, finished.stdout, finished.stderr


def cgroup_peak_bytes(peaks_path, key):
    row = memory_peaks_lib.read_peak_rows(str(peaks_path)).get(key)
    if row is None:
        return 0 if broken("missing-peak-as-zero") else None
    return row[0]


def run_one_cell(binary, cell, peaks_path, cap, runner=subprocess_runner):
    """跑一格，按 V1 重试。交 (产物行列表, 丢没丢)：成功交装置结果行加一条 name=e197_cgroup；丢了交一条 name=e197_cell_lost。"""
    current_cap = cap
    attempts_log = []
    for attempt in range(1, MAX_ATTEMPTS + 1):
        exit_code, standard_output, _ = runner(command_of(binary, cell, current_cap), environment_of(cell, peaks_path))
        attempts_log.append(f"{exit_code}@{current_cap}")
        lines = result_lines_of(standard_output)
        own_prefix = "E7RESULT name=e197_compare " if cell.compare else "E7RESULT name=e197 "
        own = [line for line in lines if line.startswith(own_prefix)]
        peak = cgroup_peak_bytes(peaks_path, cell.key()) if exit_code == 0 and not cell.compare else None
        if exit_code == 0 and len(own) == 1 and cell.compare:
            return [line for line in lines if line not in own] + own, False
        if exit_code == 0 and len(own) == 1 and peak is not None:
            others = [line for line in lines if line not in own]
            cgroup = (f"E7RESULT name=e197_cgroup key={cell.key()} cap={current_cap} attempts={attempt} "
                      f"exit=0 cgroup_peak_bytes={peak}")
            return others + own + [cgroup], False
        if exit_code == 250:
            current_cap = current_cap if broken("cap-not-raised") else RAISED_CAP
        elif exit_code not in RETRY_ON and exit_code != 0:
            break
    if broken("lost-cell-dropped"):
        return [], True
    return [f"E7RESULT name=e197_cell_lost key={cell.key()} attempts={len(attempts_log)} exits={','.join(attempts_log)}"], True


def run_cells(binary, cells, peaks_path, cap, jobs, runner=subprocess_runner):
    """并行跑，按 cells 的次序交回；每格单线程，同时跑的格数由 jobs 与内存包装的总上限定。"""
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [pool.submit(run_one_cell, binary, cell, peaks_path, cap, runner) for cell in cells]
        results = [future.result() for future in futures]
    lines = [line for result_lines, _ in results for line in result_lines]
    return lines, sum(1 for _, lost in results if lost)


# ---- 停机条款 S1：实现还是登记时那一份 ----

def sha256_of_lines(lines, start, end):
    return hashlib.sha256(("\n".join(lines[start - 1:end]) + "\n").encode("utf-8")).hexdigest()


def implementation_line(root, entry):
    """一份文件的 S1 核对行：同一段行号上 sha256 相同记 same；挪了行号而函数正文没变（按函数签名找到新位置、取同样长的一段）记 moved_same；否则 changed。"""
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


# ---- 段的收尾行、续跑 ----

def segment_done_line(segment, cells):
    return f"E7RESULT name=e197_segment_done segment={segment} cells={cells}"


def segment_is_done(artifact_path, segment):
    if not Path(artifact_path).exists():
        return False
    return any(line.startswith(f"E7RESULT name=e197_segment_done segment={segment} ")
               for line in Path(artifact_path).read_text(encoding="utf-8").splitlines())


def registered_heavy_threads():
    """配置行记的是每台默认重活线程数的登记值（不是本机核数）；取不了就带出路退出，不退回按核数算。"""
    try:
        return heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        sys.exit(f"取不了每台默认重活线程数：{error}（→ 怎么办：{error.howto}）")


def config_line(segment, binary, jobs, cap):
    digest = hashlib.sha256(Path(binary).read_bytes()).hexdigest() if Path(binary).exists() else "none"
    return (f"E7RESULT name=e197_config segment={segment} binary_sha256={digest} cell_jobs={jobs} threads_per_cell=1 "
            f"cell_memory_cap={cap} registered_heavy_threads={registered_heavy_threads()} order_of_segments=checks,compare,k64-full,k64-half,k1-full")


def append_lines(artifact_path, lines):
    """一块判完才落盘：整块一次写进去并刷盘；产物只追加，不覆盖已有内容。"""
    with open(artifact_path, "a", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def run_segment(segment, binary, artifact_path, peaks_path, jobs, cap, runner=subprocess_runner):
    if segment_is_done(artifact_path, segment) and not broken("segment-not-skipped"):
        print(f"段 {segment} 已有收尾行，跳过")
        return 0
    cells = cells_of(segment)
    lines, lost = run_cells(binary, cells, peaks_path, cap, jobs, runner)
    append_lines(artifact_path, [config_line(segment, binary, jobs, cap)] + lines + [segment_done_line(segment, len(cells))])
    print(f"段 {segment}：{len(cells)} 格，丢了 {lost} 格")
    return 1 if lost else 0


def run_checks(binary, artifact_path, root=REPOSITORY_ROOT, runner=subprocess_runner):
    """S1 先核，实现变了就停（退 3，不跑后面）；没变再起装置的 --checks 与 --diff-set（各经内存包装一次）。"""
    if segment_is_done(artifact_path, "checks"):
        print("段 checks 已有收尾行，跳过")
        return 0
    s1 = implementation_lines(root)
    if not implementation_is_the_registered_one(s1):
        append_lines(artifact_path, [config_line("checks", binary, 1, DEFAULT_CAP)] + s1)
        print("停机条款 S1：实现不是登记时那一份，停；第三节与 baseline_pc、split 的抄本要按新实现重写", file=sys.stderr)
        return 3
    lines = [config_line("checks", binary, 1, DEFAULT_CAP)] + s1
    for mode in ("--checks", "--diff-set"):
        environment = dict(os.environ, RUN_WITH_MEMORY_CAP_HOST_LOOKUP="off")
        exit_code, standard_output, _ = runner(["bash", WRAPPER, DEFAULT_CAP, str(binary), mode], environment)
        lines += result_lines_of(standard_output)
        lines.append(f"E7RESULT name=e197_run mode={mode} exit={exit_code}")
    lines.append(segment_done_line("checks", 2))
    append_lines(artifact_path, lines)
    return 0


def run_rerun_unstable(binary, artifact_path, peaks_path, jobs, cap, runner=subprocess_runner):
    """report.py --unstable 点名的格：第一轮 5 次散布超 2%，再跑重复号 6–10 的一轮（登记第十一节 V2）。"""
    artifact = report.Artifact(Path(artifact_path).read_text(encoding="utf-8").splitlines())
    cells = []
    for arm, devices, fill, named in report.cells_needing_a_rerun(artifact):
        cells += [Cell(arm, devices, fill, named, repeat) for repeat in range(6, 11)]
    if not cells:
        print("没有需要重跑的格")
        return 0
    lines, lost = run_cells(binary, cells, peaks_path, cap, jobs, runner)
    append_lines(artifact_path, [config_line("rerun-unstable", binary, jobs, cap)] + lines
                 + [segment_done_line("rerun-unstable", len(cells))])
    print(f"重跑 {len(cells)} 格，丢了 {lost} 格")
    return 1 if lost else 0


def option(arguments, name, default=None):
    if name in arguments:
        index = arguments.index(name)
        return arguments[index + 1] if index + 1 < len(arguments) else None
    return default


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["--selftest-breaks"]:
        return selftest_breaks()
    if len(arguments) >= 2 and arguments[0] == "list-cells" and (arguments[1] in SEGMENTS or arguments[1] == COMPARE_SEGMENT):
        for cell in cells_of(arguments[1]):
            print(cell.key())
        return 0
    binary, artifact_path, peaks_path = option(arguments, "--binary"), option(arguments, "--out"), option(arguments, "--peaks")
    jobs, cap = int(option(arguments, "--jobs", "5")), option(arguments, "--cap", DEFAULT_CAP)
    if arguments[:1] == ["checks"] and binary and artifact_path:
        return run_checks(binary, artifact_path)
    if len(arguments) >= 2 and arguments[0] == "segment" and (arguments[1] in SEGMENTS or arguments[1] == COMPARE_SEGMENT) and binary and artifact_path and peaks_path:
        return run_segment(arguments[1], binary, artifact_path, peaks_path, jobs, cap)
    if arguments[:1] == ["rerun-unstable"] and binary and artifact_path and peaks_path:
        return run_rerun_unstable(binary, artifact_path, peaks_path, jobs, cap)
    print(__doc__, file=sys.stderr)
    return 2


# ---- 自证：假的包装与假的装置二进制 ----

def fake_runner_factory(exits=None, write_peaks=True, unstable_cell=None):
    """假的 runner：按命令里的参数造一行装置结果行（用 report 的合成产物同一份造法），把 cgroup 峰值写进环境里 RUN_WITH_MEMORY_CAP_PEAKS 指的表。
    exits 是按调用次序给的退出码（用完的都退 0）；记下每次调用的 (命令, 键)。"""
    calls = []
    scripted = list(exits or [])

    def runner(command, environment):
        calls.append((command, environment.get("RUN_WITH_MEMORY_CAP_KEY")))
        code = scripted.pop(0) if scripted else 0
        if code != 0:
            return code, "", ""
        words = command[command.index("--arm"):] if "--arm" in command else None
        if words is None:
            return 0, "E7RESULT name=e197_checks fake=yes\n", ""
        if words[-1] == "--compare":
            values = dict(zip(words[:-1:2], words[1:-1:2]))
            return 0, (f"E7RESULT name=e197_compare arm={values['--arm']} devices={values['--devices']} fill={values['--fill']} "
                       f"named={values['--named']} equal=true records=1\n"), ""
        values = dict(zip(words[::2], words[1::2]))
        arm, devices, fill, named, repeat = values["--arm"], int(values["--devices"]), values["--fill"], int(values["--named"]), int(values["--repeat"])
        filled = report.RING_SLOTS if fill == "full" else report.RING_SLOTS // 2
        allocator = int(report.predicted_allocator_peak(arm, devices, filled, named, report.SYNTHETIC_SIZES))
        if arm == "measurement_pc":
            allocator = report.MEASUREMENT_BYTES + 4096
        cgroup = (report.SYNTHETIC_EMPTY_BYTES + allocator
                  + (report.SYNTHETIC_OVERHEAD_BYTES if arm in report.SCANNING_ARMS else 0) + 1000 * repeat)
        if unstable_cell == (arm, devices, fill, named) and repeat == 5:
            cgroup += 100 * report.MEBIBYTE
        if write_peaks:
            with open(environment["RUN_WITH_MEMORY_CAP_PEAKS"], "a", encoding="utf-8") as handle:
                handle.write(f"{cgroup}\t2026-01-01T00:00:00Z\t{command[2]}\t{environment['RUN_WITH_MEMORY_CAP_KEY']}\n")   # clock-times:allow 自证造的假峰值表行，时间戳照内存包装写的格式
        return 0, report.synthetic_line(arm, devices, fill, named, repeat, allocator) + "\n", ""

    runner.calls = calls
    return runner


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def case_cell_counts_match_the_registration():
    counts = {segment: len(cells_of(segment)) for segment in SEGMENTS}
    check(counts == {"k64-full": 105, "k64-half": 100, "k1-full": 100}, counts)
    compare = cells_of(COMPARE_SEGMENT)
    check(len(compare) == 36 and len({cell.key() for cell in compare}) == 36 and all(cell.compare and cell.arm in COMPARE_ARMS for cell in compare), len(compare))
    check(compare[0].arguments()[-1] == "--compare" and compare[0].key().startswith("e197/compare/"), compare[0].key())
    every = [cell.key() for segment in SEGMENTS for cell in cells_of(segment)]
    check(len(every) == 305 and len(set(every)) == 305, len(every))
    scanning = [key for key in every if key.split("/")[1] in report.SCANNING_ARMS]
    check(len(scanning) == 240, len(scanning))
    check(cells_of("k64-full")[0].arm == "empty" and sum(1 for key in every if "/measurement_pc/" in key) == 5, "空跑格排前、measurement_pc 5 格")


def case_command_and_environment():
    cell = Cell("split", 3, "half", 64, 2)
    check(command_of("/b/e197", cell, "8G") == ["bash", WRAPPER, "8G", "/b/e197", "--arm", "split", "--devices", "3", "--fill", "half",
                                                "--named", "64", "--repeat", "2"], command_of("/b/e197", cell, "8G"))
    environment = environment_of(cell, "/p/peaks.tsv")
    check(environment["RUN_WITH_MEMORY_CAP_KEY"] == "e197/split/3/half/64/2" and environment["RUN_WITH_MEMORY_CAP_PEAKS"] == "/p/peaks.tsv"
          and environment["RUN_WITH_MEMORY_CAP_HOST_LOOKUP"] == "off", environment)


def case_a_cap_hit_is_rerun_at_twelve_gibibytes():
    with tempfile.TemporaryDirectory() as directory:
        peaks = Path(directory, "peaks.tsv")
        runner = fake_runner_factory(exits=[250])
        lines, lost = run_one_cell("/b/e197", Cell("baseline", 1, "full", 64, 1), peaks, "8G", runner)
        check(not lost and lines[-1].startswith("E7RESULT name=e197_cgroup ") and "cap=12G attempts=2" in lines[-1], lines)
        check(runner.calls[0][0][2] == "8G" and runner.calls[1][0][2] == "12G", runner.calls)


def case_a_cell_that_keeps_failing_is_reported_lost_not_zero():
    with tempfile.TemporaryDirectory() as directory:
        peaks = Path(directory, "peaks.tsv")
        lines, lost = run_one_cell("/b/e197", Cell("baseline", 1, "full", 64, 1), peaks, "8G", fake_runner_factory(exits=[252, 252, 252]))
        check(lost and len(lines) == 1 and "name=e197_cell_lost" in lines[0] and "attempts=3" in lines[0], lines)
        lines, lost = run_one_cell("/b/e197", Cell("baseline", 1, "full", 64, 1), peaks, "8G", fake_runner_factory(exits=[1]))
        check(lost and "attempts=1" in lines[0], lines)


def case_a_missing_peak_row_is_a_failed_attempt():
    with tempfile.TemporaryDirectory() as directory:
        peaks = Path(directory, "peaks.tsv")
        lines, lost = run_one_cell("/b/e197", Cell("baseline", 1, "full", 64, 1), peaks, "8G", fake_runner_factory(write_peaks=False))
        check(lost and "name=e197_cell_lost" in lines[0] and "attempts=3" in lines[0], lines)


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


def case_the_comparison_segment_writes_one_compare_line_per_cell_and_no_cgroup_line():
    with tempfile.TemporaryDirectory() as directory:
        artifact, peaks = Path(directory, "a.out"), Path(directory, "peaks.tsv")
        binary = Path(directory, "bin")
        binary.write_bytes(b"x")
        check(run_segment(COMPARE_SEGMENT, binary, artifact, peaks, 4, "8G", fake_runner_factory()) == 0, "compare 段")
        text = artifact.read_text(encoding="utf-8")
        check(text.count("name=e197_compare ") == 36 and "name=e197_cgroup" not in text, "36 行 compare、没有 cgroup 行")
        check(len(report.Artifact(text.splitlines()).compare) == 36, "report 读得到 36 行")
        lines, lost = run_cells(binary, cells_of(COMPARE_SEGMENT)[:2], peaks, "8G", 1, fake_runner_factory(exits=[1, 1, 1]))
        check(lost == 2 and sum(1 for line in lines if "name=e197_cell_lost" in line) == 2, "compare 格失败（退 1 不重试）要报丢了")


def case_the_registered_implementation_ranges_hold_today():
    lines = implementation_lines(REPOSITORY_ROOT)
    check(implementation_is_the_registered_one(lines), lines)


def case_a_finished_segment_is_skipped_on_the_second_call():
    with tempfile.TemporaryDirectory() as directory:
        artifact, peaks = Path(directory, "a.out"), Path(directory, "peaks.tsv")
        binary = Path(directory, "bin")
        binary.write_bytes(b"x")
        runner = fake_runner_factory()
        check(run_segment("k1-full", binary, artifact, peaks, 3, "8G", runner) == 0, "第一次")
        size = artifact.stat().st_size
        calls = len(runner.calls)
        check(run_segment("k1-full", binary, artifact, peaks, 3, "8G", runner) == 0 and artifact.stat().st_size == size and len(runner.calls) == calls, "第二次要跳过")
        text = artifact.read_text(encoding="utf-8")
        check("name=e197_config segment=k1-full" in text and "cell_jobs=3 threads_per_cell=1" in text, "配置行报每格线程数与并行格数")
        check(text.count("name=e197 ") == 100 and text.count("name=e197_cgroup ") == 100 and "name=e197_segment_done segment=k1-full cells=100" in text, "格数")
        parsed = report.Artifact(text.splitlines())
        for arm in ("empty",) + report.SCANNING_ARMS:
            for devices in report.DEVICE_COUNTS:
                check(report.cell_value(parsed, arm, devices, "full", 1) is not None, f"report 读不到 {arm} {devices}")


def case_rerun_unstable_runs_repeats_six_to_ten():
    with tempfile.TemporaryDirectory() as directory:
        artifact, peaks = Path(directory, "a.out"), Path(directory, "peaks.tsv")
        binary = Path(directory, "bin")
        binary.write_bytes(b"x")
        run_segment("k1-full", binary, artifact, peaks, 3, "8G", fake_runner_factory(unstable_cell=("baseline", 2, "full", 1)))
        runner = fake_runner_factory()
        check(run_rerun_unstable(binary, artifact, peaks, 3, "8G", runner) == 0, "重跑")
        keys = [key for _, key in runner.calls]
        check(keys == [f"e197/baseline/2/full/1/{repeat}" for repeat in range(6, 11)], keys)
        check(run_rerun_unstable(binary, artifact, peaks, 3, "8G", fake_runner_factory()) == 0, "第二轮稳定后不再有要重跑的格")


SELFTEST_CASES = (
    ("cell-counts-match-the-registration", case_cell_counts_match_the_registration),
    ("command-and-environment", case_command_and_environment),
    ("a-cap-hit-is-rerun-at-twelve-gibibytes", case_a_cap_hit_is_rerun_at_twelve_gibibytes),
    ("a-cell-that-keeps-failing-is-reported-lost-not-zero", case_a_cell_that_keeps_failing_is_reported_lost_not_zero),
    ("a-missing-peak-row-is-a-failed-attempt", case_a_missing_peak_row_is_a_failed_attempt),
    ("s1-tells-a-moved-function-from-a-changed-one", case_s1_tells_a_moved_function_from_a_changed_one),
    ("the-comparison-segment-writes-one-compare-line-per-cell-and-no-cgroup-line", case_the_comparison_segment_writes_one_compare_line_per_cell_and_no_cgroup_line),
    ("the-registered-implementation-ranges-hold-today", case_the_registered_implementation_ranges_hold_today),
    ("a-finished-segment-is-skipped-on-the-second-call", case_a_finished_segment_is_skipped_on_the_second_call),
    ("rerun-unstable-runs-repeats-six-to-ten", case_rerun_unstable_runs_repeats_six_to_ten),
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
    return 0


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


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
