#!/usr/bin/env python3
# admission: always 每一次调都读此刻点名的产物与状态存档现判，上一次的结论不替这一次作保
# run-condition: none 只读产物文本与状态存档，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置、不起 task-server/task
"""E197（扫环胶水内存随盘数）出表脚本：读驱动 drive.py 写的各组产物与主 agent 存的服务状态存档，按跑前登记
research/prompts/e197-preregistration.md 第六、七、八、十、十一节与第十二节修订一、修订三逐行判，每行一条 `E7RESULT name=verdict ...`。
独立手写，不 import crates/ 任何东西；门槛跑前写死在下面的常量里。

峰值：每条装置结果行带 task= attempt=（服务给那一片的任务号与第几次），去状态存档（`task-server/task status <父任务号> --json` 的原样输出，
children 里每个子任务的 attempts）里找那一次尝试，读 memory_peak、memory_cap、host、outcome。读不到、对不上的那一次作废（V7），不当 0。
判的数 P：一格每次尝试 d_i = memory_peak_i − E(台_i, 几何)，E 是同台、同（盘数, 填满档, 点名项数）空跑格的中位数（第六节修订第 5 条），P = 5 次 d_i 的中位数。
进程内的常驻页峰值 H（登记修订三）只用来互核，不判：装置在量窗开头往 /proc/self/clear_refs 写 5、读 VmRSS 当起点，交回之后读 VmHWM，
结果行的 resident_peak_over_start_bytes 就是 H；写不进或读不出的那一次，结果行那几个字段写 unreadable，这里按 V11 作废（voided 原因 v11_window_unreadable），不当 0。

用法：
  report.py --artifacts <产物.out…> --status <状态存档.json…>                 打到标准输出（读不动的格列出来、不当 0）
  report.py --artifacts … --status … --out <判决.out>                          同样的行写进一份新文件（排他新建，已在就拒），末行 `E7RESULT name=done emitted=N`
  report.py --artifacts … --status … --unstable                                只列需要重跑一轮 5 次的格，一格一行 `arm devices fill named`
  report.py --selftest                 合成产物与合成状态存档上的自证（每一项标签就是下面「弄坏开关」点名要红的那一格）：各项过了之后接着逐个打开弄坏开关
                                       起自己，要求每个开关都红在点名的那一项（开关生效的那一趟只跑各项）
退出码：0 判完；1 自证红；2 用法、产物或状态存档读不了、判决文件已在；3 产物里装置版本不一致（整份不判）。

弄坏开关（只给自证用，生效时往 stderr 打一行；环境变量 E197_REPORT_BREAK=<一个值>，不认识的值直接拒绝）：
  E197_REPORT_BREAK=line-ignores-the-sign       甲-直线的偏离恒取 0
      自证项 「✗ 自检 line-flags-a-bent-line：」
  E197_REPORT_BREAK=pc-red-threshold-ignored    阳性对照红不红不看 1.10 的门槛，恒报红
      自证项 「✗ 自检 pc-not-red-where-it-is-not：」
  E197_REPORT_BREAK=slope-uses-the-top-point-only  乙-不涨的斜率只用 4 盘那一点除以 2 倍单盘（不减 2 盘那一点）
      自证项 「✗ 自检 split-slope-and-its-control：」
      「✗ 自检 split-rows-and-f2：」
  E197_REPORT_BREAK=median-replaced-by-minimum  一格的 d_i 取最小值不取中位数
      自证项 「✗ 自检 median-of-five-not-minimum：」
  E197_REPORT_BREAK=empty-cell-not-subtracted   不扣空跑格
      自证项 「✗ 自检 empty-cell-is-subtracted：」
      「✗ 自检 ratio-rows-tell-a-varying-ratio-from-a-steady-one：」
  E197_REPORT_BREAK=digest-mismatch-ignored     交回的表摘要对不上也报在原型上做得到
      自证项 「✗ 自检 digest-mismatch-is-caught：」
  E197_REPORT_BREAK=cap-distance-ignored        峰值顶着上限（≥ 上限 − 64 MiB）的那一次照收（V6 不判）
      自证项 「✗ 自检 an-attempt-near-its-cap-is-not-used：」
  E197_REPORT_BREAK=attempt-not-pinned          结局不是 succeeded、memory_peak 为空的那一次照收，空的当 0（V7 不判）
      自证项 「✗ 自检 a-result-line-is-pinned-to-a-succeeded-attempt：」
  E197_REPORT_BREAK=empty-cell-from-any-host    空跑格不分台，两台的混在一起取中位数（V8 不判）
      自证项 「✗ 自检 empty-cell-is-subtracted-per-host：」
  E197_REPORT_BREAK=binary-digest-ignored       二进制的 sha256 与 checks 记的不同也照收（V9 不判）
      自证项 「✗ 自检 a-run-from-another-binary-is-not-used：」
  E197_REPORT_BREAK=interval-ignored            丙各行只按中位数判，区间跨门槛也不报分不出
      自证项 「✗ 自检 bing-rows-report-indistinguishable-across-the-threshold：」
  E197_REPORT_BREAK=unreadable-window-read-as-zero   量窗写不进、读不出（unreadable）的那一次照收，读不出的数当 0（V11 不判）
      自证项 「✗ 自检 an-unreadable-window-voids-the-run-and-is-not-read-as-zero：」
"""
import json
import os
import re
import statistics
import subprocess
import sys
from itertools import product

MEBIBYTE = 1 << 20
GIBIBYTE = 1 << 30


def glibc_chunk(request):
    """glibc 一块的大小：请求字节加 8 字节块头，向上取到 16 的倍数，最小 32（登记修订三第四节式子；推的，没量）。"""
    return max(32, (request + 8 + 15) & ~15)


# ---- 跑前写死的门槛（登记第六、七、十、十一节与第十二节修订一、修订三）----
LINE_LIMIT = 0.05            # 甲-直线、丙-直线 |dev| ≤ 5%
RATIO_LIMIT = 0.15           # 甲-槽填满、甲-点名项、丙-槽填满、丙-点名项 ≤ 0.15
PC_RED_FACTOR = 1.10         # 阳性对照红 ⟺ 比基线高过 10%
CLOSE_TO_ONE_LIMIT = 1.25    # 乙-接近一 r_split(4) ≤ 1.25
SLOPE_LIMIT = 0.10           # 乙-不涨 slope ≤ 0.10
SPREAD_LIMIT = 0.02          # V2 一格 5 次 (max d − min d) ÷ 这 5 次 memory_peak 的中位数 ≤ 2%
EMPTY_CELL_LIMIT = 128 * MEBIBYTE   # V3 空跑格峰值
CAP_DISTANCE = 64 * MEBIBYTE        # V6 memory_peak ≥ memory_cap − 64 MiB 不算
EMPTY_HOSTS_LIMIT = 8 * MEBIBYTE    # V8 台上没有这一几何的空跑格时，各几何空跑格中位数之间相差 ≤ 8 MiB 才取全台的中位数
PREDICTION_TOLERANCE = 0.05  # A7：H 对常驻页推算 R（修订三第七节）
TRAJECTORY_TOLERANCE = 0.03  # 修订三第八节：最后一盘的轨迹值对「读到那一刻的常驻页」
SAME_ONE_DEVICE_RESIDENT_LIMIT = 0.01    # A9 H 之差（修订三第七节）
SAME_ONE_DEVICE_CGROUP_LIMIT = 0.03      # A9 P 之差
TWO_MEASURES_ABSOLUTE_SLACK = 32 * MEBIBYTE         # S3 H > P + 32 MiB（修订三第十一节，门槛照抄原 S3）
TWO_MEASURES_RELATIVE = (1.15, 64 * MEBIBYTE)       # S3 P > 1.15 × H + 64 MiB
CLAUSE_PAIR_BYTES = (1.86e9, 3.63e9)                # C2 条款推算（两盘、四盘）
CLAUSE_TOLERANCE = 0.25
F4_RATIO_RANGES = {2: (1.6, 2.2), 4: (3.0, 4.4)}   # F4，修订一 F8（C4）同一组区间
C3_RANGE = (1_207_959_552, 2_013_265_920)          # 修订一 C3、F7：C881 正文「两块盘约 1.5 GiB」±25%
CACHE_ENTRY_BYTES = glibc_chunk(4096) + glibc_chunk(16 + 11 * 24 + 11 * 24) / 6   # 修订三第七节 A11、A12：每项一块 chunk(4096) 的值 + 叶（544 字节的请求，一块 560）按每叶 6 键的摊分 = 4112 + 560 ÷ 6
CACHE_TOLERANCE = 0.05                              # A11、A12 ±5%
MEASUREMENT_BYTES = 256 * MEBIBYTE
MEASUREMENT_RESIDENT_WINDOW = (254 * MEBIBYTE, 260 * MEBIBYTE)   # 修订三第七节 A8、第十节 F5：measurement_pc 的 H 落在 [266 338 304, 272 629 760] 字节
MEASUREMENT_CGROUP_HIGH = 288 * MEBIBYTE
RESIDENT_NUMBER_FIELDS = ("resident_peak_over_start_bytes", "resident_after_return_bytes", "vm_rss_at_start_kib", "vm_hwm_kib")   # 修订三结果行字段（V11 要它们读得出）
TRAJECTORY_FIELD = "resident_at_each_device_end_bytes"
REPEATS_PER_ROUND = 5
DIFFERENTIAL_LINES = 20      # 差分集四例 × 五份胶水臂
COMPARE_LINES = 52           # 原三份胶水臂 36 格 + read_stage_cache 12 格 + 阳性对照 4 格
RING_SLOTS = 196608
DEVICE_COUNTS = (1, 2, 3, 4)
GLUE_ARMS = ("baseline", "baseline_pc", "split", "split_pc")
READ_STAGE_CACHE_ARMS = ("read_stage_cache", "read_stage_cache_keeps_zeros_pc")
SCANNING_ARMS = GLUE_ARMS + READ_STAGE_CACHE_ARMS
MEASURED_ARMS = ("empty",) + SCANNING_ARMS + ("measurement_pc",)
GEOMETRIES = {"k64_full": (64, "full"), "k64_half": (64, "half"), "k1_full": (1, "full")}
SUCCEEDED = "succeeded"
BREAK_SWITCHES = (
    "line-ignores-the-sign", "pc-red-threshold-ignored", "slope-uses-the-top-point-only",
    "median-replaced-by-minimum", "empty-cell-not-subtracted", "digest-mismatch-ignored",
    "cap-distance-ignored", "attempt-not-pinned", "empty-cell-from-any-host", "binary-digest-ignored", "interval-ignored",
    "unreadable-window-read-as-zero",
)
TOKEN = re.compile(r"(\w+)=(\S+)")


def broken(name):
    """弄坏开关生效了吗；不认识的值直接拒绝（免得拼错的开关被当成没开）。"""
    value = os.environ.get("E197_REPORT_BREAK", "")
    if value and value not in BREAK_SWITCHES:
        sys.exit(f"不认识的 E197_REPORT_BREAK 值：{value}")
    return value == name


def parse_line(line):
    """`E7RESULT name=… k=v …` → 字典；不是产物行交 None。值里没有空格。"""
    if not line.startswith("E7RESULT "):
        return None
    return dict(TOKEN.findall(line[len("E7RESULT "):]))


def predicted_resident_peak(arm, devices, filled, named, sizes):
    """登记修订三第七节 A7 的常驻页式子 R（model/resident_prediction.py 的 resident），按这一行里量到的 size_of 代入；与装置里的 predicted_resident_peak_bytes 同一份算术。
    记录 Vec、键表只算碰过的页（len × 元素大小，不算 2 的幂容量）；每条记录的点名项堆算一块 chunk(K × 点名项大小)；两张 B 树按每叶 6 个键摊（叶各算一块 chunk）。
    经读缓存的两臂交的是胶水那一截（同基线），缓存那一截另由 A11 算（cache_bytes）。"""
    record, unit = sizes["journal_record"], sizes["named_unit"]
    named_heap = glibc_chunk(named * unit)
    trees = filled * (glibc_chunk(16 + 11 * 16 + 11 * record) + glibc_chunk(16 + 11 * 16 + 11 * 8)) / 6

    def base(count):
        return count * filled * (record + named_heap + 16) + trees

    if arm in ("baseline", "split_pc") + READ_STAGE_CACHE_ARMS:
        return base(devices)
    if arm == "baseline_pc":
        return base(devices) + filled * named_heap
    if arm == "split":
        if devices == 1:
            return base(1)
        return trees + 2 * filled * named_heap + filled * record + devices * filled * 16
    return 0.0


def expected_last_device_end(devices, filled, fill, ring_slots, named, sizes):
    """登记修订三第八节：第 devices 块盘最后一槽被读到那一刻的常驻页 = 读到的记录数 × (记录 + 点名项块) + 环槽数 × 8；
    读到的记录数 = 盘数 × 每盘填满槽数，满槽时减 1（最后一槽读到时这一槽还没解）。"""
    read = devices * filled - (1 if fill == "full" else 0)
    return read * (sizes["journal_record"] + glibc_chunk(named * sizes["named_unit"])) + ring_slots * 8


