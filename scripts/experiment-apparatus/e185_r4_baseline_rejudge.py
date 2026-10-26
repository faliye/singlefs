#!/usr/bin/env python3
# admission: always 每一次调都重读点名的五份产物、现算门槛与判定，上一次的结论不替这一次作保
# run-condition: none 只读点名的产物文本，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置
"""E185 第 4 次重跑重判段（research/prompts/e185-r4-prereg.md 第五节「重判段怎么算」、第六节 B3–B7）：九种 CPU 写法当基线，
照第 3 次重跑的门槛算法逐点现算候选 A–D 的门槛，逐行重判第 3 次重跑第一段（与这一趟控漂移段受控臂）的每一行点行。

只读产物文本的算术，独立手写，不 import crates/ 与装置的任何东西（与装置的 Rust 判法是两份写法，靠停机 K-S3 对上）。

用法：
  e185_r4_baseline_rejudge.py --host-cpu-product <r3 第一段带 -2 的产物> --stage-split-product <r2 第二段产物>
                              --writings-product <这一趟写法与拆分段产物> --attack-ledger <out/g1.out>
                              [--drift-control-product <这一趟控漂移段产物>]
                              [--writings-sha256 <输入清单记的那一个>] [--drift-control-sha256 <输入清单记的那一个>]
      先核输入的 sha256（K-S5：g1.out、r2、r3 三份写死；这一趟两份给了 --…-sha256 就比），再现算 ρ 与 r3 打出的 machine_ratio 比（K-S4），
      再在写法甲、门槛来源「r2 产物按比例换」、c_rt 来源 r3 下逐行复现 r3 第一段的门槛与判定（PR1、K-S3：线程钟那一半 1776 行，
      每一行的 threshold_milliseconds、judgement_per_round、judgement、floor_judgement 逐字相同）；三条有一条不过就打 name=stop_clause、退 3。
      过了才出 B3–B7：name=baseline_threshold、name=baseline_rejudge_point、name=baseline_rejudge_summary、
      name=baseline_rejudge_point_summary、name=baseline_knob_flip、name=today_over_r2、name=unstable_count_r3、name=failure_clause；末行 name=done。
      没给 --drift-control-product 时 c_rt 来源只报 r3 那一端（登记第十一节「够判停机」）。
退出码：0 算完；2 用法错或产物读不了；3 停机条款触发（K-S3、K-S4、K-S5）。
口径（登记第一节「读法写死」第 1 行）：
  门槛算法照 r3 装置 candidate_thresholds_of_the_ledger：按 (f, P, K, 轮) 分组，kind=issue 那一行算签发（后来的盖掉先来的）、其余算批，
  不滤 valid；A 省 crc（签发加各批），B 是 A 加签发的 choose，C 省 crc + scan_increment（签发加各批）加签发的 choose，D 省各批的
  release_check_crc；提交次数 A、B、C 是批数 + 1，D 是批数；各轮省下的取中位 ÷ 1e6 得毫秒，提交次数取轮号最小那一轮的。
  写法 X 只换 crc、scan_increment、release_check_crc 三样，choose 不变。门槛来源两端：ratio_on_the_r2_product（r2 每条 stage 行三样各乘
  这一趟 name=writing_factor 的 φ，写法甲 φ ≡ 1、候选 C 用 g1.out 原样那一个数）、direct_from_the_r4_product（这一趟 name=stage 行定签发与批、
  name=cpu_writing_stage 行给写法 X 那一列）。门槛的机器两端：attack_ledger 原样、machine_scaled × ρ（ρ = 量 c_rt 那一次的校准基线 ÷
  量阶段 CPU 那一次的校准基线，都取 name=calibration_baseline … position=start 的 median_nanoseconds）。
  判定照 r3 R1–R3：每轮 c_rt < 门槛判 not_losing（B 写 not_losing_lower_bound），≥ 判 losing；5 轮相同写那个、不全相同写 unstable、
  有一轮读不到写 rounds_insufficient，门槛读不到写 not_run，那一行在 r3 里作废（void_…）的照写那个作废；c_floor 只在线程钟的常驻形态上判。
单测在同目录的 e185_r4_baseline_rejudge_test.py；变异表 e185_r4_baseline_rejudge.mutations.tsv（原文在这份脚本里各命中一次，执行员在草稿目录
逐行改坏副本跑单测，不走 mutate.sh）。
"""
import hashlib
import re
import sys

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

TOKEN = re.compile(r'(\w+)=(\S+)')

