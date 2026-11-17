#!/usr/bin/env python3
# admission: always 每一次调都比此刻点名的两遍产物，上一次的结论不替这一次作保
# run-condition: none 只读点名的产物文本与同目录装置源码的常量表，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置
"""E173 第 2 次重跑（research/prompts/e173-r2-prereg.md）第六节 R1–R9 的比法：留存产物与今天现跑的产物逐行比。

只读文本，独立手写，不 import crates/ 任何东西。分类门槛跑前写死在登记第一节「读法写死」与第六节：
  仅差结局名的行 = 新旧两行把各自所有「含 NoSpace 的段」整段换成同一占位符之后逐字相同；其余的行不同都是「别的字段不同」。
用法：
  e173_r2_compare.py --new <第一遍产物或 REPLAY_OUT 目录> [--second <第二遍产物或目录>] [--task-log <任务日志>]…
                     [--old <留存产物>] [--source <第二段装置源码>]
      目录取其中的 E173.1.out（replay.sh --item E173.4 真写出的名字，原始输出含 E7INPUT 准入头）；也可直接给文件路径。
      每行 E7RESULT name=<量> 键=值，末行 name=done。退出码：0 算完；2 用法或产物读不了；
      3 留存产物 sha256 或行数不是登记第四节第 3 条的值；4 第二段装置源码里的几何常量表读不出九行。
  e173_r2_compare.py --selftest [--scratch <目录>]
      在目录里（缺省临时目录）拿留存产物造登记第九节 C1–C4 四份输入，核分类，再逐条打开四条弄坏写法核它们在各自输入上翻面；退 1 有红。
"""
import difflib
import hashlib
import os
import re
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REPO = os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
STORED_PATH = os.path.join(REPO, "research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out")
SOURCE_PATH = os.path.join(REPO, "scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs")
# 登记第四节第 3 条：留存产物
STORED_SHA256 = "85bed2435162a532566e64a1fc73bcaf4ab1f8c6a26b1b7a03c76135c89b07d9"
STORED_LINES = 3523
OLD_OUTCOME = "NoSpaceAfterRaisingTheFloor"
NEW_OUTCOME = "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease"
PLACEHOLDER = "<OUTCOME>"
# replay.sh --item E173.4 把产物写成「本趟记号」E173.1 的 E173.1.out（任务日志里「--item E173.4：本趟记号 E173.1」），不是 E173.4.out
ITEM_OUTPUT_NAME = "E173.1.out"

# 登记第四节第 2 条：实验页第 1 次重跑一节引的第一段数（比对对象，不是门槛）
PAGE_X3 = {  # 几何 -> (kappa, c实 最大, n代, n条)
    "s4_g_min_64mib": (18, 6, 3, 22), "s4_g_1": (130, 6, 21, 153), "s4_g_4": (543, 10, 54, 381),
    "s8_g_min_64mib": (18, 6, 3, 22), "s8_g_1": (130, 6, 21, 153), "s8_g_4": (543, 10, 54, 381),
}
PAGE_G_MIN = {"4": "s4_g_min_64mib", "8": "s8_g_min_64mib", "16": "s16_g_min_128mib"}
PAGE_EXCESS_2 = {"s8_g_min_64mib", "s16_g_min_128mib", "s16_g_1", "s16_g_4"}
KINDS_OF_KAPPA = ("minimum_on_versions_with_trees", "maximum_on_versions_with_trees", "final_version")

# 登记第六节 R1 沿用 replay.sh 的三个 strip 函数（replay.sh strip_admission_header / strip_load_fields / strip_timing）
TIMING_NAMES = ("per_sec_milli|median_per_sec_milli|spread_bp|min|max|ratio_bp|years_at_sync1|years_at_sync8|sync1_per_sec_milli|nosync_per_sec_milli|elapsed_ns|verify_ns|ns_per_op|ns_per_lookup|lookups_per_s|ns_small|ns_big|bing_median|jia_median|ratio_median|ratio_min|ratio_max|e128_median|deviation|ratio|one_shot_ns|two_phase_ns|two_phase_plus_5ms_ns|injected_recovered_ns|spread_one_shot|spread_two_phase|per_round_ns|best_ns|t1_ns|t16_ns|mibs|mib_per_s|entries_per_s|gbps|peak_gbps|speedup|threads16_speedup|dev|secs|copy_ns|r_rand_qd1|r_seq|random_ns_per_byte_padded|random_ns_per_byte_h133|seq_ns_per_byte_padded|seq_ns_per_byte_h133|crossover_random_share")
TIMING_RE = re.compile(r"(" + TIMING_NAMES + r")=[^ ]*")
LOAD_RE = re.compile(r" (loadavg1|cpus)=[^ ]*")
SEGMENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_:]*")
NUMERIC_SEGMENT_RE = re.compile(r"[A-Za-z0-9_:]+")