def expected_cache_entries(arm, devices, filled):
    """A10：read_stage_cache 只收非 0 的槽，盘数 × 每盘填满槽数；阳性对照全收，盘数 × 环槽数；别的臂没有缓存。"""
    if arm == "read_stage_cache":
        return devices * filled
    if arm == "read_stage_cache_keeps_zeros_pc":
        return devices * RING_SLOTS
    return None


def cache_bytes(entries):
    return entries * CACHE_ENTRY_BYTES


def arms_of(geometry):
    """这一几何上跑的扫环臂：阳性对照 read_stage_cache_keeps_zeros_pc 只在 K = 64 半满上跑（登记修订一第五节）。"""
    return GLUE_ARMS + (READ_STAGE_CACHE_ARMS if geometry == "k64_half" else ("read_stage_cache",))


def median(values):
    return min(values) if broken("median-replaced-by-minimum") else statistics.median(values)


# ---- 状态存档：task-server/task status <父任务号> --json 的原样输出 ----

def attempts_of(summaries):
    """{(任务号, 第几次): 那一次尝试的记录}：父任务与 children 里每个子任务的 attempts。"""
    found, stack = {}, list(summaries)
    while stack:
        summary = stack.pop()
        if not isinstance(summary, dict):
            continue
        task_id = (summary.get("task") or {}).get("id")
        for attempt in summary.get("attempts") or []:
            if isinstance(attempt, dict):
                found[(str(attempt.get("task_id", task_id)), str(attempt.get("number")))] = attempt
        stack.extend(summary.get("children") or [])
    return found


def is_count(value):
    return isinstance(value, int) and not isinstance(value, bool)


def is_integer(text):
    return text is not None and re.fullmatch(r"-?\d+", text) is not None


def window_is_readable(fields):
    """V11（登记修订三第十一节）：量窗的重置成功（high_water_mark_reset=ok），H、期末值、起点、终点与轨迹都读出来了（读不出装置写 unreadable）。
    弄坏开关 unreadable-window-read-as-zero 下不查，读不出的照收。"""
    if broken("unreadable-window-read-as-zero"):
        return True
    trajectory = fields.get(TRAJECTORY_FIELD, "")
    return (fields.get("high_water_mark_reset") == "ok"
            and all(is_integer(fields.get(name)) for name in RESIDENT_NUMBER_FIELDS)
            and (trajectory == "none" or all(is_integer(word) for word in trajectory.split(","))))


def window_number(text):
    """量窗的一个读数；V11 查过之后一定是整数。弄坏开关下读不出的当 0（弄坏开关要证明的就是「读不到 ≠ 读到 0」不能被破）。"""
    return int(text) if is_integer(text) else 0


# ---- 产物模型 ----

class Artifact:
    """各组产物的行合在一起：每条装置结果行（name=e197）按 task=、attempt= 钉到状态存档里的一次尝试；差分集、比较、检查、S1/S4 核对各按行收着。"""

    def __init__(self, lines, summaries):
        self.runs, self.peaks, self.voided = {}, {}, {}
        self.diff, self.compare, self.checks, self.s1, self.configs, self.lost = [], [], [], [], [], []
        self.sizes_seen = set()
        claims, duplicates = {}, set()
        for line in lines:
            fields = parse_line(line)
            if fields is None:
                continue
            name = fields.get("name")
            if name == "e197":
                key = (fields["arm"], int(fields["devices"]), fields["fill"], int(fields["named"]), int(fields["repeat"]))
                if key in self.runs:
                    duplicates.add(key)
                self.runs[key] = fields
                claims.setdefault((fields.get("task"), fields.get("attempt")), []).append(key)
                self.sizes_seen.add(tuple(fields.get(f"size_{field}") for field in
                                          ("location_entry", "named_unit", "node_pointer", "journal_record")))
            elif name == "e197_diff":
                self.diff.append(fields)
            elif name == "e197_compare":
                self.compare.append(fields)
            elif name in ("e197_checks", "e197_sizes"):
                self.checks.append(fields)
            elif name == "e197_s1":
                self.s1.append(fields)
            elif name == "e197_config":
                self.configs.append(fields)
            elif name == "e197_cell_lost":
                self.lost.append(fields)
        self.checks_binary = next((fields.get("binary_sha256") for fields in self.configs if fields.get("step") == "checks"), None)
        attempts = attempts_of(summaries)
        for key, fields in self.runs.items():
            reason, reading = self.pinned_reading(key, fields, attempts, claims, duplicates)
            if reason:
                self.voided[key] = reason
            else:
                self.peaks[key] = reading

    def pinned_reading(self, key, fields, attempts, claims, duplicates):
        """(作废原因, (memory_peak, 台))：V7 钉不到服务记录、V6 顶着上限、V9 二进制不同、V11 量窗读不出、同一格两行，都交原因、不当 0。"""
        task, attempt = fields.get("task"), fields.get("attempt")
        if key in duplicates:
            return "duplicate_cell", None
        if not task or not attempt:
            return "v7_not_pinned", None
        if len(claims[(task, attempt)]) > 1:
            return "v7_attempt_claimed_twice", None
        record = attempts.get((task, attempt))
        if record is None:
            return "v7_attempt_not_in_status", None
        peak, cap = record.get("memory_peak"), record.get("memory_cap")
        if broken("attempt-not-pinned"):
            peak = peak if is_count(peak) else 0
        elif record.get("outcome") != SUCCEEDED:
            return f"v7_outcome_{record.get('outcome') or 'none'}", None
        elif not is_count(peak):
            return "v7_memory_peak_missing", None
        if not broken("cap-distance-ignored") and (not is_count(cap) or peak >= cap - CAP_DISTANCE):
            return "v6_near_the_cap", None
        if not broken("binary-digest-ignored") and (self.checks_binary is None or fields.get("binary_sha256") != self.checks_binary):
            return "v9_binary", None
        if not window_is_readable(fields):
            return "v11_window_unreadable", None
        return "", (peak, str(record.get("host") or ""))


def load(artifact_paths, status_paths):
    lines = []
    for path in artifact_paths:
        with open(path, encoding="utf-8") as handle:
            lines += handle.read().splitlines()
    summaries = []
    for path in status_paths:
        with open(path, encoding="utf-8") as handle:
            summaries.append(json.load(handle))
    return Artifact(lines, summaries)


def sizes_of(fields):
    return {name: int(fields[f"size_{name}"]) for name in ("location_entry", "named_unit", "node_pointer", "journal_record")}


def run_is_complete(artifact, key):
    """V4、V5、V6、V7、V9、V11：结果行在、字段齐、钉得到服务记录、量窗读得出；扫环格的 A2、A3 对得上；读缓存两臂的 A10 对得上；生产格的环槽数是 196 608（A1）。读不到的不当 0。"""
    fields = artifact.runs.get(key)
    if fields is None or key not in artifact.peaks:
        return False
    needed = ("ring_slots", "filled_slots", "reads", "reads_expected", "records_returned", "records_expected",
              "resident_peak_over_start_bytes", TRAJECTORY_FIELD, "resident_after_return_bytes", "vm_rss_at_start_kib", "vm_hwm_kib",
              "high_water_mark_reset", "table_digest", "cache_entries", "cache_entries_expected", "cache_value_bytes")
    if any(name not in fields for name in needed):
        return False
    if int(fields["ring_slots"]) != RING_SLOTS:
        return False
    if fields["reads"] != fields["reads_expected"] or fields["records_returned"] != fields["records_expected"]:
        return False
    expected = expected_cache_entries(key[0], key[1], int(fields["filled_slots"]))
    if expected is None:
        return fields["cache_entries"] == "none"
    return (fields["cache_entries"] == fields["cache_entries_expected"] == str(expected)
            and fields["cache_value_bytes"] == str(expected * 4096))


def round_keys(arm, devices, fill, named, round_number):
    first = REPEATS_PER_ROUND * (round_number - 1) + 1
    return [(arm, devices, fill, named, repeat) for repeat in range(first, first + REPEATS_PER_ROUND)]


def chosen_round(artifact, arm, devices, fill, named):
    """最近一轮完整的 5 次：复测轮（重复号 6–10）齐了用复测轮，否则初测轮（1–5）；都不齐交 (None, None)（本次未跑）。"""
    for round_number in (2, 1):
        keys = round_keys(arm, devices, fill, named, round_number)
        if all(run_is_complete(artifact, key) for key in keys):
            return round_number, keys
    return None, None


def empty_peaks_by_host(artifact):
    """{台: {(盘数, 填满档, 点名项数): [空跑格选用那一轮在这一台上的 memory_peak…]}}，算一次存在 artifact 上。"""
    if getattr(artifact, "empty_peaks", None) is None:
        found = {}
        for named, fill in GEOMETRIES.values():
            for devices in DEVICE_COUNTS:
                _, keys = chosen_round(artifact, "empty", devices, fill, named)
                for key in keys or []:
                    peak, host = artifact.peaks[key]
                    host = "every_host" if broken("empty-cell-from-any-host") else host
                    found.setdefault(host, {}).setdefault((devices, fill, named), []).append(peak)
        artifact.empty_peaks = found
    return artifact.empty_peaks


def empty_baseline(empty_peaks, host, geometry):
    """E(台, 几何)（第六节修订第 5 条）；台上没有这一几何的空跑格：台上各几何空跑格中位数之间相差 ≤ 8 MiB 时取全台的中位数，否则 None（V8）。"""
    if broken("empty-cell-not-subtracted"):
        return 0
    on_host = empty_peaks.get("every_host" if broken("empty-cell-from-any-host") else host, {})
    if geometry in on_host:
        return statistics.median(on_host[geometry])
    medians = [statistics.median(peaks) for peaks in on_host.values()]
    if medians and max(medians) - min(medians) <= EMPTY_HOSTS_LIMIT:
        return statistics.median([peak for peaks in on_host.values() for peak in peaks])
    return None


class CellValue:
    """一格（臂、盘数、填满档、点名项数）选用那一轮 5 次的读数：d_i = memory_peak_i − E(台_i, 几何)。"""

    def __init__(self, keys, artifact, round_number):
        self.round_number = round_number
        readings = [artifact.peaks[key] for key in keys]
        self.peaks = [peak for peak, _ in readings]
        self.hosts = [host for _, host in readings]
        self.runs = [artifact.runs[key] for key in keys]
        _, devices, fill, named, _ = keys[0]
        empty_peaks = empty_peaks_by_host(artifact)
        # 空跑格自己同样按台扣自己那一台的中位数：两台底子不同时散布按各台自己的算（V2），V3 另看 memory_peak 原值（peak_reading）
        baselines = [empty_baseline(empty_peaks, host, (devices, fill, named)) for host in self.hosts]
        self.unsubtractable = any(baseline is None for baseline in baselines)
        self.differences = [peak - (baseline or 0) for peak, baseline in zip(self.peaks, baselines)]
        self.median = median(self.differences)
        self.low, self.high = min(self.differences), max(self.differences)
        self.spread = (self.high - self.low) / max(1, statistics.median(self.peaks))
        self.stable = self.spread <= SPREAD_LIMIT
        self.resident_peak = statistics.median(window_number(run["resident_peak_over_start_bytes"]) for run in self.runs)
        self.resident_end = statistics.median(window_number(run["resident_after_return_bytes"]) for run in self.runs)
        self.digests = {run["table_digest"] for run in self.runs}

    def trajectory(self):
        """每块盘最后一槽被读到那一刻的 VmRSS 减起点，5 次的逐盘中位数；各次的点数不一样或没有轨迹（none）交空表。"""
        rows = [[window_number(word) for word in run[TRAJECTORY_FIELD].split(",") if word != "none"] for run in self.runs]
        if not rows or any(len(row) != len(rows[0]) for row in rows):
            return []
        return [statistics.median(column) for column in zip(*rows)]


def cell_value(artifact, arm, devices, fill, named):
    round_number, keys = chosen_round(artifact, arm, devices, fill, named)
    return None if keys is None else CellValue(keys, artifact, round_number)


def peak_reading(artifact, arm, devices, geometry):
    """(格读数, 说明)：格缺、空跑格 > 128 MiB（V3）、台上扣不了空跑格（V8）、这一格或空跑格 5 次散布超 2%（V2）都交 (None, 原因)。"""
    named, fill = GEOMETRIES[geometry]
    cell = cell_value(artifact, arm, devices, fill, named)
    empty = cell_value(artifact, "empty", devices, fill, named)
    if cell is None:
        return None, "not_run"
    if empty is not None and statistics.median(empty.peaks) > EMPTY_CELL_LIMIT:
        return None, "empty_cell_over_128_mebibytes"
    if cell.unsubtractable:
        return None, "empty_cell_not_run" if empty is None else "v8_no_empty_cell_on_this_host"
    if not cell.stable:
        return None, "unstable_after_rerun" if cell.round_number == 2 else "unstable_needs_rerun"
    if empty is not None and not empty.stable:
        return None, "empty_cell_unstable_after_rerun" if empty.round_number == 2 else "empty_cell_unstable_needs_rerun"
    return cell, "ok"


