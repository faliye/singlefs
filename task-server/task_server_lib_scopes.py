"""task_server_lib.py 拆出的一份：key-stats、整包测试与未点名的跑法的判据、重试决定（总说明在 task_server_lib.py 文件头）。"""
import os
import calendar
import contextlib
import math
import re
import shlex
import time
from dataclasses import dataclass
from task_server_lib_constants import (
    CARGO_VALUE_OPTIONS, COMMIT_GATE_ENTRIES, COPY_PROOF_OWNER_NAME, COPY_PROOF_OWNER_STALE_SECONDS, COPY_PROOF_PENDING_NAME,
    CRASH_CASES_ONLY_ARGUMENT, CRASH_GATE_CASES_ONLY_FORM, CRASH_GATE_FEATURE_ITEM_PREFIX, ENTRY_QUERY_ARGUMENTS, HOST_NAMES, LIBTEST_VALUE_OPTIONS,
    MERGE_ALL_PACKAGES_ARGUMENT, MERGE_VALUE_OPTIONS, NOT_SHARDED_ARGUMENTS, PROCESS_MEMORY_PEAK_METRIC, RECORD_CAP_HIT_OUTCOMES, REPOSITORY_ROOT,
    RETRY_LIMITS, SCOPE_KIND_PREFIX, SCOPE_RANGE_ARGUMENTS, TEST_STARTING_GATE_CELLS, UNNAMED_RUN_CARGO_QUERIES, UNNAMED_RUN_EXEMPTION_TEXT,
    UNNAMED_RUN_ITEM_ENTRIES, UNNAMED_RUN_PROBLEM_HEAD, UNNAMED_RUN_PROVE_RED_ENTRY, UNNAMED_RUN_PROVE_RED_QUERIES, UNNAMED_RUN_STAGE_QUERIES,
    UNSCOPED_RUN_ENTRIES, WHOLE_PACKAGE_CHECK_CELLS, WHOLE_PACKAGE_CRASH_GATE, WHOLE_PACKAGE_MARKS, WHOLE_PACKAGE_MERGE_ENTRY,
    WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS,
)
from task_server_lib_configuration import broken, command_word_index
from task_server_lib_readings import entry_is, entry_word, grown_cap, mebibytes_text, shell_inner_commands


KEY_STATS_RECENT = 5   # control key-stats 看每把命令键的近几次（同 task_server.py 的 RECORD_WINDOW：定额度读的近几次）
KEY_STATS_COPY_MARK = "singlefs/work/"   # 命令键里带这一段的是工作根下副本或草稿目录的路径（同一条命令换一份副本就是一个新键）
# control key-stats --dump 逐行写的列（research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 3 条）：id 是尝试的行号，别的取尝试记录里同名的一栏，没有这一栏的写 null
KEY_STATS_DUMP_COLUMNS = ("id", "task_id", "number", "command_key", "command", "started_at", "ended_at", "outcome", "exit_code", "memory_cap", "memory_peak",
                          "peak_metric", "stalled_at_cap", "prediction", "shard_count", "host", "item_plan")
KEY_STATS_SINCE_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")   # --since 的日期写法（取那一天开头，日期照 time.gmtime 算：与 control records 打的起跑日期同一种算法）


def current_peak_metric(attempt):
    """尝试记录的峰值口径是不是这一版的（文件头「资源记录」的峰值口径一栏）：peak_metric 是 PROCESS_MEMORY_PEAK_METRIC 的才算。
    定额度（task_server.py 的 current_metric）与 control key-stats（key_stats）共用这一个判法。"""
    return attempt.get("peak_metric") == PROCESS_MEMORY_PEAK_METRIC


def key_stats_since(text):
    """control key-stats --since 的值 → (epoch 秒, 认不出的原因)：写成 epoch 秒（整数或小数）或 YYYY-MM-DD（那一天开头，日期照 time.gmtime 算、calendar.timegm 换回 epoch 秒）。认不出交 (None, 原因)。"""
    with contextlib.suppress(ValueError):
        value = float(text)
        if math.isfinite(value) and value >= 0:
            return value, ""
    if KEY_STATS_SINCE_DATE.fullmatch(text):
        with contextlib.suppress(ValueError):
            return float(calendar.timegm(time.strptime(text, "%Y-%m-%d"))), ""
    return None, f"--since 的值「{text}」认不出：写成 epoch 秒或 YYYY-MM-DD"


def key_stats_after(attempts, since):
    """起跑时刻（started_at）不早于 since（epoch 秒）的尝试；since 是 None 的照原样交。弄坏开关 key-stats-since-ignored 下不筛。"""
    if since is None or broken("key-stats-since-ignored"):
        return list(attempts)
    return [attempt for attempt in attempts
            if isinstance(attempt.get("started_at"), (int, float)) and not isinstance(attempt.get("started_at"), bool) and float(attempt["started_at"]) >= since]


def key_stats_mark(attempt):
    """key_stats 里一次尝试的标记：撞上限（RECORD_CAP_HIT_OUTCOMES）记 hit，跑成记 clean，别的结局交空串（不进交替的数）。"""
    if attempt.get("outcome") in RECORD_CAP_HIT_OUTCOMES:
        return "hit"
    return "clean" if attempt.get("outcome") == "succeeded" else ""