BREAKS = ("zero-diff-as-1", "no-placeholder", "numeric-as-placeholder", "no-row-count")


class Unreadable(Exception):
    pass


def read_lines(path):
    try:
        with open(path, "r", encoding="utf-8", newline="") as handle:
            return handle.read().split("\n")[:-1]
    except OSError as error:
        raise Unreadable(f"读不了 {path}：{error}")


def resolve_product(path):
    if os.path.isdir(path):
        path = os.path.join(path, ITEM_OUTPUT_NAME)
    if not os.path.isfile(path):
        raise Unreadable(f"没有这份产物 {path}（目录里要有 {ITEM_OUTPUT_NAME}）")
    return path


def stripped(lines, timing):
    """返回 [(原文件行号, 去掉准入头、负载字段（、计时字段）之后的行)]。"""
    kept = []
    for number, line in enumerate(lines, 1):
        if line.startswith("E7INPUT "):
            continue
        if line.startswith("E7RESULT name=done "):
            line = LOAD_RE.sub("", line)
        if timing:
            line = TIMING_RE.sub(lambda match: match.group(1) + "=X", line)
        kept.append((number, line))
    return kept


def fields_of(line):
    pairs = []
    for token in line.split(" "):
        if "=" in token:
            key, value = token.split("=", 1)
            pairs.append((key, value))
    return pairs


def outcome_segments(value, breaks):
    """一个字段值里的「结局名」段：字母或下划线起头、只含字母数字下划线冒号的最长一段，且含 NoSpace。"""
    found = []
    if "numeric-as-placeholder" in breaks:  # 弄坏写法：占位符范围扩到全部含数字的段（数字起头的也算）
        for match in NUMERIC_SEGMENT_RE.finditer(value):
            text = match.group(0)
            if "NoSpace" in text or any(ch.isdigit() for ch in text):
                found.append(text)
        return found
    for match in SEGMENT_RE.finditer(value):
        text = match.group(0)
        if "NoSpace" in text:
            found.append(text)
    return found


def masked(line, breaks):
    if "no-placeholder" in breaks:
        return line
    out = []
    for token in line.split(" "):
        if "=" in token:
            key, value = token.split("=", 1)
            for text in sorted(set(outcome_segments(value, breaks)), key=len, reverse=True):
                value = value.replace(text, PLACEHOLDER)
            token = key + "=" + value
        out.append(token)
    return " ".join(out)


def differing_fields(old_line, new_line):
    old_fields, new_fields = fields_of(old_line), fields_of(new_line)
    if [key for key, _ in old_fields] != [key for key, _ in new_fields]:
        old_keys, new_keys = [k for k, _ in old_fields], [k for k, _ in new_fields]
        listing = sorted(set(old_keys) ^ set(new_keys)) or ["字段次序"]
        return [(key, "(字段增减)", "(字段增减)") for key in listing]
    return [(ok, ov, nv) for (ok, ov), (_, nv) in zip(old_fields, new_fields) if ov != nv]


def align(old, new):
    """old、new 是 [(行号, 行)]；返回 (配对的不同行 [(旧, 新)], 无对应的旧行, 无对应的新行)。"""
    matcher = difflib.SequenceMatcher(None, [line for _, line in old], [line for _, line in new], autojunk=False)
    pairs, only_old, only_new = [], [], []
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            continue
        shared = min(i2 - i1, j2 - j1)
        for offset in range(shared):
            pairs.append((old[i1 + offset], new[j1 + offset]))
        only_old.extend(old[i1 + shared:i2])
        only_new.extend(new[j1 + shared:j2])
    return pairs, only_old, only_new