def subtracted_peak(artifact, arm, devices, geometry):
    """判的那一个数 P = 这一格 5 次 d_i 的中位数（第六节「判的那一个数」与修订一第六节第 5 条）。交 (P, 说明)。"""
    cell, reason = peak_reading(artifact, arm, devices, geometry)
    return (None, reason) if cell is None else (cell.median, "ok")


def cells_needing_a_rerun(artifact):
    """初测轮 5 次散布 > 2% 而复测轮还没有的格（扫环格、空跑格与 measurement_pc），登记第十一节 V2。"""
    found = []
    for geometry, (named, fill) in GEOMETRIES.items():
        arms = ("empty",) + arms_of(geometry) + (("measurement_pc",) if geometry == "k64_full" else ())
        for arm in arms:
            for devices in (DEVICE_COUNTS if arm != "measurement_pc" else (1,)):
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is not None and not cell.stable and cell.round_number == 1:
                    found.append((arm, devices, fill, named))
    return found


# ---- 行的判法 ----

def format_number(value):
    return f"{value:+.4f}"


def self_proof(value, contrast, threshold, judge):
    """判别力自证（登记第八节末）：把门槛挪到被比两个数（value 与 contrast）中间重判，判定必须翻面。
    原门槛可能已经落在两数之间（判定本来就是「红」的那一侧），所以另把门槛挪到 value 的另一侧再判一次；两次挪动里有一次判定与原判定不同就算翻面，
    两次都不变（value 等于 contrast，或判法不看门槛）记「门槛分不出」，不判。"""
    middle = (value + contrast) / 2
    outer = value + (value - contrast) / 2
    original = judge(value, threshold)
    return "flips" if judge(value, middle) != original or judge(value, outer) != original else "threshold_not_separating"


def deviation_from_the_line(p_one, p_two, p_n, devices):
    """(P(N) − [P(1) + (N−1)·(P(2) − P(1))]) ÷ P(N)；正 = 超线性。"""
    if broken("line-ignores-the-sign"):
        return 0.0
    return (p_n - (p_one + (devices - 1) * (p_two - p_one))) / p_n


def peaks_by_device(artifact, arm, geometry):
    """{盘数: P 或 None}、{盘数: 原因}。"""
    values, reasons = {}, {}
    for devices in DEVICE_COUNTS:
        values[devices], reasons[devices] = subtracted_peak(artifact, arm, devices, geometry)
    return values, reasons


def not_judged(row, geometry, reason):
    return f"E7RESULT name=verdict row={row} geometry={geometry} judgment=not_judged reason={reason}"


def line_judge(value, limit):
    return "on_line" if abs(value) <= limit else "off_line"


def line_rows(artifact):
    """甲-直线：baseline 在三个几何上的 N=3、N=4 偏离。"""
    lines = []
    for geometry in GEOMETRIES:
        values, reasons = peaks_by_device(artifact, "baseline", geometry)
        for devices in (3, 4):
            row = f"jia_line{devices}"
            needed = (values[1], values[2], values[devices])
            if any(value is None for value in needed):
                reason = next(reasons[n] for n in (1, 2, devices) if values[n] is None)
                lines.append(not_judged(row, geometry, reason))
                continue
            deviation = deviation_from_the_line(values[1], values[2], values[devices], devices)
            lines.append(
                f"E7RESULT name=verdict row={row} geometry={geometry} deviation={format_number(deviation)} limit={LINE_LIMIT} "
                f"judgment={line_judge(deviation, LINE_LIMIT)} direction={'superlinear' if deviation > 0 else 'sublinear_or_flat'} "
                f"self_proof={self_proof(abs(deviation), 0.0, LINE_LIMIT, line_judge)}")
    return lines


def ratios(artifact, arm, geometry):
    """r(N) = P(N) ÷ P(1)，N = 2、3、4；缺一个就交 None 与原因。"""
    values, reasons = peaks_by_device(artifact, arm, geometry)
    if values[1] is None:
        return None, reasons[1]
    result = {}
    for devices in (2, 3, 4):
        if values[devices] is None:
            return None, reasons[devices]
        result[devices] = values[devices] / values[1]
    return result, "ok"


def ratio_judge(value, limit):
    return "ratio_does_not_vary" if value <= limit else "ratio_varies"


def contrast_rows(artifact, row, geometry_pair):
    """甲-槽填满（k64_full 对 k64_half）、甲-点名项（k64_full 对 k1_full）：baseline 倍数之差的最大值。"""
    first, second = geometry_pair
    one, reason_one = ratios(artifact, "baseline", first)
    other, reason_other = ratios(artifact, "baseline", second)
    if one is None or other is None:
        return [not_judged(row, f"{first}_vs_{second}", reason_one if one is None else reason_other)]
    differences = {devices: abs(one[devices] - other[devices]) for devices in (2, 3, 4)}
    largest = max(differences.values())
    detail = " ".join(f"difference{devices}={differences[devices]:.4f}" for devices in (2, 3, 4))
    return [f"E7RESULT name=verdict row={row} geometry={first}_vs_{second} largest_difference={largest:.4f} {detail} "
            f"limit={RATIO_LIMIT} judgment={ratio_judge(largest, RATIO_LIMIT)} "
            f"self_proof={self_proof(largest, 0.0, RATIO_LIMIT, ratio_judge)}"]


def red_judge(value, factor):
    return "red" if broken("pc-red-threshold-ignored") or value > factor else "not_red"


def control_rows(artifact):
    """甲-判别力：同次自比（不判、照报）与绝对字节（判）。pc 红 ⟺ 比基线高过 10%。"""
    lines = []
    for geometry in GEOMETRIES:
        base_values, base_reasons = peaks_by_device(artifact, "baseline", geometry)
        pc_values, pc_reasons = peaks_by_device(artifact, "baseline_pc", geometry)
        for devices in DEVICE_COUNTS:
            if base_values[devices] is None or pc_values[devices] is None:
                reason = base_reasons[devices] if base_values[devices] is None else pc_reasons[devices]
                lines.append(not_judged("jia_pc_absolute", f"{geometry}_n{devices}", reason))
                continue
            ratio = pc_values[devices] / base_values[devices]
            lines.append(
                f"E7RESULT name=verdict row=jia_pc_absolute geometry={geometry} devices={devices} pc_over_baseline={ratio:.4f} "
                f"factor={PC_RED_FACTOR} judgment={red_judge(ratio, PC_RED_FACTOR)} "
                f"self_proof={self_proof(ratio, 1.0, PC_RED_FACTOR, red_judge)}")
        base_ratios, _ = ratios(artifact, "baseline", geometry)
        pc_ratios, _ = ratios(artifact, "baseline_pc", geometry)
        for devices in (2, 3, 4):
            if base_ratios is None or pc_ratios is None:
                lines.append(not_judged("jia_pc_self_ratio", f"{geometry}_n{devices}", "ratio_missing"))
                continue
            ratio = pc_ratios[devices] / base_ratios[devices]
            lines.append(
                f"E7RESULT name=verdict row=jia_pc_self_ratio geometry={geometry} devices={devices} pc_ratio_over_baseline_ratio={ratio:.4f} "
                f"factor={PC_RED_FACTOR} judgment={'red' if ratio > PC_RED_FACTOR else 'not_red'} note=reading_deleted_reported_only")
    return lines


def close_judge(value, limit):
    return "close_to_one" if value <= limit else "not_close_to_one"


def slope_of(values):
    """(P(4) − P(2)) ÷ (2 × P(1))：每多一盘峰值涨多少个单盘峰值。"""
    if broken("slope-uses-the-top-point-only"):
        return values[4] / (2 * values[1])
    return (values[4] - values[2]) / (2 * values[1])


def slope_judge(value, limit):
    return "no_growth" if value <= limit else "still_grows"


def split_rows(artifact):
    """乙-接近一（不判，照报）与乙-不涨（判）：split 与 split_pc 各三个几何。"""
    lines = []
    for geometry in GEOMETRIES:
        split_ratios, split_reason = ratios(artifact, "split", geometry)
        control_ratios, control_reason = ratios(artifact, "split_pc", geometry)
        split_values, _ = peaks_by_device(artifact, "split", geometry)
        control_values, _ = peaks_by_device(artifact, "split_pc", geometry)
        if split_ratios is None or control_ratios is None:
            reason = split_reason if split_ratios is None else control_reason
            lines.append(not_judged("yi_close_to_one", geometry, reason))
            lines.append(not_judged("yi_no_growth", geometry, reason))
            continue
        lines.append(
            f"E7RESULT name=verdict row=yi_close_to_one geometry={geometry} split_r2={split_ratios[2]:.4f} split_r3={split_ratios[3]:.4f} "
            f"split_r4={split_ratios[4]:.4f} split_pc_r4={control_ratios[4]:.4f} limit={CLOSE_TO_ONE_LIMIT} "
            f"judgment={close_judge(split_ratios[4], CLOSE_TO_ONE_LIMIT)} note=reading_deleted_reported_only "
            f"self_proof={self_proof(split_ratios[4], control_ratios[4], CLOSE_TO_ONE_LIMIT, close_judge)}")
        split_slope, control_slope = slope_of(split_values), slope_of(control_values)
        lines.append(
            f"E7RESULT name=verdict row=yi_no_growth geometry={geometry} split_slope={split_slope:.4f} split_pc_slope={control_slope:.4f} "
            f"limit={SLOPE_LIMIT} judgment={slope_judge(split_slope, SLOPE_LIMIT)} "
            f"self_proof={self_proof(split_slope, control_slope, SLOPE_LIMIT, slope_judge)}")
    return lines


def doable_row(artifact):
    """乙-做得到：各臂交回的表摘要与基线臂同格的相同（生产格；读缓存两臂不改交回值，一并比），加差分集 D 的二十行、compare 的五十二行全等。"""
    mismatches, compared = 0, 0
    for geometry, (named, fill) in GEOMETRIES.items():
        for devices in DEVICE_COUNTS:
            digests = {}
            for arm in arms_of(geometry):
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is not None:
                    digests[arm] = cell.digests
            if "baseline" not in digests or len(digests) < 2:
                continue
            for arm, found in digests.items():
                compared += 1
                if found != digests["baseline"] or len(found) != 1:
                    mismatches += 1
    if broken("digest-mismatch-ignored"):
        mismatches = 0
    diff_mismatches = sum(1 for fields in artifact.diff if fields.get("equal") != "true")
    compare_mismatches = sum(1 for fields in artifact.compare if fields.get("equal") != "true")
    if compared == 0 and not artifact.diff and not artifact.compare:
        return ["E7RESULT name=verdict row=yi_doable judgment=not_judged reason=not_run"]
    if mismatches or diff_mismatches or compare_mismatches:
        judgment = "not_doable"
    elif len(artifact.diff) == DIFFERENTIAL_LINES and len(artifact.compare) == COMPARE_LINES:
        judgment = "doable_on_the_prototype"
    else:
        judgment = "incomplete"
    return [f"E7RESULT name=verdict row=yi_doable arms_compared={compared} digest_mismatches={mismatches} "
            f"diff_lines={len(artifact.diff)} diff_mismatches={diff_mismatches} "
            f"compare_lines={len(artifact.compare)} compare_mismatches={compare_mismatches} judgment={judgment}"]


def triggered(flag):
    return "yes" if flag else "no"


def failure_rows(artifact):
    """失败条款 F1–F5（登记第十节），与条款推算数的对照 C2（第七节）。"""
    lines = []
    base, _ = peaks_by_device(artifact, "baseline", "k64_full")
    control, _ = peaks_by_device(artifact, "baseline_pc", "k64_full")
    if all(base[n] is not None and control[n] is not None for n in DEVICE_COUNTS):
        weak = [n for n in DEVICE_COUNTS if control[n] <= PC_RED_FACTOR * base[n]]
        lines.append(f"E7RESULT name=verdict row=F1 triggered={triggered(weak)} devices_not_above_factor={','.join(map(str, weak)) or 'none'}")
    else:
        lines.append("E7RESULT name=verdict row=F1 triggered=not_judged reason=not_run")
    split_control, _ = peaks_by_device(artifact, "split_pc", "k64_full")
    if all(split_control[n] is not None for n in DEVICE_COUNTS):
        control_slope = slope_of(split_control)
        control_ratio = split_control[4] / split_control[1]
        lines.append(f"E7RESULT name=verdict row=F2 triggered={triggered(control_slope <= SLOPE_LIMIT or control_ratio <= CLOSE_TO_ONE_LIMIT)} "
                     f"split_pc_slope={control_slope:.4f} split_pc_r4={control_ratio:.4f}")
    else:
        lines.append("E7RESULT name=verdict row=F2 triggered=not_judged reason=not_run")
    if base[2] is not None and base[4] is not None:
        outside = [n for n, claimed in ((2, CLAUSE_PAIR_BYTES[0]), (4, CLAUSE_PAIR_BYTES[1])) if abs(base[n] / claimed - 1) > CLAUSE_TOLERANCE]
        lines.append(f"E7RESULT name=verdict row=F3 triggered={triggered(outside)} baseline_p2={base[2]:.0f} baseline_p4={base[4]:.0f} "
                     f"clause_two_devices={CLAUSE_PAIR_BYTES[0]:.0f} clause_four_devices={CLAUSE_PAIR_BYTES[1]:.0f} outside_devices={','.join(map(str, outside)) or 'none'}")
    else:
        lines.append("E7RESULT name=verdict row=F3 triggered=not_judged reason=not_run")
    if all(base[n] is not None for n in DEVICE_COUNTS):
        ratio2, ratio4 = base[2] / base[1], base[4] / base[1]
        outside = [n for n, value in ((2, ratio2), (4, ratio4)) if not F4_RATIO_RANGES[n][0] <= value <= F4_RATIO_RANGES[n][1]]
        lines.append(f"E7RESULT name=verdict row=F4 triggered={triggered(outside)} baseline_r2={ratio2:.4f} baseline_r4={ratio4:.4f} outside_devices={','.join(map(str, outside)) or 'none'}")
    else:
        lines.append("E7RESULT name=verdict row=F4 triggered=not_judged reason=not_run")
    lines.append(measurement_row(artifact))
    return lines