# 登记第十一节停机 K-S5：三份已有输入的 sha256
REGISTERED_SHA256 = {
    "attack_ledger": "61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025",
    "stage_split_product": "f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a",
    "host_cpu_product": "840b68352e6d849e9a397dd71c3fb9d7a1f30c0a5b7f7f362e620b2a7ef69993",
}
# 登记第七节乙类：r3 第一段线程钟那一半的点行数
THREAD_CLOCK_POINT_LINES_OF_THE_R3_PRODUCT = 1776
WRITINGS = (
    "today",
    "hardware_crc_single",
    "hardware_crc_three_way",
    "identity_index_with_build",
    "identity_index_without_build",
    "both_single_with_build",
    "both_single_without_build",
    "both_three_way_with_build",
    "both_three_way_without_build",
)
TODAY = "today"
# 每种写法 = （CRC 那两项取哪一列、扫描取哪一份）；列名照装置 name=cpu_writing_stage 的键
CRC_COLUMN = {"today": "today", "single": "hardware_single", "three_way": "hardware_three_way"}
WRITING_PARTS = {
    "today": ("today", "today"),
    "hardware_crc_single": ("single", "today"),
    "hardware_crc_three_way": ("three_way", "today"),
    "identity_index_with_build": ("today", "with_build"),
    "identity_index_without_build": ("today", "without_build"),
    "both_single_with_build": ("single", "with_build"),
    "both_single_without_build": ("single", "without_build"),
    "both_three_way_with_build": ("three_way", "with_build"),
    "both_three_way_without_build": ("three_way", "without_build"),
}
STAGES = ("crc", "scan_increment", "release_check_crc")
CANDIDATES = ("scan_checksum_only", "scan_checksum_and_choice", "every_stage", "release_check_only")
EVERY_STAGE = "every_stage"
LOWER_BOUND_CANDIDATE = "scan_checksum_and_choice"
RATIO_ON_THE_R2_PRODUCT = "ratio_on_the_r2_product"
DIRECT_FROM_THE_R4_PRODUCT = "direct_from_the_r4_product"
THRESHOLD_SOURCES = (RATIO_ON_THE_R2_PRODUCT, DIRECT_FROM_THE_R4_PRODUCT)
CRT_FROM_R3 = "r3_host_cpu_per_submission"
CRT_FROM_THE_DRIFT_CONTROL = "r4_drift_control_controlled"
THRESHOLD_KINDS = ("attack_ledger", "machine_scaled")
ROUNDS = 5
# 一行点行的键（同一点、候选、卡、形态、等法、间隔、钟、门槛的机器）
LINE_KEY_FIELDS = ("fraction", "pool_data_units", "batch_units", "candidate", "index", "form", "wait", "gap_milliseconds", "clock", "threshold_kind")
NOT_A_JUDGEMENT = ("rounds_insufficient", "not_run", "not_applicable")
# B7（第十条 L9）：这一趟写法甲 ÷ r2 不在这一段里就记
TODAY_OVER_R2_LOWER = 0.80
TODAY_OVER_R2_UPPER = 1.25
# 第八节几何敏感性的旋钮：（名、两端的写法或取值的对子）
LANE_PAIRS = (("hardware_crc_single", "hardware_crc_three_way"), ("both_single_with_build", "both_three_way_with_build"),
              ("both_single_without_build", "both_three_way_without_build"))
BUILD_PAIRS = (("identity_index_with_build", "identity_index_without_build"), ("both_single_with_build", "both_single_without_build"),
               ("both_three_way_with_build", "both_three_way_without_build"))


class Unreadable(Exception):
    pass


def fields_of(line):
    """一行产物（带不带 E7RESULT 前缀都行）拆成「键 → 值」。"""
    text = line[len("E7RESULT "):] if line.startswith("E7RESULT ") else line
    return dict(TOKEN.findall(text))


def lines_of(text):
    return [fields_of(line) for line in text.splitlines() if line.startswith("E7RESULT ") or line.startswith("name=")]


def number_or_none(text):
    if text is None or text == "none":
        return None
    return float(text)


def median(values):
    """照装置的中位：排好，奇数个取中间那个，偶数个取中间两个的和 ÷ 2；空的交 None。"""
    if not values:
        return None
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2 == 1:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) / 2


# ── 门槛账：一条 stage 行要的几样（r3 装置 stage_ledger_rows_of 读的那几样） ────────────────────

def intent_key_of(fields):
    return (fields["fraction"], fields["pool_data_units"], fields["batch_units"], fields["round"])


def ledger_rows_of_the_stage_lines(lines):
    """r2 第二段（或这一趟写法与拆分段）的 name=stage 行，不滤 valid；crc、乙段增量、释放前读盘核缺一样就整份读不成。"""
    rows = []
    for fields in lines:
        if fields.get("name") != "stage":
            continue
        try:
            rows.append({
                "intent": intent_key_of(fields),
                "call_index": fields["call_index"],
                "is_issue": fields["kind"] == "issue",
                "crc": float(fields["crc_cpu_nanoseconds"]),
                "scan_increment": float(fields["scan_increment_cpu_nanoseconds"]),
                "choose": number_or_none(fields.get("choose_cpu_nanoseconds")),
                "release_check_crc": float(fields["release_check_crc_cpu_nanoseconds"]),
            })
        except (KeyError, ValueError) as error:
            raise Unreadable(f"stage 行缺数：{error}")
    return rows


def writing_stage_value(pieces, writing, stage):
    """写法 X 下一样阶段的值（登记第一节写法乙丙丁）：crc、release_check_crc 取那一种 CRC 写法的列；scan_increment = 那一份扫描
    （丙丁按端加不加建索引）− 今天写法的 crc（乙不改扫描里 parse_data_unit 调的那一份 CRC）。有一样读不到交 None。"""
    crc_writing, scan_writing = WRITING_PARTS[writing]
    column = CRC_COLUMN[crc_writing]
    if stage == "crc":
        return number_or_none(pieces.get(f"crc_{column}_cpu_nanoseconds"))
    if stage == "release_check_crc":
        return number_or_none(pieces.get(f"release_check_crc_{column}_cpu_nanoseconds"))
    lookup = number_or_none(pieces.get("scan_identity_index_lookup_cpu_nanoseconds"))
    build = number_or_none(pieces.get("identity_index_build_cpu_nanoseconds"))
    today_scan = number_or_none(pieces.get("scan_today_cpu_nanoseconds"))
    today_crc = number_or_none(pieces.get("crc_today_cpu_nanoseconds"))
    if scan_writing == "today":
        scan = today_scan
    elif scan_writing == "with_build":
        scan = None if lookup is None or build is None else lookup + build
    else:
        scan = lookup
    if scan is None or today_crc is None:
        return None
    return scan - today_crc