def classify(old_lines, new_lines, timing, breaks):
    old, new = stripped(old_lines, timing), stripped(new_lines, timing)
    pairs, only_old, only_new = align(old, new)
    rows = []
    for (old_number, old_line), (new_number, new_line) in pairs:
        if masked(old_line, breaks) == masked(new_line, breaks):
            kind, fields = "outcome_name_only", []
            old_seg = [s for _, v in fields_of(old_line) for s in outcome_segments(v, breaks)]
            new_seg = [s for _, v in fields_of(new_line) for s in outcome_segments(v, breaks)]
            fields = [("outcome", ",".join(old_seg), ",".join(new_seg))]
        else:
            kind, fields = "other_fields", differing_fields(old_line, new_line)
            if not fields:
                fields = [("line", old_line, new_line)]
        rows.append({"old_line": old_number, "new_line": new_number, "kind": kind, "fields": fields})
    others = [row for row in rows if row["kind"] == "other_fields"]
    if not rows:
        verdict = "1" if "zero-diff-as-1" in breaks else "premise_not_met"
    elif others:
        verdict = "2"
    else:
        verdict = "1"
    return {"rows": rows, "only_old": only_old, "only_new": only_new, "class": verdict,
            "diff_lines": 2 * len(rows) + len(only_old) + len(only_new)}


def count_results(lines):
    return sum(1 for line in lines if line.startswith("E7RESULT"))


def done_n(lines):
    values = [int(m.group(1)) for line in lines for m in [re.match(r"E7RESULT name=done emitted=(\d+)", line)] if m]
    return values[-1] if values else None


def row_count_check(old_lines, new_lines, breaks):
    old_stripped, new_stripped = stripped(old_lines, False), stripped(new_lines, False)
    old_n, new_n = done_n(old_lines), done_n(new_lines)
    differs = len(old_stripped) != len(new_stripped) or old_n != new_n
    if "no-row-count" in breaks:
        differs = False
    return {"old_rows": len(old_stripped), "new_rows": len(new_stripped), "old_n": old_n, "new_n": new_n,
            "differs": differs, "new_n_equals_results": new_n == count_results(new_lines),
            "new_results": count_results(new_lines)}


def lines_named(lines, name):
    return [(n, l) for n, l in stripped(lines, False) if l.startswith(f"E7RESULT name={name} ")]


def value_of(line, key):
    for k, v in fields_of(line):
        if k == key:
            return v
    return None


def by_identity(lines, name, keys):
    table = {}
    for number, line in lines_named(lines, name):
        table[tuple(value_of(line, k) for k in keys)] = (number, line)
    return table


def compare_rows(old_lines, new_lines, name, keys):
    """同一身份（name 加 keys）的行整行比：返回 (相同数, [(身份, [(字段, 旧, 新)])], 缺的身份, 多的身份)。"""
    old_table, new_table = by_identity(old_lines, name, keys), by_identity(new_lines, name, keys)
    same, changed = 0, []
    for identity, (_, old_line) in old_table.items():
        if identity not in new_table:
            continue
        new_line = new_table[identity][1]
        if old_line == new_line:
            same += 1
        else:
            changed.append((identity, differing_fields(old_line, new_line) or [("line", old_line, new_line)]))
    return same, changed, sorted(set(old_table) - set(new_table)), sorted(set(new_table) - set(old_table))


def x3_numbers(lines):
    """几何 -> (kappa, c实 最大, n代, n条)，取 minimum_on_versions_with_trees 那一行。"""
    result = {}
    for _, line in lines_named(lines, "x3_reserve_fit"):
        if value_of(line, "kappa_of_the_geometry") == "minimum_on_versions_with_trees":
            try:
                result[value_of(line, "geometry")] = tuple(int(value_of(line, k)) for k in (
                    "kappa", "largest_actual_empty_publish_slots", "code_empty_publishes_held", "clause_empty_publishes_held"))
            except (TypeError, ValueError):
                result[value_of(line, "geometry")] = None
    return result


def judge_r3(old_lines, new_lines):
    same, changed, missing, extra = compare_rows(old_lines, new_lines, "x3_reserve_fit", ("geometry", "kappa_of_the_geometry"))
    numbers = x3_numbers(new_lines)
    page = {g: (g in numbers and numbers[g] == v) for g, v in PAGE_X3.items()}
    claims = {
        "g_min_n_code_3_3_4": [numbers.get(g, (0, 0, None, 0))[2] for g in ("s4_g_min_64mib", "s8_g_min_64mib", "s16_g_min_128mib")] == [3, 3, 4],
        "other_six_n_code_ge_21": all(numbers.get(g) is not None and numbers[g][2] >= 21 for g in numbers if "g_min" not in g) and len([g for g in numbers if "g_min" not in g]) == 6,
        "nine_n_clause_ge_22": len(numbers) == 9 and all(v is not None and v[3] >= 22 for v in numbers.values()),
    }
    unchanged = not changed and not missing and not extra and same > 0
    return {"same": same, "changed": changed, "missing": missing, "extra": extra, "geometries": len(numbers),
            "verdict": "照旧" if unchanged else "变了", "page": page, "claims": claims}