def measurement_row(artifact):
    """F5（登记修订三第十节）：measurement_pc 的 H 落在 [254 MiB, 260 MiB]，P（扣同台同几何空跑格）落在 [256 MiB, 288 MiB]。"""
    cell, reason = peak_reading(artifact, "measurement_pc", 1, "k64_full")
    if cell is None:
        return f"E7RESULT name=verdict row=F5 triggered=not_judged reason={reason}"
    resident_bad = not MEASUREMENT_RESIDENT_WINDOW[0] <= cell.resident_peak <= MEASUREMENT_RESIDENT_WINDOW[1]
    cgroup_bad = not MEASUREMENT_BYTES <= cell.median <= MEASUREMENT_CGROUP_HIGH
    return (f"E7RESULT name=verdict row=F5 triggered={triggered(resident_bad or cgroup_bad)} resident_peak_over_start={cell.resident_peak:.0f} "
            f"cgroup_over_empty={cell.median:.0f} resident_outside_window={triggered(resident_bad)} cgroup_outside_window={triggered(cgroup_bad)}")


def each_scanning_cell(artifact, arms):
    """(几何, 臂, 盘数, 填满档, 点名项数, 格读数) 逐个交出已有完整 5 次的扫环格（只交 arms 里的、这一几何上跑的臂）。"""
    for geometry, (named, fill) in GEOMETRIES.items():
        for arm in arms_of(geometry):
            if arm not in arms:
                continue
            for devices in DEVICE_COUNTS:
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is not None:
                    yield geometry, arm, devices, fill, named, cell


def anchor_rows(artifact):
    """A7（四个胶水臂的 H 对常驻页推算 R ±5%）、S3（H 与 P 两条量法互相包含；读缓存两臂的那几格另进 S5）、轨迹两条断言、A9（登记修订三第七、八、十一节）。"""
    outside_a7, outside_s3, outside_trajectory, cells, s3_cells = [], [], [], 0, 0
    worst = 0.0
    for geometry, arm, devices, fill, named, cell in each_scanning_cell(artifact, GLUE_ARMS):
        cells += 1
        run = cell.runs[0]
        filled = int(run["filled_slots"])
        predicted = predicted_resident_peak(arm, devices, filled, named, sizes_of(run))
        deviation = cell.resident_peak / predicted - 1
        worst = max(worst, abs(deviation))
        if abs(deviation) > PREDICTION_TOLERANCE:
            outside_a7.append(f"{geometry}:{arm}:n{devices}:{deviation:+.3f}")
        if arm in ("baseline", "baseline_pc"):
            # 修订三第八节两条：最后一盘的轨迹值对「读到那一刻的常驻页」±3%；H ≥ 这一格轨迹里最大的那个数
            expected = expected_last_device_end(devices, filled, fill, int(run["ring_slots"]), named, sizes_of(run))
            ends = cell.trajectory()
            if len(ends) != devices or abs(ends[-1] / expected - 1) > TRAJECTORY_TOLERANCE or cell.resident_peak < max(ends):
                outside_trajectory.append(f"{geometry}:{arm}:n{devices}")
    for geometry, arm, devices, fill, named, cell in each_scanning_cell(artifact, SCANNING_ARMS):
        s3_cells += 1
        peak, _ = subtracted_peak(artifact, arm, devices, geometry)
        if peak is not None and (cell.resident_peak > peak + TWO_MEASURES_ABSOLUTE_SLACK
                                 or peak > TWO_MEASURES_RELATIVE[0] * cell.resident_peak + TWO_MEASURES_RELATIVE[1]):
            outside_s3.append(f"{geometry}:{arm}:n{devices}")
    rows = [f"E7RESULT name=verdict row=A7 cells={cells} largest_deviation={worst:.4f} cells_outside={len(outside_a7)} "
            f"outside={','.join(outside_a7) or 'none'}",
            f"E7RESULT name=verdict row=S3_two_measures cells={s3_cells} cells_outside={len(outside_s3)} outside={','.join(outside_s3) or 'none'}",
            f"E7RESULT name=verdict row=trajectory cells={cells} cells_outside={len(outside_trajectory)} outside={','.join(outside_trajectory) or 'none'}"]
    rows.append(same_one_device_row(artifact))
    return rows


def same_one_device_row(artifact):
    """A9（登记修订三第七节）：只有一盘时 split 与 baseline 做的是同一串分配，H 之差 ≤ baseline 那一格 H 的 1%、P 之差 ≤ 3%。"""
    outside, compared = [], 0
    for geometry, (named, fill) in GEOMETRIES.items():
        base = cell_value(artifact, "baseline", 1, fill, named)
        split = cell_value(artifact, "split", 1, fill, named)
        if base is None or split is None:
            continue
        compared += 1
        resident_difference = abs(split.resident_peak - base.resident_peak) / base.resident_peak
        base_peak, _ = subtracted_peak(artifact, "baseline", 1, geometry)
        split_peak, _ = subtracted_peak(artifact, "split", 1, geometry)
        cgroup_difference = abs(split_peak / base_peak - 1) if base_peak and split_peak else None
        if resident_difference > SAME_ONE_DEVICE_RESIDENT_LIMIT or cgroup_difference is None or cgroup_difference > SAME_ONE_DEVICE_CGROUP_LIMIT:
            outside.append(geometry)
    return f"E7RESULT name=verdict row=A9 geometries={compared} geometries_outside={len(outside)} outside={','.join(outside) or 'none'}"


# ---- 丙（读缓存上界，C881）：登记修订一第六、七、十、十一节 ----

def interval_of(function, boxes):
    """function 在各量区间 [低, 高] 的端点上取遍，交 (最小, 最大)：登记修订一「分辨率」一段的区间算术（各行的式子对每个量单调）。"""
    values = [function(*corner) for corner in product(*boxes)]
    return min(values), max(values)


def absolute_interval(low, high):
    if low <= 0 <= high:
        return 0.0, max(-low, high)
    return min(abs(low), abs(high)), max(abs(low), abs(high))


def judged_by_interval(point, low, high, limit, inside, outside):
    """区间整个 ≤ 门槛判 inside、整个 > 门槛判 outside，跨门槛报 indistinguishable（分不出，不判）；弄坏开关 interval-ignored 下只看中位数。"""
    if broken("interval-ignored"):
        return inside if point <= limit else outside
    if high <= limit:
        return inside
    if low > limit:
        return outside
    return "indistinguishable"


def q_reading(artifact, arm, devices, geometry):
    """(Q, 低, 高, 说明)：Q = P(臂) − P(baseline)，同盘数同几何（修订一第一节「读法写死（丙行）」）；区间取两格 d_i 的端点。"""
    cell, reason = peak_reading(artifact, arm, devices, geometry)
    base, base_reason = peak_reading(artifact, "baseline", devices, geometry)
    if cell is None or base is None:
        return None, None, None, reason if cell is None else base_reason
    return cell.median - base.median, cell.low - base.high, cell.high - base.low, "ok"


def q_by_device(artifact, arm, geometry):
    """{盘数: (Q, 低, 高) 或 None}、{盘数: 原因}。"""
    values, reasons = {}, {}
    for devices in DEVICE_COUNTS:
        point, low, high, reasons[devices] = q_reading(artifact, arm, devices, geometry)
        values[devices] = None if point is None else (point, low, high)
    return values, reasons


def box(value):
    return (value[1], value[2])


def bing_line_rows(artifact):
    """丙-直线（3 盘、4 盘）：read_stage_cache 的 Q 在三个几何上对过 1、2 盘两点的直线的偏离，带区间。"""
    lines = []
    for geometry in GEOMETRIES:
        values, reasons = q_by_device(artifact, "read_stage_cache", geometry)
        for devices in (3, 4):
            row = f"bing_line{devices}"
            if any(values[n] is None for n in (1, 2, devices)):
                lines.append(not_judged(row, geometry, next(reasons[n] for n in (1, 2, devices) if values[n] is None)))
                continue
            point = deviation_from_the_line(values[1][0], values[2][0], values[devices][0], devices)
            low, high = absolute_interval(*interval_of(lambda one, two, top: deviation_from_the_line(one, two, top, devices),
                                                       [box(values[1]), box(values[2]), box(values[devices])]))
            lines.append(
                f"E7RESULT name=verdict row={row} geometry={geometry} deviation={format_number(point)} absolute_interval={low:.4f},{high:.4f} "
                f"limit={LINE_LIMIT} judgment={judged_by_interval(abs(point), low, high, LINE_LIMIT, 'on_line', 'off_line')} "
                f"direction={'superlinear' if point > 0 else 'sublinear_or_flat'} self_proof={self_proof(abs(point), 0.0, LINE_LIMIT, line_judge)}")
    return lines


def q_ratios(artifact, arm, geometry):
    """r_Q(N) = Q(N) ÷ Q(1)，N = 2、3、4：{N: (点, 低, 高)}；缺一个交 None 与原因。"""
    values, reasons = q_by_device(artifact, arm, geometry)
    for devices in DEVICE_COUNTS:
        if values[devices] is None:
            return None, reasons[devices]
    found = {}
    for devices in (2, 3, 4):
        low, high = interval_of(lambda top, one: top / one, [box(values[devices]), box(values[1])])
        found[devices] = (values[devices][0] / values[1][0], low, high)
    return found, "ok"


def bing_contrast_rows(artifact, row, geometry_pair):
    """丙-槽填满（k64_full 对 k64_half）、丙-点名项（k64_full 对 k1_full）：read_stage_cache 的 r_Q 之差的最大值，带区间。"""
    first, second = geometry_pair
    one, reason_one = q_ratios(artifact, "read_stage_cache", first)
    other, reason_other = q_ratios(artifact, "read_stage_cache", second)
    if one is None or other is None:
        return [not_judged(row, f"{first}_vs_{second}", reason_one if one is None else reason_other)]
    points, lows, highs = [], [], []
    for devices in (2, 3, 4):
        points.append(abs(one[devices][0] - other[devices][0]))
        low, high = absolute_interval(one[devices][1] - other[devices][2], one[devices][2] - other[devices][1])
        lows.append(low)
        highs.append(high)
    largest, low, high = max(points), max(lows), max(highs)
    return [f"E7RESULT name=verdict row={row} geometry={first}_vs_{second} largest_difference={largest:.4f} interval={low:.4f},{high:.4f} "
            f"limit={RATIO_LIMIT} judgment={judged_by_interval(largest, low, high, RATIO_LIMIT, 'ratio_does_not_vary', 'ratio_varies')} "
            f"self_proof={self_proof(largest, 0.0, RATIO_LIMIT, ratio_judge)}"]