def direct_rows_of_the_writing(stage_rows, writing_lines, writing):
    """「这一趟产物直接算」：签发与批照这一趟的 name=stage 行（choose 也取它），三样取 name=cpu_writing_stage 那一次调用写法 X 的列；
    那一次调用读不到（readable=false、缺列）的三样是 None。"""
    pieces_by_call = {}
    for fields in writing_lines:
        if fields.get("name") == "cpu_writing_stage":
            pieces_by_call[intent_key_of(fields) + (fields["call_index"],)] = fields
    rows = []
    for row in stage_rows:
        pieces = pieces_by_call.get(row["intent"] + (row["call_index"],))
        readable = pieces is not None and pieces.get("readable") == "true"
        replaced = dict(row)
        for stage in STAGES:
            replaced[stage] = writing_stage_value(pieces, writing, stage) if readable else None
        rows.append(replaced)
    return rows


def writing_factors_of(writing_lines):
    """这一趟的写法系数 φ：键 (f, P, K, 写法, 阶段) → （系数或 None、状态）。"""
    factors = {}
    for fields in writing_lines:
        if fields.get("name") == "writing_factor":
            key = (fields["fraction"], fields["pool_data_units"], fields["batch_units"], fields["writing"], fields["stage"])
            factors[key] = (number_or_none(fields.get("factor")), fields.get("status", "not_run"))
    return factors


def scaled_rows_of_the_writing(r2_rows, factors, writing):
    """「r2 产物按比例换」：r2 每条 stage 行的三样各乘那一点写法 X 的系数（choose 不乘；写法甲 φ ≡ 1、一行不动）。系数读不到的那一样是 None。"""
    if writing == TODAY:
        return [dict(row) for row in r2_rows]
    rows = []
    for row in r2_rows:
        scaled = dict(row)
        for stage in STAGES:
            factor, _status = factors.get(row["intent"][:3] + (writing, stage), (None, "not_run"))
            scaled[stage] = None if factor is None else row[stage] * factor
        rows.append(scaled)
    return rows


def voided_writings_of(factors):
    """PW1、PW2 没过的计时件让用到它的写法整段作废（装置在 name=writing_factor 写 status=void_positive_control）。"""
    return {key[3] for key, (_factor, status) in factors.items() if status == "void_positive_control"}


# ── 候选门槛（r3 装置 saved_nanoseconds_of_the_intent、candidate_thresholds_of_the_ledger 的算法与加法次序） ────────────────────

def saved_of_the_intent(candidate, issue, batches):
    """一条意图一个候选：交回（省下的纳秒或 None、提交次数、读不成的原因）。加法次序照装置：先按批的次序累加，再加签发那几项。"""
    if issue is None:
        return None, None, "issue_missing"
    batch_count = len(batches)
    needed = {"scan_checksum_only": ("crc",), "scan_checksum_and_choice": ("crc",),
              "every_stage": ("crc", "scan_increment"), "release_check_only": ("release_check_crc",)}[candidate]
    rows_needed = batches if candidate == "release_check_only" else batches + [issue]
    if any(row[stage] is None for row in rows_needed for stage in needed):
        return None, None, "reading_unreadable"
    if candidate in ("scan_checksum_and_choice", EVERY_STAGE) and issue["choose"] is None:
        return None, None, "reading_unreadable"
    if candidate == EVERY_STAGE and any(row["scan_increment"] < 0 for row in rows_needed):
        return None, None, "scan_increment_below_zero"
    total = 0.0
    if candidate == "scan_checksum_only":
        for batch in batches:
            total = total + batch["crc"]
        return total + issue["crc"], batch_count + 1, None
    if candidate == "scan_checksum_and_choice":
        for batch in batches:
            total = total + batch["crc"]
        return total + issue["crc"] + issue["choose"], batch_count + 1, None
    if candidate == EVERY_STAGE:
        for batch in batches:
            total = total + (batch["crc"] + batch["scan_increment"])
        return total + (issue["crc"] + issue["scan_increment"] + issue["choose"]), batch_count + 1, None
    for batch in batches:
        total = total + batch["release_check_crc"]
    return total, batch_count, None


def thresholds_of_the_ledger(rows):
    """每个点 (f, P, K) 四个候选：{"saved_ms", "submissions", "threshold", "status", "reason"}。按 (f, P, K, 轮) 分组，kind=issue 那一行算签发
    （后来的盖掉先来的），其余算批；各轮省下的取中位 ÷ 1e6，提交次数取轮号最小那一轮的。有一轮读不成，那一点那个候选 not_run。"""
    intents = {}
    for row in rows:
        entry = intents.setdefault(row["intent"], [None, []])
        if row["is_issue"]:
            entry[0] = row
        else:
            entry[1].append(row)
    per_point = {}
    for key in sorted(intents):
        issue, batches = intents[key]
        for candidate in CANDIDATES:
            per_point.setdefault(key[:3], {}).setdefault(candidate, []).append(saved_of_the_intent(candidate, issue, batches))
    thresholds = {}
    for point, by_candidate in per_point.items():
        for candidate, savings in by_candidate.items():
            reasons = [reason for _saved, _count, reason in savings if reason is not None]
            if reasons:
                thresholds[(point, candidate)] = {"saved_ms": None, "submissions": None, "threshold": None,
                                                  "status": "not_run", "reason": reasons[0]}
                continue
            saved_ms = median([saved for saved, _count, _reason in savings]) / 1e6
            submissions = savings[0][1]
            thresholds[(point, candidate)] = {"saved_ms": saved_ms, "submissions": submissions,
                                              "threshold": saved_ms / submissions, "status": "measured", "reason": None}
    return thresholds