def judge_r4(old_lines, new_lines):
    chosen = {value_of(l, "slots_per_region"): value_of(l, "chosen") for _, l in lines_named(new_lines, "g_min")}
    old_chosen = {value_of(l, "slots_per_region"): value_of(l, "chosen") for _, l in lines_named(old_lines, "g_min")}
    trials, trial_changed, trial_missing, trial_extra = compare_rows(old_lines, new_lines, "g_min_trial", ("geometry",))
    return {"chosen": chosen, "old_chosen": old_chosen, "verdict": "照旧" if chosen == PAGE_G_MIN else "G-min 变了",
            "trial_same": trials, "trial_changed": trial_changed, "trial_missing": trial_missing, "trial_extra": trial_extra}


def judge_r5(old_lines, new_lines):
    new_x5 = {value_of(l, "geometry"): value_of(l, "excess_slots") for _, l in lines_named(new_lines, "x5_overwrite_fixed_point")}
    holds = len(new_x5) == 9 and all((v == "2") == (g in PAGE_EXCESS_2) for g, v in new_x5.items())
    same, changed, missing, extra = compare_rows(old_lines, new_lines, "x5_overwrite_fixed_point", ("geometry",))
    return {"excess": new_x5, "verdict": "照旧" if holds else "变了", "same_as_stored": same, "changed": changed,
            "missing": missing, "extra": extra}


def judge_r6(old_lines, new_lines):
    old_rows = [l for _, l in lines_named(old_lines, "x3_reserve_fit") if value_of(l, "geometry") == "s16_g_1"]
    new_rows = [l for _, l in lines_named(new_lines, "x3_reserve_fit") if value_of(l, "geometry") == "s16_g_1"]
    n_code = {value_of(l, "code_empty_publishes_held") for l in new_rows}
    same = bool(new_rows) and old_rows == new_rows
    changed = []
    if old_rows and new_rows and not same:
        for old_line, new_line in zip(old_rows, new_rows):
            changed.extend(differing_fields(old_line, new_line))
    return {"rows": len(new_rows), "verdict": "照旧" if same else "变了", "n_code": sorted(n_code), "changed": changed}


def parse_second_segment_table(source_path):
    try:
        with open(source_path, "r", encoding="utf-8") as handle:
            text = handle.read()
    except OSError as error:
        raise Unreadable(f"读不了 {source_path}：{error}")
    start = text.find("const CHECKPOINT_COST_GEOMETRIES")
    if start < 0:
        raise Unreadable("第二段装置源码里没有 CHECKPOINT_COST_GEOMETRIES 常量表")
    end = text.find("\n];", start)
    block = text[start:end if end > 0 else start + 9000]
    pattern = re.compile(r"slots_per_region:\s*(\d+),\s*pool_size:\s*SampledPoolSize::(\w+)(?:\s*\{\s*mebibytes:\s*(\d+)\s*\})?,\s*"
                         r"largest_actual_empty_publish_slots:\s*(\d+),\s*checkpoint_cost_slots:\s*(\d+),")
    table = []
    for slots, pool, mebibytes, c_actual, kappa in pattern.findall(block):
        label = f"s{slots}_g_min_{mebibytes}mib" if pool == "SmallestCompleting" else (f"s{slots}_g_1" if pool == "OneGibibyte" else f"s{slots}_g_4")
        table.append((label, int(slots), int(c_actual), int(kappa)))
    if len(table) != 9:
        raise Unreadable(f"第二段几何常量表读出 {len(table)} 行，不是 9 行")
    return table


def judge_r7(new_lines, table):
    numbers = x3_numbers(new_lines)
    differences = []
    for label, slots, c_actual, kappa in table:
        row = numbers.get(label)
        if row is None:
            differences.append((label, "几何", "表里有", "新产物没有这个几何的 x3 行"))
            continue
        if row[0] != kappa:
            differences.append((label, "kappa", str(kappa), str(row[0])))
        if row[1] != c_actual:
            differences.append((label, "c_actual_max", str(c_actual), str(row[1])))
        if not label.startswith(f"s{slots}_"):
            differences.append((label, "S", str(slots), label))
    extra = sorted(set(numbers) - {label for label, *_ in table})
    for label in extra:
        differences.append((label, "几何", "表里没有", "新产物多出这个几何"))
    return {"differences": differences, "verdict": "第二段输入没变" if not differences else "第二段输入变了"}