def bing_control_rows(artifact):
    """丙-判别力·绝对字节（K = 64 半满，带区间）与丙-判别力·同次自比（照报），丙-倍数（三个几何照报）。"""
    lines = []
    values, reasons = q_by_device(artifact, "read_stage_cache", "k64_half")
    controls, control_reasons = q_by_device(artifact, "read_stage_cache_keeps_zeros_pc", "k64_half")
    for devices in DEVICE_COUNTS:
        if values[devices] is None or controls[devices] is None:
            lines.append(not_judged("bing_pc_absolute", f"k64_half_n{devices}", reasons[devices] if values[devices] is None else control_reasons[devices]))
            continue
        point = controls[devices][0] / values[devices][0]
        low, high = interval_of(lambda control, value: control / value, [box(controls[devices]), box(values[devices])])
        judgment = judged_by_interval(point, low, high, PC_RED_FACTOR, "not_red", "red")
        lines.append(f"E7RESULT name=verdict row=bing_pc_absolute geometry=k64_half devices={devices} pc_over_cache={point:.4f} "
                     f"interval={low:.4f},{high:.4f} factor={PC_RED_FACTOR} judgment={judgment} self_proof={self_proof(point, 1.0, PC_RED_FACTOR, red_judge)}")
    cache_ratios, _ = q_ratios(artifact, "read_stage_cache", "k64_half")
    control_ratios, _ = q_ratios(artifact, "read_stage_cache_keeps_zeros_pc", "k64_half")
    for devices in (2, 3, 4):
        if cache_ratios is None or control_ratios is None:
            lines.append(not_judged("bing_pc_self_ratio", f"k64_half_n{devices}", "ratio_missing"))
            continue
        ratio = control_ratios[devices][0] / cache_ratios[devices][0]
        lines.append(f"E7RESULT name=verdict row=bing_pc_self_ratio geometry=k64_half devices={devices} pc_ratio_over_cache_ratio={ratio:.4f} "
                     f"factor={PC_RED_FACTOR} judgment={'red' if ratio > PC_RED_FACTOR else 'not_red'} note=reported_only")
    for geometry in GEOMETRIES:
        found, reason = q_ratios(artifact, "read_stage_cache", geometry)
        own, own_reason = ratios(artifact, "read_stage_cache", geometry)
        if found is None or own is None:
            lines.append(not_judged("bing_ratios", geometry, reason if found is None else own_reason))
            continue
        lines.append(f"E7RESULT name=verdict row=bing_ratios geometry={geometry} "
                     + " ".join(f"q_r{devices}={found[devices][0]:.4f} q_r{devices}_interval={found[devices][1]:.4f},{found[devices][2]:.4f}" for devices in (2, 3, 4))
                     + " " + " ".join(f"cache_arm_r{devices}={own[devices]:.4f}" for devices in (2, 3, 4)) + " judgment=reported_only")
    return lines


def bing_failure_rows(artifact):
    """F6（阳性对照分不出）、F7（C3：Q(2) 对 C881 正文 1.5 GiB ±25%）、F8（C4：r_Q(2)、r_Q(4) 的区间）。"""
    lines = []
    values, _ = q_by_device(artifact, "read_stage_cache", "k64_half")
    controls, _ = q_by_device(artifact, "read_stage_cache_keeps_zeros_pc", "k64_half")
    if all(values[n] is not None and controls[n] is not None for n in DEVICE_COUNTS):
        weak = [n for n in DEVICE_COUNTS if controls[n][0] <= PC_RED_FACTOR * values[n][0]]
        lines.append(f"E7RESULT name=verdict row=F6 triggered={triggered(weak)} devices_not_above_factor={','.join(map(str, weak)) or 'none'}")
    else:
        lines.append("E7RESULT name=verdict row=F6 triggered=not_judged reason=not_run")
    full, _ = q_by_device(artifact, "read_stage_cache", "k64_full")
    if full[2] is not None:
        outside = not C3_RANGE[0] <= full[2][0] <= C3_RANGE[1]
        lines.append(f"E7RESULT name=verdict row=F7 triggered={triggered(outside)} q2={full[2][0]:.0f} clause_range={C3_RANGE[0]},{C3_RANGE[1]}")
    else:
        lines.append("E7RESULT name=verdict row=F7 triggered=not_judged reason=not_run")
    if all(full[n] is not None for n in DEVICE_COUNTS):
        ratio2, ratio4 = full[2][0] / full[1][0], full[4][0] / full[1][0]
        outside = [n for n, value in ((2, ratio2), (4, ratio4)) if not F4_RATIO_RANGES[n][0] <= value <= F4_RATIO_RANGES[n][1]]
        lines.append(f"E7RESULT name=verdict row=F8 triggered={triggered(outside)} q_r2={ratio2:.4f} q_r4={ratio4:.4f} outside_devices={','.join(map(str, outside)) or 'none'}")
    else:
        lines.append("E7RESULT name=verdict row=F8 triggered=not_judged reason=not_run")
    return lines


def cache_anchor_rows(artifact):
    """A10（缓存项数与值的字节和，逐次）、A11（H 比同格基线多的 = 项数 × (4112 + 560 ÷ 6)，±5%）、A12（期末值同理）、S5（登记修订三第七、十一节）。"""
    a10_runs, a10_outside = 0, []
    for key, fields in artifact.runs.items():
        if key[0] not in READ_STAGE_CACHE_ARMS:
            continue
        a10_runs += 1
        expected = expected_cache_entries(key[0], key[1], int(fields.get("filled_slots", "-1")))
        if fields.get("cache_entries") != str(expected) or fields.get("cache_value_bytes") != str(expected * 4096):
            a10_outside.append("/".join(map(str, key)))
    a11_outside, a12_outside, compared = [], [], 0
    for geometry, arm, devices, fill, named, cell in each_scanning_cell(artifact, READ_STAGE_CACHE_ARMS):
        base = cell_value(artifact, "baseline", devices, fill, named)
        if base is None:
            continue
        compared += 1
        predicted = cache_bytes(expected_cache_entries(arm, devices, int(cell.runs[0]["filled_slots"])))
        for outside, measured in ((a11_outside, cell.resident_peak - base.resident_peak), (a12_outside, cell.resident_end - base.resident_end)):
            if abs(measured / predicted - 1) > CACHE_TOLERANCE:
                outside.append(f"{geometry}:{arm}:n{devices}:{measured / predicted - 1:+.3f}")
    s3 = [label for row in anchor_rows(artifact)[1:2] for label in parse_line(row)["outside"].split(",") if "read_stage_cache" in label]
    stop = bool(a11_outside or a12_outside or s3)
    return [f"E7RESULT name=verdict row=A10 runs={a10_runs} runs_outside={len(a10_outside)} outside={','.join(a10_outside) or 'none'}",
            f"E7RESULT name=verdict row=A11 cells={compared} cells_outside={len(a11_outside)} outside={','.join(a11_outside) or 'none'}",
            f"E7RESULT name=verdict row=A12 cells={compared} cells_outside={len(a12_outside)} outside={','.join(a12_outside) or 'none'}",
            f"E7RESULT name=verdict row=S5 stop={'yes' if stop else 'no'} s3_cache_cells_outside={','.join(s3) or 'none'}"]


def host_rows(artifact):
    """两台空跑格之差（V8 另报一行，不判），与每条结果行钉没钉到服务记录的计数（V6、V7、V9、同一格两行）。"""
    empty_peaks = empty_peaks_by_host(artifact)
    differences = []
    for geometry in sorted({geometry for by_geometry in empty_peaks.values() for geometry in by_geometry}, key=str):
        here, there = empty_peaks.get("local", {}).get(geometry), empty_peaks.get("peer", {}).get(geometry)
        if here and there:
            devices, fill, named = geometry
            differences.append(f"k{named}_{fill}_n{devices}:{statistics.median(here) - statistics.median(there):+.0f}")
    reasons = {}
    for reason in artifact.voided.values():
        reasons[reason] = reasons.get(reason, 0) + 1
    counted = " ".join(f"{reason}={count}" for reason, count in sorted(reasons.items()))
    return [f"E7RESULT name=verdict row=empty_cell_between_hosts geometries={len(differences)} local_minus_peer={','.join(differences) or 'none'} judgment=reported_only",
            f"E7RESULT name=attempt_summary result_lines={len(artifact.runs)} pinned={len(artifact.peaks)} voided={len(artifact.voided)} "
            f"cells_lost_in_the_artifacts={len(artifact.lost)} {counted}".rstrip()]


def cell_summary_rows(artifact):
    """每格一行（不判）：memory_peak 中位数、各次的台、散布、轮次、P 与 d_i 的区间、H、期末值、VmHWM、扫环秒数。"""
    lines = []
    for geometry, (named, fill) in GEOMETRIES.items():
        for arm in ("empty",) + arms_of(geometry) + (("measurement_pc",) if geometry == "k64_full" else ()):
            for devices in (DEVICE_COUNTS if arm != "measurement_pc" else (1,)):
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is None:
                    continue
                peak, reason = subtracted_peak(artifact, arm, devices, geometry)
                hwm = statistics.median(window_number(run["vm_hwm_kib"]) for run in cell.runs)
                seconds = statistics.median(float(run.get("scan_seconds", "0")) for run in cell.runs)
                lines.append(
                    f"E7RESULT name=cell_summary arm={arm} devices={devices} geometry={geometry} round={cell.round_number} "
                    f"memory_peak_median_bytes={statistics.median(cell.peaks):.0f} hosts={','.join(cell.hosts)} spread={cell.spread:.4f} "
                    f"stable={'yes' if cell.stable else 'no'} p_bytes={'none' if peak is None else format(peak, '.0f')} p_reason={reason} "
                    f"difference_interval={cell.low:.0f},{cell.high:.0f} resident_peak_bytes={cell.resident_peak:.0f} "
                    f"resident_end_bytes={cell.resident_end:.0f} vm_hwm_median_kib={hwm:.0f} scan_seconds_median={seconds:.3f}")
    return lines


def all_rows(artifact):
    """整份的判决行，登记第六节的次序，丙各行接在后面；缺的格逐行报 not_judged，不当 0。"""
    rows = cell_summary_rows(artifact)
    rows += line_rows(artifact)
    rows += contrast_rows(artifact, "jia_slot_fill", ("k64_full", "k64_half"))
    rows += contrast_rows(artifact, "jia_named", ("k64_full", "k1_full"))
    rows += control_rows(artifact)
    rows += split_rows(artifact)
    rows += doable_row(artifact)
    rows += failure_rows(artifact)
    rows += anchor_rows(artifact)
    rows += bing_line_rows(artifact)
    rows += bing_contrast_rows(artifact, "bing_slot_fill", ("k64_full", "k64_half"))
    rows += bing_contrast_rows(artifact, "bing_named", ("k64_full", "k1_full"))
    rows += bing_control_rows(artifact)
    rows += bing_failure_rows(artifact)
    rows += cache_anchor_rows(artifact)
    rows += host_rows(artifact)
    return rows


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


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    artifact_paths, status_paths = options_after(arguments, "--artifacts"), options_after(arguments, "--status")
    output = options_after(arguments, "--out")
    known = {"--artifacts", "--status", "--out", "--unstable"}
    if not artifact_paths or not status_paths or "--out" in arguments and len(output) != 1 or any(
            word.startswith("--") and word not in known for word in arguments):
        print("用法：report.py --artifacts <产物.out…> --status <状态存档.json…> [--out <判决.out> | --unstable] | --selftest", file=sys.stderr)
        return 2
    try:
        artifact = load(artifact_paths, status_paths)
    except (OSError, ValueError) as error:
        print(f"读不了产物或状态存档：{error}", file=sys.stderr)
        return 2
    if len(artifact.sizes_seen) > 1:
        print(f"产物里的 size_of 不止一组：{sorted(artifact.sizes_seen)}（装置版本不一致，整份不判）", file=sys.stderr)
        return 3
    if "--unstable" in arguments:
        for arm, devices, fill, named in cells_needing_a_rerun(artifact):
            print(f"{arm} {devices} {fill} {named}")
        return 0
    rows = all_rows(artifact)
    if output:
        try:
            with open(output[0], "x", encoding="utf-8") as handle:
                handle.write("\n".join(rows) + f"\nE7RESULT name=done emitted={len(rows) + 1}\n")
        except FileExistsError:
            print(f"判决文件 {output[0]} 已在，不覆盖（→ 怎么办：换一个新文件名重判）", file=sys.stderr)
            return 2
    for row in rows:
        print(row)
    return 0


# ---- 自证：合成产物与合成状态存档 ----

SYNTHETIC_SIZES = {"location_entry": 16, "named_unit": 64, "node_pointer": 56, "journal_record": 320}
SYNTHETIC_EMPTY_BYTES = 60 * MEBIBYTE
SYNTHETIC_OVERHEAD_BYTES = 20 * MEBIBYTE   # 扫环格的 memory_peak 比 H 多的那一截（页表、内核记账、装置二进制与驱动的页，H 没数到的）
SYNTHETIC_BINARY = "synthetic-binary-sha256"
SYNTHETIC_CAP = 16 * GIBIBYTE
SYNTHETIC_CHECKS_LINE = f"E7RESULT name=e197_config step=checks binary_sha256={SYNTHETIC_BINARY}"
SYNTHETIC_TASKS = iter(range(100000, 10 ** 9))