def attack_ledger_points_of(text):
    """out/g1.out 的 name=g1_ledger 行：键 (f, P, K) → 门槛原样那一个数（四位小数的文字转成数，同 r3 装置）。"""
    points = {}
    for fields in lines_of(text):
        if fields.get("name") == "g1_ledger":
            points[(fields["f"], fields["P"], fields["K"])] = float(fields["break_even_host_cpu_per_submission_ms"])
    return points


def machine_ratio(local_baseline_nanoseconds, peer_baseline_nanoseconds):
    """ρ = 量 c_rt 那一次本机的校准基线 ÷ 量阶段 CPU 那一次的校准基线；有一样读不到交 None。"""
    if local_baseline_nanoseconds is None or peer_baseline_nanoseconds is None:
        return None
    return local_baseline_nanoseconds / peer_baseline_nanoseconds


def calibration_baseline_of(lines, segment):
    for fields in lines:
        if fields.get("name") == "calibration_baseline" and fields.get("segment") == segment and fields.get("position") == "start":
            return number_or_none(fields.get("median_nanoseconds"))
    return None


def scaled_threshold(base, kind, rho):
    """门槛的机器：attack_ledger 原样，machine_scaled × ρ（ρ 读不到那几行 not_run）。"""
    if base is None:
        return None
    if kind == "attack_ledger":
        return base
    return None if rho is None else base * rho


# ── 判定（r3 装置 judgement_against_the_threshold、judgements_of_the_point、judgement_of_the_valid_rounds） ────────────────────

def judgement_against_the_threshold(host_cpu_milliseconds, threshold_milliseconds, candidate):
    if host_cpu_milliseconds < threshold_milliseconds:
        return "not_losing_lower_bound" if candidate == LOWER_BOUND_CANDIDATE else "not_losing"
    return "losing"


def judgement_of_the_valid_rounds(per_round):
    if len(per_round) < ROUNDS or any(judgement is None for judgement in per_round):
        return "rounds_insufficient"
    return per_round[0] if all(judgement == per_round[0] for judgement in per_round) else "unstable"


def judgements_of_the_point(values, threshold, candidate, voiding):
    per_round = [None if value is None or threshold is None else judgement_against_the_threshold(value, threshold, candidate)
                 for value in values]
    if voiding is not None:
        return per_round, voiding
    if threshold is None:
        return per_round, "not_run"
    return per_round, judgement_of_the_valid_rounds(per_round)


def per_round_values(text):
    return [number_or_none(value) for value in text.split(",")]


def per_round_text(per_round):
    return ",".join("not_run" if judgement is None else judgement for judgement in per_round)


def voiding_of_the_line(fields):
    """那一行在它的产物里作废了（P1、P3 或卡作废）就照写那个作废原因；作废按（卡、形态、等法）定，不随门槛变。"""
    judgement = fields.get("judgement", "")
    return judgement if judgement.startswith("void_") else None


def rejudge_the_line(fields, threshold):
    """一行点行在给定门槛下的判定：c_rt（host_cpu_per_submission_milliseconds_per_round）每轮比门槛；c_floor 只在线程钟、常驻形态上判。"""
    candidate = fields["candidate"]
    voiding = voiding_of_the_line(fields)
    values = per_round_values(fields["host_cpu_per_submission_milliseconds_per_round"])
    per_round, judgement = judgements_of_the_point(values, threshold, candidate, voiding)
    if fields["clock"] == "thread" and fields["form"] != "today_single_kernel":
        floor_values = per_round_values(fields["floor_host_cpu_per_submission_milliseconds_per_round"])
        floor_per_round, floor_judgement = judgements_of_the_point(floor_values, threshold, candidate, voiding)
    else:
        floor_per_round, floor_judgement = [None] * ROUNDS, "not_applicable"
    return {"threshold": threshold, "judgement_per_round": per_round_text(per_round), "judgement": judgement,
            "floor_judgement_per_round": per_round_text(floor_per_round), "floor_judgement": floor_judgement}


def threshold_text(threshold):
    return "none" if threshold is None else f"{threshold:.6f}"


def line_key_of(fields):
    return tuple(fields.get(name, "") for name in LINE_KEY_FIELDS)


def point_lines_of_the_r3_product(lines):
    return [fields for fields in lines if fields.get("name") == "host_cpu_per_submission_point"]


def point_lines_of_the_drift_control_product(lines):
    """控漂移段受控臂的点行（第一节「c_rt 来源」：同名同义的那两列；复现臂不当 c_rt 来源）。"""
    return [fields for fields in lines if fields.get("name") == "drift_control_point" and fields.get("arm") == "controlled"]


def reproduction_findings(r3_point_lines, base_thresholds, rho, expected_thread_lines=None):
    """PR1、停机 K-S3：写法甲、门槛来源「r2 产物按比例换」、c_rt 来源 r3 下逐行重判，门槛的文字、每轮判定、判定、c_floor 判定都要与 r3 那一行
    逐字相同；线程钟那一半要正好 expected_thread_lines 行（不给取登记的 1776）。交回对不上的几句（全对交空表）。"""
    if expected_thread_lines is None:
        expected_thread_lines = THREAD_CLOCK_POINT_LINES_OF_THE_R3_PRODUCT
    findings = []
    thread_lines = sum(1 for fields in r3_point_lines if fields["clock"] == "thread")
    if thread_lines != expected_thread_lines:
        findings.append(f"clause_k_s3_thread_clock_point_lines_{thread_lines}_not_{expected_thread_lines}")
    for line_number, fields in enumerate(r3_point_lines, 1):
        point = (fields["fraction"], fields["pool_data_units"], fields["batch_units"])
        base = base_thresholds.get((point, fields["candidate"]))
        rejudged = rejudge_the_line(fields, scaled_threshold(base, fields["threshold_kind"], rho))
        for name, computed in (("threshold_milliseconds", threshold_text(rejudged["threshold"])),
                               ("judgement_per_round", rejudged["judgement_per_round"]),
                               ("judgement", rejudged["judgement"]),
                               ("floor_judgement", rejudged["floor_judgement"])):
            if computed != fields.get(name):
                findings.append(f"clause_k_s3_point_line_{line_number}_{name}_{computed}_differs_from_the_r3_product_{fields.get(name)}")
    return findings