def count_guards(lines):
    patterns = ("name=void", "name=stop", "geometry_void")
    return sum(1 for line in lines if any(p in line for p in patterns)), sum(1 for line in lines if "NOT_RUN" in line)


def compare(old_lines, new_lines, second_lines=None, task_logs=None, table=None, breaks=()):
    registered = classify(old_lines, new_lines, True, breaks)
    unmasked = classify(old_lines, new_lines, False, breaks)
    rows = row_count_check(old_lines, new_lines, breaks)
    candidate = "2" if ("2" in (registered["class"], unmasked["class"]) or rows["differs"]) else registered["class"]
    result = {"registered": registered, "unmasked": unmasked, "rows": rows, "candidate": candidate,
              "new_outcome_lines": sum(1 for line in new_lines if NEW_OUTCOME in line),
              "r3": judge_r3(old_lines, new_lines), "r4": judge_r4(old_lines, new_lines), "r5": judge_r5(old_lines, new_lines),
              "r6": judge_r6(old_lines, new_lines)}
    if table is not None:
        result["r7"] = judge_r7(new_lines, table)
    result["r8"] = None
    if task_logs is not None:
        product = count_guards(new_lines)
        logs = [count_guards(log) for log in task_logs]
        result["r8"] = {"product": product, "logs": logs, "total": product[0] + product[1] + sum(a + b for a, b in logs)}
    if second_lines is not None:
        a, b = stripped(new_lines, True), stripped(second_lines, True)
        result["r9"] = {"deterministic": [l for _, l in a] == [l for _, l in b], "rows": (len(a), len(b)),
                        "diff_lines": 2 * len(align(a, b)[0]) + len(align(a, b)[1]) + len(align(a, b)[2])}
    return result


def format_fields(fields):
    return ";".join(f"{key}:{old}->{new}" for key, old, new in fields)


