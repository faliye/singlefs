#!/usr/bin/env python3
# admission: always 被 e165-replay-mutation-kill-bounds.sh 载入、每一趟照那一趟交进来的装置与基线现判，上一次的结论不替这一次作保
# run-condition: none 被 E165 第二段驱动载入的判法模块，自己不编、不读表；直接执行只打用法
"""E165（恢复重放有界穷举与变异杀伤界）第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 的判法模块：判一个变体的那几个函数。

驱动的按项判定请求里 judge: 指这里的函数（驱动的 JUDGE_FUNCTIONS，三套各两个）：
  不给 --profile   judge_variant 与 baseline_item_line
  --profile r3    judge_variant_r3 与 baseline_item_line_r3
  --profile r4    judge_variant_r4 与 baseline_item_line_r4
judge_variant* 拿驱动建副本、改坏、编出来的装置（编不过交 None），起它、读它的输出、按「红」的口径（驱动文件头讲口径的那一段）出这一项的产物行与红例账行；
baseline_item_line* 写基线那一项的产物行。判法闭包（scripts/gate/lib_function_units.py）收这几个函数、它们调到的单元与这份模块 import 时执行的语句，
所以这份模块守三样：
  一、不放自证、钉红例、选行与读写判定库的代码（都在驱动里），这里的函数也不调 lib_reuse_checks、e165-pin-red-examples.py、read_table 这类读整张表的函数：
      变异行原文经驱动交进来的 item 带到；
  二、模块级只写 import、常量与 def，不读文件、不起进程、不读环境变量；
  三、建副本、改坏、编译、拷副本的范围与副本里起命令的环境（scripts/hosts/lib_repository_copy.py）留在驱动：这里起装置时环境经 setup.command_environment() 取，
      副本还原与删除经驱动交进来的 restore() 做。
弄坏开关：BREAK 是驱动与这份模块共用的那一份集合，驱动照 E165_KILL_BOUNDS_BREAK 填（名字、弄坏哪一处、点名的格在驱动文件头「弄坏开关」）；这里认其中的
  noprofile、r3noprofile、r4noprofile、r4gateignored、j6qnfromj6、r3firstredswap、r3kapinunchecked、r4firstredswap、r4kgcountsasred、r4kgpinunchecked。
这份模块自己没有自证：判法由驱动的自证覆盖（起法在驱动文件头「用法」里自证那一条，经临时 prune-server 起）。
"""
import hashlib
import os
import re
import subprocess
import sys
import time

sys.dont_write_bytecode = True
BREAK = set()   # 驱动照 E165_KILL_BOUNDS_BREAK 填（只给驱动的自证证红用）；这份模块自己不读环境变量

# 按项判定的判法入口：驱动的 JUDGE_FUNCTIONS 按档取的就是这六个（驱动的自证核两边同名）；算键库自证从这一行现取、逐个收闭包
JUDGING_ENTRIES = ("judge_variant", "baseline_item_line", "judge_variant_r3", "baseline_item_line_r3", "judge_variant_r4", "baseline_item_line_r4")
BIN_NAME = "e165_replay_bounded_enumeration"
# 装置的档参数（r2 登记 5.1、9.4 第 2 条）：--bounds、--list-states、--replay-state 每一次都带。
DEVICE_PROFILE_ARGUMENTS = ["--profile", "r2"]
# r2 9.4 第 3 条：条款判据栏加 r2 档的新行，名字与装置的输出行同一组。
J6_ROWS = ("J6_premise_holds", "J6_premise_fails")
J6QN_ROWS = ("J6Qn_premise_holds", "J6Qn_premise_fails")
J7_ROWS = ("J7a_instance_table", "J7a_allocation_record_tree_root", "J7a_filesystem_identifier", "J7b", "J7c")
CLAUSE_CRITERIA = {"J0", "J1", "J2", "J3", "J4", *J6_ROWS, *J6QN_ROWS, *J7_ROWS, "A0", "AS_whole_table",
                   "AS_consecutive_gap_continues", "AS_consecutive_gap_stops", "A_structure", "A_limit"}
ANCHOR_CRITERIA = {"IA1", "IA2"}
# 登记第七节甲类锚点：这几个域里原实现报错的态数是 0。
DOMAINS_WHERE_FAILED_STATES_ARE_PINNED = {"L", "L6", "L9", "U_verify_on", "X", "E1"}
ADVERSARIAL_DOMAINS = {"A", "A6", "AC"}

# ── r3 档（登记 r3 9.3，第三段）的装置参数与判据栏。不给 --profile 是 r2 档，上面那一套一个字不动。
R3_DEVICE_PROFILE_ARGUMENTS = ["--profile", "r3"]
R3_BOUNDS_ARGUMENTS = ["--bounds", "segment1", "--family", "root_choice"]
R3_LEGAL_DOMAINS = ("K", "K9", "KW")
R3_ALL_DOMAINS = ("K", "K9", "KW", "KA")
# 首红先报登记要的 KJ1、KJ2（TQ5），再附带 KJ0、KJ3
R3_CRITERIA = ("KJ1", "KJ2", "KJ0", "KJ3")
R3_ROW_CRITERIA = ("KJ0", "KJ1", "KJ2", "KJ3")

# ── r4 档（登记 r4 9.3，第二段）的装置参数与判据栏。不给 --profile 是 r2 档、--profile r3 是 r3 档，上面两套一个字不动。
R4_DEVICE_PROFILE_ARGUMENTS = ["--profile", "r4"]
R4_BOUNDS_ARGUMENTS = ["--bounds", "segment1", "--family", "root_choice_faults"]
R4_LEGAL_DOMAINS = ("KF", "KF3", "KF9", "KFW")
R4_ALL_DOMAINS = ("KF", "KF3", "KF9", "KFW", "KG", "KA")
# 首红先报登记要的 KJ1、KJ1T、KJ2（TQ9），再附带 KJ0、KJ3
R4_CRITERIA = ("KJ1", "KJ1T", "KJ2", "KJ0", "KJ3")
R4_ROW_CRITERIA = ("KJ0", "KJ1", "KJ1T", "KJ2", "KJ3")
R4_CLAUSE_CRITERIA = ("KJ1", "KJ1T", "KJ2")
# 首红的次序：同一 (M, U) 格里先按被判时故障（无、RN、RI、D0、D1），再按历史标签（罩着、不罩）（登记 TQ9）
R4_JUDGED_FAULTS = ("none", "RN", "RI", "D0", "D1")
R4_HISTORY_LABELS = ("inside_the_promise", "outside_the_promise")
R4_SUBCELLS = tuple((fault, label) for fault in R4_JUDGED_FAULTS for label in R4_HISTORY_LABELS)
# 变体登记名（r3 的编号，今天主工作区里在第 1821–1826 行）对 KG 钉值取哪一列：v1827 对 v1823 那一列，别的对各自那一列（登记 5.6、第七节）
R4_KG_COLUMN_OF = {"v1822": "v1822", "v1823": "v1823", "v1824": "v1824", "v1825": "v1825", "v1826": "v1826", "v1827": "v1823"}
# 第五段 5-乙（登记修订 1「修1.九」乙第 1–4 条）：--gate r2 只在 --profile r4 --pins r5 同给时认；调装置在 R4_BOUNDS_ARGUMENTS 之后加这两项。
R4_GATE_R2_ARGUMENTS = ["--gate", "r2"]