# ── 汇总：B5、B6、旋钮翻面、B7、D5 ────────────────────

def is_a_judgement(judgement):
    return judgement not in NOT_A_JUDGEMENT and not judgement.startswith("void_") and judgement != "absent"


def is_not_losing(judgement):
    return judgement in ("not_losing", "not_losing_lower_bound")


def summary_lines(rejudged):
    """B5（按写法、门槛来源、c_rt 来源、候选、卡、门槛的机器、钟）与 B6（线程钟，按点再分）。"""
    lines = []
    summaries, point_summaries = {}, {}
    for row in rejudged:
        key = (row["writing"], row["threshold_source"], row["crt_source"], row["candidate"], row["index"], row["threshold_kind"], row["clock"])
        counts = summaries.setdefault(key, {})
        for name in ("lines", row["judgement"]):
            counts[name] = counts.get(name, 0) + 1
        if row["changed"] == "true":
            counts["changed"] = counts.get("changed", 0) + 1
            target = "changed_to_losing" if row["judgement"] == "losing" else (
                "changed_to_unstable" if row["judgement"] == "unstable" else (
                    "changed_to_not_losing" if is_not_losing(row["judgement"]) else "changed_to_other"))
            counts[target] = counts.get(target, 0) + 1
        if row["clock"] == "thread":
            point_key = key[:6] + (row["fraction"], row["pool_data_units"], row["batch_units"])
            point_counts = point_summaries.setdefault(point_key, {"not_losing": 0, "losing": 0, "unstable": 0, "judged": 0})
            judgement = row["judgement"]
            if is_not_losing(judgement):
                point_counts["not_losing"] += 1
            elif judgement in ("losing", "unstable"):
                point_counts[judgement] += 1
            point_counts["judged"] += int(is_a_judgement(judgement))
    for key, counts in summaries.items():
        writing, source, crt, candidate, index, kind, clock = key
        lines.append(
            f"name=baseline_rejudge_summary writing={writing} threshold_source={source} crt_source={crt} candidate={candidate} index={index} "
            f"threshold_kind={kind} clock={clock} lines={counts['lines']} "
            + " ".join(f"{name}={counts.get(name, 0)}" for name in (
                "not_losing", "not_losing_lower_bound", "losing", "unstable", "rounds_insufficient", "not_run", "void_positive_control",
                "void_card", "changed", "changed_to_losing", "changed_to_unstable", "changed_to_not_losing", "changed_to_other")))
    for key, counts in point_summaries.items():
        writing, source, crt, candidate, index, kind, fraction, pool, batch = key
        lines.append(
            f"name=baseline_rejudge_point_summary writing={writing} threshold_source={source} crt_source={crt} candidate={candidate} "
            f"index={index} threshold_kind={kind} fraction={fraction} pool_data_units={pool} batch_units={batch} "
            f"forms_not_losing={counts['not_losing']} forms_losing={counts['losing']} forms_unstable={counts['unstable']} forms_judged={counts['judged']}")
    return lines


def knob_flip_counts(rejudged):
    """第八节几何敏感性：每个旋钮两端同一行（别的键都相同）的判定比一比，两端都有判定才算一对；交回 [(旋钮, 一端, 另一端, 候选, 对数, 翻面数)]。"""
    judgements = {}
    for row in rejudged:
        judgements[(row["writing"], row["threshold_source"], row["crt_source"], row["key"])] = row["judgement"]
    counts = {}

    def compare(knob, end_one, end_other, candidate, one, other):
        if one is None or other is None or not (is_a_judgement(one) and is_a_judgement(other)):
            return
        entry = counts.setdefault((knob, end_one, end_other, candidate), [0, 0])
        entry[0] += 1
        entry[1] += int(one != other)

    for (writing, source, crt, key), judgement in judgements.items():
        candidate = key[LINE_KEY_FIELDS.index("candidate")]
        for knob, pairs in (("hardware_crc_lanes", LANE_PAIRS), ("identity_index_build", BUILD_PAIRS)):
            for end_one, end_other in pairs:
                if writing == end_one:
                    compare(knob, end_one, end_other, candidate, judgement, judgements.get((end_other, source, crt, key)))
        if source == RATIO_ON_THE_R2_PRODUCT:
            compare("threshold_source", f"{writing}:{RATIO_ON_THE_R2_PRODUCT}", f"{writing}:{DIRECT_FROM_THE_R4_PRODUCT}", candidate,
                    judgement, judgements.get((writing, DIRECT_FROM_THE_R4_PRODUCT, crt, key)))
        if crt == CRT_FROM_R3:
            compare("crt_source", f"{writing}:{CRT_FROM_R3}", f"{writing}:{CRT_FROM_THE_DRIFT_CONTROL}", candidate,
                    judgement, judgements.get((writing, source, CRT_FROM_THE_DRIFT_CONTROL, key)))
        kind_index = LINE_KEY_FIELDS.index("threshold_kind")
        if key[kind_index] == "attack_ledger":
            other_key = key[:kind_index] + ("machine_scaled",) + key[kind_index + 1:]
            compare("threshold_kind", f"{writing}:attack_ledger", f"{writing}:machine_scaled", candidate,
                    judgement, judgements.get((writing, source, crt, other_key)))
    return [(knob, one, other, candidate, pair_count, flipped) for (knob, one, other, candidate), (pair_count, flipped) in sorted(counts.items())]