def key_stats_flips(marks):
    """一串标记（空串去掉）里前后不同的回数。"""
    marks = [mark for mark in marks if mark]
    return sum(1 for before, after in zip(marks, marks[1:]) if before != after), marks


def key_stats(attempts, recent=KEY_STATS_RECENT):
    """control key-stats 的统计（records/2026-11-17-subagent账本自上而下清欠.md 八十节 ①④ 回看要数的几样；research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 2 条；只读、不改任何判法）：
    attempts 是尝试记录的列表（命令键、任务号、第几次、state、结局、峰值、峰值口径、起跑时刻），只看收尾了的、有命令键的、峰值口径是这一版的（current_peak_metric，与定额度读的同一个判法），
    按命令键分组，组里按起跑先后排。→ {used, keys, single, copy_keys, alternating: [键…], alternating_attempts: [键…], hit_then_clean, dispersion: [(键, 最大峰值÷最小峰值)…], median, p90, over_one_and_half}：
    used 是进统计的尝试数；alternating 按任务数：同一任务（task_id）的几次尝试折成一条，标记取这个任务第一次尝试（number 最小的）的结局（key_stats_mark），
    任务按第一次尝试的起跑先后排，近 recent 个任务里撞上限与跑成翻面至少两回的键；hit_then_clean 同样按任务数：近 recent 个任务里有撞过上限的、最近一个任务跑成的键数；
    alternating_attempts 按尝试数：近 recent 次尝试逐次的标记翻面至少两回的键；dispersion 是近 recent 次尝试里跑成并量到峰值至少两次的键的最大峰值比最小峰值（中位数与第 90 百分位取最近排名法）。
    弄坏开关 key-stats-alternation-ignored 下 alternating 交空，key-stats-metric-unfiltered 下不筛峰值口径，key-stats-retries-unfolded 下按任务数的两样也逐次尝试数。"""
    used = [attempt for attempt in attempts if attempt.get("state") == "ended" and attempt.get("command_key")
            and (current_peak_metric(attempt) or broken("key-stats-metric-unfiltered"))]
    by_key = {}
    for attempt in sorted(used, key=lambda a: (float(a.get("started_at") or 0), int(a.get("id") or 0))):
        by_key.setdefault(attempt["command_key"], []).append(attempt)
    alternating, alternating_attempts, hit_then_clean, dispersion = [], [], 0, []
    for key, runs in by_key.items():
        latest = runs[-recent:]
        if key_stats_flips(key_stats_mark(run) for run in latest)[0] >= 2:
            alternating_attempts.append(key)
        by_task = {}
        for run in runs:
            by_task.setdefault(run.get("task_id"), []).append(run)
        firsts = [min(group, key=lambda a: (int(a.get("number") or 0), float(a.get("started_at") or 0), int(a.get("id") or 0))) for group in by_task.values()]
        if broken("key-stats-retries-unfolded"):
            firsts = runs
        flips, marks = key_stats_flips(key_stats_mark(run) for run in firsts[-recent:])
        if flips >= 2 and not broken("key-stats-alternation-ignored"):
            alternating.append(key)
        if "hit" in marks and marks[-1:] == ["clean"]:
            hit_then_clean += 1
        peaks = [run["memory_peak"] for run in latest if run.get("outcome") == "succeeded" and isinstance(run.get("memory_peak"), int) and not isinstance(run.get("memory_peak"), bool)
                 and run["memory_peak"] > 0]
        if len(peaks) >= 2:
            dispersion.append((key, max(peaks) / min(peaks)))
    ratios = sorted(ratio for _, ratio in dispersion)
    return {"used": len(used), "keys": len(by_key), "single": sum(1 for runs in by_key.values() if len(runs) == 1), "copy_keys": sum(1 for key in by_key if KEY_STATS_COPY_MARK in key),
            "alternating": alternating, "alternating_attempts": alternating_attempts, "hit_then_clean": hit_then_clean, "dispersion": dispersion,
            "median": ratios[(len(ratios) - 1) // 2] if ratios else None, "p90": ratios[max(0, math.ceil(0.9 * len(ratios)) - 1)] if ratios else None,
            "over_one_and_half": sum(1 for ratio in ratios if ratio >= 1.5)}


def key_stats_lines(stats):
    """key_stats 的结果写成几行给人看（control key-stats 打）。"""
    def named(keys):
        return "：" + "；".join(key[:80] for key in keys[:5]) if keys else ""
    lines = [f"命令键 {stats['keys']} 个（只跑过一次的 {stats['single']} 个），其中带工作根下副本或草稿目录路径的 {stats['copy_keys']} 个",
             f"近 {KEY_STATS_RECENT} 个任务里撞上限与跑成交替至少两回的键 {len(stats['alternating'])} 个（按任务数：同一任务的几次尝试折成一条，取第一次尝试的结局）" + named(stats["alternating"]),
             f"近 {KEY_STATS_RECENT} 个任务里撞过上限、最近一个任务跑成的键 {stats['hit_then_clean']} 个（按任务数，同上）",
             f"按尝试数：近 {KEY_STATS_RECENT} 次尝试里撞上限与跑成交替至少两回的键 {len(stats['alternating_attempts'])} 个" + named(stats["alternating_attempts"])]
    if stats["dispersion"]:
        lines.append(f"近 {KEY_STATS_RECENT} 次里跑成并量到峰值至少两次的键 {len(stats['dispersion'])} 个，各键最大峰值比最小峰值：中位 {stats['median']:.2f} 倍、第 90 百分位 {stats['p90']:.2f} 倍，"
                     f"到 1.5 倍以上的 {stats['over_one_and_half']} 个")
    else:
        lines.append(f"近 {KEY_STATS_RECENT} 次里跑成并量到峰值至少两次的键 0 个")
    return lines


def key_stats_dump_rows(attempts):
    """control key-stats --dump 逐行写的内容：收尾了的尝试按行号排，每条取 KEY_STATS_DUMP_COLUMNS 那几列（尝试记录没有那一栏的交 None，写出来是 null）。"""
    ended = sorted((attempt for attempt in attempts if attempt.get("state") == "ended"), key=lambda a: int(a.get("id") or 0))
    return [{column: attempt.get(column) for column in KEY_STATS_DUMP_COLUMNS} for attempt in ended]


def peak_memory_note(stat):
    """内存最高那一刻的 memory.stat（task_server.py 文件头 ①「内存最高那一刻」：attempt_runner.py 的 memory_stat_at_peak，第二台是 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行）
    换成给人看的一句（读不到的那一项写读不到）；没有交空串。大头在 file（页缓存，含没写回的脏页）的另写一句「大头是页缓存」，在 anon 的写「大头是进程自己的内存（堆）」：只记、不改判法。"""
    if not isinstance(stat, dict):
        return ""

    def shown(name):
        return mebibytes_text(stat[name]) if isinstance(stat.get(name), int) else "读不到"
    text = (f"内存最高那一刻（memory.current {shown('current')}）scope 的内存明细：anon {shown('anon')}、file {shown('file')}（其中没写回的脏页 file_dirty {shown('file_dirty')}、"
            f"正在写回 file_writeback {shown('file_writeback')}）、shmem {shown('shmem')}")
    if isinstance(stat.get("anon"), int) and isinstance(stat.get("file"), int):
        text += "：大头是页缓存（file）" if stat["file"] > stat["anon"] else "：大头是进程自己的内存（anon，堆）"
    return text


def whole_package_cargo_test(rest):
    """一条 cargo test 后面的参数（rest）是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句是哪一种，或空串（点名了测试目标）。
    带 --workspace / --all 的是整包；点名了 --test、--bin、或带测试名（`--` 前后不以 - 起头的词、选项的值不算：CARGO_VALUE_OPTIONS 里的选项值写在下一个词里的连同值跳过，
    `--名=值` 与连写的 `-p包` 不吞下一个词）的不是；--lib 与 --tests 都带、没带测试名的是整包；别的都是。
    弄坏开关 cargo-exclude-value-unskipped 下 --exclude 的值当测试名（`-p 包 --exclude 别的` 当点名了测试）。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (list(rest), [])
    value_options = tuple(option for option in CARGO_VALUE_OPTIONS if option != "--exclude") if broken("cargo-exclude-value-unskipped") else CARGO_VALUE_OPTIONS
    options, names, index = set(), [], 0
    while index < len(before):
        word = before[index]
        name = word.split("=", 1)[0] if word.startswith("--") else word
        if word.startswith("-"):
            options.add(name)
            if name in value_options and "=" not in word:   # 连写的 -p包 name 是整个词、不在表里，不吞下一个词
                index += 1
        else:
            names.append(word)
        index += 1
    skip = False
    for word in after:
        if skip:
            skip = False
            continue
        if word in LIBTEST_VALUE_OPTIONS:
            skip = True
        elif not word.startswith("-"):
            names.append(word)
    if options & {"--workspace", "--all"}:
        return "cargo test 带 --workspace（整个工作区的每一个包）"
    if names:
        return ""
    if {"--lib", "--tests"} <= options:
        return "cargo test 带 --lib --tests、没带测试名（包里全部测试目标）"
    if options & {"--test", "--bin"}:
        return ""
    return "cargo test 没点名 --test、--bin，也没带测试名（整个包的测试）"


def prove_red_copies(command, cwd, environment=None):
    """命令（含 bash -c 里命令位置上的每一条）里起的证红 scripts/mutation/prove-red.sh --copy <副本> 点名的副本，照工作目录解开、解开符号链接 → [真路径…]；不是证红交 []。
    task_server.py 文件头 ②「提交时拒」⑨ 用它认哪条任务在对哪份副本证红。"""
    copies = []
    for words in [list(command), *shell_inner_commands(command, environment)]:
        found = entry_word(words, cwd)
        if found is not None and entry_is(found, UNNAMED_RUN_PROVE_RED_ENTRY) and len(found[2]) >= 2 and found[2][0] == "--copy":
            copies.append(os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, found[2][1])))
    return copies


def inside_directory(path, directory):
    """path 是 directory 本身或在它底下（都是真路径）。"""
    return path == directory or path.startswith(directory.rstrip(os.sep) + os.sep)


def copy_proof_marker(directory, now=None):
    """directory 起往上走，头一份留着证红进行中的标记的目录 → (那份副本的根, 标记名, 标记读出来的意思)，一份都没有交 None。
    标记两种：还原日志 COPY_PROOF_PENDING_NAME 在（证红施加了变异、没走到写回就被停了，源码此刻带着变异）；副本锁持有者记录 COPY_PROOF_OWNER_NAME 在
    COPY_PROOF_OWNER_STALE_SECONDS 秒内刷新过（证红还持着这份副本）。持有者记录刷新过期、还原日志不在的算没有：证红被杀而没施加过变异。"""
    now = time.time() if now is None else now
    path = os.path.realpath(directory)
    while True:
        if os.path.exists(os.path.join(path, COPY_PROOF_PENDING_NAME)):
            return path, COPY_PROOF_PENDING_NAME, "证红施加了变异、没走到写回就被停了，源码此刻带着变异"
        try:
            age = now - os.stat(os.path.join(path, COPY_PROOF_OWNER_NAME)).st_mtime
        except OSError:
            age = None
        if age is not None and age < COPY_PROOF_OWNER_STALE_SECONDS:
            return path, COPY_PROOF_OWNER_NAME, f"证红持着这份副本（持有者记录 {age:.0f} 秒前刷新过）"
        parent = os.path.dirname(path)
        if parent == path:
            return None
        path = parent


def checked_cells(arguments):
    """--check <格,…> 与 --check=<格,…> 点的格（给几次取并集）。"""
    cells = set()
    for position, argument in enumerate(arguments):
        if argument == "--check" and position + 1 < len(arguments):
            cells.update(arguments[position + 1].split(","))
        elif argument.startswith("--check="):
            cells.update(argument[len("--check="):].split(","))
    return cells


def whole_package_script(argv, cwd, environment=None):
    """命令位置上起的入口是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句或空串。带 --selftest、--list-items 这类的不是。
    crash-point-replay.sh 门禁的 --check 点了 WHOLE_PACKAGE_CHECK_CELLS 里的哪一格、几格都是（crash-case-replay 带 --crash-cases-only 的那一格不算；
    feature-gated-tests 带了 CRASH_GATE_FEATURE_ITEM_PREFIX 起头的 --item 的那一格不算：只编、只跑点名的挂特性文件，同 crash_gate_whole_package_cells）；
    merge-and-verify.sh 没给补丁目录、或带 --all-packages 的是。弄坏开关 whole-package-cells-all-three-only 下 --check 要三格全点才算，
    merge-all-packages-accepted 下 --all-packages 不算，whole-package-feature-items-uncounted 下点了挂特性文件的 feature-gated-tests 照算整包，
    whole-package-any-item-exempts 下带了任何一个 --item 的 feature-gated-tests 都不算（崩溃枚举用例的项也算点了挂特性文件）。"""
    found = entry_word(argv, cwd)
    if found is None or any(argument in NOT_SHARDED_ARGUMENTS for argument in found[2]):
        return ""
    arguments = found[2]
    if entry_is(found, WHOLE_PACKAGE_CRASH_GATE):
        cells = checked_cells(arguments)
        if broken("whole-package-cells-all-three-only", environment):
            named = list(WHOLE_PACKAGE_CHECK_CELLS) if set(WHOLE_PACKAGE_CHECK_CELLS) <= cells else []
        else:
            crash_cases_only = CRASH_CASES_ONLY_ARGUMENT in arguments
            items = item_option_values(arguments)
            if broken("whole-package-any-item-exempts", environment):
                feature_items = items
            else:
                feature_items = [] if broken("whole-package-feature-items-uncounted", environment) else [
                    item for item in items if item.startswith(CRASH_GATE_FEATURE_ITEM_PREFIX)]
            named = [cell for cell in WHOLE_PACKAGE_CHECK_CELLS if cell in cells and not (cell == "crash-case-replay" and crash_cases_only)
                     and not (cell == "feature-gated-tests" and feature_items)]
        if not named:
            return ""
        alone = "（crash-case-replay 没带 --crash-cases-only）" if "crash-case-replay" in named else ""
        if "feature-gated-tests" in named:
            alone += f"（feature-gated-tests 没点 {CRASH_GATE_FEATURE_ITEM_PREFIX}<文件名> 项）"
        return f"{WHOLE_PACKAGE_CRASH_GATE} --check 点了 {'、'.join(named)}{alone}：验证包那一步的格是整包的测试"
    if entry_is(found, WHOLE_PACKAGE_MERGE_ENTRY):
        if MERGE_ALL_PACKAGES_ARGUMENT in arguments and not broken("merge-all-packages-accepted", environment):
            return f"{WHOLE_PACKAGE_MERGE_ENTRY} 带 {MERGE_ALL_PACKAGES_ARGUMENT}（每个包都跑）"
        position, given = 0, []
        while position < len(arguments):
            argument = arguments[position]
            if argument in MERGE_VALUE_OPTIONS:
                position += 2
                continue
            if not argument.startswith("-"):
                given.append(argument)
            position += 1
        return "" if given else f"{WHOLE_PACKAGE_MERGE_ENTRY} 没给补丁目录（合入验证整包跑一遍）"
    return ""


def unscoped_run_entry(argv, cwd, environment=None):
    """命令位置上起的是 UNSCOPED_RUN_ENTRIES 里的门禁或入口、又没带选条参数（不指定范围，跑全部项）：→ 一句或空串。带 --selftest、--list-items 这类的、
    门禁阶段带 --list、--tool-versions 的（query_only_entry）、带了那一份「不跑」的选项的不是。带全量门禁标识（task submit --full-gate）的由调用方先放行。
    弄坏开关 unscoped-gate-accepted 下一律交空串。"""
    found = entry_word(argv, cwd)
    if (found is None or broken("unscoped-gate-accepted", environment) or any(argument in NOT_SHARDED_ARGUMENTS for argument in found[2])
            or query_only_entry(found, environment)):
        return ""
    arguments = found[2]
    for relative, (options, positional, not_runs) in UNSCOPED_RUN_ENTRIES.items():
        if not entry_is(found, relative):
            continue
        if any(argument in not_runs for argument in arguments):
            return ""
        if any(argument in options or argument.startswith(tuple(option + "=" for option in options)) for argument in arguments):
            return ""
        if positional and any(not argument.startswith("-") for argument in arguments):
            return ""
        wanted = "、".join(list(options) + (["实验号"] if positional else []))
        return f"{relative} 没带选条参数（{wanted}）：不指定范围、跑全部项"
    return ""


def whole_package_confirmation_problem(path, shared_root_of):
    """整包确认文件（task submit --whole-package-confirmed <文件>；task_server.py 文件头 ②「提交时拒」⑥）的形式：绝对路径、真路径落在某一段共享根下（shared_root_of 交那一段，
    不在交空串）、读得到、有以 WHOLE_PACKAGE_MARKS 三个记号起头的三段，每段去掉空白与记号至少 WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS 个字。
    → 不合的那一句（点名缺哪一段、哪一段太短），合的交空串。只判形式，不判写得对不对。"""
    if not path or not os.path.isabs(path):
        return f"--whole-package-confirmed 给的「{path}」不是绝对路径"
    real = os.path.realpath(path)
    if not shared_root_of(real):
        return f"整包确认文件 {path} 不在多机配置 NFS_SHARED_ROOTS 的共享根下（两台都要读得到）"
    try:
        with open(real, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError as error:
        return f"整包确认文件 {path} 读不了（{error}）"
    sections, current = {}, None
    for line in text.splitlines():
        stripped = line.strip().lstrip("-*#> ").strip()
        mark = next((mark for mark in WHOLE_PACKAGE_MARKS if stripped.startswith(mark)), None)
        if mark is not None:
            current = mark
            sections[mark] = sections.get(mark, "") + stripped[len(mark):]
        elif current is not None:
            sections[current] += line
    missing = [mark for mark in WHOLE_PACKAGE_MARKS if mark not in sections]
    if missing:
        return f"整包确认文件 {path} 缺以 {'、'.join(missing)} 起头的那一段（三段：①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够）"
    short = [mark for mark in WHOLE_PACKAGE_MARKS if len("".join(sections[mark].split())) < WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS]
    if short:
        return f"整包确认文件 {path} 的 {'、'.join(short)} 那一段去掉空白不到 {WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS} 个字"
    return ""


def query_only_entry(found, environment=None):
    """entry_word 交的这一条只查不跑（不编、不跑测试、不判，不是不指定范围的那一趟）：门禁阶段带 UNNAMED_RUN_STAGE_QUERIES 之一，
    别的入口带 ENTRY_QUERY_ARGUMENTS 之一——与 unnamed_run_problem 放行的同一组写法。commit_gate_entry 与 unscoped_run_entry 照它不判。
    弄坏开关 commit-gate-query-forms-judged 下一律交 False（只查不跑的照不指定范围那一趟判）。"""
    if found is None or broken("commit-gate-query-forms-judged", environment):
        return False
    queries = UNNAMED_RUN_STAGE_QUERIES if gate_stage_of_entry(found) else ENTRY_QUERY_ARGUMENTS
    return any(argument in queries for argument in found[2])


def commit_gate_entry(argv, cwd, environment=None):
    """全量门禁标识（task submit --full-gate；文件头「异常表」long-run-unconfirmed）认的命令：起的是 COMMIT_GATE_ENTRIES 之一（剥法同 sharded_entry）、
    参数里没有 --check 与 --item（「--check=…」这种写法同样算），也不是只查不跑的写法（query_only_entry：--selftest、--list、--list-items、--help 这类）。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, word))
    if not any(script.endswith("/" + relative) or word == relative for relative in COMMIT_GATE_ENTRIES):
        return False
    if query_only_entry(entry_word(argv, cwd), environment):
        return False
    return not any(argument in SCOPE_RANGE_ARGUMENTS or argument.startswith(tuple(name + "=" for name in SCOPE_RANGE_ARGUMENTS)) for argument in arguments)