def mark(options, name, text):
    """在做的这一项走到哪一步（进度行里报）；没有进度的用法（--measure）什么都不做。"""
    progress = options.get("progress")
    if progress is not None:
        progress.phase(name, text)


def device_bounds_arguments(setup):
    """每次整份跑装置的界参数：r2 是 --bounds segment2；r3 是 --bounds segment1 --family root_choice（登记 r3 9.3 第 3 条）；
    r4 是 --bounds segment1 --family root_choice_faults，给了 --gate r2 时再加 --gate r2（登记 r5 修1.九乙第 1 条；弄坏开关 r4gateignored 不加）。"""
    if setup.profile == "r2":
        return ["--bounds", "segment2"]
    if setup.profile == "r3":
        return list(R3_BOUNDS_ARGUMENTS)
    gate = list(R4_GATE_R2_ARGUMENTS) if setup.gate == "r2" and "r4gateignored" not in BREAK else []
    return list(R4_BOUNDS_ARGUMENTS) + gate


def device_command_and_environment(setup, command):
    """跑编译出来的二进制：直接起，不套内存包装。command 第一项是二进制，档参数接在它后面（r2 9.4 第 2 条；r3 同法带 --profile r3）。
    环境由驱动的 setup.command_environment() 给（副本里起命令的多机配置那一层，不是判法）。"""
    if setup.profile == "r2":
        profile = [] if "noprofile" in BREAK else DEVICE_PROFILE_ARGUMENTS
    elif setup.profile == "r3":
        profile = [] if "r3noprofile" in BREAK else R3_DEVICE_PROFILE_ARGUMENTS
    else:
        profile = [] if "r4noprofile" in BREAK else R4_DEVICE_PROFILE_ARGUMENTS
    return command[:1] + profile + command[1:], setup.command_environment()


# ── 装置输出的行

def parse_row(line):
    """`E7RESULT name=… k=v …`：值里带空格的字段（witness、rerun），不带「键=」开头的词并进前一个字段。"""
    fields = {}
    last_key = None
    for token in line.split(" ")[1:]:
        if re.match(r"^[a-z_0-9]+=", token):
            key, value = token.split("=", 1)
            fields[key] = value
            last_key = key
        elif last_key is not None:
            fields[last_key] += " " + token
    return fields


def result_rows(text):
    return [line for line in text.split("\n") if line.startswith("E7RESULT ")]


def rows_by_place(rows):
    """(域, N) → {行的键: 字段}；只收带 domain= 与 n= 的行（每个 (域, N) 的量），判决行与自检、对照行不在内。"""
    places = {}
    order = []
    for line in rows:
        fields = parse_row(line)
        if "domain" not in fields or "n" not in fields or fields.get("name") == "state":
            continue
        place = (fields["domain"], int(fields["n"]))
        if place not in places:
            places[place] = {}
            order.append(place)
        key = (fields["name"], fields.get("criterion") or fields.get("reading") or fields.get("readings") or fields.get("set") or "")
        places[place][key] = fields
    return places, order


def differences_at(domain, baseline_fields, mutant_fields):
    """一个 (域, N) 上变体与基线不同的量，分三栏：条款判据、实现锚点、别的。"""
    clause, anchors, other = [], [], []
    for key, baseline in baseline_fields.items():
        mutant = mutant_fields.get(key)
        name, discriminator = key
        if mutant is None:
            other.append(f"{name}.{discriminator or 'row'}.missing")
            continue
        for field, baseline_value in baseline.items():
            if mutant.get(field) == baseline_value:
                continue
            if name == "violations" and field == "violations":
                if discriminator in CLAUSE_CRITERIA:
                    clause.append(discriminator)
                elif discriminator in ANCHOR_CRITERIA:
                    anchors.append(discriminator)
                else:
                    other.append(f"violations.{discriminator}")
            elif name == "pairs" and field in ("today_j5a", "today_j5b"):
                clause.append(f"{'J5a' if field == 'today_j5a' else 'J5b'}[{discriminator}]")
            elif name == "states" and field == "failed" and domain in DOMAINS_WHERE_FAILED_STATES_ARE_PINNED:
                anchors.append("failed_states")
            elif name == "states" and field == "panicked":
                pass  # 与 J0 / A0 那一行是同一个数
            else:
                other.append(f"{name}.{discriminator + '.' if discriminator else ''}{field}")
    for key in mutant_fields:
        if key not in baseline_fields:
            name, discriminator = key
            if name == "violations" and discriminator in ANCHOR_CRITERIA:
                anchors.append(discriminator)
            else:
                other.append(f"{name}.{discriminator or 'row'}.extra")
    return clause, anchors, other


def compare_with_baseline(baseline_rows, mutant_rows):
    baseline_places, order = rows_by_place(baseline_rows)
    mutant_places, _ = rows_by_place(mutant_rows)
    domains = []
    for domain, _size in order:
        if domain not in domains:
            domains.append(domain)
    per_domain = {}
    for domain in domains:
        sizes = [size for (place_domain, size) in order if place_domain == domain]
        entry = {"upper_bound": max(sizes), "clause": None, "anchor": None, "other": None, "missing": None}
        for size in sizes:
            if (domain, size) not in mutant_places:
                entry["missing"] = size
                break
            clause, anchors, other = differences_at(domain, baseline_places[(domain, size)], mutant_places[(domain, size)])
            if clause and entry["clause"] is None:
                entry["clause"] = (size, clause)
            if anchors and entry["anchor"] is None:
                entry["anchor"] = (size, anchors)
            if other and entry["other"] is None:
                entry["other"] = (size, other)
            if entry["clause"] is not None:
                break  # 红了就不再往上看这个域
        per_domain[domain] = entry
    return domains, per_domain