def report(result):
    out = []
    for label, part in (("r1_registered", result["registered"]), ("r1_unmasked", result["unmasked"])):
        out.append(f"E7RESULT name={label} class={part['class']} different_pairs={len(part['rows'])} outcome_name_only={sum(1 for r in part['rows'] if r['kind'] == 'outcome_name_only')} "
                   f"other_fields={sum(1 for r in part['rows'] if r['kind'] == 'other_fields')} unpaired_old={len(part['only_old'])} unpaired_new={len(part['only_new'])} diff_lines={part['diff_lines']}")
        for row in part["rows"]:
            out.append(f"E7RESULT name={label}_row old_line={row['old_line']} new_line={row['new_line']} kind={row['kind']} fields={format_fields(row['fields'])}")
        for number, line in part["only_old"]:
            out.append(f"E7RESULT name={label}_unpaired side=old line={number}")
        for number, line in part["only_new"]:
            out.append(f"E7RESULT name={label}_unpaired side=new line={number}")
    rows = result["rows"]
    out.append(f"E7RESULT name=r2_rows old_rows={rows['old_rows']} new_rows={rows['new_rows']} old_n={rows['old_n']} new_n={rows['new_n']} "
               f"differs={str(rows['differs']).lower()} new_n_equals_results={str(rows['new_n_equals_results']).lower()} new_results={rows['new_results']}")
    out.append(f"E7RESULT name=rf5 new_outcome_lines={result['new_outcome_lines']} path_run={str(result['new_outcome_lines'] > 0).lower()}")
    r3 = result["r3"]
    out.append(f"E7RESULT name=r3 verdict={r3['verdict']} geometries={r3['geometries']} same_rows={r3['same']} changed_rows={len(r3['changed'])} missing={len(r3['missing'])} extra={len(r3['extra'])} "
               f"page_x3_holds={','.join(g + '=' + str(v).lower() for g, v in r3['page'].items())} "
               f"page_claims={','.join(k + '=' + str(v).lower() for k, v in r3['claims'].items())}")
    for identity, fields in r3["changed"]:
        out.append(f"E7RESULT name=r3_changed geometry={identity[0]} kappa_kind={identity[1]} fields={format_fields(fields)}")
    r4 = result["r4"]
    out.append(f"E7RESULT name=r4 verdict={r4['verdict']} chosen={','.join(k + ':' + str(v) for k, v in sorted(r4['chosen'].items()))} "
               f"trial_same={r4['trial_same']} trial_changed={len(r4['trial_changed'])} trial_missing={len(r4['trial_missing'])} trial_extra={len(r4['trial_extra'])}")
    for identity, fields in r4["trial_changed"]:
        out.append(f"E7RESULT name=r4_trial_changed geometry={identity[0]} fields={format_fields(fields)}")
    r5 = result["r5"]
    out.append(f"E7RESULT name=r5 verdict={r5['verdict']} excess={','.join(g + ':' + str(v) for g, v in r5['excess'].items())} same_as_stored={r5['same_as_stored']} changed={len(r5['changed'])}")
    for identity, fields in r5["changed"]:
        out.append(f"E7RESULT name=r5_changed geometry={identity[0]} fields={format_fields(fields)}")
    r6 = result["r6"]
    out.append(f"E7RESULT name=r6 verdict={r6['verdict']} rows={r6['rows']} n_code={','.join(r6['n_code'])} changed={format_fields(r6['changed'])}")
    if "r7" in result:
        r7 = result["r7"]
        out.append(f"E7RESULT name=r7 verdict={r7['verdict']} differences={len(r7['differences'])}")
        for label, field, table_value, new_value in r7["differences"]:
            out.append(f"E7RESULT name=r7_difference geometry={label} field={field} table={table_value} new={new_value}")
    if result["r8"] is not None:
        r8 = result["r8"]
        out.append(f"E7RESULT name=r8 product_guard_lines={r8['product'][0]} product_not_run={r8['product'][1]} "
                   f"log_guard_not_run={'|'.join(f'{a}+{b}' for a, b in r8['logs'])} total={r8['total']} ok={str(r8['total'] == 0).lower()}")
    if "r9" in result:
        r9 = result["r9"]
        out.append(f"E7RESULT name=r9 deterministic={str(r9['deterministic']).lower()} rows={r9['rows'][0]}/{r9['rows'][1]} diff_lines={r9['diff_lines']}")
    candidate = result["candidate"]
    if candidate == "1" and result["new_outcome_lines"] == 0:
        candidate = "1_but_new_name_path_not_run"
    out.append(f"E7RESULT name=r1_candidate candidate={candidate} registered_class={result['registered']['class']} unmasked_class={result['unmasked']['class']} row_count_differs={str(rows['differs']).lower()}")
    out.append(f"E7RESULT name=done emitted={len(out)}")
    return out


def run_compare(options):
    old_path = options.get("old", STORED_PATH)
    old_lines = read_lines(old_path)
    if old_path == STORED_PATH or "old" not in options:
        with open(old_path, "rb") as handle:
            digest = hashlib.sha256(handle.read()).hexdigest()
        if digest != STORED_SHA256 or len(old_lines) != STORED_LINES:
            print(f"  ✗ 留存产物 sha256={digest} 行数={len(old_lines)}，不是登记第四节第 3 条的 {STORED_SHA256} 与 {STORED_LINES}", file=sys.stderr)
            return 3
    new_lines = read_lines(resolve_product(options["new"]))
    second = read_lines(resolve_product(options["second"])) if "second" in options else None
    logs = [read_lines(path) for path in options.get("task_logs", [])] or None
    try:
        table = parse_second_segment_table(options.get("source", SOURCE_PATH))
    except Unreadable as unreadable:
        print(f"  ✗ {unreadable}", file=sys.stderr)
        return 4
    print("\n".join(report(compare(old_lines, new_lines, second, logs, table))))
    return 0


# ── 自证：登记第九节 C1–C4 与四条弄坏写法 ──────────────────────────────────────

def write_lines(path, lines):
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write("\n".join(lines) + "\n")


def build_inputs(old_lines, scratch):
    c2 = [line.replace(OLD_OUTCOME, NEW_OUTCOME) if OLD_OUTCOME in line else line for line in old_lines]
    c3 = list(c2)
    first_x3 = next(i for i, line in enumerate(c3) if line.startswith("E7RESULT name=x3_reserve_fit "))
    c3[first_x3] = re.sub(r" kappa=(\d+)", lambda m: f" kappa={int(m.group(1)) + 1}", c3[first_x3], count=1)
    done = max(i for i, line in enumerate(c2) if line.startswith("E7RESULT name=done "))
    c4 = c2[:done - 1] + c2[done:]
    inputs = {"C1": list(old_lines), "C2": c2, "C3": c3, "C4": c4}
    for name, lines in inputs.items():
        write_lines(os.path.join(scratch, f"{name.lower()}.out"), lines)
    return inputs