def synthetic_line(arm, devices, fill, named, repeat, resident, ends=None, digest=None, end=0, reading=None):
    """一条装置结果行，带 drive.py 的片加上的 task= attempt= shard_host= binary_sha256=；reading 里另写合成状态存档要的那一次尝试（synthetic_*）；
    reading 里 unreadable=True 的，量窗那几个字段写 unreadable（装置写不进 clear_refs 或读不出 /proc/self/status 的样子）。"""
    reading = dict({"peak": 0, "host": "local", "outcome": SUCCEEDED, "cap": SYNTHETIC_CAP, "binary": SYNTHETIC_BINARY, "pinned": True,
                    "task": next(SYNTHETIC_TASKS), "unreadable": False}, **(reading or {}))
    filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
    scans = arm in SCANNING_ARMS
    ends = ends or ([0] * devices if scans else [])
    expected = expected_cache_entries(arm, devices, filled)
    cache = ("cache_entries=none cache_entries_expected=none cache_value_bytes=none" if expected is None
             else f"cache_entries={expected} cache_entries_expected={expected} cache_value_bytes={expected * 4096}")
    sizes = " ".join(f"size_{name}={value}" for name, value in SYNTHETIC_SIZES.items())
    pinned = f" task={reading['task']} attempt=1" if reading["pinned"] else ""
    if reading["unreadable"]:
        window = ("resident_peak_over_start_bytes=unreadable resident_at_each_device_end_bytes=unreadable resident_after_return_bytes=unreadable "
                  "vm_rss_at_start_kib=unreadable vm_hwm_kib=unreadable high_water_mark_reset=unreadable")
    else:
        window = (f"resident_peak_over_start_bytes={resident} resident_at_each_device_end_bytes={','.join(map(str, ends)) or 'none'} "
                  f"resident_after_return_bytes={end} vm_rss_at_start_kib=1000 vm_hwm_kib=2000 high_water_mark_reset=ok")
    return (f"E7RESULT name=e197 arm={arm} devices={devices} fill={fill} named={named} repeat={repeat} ring_slots={RING_SLOTS} "
            f"filled_slots={filled} reads={devices * RING_SLOTS if scans else 0} reads_expected={devices * RING_SLOTS if scans else 0} "
            f"records_returned={filled if scans else 0} records_expected={filled if scans else 0} anchors_hold=true "
            f"{window} scan_seconds=1.000000 "
            f"table_digest={digest or ('none' if not scans else f'd{devices}{fill}{named}')} {cache} {sizes}{pinned} shard_host=synthetic "
            f"binary_sha256={reading['binary']} synthetic_memory_peak={reading['peak']} synthetic_host={reading['host']} "
            f"synthetic_outcome={reading['outcome']} synthetic_cap={reading['cap']}")


def synthetic_status_of(lines):
    """合成产物行里 synthetic_* 写的那几次尝试 → 一份照 task-server/task status --json 形状的状态存档（父任务加 children）。"""
    children = []
    for line in lines:
        fields = parse_line(line)
        if fields is None or fields.get("name") != "e197" or "task" not in fields:
            continue
        peak = fields["synthetic_memory_peak"]
        children.append({"task": {"id": int(fields["task"]), "role": "shard"},
                         "attempts": [{"task_id": int(fields["task"]), "number": 1, "host": fields["synthetic_host"],
                                       "memory_peak": int(peak) if peak.isdigit() else None, "memory_cap": int(fields["synthetic_cap"]),
                                       "outcome": fields["synthetic_outcome"]}]})
    return [{"task": {"id": 1, "sharded_entry": "drive.py"}, "attempts": [], "children": children}]


def synthetic_artifact(lines):
    return Artifact(lines, synthetic_status_of(lines))


def synthetic_artifact_lines(mutate=None):
    """每一格 5 次（重复号 1–5）：H 取常驻页推算值（读缓存两臂另加缓存那一截），memory_peak = 空跑格 + H + 20 MiB，5 次之间差 1000 字节。
    mutate(arm, devices, fill, named, repeat, fields) 可以改一格里的字段（resident、cgroup、digest、host、outcome、cap、binary、pinned、task、unreadable、peak_missing），
    交 None 的整次删掉。另带 checks 的配置行、差分集二十行、compare 五十二行。"""
    lines = [SYNTHETIC_CHECKS_LINE]
    for geometry, (named, fill) in GEOMETRIES.items():
        filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
        arms = ("empty",) + arms_of(geometry) + (("measurement_pc",) if geometry == "k64_full" else ())
        for arm in arms:
            for devices in (DEVICE_COUNTS if arm != "measurement_pc" else (1,)):
                resident = int(predicted_resident_peak(arm, devices, filled, named, SYNTHETIC_SIZES))
                end = 0
                if arm in READ_STAGE_CACHE_ARMS:
                    end = int(cache_bytes(expected_cache_entries(arm, devices, filled)))
                    resident += end
                cgroup = SYNTHETIC_EMPTY_BYTES + resident + (SYNTHETIC_OVERHEAD_BYTES if arm in SCANNING_ARMS else 0)
                if arm == "measurement_pc":
                    resident, cgroup = MEASUREMENT_BYTES + 4096, SYNTHETIC_EMPTY_BYTES + MEASUREMENT_BYTES + 8 * MEBIBYTE
                for repeat in range(1, REPEATS_PER_ROUND + 1):
                    ends = None
                    if arm in ("baseline", "baseline_pc"):
                        ends = [int(expected_last_device_end(d, filled, fill, RING_SLOTS, named, SYNTHETIC_SIZES)) for d in range(1, devices + 1)]
                    fields = {"resident": resident, "cgroup": cgroup + 1000 * repeat, "digest": None}
                    if mutate is not None:
                        fields = mutate(arm, devices, fill, named, repeat, fields)
                        if fields is None:
                            continue
                    reading = {name: fields[name] for name in ("host", "outcome", "cap", "binary", "pinned", "task", "unreadable") if name in fields}
                    reading["peak"] = "none" if fields.get("peak_missing") else fields["cgroup"]
                    lines.append(synthetic_line(arm, devices, fill, named, repeat, fields["resident"], ends, fields["digest"], end, reading))
    for case in ("D1", "D2", "D3", "D4"):
        for arm in ("baseline_pc", "split", "split_pc") + READ_STAGE_CACHE_ARMS:
            lines.append(f"E7RESULT name=e197_diff case={case} arm={arm} equal=true reference_records=1")
    for geometry, (named, fill) in GEOMETRIES.items():
        for arm in ("baseline_pc", "split", "split_pc") + tuple(arm for arm in arms_of(geometry) if arm in READ_STAGE_CACHE_ARMS):
            for devices in DEVICE_COUNTS:
                lines.append(f"E7RESULT name=e197_compare arm={arm} devices={devices} fill={fill} named={named} equal=true records=1")
    return lines


def verdicts(lines, row, **wanted):
    """合成产物上判出来的、row 对得上且带 wanted 里每个键值的判决行（字段字典）。"""
    found = []
    for line in all_rows(synthetic_artifact(lines)):
        fields = parse_line(line)
        if fields.get("name") == "verdict" and fields.get("row") == row and all(fields.get(k) == v for k, v in wanted.items()):
            found.append(fields)
    return found


def one(lines, row, **wanted):
    found = verdicts(lines, row, **wanted)
    if len(found) != 1:
        raise AssertionError(f"{row} {wanted} 该有 1 行，有 {len(found)} 行")
    return found[0]


def predicted_peak(arm, devices, filled, named):
    return int(predicted_resident_peak(arm, devices, filled, named, SYNTHETIC_SIZES))


def case_empty_cell_is_subtracted():
    lines = synthetic_artifact_lines()
    resident = predicted_peak("baseline", 1, RING_SLOTS, 64)
    peak, reason = subtracted_peak(synthetic_artifact(lines), "baseline", 1, "k64_full")
    assert reason == "ok" and peak == resident + SYNTHETIC_OVERHEAD_BYTES, (peak, resident + SYNTHETIC_OVERHEAD_BYTES, reason)


def case_median_of_five_not_minimum():
    cell = cell_value(synthetic_artifact(synthetic_artifact_lines()), "baseline", 1, "full", 64)
    expected = predicted_peak("baseline", 1, RING_SLOTS, 64) + SYNTHETIC_OVERHEAD_BYTES
    assert cell.median == expected, (cell.median, expected)