def violation_lines_by_place(rows):
    """(域, N) → {判据名: 那一行的整行文字}，只收 name=violations 行；域与 N 的次序照行出现的先后。"""
    places, order = {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") != "violations" or "domain" not in fields or "n" not in fields or "criterion" not in fields:
            continue
        place = (fields["domain"], int(fields["n"]))
        if place not in places:
            places[place] = {}
            order.append(place)
        places[place][fields["criterion"]] = line
    return places, order


def domains_in_order(order):
    domains = []
    for domain, _size in order:
        if domain not in domains:
            domains.append(domain)
    return domains


def first_red_by_criterion(baseline_rows, mutant_rows):
    """r2 9.4 第 4 条（SQ2）：J6 两行合看、J6Qn 两行合看，每个域各求最小的 N——那条判据任一行与基线同一 (域, N) 的同一行文字不同
    （变体缺那一行也算不同）；到上沿都相同记 None，基线在那个域上一行那条判据都没有记 "no_rows"（与「界内不红」分开）。整份比完，不早停。
    返回 {"J6": [(域, N、None 或 "no_rows")…], "J6Qn": […]}，域的次序照基线。弄坏开关 j6qnfromj6：J6Qn 取了 J6 那两行。"""
    baseline_places, order = violation_lines_by_place(baseline_rows)
    mutant_places, _ = violation_lines_by_place(mutant_rows)
    families = {"J6": J6_ROWS, "J6Qn": J6_ROWS if "j6qnfromj6" in BREAK else J6QN_ROWS}
    result = {}
    for family, criteria in families.items():
        per_domain = []
        for domain in domains_in_order(order):
            sizes = sorted(size for (place_domain, size) in order if place_domain == domain)
            first = None if any(criterion in baseline_places[(domain, size)] for size in sizes for criterion in criteria) else "no_rows"
            for size in sizes if first is None else []:
                baseline_lines = baseline_places[(domain, size)]
                mutant_lines = mutant_places.get((domain, size), {})
                if any(criterion in baseline_lines and mutant_lines.get(criterion) != baseline_lines[criterion] for criterion in criteria):
                    first = size
                    break
            per_domain.append((domain, first))
        result[family] = per_domain
    return result


def first_red_by_criterion_text(first_reds):
    return ";".join(f"{family}:" + ",".join(f"{domain}={'none' if size is None else size}" for domain, size in per_domain)
                    for family, per_domain in first_reds.items())


def first_text_red_criteria(baseline_rows, mutant_rows):
    """r2 9.4 第 4 条后半（SQ5）：每个域上任一条判据的 name=violations 行文字与基线不同的最小 N，与那个 N 上文字不同的全部判据名（基线的次序）。
    交 [(域, N, [判据名…])]，到上沿都相同的域不列。"""
    baseline_places, order = violation_lines_by_place(baseline_rows)
    mutant_places, _ = violation_lines_by_place(mutant_rows)
    found = []
    for domain in domains_in_order(order):
        for size in sorted(size for (place_domain, size) in order if place_domain == domain):
            baseline_lines = baseline_places[(domain, size)]
            mutant_lines = mutant_places.get((domain, size), {})
            differing = [criterion for criterion, line in baseline_lines.items() if mutant_lines.get(criterion) != line]
            differing += [criterion for criterion in mutant_lines if criterion not in baseline_lines]
            if differing:
                found.append((domain, size, differing))
                break
    return found


# ── 一个变体：起装置、读它的输出、比、出产物行与红例账行（建副本、改坏、编在驱动里）

def run_captured(command, environment, cwd=None, log_path=None):
    completed = subprocess.run(command, env=environment, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if log_path is not None:
        with open(log_path, "w", encoding="utf-8") as handle:
            handle.write(completed.stdout)
            handle.write("\n--- stderr ---\n")
            handle.write(completed.stderr)
    return completed


def run_segment_two(setup, binary, threads, log_directory):
    command, environment = device_command_and_environment(setup, [binary] + device_bounds_arguments(setup))
    environment["E165_THREADS"] = str(threads)
    started = time.monotonic()
    completed = run_captured(["nice", "-n", "19"] + command, environment, log_path=os.path.join(log_directory, "run.log"))
    return completed, time.monotonic() - started


def state_lines(setup, binary, kind, place):
    command, environment = device_command_and_environment(setup, [binary, "--list-states" if kind == "list-states" else "--replay-state", place])
    return subprocess.Popen(["nice", "-n", "19"] + command, env=environment, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)


def counters_of(line):
    match = re.search(r" counters=(\S+)", line)
    return match.group(1) if match else ""


def first_differing_state(setup, baseline_binary, mutant_binary, domain, size):
    """两边的逐态行照枚举次序逐行比：先找第一个「加了一的量」不同的状态；一个都没有时退到第一个整行不同的状态（只有结局不同）。"""
    place = f"{domain}:{size}"
    baseline_process = state_lines(setup, baseline_binary, "list-states", place)
    mutant_process = state_lines(setup, mutant_binary, "list-states", place)
    found = None
    only_outcome = None
    for baseline_line, mutant_line in zip(baseline_process.stdout, mutant_process.stdout):
        if baseline_line == mutant_line or " name=state " not in baseline_line:
            continue
        ordinal = parse_row(baseline_line.rstrip("\n")).get("ordinal")
        if counters_of(baseline_line) != counters_of(mutant_line):
            found = ("counters", ordinal, baseline_line.rstrip("\n"), mutant_line.rstrip("\n"))
            break
        if only_outcome is None:
            only_outcome = ("outcome_only", ordinal, baseline_line.rstrip("\n"), mutant_line.rstrip("\n"))
    for process in (baseline_process, mutant_process):
        process.stdout.close()  # 还在写的那一边收到管道关闭就退出
        process.wait()
    return found or only_outcome


def replayed_state_line(setup, binary, domain, size, ordinal):
    process = state_lines(setup, binary, "replay-state", f"{domain}:{size}:{ordinal}")
    output = process.stdout.read()
    process.stdout.close()
    process.wait()
    lines = [line for line in output.split("\n") if " name=state " in line]
    return lines[0] if len(lines) == 1 else None


def counter_difference(baseline_line, mutant_line):
    baseline = set(counters_of(baseline_line).split(","))
    mutant = set(counters_of(mutant_line).split(","))
    parts = [f"+{name}" for name in sorted(mutant - baseline)] + [f"-{name}" for name in sorted(baseline - mutant)]
    return ",".join(parts) if parts else "none"


def rerun_command(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh{options.get('pins_text', '')} --scratch <草稿目录> --output {options['output_shown']} "
            f"--ledger {options['ledger_shown']} --recheck --item {item['name']}")


def judge_variant(setup, item, baseline, options, binary, build_problem, log_directory, restore):
    """不给 --profile 那一套判一个变体：binary 是驱动建副本、改坏、编出来的装置（编不过是 None，build_problem 是编译的头一行错），起它跑 --bounds segment2
    （运行日志写进 log_directory）、逐 (域, N) 与基线比，红了的域在首红的 N 上找一个反例状态、两边各单条重放一遍，按「红」的口径出这一项的产物行与红例账行。
    restore() 由驱动给：把副本里被改的那一份还原、删掉副本与编译目录，交还原成没成（产物行的 copy_restored）。
    交 {lines, ledger, problem, run_seconds}；编译与这一项的挂钟由驱动补成 timing。"""

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}")

    def tail(title):
        return f"rerun=\"{rerun_command(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if binary is None:
        restored = restore()
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "run_seconds": 0.0}
    mark(options, item["name"], "跑 --bounds segment2")
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], log_directory)
    mutant_rows = result_rows(completed.stdout)
    domains, per_domain = compare_with_baseline(baseline["rows"], mutant_rows)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    clause_parts, anchor_parts, other_parts, not_red, missing_parts = [], [], [], [], []
    legal_minimum = adversarial_minimum = None
    ledger = []
    for domain in domains:
        entry = per_domain[domain]
        red_sizes = [found[0] for found in (entry["clause"], entry["anchor"]) if found is not None]
        if entry["clause"] is not None:
            clause_parts.append(f"{domain}:{entry['clause'][0]}:{'+'.join(entry['clause'][1])}")
        if entry["anchor"] is not None:
            anchor_parts.append(f"{domain}:{entry['anchor'][0]}:{'+'.join(entry['anchor'][1])}")
        if entry["other"] is not None and (not red_sizes or entry["other"][0] < min(red_sizes)):
            other_parts.append(f"{domain}:{entry['other'][0]}:{'+'.join(entry['other'][1])}")
        if entry["missing"] is not None:
            missing_parts.append(f"{domain}:{entry['missing']}")
        if not red_sizes:
            if entry["missing"] is None:
                not_red.append(f"{domain}(≤{entry['upper_bound']})")
            continue
        first_red = min(red_sizes)
        if domain in ADVERSARIAL_DOMAINS:
            adversarial_minimum = first_red if adversarial_minimum is None else min(adversarial_minimum, first_red)
        else:
            legal_minimum = first_red if legal_minimum is None else min(legal_minimum, first_red)
        # 红例：这个域首红的 N 上第一个判定不同的状态，两边各单条重放一遍。
        rows_text = "+".join(str(row) for row in item["rows"])
        mark(options, item["name"], f"在 {domain}:{first_red} 上找红例（两边 --list-states 逐行比）")
        example = first_differing_state(setup, baseline["binary"], binary, domain, first_red)
        if example is None:
            ledger.append({"domain": domain, "size": first_red, "found": False, "fields": [
                item["name"], rows_text, domain, str(first_red), "none", "no_single_state_differs", "none", "false", "false", "", "", ""]})
            continue
        kind, ordinal, baseline_line, mutant_line = example
        mark(options, item["name"], f"在 {domain}:{first_red} 上单条重放第 {ordinal} 个状态")
        on_original = replayed_state_line(setup, baseline["binary"], domain, first_red, ordinal)
        on_mutant = replayed_state_line(setup, binary, domain, first_red, ordinal)
        ledger.append({"domain": domain, "size": first_red, "found": True, "fields": [
            item["name"], rows_text, domain, str(first_red), ordinal, kind,
            counter_difference(baseline_line, mutant_line) if kind == "counters" else "outcome",
            str(on_original == baseline_line).lower(), str(on_mutant == mutant_line).lower(),
            f"target/release/{BIN_NAME} --replay-state {domain}:{first_red}:{ordinal}", baseline_line, mutant_line]})
    if clause_parts:
        outcome = "red"
    elif anchor_parts:
        outcome = "red_only_by_implementation_anchor"
    elif missing_parts or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    found_entries = [entry for entry in ledger if entry["found"]]
    first_of_variant = min(found_entries, key=lambda entry: (entry["size"], domains.index(entry["domain"]))) if found_entries else None
    for entry in ledger:
        entry["fields"].insert(9, str(entry is first_of_variant).lower())
    confirmed = sum(1 for entry in ledger if entry["fields"][7] == "true" and entry["fields"][8] == "true")
    restored = restore()
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    text_reds = first_text_red_criteria(baseline["rows"], mutant_rows)
    text_red_text = ";".join(f"{domain}:{size}:{'+'.join(criteria)}" for domain, size, criteria in text_reds) or "none"
    body = (f"outcome={outcome} first_clause_red={';'.join(clause_parts) or 'none'} first_anchor_red={';'.join(anchor_parts) or 'none'} "
            f"first_red_by_criterion={first_red_by_criterion_text(first_red_by_criterion(baseline['rows'], mutant_rows))} "
            f"first_text_red_criteria={text_red_text} "
            f"other_difference_before_any_red={';'.join(other_parts) or 'none'} not_red_domains={','.join(not_red) or 'none'} "
            f"legal_family_minimum={legal_minimum if legal_minimum is not None else 'none'} "
            f"adversarial_family_minimum={adversarial_minimum if adversarial_minimum is not None else 'none'} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} domains_without_rows={';'.join(missing_parts) or 'none'} "
            f"abort={abort} red_examples={len(ledger)} red_examples_confirmed_by_single_state_replay={confirmed} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome.startswith("red") else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": ["\t".join(entry["fields"]) for entry in ledger], "problem": problem, "run_seconds": run_seconds}