def intent_stage_sums(rows):
    """每条意图三样阶段之和（crc 与乙段增量：签发加各批；释放前读盘核：各批），同系数 φ 的取法；键 (f, P, K, 阶段) → 各轮之和。"""
    sums = {}
    for row in rows:
        for stage in STAGES:
            if stage == "release_check_crc" and row["is_issue"]:
                continue
            key = row["intent"][:3] + (stage,)
            per_intent = sums.setdefault(key, {})
            value = row[stage]
            current = per_intent.get(row["intent"], 0.0)
            per_intent[row["intent"]] = None if value is None or current is None else current + value
    return {key: [value for value in per_intent.values() if value is not None] for key, per_intent in sums.items()}


def today_over_r2_lines(r4_today_rows, r2_rows):
    """B7：这一趟写法甲各条意图之和的中位 ÷ r2 同一点同一样的中位；不在 [0.80, 1.25] 记 outside_the_band（第十条 L9）。"""
    lines, outside = [], 0
    this_run, previous = intent_stage_sums(r4_today_rows), intent_stage_sums(r2_rows)
    for key in sorted(previous):
        this_median, previous_median = median(this_run.get(key, [])), median(previous[key])
        ratio = None if this_median is None or not previous_median else this_median / previous_median
        judgement = "not_run" if ratio is None else ("inside_the_band" if TODAY_OVER_R2_LOWER <= ratio <= TODAY_OVER_R2_UPPER else "outside_the_band")
        outside += int(judgement == "outside_the_band")
        lines.append(f"name=today_over_r2 fraction={key[0]} pool_data_units={key[1]} batch_units={key[2]} stage={key[3]} "
                     f"ratio={threshold_text(ratio)} judgement={judgement}")
    return lines, outside


def unstable_count_lines(r3_point_lines):
    """D5：r3 第一段每张卡全部点行的 judgement 列里 unstable 几行（口径同第二轮判决数 1054 的那一列）。"""
    counts = {}
    for fields in r3_point_lines:
        entry = counts.setdefault(fields["index"], [0, 0])
        entry[0] += 1
        entry[1] += int(fields["judgement"] == "unstable")
    return [f"name=unstable_count_r3 index={index} lines={lines} unstable={unstable}" for index, (lines, unstable) in sorted(counts.items())]


# ── 跑一遍 ────────────────────

OPTIONS = ("--host-cpu-product", "--stage-split-product", "--writings-product", "--attack-ledger", "--drift-control-product",
           "--writings-sha256", "--drift-control-sha256")
REQUIRED_OPTIONS = ("host_cpu_product", "stage_split_product", "writings_product", "attack_ledger")


def options_of(arguments):
    """`--名字 值` 成对；认不得的、缺值的、必给的没给，交 None。"""
    if len(arguments) % 2 == 1:
        return None
    options = {}
    for name, value in zip(arguments[0::2], arguments[1::2]):
        if name not in OPTIONS or name[2:].replace("-", "_") in options:
            return None
        options[name[2:].replace("-", "_")] = value
    return options if all(name in options for name in REQUIRED_OPTIONS) else None


def sha256_of(text_bytes):
    return hashlib.sha256(text_bytes).hexdigest()


def input_file_lines(contents, options):
    """K-S5：三份已有输入核写死的 sha256；这一趟两份给了 --…-sha256 就核它。交回（行、对不上的几句）。"""
    lines, findings = [], []
    checks = [(item, REGISTERED_SHA256[item]) for item in ("attack_ledger", "stage_split_product", "host_cpu_product")]
    for item, option in (("writings_product", "writings_sha256"), ("drift_control_product", "drift_control_sha256")):
        if item in contents:
            checks.append((item, options.get(option, "not_given").lower()))
    for item, registered in checks:
        digest = sha256_of(contents[item])
        holds = registered == "not_given" or digest == registered
        lines.append(f"name=input_file item={item} sha256={digest} registered_sha256={registered} holds={str(holds).lower()}")
        if not holds:
            findings.append(f"clause_k_s5_{item}_sha256_{digest}")
    return lines, findings