def item_option_values(arguments):
    """--item <项> 与 --item=<项> 点的项（值不空的，次序照命令）。"""
    values = []
    for position, argument in enumerate(arguments):
        if argument == "--item" and position + 1 < len(arguments) and arguments[position + 1]:
            values.append(arguments[position + 1])
        elif argument.startswith("--item=") and len(argument) > len("--item="):
            values.append(argument[len("--item="):])
    return values


def item_option_given(arguments):
    """参数里有没有 --item <项> 或 --item=<项>（值不空）。"""
    return bool(item_option_values(arguments))


def test_starting_gate_cells(environment=None):
    """起测试的格表 TEST_STARTING_GATE_CELLS（unnamed_run_problem 与自证 test-starting-cells-listed 都经这里读）。
    弄坏开关 test-starting-cells-misnamed 下 lkmm 那一行的格名写错一个字（那一道的格名表里没有它）。"""
    if broken("test-starting-cells-misnamed", environment):
        return {**TEST_STARTING_GATE_CELLS, ".claude/gate.d/lkmm.sh": ("litmus-verdict",)}
    return TEST_STARTING_GATE_CELLS


def crash_gate_whole_package_cells(arguments, environment=None):
    """门禁 crash-point-replay 带了 --item 的这一条（arguments 是阶段文件后面的参数）照它文件头「逐项点名」「--crash-cases-only」实际要整格跑的
    WHOLE_PACKAGE_CHECK_CELLS 里的格 → 格名列表（次序照那张表），空的是只跑点名的项。
    要跑的格：给了 --check 的照点的格；没给 --check 的，点了崩溃枚举用例（不以 CRASH_GATE_FEATURE_ITEM_PREFIX 起头的项）照旧九格，只点挂特性的文件的只跑 feature-gated-tests 一格。
    其中不整格跑的：crash-case-replay 带 --crash-cases-only（只走按块那一路、按条复用那一趟不起）；feature-gated-tests 点了挂特性的文件（只编、只跑点名的那几个）。
    product-gpu-real-card-tests 不收点名，跑到就是整格。
    弄坏开关 unnamed-crash-gate-item-whole-accepted 下一律交空（带 --item 就算点名），unnamed-crash-gate-cases-only-uncounted 下 --crash-cases-only 不算数。"""
    if broken("unnamed-crash-gate-item-whole-accepted", environment):
        return []
    items = item_option_values(arguments)
    feature_items = [item for item in items if item.startswith(CRASH_GATE_FEATURE_ITEM_PREFIX)]
    cells = checked_cells(arguments) or (set(WHOLE_PACKAGE_CHECK_CELLS) if len(feature_items) < len(items) else {"feature-gated-tests"})
    cases_only = CRASH_CASES_ONLY_ARGUMENT in arguments and not broken("unnamed-crash-gate-cases-only-uncounted", environment)
    return [cell for cell in WHOLE_PACKAGE_CHECK_CELLS
            if cell in cells and not (cell == "crash-case-replay" and cases_only) and not (cell == "feature-gated-tests" and feature_items)]