def baseline_item_line(item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_found_verbatim_in_segment1_products={baseline['verbatim_rows']} rerun=\"{rerun_command(item, options)}\" title=基线（不改） complete=yes")


def retained_comparison_text(baseline, compared_text):
    """基线那一行、收尾判决行里比对那一栏：留存产物不在、没比的写 not_run，比过的写 compared_text。"""
    return "not_run" if baseline.get("verbatim_rows") == "not_run" else compared_text


# ── r3 档（登记 r3 9.3，第三段）：整份装置输出的读法。判据是违例数，格是 (域, M, U)，KA 的 M、U 都写 0。

def r3_parse(rows):
    """装置 --family root_choice 的 E7RESULT 行 → (违例, 计数行, 格的先后次序)。
    违例：{格: {判据: (违例数, 整行)}}（只收 name=violations）；计数行：{格: (字段, 整行)}（name=root_choice_states）；格 = (域, M, U)。"""
    violations, states, order = {}, {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") not in ("violations", "root_choice_states") or not {"domain", "mounts", "user_publishes"} <= set(fields):
            continue
        cell = (fields["domain"], int(fields["mounts"]), int(fields["user_publishes"]))
        if cell not in order:
            order.append(cell)
        if fields["name"] == "violations":
            if "criterion" in fields:
                violations.setdefault(cell, {})[fields["criterion"]] = (int(fields["violations"]), line)
        else:
            states[cell] = (fields, line)
    return violations, states, order


def r3_cells_of(order, domain):
    """一个域的格，按 (M 升序, U 升序)。"""
    return sorted((mounts, publishes) for cell_domain, mounts, publishes in order if cell_domain == domain)


def r3_first_reds(baseline, mutant):
    """登记 TQ5：每个变体、每个合法历史域、每条判据，按 (M 升序, U 升序) 第一个违例数与基线不同的格（变体缺那一格或缺那一行也算不同）；
    到上沿都相同记 None；基线在那个域上一行那条判据都没有记 "no_rows"。整份比完，不早停。返回 {判据: [(域, (M, U) 或 None 或 "no_rows")…]}。
    弄坏开关 r3firstredswap：KJ1 与 KJ2 两列对调。"""
    baseline_violations, _states, baseline_order = baseline
    mutant_violations = mutant[0]
    result = {}
    for criterion in R3_CRITERIA:
        per_domain = []
        for domain in R3_LEGAL_DOMAINS:
            cells = r3_cells_of(baseline_order, domain)
            if not any(criterion in baseline_violations[(domain, m, u)] for m, u in cells):
                per_domain.append((domain, "no_rows"))
                continue
            first = None
            for m, u in cells:
                baseline_reading = baseline_violations[(domain, m, u)].get(criterion)
                mutant_reading = mutant_violations.get((domain, m, u), {}).get(criterion)
                if baseline_reading is not None and (mutant_reading is None or mutant_reading[0] != baseline_reading[0]):
                    first = (m, u)
                    break
            per_domain.append((domain, first))
        result[criterion] = per_domain
    if "r3firstredswap" in BREAK:
        result["KJ1"], result["KJ2"] = result["KJ2"], result["KJ1"]
    return result


def r3_first_red_text(first_reds):
    def one(value):
        return "none" if value is None else value if isinstance(value, str) else f"M{value[0]}U{value[1]}"
    return ";".join(f"{criterion}:" + ",".join(f"{domain}={one(value)}" for domain, value in per_domain) for criterion, per_domain in first_reds.items())


def r3_differing_cells_text(baseline, mutant):
    """每条判据、每个合法历史域：违例数与基线不同的格数（格数总数写在 of 后面）。"""
    baseline_violations, _states, baseline_order = baseline
    parts = []
    for criterion in R3_CRITERIA:
        per_domain = []
        for domain in R3_LEGAL_DOMAINS:
            cells = r3_cells_of(baseline_order, domain)
            differing = sum(1 for m, u in cells if criterion in baseline_violations[(domain, m, u)]
                            and mutant[0].get((domain, m, u), {}).get(criterion, (None,))[0] != baseline_violations[(domain, m, u)][criterion][0])
            per_domain.append(f"{domain}={differing}of{len(cells)}")
        parts.append(f"{criterion}:" + ",".join(per_domain))
    return ";".join(parts)


def r3_cells_text(violations, order):
    """每个格的 KJ0/KJ1/KJ2/KJ3 违例数（没有那一行写 -）：<域>.M<m>U<u>=a/b/c/d，格用逗号连（KA 的 M、U 是 0）。"""
    parts = []
    for domain in R3_ALL_DOMAINS:
        for m, u in r3_cells_of(order, domain):
            readings = violations.get((domain, m, u), {})
            parts.append(f"{domain}.M{m}U{u}=" + "/".join(str(readings[criterion][0]) if criterion in readings else "-" for criterion in R3_ROW_CRITERIA))
    return ",".join(parts) or "none"


def r3_ka_kj2(violations):
    """KA 上 KJ2 的违例数；KA 那一格或那一行不在交 None。"""
    for (domain, _m, _u), readings in violations.items():
        if domain == "KA" and "KJ2" in readings:
            return readings["KJ2"][0]
    return None


def r3_states_row_differences(baseline, mutant):
    """每个域：计数行与基线不同的第一个格，与那一格上不同的字段名；全相同的域不列。"""
    _violations, baseline_states, baseline_order = baseline
    mutant_states = mutant[1]
    found = []
    for domain in R3_ALL_DOMAINS:
        for m, u in r3_cells_of(baseline_order, domain):
            cell = (domain, m, u)
            if cell not in mutant_states:
                found.append(f"{domain}=M{m}U{u}:missing")
                break
            differing = sorted(key for key in baseline_states[cell][0] if mutant_states[cell][0].get(key) != baseline_states[cell][0][key])
            if differing:
                found.append(f"{domain}=M{m}U{u}:" + "+".join(differing))
                break
    return ";".join(found) or "none"


def r3_state_totals(states, field, how):
    """每个域上计数行某个字段的合计（how 是 sum 或 max）：K:a,K9:b,KW:c,KA:d。"""
    parts = []
    for domain in R3_ALL_DOMAINS:
        values = [int(fields.get(field, "0")) for (cell_domain, _m, _u), (fields, _line) in states.items() if cell_domain == domain]
        parts.append(f"{domain}:{(sum(values) if how == 'sum' else max(values, default=0))}")
    return ",".join(parts)


def r3_ka_pin(entry, ka_kj2):
    """TQ6：变体在 KA 上 KJ2 的违例数对着登记。entry = ("pinned", n) 或 ("inferred", n)。返回 (ka_expected, ka_pin, ka_inferred)；
    pinned 的等于钉值写 match、不等写 mismatch；inferred 的不钉（ka_pin=not_pinned），只在 ka_inferred 里报 match 或 mismatch。弄坏开关 r3kapinunchecked：不核。"""
    kind, expected = entry
    if "r3kapinunchecked" in BREAK:
        return expected, "unchecked", "none"
    if kind == "pinned":
        return expected, "match" if ka_kj2 == expected else "mismatch", "none"
    return expected, "not_pinned", "match" if ka_kj2 == expected else "mismatch"


def rerun_command_r3(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r3{options.get('pins_text', '')} --root {'<r5 第一段根>' if options.get('pins_text') else '<第二段冻结根>'} --scratch <草稿目录> "
            f"--output {options['output_shown']} --item {item['name']}")


def baseline_item_line_r3(item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_identical_to_segment2_product={retained_comparison_text(baseline, str(baseline.get('verbatim_rows', 0) == len(rows)).lower())} rerun=\"{rerun_command_r3(item, options)}\" title=基线（不改） complete=yes")


def judge_variant_r3(setup, item, baseline, options, binary, build_problem, log_directory, restore):
    """--profile r3 那一套判一个变体（--bounds segment1 --family root_choice，逐 (域, M, U) 比）。
    选行四条有一条不满足（item["failure"]，登记 TH5）的不看 binary，记 outcome=not_compared；别的 binary 是驱动建副本、改坏、编出来的装置
    （编不过是 None，build_problem 是编译的头一行错），起它整份跑完（运行日志写进 log_directory）、逐格与基线比、不早停，按「红」的口径出这一项的产物行（红例账恒空）。
    restore() 由驱动给：把副本里被改的那一份还原、删掉副本与编译目录，交还原成没成（产物行的 copy_restored）。
    交 {lines, ledger, problem, run_seconds}；编译与这一项的挂钟由驱动补成 timing。"""

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}")

    def tail(title):
        return f"rerun=\"{rerun_command_r3(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if item["failure"] is not None:     # 登记 TH5：四条里有一条不满足，记没比、别的行照做
        lines = [f"{head(row)} outcome=not_compared failed_condition={item['failure']} registered_table_row={item['registered_row']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "run_seconds": 0.0}
    if binary is None:
        restored = restore()
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "run_seconds": 0.0}
    mark(options, item["name"], "跑 " + " ".join(device_bounds_arguments(setup)))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], log_directory)
    mutant_rows = result_rows(completed.stdout)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    baseline_parsed, mutant_parsed = r3_parse(baseline["rows"]), r3_parse(mutant_rows)
    first_reds = r3_first_reds(baseline_parsed, mutant_parsed)
    red_domains = [domain for domain in R3_LEGAL_DOMAINS
                   if any(isinstance(value, tuple) for criterion in ("KJ1", "KJ2") for found_domain, value in first_reds[criterion] if found_domain == domain)]
    other_red = any(isinstance(value, tuple) for criterion in ("KJ0", "KJ3") for _domain, value in first_reds[criterion])
    cells_without_rows = sum(1 for cell in baseline_parsed[2] if cell not in mutant_parsed[2])
    ka_kj2 = r3_ka_kj2(mutant_parsed[0])
    ka_expected, ka_pin, ka_inferred = r3_ka_pin(item["pin"], ka_kj2)
    if red_domains:
        outcome = "red"
    elif other_red:
        outcome = "red_only_by_kj0_or_kj3"
    elif cells_without_rows or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    restored = restore()
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    halts = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=halt "))
    voids = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=void "))
    body = (f"outcome={outcome} red_legal_domains={','.join(red_domains) or 'none'} first_red_by_criterion={r3_first_red_text(first_reds)} "
            f"differing_cells={r3_differing_cells_text(baseline_parsed, mutant_parsed)} cells={r3_cells_text(mutant_parsed[0], mutant_parsed[2])} "
            f"ka_kj2={'none' if ka_kj2 is None else ka_kj2} ka_expected={ka_expected} ka_pin={ka_pin} ka_inferred={ka_inferred} "
            f"states_row_differences={r3_states_row_differences(baseline_parsed, mutant_parsed)} "
            f"above_chosen_root_max={r3_state_totals(mutant_parsed[1], 'above_chosen_root_max', 'max')} "
            f"above_chosen_root_over_limit={r3_state_totals(mutant_parsed[1], 'above_chosen_root_over_limit', 'sum')} "
            f"choose_root_returned_none={r3_state_totals(mutant_parsed[1], 'choose_root_returned_none', 'sum')} "
            f"replay_failed={r3_state_totals(mutant_parsed[1], 'replay_failed', 'sum')} halt_conditions={halts} void_conditions={voids} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} cells_without_rows={cells_without_rows} "
            f"abort={abort} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome == "red" else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": [], "problem": problem, "run_seconds": run_seconds}