def rejudge(contents):
    """出全部判定行；交回（行、停机的几句）。停机条款有一条不过就只交停机之前的行。"""
    texts = {item: data.decode("utf-8") for item, data in contents.items()}
    r3_lines, r2_lines, writing_lines = lines_of(texts["host_cpu_product"]), lines_of(texts["stage_split_product"]), lines_of(texts["writings_product"])
    drift_lines = lines_of(texts["drift_control_product"]) if "drift_control_product" in texts else None
    out, findings = [], []
    baselines = {"r3": calibration_baseline_of(r3_lines, "host_cpu_per_submission"), "r2": calibration_baseline_of(r2_lines, "stage_split"),
                 "writings": calibration_baseline_of(writing_lines, "cpu_writings"),
                 "drift": None if drift_lines is None else calibration_baseline_of(drift_lines, "host_cpu_drift_control")}
    rho = {(CRT_FROM_R3, RATIO_ON_THE_R2_PRODUCT): machine_ratio(baselines["r3"], baselines["r2"]),
           (CRT_FROM_R3, DIRECT_FROM_THE_R4_PRODUCT): machine_ratio(baselines["r3"], baselines["writings"]),
           (CRT_FROM_THE_DRIFT_CONTROL, RATIO_ON_THE_R2_PRODUCT): machine_ratio(baselines["drift"], baselines["r2"]),
           (CRT_FROM_THE_DRIFT_CONTROL, DIRECT_FROM_THE_R4_PRODUCT): machine_ratio(baselines["drift"], baselines["writings"])}
    printed = next((fields.get("ratio") for fields in r3_lines if fields.get("name") == "machine_ratio"), None)
    computed = threshold_text(rho[(CRT_FROM_R3, RATIO_ON_THE_R2_PRODUCT)])
    out.append(f"name=machine_ratio_check computed={computed} printed_by_the_r3_product={printed} holds={str(computed == printed).lower()}")
    if computed != printed:
        findings.append(f"clause_k_s4_machine_ratio_{computed}_differs_from_the_r3_product_{printed}")
    for (crt, source), value in sorted(rho.items()):
        out.append(f"name=machine_ratio crt_source={crt} threshold_source={source} ratio={threshold_text(value)}")
    attack_points = attack_ledger_points_of(texts["attack_ledger"])
    r2_rows = ledger_rows_of_the_stage_lines(r2_lines)
    r4_stage_rows = ledger_rows_of_the_stage_lines(writing_lines)
    factors = writing_factors_of(writing_lines)
    voided = voided_writings_of(factors)
    computed_thresholds, base_thresholds = {}, {}
    for writing in WRITINGS:
        for source in THRESHOLD_SOURCES:
            rows = (scaled_rows_of_the_writing(r2_rows, factors, writing) if source == RATIO_ON_THE_R2_PRODUCT
                    else direct_rows_of_the_writing(r4_stage_rows, writing_lines, writing))
            thresholds = thresholds_of_the_ledger(rows)
            computed_thresholds[(writing, source)] = thresholds
            for (point, candidate), entry in thresholds.items():
                value = entry["threshold"]
                if writing == TODAY and source == RATIO_ON_THE_R2_PRODUCT and candidate == EVERY_STAGE:
                    value = attack_points.get(point)
                base_thresholds[(writing, source, point, candidate)] = value
    r3_point_lines = point_lines_of_the_r3_product(r3_lines)
    today_from_r2 = {(point, candidate): value for (writing, source, point, candidate), value in base_thresholds.items()
                     if writing == TODAY and source == RATIO_ON_THE_R2_PRODUCT}
    findings.extend(reproduction_findings(r3_point_lines, today_from_r2, rho[(CRT_FROM_R3, RATIO_ON_THE_R2_PRODUCT)]))
    out.append(f"name=positive_control row=reproduction_of_the_r3_product lines={len(r3_point_lines)} findings={len(findings)} holds={str(not findings).lower()}")
    if findings:
        return out, findings
    out.extend(threshold_lines(computed_thresholds, base_thresholds, rho, voided, drift_lines is not None))
    rejudged = rejudged_rows(r3_point_lines, drift_lines, base_thresholds, rho, voided)
    out.extend(rejudged_line(row) for row in rejudged)
    out.extend(summary_lines(rejudged))
    flips = knob_flip_counts(rejudged)
    for knob, one, other, candidate, pair_count, flipped in flips:
        out.append(f"name=baseline_knob_flip knob={knob} end_one={one} end_other={other} candidate={candidate} pairs={pair_count} flipped={flipped}")
    today_lines, outside = today_over_r2_lines(direct_rows_of_the_writing(r4_stage_rows, writing_lines, TODAY), r2_rows)
    out.extend(today_lines)
    out.extend(unstable_count_lines(r3_point_lines))
    out.extend(failure_clause_lines(rejudged, flips, outside))
    out.append(f"name=verdict segment=baseline_rejudge stop_clause_findings=0 rejudged_lines={len(rejudged)} voided_writings={','.join(sorted(voided)) or 'none'} "
               f"drift_control_end={'measured' if drift_lines is not None else 'not_run'}")
    return out, []


def threshold_lines(computed_thresholds, base_thresholds, rho, voided, has_the_drift_control):
    """B3：每（写法、门槛来源、门槛的机器、c_rt 来源、点、候选）一行；threshold_over_today 是同一格写法甲的门槛之比。"""
    lines = []
    crt_sources = (CRT_FROM_R3, CRT_FROM_THE_DRIFT_CONTROL) if has_the_drift_control else (CRT_FROM_R3,)
    for (writing, source), thresholds in computed_thresholds.items():
        for crt in crt_sources:
            for kind in THRESHOLD_KINDS:
                for (point, candidate), entry in sorted(thresholds.items()):
                    threshold = scaled_threshold(base_thresholds[(writing, source, point, candidate)], kind, rho[(crt, source)])
                    today = scaled_threshold(base_thresholds.get((TODAY, source, point, candidate)), kind, rho[(crt, source)])
                    over_today = None if threshold is None or not today else threshold / today
                    if writing in voided:
                        status = "status=void_positive_control"
                    elif threshold is None:
                        status = f"status=not_run reason={entry['reason'] or 'machine_ratio_unreadable'}"
                    else:
                        status = "status=measured"
                    lines.append(
                        f"name=baseline_threshold writing={writing} threshold_source={source} threshold_kind={kind} crt_source={crt} "
                        f"fraction={point[0]} pool_data_units={point[1]} batch_units={point[2]} candidate={candidate} "
                        f"saved_per_intent_milliseconds={threshold_text(entry['saved_ms'])} submissions_per_intent={entry['submissions'] or 'none'} "
                        f"threshold_milliseconds={threshold_text(threshold)} threshold_over_today={threshold_text(over_today)} {status}")
    return lines