def case_line_flags_a_bent_line():
    clean = synthetic_artifact_lines()
    assert one(clean, "jia_line4", geometry="k64_full")["judgment"] == "on_line"
    assert one(clean, "jia_line3", geometry="k64_full")["judgment"] == "on_line"

    def inflate(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline", 4, "full", 64):
            fields["cgroup"] += int(0.15 * (fields["resident"] + SYNTHETIC_OVERHEAD_BYTES))
        return fields

    bent = one(synthetic_artifact_lines(inflate), "jia_line4", geometry="k64_full")
    assert bent["judgment"] == "off_line" and bent["direction"] == "superlinear", bent


def case_pc_not_red_where_it_is_not():
    lines = synthetic_artifact_lines()
    assert one(lines, "jia_pc_absolute", geometry="k64_full", devices="4")["judgment"] == "red"
    assert one(lines, "jia_pc_absolute", geometry="k1_full", devices="4")["judgment"] == "not_red"
    for row in verdicts(lines, "jia_pc_absolute"):
        assert row["self_proof"] == "flips", row


def case_split_slope_and_its_control():
    row = one(synthetic_artifact_lines(), "yi_no_growth", geometry="k64_full")
    assert row["judgment"] == "no_growth" and float(row["split_slope"]) <= SLOPE_LIMIT and float(row["split_pc_slope"]) > 0.5, row
    assert row["self_proof"] == "flips", row


def case_digest_mismatch_is_caught():
    clean = one(synthetic_artifact_lines(), "yi_doable")
    assert clean["judgment"] == "doable_on_the_prototype" and clean["digest_mismatches"] == "0", clean

    def change(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("split", 2, "full", 64):
            fields["digest"] = "other"
        return fields

    caught = one(synthetic_artifact_lines(change), "yi_doable")
    assert caught["judgment"] == "not_doable" and int(caught["digest_mismatches"]) >= 1, caught
    assert clean["compare_lines"] == "52" and clean["compare_mismatches"] == "0" and clean["diff_lines"] == "20", clean
    unequal = [line.replace("equal=true", "equal=false") if "name=e197_compare arm=split devices=3 fill=full named=64" in line else line for line in synthetic_artifact_lines()]
    row = one(unequal, "yi_doable")
    assert row["judgment"] == "not_doable" and row["compare_mismatches"] == "1", row
    fewer = [line for line in synthetic_artifact_lines() if "name=e197_compare arm=read_stage_cache_keeps_zeros_pc" not in line]
    assert one(fewer, "yi_doable")["judgment"] == "incomplete"


def second_round_lines(arm, devices, fill, named, extra):
    """重复号 6–10 的 5 次，memory_peak = 空跑格 + H + 20 MiB，再加 extra(repeat) 字节。"""
    filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
    resident = predicted_peak(arm, devices, filled, named)
    return [synthetic_line(arm, devices, fill, named, repeat, resident,
                           reading={"peak": SYNTHETIC_EMPTY_BYTES + resident + SYNTHETIC_OVERHEAD_BYTES + extra(repeat)}) for repeat in range(6, 11)]


def case_unstable_cell_is_not_judged():
    def spread_in_round_one(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline", 2, "full", 64) and repeat == 5:
            fields["cgroup"] += 100 * MEBIBYTE
        return fields

    lines = synthetic_artifact_lines(spread_in_round_one)
    row = one(lines, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "unstable_needs_rerun", row
    assert ("baseline", 2, "full", 64) in cells_needing_a_rerun(synthetic_artifact(lines))
    steady = one(lines + second_round_lines("baseline", 2, "full", 64, lambda repeat: 1000 * repeat), "jia_line3", geometry="k64_full")
    assert steady["judgment"] == "on_line", steady     # 复测轮 5 次齐了且稳定，改用复测轮
    still = one(lines + second_round_lines("baseline", 2, "full", 64, lambda repeat: 100 * MEBIBYTE * (repeat == 10)), "jia_line3", geometry="k64_full")
    assert still["judgment"] == "not_judged" and still["reason"] == "unstable_after_rerun", still


def case_empty_cell_over_the_limit_voids_the_geometry():
    def heavy_empty(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("empty", 1, "full", 64):
            fields["cgroup"] += 200 * MEBIBYTE
        return fields

    row = one(synthetic_artifact_lines(heavy_empty), "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "empty_cell_over_128_mebibytes", row


def case_a7_outside_is_flagged():
    def inflate(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline", 2, "full", 64):
            fields["resident"] = int(fields["resident"] * 1.2)
        return fields

    assert one(synthetic_artifact_lines(), "A7")["cells_outside"] == "0"
    assert one(synthetic_artifact_lines(inflate), "A7")["cells_outside"] == "1"


def case_f1_triggers_when_the_control_is_not_above():
    def equal_to_baseline(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline_pc", 3, "full", 64):
            fields["cgroup"] -= predicted_peak("baseline_pc", 3, RING_SLOTS, 64) - predicted_peak("baseline", 3, RING_SLOTS, 64)
        return fields

    assert one(synthetic_artifact_lines(), "F1")["triggered"] == "no"
    assert one(synthetic_artifact_lines(equal_to_baseline), "F1")["triggered"] == "yes"


def case_measurement_window_and_clause_numbers():
    lines = synthetic_artifact_lines()
    assert one(lines, "F5")["triggered"] == "no"
    assert one(lines, "F3")["triggered"] == "no" and one(lines, "F4")["triggered"] == "no" and one(lines, "F2")["triggered"] == "no"

    def wrong_window(arm, devices, fill, named, repeat, fields):
        if arm == "measurement_pc":
            fields["resident"] = 100 * MEBIBYTE
        return fields

    assert one(synthetic_artifact_lines(wrong_window), "F5")["triggered"] == "yes"
    # H 的窗是 [254 MiB, 260 MiB]：窗内两端不触发，窗外 1 字节触发（第七节修订 A8、第十节修订 F5）
    for resident, wanted in ((MEASUREMENT_RESIDENT_WINDOW[0], "no"), (MEASUREMENT_RESIDENT_WINDOW[1], "no"),
                             (MEASUREMENT_RESIDENT_WINDOW[0] - 1, "yes"), (MEASUREMENT_RESIDENT_WINDOW[1] + 1, "yes")):
        edge = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields, resident=resident:
                                        dict(fields, resident=resident) if arm == "measurement_pc" else fields)
        assert one(edge, "F5")["triggered"] == wanted, (resident, one(edge, "F5"))


def case_missing_reading_is_not_run_and_not_zero():
    def drop_one(arm, devices, fill, named, repeat, fields):
        return None if (arm, devices, fill, named, repeat) == ("baseline", 1, "full", 64, 3) else fields

    lines = synthetic_artifact_lines(drop_one)
    row = one(lines, "jia_line4", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row
    assert one(lines, "F1")["triggered"] == "not_judged"


def case_prediction_reproduces_the_registered_numbers():
    # 登记修订三第四节 resident_prediction.py 输出的 resident 各行（记录 320、点名项 64），单位 GiB，容差 0.002
    wanted = [("baseline", 1, RING_SLOTS, 64, 0.937), ("baseline", 2, RING_SLOTS, 64, 1.751), ("baseline", 3, RING_SLOTS, 64, 2.566),
              ("baseline", 4, RING_SLOTS, 64, 3.380), ("baseline_pc", 4, RING_SLOTS, 64, 4.133), ("split", 2, RING_SLOTS, 64, 1.693),
              ("split", 4, RING_SLOTS, 64, 1.699), ("split_pc", 4, RING_SLOTS, 64, 3.380), ("baseline", 4, RING_SLOTS // 2, 64, 1.690),
              ("split", 4, RING_SLOTS // 2, 64, 0.849), ("baseline", 4, RING_SLOTS, 1, 0.427), ("split", 4, RING_SLOTS, 1, 0.222)]
    for arm, devices, filled, named, gibibytes in wanted:
        predicted = predicted_resident_peak(arm, devices, filled, named, SYNTHETIC_SIZES) / GIBIBYTE
        assert abs(predicted - gibibytes) < 0.002, (arm, devices, filled, named, predicted, gibibytes)
    # 经读缓存的臂：胶水那一截同基线，另加缓存那一截（4 盘：K = 64 共 6.460 GiB、K = 1 共 3.507 GiB）
    for named, gibibytes in ((64, 6.460), (1, 3.507)):
        total = (predicted_resident_peak("read_stage_cache", 4, RING_SLOTS, named, SYNTHETIC_SIZES) + cache_bytes(4 * RING_SLOTS)) / GIBIBYTE
        assert abs(total - gibibytes) < 0.002, (named, total, gibibytes)
    for named in (64, 1):
        for devices in (1, 2, 3, 4):   # 常驻页口径下 split_pc 与 baseline 同式；split 在 1 盘上同 baseline
            assert predicted_resident_peak("split_pc", devices, RING_SLOTS, named, SYNTHETIC_SIZES) == predicted_resident_peak("baseline", devices, RING_SLOTS, named, SYNTHETIC_SIZES)
        assert predicted_resident_peak("split", 1, RING_SLOTS, named, SYNTHETIC_SIZES) == predicted_resident_peak("baseline", 1, RING_SLOTS, named, SYNTHETIC_SIZES)
    # 修订三第七节 A11、A12：每项 chunk(4096) + chunk(544) ÷ 6 = 4112 + 560 ÷ 6 = 4205.333 字节
    assert abs(CACHE_ENTRY_BYTES - (4112 + 560 / 6)) < 0.001 and abs(cache_bytes(RING_SLOTS) / GIBIBYTE - 0.770) < 0.001
    # 修订三第四节 trajectory_last_device_end 各行，单位 MiB，容差 0.05
    for devices, filled, fill, named, mebibytes in ((1, RING_SLOTS, "full", 64, 832.5), (4, RING_SLOTS, "full", 64, 3325.5),
                                                    (1, RING_SLOTS // 2, "half", 64, 417.0), (4, RING_SLOTS // 2, "half", 64, 1663.5),
                                                    (1, RING_SLOTS, "full", 1, 76.5), (4, RING_SLOTS, "full", 1, 301.5)):
        expected = expected_last_device_end(devices, filled, fill, RING_SLOTS, named, SYNTHETIC_SIZES) / MEBIBYTE
        assert abs(expected - mebibytes) < 0.05, (devices, filled, fill, named, expected, mebibytes)


def scale_cells(factor, selected):
    """selected(arm, devices, fill, named) 命中的格，memory_peak 减空跑格的那一截乘 factor。"""
    def mutate(arm, devices, fill, named, repeat, fields):
        if selected(arm, devices, fill, named):
            fields["cgroup"] = int(SYNTHETIC_EMPTY_BYTES + (fields["cgroup"] - SYNTHETIC_EMPTY_BYTES) * factor)
        return fields
    return mutate


def case_ratio_rows_tell_a_varying_ratio_from_a_steady_one():
    clean = synthetic_artifact_lines()
    assert one(clean, "jia_slot_fill")["judgment"] == "ratio_does_not_vary"
    assert one(clean, "jia_named")["judgment"] == "ratio_varies"
    bent = synthetic_artifact_lines(scale_cells(1.4, lambda arm, devices, fill, named: (arm, devices, fill) == ("baseline", 4, "half")))
    row = one(bent, "jia_slot_fill")
    assert row["judgment"] == "ratio_varies" and float(row["largest_difference"]) > RATIO_LIMIT, row
    assert float(one(clean, "jia_pc_self_ratio", geometry="k64_full", devices="4")["pc_ratio_over_baseline_ratio"]) < 1.0
    # 几何敏感性：常驻页口径下基线随盘数是一条直线加截距，三个几何的 3 盘偏离都推在 0（登记修订三第四节第 1 条），都在 5% 之内；弯的线由 line-flags-a-bent-line 查
    assert one(clean, "jia_line3", geometry="k1_full")["judgment"] == "on_line"
    assert one(clean, "jia_line3", geometry="k64_half")["judgment"] == "on_line"


def case_split_rows_and_f2():
    clean = synthetic_artifact_lines()
    assert one(clean, "yi_close_to_one", geometry="k64_full")["judgment"] == "not_close_to_one"
    assert one(clean, "yi_close_to_one", geometry="k1_full")["judgment"] == "close_to_one"
    assert one(clean, "F2")["triggered"] == "no"

    def flat(arm, devices, fill, named, repeat, fields):
        if arm != "split_pc":
            return fields
        resident = predicted_peak("split", devices, RING_SLOTS if fill == "full" else RING_SLOTS // 2, named)
        return dict(fields, resident=resident, cgroup=SYNTHETIC_EMPTY_BYTES + SYNTHETIC_OVERHEAD_BYTES + 1000 * repeat + resident)

    assert one(synthetic_artifact_lines(flat), "F2")["triggered"] == "yes"   # 逐盘不释放的对照若和 split 一样平：测量分不出逐盘释放


def case_clause_numbers_trigger_f3_and_f4():
    off = synthetic_artifact_lines(scale_cells(1.6, lambda arm, devices, fill, named: (arm, devices, fill, named) == ("baseline", 2, "full", 64)))
    assert one(off, "F3")["triggered"] == "yes" and "2" in one(off, "F3")["outside_devices"].split(","), one(off, "F3")
    assert one(off, "F4")["triggered"] == "yes", one(off, "F4")


def case_a_run_whose_anchors_fail_is_not_used():
    lines = synthetic_artifact_lines()
    spoiled = [line.replace("records_returned=196608", "records_returned=196607") if "arm=baseline devices=1 fill=full named=64 repeat=2 " in line else line for line in lines]
    assert spoiled != lines
    row = one(spoiled, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row      # A3 不符：那一格作废，不当 0
    other_ring = [line.replace("ring_slots=196608", "ring_slots=100") if "arm=baseline devices=2 fill=full named=64 repeat=1 " in line else line for line in lines]
    assert one(other_ring, "jia_line3", geometry="k64_full")["reason"] == "not_run"     # A1 不符：生产格的环槽数不是 196 608
    cache_off = [line.replace("cache_entries=393216 ", "cache_entries=196608 ") if "arm=read_stage_cache devices=2 fill=full named=64 repeat=4 " in line else line for line in lines]
    assert cache_off != lines and one(cache_off, "bing_line4", geometry="k64_full")["reason"] == "not_run"   # A10 不符：那一次作废
    assert one(cache_off, "A10")["runs_outside"] == "1" and one(lines, "A10")["runs_outside"] == "0"


def edit_trajectory(lines, cell_prefix, edit):
    """cell_prefix 点名的那一格（5 次）每一行的轨迹字段，按 edit(数字列表) 改成新的数字列表。"""
    edited = []
    for line in lines:
        if cell_prefix in line:
            match = re.search(rf"{TRAJECTORY_FIELD}=(\S+)", line)
            numbers = edit([int(word) for word in match.group(1).split(",")])
            line = line[:match.start(1)] + ",".join(map(str, numbers)) + line[match.end(1):]
        edited.append(line)
    return edited


def case_s3_a9_and_trajectory_are_flagged():
    clean = synthetic_artifact_lines()
    assert one(clean, "S3_two_measures")["cells_outside"] == "0" and one(clean, "A9")["geometries_outside"] == "0"
    assert one(clean, "trajectory")["cells_outside"] == "0"
    big_gap = synthetic_artifact_lines(scale_cells(1.5, lambda arm, devices, fill, named: (arm, devices, fill, named) == ("baseline", 3, "full", 64)))
    assert int(one(big_gap, "S3_two_measures")["cells_outside"]) >= 1      # P 比 H 大得多（P > 1.15 × H + 64 MiB）：两条量法互不包含
    small_gap = synthetic_artifact_lines(scale_cells(0.5, lambda arm, devices, fill, named: (arm, devices, fill, named) == ("baseline", 3, "full", 64)))
    assert int(one(small_gap, "S3_two_measures")["cells_outside"]) >= 1    # H 比 P 大得多（H > P + 32 MiB）：两条量法互不包含
    split_drift = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(fields, resident=int(fields["resident"] * 1.03))
                                           if (arm, devices, fill, named) == ("split", 1, "full", 64) else fields)
    assert one(split_drift, "A9")["geometries_outside"] == "1"                # 1 盘上 split 与 baseline 的 H 差 3% > 1%
    # 轨迹两条断言各查各的（修订三第八节）：最后一个值离「读到那一刻的常驻页」超过 3%；H 小于轨迹里最大的那个数
    cell = "arm=baseline devices=2 fill=full named=64 repeat="
    last_off = edit_trajectory(clean, cell, lambda numbers: numbers[:-1] + [int(numbers[-1] * 1.1)])
    assert int(one(last_off, "trajectory")["cells_outside"]) == 1
    above_the_peak = edit_trajectory(clean, cell, lambda numbers: [predicted_peak("baseline", 2, RING_SLOTS, 64) + 1] + numbers[1:])
    assert int(one(above_the_peak, "trajectory")["cells_outside"]) == 1
    fewer_points = edit_trajectory(clean, cell, lambda numbers: numbers[-1:])
    assert int(one(fewer_points, "trajectory")["cells_outside"]) == 1       # 轨迹点数不是盘数


def case_cache_anchors_and_s5():
    clean = synthetic_artifact_lines()
    assert one(clean, "A11")["cells_outside"] == "0" and one(clean, "A12")["cells_outside"] == "0" and one(clean, "S5")["stop"] == "no"
    assert one(clean, "A11")["cells"] == "16"     # read_stage_cache 三个几何 × 4 盘 + 阳性对照 K = 64 半满 × 4 盘
    shrunk = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(fields, resident=fields["resident"] - 300 * MEBIBYTE)
                                      if (arm, devices, fill, named) == ("read_stage_cache", 4, "full", 64) else fields)
    assert one(shrunk, "A11")["cells_outside"] == "1" and one(shrunk, "S5")["stop"] == "yes"


def only_cell(selected_arm, selected_devices, selected_repeat, change):
    """把 (臂, 盘数, 重复号) 那一次（K = 64 满槽）的字段按 change 改掉。"""
    def mutate(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named, repeat) == (selected_arm, selected_devices, "full", 64, selected_repeat):
            fields = change(dict(fields))
        return fields
    return mutate


def case_an_attempt_near_its_cap_is_not_used():
    lines = synthetic_artifact_lines(only_cell("baseline", 2, 3, lambda fields: dict(fields, cap=fields["cgroup"] + 32 * MEBIBYTE)))
    row = one(lines, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row      # V6：顶着上限的那一次不算
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(lines)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("v6_near_the_cap") == "1", summary


def case_a_result_line_is_pinned_to_a_succeeded_attempt():
    clean = synthetic_artifact_lines()
    assert one(clean, "jia_line3", geometry="k64_full")["judgment"] == "on_line"
    for label, change in (("outcome", lambda fields: dict(fields, outcome="memory-cap-hit")),
                          ("peak", lambda fields: dict(fields, peak_missing=True)),
                          ("pinned", lambda fields: dict(fields, pinned=False))):
        row = one(synthetic_artifact_lines(only_cell("baseline", 2, 3, change)), "jia_line3", geometry="k64_full")
        assert row["judgment"] == "not_judged" and row["reason"] == "not_run", (label, row)     # V7：钉不到成了的那一次，不当 0
    claimed = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(fields, task=777)
                                       if (arm, devices, fill, named) == ("baseline", 2, "full", 64) and repeat in (2, 3) else fields)
    assert one(claimed, "jia_line3", geometry="k64_full")["reason"] == "not_run"     # 同一次尝试被两行认领，两行都作废
    doubled = clean + [line for line in clean if "arm=baseline devices=2 fill=full named=64 repeat=3 " in line]
    assert one(doubled, "jia_line3", geometry="k64_full")["reason"] == "not_run"     # 修订二第 5 条：同一格两行，两行都作废
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(doubled)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("duplicate_cell") == "1", summary


def case_empty_cell_is_subtracted_per_host():
    def on_the_peer(arm, devices, fill, named, repeat, fields):
        if (devices, fill, named) == (2, "full", 64) and arm in ("empty", "baseline") and repeat in (4, 5):
            fields = dict(fields, host="peer", cgroup=fields["cgroup"] + 60 * MEBIBYTE)
        return fields

    artifact = synthetic_artifact(synthetic_artifact_lines(on_the_peer))
    peak, reason = subtracted_peak(artifact, "baseline", 2, "k64_full")
    expected = predicted_peak("baseline", 2, RING_SLOTS, 64) + SYNTHETIC_OVERHEAD_BYTES
    assert reason == "ok" and peak == expected, (peak, expected, reason)     # 两台各扣各的空跑格（第六节修订第 5 条）
    between = one(synthetic_artifact_lines(on_the_peer), "empty_cell_between_hosts")
    assert between["geometries"] == "1" and between["local_minus_peer"].startswith("k64_full_n2:-6291"), between
    lonely = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(fields, host="peer")
                                      if (arm, devices, fill, named, repeat) == ("baseline", 3, "full", 64, 1) else fields)
    assert one(lonely, "jia_line3", geometry="k64_full")["reason"] == "v8_no_empty_cell_on_this_host"   # V8：第二台上没有空跑格，扣不了


def case_a_run_from_another_binary_is_not_used():
    # 用例里的 binary 字段经 synthetic_line 写成产物行的 binary_sha256=；其余字段与基线一样，所以这一次只可能被 V9 作废。
    # 判决行没被作废时没有 reason 字段，所以先断 judgment，再断 reason；作废原因的计数从 attempt_summary 读（v9_binary），不靠 not_run 一个值。
    lines = synthetic_artifact_lines(only_cell("baseline", 2, 3, lambda fields: dict(fields, binary="other-binary")))
    row = one(lines, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row.get("reason") == "not_run", row     # V9：与 checks 记的二进制不同的那一次作废
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(lines)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("v9_binary") == "1" and summary.get("voided") == "1", summary
    # 去掉 checks 那一行：没有可比的二进制，每一次都作废（V9 里 checks_binary 为空的那一支）
    without_checks = [line for line in synthetic_artifact_lines() if line != SYNTHETIC_CHECKS_LINE]
    row = one(without_checks, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row.get("reason") == "not_run", row
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(without_checks)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("v9_binary") == summary.get("result_lines") == summary.get("voided") and summary.get("pinned") == "0", summary


def case_bing_rows_report_indistinguishable_across_the_threshold():
    clean = synthetic_artifact_lines()
    for geometry in GEOMETRIES:
        assert one(clean, "bing_line4", geometry=geometry)["judgment"] == "on_line"
        assert one(clean, "bing_line3", geometry=geometry)["judgment"] == "on_line"
    assert one(clean, "bing_slot_fill")["judgment"] == "ratio_does_not_vary" and one(clean, "bing_named")["judgment"] == "ratio_does_not_vary"
    assert all(row["judgment"] == "red" for row in verdicts(clean, "bing_pc_absolute")) and len(verdicts(clean, "bing_pc_absolute")) == 4
    assert one(clean, "F6")["triggered"] == "no" and one(clean, "F7")["triggered"] == "no" and one(clean, "F8")["triggered"] == "no"
    assert abs(float(one(clean, "bing_ratios", geometry="k64_full")["q_r4"]) - 4.0) < 0.001
    spread = synthetic_artifact_lines(only_cell("read_stage_cache", 2, 5, lambda fields: dict(fields, cgroup=fields["cgroup"] + 60 * MEBIBYTE)))
    row = one(spread, "bing_line4", geometry="k64_full")
    assert row["judgment"] == "indistinguishable" and abs(float(row["deviation"])) < LINE_LIMIT, row   # 中位数在门槛里、区间跨门槛：分不出
    control_like = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(
        fields, cgroup=fields["cgroup"] - int(cache_bytes(devices * RING_SLOTS // 2)))
        if arm == "read_stage_cache_keeps_zeros_pc" else fields)
    assert one(control_like, "F6")["triggered"] == "yes"     # 阳性对照与臂一样多：分不出「连全 0 也收」，丙各行作废


def case_an_unreadable_window_voids_the_run_and_is_not_read_as_zero():
    clean = synthetic_artifact_lines()
    assert one(clean, "jia_line3", geometry="k64_full")["judgment"] == "on_line"
    # 装置写不进 clear_refs 或读不出 /proc/self/status：量窗那几个字段全是 unreadable，那一次作废（V11），这一格缺一次、不当 0
    lines = synthetic_artifact_lines(only_cell("baseline", 2, 3, lambda fields: dict(fields, unreadable=True)))
    row = one(lines, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(lines)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("v11_window_unreadable") == "1" and summary.get("voided") == "1", summary
    # 只有重置那一项不是 ok、别的字段都是数：同样作废；轨迹里有一项读不出：同样作废
    cell = "arm=baseline devices=2 fill=full named=64 repeat=3 "
    not_reset = [line.replace("high_water_mark_reset=ok", "high_water_mark_reset=unreadable") if cell in line else line for line in clean]
    assert one(not_reset, "jia_line3", geometry="k64_full")["reason"] == "not_run"
    gap_in_trajectory = edit_trajectory_text(clean, cell, "unreadable,")
    assert one(gap_in_trajectory, "jia_line3", geometry="k64_full")["reason"] == "not_run"
    summary = parse_line(next(line for line in all_rows(synthetic_artifact(not_reset)) if line.startswith("E7RESULT name=attempt_summary ")))
    assert summary.get("v11_window_unreadable") == "1", summary


def edit_trajectory_text(lines, cell_prefix, prefix_text):
    """cell_prefix 点名的那一次，轨迹字段前面接上 prefix_text（造出读不出的一项）。"""
    return [line.replace(f"{TRAJECTORY_FIELD}=", f"{TRAJECTORY_FIELD}={prefix_text}") if cell_prefix in line else line for line in lines]


SELFTEST_CASES = (
    ("prediction-reproduces-the-registered-numbers", case_prediction_reproduces_the_registered_numbers),
    ("empty-cell-is-subtracted", case_empty_cell_is_subtracted),
    ("median-of-five-not-minimum", case_median_of_five_not_minimum),
    ("line-flags-a-bent-line", case_line_flags_a_bent_line),
    ("pc-not-red-where-it-is-not", case_pc_not_red_where_it_is_not),
    ("split-slope-and-its-control", case_split_slope_and_its_control),
    ("digest-mismatch-is-caught", case_digest_mismatch_is_caught),
    ("unstable-cell-is-not-judged", case_unstable_cell_is_not_judged),
    ("empty-cell-over-the-limit-voids-the-geometry", case_empty_cell_over_the_limit_voids_the_geometry),
    ("a7-outside-is-flagged", case_a7_outside_is_flagged),
    ("f1-triggers-when-the-control-is-not-above", case_f1_triggers_when_the_control_is_not_above),
    ("measurement-window-and-clause-numbers", case_measurement_window_and_clause_numbers),
    ("missing-reading-is-not-run-and-not-zero", case_missing_reading_is_not_run_and_not_zero),
    ("ratio-rows-tell-a-varying-ratio-from-a-steady-one", case_ratio_rows_tell_a_varying_ratio_from_a_steady_one),
    ("split-rows-and-f2", case_split_rows_and_f2),
    ("clause-numbers-trigger-f3-and-f4", case_clause_numbers_trigger_f3_and_f4),
    ("a-run-whose-anchors-fail-is-not-used", case_a_run_whose_anchors_fail_is_not_used),
    ("s3-a9-and-trajectory-are-flagged", case_s3_a9_and_trajectory_are_flagged),
    ("cache-anchors-and-s5", case_cache_anchors_and_s5),
    ("an-attempt-near-its-cap-is-not-used", case_an_attempt_near_its_cap_is_not_used),
    ("a-result-line-is-pinned-to-a-succeeded-attempt", case_a_result_line_is_pinned_to_a_succeeded_attempt),
    ("empty-cell-is-subtracted-per-host", case_empty_cell_is_subtracted_per_host),
    ("a-run-from-another-binary-is-not-used", case_a_run_from_another_binary_is_not_used),
    ("bing-rows-report-indistinguishable-across-the-threshold", case_bing_rows_report_indistinguishable_across_the_threshold),
    ("an-unreadable-window-voids-the-run-and-is-not-read-as-zero", case_an_unreadable_window_voids_the_run_and_is_not_read_as_zero),
)
BREAK_TO_CASE = {
    "line-ignores-the-sign": "line-flags-a-bent-line",
    "pc-red-threshold-ignored": "pc-not-red-where-it-is-not",
    "slope-uses-the-top-point-only": "split-slope-and-its-control",
    "median-replaced-by-minimum": "median-of-five-not-minimum",
    "empty-cell-not-subtracted": "empty-cell-is-subtracted",
    "digest-mismatch-ignored": "digest-mismatch-is-caught",
    "cap-distance-ignored": "an-attempt-near-its-cap-is-not-used",
    "attempt-not-pinned": "a-result-line-is-pinned-to-a-succeeded-attempt",
    "empty-cell-from-any-host": "empty-cell-is-subtracted-per-host",
    "binary-digest-ignored": "a-run-from-another-binary-is-not-used",
    "interval-ignored": "bing-rows-report-indistinguishable-across-the-threshold",
    "unreadable-window-read-as-zero": "an-unreadable-window-voids-the-run-and-is-not-read-as-zero",
}


def selftest():
    failed = 0
    for label, case in SELFTEST_CASES:
        try:
            case()
            print(f"  ✓ 自检 {label}")
        except AssertionError as error:
            failed += 1
            print(f"  ✗ 自检 {label}：{error}")  # gate-lint:detail
    broken_switch = os.environ.get("E197_REPORT_BREAK", "")
    if broken_switch:
        print(f"（弄坏开关 {broken_switch} 生效）", file=sys.stderr)
    if failed:
        print(f"自证红：{failed} 项没过，共 {len(SELFTEST_CASES)} 项（→ 怎么办：按上面 ✗ 行的项名改 report.py 里对应的判法，E197_REPORT_BREAK 设着的话本来就该红）")
        return 1
    print(f"自证通过：{len(SELFTEST_CASES)} 项")
    if any(os.environ.get(name) for name in BREAK_VARIABLES):
        return 0   # 弄坏开关生效的这一趟（开关轮起的子进程）只跑各项，不再套开关轮
    return selftest_breaks()


def selftest_breaks():
    """逐个打开弄坏开关起自己的 --selftest，要求退 1 且红在点名的那一项。"""
    bad = 0
    for switch in BREAK_SWITCHES:
        environment = dict(os.environ, E197_REPORT_BREAK=switch)
        result = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "--selftest"], env=environment,
                                capture_output=True, text=True, check=False)
        target = BREAK_TO_CASE[switch]
        if result.returncode == 1 and f"✗ 自检 {target}：" in result.stdout:
            print(f"  ✓ 弄坏开关 {switch} 红在 {target}")
        else:
            bad += 1
            print(f"  ✗ 弄坏开关 {switch} 没红在 {target}（退出码 {result.returncode}）")  # gate-lint:detail
    print(f"弄坏开关自证{'通过' if not bad else '红'}：{len(BREAK_SWITCHES)} 个开关，{bad} 个没红在点名的项"
          + ("（→ 怎么办：对着没红的开关核 report.py 里那条判法有没有被开关真的弄坏、BREAK_TO_CASE 点名的项对不对）" if bad else ""))
    return 1 if bad else 0


BREAK_VARIABLES = ("E197_REPORT_BREAK",)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行的第一个参数不是 --selftest 而 BREAK_VARIABLES 里有一个非空（不管值是什么），
    退 2 并点名它只在自证里认；在参数解析之前判。"""
    for name in BREAK_VARIABLES:
        if os.environ.get(name) and arguments[:1] != ["--selftest"]:
            print(f"  ✗ {name} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
            print(f"     → 怎么办：去掉 {name} 再跑；要证红就跑 {name}=<项> python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py --selftest", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    sys.exit(main(sys.argv[1:]))