# ── r4 档（登记 r4 9.3，第二段）：整份装置输出的读法。格是 (域, M, U)，子格是 (被判时故障, 历史标签)。

def r4_parse(rows):
    """装置 --family root_choice_faults 的 E7RESULT 行 → (违例, 计数行, 格的先后次序)。
    违例：{格: {(被判时故障, 历史标签): {判据: (违例数, 整行)}}}（只收 name=violations；KA 的行是 r3 的写法、没有这两样，记成 ("none", "inside_the_promise")）；
    计数行：{格: (字段, 整行)}（name=root_choice_fault_states，KA 取 name=root_choice_states）；格 = (域, M, U)。"""
    violations, states, order = {}, {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") not in ("violations", "root_choice_fault_states", "root_choice_states") or not {"domain", "mounts", "user_publishes"} <= set(fields):
            continue
        cell = (fields["domain"], int(fields["mounts"]), int(fields["user_publishes"]))
        if cell not in order:
            order.append(cell)
        if fields["name"] == "violations":
            if "criterion" in fields:
                subcell = (fields.get("judged_fault", "none"), fields.get("history", "inside_the_promise"))
                violations.setdefault(cell, {}).setdefault(subcell, {})[fields["criterion"]] = (int(fields["violations"]), line)
        else:
            states[cell] = (fields, line)
    return violations, states, order


def r4_cells_of(order, domain):
    """一个域的格，按 (M 升序, U 升序)。"""
    return sorted((mounts, publishes) for cell_domain, mounts, publishes in order if cell_domain == domain)


def r4_subcell_text(subcell):
    fault, label = subcell
    return f"{fault}/{'inside' if label == 'inside_the_promise' else 'outside'}"


def r4_first_reds(baseline, mutant):
    """登记 TQ9：每个变体、每个合法历史域、每条判据，按 (M 升序, U 升序, 被判时故障, 历史标签) 第一个违例数与基线不同的格（变体缺那一格或缺那一行也算不同）；
    到上沿都相同记 None；基线在那个域上一行那条判据都没有记 "no_rows"。整份比完，不早停。返回 {判据: [(域, (M, U, 子格) 或 None 或 "no_rows")…]}。
    弄坏开关 r4firstredswap：KJ1T 与 KJ2 两列对调。"""
    baseline_violations, _states, baseline_order = baseline
    mutant_violations = mutant[0]
    result = {}
    for criterion in R4_CRITERIA:
        per_domain = []
        for domain in R4_LEGAL_DOMAINS:
            cells = r4_cells_of(baseline_order, domain)
            if not any(criterion in readings for m, u in cells for readings in baseline_violations.get((domain, m, u), {}).values()):
                per_domain.append((domain, "no_rows"))
                continue
            first = None
            for m, u in cells:
                for subcell in R4_SUBCELLS:
                    baseline_reading = baseline_violations.get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    mutant_reading = mutant_violations.get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    if baseline_reading is not None and (mutant_reading is None or mutant_reading[0] != baseline_reading[0]):
                        first = (m, u, subcell)
                        break
                if first is not None:
                    break
            per_domain.append((domain, first))
        result[criterion] = per_domain
    if "r4firstredswap" in BREAK:
        result["KJ1T"], result["KJ2"] = result["KJ2"], result["KJ1T"]
    return result


def r4_first_red_text(first_reds):
    def one(value):
        if value is None:
            return "none"
        if isinstance(value, str):
            return value
        return f"M{value[0]}U{value[1]}/{r4_subcell_text(value[2])}"
    return ";".join(f"{criterion}:" + ",".join(f"{domain}={one(value)}" for domain, value in per_domain) for criterion, per_domain in first_reds.items())


def r4_differing_text(baseline, mutant, domains):
    """每条判据、给定的每个域：违例数与基线不同的 (格, 被判时故障, 历史标签) 子格数（子格总数写在 of 后面）。"""
    baseline_violations, _states, baseline_order = baseline
    parts = []
    for criterion in R4_CRITERIA:
        per_domain = []
        for domain in domains:
            total = differing = 0
            for m, u in r4_cells_of(baseline_order, domain):
                for subcell, readings in baseline_violations.get((domain, m, u), {}).items():
                    if criterion not in readings:
                        continue
                    total += 1
                    mutant_reading = mutant[0].get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    differing += int(mutant_reading is None or mutant_reading[0] != readings[criterion][0])
            per_domain.append(f"{domain}={differing}of{total}")
        parts.append(f"{criterion}:" + ",".join(per_domain))
    return ";".join(parts)


def r4_cell_sums(violations, cell):
    """一个格上五条判据的违例数合计（各子格相加）：{判据: 合计}，那一格没有的判据不列。"""
    sums = {}
    for readings in violations.get(cell, {}).values():
        for criterion, (count, _line) in readings.items():
            sums[criterion] = sums.get(criterion, 0) + count
    return sums


def r4_cells_text(violations, order):
    """每个格的 KJ0/KJ1/KJ1T/KJ2/KJ3 违例数合计（没有那一行写 -）：<域>.M<m>U<u>=a/b/c/d/e，格用逗号连（KA 的 M、U 是 0）。"""
    parts = []
    for domain in R4_ALL_DOMAINS:
        for m, u in r4_cells_of(order, domain):
            sums = r4_cell_sums(violations, (domain, m, u))
            parts.append(f"{domain}.M{m}U{u}=" + "/".join(str(sums[criterion]) if criterion in sums else "-" for criterion in R4_ROW_CRITERIA))
    return ",".join(parts) or "none"


def r4_ka_kj2(violations):
    """KA 上 KJ2 的违例数；KA 那一格或那一行不在交 None。"""
    for (domain, _m, _u), subcells in violations.items():
        if domain == "KA":
            for readings in subcells.values():
                if "KJ2" in readings:
                    return readings["KJ2"][0]
    return None


def r4_states_row_differences(baseline, mutant):
    """每个域：计数行与基线不同的第一个格，与那一格上不同的字段名（见证文字不比）；全相同的域不列。"""
    _violations, baseline_states, baseline_order = baseline
    mutant_states = mutant[1]
    found = []
    for domain in R4_ALL_DOMAINS:
        for m, u in r4_cells_of(baseline_order, domain):
            cell = (domain, m, u)
            if cell not in mutant_states:
                found.append(f"{domain}=M{m}U{u}:missing")
                break
            differing = sorted(key for key in baseline_states[cell][0]
                               if not key.endswith("witness") and mutant_states[cell][0].get(key) != baseline_states[cell][0][key])
            if differing:
                found.append(f"{domain}=M{m}U{u}:" + "+".join(differing))
                break
    return ";".join(found) or "none"


def r4_state_totals(states, field, how):
    """每个域上计数行某个字段的合计（how 是 sum 或 max）：KF:a,KF3:b,KF9:c,KFW:d,KG:e,KA:f。"""
    parts = []
    for domain in R4_ALL_DOMAINS:
        values = [int(fields.get(field, "0")) for (cell_domain, _m, _u), (fields, _line) in states.items() if cell_domain == domain]
        parts.append(f"{domain}:{(sum(values) if how == 'sum' else max(values, default=0))}")
    return ",".join(parts)

def r4_ka_pin(entry, ka_kj2):
    """TQ10：变体在 KA 上 KJ2 的违例数对着登记。entry = ("pinned", n) 或 ("inferred", n)。返回 (ka_expected, ka_pin, ka_inferred)；
    pinned 的等于钉值写 match、不等写 mismatch；inferred 的不钉（ka_pin=not_pinned），只在 ka_inferred 里报 match 或 mismatch。弄坏开关 r4kapinunchecked 不存在：
    KA 钉值与 r3 同一组、r3 那一格已经证过它会红。"""
    kind, expected = entry
    if kind == "pinned":
        return expected, "match" if ka_kj2 == expected else "mismatch", "none"
    return expected, "not_pinned", "match" if ka_kj2 == expected else "mismatch"


def r4_kg_pin(setup, label, violations):
    """TQ10：变体在 KG 上每个 (M, U) 格的 KJ1、KJ1T、KJ2 违例数（各子格相加）对着登记第七节「本地择根序」表里它那一列（v1827 取 v1823 那一列）。
    返回 (kg_pin, 不符的格)；kg_pin 是 match 或 mismatch；弄坏开关 r4kgpinunchecked：不核（unchecked）。"""
    if "r4kgpinunchecked" in BREAK:
        return "unchecked", "none"
    column = setup.registered["kg_columns"][R4_KG_COLUMN_OF[label]]
    mismatched = []
    for key, expected in column.items():
        m, u = (int(part) for part in key.split(","))
        sums = r4_cell_sums(violations, ("KG", m, u))
        got = [sums.get(criterion) for criterion in R4_CLAUSE_CRITERIA]
        if got != list(expected):
            mismatched.append(f"M{m}U{u}:" + "/".join("-" if value is None else str(value) for value in got) + "!=" + "/".join(str(value) for value in expected))
    return ("mismatch" if mismatched else "match"), (";".join(mismatched) or "none")


def r4_baseline_kg_today_pin(setup, violations):
    """基线在 KG 上每格的读数对「今天的序」那一列（登记 5.6：本地理想恢复与今天的三步逐格同值）；只报，不停机。"""
    column = setup.registered["kg_columns"]["today"]
    for key, expected in column.items():
        m, u = (int(part) for part in key.split(","))
        sums = r4_cell_sums(violations, ("KG", m, u))
        if [sums.get(criterion) for criterion in R4_CLAUSE_CRITERIA] != list(expected):
            return "mismatch"
    return "match"


def r4_differs_on(baseline, mutant):
    """变体在合法域不红时，KG 与 KA 上有没有与基线不同的量：KG（三条条款判据任一子格违例数不同）、KA（KJ2）。"""
    baseline_violations = baseline[0]
    on_kg = any(
        (mutant[0].get(("KG", m, u), {}).get(subcell, {}).get(criterion) is None
         or mutant[0][("KG", m, u)][subcell][criterion][0] != reading[0])
        for (domain, m, u), subcells in baseline_violations.items() if domain == "KG"
        for subcell, readings in subcells.items() for criterion, reading in readings.items() if criterion in R4_CLAUSE_CRITERIA)
    on_ka = r4_ka_kj2(baseline_violations) != r4_ka_kj2(mutant[0])
    return "+".join(name for name, differs in (("KG", on_kg), ("KA", on_ka)) if differs) or "none", on_kg


def rerun_command_r4(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r4{options.get('pins_text', '')} --root <第一段根> --scratch <草稿目录> "
            f"--output {options['output_shown']} --item {item['name']}")


def baseline_item_line_r4(setup, item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    kg_pin = r4_baseline_kg_today_pin(setup, r4_parse(rows)[0]) if rows else "no_rows"
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none{options.get('gate_field', '')} outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_identical_to_segment1_product={retained_comparison_text(baseline, str(baseline.get('verbatim_rows', 0) == len(rows)).lower())} kg_today_pin={kg_pin} "
            f"rerun=\"{rerun_command_r4(item, options)}\" title=基线（不改） complete=yes")


def judge_variant_r4(setup, item, baseline, options, binary, build_problem, log_directory, restore):
    """--profile r4 那一套判一个变体（--bounds segment1 --family root_choice_faults，逐 (域, M, U, 被判时故障, 历史标签) 比；
    给了 --gate r2 时每一行在 rows_sharing_this_replacement 之后多一个 gate=r2，登记 r5 修1.九乙第 3 条）。
    选行四条有一条不满足（item["failure"]，登记 TH5）的不看 binary，记 outcome=not_compared；别的 binary 是驱动建副本、改坏、编出来的装置
    （编不过是 None，build_problem 是编译的头一行错），起它整份跑完（运行日志写进 log_directory）、逐格与基线比、不早停，按「红」的口径出这一项的产物行（红例账恒空）。
    restore() 由驱动给：把副本里被改的那一份还原、删掉副本与编译目录，交还原成没成（产物行的 copy_restored）。
    交 {lines, ledger, problem, run_seconds}；编译与这一项的挂钟由驱动补成 timing。"""

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}{options.get('gate_field', '')}")

    def tail(title):
        return f"rerun=\"{rerun_command_r4(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if item["failure"] is not None:     # 登记 TH5：四条里有一条不满足，记没比、别的行照做
        lines = [f"{head(row)} outcome=not_compared failed_condition={item['failure']} registered_table_row={item['registered_row']} variant={item['label']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "run_seconds": 0.0}
    if binary is None:
        restored = restore()
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} variant={item['label']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "run_seconds": 0.0}
    mark(options, item["name"], "跑 " + " ".join(device_bounds_arguments(setup)))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], log_directory)
    mutant_rows = result_rows(completed.stdout)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    baseline_parsed, mutant_parsed = r4_parse(baseline["rows"]), r4_parse(mutant_rows)
    first_reds = r4_first_reds(baseline_parsed, mutant_parsed)
    red_domains = [domain for domain in R4_LEGAL_DOMAINS
                   if any(isinstance(value, tuple) for criterion in R4_CLAUSE_CRITERIA for found_domain, value in first_reds[criterion] if found_domain == domain)]
    differs_on, kg_differs = r4_differs_on(baseline_parsed, mutant_parsed)
    if "r4kgcountsasred" in BREAK and kg_differs:      # 弄坏开关：KG 当合法域
        red_domains = red_domains + ["KG"]
    other_red = any(isinstance(value, tuple) for criterion in ("KJ0", "KJ3") for _domain, value in first_reds[criterion])
    cells_without_rows = sum(1 for cell in baseline_parsed[2] if cell not in mutant_parsed[2])
    ka_kj2 = r4_ka_kj2(mutant_parsed[0])
    ka_expected, ka_pin, ka_inferred = r4_ka_pin(item["pin"], ka_kj2)
    kg_pin, kg_mismatched = r4_kg_pin(setup, item["label"], mutant_parsed[0])
    if red_domains:
        outcome = "red"
    elif other_red:
        outcome = "red_only_by_kj0_or_kj3"
    elif cells_without_rows or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    restored = restore()
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    halts = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=halt "))
    voids = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=void "))
    body = (f"outcome={outcome} variant={item['label']} red_legal_domains={','.join(red_domains) or 'none'} "
            f"differs_on={differs_on if outcome == 'not_red_within_bounds' else 'not_applicable'} "
            f"first_red_by_criterion={r4_first_red_text(first_reds)} "
            f"differing_subcells={r4_differing_text(baseline_parsed, mutant_parsed, R4_LEGAL_DOMAINS)} "
            f"differing_subcells_kg={r4_differing_text(baseline_parsed, mutant_parsed, ('KG',))} cells={r4_cells_text(mutant_parsed[0], mutant_parsed[2])} "
            f"ka_kj2={'none' if ka_kj2 is None else ka_kj2} ka_expected={ka_expected} ka_pin={ka_pin} ka_inferred={ka_inferred} "
            f"kg_pin={kg_pin} kg_mismatched_cells={kg_mismatched} "
            f"states_row_differences={r4_states_row_differences(baseline_parsed, mutant_parsed)} "
            f"choose_root_returned_none_with_a_readable_root={r4_state_totals(mutant_parsed[1], 'choose_root_returned_none_with_a_readable_root', 'sum')} "
            f"replay_failed={r4_state_totals(mutant_parsed[1], 'replay_failed', 'sum')} panicked={r4_state_totals(mutant_parsed[1], 'panicked', 'sum')} "
            f"kj1_and_kj1t_differing_states={r4_state_totals(mutant_parsed[1], 'kj1_and_kj1t_differing_states', 'sum')} "
            f"halt_conditions={halts} void_conditions={voids} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} cells_without_rows={cells_without_rows} "
            f"abort={abort} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome == "red" else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": [], "problem": problem, "run_seconds": run_seconds}


def preflight(path):
    """直接执行这份模块之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）。project_preflight 在这里才 import：
    判法闭包收这份模块 import 时执行的语句，判条件那一层只在直接执行时用，不进判法键。"""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(path)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight as run_preflight
    run_preflight(path)


def main(arguments):
    print("用法：本文件是 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 载入的判法模块，不单独跑；"
          "判法由那份驱动的自证覆盖，起法见它文件头「用法」", file=sys.stderr)
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