def rejudged_rows(r3_point_lines, drift_lines, base_thresholds, rho, voided):
    """B4：c_rt 来源两端 × 九种写法 × 门槛来源两端，逐行重判；r3 那一行按键找（同键的头一行），找不到写 absent、changed=not_applicable。"""
    r3_by_key = {}
    for fields in r3_point_lines:
        r3_by_key.setdefault(line_key_of(fields), fields)
    sources_of_crt = [(CRT_FROM_R3, r3_point_lines)]
    if drift_lines is not None:
        sources_of_crt.append((CRT_FROM_THE_DRIFT_CONTROL, point_lines_of_the_drift_control_product(drift_lines)))
    rows = []
    for crt, point_lines in sources_of_crt:
        for writing in WRITINGS:
            for source in THRESHOLD_SOURCES:
                for fields in point_lines:
                    point = (fields["fraction"], fields["pool_data_units"], fields["batch_units"])
                    base = base_thresholds.get((writing, source, point, fields["candidate"]))
                    threshold = scaled_threshold(base, fields["threshold_kind"], rho[(crt, source)])
                    if writing in voided:
                        result = {"threshold": threshold, "judgement_per_round": ",".join(["void_positive_control"] * ROUNDS),
                                  "judgement": "void_positive_control", "floor_judgement_per_round": ",".join(["void_positive_control"] * ROUNDS),
                                  "floor_judgement": "void_positive_control"}
                    else:
                        result = rejudge_the_line(fields, threshold)
                    key = line_key_of(fields)
                    r3_fields = fields if crt == CRT_FROM_R3 else r3_by_key.get(key)
                    r3_judgement = "absent" if r3_fields is None else r3_fields["judgement"]
                    r3_floor_judgement = "absent" if r3_fields is None else r3_fields["floor_judgement"]
                    row = dict(zip(LINE_KEY_FIELDS, key))
                    row.update(result)
                    row.update({"writing": writing, "threshold_source": source, "crt_source": crt, "key": key,
                                "r3_judgement": r3_judgement, "r3_floor_judgement": r3_floor_judgement,
                                "changed": "not_applicable" if r3_fields is None else str(result["judgement"] != r3_judgement).lower(),
                                "floor_changed": "not_applicable" if r3_fields is None else str(result["floor_judgement"] != r3_floor_judgement).lower()})
                    rows.append(row)
    return rows


def rejudged_line(row):
    keys = " ".join(f"{name}={row[name]}" for name in LINE_KEY_FIELDS)
    return (f"name=baseline_rejudge_point writing={row['writing']} threshold_source={row['threshold_source']} crt_source={row['crt_source']} "
            f"{keys} threshold_milliseconds={threshold_text(row['threshold'])} judgement_per_round={row['judgement_per_round']} "
            f"judgement={row['judgement']} r3_judgement={row['r3_judgement']} changed={row['changed']} "
            f"floor_judgement_per_round={row['floor_judgement_per_round']} floor_judgement={row['floor_judgement']} "
            f"r3_floor_judgement={row['r3_floor_judgement']} floor_changed={row['floor_changed']}")


def failure_clause_lines(rejudged, flips, today_outside):
    """第十条 L1（换了写法之后候选成批变赔）、L2（写法的两端定输赢）、L3（门槛来源定输赢）、L9（今天的写法这一趟离 r2 远）的触发。"""
    to_losing = {}
    for row in rejudged:
        key = (row["writing"], row["threshold_source"], row["crt_source"])
        to_losing[key] = to_losing.get(key, 0) + int(row["changed"] == "true" and row["judgement"] == "losing" and is_not_losing(row["r3_judgement"]))
    lines = [f"name=failure_clause clause=L1 writing={writing} threshold_source={source} crt_source={crt} changed_to_losing={count} triggered={str(count > 0).lower()}"
             for (writing, source, crt), count in sorted(to_losing.items())]
    for clause, knobs in (("L2", ("hardware_crc_lanes", "identity_index_build")), ("L3", ("threshold_source",))):
        flipped = sum(count for knob, _one, _other, _candidate, _pairs, count in flips if knob in knobs)
        lines.append(f"name=failure_clause clause={clause} knobs={','.join(knobs)} flipped_pairs={flipped} triggered={str(flipped > 0).lower()}")
    lines.append(f"name=failure_clause clause=L9 points_outside_the_band={today_outside} triggered={str(today_outside > 0).lower()}")
    return lines


def run(arguments, out):
    """跑一遍，行追加进 out；交回退出码。"""
    options = options_of(arguments)
    if options is None:
        out.append("name=verdict segment=baseline_rejudge status=not_run reason=usage")
        return 2
    contents = {}
    for item in ("host_cpu_product", "stage_split_product", "writings_product", "attack_ledger", "drift_control_product"):
        if item in options:
            try:
                with open(options[item], "rb") as handle:
                    contents[item] = handle.read()
            except OSError as error:
                out.append(f"name=verdict segment=baseline_rejudge status=not_run reason=input_unreadable:{item}:{type(error).__name__}")
                return 2
    lines, findings = input_file_lines(contents, options)
    out.extend(lines)
    if not findings:
        try:
            lines, findings = rejudge(contents)
        except (Unreadable, KeyError, ValueError, UnicodeDecodeError) as error:
            out.append(f"name=verdict segment=baseline_rejudge status=not_run reason=product_unreadable:{type(error).__name__}")
            return 2
        out.extend(lines)
    if findings:
        clause = {"clause_k_s3": "K-S3", "clause_k_s4": "K-S4", "clause_k_s5": "K-S5"}
        for finding in findings:
            out.append(f"name=stop_clause clause={clause.get(finding[:11], 'K-S3')} finding={finding}")
        out.append(f"name=verdict segment=baseline_rejudge stop_clause_findings={len(findings)}")
        return 3
    return 0


def main():
    out = []
    code = run(sys.argv[1:], out)
    out.append(f"name=done emitted={len(out) + 1}")
    sys.stdout.write("".join(f"E7RESULT {line}\n" for line in out))
    if code == 2 and out[0].endswith("reason=usage"):
        sys.stderr.write(__doc__)
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main())
