#!/usr/bin/env python3
# admission: always 每一次调都读此刻点名的产物现判，上一次的结论不替这一次作保
# run-condition: none 只读产物文本，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置
"""E197（扫环胶水内存随盘数）出表脚本：读驱动 drive.py 写的产物，按跑前登记 research/prompts/e197-preregistration.md 第六、七、八、十、十一节
逐行判，每行一条 `E7RESULT name=verdict ...`。独立手写，不 import crates/ 任何东西；门槛跑前写死在下面的常量里。

用法：
  report.py <产物 .out>                打到标准输出（读不动的格列出来、不当 0）
  report.py <产物 .out> --append       同样的行追加到产物末尾，再接收尾行 `E7RESULT name=done emitted=N`（N 是整份产物的 E7RESULT 行数，含这一行）
  report.py <产物 .out> --unstable     只列需要重跑一轮 5 次的格（第一轮散布 > 2% 而第二轮还没跑），一格一行 `arm devices fill named`
  report.py --selftest                 合成产物上的自证（每一项标签就是下面「弄坏开关」点名要红的那一格）
  report.py --selftest-breaks          逐个打开弄坏开关起自己，要求每个开关都红在点名的那一项
退出码：0 判完；1 自证红；2 用法或产物读不了；3 产物里装置版本或几何不符（整份不判）。

弄坏开关（只给自证用，生效时往 stderr 打一行；环境变量 E197_REPORT_BREAK=<一个值>，不认识的值直接拒绝）：
  E197_REPORT_BREAK=line-ignores-the-sign       甲-直线的偏离取绝对值之前先把负号去掉的位置错了：改成偏离恒取 0
      自证项 「✗ 自检 line-flags-a-bent-line：」
      「✗ 自检 ratio-rows-tell-a-varying-ratio-from-a-steady-one：」
  E197_REPORT_BREAK=pc-red-threshold-ignored    阳性对照红不红不看 1.10 的门槛，恒报红
      自证项 「✗ 自检 pc-not-red-where-it-is-not：」
  E197_REPORT_BREAK=slope-uses-the-top-point-only  乙-不涨的斜率只用 4 盘那一点除以 2 倍单盘（不减 2 盘那一点）
      自证项 「✗ 自检 split-slope-and-its-control：」
      「✗ 自检 split-rows-and-f2：」
  E197_REPORT_BREAK=median-replaced-by-minimum  一格的 cgroup 峰值取最小值不取中位数
      自证项 「✗ 自检 median-of-five-not-minimum：」
  E197_REPORT_BREAK=empty-cell-not-subtracted   不扣空跑格
      自证项 「✗ 自检 empty-cell-is-subtracted：」
      「✗ 自检 ratio-rows-tell-a-varying-ratio-from-a-steady-one：」
  E197_REPORT_BREAK=digest-mismatch-ignored     交回的表摘要对不上也报在原型上做得到
      自证项 「✗ 自检 digest-mismatch-is-caught：」
"""
import os
import re
import statistics
import subprocess
import sys

MEBIBYTE = 1 << 20
GIBIBYTE = 1 << 30