def gate_stage_of_entry(found):
    """entry_word 交的那个词起的是不是 .claude/gate.d/ 顶层的门禁阶段（lib/ 下的不算）：是交「.claude/gate.d/<文件名>」，不是交空串。"""
    word, real, _ = found
    for path in (real, os.path.normpath(word)):
        parent = os.path.dirname(path)
        if path.endswith(".sh") and os.path.basename(parent) == "gate.d" and os.path.basename(os.path.dirname(parent)) == ".claude":
            return ".claude/gate.d/" + os.path.basename(path)
    return ""


def unnamed_cargo_test_problem(after_test, shown, environment=None):
    """cargo test / cargo t 后面的参数（after_test）点没点到 --exact 的单条测试函数：点了、只编不跑、只列用例交 None，没点交 {problem, next_step}。
    shown 是这条命令照原样写成的一串（拒绝说明里改成点名的样子照它接）。
    弄坏开关 unnamed-cargo-filter-accepted 下带了测试目标或筛选词就算点名，unnamed-cargo-exact-uncounted 下 --exact 的测试名不算点名。"""
    before, after = (after_test[:after_test.index("--")], after_test[after_test.index("--") + 1:]) if "--" in after_test else (list(after_test), [])
    if any(word in UNNAMED_RUN_CARGO_QUERIES for word in before) or "--list" in after:
        return None
    names, skip = [], False
    for word in after:
        if skip:
            skip = False
        elif word in LIBTEST_VALUE_OPTIONS:
            skip = True
        elif not word.startswith("-"):
            names.append(word)
    if "--exact" in after and names and not broken("unnamed-cargo-exact-uncounted", environment):
        return None
    if broken("unnamed-cargo-filter-accepted", environment) and (names or any(word in ("--test", "--bin", "--lib") or not word.startswith("-") for word in before)):
        return None
    head = shown.split(" -- ", 1)[0] if " -- " in shown else shown
    example = f"{head} -- {' '.join(names) if names else '<测试全名>'} --exact"
    return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没点名到单条测试函数（`--` 之后要带 --exact 与测试全名；只点测试目标、只给筛选词的都跑整个目标或一批测试）",
            "next_step": f"改成点名改到的那几条测试函数：{example}（几条就写几个全名，模块路径写全）；{UNNAMED_RUN_EXEMPTION_TEXT}"}