def selftest(scratch):
    old_lines = read_lines(STORED_PATH)
    with open(STORED_PATH, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    table = parse_second_segment_table(SOURCE_PATH)
    results = []

    def check(label, condition, detail=""):
        results.append((label, bool(condition), detail))

    check("留存产物 sha256 与行数是登记第四节第 3 条的值", digest == STORED_SHA256 and len(old_lines) == STORED_LINES, f"{digest} {len(old_lines)}")
    inputs = build_inputs(old_lines, scratch)
    check("留存产物里旧结局名的行恰 2 行（登记第四节第 1 条）", sum(1 for l in old_lines if OLD_OUTCOME in l) == 2)
    check("C4 比 C2 少 1 行", len(inputs["C4"]) == len(inputs["C2"]) - 1)
    run = {name: compare(old_lines, lines, None, None, table) for name, lines in inputs.items()}
    check("C1 分类前提不成立（0 行不同）", run["C1"]["registered"]["class"] == "premise_not_met" and run["C1"]["candidate"] == "premise_not_met", run["C1"]["candidate"])
    check("C2 分类 ①：2 行配对、4 个 diff 行、全是仅差结局名", run["C2"]["candidate"] == "1" and run["C2"]["registered"]["diff_lines"] == 4
          and all(r["kind"] == "outcome_name_only" for r in run["C2"]["registered"]["rows"]), f"{run['C2']['candidate']} {run['C2']['registered']['diff_lines']}")
    c3_rows = [r for r in run["C3"]["registered"]["rows"] if r["kind"] == "other_fields"]
    check("C3 分类 ②，别的字段只有 kappa", run["C3"]["candidate"] == "2" and len(c3_rows) == 1 and [f[0] for f in c3_rows[0]["fields"]] == ["kappa"], str(c3_rows))
    check("C4 分类 ②（行数不等，配对的行仍全是仅差结局名）", run["C4"]["candidate"] == "2" and run["C4"]["rows"]["differs"]
          and run["C4"]["registered"]["class"] == "1", f"{run['C4']['candidate']} {run['C4']['rows']}")
    # 弄坏写法：各在自己那份输入上翻面
    flips = (("zero-diff-as-1", "C1", "premise_not_met", "1"), ("no-placeholder", "C2", "1", "2"),
             ("numeric-as-placeholder", "C3", "2", "1"), ("no-row-count", "C4", "2", "1"))
    for broken, name, before, after in flips:
        got = compare(old_lines, inputs[name], None, None, table, breaks=(broken,))["candidate"]
        check(f"弄坏写法 {broken} 在 {name} 上由 {before} 变 {after}", got == after and run[name]["candidate"] == before, f"正常 {run[name]['candidate']} 弄坏 {got}")
    # R3–R9 的路径
    c1 = run["C1"]
    check("C1 上 R3 照旧、九个几何、页上六行数与三条页上断言都成立",
          c1["r3"]["verdict"] == "照旧" and c1["r3"]["geometries"] == 9 and all(c1["r3"]["page"].values()) and all(c1["r3"]["claims"].values()), str(c1["r3"]["page"]) + str(c1["r3"]["claims"]))
    check("C1 上 R4、R5、R6 照旧、R7 第二段输入没变",
          c1["r4"]["verdict"] == "照旧" and c1["r5"]["verdict"] == "照旧" and c1["r6"]["verdict"] == "照旧" and c1["r7"]["verdict"] == "第二段输入没变", str(c1["r7"]))
    check("C3 上 R3 变了，点名 s4_g_min_64mib 的 kappa", run["C3"]["r3"]["verdict"] == "变了" and any(i[0] == "s4_g_min_64mib" and any(f[0] == "kappa" for f in fs) for i, fs in run["C3"]["r3"]["changed"]), str(run["C3"]["r3"]["changed"]))
    check("C3 上 R7 点名第二段输入变了（s4_g_min_64mib 的 kappa）", run["C3"]["r7"]["verdict"] == "第二段输入变了" and ("s4_g_min_64mib", "kappa", "18", "19") in run["C3"]["r7"]["differences"], str(run["C3"]["r7"]["differences"]))
    x5 = list(old_lines)
    index = next(i for i, l in enumerate(x5) if "name=x5_overwrite_fixed_point geometry=s8_g_min_64mib " in l)
    x5[index] = x5[index].replace("excess_slots=2", "excess_slots=0")
    check("改一个 x5 的 excess_slots 后 R5 变了", judge_r5(old_lines, x5)["verdict"] == "变了")
    hidden = list(old_lines)
    index = next(i for i, l in enumerate(hidden) if "name=x5_overwrite_fixed_point geometry=s4_g_4 " in l)
    hidden[index] = hidden[index].replace(" max=10 ", " max=11 ")
    run_hidden = compare(old_lines, hidden, None, None, table)
    check("x5 的 max= 被 strip_timing 抹成 X：登记那一套比不出，不抹的那一套比出，合判 ②",
          run_hidden["registered"]["class"] == "premise_not_met" and run_hidden["unmasked"]["class"] == "2" and run_hidden["candidate"] == "2", f"{run_hidden['registered']['class']} {run_hidden['unmasked']['class']}")
    check("R8：含 NOT_RUN 的日志合计不为 0；干净日志合计 0",
          compare(old_lines, inputs["C2"], None, [["E7RESULT name=x NOT_RUN"]], table)["r8"]["total"] == 1
          and compare(old_lines, inputs["C2"], None, [["ok"]], table)["r8"]["total"] == 0)
    with_header = ["E7INPUT name=input_fingerprint key=E173 x"] + inputs["C2"]
    check("新产物带 E7INPUT 准入头时 R1 照样判 ①（头先去掉）", compare(old_lines, with_header, None, None, table)["candidate"] == "1")
    check("R9：同一份两遍确定；两份不同不确定", compare(old_lines, inputs["C2"], list(inputs["C2"]), None, table)["r9"]["deterministic"]
          and not compare(old_lines, inputs["C2"], inputs["C3"], None, table)["r9"]["deterministic"])
    as_replay_writes = os.path.join(scratch, "as-replay-writes")
    only_item_number = os.path.join(scratch, "only-item-number")
    os.makedirs(as_replay_writes, exist_ok=True)
    os.makedirs(only_item_number, exist_ok=True)
    for directory, name in ((as_replay_writes, "E173.1.out"), (only_item_number, "E173.4.out")):
        with open(os.path.join(directory, name), "w", encoding="utf-8", newline="") as handle:
            handle.write("\n".join(inputs["C2"]) + "\n")
    try:
        resolved = resolve_product(as_replay_writes)
    except Unreadable:
        resolved = ""
    refused = False
    try:
        resolve_product(only_item_number)
    except Unreadable:
        refused = True
    check("目录里是 replay.sh 真写的 E173.1.out 就认；只有 E173.4.out 的目录不认（名字取 replay.sh 的本趟记号）",
          resolved == os.path.join(as_replay_writes, "E173.1.out") and refused, f"{resolved} refused={refused}")
    bad = 0
    for label, ok, detail in results:
        print(f"  {'✓' if ok else '✗'} {label}" + ("" if ok else f"    细节：{detail}"))
        bad += 0 if ok else 1
    print(f"自证 {len(results)} 项，红 {bad} 项")
    return 1 if bad else 0


def main(argv):
    options = {"task_logs": []}
    names = {"--new": "new", "--second": "second", "--old": "old", "--source": "source", "--task-log": "task_log", "--scratch": "scratch"}
    selftest_requested = False
    position = 0
    while position < len(argv):
        if argv[position] == "--selftest":
            selftest_requested = True
            position += 1
            continue
        if argv[position] not in names or position + 1 >= len(argv):
            print(__doc__)
            return 2
        key = names[argv[position]]
        if key == "task_log":
            options["task_logs"].append(argv[position + 1])
        else:
            options[key] = argv[position + 1]
        position += 2
    try:
        if selftest_requested:
            scratch = options.get("scratch") or tempfile.mkdtemp(prefix="e173-r2-compare-selftest-")
            os.makedirs(scratch, exist_ok=True)
            return selftest(scratch)
        if "new" not in options:
            print(__doc__)
            return 2
        return run_compare(options)
    except Unreadable as unreadable:
        print(f"  ✗ {unreadable}", file=sys.stderr)
        print("     → 怎么办：核命令行给的产物路径在、读得了，再同一条命令重跑", file=sys.stderr)
        return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