# ---- 跑前写死的门槛（登记第六、七、十、十一节）----
LINE_LIMIT = 0.05            # 甲-直线 |dev| ≤ 5%
RATIO_LIMIT = 0.15           # 甲-槽填满、甲-点名项 ≤ 0.15
PC_RED_FACTOR = 1.10         # 阳性对照红 ⟺ 比基线高过 10%
CLOSE_TO_ONE_LIMIT = 1.25    # 乙-接近一 r_split(4) ≤ 1.25
SLOPE_LIMIT = 0.10           # 乙-不涨 slope ≤ 0.10
SPREAD_LIMIT = 0.02          # V2 一格 5 次 cgroup 峰值 (最大−最小)÷中位数 ≤ 2%
EMPTY_CELL_LIMIT = 128 * MEBIBYTE   # V3 空跑格 cgroup 峰值
PREDICTION_TOLERANCE = 0.05  # A7
SAME_ONE_DEVICE_ALLOCATOR_LIMIT = 0.01   # A9 分配器峰值之差
SAME_ONE_DEVICE_CGROUP_LIMIT = 0.03      # A9 P 之差
TWO_MEASURES_ABSOLUTE_SLACK = 32 * MEBIBYTE         # S3 分配器 > P + 32 MiB
TWO_MEASURES_RELATIVE = (1.15, 64 * MEBIBYTE)       # S3 P > 1.15 × 分配器 + 64 MiB
CLAUSE_PAIR_BYTES = (1.86e9, 3.63e9)                # C2 条款推算（两盘、四盘）
CLAUSE_TOLERANCE = 0.25
F4_RATIO_RANGES = {2: (1.6, 2.2), 4: (3.0, 4.4)}
MEASUREMENT_BYTES = 256 * MEBIBYTE
MEASUREMENT_ALLOCATOR_SLACK = 65536
MEASUREMENT_CGROUP_HIGH = 288 * MEBIBYTE
REPEATS_PER_ROUND = 5
DIFFERENTIAL_LINES = 12      # 差分集四例 × 三份胶水臂
COMPARE_LINES = 36           # 三份胶水臂 × 盘数 1–4 × 三个几何，各一格
RING_SLOTS = 196608
DEVICE_COUNTS = (1, 2, 3, 4)
SCANNING_ARMS = ("baseline", "baseline_pc", "split", "split_pc")
GEOMETRIES = {"k64_full": (64, "full"), "k64_half": (64, "half"), "k1_full": (1, "full")}
BREAK_SWITCHES = (
    "line-ignores-the-sign", "pc-red-threshold-ignored", "slope-uses-the-top-point-only",
    "median-replaced-by-minimum", "empty-cell-not-subtracted", "digest-mismatch-ignored",
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


def capacity_after_pushes(count):
    capacity = 4
    while capacity < count:
        capacity *= 2
    return capacity


def predicted_allocator_peak(arm, devices, filled, named, sizes):
    """登记第四节 predict.py 的式子（A7），按这一行里量到的 size_of 代入；与装置里的 predicted_allocator_peak_bytes 同一份算术。"""
    record, unit = sizes["journal_record"], sizes["named_unit"]
    leaf_of_records = 8 + 2 + 2 + 4 + 11 * 16 + 11 * record
    leaf_of_positions = 8 + 2 + 2 + 4 + 11 * 16 + 11 * 8
    tree_of_records = filled * leaf_of_records / 6
    tree_of_positions = filled * leaf_of_positions / 6

    def base(count):
        read = count * filled
        return (capacity_after_pushes(read) * record + read * named * unit + read * 16
                + tree_of_positions + tree_of_records)

    if arm == "baseline":
        return base(devices)
    if arm == "split_pc":
        # 登记写「同基线」；装置的 split_pc 另留一个重复记录 Vec，最后一盘那一遍的 Vec 也还在，代替基线那一个总 Vec 的容量（与 Rust 里的推算同一式）
        duplicates = (devices - 1) * filled
        return (tree_of_records + devices * filled * named_per_record_bytes(named, unit)
                + (capacity_after_pushes(duplicates) if duplicates else 0) * record
                + capacity_after_pushes(filled) * record + devices * filled * 16 + tree_of_positions)
    if arm == "baseline_pc":
        return base(devices) + filled * named * unit
    if arm == "split":
        if devices == 1:
            return base(1)
        return (tree_of_records + filled * named * unit + capacity_after_pushes(filled) * record
                + filled * named * unit + devices * filled * 16 + tree_of_positions)
    return 0.0


def named_per_record_bytes(named, unit):
    return named * unit


def median(values):
    return min(values) if broken("median-replaced-by-minimum") else statistics.median(values)


# ---- 产物模型 ----

class Artifact:
    """驱动写的产物：每个装置结果行（name=e197）配一条同键的 cgroup 行（name=e197_cgroup）；差分集、检查、S1 核对各按行收着。"""

    def __init__(self, lines):
        self.runs = {}        # (arm, devices, fill, named, repeat) → 装置结果行字段
        self.cgroups = {}     # 同一个键 → cgroup 峰值字节
        self.diff = []
        self.compare = []
        self.checks = []
        self.s1 = []
        self.sizes_seen = set()
        for line in lines:
            fields = parse_line(line)
            if fields is None:
                continue
            name = fields.get("name")
            if name == "e197":
                key = (fields["arm"], int(fields["devices"]), fields["fill"], int(fields["named"]), int(fields["repeat"]))
                self.runs[key] = fields
                self.sizes_seen.add(tuple(fields.get(f"size_{field}") for field in
                                          ("location_entry", "named_unit", "node_pointer", "journal_record")))
            elif name == "e197_cgroup":
                arm, devices, fill, named, repeat = fields["key"].split("/")[1:]
                if fields.get("cgroup_peak_bytes", "").isdigit():
                    self.cgroups[(arm, int(devices), fill, int(named), int(repeat))] = int(fields["cgroup_peak_bytes"])
            elif name == "e197_diff":
                self.diff.append(fields)
            elif name == "e197_compare":
                self.compare.append(fields)
            elif name in ("e197_checks", "e197_sizes"):
                self.checks.append(fields)
            elif name == "e197_s1":
                self.s1.append(fields)


def sizes_of(fields):
    return {name: int(fields[f"size_{name}"]) for name in ("location_entry", "named_unit", "node_pointer", "journal_record")}


def run_is_complete(artifact, key):
    """V4、V5：结果行在、字段齐、cgroup 行在；扫环格的 A2、A3 对得上；生产格的环槽数是 196 608（A1）。读不到的不当 0。"""
    fields = artifact.runs.get(key)
    if fields is None or key not in artifact.cgroups:
        return False
    needed = ("ring_slots", "reads", "reads_expected", "records_returned", "records_expected",
              "allocator_peak_over_start_bytes", "allocator_live_after_return_bytes", "vm_hwm_kib", "table_digest")
    if any(name not in fields for name in needed):
        return False
    if int(fields["ring_slots"]) != RING_SLOTS or not fields["vm_hwm_kib"].isdigit():
        return False
    return fields["reads"] == fields["reads_expected"] and fields["records_returned"] == fields["records_expected"]


class CellValue:
    """一格（臂、盘数、填满档、点名项数）5 次里最近一轮的读数。"""

    def __init__(self, keys, artifact, round_number):
        self.round_number = round_number
        self.cgroup = [artifact.cgroups[key] for key in keys]
        self.runs = [artifact.runs[key] for key in keys]
        self.median = median(self.cgroup)
        spread = (max(self.cgroup) - min(self.cgroup)) / statistics.median(self.cgroup)
        self.spread = spread
        self.stable = spread <= SPREAD_LIMIT
        self.allocator_peak = statistics.median(int(run["allocator_peak_over_start_bytes"]) for run in self.runs)
        self.digests = {run["table_digest"] for run in self.runs}


def cell_value(artifact, arm, devices, fill, named):
    """最近一轮完整的 5 次：第二轮（重复号 6–10）齐了用第二轮，否则第一轮（1–5）；都不齐交 None（本次未跑）。"""
    for round_number in (2, 1):
        first = REPEATS_PER_ROUND * (round_number - 1) + 1
        keys = [(arm, devices, fill, named, repeat) for repeat in range(first, first + REPEATS_PER_ROUND)]
        if all(run_is_complete(artifact, key) for key in keys):
            return CellValue(keys, artifact, round_number)
    return None


def subtracted_peak(artifact, arm, devices, geometry):
    """判的那一个数 P = 这一格 cgroup 峰值中位数 − 同 (盘数, 填满档, 点名项数) 空跑格的中位数。
    交 (P, 说明)：空跑格缺、空跑格 > 128 MiB（V3）、这一格缺、这一格 5 次散布超 2% 且已是第二轮（V2）都交 (None, 原因)。"""
    named, fill = GEOMETRIES[geometry]
    cell = cell_value(artifact, arm, devices, fill, named)
    empty = cell_value(artifact, "empty", devices, fill, named)
    if cell is None:
        return None, "not_run"
    if empty is None:
        return None, "empty_cell_not_run"
    if empty.median > EMPTY_CELL_LIMIT:
        return None, "empty_cell_over_128_mebibytes"
    if not cell.stable and cell.round_number == 2:
        return None, "unstable_after_rerun"
    if not cell.stable:
        return None, "unstable_needs_rerun"
    if not empty.stable and empty.round_number == 2:
        return None, "empty_cell_unstable_after_rerun"
    if not empty.stable:
        return None, "empty_cell_unstable_needs_rerun"
    return (cell.median if broken("empty-cell-not-subtracted") else cell.median - empty.median), "ok"


def cells_needing_a_rerun(artifact):
    """第一轮 5 次散布 > 2%（扫环格与空跑格）而第二轮还没有的格，登记第十一节 V2。"""
    found = []
    for named, fill in GEOMETRIES.values():
        for arm in ("empty",) + SCANNING_ARMS:
            for devices in DEVICE_COUNTS:
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
    """乙-做得到：各臂交回的表摘要与基线臂同格的相同（生产格），加差分集 D 的十二行全等。"""
    mismatches, compared = 0, 0
    for named, fill in GEOMETRIES.values():
        for devices in DEVICE_COUNTS:
            digests = {}
            for arm in SCANNING_ARMS:
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
    if broken("digest-mismatch-ignored"):
        mismatches = 0
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
    base, base_reasons = peaks_by_device(artifact, "baseline", "k64_full")
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
    """F5：measurement_pc 的分配器峰值落在 [256 MiB, 256 MiB + 64 KiB]，cgroup 峰值减同几何空跑格落在 [256 MiB, 288 MiB]。"""
    cell = cell_value(artifact, "measurement_pc", 1, "full", 64)
    empty = cell_value(artifact, "empty", 1, "full", 64)
    if cell is None or empty is None:
        return "E7RESULT name=verdict row=F5 triggered=not_judged reason=not_run"
    allocator_bad = not MEASUREMENT_BYTES <= cell.allocator_peak <= MEASUREMENT_BYTES + MEASUREMENT_ALLOCATOR_SLACK
    cgroup_over_empty = cell.median - empty.median
    cgroup_bad = not MEASUREMENT_BYTES <= cgroup_over_empty <= MEASUREMENT_CGROUP_HIGH
    return (f"E7RESULT name=verdict row=F5 triggered={triggered(allocator_bad or cgroup_bad)} allocator_peak_over_start={cell.allocator_peak:.0f} "
            f"cgroup_over_empty={cgroup_over_empty:.0f} allocator_outside_window={triggered(allocator_bad)} cgroup_outside_window={triggered(cgroup_bad)}")


def each_scanning_cell(artifact):
    """(几何, 臂, 盘数, 填满档, 点名项数, 格读数) 逐个交出已有完整 5 次的扫环格。"""
    for geometry, (named, fill) in GEOMETRIES.items():
        for arm in SCANNING_ARMS:
            for devices in DEVICE_COUNTS:
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is not None:
                    yield geometry, arm, devices, fill, named, cell


def anchor_rows(artifact):
    """A7（分配器峰值对推算 ±5%）、A9（1 盘上 split 与 baseline 同一串分配）、S3（两条量法互相包含）、轨迹（登记第七、八、十一节）。"""
    outside_a7, outside_s3, outside_trajectory, cells = [], [], [], 0
    worst = 0.0
    for geometry, arm, devices, fill, named, cell in each_scanning_cell(artifact):
        cells += 1
        run = cell.runs[0]
        filled = int(run["filled_slots"])
        predicted = predicted_allocator_peak(arm, devices, filled, named, sizes_of(run))
        deviation = cell.allocator_peak / predicted - 1
        worst = max(worst, abs(deviation))
        label = f"{geometry}:{arm}:n{devices}"
        if abs(deviation) > PREDICTION_TOLERANCE:
            outside_a7.append(f"{label}:{deviation:+.3f}")
        peak, _ = subtracted_peak(artifact, arm, devices, geometry)
        if peak is not None and (cell.allocator_peak > peak + TWO_MEASURES_ABSOLUTE_SLACK
                                 or peak > TWO_MEASURES_RELATIVE[0] * cell.allocator_peak + TWO_MEASURES_RELATIVE[1]):
            outside_s3.append(label)
        if arm in ("baseline", "baseline_pc"):
            read = devices * filled - (1 if fill == "full" else 0)
            expected = (capacity_after_pushes(read) * sizes_of(run)["journal_record"]
                        + read * named * sizes_of(run)["named_unit"] + int(run["ring_slots"]) * 8)
            ends = [int(value) for value in run["allocator_live_at_each_device_end_bytes"].split(",") if value.lstrip("-").isdigit()]
            if len(ends) != devices or abs(ends[-1] / expected - 1) > 0.03 or cell.allocator_peak < max(ends):
                outside_trajectory.append(label)
    rows = [f"E7RESULT name=verdict row=A7 cells={cells} largest_deviation={worst:.4f} cells_outside={len(outside_a7)} "
            f"outside={','.join(outside_a7) or 'none'}",
            f"E7RESULT name=verdict row=S3_two_measures cells={cells} cells_outside={len(outside_s3)} outside={','.join(outside_s3) or 'none'}",
            f"E7RESULT name=verdict row=trajectory cells={cells} cells_outside={len(outside_trajectory)} outside={','.join(outside_trajectory) or 'none'}"]
    rows.append(same_one_device_row(artifact))
    return rows


def same_one_device_row(artifact):
    """A9：只有一盘时 split 与 baseline 做的是同一串分配，分配器峰值之差 ≤ 1%、P 之差 ≤ 3%。"""
    outside, compared = [], 0
    for geometry, (named, fill) in GEOMETRIES.items():
        base = cell_value(artifact, "baseline", 1, fill, named)
        split = cell_value(artifact, "split", 1, fill, named)
        if base is None or split is None:
            continue
        compared += 1
        allocator_difference = abs(split.allocator_peak / base.allocator_peak - 1)
        base_peak, _ = subtracted_peak(artifact, "baseline", 1, geometry)
        split_peak, _ = subtracted_peak(artifact, "split", 1, geometry)
        cgroup_difference = abs(split_peak / base_peak - 1) if base_peak and split_peak else None
        if allocator_difference > SAME_ONE_DEVICE_ALLOCATOR_LIMIT or cgroup_difference is None or cgroup_difference > SAME_ONE_DEVICE_CGROUP_LIMIT:
            outside.append(geometry)
    return f"E7RESULT name=verdict row=A9 geometries={compared} geometries_outside={len(outside)} outside={','.join(outside) or 'none'}"


def cell_summary_rows(artifact):
    """每格一行（不判）：cgroup 中位数、散布、轮次、扣空跑格后的 P、分配器峰值、VmHWM 中位数、扫环秒数中位数。"""
    lines = []
    for geometry, (named, fill) in GEOMETRIES.items():
        for arm in ("empty",) + SCANNING_ARMS + ("measurement_pc",):
            for devices in DEVICE_COUNTS:
                cell = cell_value(artifact, arm, devices, fill, named)
                if cell is None:
                    continue
                peak, reason = subtracted_peak(artifact, arm, devices, geometry) if arm != "measurement_pc" else (None, "not_applicable")
                hwm = statistics.median(int(run["vm_hwm_kib"]) for run in cell.runs)
                seconds = statistics.median(float(run.get("scan_seconds", "0")) for run in cell.runs)
                lines.append(
                    f"E7RESULT name=cell_summary arm={arm} devices={devices} geometry={geometry} round={cell.round_number} "
                    f"cgroup_median_bytes={cell.median:.0f} spread={cell.spread:.4f} stable={'yes' if cell.stable else 'no'} "
                    f"p_bytes={'none' if peak is None else format(peak, '.0f')} p_reason={reason} allocator_peak_bytes={cell.allocator_peak:.0f} "
                    f"vm_hwm_median_kib={hwm:.0f} scan_seconds_median={seconds:.3f}")
    return lines


def all_rows(artifact):
    """整份产物的判决行，登记第六节的次序；缺的格逐行报 not_judged，不当 0。"""
    rows = cell_summary_rows(artifact)
    rows += line_rows(artifact)
    rows += contrast_rows(artifact, "jia_slot_fill", ("k64_full", "k64_half"))
    rows += contrast_rows(artifact, "jia_named", ("k64_full", "k1_full"))
    rows += control_rows(artifact)
    rows += split_rows(artifact)
    rows += doable_row(artifact)
    rows += failure_rows(artifact)
    rows += anchor_rows(artifact)
    return rows


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["--selftest-breaks"]:
        return selftest_breaks()
    if not arguments or arguments[0].startswith("--") or any(argument not in ("--append", "--unstable") for argument in arguments[1:]):
        print("用法：report.py <产物 .out> [--append | --unstable] | --selftest | --selftest-breaks", file=sys.stderr)
        return 2
    try:
        with open(arguments[0], encoding="utf-8") as handle:
            text_lines = handle.read().splitlines()
    except OSError as error:
        print(f"读不了产物 {arguments[0]}：{error}", file=sys.stderr)
        return 2
    artifact = Artifact(text_lines)
    if len(artifact.sizes_seen) > 1:
        print(f"产物里的 size_of 不止一组：{sorted(artifact.sizes_seen)}（装置版本不一致，整份不判）", file=sys.stderr)
        return 3
    if "--unstable" in arguments:
        for arm, devices, fill, named in cells_needing_a_rerun(artifact):
            print(f"{arm} {devices} {fill} {named}")
        return 0
    if "--append" in arguments and any(line.startswith("E7RESULT name=verdict ") or line.startswith("E7RESULT name=done ") for line in text_lines):
        print("产物里已经有判决行或收尾行，不再追加（换一个新文件名重判）", file=sys.stderr)
        return 2
    rows = all_rows(artifact)
    for row in rows:
        print(row)
    if "--append" in arguments:
        emitted = sum(1 for line in text_lines if line.startswith("E7RESULT ")) + len(rows) + 1
        with open(arguments[0], "a", encoding="utf-8") as handle:
            handle.write("\n".join(rows) + f"\nE7RESULT name=done emitted={emitted}\n")
    return 0


# ---- 自证：合成产物 ----

SYNTHETIC_SIZES = {"location_entry": 16, "named_unit": 64, "node_pointer": 56, "journal_record": 320}
SYNTHETIC_EMPTY_BYTES = 60 * MEBIBYTE
SYNTHETIC_OVERHEAD_BYTES = 20 * MEBIBYTE   # 扫环格的 cgroup 峰值比分配器峰值多的那一截（进程里分配器没数到的）


def synthetic_line(arm, devices, fill, named, repeat, allocator, ends=None, digest=None):
    filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
    scans = arm in SCANNING_ARMS
    ends = ends or [0] * devices
    sizes = " ".join(f"size_{name}={value}" for name, value in SYNTHETIC_SIZES.items())
    return (f"E7RESULT name=e197 arm={arm} devices={devices} fill={fill} named={named} repeat={repeat} ring_slots={RING_SLOTS} "
            f"filled_slots={filled} reads={devices * RING_SLOTS if scans else 0} reads_expected={devices * RING_SLOTS if scans else 0} "
            f"records_returned={filled if scans else 0} records_expected={filled if scans else 0} anchors_hold=true "
            f"allocator_peak_over_start_bytes={allocator} allocator_live_at_each_device_end_bytes={','.join(map(str, ends))} "
            f"allocator_live_after_return_bytes=0 vm_hwm_kib=1000 scan_seconds=1.000000 table_digest={digest or ('none' if not scans else f'd{devices}{fill}{named}')} {sizes}")


def synthetic_artifact_lines(mutate=None):
    """每一格 5 次（重复号 1–5）：分配器峰值取推算值，cgroup 峰值 = 空跑格 + 分配器峰值 + 20 MiB，5 次之间差 1000 字节。
    mutate(arm, devices, fill, named, repeat, fields) 可以改一格里的字段（fields: allocator、cgroup、digest），交 None 的整次删掉。"""
    lines = []
    for geometry, (named, fill) in GEOMETRIES.items():
        filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
        arms = ("empty",) + SCANNING_ARMS + (("measurement_pc",) if geometry == "k64_full" else ())
        for arm in arms:
            for devices in (DEVICE_COUNTS if arm != "measurement_pc" else (1,)):
                allocator = int(predicted_allocator_peak(arm, devices, filled, named, SYNTHETIC_SIZES))
                cgroup = SYNTHETIC_EMPTY_BYTES + allocator + (SYNTHETIC_OVERHEAD_BYTES if arm in SCANNING_ARMS else 0)
                if arm == "measurement_pc":
                    allocator, cgroup = MEASUREMENT_BYTES + 4096, SYNTHETIC_EMPTY_BYTES + MEASUREMENT_BYTES + 8 * MEBIBYTE
                for repeat in range(1, REPEATS_PER_ROUND + 1):
                    ends = None
                    if arm in ("baseline", "baseline_pc"):
                        ends = [capacity_after_pushes(d * filled - (1 if fill == "full" else 0)) * SYNTHETIC_SIZES["journal_record"]
                                + (d * filled - (1 if fill == "full" else 0)) * named * SYNTHETIC_SIZES["named_unit"] + RING_SLOTS * 8
                                for d in range(1, devices + 1)]
                    fields = {"allocator": allocator, "cgroup": cgroup + 1000 * repeat, "digest": None}
                    if mutate is not None:
                        fields = mutate(arm, devices, fill, named, repeat, fields)
                        if fields is None:
                            continue
                    lines.append(synthetic_line(arm, devices, fill, named, repeat, fields["allocator"], ends, fields["digest"]))
                    lines.append(f"E7RESULT name=e197_cgroup key=e197/{arm}/{devices}/{fill}/{named}/{repeat} cap=8G attempts=1 exit=0 "
                                 f"cgroup_peak_bytes={fields['cgroup']}")
    for case in ("D1", "D2", "D3", "D4"):
        for arm in ("baseline_pc", "split", "split_pc"):
            lines.append(f"E7RESULT name=e197_diff case={case} arm={arm} equal=true reference_records=1")
    for arm in ("baseline_pc", "split", "split_pc"):
        for named, fill in GEOMETRIES.values():
            for devices in DEVICE_COUNTS:
                lines.append(f"E7RESULT name=e197_compare arm={arm} devices={devices} fill={fill} named={named} equal=true records=1")
    return lines


def verdicts(lines, row, **wanted):
    """合成产物上判出来的、row 对得上且带 wanted 里每个键值的判决行（字段字典）。"""
    found = []
    for line in all_rows(Artifact(lines)):
        fields = parse_line(line)
        if fields.get("name") == "verdict" and fields.get("row") == row and all(fields.get(k) == v for k, v in wanted.items()):
            found.append(fields)
    return found


def one(lines, row, **wanted):
    found = verdicts(lines, row, **wanted)
    if len(found) != 1:
        raise AssertionError(f"{row} {wanted} 该有 1 行，有 {len(found)} 行")
    return found[0]


def case_empty_cell_is_subtracted():
    lines = synthetic_artifact_lines()
    filled = RING_SLOTS
    allocator = int(predicted_allocator_peak("baseline", 1, filled, 64, SYNTHETIC_SIZES))
    peak, reason = subtracted_peak(Artifact(lines), "baseline", 1, "k64_full")
    assert reason == "ok" and peak == allocator + SYNTHETIC_OVERHEAD_BYTES, (peak, allocator + SYNTHETIC_OVERHEAD_BYTES, reason)


def case_median_of_five_not_minimum():
    artifact = Artifact(synthetic_artifact_lines())
    cell = cell_value(artifact, "baseline", 1, "full", 64)
    allocator = int(predicted_allocator_peak("baseline", 1, RING_SLOTS, 64, SYNTHETIC_SIZES))
    expected = SYNTHETIC_EMPTY_BYTES + allocator + SYNTHETIC_OVERHEAD_BYTES + 3000
    assert cell.median == expected, (cell.median, expected)


def case_line_flags_a_bent_line():
    clean = synthetic_artifact_lines()
    assert one(clean, "jia_line4", geometry="k64_full")["judgment"] == "on_line"
    assert one(clean, "jia_line3", geometry="k64_full")["judgment"] == "on_line"

    def inflate(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline", 4, "full", 64):
            fields["cgroup"] += int(0.15 * (fields["allocator"] + SYNTHETIC_OVERHEAD_BYTES))
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
    assert clean["compare_lines"] == "36" and clean["compare_mismatches"] == "0", clean
    unequal = [line.replace("equal=true", "equal=false") if "name=e197_compare arm=split devices=3 fill=full named=64" in line else line for line in synthetic_artifact_lines()]
    row = one(unequal, "yi_doable")
    assert row["judgment"] == "not_doable" and row["compare_mismatches"] == "1", row
    fewer = [line for line in synthetic_artifact_lines() if "name=e197_compare arm=split_pc" not in line]
    assert one(fewer, "yi_doable")["judgment"] == "incomplete"


def second_round_lines(arm, devices, fill, named, extra):
    """重复号 6–10 的 5 次（第二轮），cgroup 峰值 = 空跑格 + 分配器峰值 + 20 MiB，再加 extra(repeat) 字节。"""
    filled = RING_SLOTS if fill == "full" else RING_SLOTS // 2
    allocator = int(predicted_allocator_peak(arm, devices, filled, named, SYNTHETIC_SIZES))
    lines = []
    for repeat in range(6, 11):
        cgroup = SYNTHETIC_EMPTY_BYTES + allocator + SYNTHETIC_OVERHEAD_BYTES + extra(repeat)
        lines.append(synthetic_line(arm, devices, fill, named, repeat, allocator))
        lines.append(f"E7RESULT name=e197_cgroup key=e197/{arm}/{devices}/{fill}/{named}/{repeat} cap=8G attempts=1 exit=0 cgroup_peak_bytes={cgroup}")
    return lines


def case_unstable_cell_is_not_judged():
    def spread_in_round_one(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline", 2, "full", 64) and repeat == 5:
            fields["cgroup"] += 100 * MEBIBYTE
        return fields

    lines = synthetic_artifact_lines(spread_in_round_one)
    row = one(lines, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "unstable_needs_rerun", row
    assert ("baseline", 2, "full", 64) in cells_needing_a_rerun(Artifact(lines))
    steady = one(lines + second_round_lines("baseline", 2, "full", 64, lambda repeat: 1000 * repeat), "jia_line3", geometry="k64_full")
    assert steady["judgment"] == "on_line", steady     # 第二轮 5 次齐了且稳定，改用第二轮
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
            fields["allocator"] = int(fields["allocator"] * 1.2)
        return fields

    assert one(synthetic_artifact_lines(), "A7")["cells_outside"] == "0"
    assert one(synthetic_artifact_lines(inflate), "A7")["cells_outside"] == "1"


def case_f1_triggers_when_the_control_is_not_above():
    def equal_to_baseline(arm, devices, fill, named, repeat, fields):
        if (arm, devices, fill, named) == ("baseline_pc", 3, "full", 64):
            fields["cgroup"] -= int(predicted_allocator_peak("baseline_pc", 3, RING_SLOTS, 64, SYNTHETIC_SIZES)
                                    - predicted_allocator_peak("baseline", 3, RING_SLOTS, 64, SYNTHETIC_SIZES))
        return fields

    assert one(synthetic_artifact_lines(), "F1")["triggered"] == "no"
    assert one(synthetic_artifact_lines(equal_to_baseline), "F1")["triggered"] == "yes"


def case_measurement_window_and_clause_numbers():
    lines = synthetic_artifact_lines()
    assert one(lines, "F5")["triggered"] == "no"
    assert one(lines, "F3")["triggered"] == "no" and one(lines, "F4")["triggered"] == "no" and one(lines, "F2")["triggered"] == "no"

    def wrong_window(arm, devices, fill, named, repeat, fields):
        if arm == "measurement_pc":
            fields["allocator"] = 100 * MEBIBYTE
        return fields

    assert one(synthetic_artifact_lines(wrong_window), "F5")["triggered"] == "yes"


def case_missing_reading_is_not_run_and_not_zero():
    def drop_one(arm, devices, fill, named, repeat, fields):
        return None if (arm, devices, fill, named, repeat) == ("baseline", 1, "full", 64, 3) else fields

    lines = synthetic_artifact_lines(drop_one)
    row = one(lines, "jia_line4", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row
    assert one(lines, "F1")["triggered"] == "not_judged"


def case_prediction_reproduces_the_registered_numbers():
    registered = {"location_entry": 16, "named_unit": 64, "node_pointer": 56, "journal_record": 376}
    wanted = [("baseline", 1, RING_SLOTS, 64, 0.985), ("baseline", 2, RING_SLOTS, 64, 1.830), ("baseline", 3, RING_SLOTS, 64, 2.767),
              ("baseline", 4, RING_SLOTS, 64, 3.520), ("baseline_pc", 4, RING_SLOTS, 64, 4.270), ("split", 2, RING_SLOTS, 64, 1.738),
              ("split", 4, RING_SLOTS, 64, 1.744), ("baseline", 4, RING_SLOTS // 2, 64, 1.760), ("split", 4, RING_SLOTS // 2, 64, 0.872),
              ("baseline", 4, RING_SLOTS, 1, 0.566), ("split", 4, RING_SLOTS, 1, 0.268)]
    for arm, devices, filled, named, gibibytes in wanted:
        predicted = predicted_allocator_peak(arm, devices, filled, named, registered) / GIBIBYTE
        assert abs(predicted - gibibytes) < 0.002, (arm, devices, filled, named, predicted, gibibytes)
    for devices in (1, 2):
        assert predicted_allocator_peak("split_pc", devices, RING_SLOTS, 64, registered) == predicted_allocator_peak("baseline", devices, RING_SLOTS, 64, registered)
    assert predicted_allocator_peak("split_pc", 3, RING_SLOTS, 1, registered) < 0.9 * predicted_allocator_peak("baseline", 3, RING_SLOTS, 1, registered)


def scale_cells(factor, selected):
    """selected(arm, devices, fill, named) 命中的格，cgroup 峰值减空跑格的那一截乘 factor。"""
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
    # 几何敏感性：K = 1 满槽的 3 盘偏离按登记推在 +16% 上下，在 5% 之外；K = 64 两档在 5% 之内
    assert one(clean, "jia_line3", geometry="k1_full")["judgment"] == "off_line"
    assert one(clean, "jia_line3", geometry="k64_half")["judgment"] == "on_line"


def case_split_rows_and_f2():
    clean = synthetic_artifact_lines()
    assert one(clean, "yi_close_to_one", geometry="k64_full")["judgment"] == "not_close_to_one"
    assert one(clean, "yi_close_to_one", geometry="k1_full")["judgment"] == "close_to_one"
    assert one(clean, "F2")["triggered"] == "no"
    # 逐盘不释放的对照若不涨（和 split 一样平），F2 触发：这一次的测量分不出逐盘释放
    flat = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(
        fields, allocator=int(predicted_allocator_peak("split", devices, RING_SLOTS if fill == "full" else RING_SLOTS // 2, named, SYNTHETIC_SIZES)),
        cgroup=SYNTHETIC_EMPTY_BYTES + SYNTHETIC_OVERHEAD_BYTES + 1000 * repeat + int(predicted_allocator_peak("split", devices, RING_SLOTS if fill == "full" else RING_SLOTS // 2, named, SYNTHETIC_SIZES)))
        if arm == "split_pc" else fields)
    assert one(flat, "F2")["triggered"] == "yes"


def case_clause_numbers_trigger_f3_and_f4():
    off = synthetic_artifact_lines(scale_cells(1.6, lambda arm, devices, fill, named: (arm, devices, fill, named) == ("baseline", 2, "full", 64)))
    assert one(off, "F3")["triggered"] == "yes" and "2" in one(off, "F3")["outside_devices"].split(","), one(off, "F3")
    assert one(off, "F4")["triggered"] == "yes", one(off, "F4")


def case_a_run_whose_anchors_fail_is_not_used():
    def broken_anchor(arm, devices, fill, named, repeat, fields):
        return fields

    lines = synthetic_artifact_lines(broken_anchor)
    spoiled = [line.replace("records_returned=196608", "records_returned=196607") if "arm=baseline devices=1 fill=full named=64 repeat=2 " in line else line for line in lines]
    assert spoiled != lines
    row = one(spoiled, "jia_line3", geometry="k64_full")
    assert row["judgment"] == "not_judged" and row["reason"] == "not_run", row      # A3 不符：那一格作废，不当 0
    other_ring = [line.replace("ring_slots=196608", "ring_slots=100") if "arm=baseline devices=2 fill=full named=64 repeat=1 " in line else line for line in lines]
    assert one(other_ring, "jia_line3", geometry="k64_full")["reason"] == "not_run"     # A1 不符：生产格的环槽数不是 196 608


def case_s3_a9_and_trajectory_are_flagged():
    clean = synthetic_artifact_lines()
    assert one(clean, "S3_two_measures")["cells_outside"] == "0" and one(clean, "A9")["geometries_outside"] == "0"
    assert one(clean, "trajectory")["cells_outside"] == "0"
    big_gap = synthetic_artifact_lines(scale_cells(1.5, lambda arm, devices, fill, named: (arm, devices, fill, named) == ("baseline", 3, "full", 64)))
    assert int(one(big_gap, "S3_two_measures")["cells_outside"]) >= 1      # cgroup 比分配器峰值大得多：两条量法互不包含
    split_drift = synthetic_artifact_lines(lambda arm, devices, fill, named, repeat, fields: dict(fields, allocator=int(fields["allocator"] * 1.03))
                                           if (arm, devices, fill, named) == ("split", 1, "full", 64) else fields)
    assert one(split_drift, "A9")["geometries_outside"] == "1"                # 1 盘上 split 与 baseline 的分配器峰值差 3% > 1%
    text = [line.replace("allocator_live_at_each_device_end_bytes=", "allocator_live_at_each_device_end_bytes=9") if "arm=baseline devices=2 fill=full named=64 repeat=1 " in line else line for line in clean]
    assert int(one(text, "trajectory")["cells_outside"]) >= 1


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
)
BREAK_TO_CASE = {
    "line-ignores-the-sign": "line-flags-a-bent-line",
    "pc-red-threshold-ignored": "pc-not-red-where-it-is-not",
    "slope-uses-the-top-point-only": "split-slope-and-its-control",
    "median-replaced-by-minimum": "median-of-five-not-minimum",
    "empty-cell-not-subtracted": "empty-cell-is-subtracted",
    "digest-mismatch-ignored": "digest-mismatch-is-caught",
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
    return 0


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


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.exit(main(sys.argv[1:]))