def unnamed_run_problem(argv, cwd, full_gate=False, whole_package_confirmed=False, environment=None):
    """启动测试与验证的一条命令（命令位置上那一条，前缀照 command_word_index 剥）点没点名（task_server.py 文件头 ②「提交时拒」⑦；异常表 unnamed-run）：
    → None（点了名、只查不跑、不是这几种、带了例外的标识），或 {problem: 哪一种没点名, next_step: 改成点名的样子}。
    例外两种：交它的那一条带 --full-gate（提交时那一趟；服务的 full_gate_refusal 先核过它只给不指定范围的提交时门禁），带 --whole-package-confirmed（用户确认过的整包，服务先核过确认文件的形式）。
    算点名的：cargo test / cargo t 的 `--` 之后带 --exact 与至少一个测试全名（LIBTEST_VALUE_OPTIONS 的值不算；--no-run、--help 与 `-- --list` 不起测试，不判）；
      UNNAMED_RUN_ITEM_ENTRIES 那三份带 --item；证红 prove-red.sh --copy <副本> <crate> 之后至少一个变异名；
      .claude/gate.d/ 顶层的门禁阶段带 --item，或带 --check 而点的格全不在 TEST_STARTING_GATE_CELLS 这一道那一行；门禁 crash-point-replay 带了 --item 还要
      crash_gate_whole_package_cells 交空（--check 把起整个验证包的格排掉）。只查不跑的参数（各自那一张 *_QUERIES）不判。
    bash -c 的脚本由调用方逐条拆开再问（服务用 shell_inner_commands，范围声明闸用它自己的切词），这里只判一条。
    弄坏开关：unnamed-run-exemptions-ignored 下带了例外标识照判；unnamed-entry-item-unchecked 下三份入口不带 --item 也算点名，unnamed-entry-item-uncounted 下带了也不算；
      unnamed-prove-red-unchecked 下证红不给变异名也算点名，unnamed-prove-red-name-offset 下变异名从第二个起数（只点一个的算没点）；
      unnamed-stage-whole-accepted 下门禁阶段 --item、--check 都不带也算点名，unnamed-check-cells-accepted 下带 --check 就算点名、不看点的格起不起测试；
      cargo test 的两个见 unnamed_cargo_test_problem，门禁 crash-point-replay 带 --item 的两个见 crash_gate_whole_package_cells，起测试的格表的见 test_starting_gate_cells。"""
    if (full_gate or whole_package_confirmed) and not broken("unnamed-run-exemptions-ignored", environment):
        return None
    argv = list(argv)
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    shown = shlex.join(argv[index:])
    if os.path.basename(argv[index]) == "cargo":
        position = index + 1
        while position < len(argv) and (argv[position].startswith("+") or argv[position].startswith("-")):
            position += 2 if argv[position] in ("-Z", "--config", "--color", "-C") else 1
        if position < len(argv) and argv[position] in ("test", "t"):
            return unnamed_cargo_test_problem(argv[position + 1:], shown, environment)
        return None
    found = entry_word(argv, cwd)
    word, _, arguments = found
    arguments = [argument for argument in arguments if argument != "--force"]

    def named_form(addition):
        return " ".join([shlex.quote(word), addition] + ([shlex.join(arguments)] if arguments else []))
    for relative, (named, queries) in UNNAMED_RUN_ITEM_ENTRIES.items():
        if not entry_is(found, relative):
            continue
        if any(argument in queries for argument in arguments):
            return None
        given = item_option_given(arguments) and not broken("unnamed-entry-item-uncounted", environment)
        if given or broken("unnamed-entry-item-unchecked", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没带 --item（{relative} 不带 --item 就跑它自己挑的全部项或全部目标）",
                "next_step": f"改成只跑改到的、红了的那几项：{named_form(named)}；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    if entry_is(found, UNNAMED_RUN_PROVE_RED_ENTRY):
        if any(argument in UNNAMED_RUN_PROVE_RED_QUERIES for argument in arguments):
            return None
        names_start = 4 if broken("unnamed-prove-red-name-offset", environment) else 3   # --copy <副本> <crate> 之后是变异名
        if (len(arguments) > names_start and arguments[0] == "--copy") or broken("unnamed-prove-red-unchecked", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没点名变异（证红要在 --copy <副本> <crate> 之后写至少一个变异名）",
                "next_step": f"改成点名这一次新写的变异行：bash {UNNAMED_RUN_PROVE_RED_ENTRY} --copy <副本> <crate> <变异名…>（变异名是 crates/mutations.tsv 第一段）；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    stage = gate_stage_of_entry(found)
    if not stage or any(argument in UNNAMED_RUN_STAGE_QUERIES for argument in arguments):
        return None
    if item_option_given(arguments):
        whole = crash_gate_whole_package_cells(arguments, environment) if stage == WHOLE_PACKAGE_CRASH_GATE else []
        if not whole:
            return None
        rest, position = [], 0
        while position < len(arguments):   # 改法照原命令去掉 --check 与 --crash-cases-only，接上 CRASH_GATE_CASES_ONLY_FORM
            if arguments[position] == "--check":
                position += 2
                continue
            if not arguments[position].startswith("--check=") and arguments[position] != CRASH_CASES_ONLY_ARGUMENT:
                rest.append(arguments[position])
            position += 1
        rewritten = " ".join([shlex.quote(word), CRASH_GATE_CASES_ONLY_FORM, shlex.join(rest)])
        return {"problem": (f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 带了 --item，--check 却没把起整个验证包的格排掉，{'、'.join(whole)} 照跑整格"
                            f"（门禁 {stage} 文件头「逐项点名」：没给 --check 而点了崩溃枚举用例的照旧九格；crash-case-replay 不带 --crash-cases-only 的连按条复用那一趟一起起）"),
                "next_step": (f"点崩溃枚举用例的加 {CRASH_GATE_CASES_ONLY_FORM}：{rewritten}；只点 {CRASH_GATE_FEATURE_ITEM_PREFIX}<文件名> 的不带 --check、"
                              f"不与崩溃枚举用例写在同一条（只跑 feature-gated-tests 一格、只编点名的文件）；{UNNAMED_RUN_EXEMPTION_TEXT}")}
    cells = checked_cells(arguments)
    if not cells:
        if broken("unnamed-stage-whole-accepted", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没带 --item 也没带 --check（门禁 {stage} 的全部格、全部项）",
                "next_step": f"改成点名：{named_form('--item <项>')}（可点的项用 --list-items 列），或只点不起测试的格 --check <格>（格名用 --list 列）；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    starting = sorted(cells & set(test_starting_gate_cells(environment).get(stage, ())))
    if not starting or broken("unnamed-check-cells-accepted", environment):
        return None
    return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 的 --check 点了起测试的格 {'、'.join(starting)}，又没带 --item（起测试的格只点格不算点名）",
            "next_step": f"改成点名项：{named_form('--item <项>')}（可点的项用 --list-items 列；崩溃枚举用例写 --check crash-case-replay --crash-cases-only --item crash-case:<用例名>）；{UNNAMED_RUN_EXEMPTION_TEXT}"}


def scope_declaration_segments(environment, command):
    """范围声明 SINGLEFS_RUN_SCOPE 的原文照「;」切成的几段（确认单逐段打）：任务带的环境里的，或命令里起头的 env 后面写的；都没有交 []。"""
    text = (environment or {}).get("SINGLEFS_RUN_SCOPE", "")
    if not text and command and os.path.basename(command[0]) == "env":
        text = next((word.split("=", 1)[1] for word in command[1:] if word.startswith("SINGLEFS_RUN_SCOPE=")), "")
    return [segment.strip() for segment in text.split(";") if segment.strip()]


def declared_scope_kind(environment, command):
    """范围声明 SINGLEFS_RUN_SCOPE 里「种类=」那一段的值（whole、items、target 这几种，task_server.py 文件头 ⑦「只有长任务要确认」认 whole）；没带声明、没有这一段交空串。"""
    for segment in scope_declaration_segments(environment, command):
        if segment.startswith(SCOPE_KIND_PREFIX):
            return segment[len(SCOPE_KIND_PREFIX):].strip()
    return ""


def iso_epoch(text):
    """「2024-01-30T12:34:56Z」这类时刻 → 秒；认不出交 0.0。"""  # clock-times:allow 格式示例说明
    try:
        return float(calendar.timegm(time.strptime(text[:19], "%Y-%m-%dT%H:%M:%S")))
    except (TypeError, ValueError):
        return 0.0


@dataclass
class RetryDecision:
    retry: bool
    outcome: str = ""      # 不重试时的结局
    cap_bytes: int = 0     # 重试的上限
    host: str = ""         # 重试钉在哪台（空串是照原来的取法）
    why: str = ""


def retry_decision(outcome_counts, outcome, cap_bytes, host, pinned_host, maxima, environment=None):
    """outcome_counts 是含这一次在内每种类型出过几次；maxima 是 {台: 那台的总上限字节或 None（查不到）}。交 RetryDecision（文件头「重试」）。
    脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认。"""
    used = outcome_counts.get(outcome, 0)
    if outcome not in RETRY_LIMITS:
        return RetryDecision(False, outcome=outcome)
    if outcome == "squeezed-out" and broken("squeeze-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if outcome == "peer-run-failed" and broken("peer-run-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if used > RETRY_LIMITS[outcome]:
        return RetryDecision(False, outcome=outcome, why=f"{outcome} 已重试 {RETRY_LIMITS[outcome]} 次")
    if outcome != "memory-cap-hit":
        return RetryDecision(True, cap_bytes=cap_bytes, host=pinned_host, why=f"{outcome}：同一上限重排")
    grown = cap_bytes if broken("cap-growth-skipped", environment) else grown_cap(cap_bytes)
    maximum = maxima.get(host)
    if maximum is None:
        return RetryDecision(True, cap_bytes=grown, host=pinned_host or (host if broken("retry-pinned-to-host", environment) else ""),
                             why="上限乘 1.5（这台的总上限查不到），哪台装得下去哪台")
    # 用户：「涨到总上限就打回」：乘 1.5 还在总上限以内的才重排；到或超过总上限的不重排（slice 里总挂着一点回收不掉的内核内存，记满一整台永远凑不齐）
    if grown < maximum or broken("cap-unclamped", environment):
        return RetryDecision(True, cap_bytes=grown, host=pinned_host or (host if broken("retry-pinned-to-host", environment) else ""),
                             why="上限乘 1.5（还在这台的总上限以内）、哪台装得下去哪台")
    if not broken("host-switch-skipped", environment):
        others = [(other_maximum, other) for other, other_maximum in maxima.items() if other != host and other_maximum and other_maximum > grown]
        if others:
            other_maximum, other = max(others)
            return RetryDecision(True, cap_bytes=grown, host=other,
                                 why=f"乘 1.5 就到这台的总上限，换总上限更大的{'第二台' if other == 'peer' else '本机'}")
    return RetryDecision(False, outcome="memory-cap-at-maximum",
                         why=f"撞上限、再乘 1.5 就到两台的总上限（{HOST_NAMES.get(host, host)} 撞在 {mebibytes_text(cap_bytes)}）")
