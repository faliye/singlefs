"""crates-mutation-rows.py 拆出的一份。
基线的切法：按批与按块切行、长测试单列、派变异的次序、行的分组，连同这几样与判结局那几样（在 judging）的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import contextlib
import os
import re
import signal
import subprocess
import sys
import time
from crates_mutation_rows_base import (CARGO_CONFIGURATION_DIRECTORY, GPU_TEST_THREADS_RELEASED_TARGETS, MODULE_PATH, Row,
    TABLE, break_is, mutation_units, read_table, split_at_double_dash, under_temporary_prune_server)
from crates_mutation_rows_judging import (baseline_problem, results_by_test, row_baseline_output, row_libtest_selection, verdict_of_run)
from crates_mutation_rows_support import (build_selftest_repository, mutation_invocations, plain_invocations, selftest_new_library_copy,
    selftest_write, source_hash)
from crates_mutation_rows_static import (release_only_tests_module, release_registered_targets)
from crates_mutation_rows_units import (library_of, read_timing, row_content_hash, run_records_directory, selftest_library,
    selftest_seed_short_timings, selftest_write_timings)


def batch_key(arguments):
    before, after = split_at_double_dash(arguments)
    return tuple(before), tuple(word for word in after if word.startswith("--"))


def baseline_batches(rows):
    """{批的键: [这一批的行，按表序]}，批按第一行在表里的次序排。"""
    batches = collections.OrderedDict()
    for row in rows:
        batches.setdefault(batch_key(row.arguments), []).append(row)
    return batches


def baseline_chunk_filter_limit(environment):
    """重基线批每块至多几个过滤词（文件头「基线」的「重基线批切块」）：GATE_MUTATION_BASELINE_CHUNK_FILTERS，没设 16。写法由 run_stage 开头核过。"""
    text = environment.get("GATE_MUTATION_BASELINE_CHUNK_FILTERS", "").strip()
    return int(text) if text else 16


def heavy_baseline_batch(key, rows, registered):
    """这一批要不要切块（文件头「基线」的「重基线批切块」）：`--` 之前点名了 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标，或命中重测试目标只许 release（registered 是 crash_case_targets 交的那一份）。
    验证包登记的只经卡池取卡的那几个目标不另认：它们都带 gpu 特性，照 release 那一条已经算重。"""
    released = set(GPU_TEST_THREADS_RELEASED_TARGETS)
    if any(kind == "test" and value in released for kind, value in mutation_units.parse_cargo_arguments(list(key[0])).targets):
        return True
    return release_only_tests_module().release_requirement(rows[0].arguments, registered=registered).required


def split_baseline_rows(rows, limit, environment):
    """同键的行切成块 [[行…]…]，每块至多 limit 个过滤词，块内与块之间都照表序（一行自己的过滤词多过 limit 的独占一块）；
    过滤词已在某一块里、放得下的行进那一块（同一条用例不白跑两遍），别的行接在最后一块后面，放不下开新块。有一行没有过滤词的整批一块（它跑全部用例）。
    弄坏开关 heavy-batch-row-dropped：只交头一块。"""
    selections = [set(row_libtest_selection(row.arguments)[0]) for row in rows]
    if not all(selections):
        return [list(rows)]
    chunk_rows, chunk_words = [], []
    for row, words in zip(rows, selections):
        home = next((index for index, held in enumerate(chunk_words) if words & held and len(words | held) <= limit), None)
        if home is None and chunk_words and len(words | chunk_words[-1]) <= limit:
            home = len(chunk_words) - 1
        if home is None:
            chunk_rows.append([])
            chunk_words.append(set())
            home = len(chunk_words) - 1
        chunk_rows[home].append(row)
        chunk_words[home] |= words
    if break_is(environment, "heavy-batch-row-dropped"):
        return chunk_rows[:1]
    return chunk_rows


def baseline_long_test_seconds(environment):
    """长测试的门槛秒数（文件头「基线」的「没有用时的与长的测试各占一块」）：一行点名的测试上一次现跑的用时不短于它就单列一块。门槛写在函数里（不做模块级常量）。
    GATE_MUTATION_BASELINE_LONG_TEST_SECONDS 只给自证：正整数、只在临时 prune-server 底下认，别处一律取函数里的那一个。"""
    long_test_seconds = 900
    override = environment.get("GATE_MUTATION_BASELINE_LONG_TEST_SECONDS", "").strip()
    if re.fullmatch(r"[1-9][0-9]*", override) and under_temporary_prune_server(environment):
        return int(override)
    return long_test_seconds


def baseline_test_groups(rows):
    """同键的行按共用过滤词归成测试：[[行在 rows 里的下标…]…]（并查集：共用一个过滤词的行是同一条测试），组内照表序、组按头一个下标的次序；
    有一行没有过滤词（它跑全部用例）交 None。"""
    selections = [set(row_libtest_selection(row.arguments)[0]) for row in rows]
    if not all(selections):
        return None
    owner = list(range(len(rows)))

    def find(index):
        while owner[index] != index:
            owner[index] = owner[owner[index]]
            index = owner[index]
        return index
    holder_of_word = {}
    for index, words in enumerate(selections):
        for word in sorted(words):
            if word in holder_of_word:
                owner[find(index)] = find(holder_of_word[word])
            else:
                holder_of_word[word] = index
    members_of = collections.OrderedDict()
    for index in range(len(rows)):
        members_of.setdefault(find(index), []).append(index)
    return list(members_of.values())


def baseline_group_basis(group_rows, timings_directory, long_seconds):
    """(这一条测试的类别, 上一次用时或 None, 依据的文字)（文件头「基线」的「没有用时的与长的测试各占一块」）：类别 "long"、"unknown"、"short"。
    各行在用时表里上一次现跑的用时：有一行不短于 long_seconds 算长（用时取最长的）；不然有一行没有用时数就算没有用时（不知道长短的照长的办）；
    每一行都有用时、都短于它才算短。"""
    timings = [read_timing(timings_directory, row_content_hash(row)) for row in group_rows]
    known = [timing for timing in timings if timing is not None]
    if any(timing >= long_seconds for timing in known):
        return "long", max(known), f"上一次用时 {max(known):.0f} 秒"
    if len(known) < len(timings):
        return "unknown", None, "没有用时数据"
    return "short", max(known), ""


def split_rows_by_length(rows, timings_directory, long_seconds):
    """同键的行分成 ([(没有用时的测试的行, None, 依据)…], [(长测试的行, 用时, 依据)…], 短的行)（文件头「基线」的「没有用时的与长的测试各占一块」）：共用过滤词的行是同一条测试
    （baseline_test_groups），按 baseline_group_basis 分类，没有用时的与长的每条测试一组、组内与短的行都照表序，组照它们在表里头一次出现的次序交（跨批的先后由 baseline_chunks 排）。
    有一行没有过滤词的整批不分（它跑全部用例）。"""
    groups = baseline_test_groups(rows)
    if groups is None:
        return [], [], list(rows)
    unknown_groups, long_groups, short_indexes = [], [], []
    for members in groups:
        group_rows = [rows[index] for index in members]
        kind, timing, text = baseline_group_basis(group_rows, timings_directory, long_seconds)
        if kind == "unknown":
            unknown_groups.append((group_rows, timing, text))
        elif kind == "long":
            long_groups.append((group_rows, timing, text))
        else:
            short_indexes += members
    return unknown_groups, long_groups, [rows[index] for index in sorted(short_indexes)]


def baseline_chunk_label(key, rows):
    """一块基线点名的是什么：<测试目标> 的 <过滤词…>（报数与起跑那一行用）。"""
    targets = "、".join(f"{kind} {value}" if value else kind for kind, value in mutation_units.parse_cargo_arguments(list(key[0])).targets) or "（没点名测试目标）"
    words = []
    for row in rows:
        words += [word for word in row_libtest_selection(row.arguments)[0] if word not in words]
    return f"{targets} 的 {'、'.join(words)}"


def baseline_chunks(rows, registered, environment, timings_directory):
    """这一趟要派的基线 ([(批的键, 这一块的行)], {没有用时的测试独占那一块的头一行的行号: 起跑那一行写的点名文字}, {长测试独占那一块的头一行的行号: 同})
    （文件头「基线」的「重基线批切块」「没有用时的与长的测试各占一块」）：先按用时把每一批里的测试分成没有用时的、长的、短的（split_rows_by_length，timings_directory 是用时表目录），
    次序是三段：短的行（轻的批整批在前、照批的次序，重的批照过滤词个数切块排在后面）、没有用时的测试各占一块（照表序）、长测试各占一块（用时从短到长，先出结局的先出）。
    弄坏开关 heavy-batch-unsplit：不切、不挪、不分；long-baseline-not-isolated：没有用时的与长的都不单列；unknown-timing-merged-as-short：没有用时的照短的并进按个数切的块。"""
    batches = baseline_batches(rows)
    if break_is(environment, "heavy-batch-unsplit"):
        return list(batches.items()), {}, {}
    limit = baseline_chunk_filter_limit(environment)
    long_seconds = baseline_long_test_seconds(environment)
    isolated = not break_is(environment, "long-baseline-not-isolated")
    unknown_merged = break_is(environment, "unknown-timing-merged-as-short")
    light, heavy, unknown_chunks, long_chunks = [], [], [], []
    for key, batch_rows in batches.items():
        unknown_groups, long_groups, short_rows = split_rows_by_length(batch_rows, timings_directory, long_seconds) if isolated else ([], [], list(batch_rows))
        if unknown_merged:
            short_rows = sorted(short_rows + [row for group_rows, _timing, _text in unknown_groups for row in group_rows], key=lambda row: row.line_number)
            unknown_groups = []
        if short_rows and heavy_baseline_batch(key, batch_rows, registered):
            heavy += [(key, chunk) for chunk in split_baseline_rows(short_rows, limit, environment)]
        elif short_rows:
            light.append((key, short_rows))
        unknown_chunks += [(key, group_rows, timing, text) for group_rows, timing, text in unknown_groups]
        long_chunks += [(key, group_rows, timing, text) for group_rows, timing, text in long_groups]
    unknown_chunks.sort(key=lambda chunk: chunk[1][0].line_number)
    long_chunks.sort(key=lambda chunk: (chunk[2], chunk[1][0].line_number))

    def notes_of(chunks):
        return {group_rows[0].line_number: f"{baseline_chunk_label(key, group_rows)}（{text}；批首行是变异表第 {group_rows[0].line_number} 行）"
                for key, group_rows, _timing, text in chunks}
    isolated_chunks = [(key, group_rows) for key, group_rows, _timing, _text in unknown_chunks + long_chunks]
    return light + heavy + isolated_chunks, notes_of(unknown_chunks), notes_of(long_chunks)


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：同一个工作进程连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def rows_grouped_by_crate(rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接（行多的堆在前）。"""
    groups = {}
    for row in rows:
        groups.setdefault(crate_of(row.path), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


def rows_grouped_by_configuration(rows):
    """先按编译配置（cargo 参数在 `--` 之前的那几个词：包、测试目标、--release、特性）归堆，每堆里再按 crate、改的文件归堆，堆内按行号：
    一个工作进程领到的相邻几格编同一个测试目标、改同一个 crate 里的同一个文件，增量编译的命中才稳，少在 dev、release、gpu 与各测试目标之间来回重编下游。
    堆与堆首尾相接：编译配置行多的在前、crate 行多的在前，同样多的按名字（编译配置按词的元组比）。弄坏开关 dispatch-by-crate-only：只按 crate 归堆（rows_grouped_by_crate）。"""
    configurations = {}
    for row in rows:
        configurations.setdefault(tuple(split_at_double_dash(row.arguments)[0]), []).append(row)
    ordered = []
    for configuration in sorted(configurations, key=lambda key: (-len(configurations[key]), key)):
        by_crate = {}
        for row in configurations[configuration]:
            by_crate.setdefault(crate_of(row.path), []).append(row)
        for crate_name in sorted(by_crate, key=lambda name: (-len(by_crate[name]), name)):
            ordered.extend(sorted(by_crate[crate_name], key=lambda row: (row.path, row.line_number)))
    return ordered


def dispatch_order(rows, timings_directory, long_row_seconds, environment):
    """(派变异的次序, 前移了几条)：上一次用时不短于 long_row_seconds 的行先派、按用时从长到短，其余照按编译配置与 crate、改的文件归堆的次序（rows_grouped_by_configuration）。"""
    grouped = rows_grouped_by_crate if break_is(environment, "dispatch-by-crate-only") else rows_grouped_by_configuration
    if break_is(environment, "timingsignored"):
        return grouped(rows), 0
    long_rows = []
    for row in rows:
        timing = read_timing(timings_directory, row_content_hash(row))
        if timing is not None and timing >= long_row_seconds:
            long_rows.append((timing, row))
    long_rows.sort(key=lambda item: (-item[0], item[1].line_number))
    moved_lines = {row.line_number for _seconds, row in long_rows}
    return [row for _seconds, row in long_rows] + grouped([row for row in rows if row.line_number not in moved_lines]), len(long_rows)


def selftest_libtest_result_cell(selftest, work):
    """libtest 结果行经 lib_libtest_result_lines.py：should panic 的用例红了判抓到；点名省掉模块路径时只对上全名以「::<点名>」收尾的那一个。"""
    row = Row(8, "selftest-should-panic", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib", "answer_panics", [])
    output = ("running 2 tests\ntest tests::answer_is_42 ... ok\ntest tests::answer_panics - should panic ... FAILED\n\n"
              "test result: FAILED. 1 passed; 1 failed; 0 ignored\n\nerror: test failed, to rerun pass `-p selftest --lib`\n")
    verdict, text = verdict_of_run(row, 101, output, environment=os.environ)
    results = results_by_test("answer_panics", output + "test other::not_answer_panics ... ok\n")
    selftest.check("should panic 的用例红了判抓到、点名省掉模块路径只对上那一个",
                   verdict == "caught" and results == {"tests::answer_panics": ["FAILED"]}, f"判成 {verdict}：{text[:200]}；对上的 {results}")


def selftest_split_result_line_cell(selftest, work):
    """结果词被测试中途打的行隔开：`test <名> ... ` 之后隔着 GPU_LIFECYCLE 行、结局另起一行单独写。
    两条行共一批基线：各自照认基线绿（只看自己过滤词选中的那条）；隔开的 FAILED 照认抓到；起头之后先来了别的起头的那一条认不出结局，不接；
    没起头可接的单独结局行多出来时整段不接。弄坏开关 split-result-line-unseen：不接，头一问判成「一行都没出现」。"""
    environment = os.environ
    first = Row(8, "selftest-split-first", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib -- the_gpu_judge_reddens_first", "the_gpu_judge_reddens_first", [])
    second = Row(9, "selftest-split-second", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib -- the_gpu_judge_reddens_second", "the_gpu_judge_reddens_second", [])
    lifecycle = SELFTEST_SPLIT_LIFECYCLE_LINE
    split_output = ("running 2 tests\n"
                    "test tests::the_gpu_judge_reddens_first ... " + lifecycle.format(name="first") + "\n" + lifecycle.format(name="first") + "\nok\n"
                    "test tests::the_gpu_judge_reddens_second ... " + lifecycle.format(name="second") + "\nFAILED\n\n"
                    "test result: FAILED. 1 passed; 1 failed; 0 ignored\n")
    first_view = row_baseline_output(first, split_output, environment)
    second_view = row_baseline_output(second, split_output, environment)
    first_problem = baseline_problem(first, 101, first_view)
    first_results = results_by_test(first.expected, first_view, environment)
    second_results = results_by_test(second.expected, split_output, environment)
    verdict, text = verdict_of_run(second, 101, split_output, environment)
    selftest.check("结果词被插进来的 GPU_LIFECYCLE 行隔开时照认基线绿：一批两条各看自己的、隔开的 FAILED 判抓到",
                   first_problem is None and first_results == {"tests::the_gpu_judge_reddens_first": ["ok"]}
                   and first_view.count("test tests::") == 1 and second_view.count("test tests::") == 1
                   and second_results == {"tests::the_gpu_judge_reddens_second": ["FAILED"]} and verdict == "caught",
                   f"第一条的基线问题 {first_problem}，对上的 {first_results}；第二条对上的 {second_results}，判成 {verdict}：{text[:160]}")
    interrupted = ("running 2 tests\ntest tests::lost_one ... " + lifecycle.format(name="lost_one") + "\n"
                   "test tests::kept_one ... " + lifecycle.format(name="kept_one") + "\nok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n")
    lost_results = results_by_test("lost_one", interrupted, environment)
    orphaned = split_output + "ok\n"
    orphaned_results = results_by_test("the_gpu_judge_reddens_first", orphaned, environment)
    selftest.check("不该接的不接：起头之后先来了别的起头的那一条认不出结局、没起头可接的单独结局行多出来时整段不接",
                   lost_results == {} and orphaned_results == {},
                   f"被打断的那一条对上 {lost_results}，多出一行单独结局时第一条对上 {orphaned_results}")


def selftest_coverage_exact_cell(selftest, work):
    """记覆盖时点名用例前自己补的 --exact，不与行里已有的 --exact 叠成两个（libtest 报 Option 'exact' given more than once、覆盖记不成）：
    一行 libtest 参数已带 --exact 的变异，假 cargo 见两个 --exact 退 101；这一行抓到、覆盖记成 1 份，每一趟带 --exact 的 cargo 命令里 --exact 只出现一次。
    行里 --exact 后面写全名 tests::adds：libtest 带 --exact 时过滤词要与全名逐字相同（基线按行取结果的 row_baseline_output 与假 cargo 都照这一条选），
    写短名 adds 真 libtest 一条都不跑，基线判「一行都没出现」。弄坏开关 exactduplicated 下必须判错。"""
    repository = os.path.join(work, "coverage-exact")
    build_selftest_repository(repository, ["add-becomes-mul-exact\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- --exact tests::adds\tadds"])
    exit_code, stdout, stderr, invocations = selftest.run(repository)
    exact_lines = [line.split("\t")[0] for line in invocations if "--exact" in line.split("\t")[0].split()]
    selftest.check("行里已带 --exact 的变异：覆盖记成 1 份、没记成 0 份，带 --exact 的每一条 cargo 命令里 --exact 只出现一次（记覆盖那一趟也是）",
                   exit_code == 0 and "覆盖：这一趟记了 1 份" in stdout and "没记成 0 份" in stdout and exact_lines
                   and all(line.split().count("--exact") == 1 for line in exact_lines) and any("--message-format" not in line and "--no-run" not in line for line in exact_lines),
                   f"退 {exit_code}；带 --exact 的命令 {exact_lines}；{stdout[-300:]}；{stderr[-300:]}")


def selftest_not_run_item_name_cell(selftest, work):
    """本次未跑认项名形态（not_run_reason，文件头「本次未跑」）：点名的测试 ok、它那一段输出里是崩溃枚举用例判段不跑打的
    NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::<项名>（项名 fixed_script::stale_tail 的末一段不是测试名），记本次未跑、不记没红；
    同一份输出去掉那一行照记没红。弄坏开关 not-run-test-name-only（只认 test= 对得上测试名的）下这一格判错。"""
    environment = dict(selftest.base_environment)
    row = Row(7, "judge-step-skipped", "crates/tiny/src/lib.rs", "{ 1 }", "{ 2 }", "-p tiny --lib -- stale_tail_recovers_every_crash_state",
              "stale_tail_recovers_every_crash_state", ())
    verdict, text = verdict_of_run(row, 0, SELFTEST_NOT_RUN_ITEM_OUTPUT, environment)
    plain, plain_text = verdict_of_run(row, 0, SELFTEST_NOT_RUN_ITEM_OUTPUT.replace(SELFTEST_NOT_RUN_ITEM_LINE, ""), environment)
    selftest.check("本次未跑认项名：点名的测试 ok、输出里是 test=crash_state_enumeration::<项名> 的本次未跑行（项名不是测试名），记本次未跑、不记没红；没有那一行照记没红",
                   verdict == "notrun" and "test=crash_state_enumeration::fixed_script::stale_tail" in text and "对不上点名的测试名" in text and plain == "failure",
                   f"有那一行判 {verdict}（{text}）；没有那一行判 {plain}（{plain_text.splitlines()[0] if plain_text else ''}）")


def selftest_not_run_mid_line_cell(selftest, work):
    """本次未跑的记号不在行首也认（not_run_marks_in、not_run_reason，文件头「本次未跑」）：点名的测试 ok，它那一段输出里用例自己没换行就打了记号——接在别的输出后面、前面有颜色码、
    同一行里别的用例的记号排在点名的那一处前面——都记本次未跑、不记没红，reason= 取点名的那一处自己的；同一份输出里没有记号、或记号是更长的词
    （NOT_RUN_NO_GPU_ADAPTER_X）的照记没红。认法是门禁那一份库的 marker_pieces。弄坏开关 not-run-line-start-only（只认行首的记号）下这一格判错。"""
    environment = dict(selftest.base_environment)
    row = Row(7, "judge-step-skipped-mid-line", "crates/tiny/src/lib.rs", "{ 1 }", "{ 2 }", "-p tiny --lib -- mid_line_mark", "mid_line_mark", ())
    frame = ("running 1 test\ntest tests::mid_line_mark ... ok\n\nsuccesses:\n\n---- tests::mid_line_mark stdout ----\n{body}\n\n"
             "successes:\n    tests::mid_line_mark\n\ntest result: ok. 1 passed; 0 failed\n")
    bodies = {
        "mid-line": "进度 42% NOT_RUN_NO_GPU_ADAPTER test=tests::mid_line_mark reason=样本：接在别的输出后面",
        "colored": "\x1b[33mNOT_RUN_NO_GPU_ADAPTER test=tests::mid_line_mark reason=样本：前面有颜色码\x1b[0m",
        "after-another-mark": ("NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::other_item reason=样本：别的用例的 "
                               "NOT_RUN_NO_GPU_ADAPTER test=tests::mid_line_mark reason=样本：点名的那一处排在别的记号后面"),
    }
    wanted = {"mid-line": "样本：接在别的输出后面", "colored": "样本：前面有颜色码", "after-another-mark": "样本：点名的那一处排在别的记号后面"}
    problems = []
    for name, body in bodies.items():
        verdict, text = verdict_of_run(row, 0, frame.format(body=body), environment)
        if verdict != "notrun" or f"（{wanted[name]}）" not in text:
            problems.append(f"{name}：应当判 notrun、点名那一处自己的 reason=（{wanted[name]}），实际 {verdict}（{text.splitlines()[0] if text else ''}）")
    for name, body in (("no-mark", ""), ("longer-word", "进度 42% NOT_RUN_NO_GPU_ADAPTER_X test=tests::mid_line_mark reason=样本：记号是更长的词")):
        verdict, text = verdict_of_run(row, 0, frame.format(body=body), environment)
        if verdict != "failure":
            problems.append(f"{name}：没有记号、或记号是更长的词，应当照记没红（failure），实际 {verdict}（{text.splitlines()[0] if text else ''}）")
    selftest.check("本次未跑的记号不在行首也认：接在别的输出后面、前面有颜色码、排在同一行别的记号后面的，记本次未跑、取点名那一处自己的 reason=；没有记号、记号是更长的词的照记没红",
                   not problems, "；".join(problems))


def selftest_device_lost_marker(thread_name):
    """假的标记行：线程名是 thread_name（main、unnamed-thread、测试全名都是真会出现的）；卡名带空格。"""
    return f"GPU_DEVICE_LOST test={thread_name} {SELFTEST_DEVICE_LOST_FIELDS}\n"


def selftest_device_lost_output(result, own_body, other_body=None, live_line="", trailing_line=""):
    """假 libtest 输出：点名的测试 tests::gpu_parity 的结果是 result（ok 或 FAILED），它那一段输出是 own_body（可以是空串）；
    other_body 给了，另有一条用例 tests::other_gpu_case 也红了，它那一段输出是 other_body；
    live_line 给了，它夹在实时打印的部分（结果行之前，没带输出捕获的线程打的行）；trailing_line 给了，它接在「test result:」之后。这两处都在所有段之外。"""
    block = "failures" if result == "FAILED" else "successes"
    others = other_body is not None
    sections = f"---- tests::gpu_parity stdout ----\n{own_body}\n" + (f"---- tests::other_gpu_case stdout ----\n{other_body}\n" if others else "")
    return (f"running {2 if others else 1} test{'s' if others else ''}\n" + live_line
            + f"test tests::gpu_parity ... {result}\n"
            + ("test tests::other_gpu_case ... FAILED\n" if others else "")
            + f"\n{block}:\n\n{sections}\n{block}:\n    tests::gpu_parity\n" + ("    tests::other_gpu_case\n" if others else "")
            + f"\ntest result: {'FAILED. 0 passed; 1 failed' if result == 'FAILED' else 'ok. 1 passed; 0 failed'}\n" + trailing_line)


def selftest_device_lost_cell(selftest, work):
    """设备丢失（device_lost_line_of_test、device_lost_line_outside_sections，文件头「本次未跑」）：点名的测试 FAILED 且它自己那一段输出里有行首 GPU_DEVICE_LOST 的行，
    记本次未跑、不记抓到，test= 是 main、unnamed-thread、点名的全名都一样；FAILED 而那一行在所有段之外（实时打印的部分、「test result:」之后）同样记本次未跑；
    FAILED 而自己那一段没有、所有段之外也没有（断言红）、标记行只在别的用例的那一段里（含那一行的 test= 正是点名的全名）、标记不在行首，照记抓到；
    ok 带标记（段里、段外）照 ok 的判法（与不带的判得一样）。弄坏开关 device-lost-ignored（不认这一行）下头一条与段外那一条判错。"""
    environment = dict(selftest.base_environment)
    row = Row(7, "gpu-parity-changed", "crates/tiny/src/lib.rs", "{ 1 }", "{ 2 }", "-p tiny --lib -- gpu_parity", "gpu_parity", ())
    marker_main = selftest_device_lost_marker("main")
    lost = {name: verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC + selftest_device_lost_marker(name)), environment)
            for name in ("main", "unnamed-thread", "tests::gpu_parity")}
    bare = verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC), environment)
    in_other_case = verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, selftest_device_lost_marker("tests::other_gpu_case")), environment)
    in_other_case_under_own_name = verdict_of_run(
        row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, selftest_device_lost_marker("tests::gpu_parity")), environment)
    not_at_line_start = verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC + "progress " + marker_main), environment)
    outside = {place: verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, **{place: selftest_device_lost_marker("wgpu-callback")}), environment)
               for place in ("live_line", "trailing_line")}
    outside_and_in_other_case = verdict_of_run(
        row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, selftest_device_lost_marker("tests::other_gpu_case"),
                                              live_line=selftest_device_lost_marker("wgpu-callback")), environment)
    outside_not_at_line_start = verdict_of_run(
        row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, live_line="progress " + marker_main), environment)
    passed_with_line = verdict_of_run(row, 0, selftest_device_lost_output("ok", marker_main), environment)
    outside_after_the_test_line = verdict_of_run(row, 101, selftest_device_lost_output("FAILED", SELFTEST_DEVICE_LOST_PANIC, live_line="test tests::gpu_parity ... " + marker_main), environment)
    passed_with_line_outside = verdict_of_run(row, 0, selftest_device_lost_output("ok", "", live_line=marker_main), environment)
    passed_bare = verdict_of_run(row, 0, selftest_device_lost_output("ok", ""), environment)
    selftest.check("设备丢失：点名的测试 FAILED、自己那一段里有行首 GPU_DEVICE_LOST（test= 是 main、unnamed-thread、点名的全名各一），都记本次未跑（点名卡名与一句），不记抓到",
                   all(verdict == "notrun" and "NVIDIA GeForce RTX 5090" in text and "Xid 31：样本里设备丢失" in text for verdict, text in lost.values()),
                   "；".join(f"test={name} 判 {verdict}（{text}）" for name, (verdict, text) in lost.items()))
    selftest.check("设备丢失：点名的测试 FAILED、自己那一段里没有（断言红）、所有段之外也没有，照记抓到", bare[0] == "caught", f"判 {bare[0]}（{bare[1]}）")
    selftest.check("设备丢失：点名的测试 FAILED、标记行只在别的用例的那一段里照记抓到（那一行的 test= 是别的名字、是点名的全名都一样，看的是哪一段、不比 test=；段里的行不算进段外）",
                   in_other_case[0] == "caught" and in_other_case_under_own_name[0] == "caught",
                   f"test= 是别的名字判 {in_other_case[0]}（{in_other_case[1]}）；test= 是点名的全名判 {in_other_case_under_own_name[0]}（{in_other_case_under_own_name[1]}）")
    selftest.check("设备丢失：点名的测试 FAILED、标记行在所有段之外（实时打印的部分、「test result:」之后各一）而自己那一段只有断言，记本次未跑、写明在段外（归属不明）；别的用例的那一段里另有一行也一样",
                   all(verdict == "notrun" and "所有 ---- <名> stdout ---- 段之外" in text and "wgpu-callback" in text for verdict, text in (*outside.values(), outside_and_in_other_case)),
                   "；".join(f"{place} 判 {verdict}（{text}）" for place, (verdict, text) in outside.items()) + f"；段外加别的用例的段判 {outside_and_in_other_case[0]}（{outside_and_in_other_case[1]}）")
    selftest.check("设备丢失：标记行在段外、接在 libtest 不换行的「test <全名> ... 」后面（--test-threads 1 的形态），记本次未跑",
                   outside_after_the_test_line[0] == "notrun", f"判 {outside_after_the_test_line[0]}（{outside_after_the_test_line[1]}）")
    selftest.check("设备丢失：标记不在行首（前面接着别的字）不认，点名的测试 FAILED 照记抓到（段里、段外各一）",
                   not_at_line_start[0] == outside_not_at_line_start[0] == "caught",
                   f"段里判 {not_at_line_start[0]}（{not_at_line_start[1]}）；段外判 {outside_not_at_line_start[0]}（{outside_not_at_line_start[1]}）")
    selftest.check("设备丢失：点名的测试 ok 带标记（测试自己认下；在自己那一段里、在段外各一）照 ok 的判法，与不带的判一样（没红）",
                   passed_with_line[0] == passed_with_line_outside[0] == passed_bare[0] == "failure",
                   f"段里带标记判 {passed_with_line[0]}；段外带标记判 {passed_with_line_outside[0]}；不带判 {passed_bare[0]}")


def selftest_heavy_chunk_cells(selftest):
    """切块的算法（文件头「基线」的「重基线批切块」），只调函数、不起 cargo。每一行都在用时表里记了 10 秒（短的）：没有用时的测试各占一块，不在这一格比的范围里。"""
    environment = dict(selftest.base_environment)
    registered = release_registered_targets()
    timings_directory = os.path.join(selftest.work, "chunk-timings-short-known")

    def row_of(line_number, arguments):
        fields = (f"selftest-chunk-{line_number}", "crates/tiny/src/lib.rs", "a", "b", arguments, "unused")   # 六段各行不同：用时表里一行一份
        return Row(line_number, fields[0], fields[1], "a", "b", arguments, "unused", fields)
    heavy_rows = [row_of(100 + index, f"-p tiny --release --features gpu --lib -- case_{index:02d}_holds") for index in range(40)]
    light_rows = [row_of(10, "-p tiny --lib -- adds"), row_of(11, "-p tiny --lib -- doubles")]
    selftest_write_timings(timings_directory, heavy_rows + light_rows, 10)
    chunks, unknown_notes, long_notes = baseline_chunks(heavy_rows[:20] + light_rows + heavy_rows[20:], registered, environment, timings_directory)
    heavy_chunks = [chunk for key, chunk in chunks if "gpu" in key[0]]
    selftest.check("重基线批切块：同键 40 行的重目标按过滤词切成 16、16、8 三块，每行恰好进一块、次序不变，批的键不动，轻的批不切、排在重的前面",
                   [len(chunk) for chunk in heavy_chunks] == [16, 16, 8] and [row for chunk in heavy_chunks for row in chunk] == heavy_rows
                   and {key for key, _chunk in chunks if "gpu" in key[0]} == {batch_key(heavy_rows[0].arguments)}
                   and chunks[0] == (batch_key(light_rows[0].arguments), light_rows) and len(chunks) == 4 and unknown_notes == {} and long_notes == {},
                   f"{[(key, [row.line_number for row in chunk]) for key, chunk in chunks]}；没有用时的 {unknown_notes}；长测试 {long_notes}")
    shared_filter = [row_of(200 + index, f"-p tiny --features gpu --lib -- {word}") for index, word in enumerate(("alpha", "beta", "gamma", "alpha", "delta"))]
    unfiltered = [row_of(300, "-p tiny --features gpu --lib -- alpha"), row_of(301, "-p tiny --features gpu --lib"), row_of(302, "-p tiny --features gpu --lib -- beta")]
    released = GPU_TEST_THREADS_RELEASED_TARGETS[0]
    released_rows = [row_of(400 + index, f"-p tiny --test {released} -- case_{index:02d}_holds") for index in range(3)]
    limited = dict(environment, GATE_MUTATION_BASELINE_CHUNK_FILTERS="2")
    selftest_write_timings(timings_directory, shared_filter + unfiltered + released_rows, 10)

    def lines_of(rows):
        return [[row.line_number for row in chunk] for _key, chunk in baseline_chunks(rows, registered, limited, timings_directory)[0]]
    selftest.check("重基线批切块：同一个过滤词的行进同一块；有一行没有过滤词的批不切（整批跑全部用例）；点名放开线程的测试目标的批不带 gpu 特性也切",
                   lines_of(shared_filter) == [[200, 201, 203], [202, 204]] and lines_of(unfiltered) == [[300, 301, 302]]
                   and lines_of(released_rows) == [[400, 401], [402]],
                   f"同一个过滤词 {lines_of(shared_filter)}；没有过滤词 {lines_of(unfiltered)}；放开线程的目标 {lines_of(released_rows)}")
    no_filter_at_all = [row_of(310, "-p tiny --features gpu --lib"), row_of(311, "-p tiny --features gpu --lib")]
    selftest_write_timings(timings_directory, no_filter_at_all, 10)
    limit_answers = (baseline_chunk_filter_limit({}), baseline_chunk_filter_limit({"GATE_MUTATION_BASELINE_CHUNK_FILTERS": "5"}), lines_of(no_filter_at_all))
    selftest.check("切块大小不写死在调用处：GATE_MUTATION_BASELINE_CHUNK_FILTERS 设了照它、没设 16；整批都没有过滤词的不切",
                   limit_answers == (16, 5, [[310, 311]]), f"{limit_answers}")


def selftest_heavy_chunk_repository(directory, cases):
    """切块那几格的小仓：cases 是 [(用例全名 <模块>::<名>, 基数, 基线绿不绿)]，每条一个函数 `{ <基数> + 1 }` 与它的 #[test]，变异把 + 1 改成 + 9；
    基线不绿的那条让假 cargo 判它要的字在源码里没有。行都带 gpu 特性（命中重测试目标），过滤词写全名、必须红的测试名写短名。"""
    comments, functions, modules, rows = [], [], {}, []
    for full_name, base, green in cases:
        module, name = full_name.split("::")
        function = f"value_of_{module}_{name}"
        comments.append(f"// TEST {full_name} NEEDS {base} + {1 if green else 7}\n")
        functions.append(f"pub fn {function}() -> u32 {{ {base} + 1 }}\n")
        modules.setdefault(module, []).append(f"    #[test]\n    fn {name}() {{ assert_eq!(super::{function}(), {base + 1}); }}\n")
        rows.append(f"chunk-{module}-{name}\tcrates/tiny/src/lib.rs\t{{ {base} + 1 }}\t{{ {base} + 9 }}\t-p tiny --features gpu --lib -- {full_name}\t{name}")
    tests = "".join(f"#[cfg(test)]\nmod {module} {{\n{''.join(lines)}}}\n" for module, lines in modules.items())
    build_selftest_repository(directory, rows, source="".join(comments) + "".join(functions) + tests)
    selftest_write(os.path.join(directory, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)


def selftest_baseline_timing_write_cell(selftest, work):
    """基线一块判完记下点名测试的用时（文件头「基线」的「没有用时的与长的测试各占一块」）：7 行都没有用时（每条测试各占一块、7 块），一个工作进程、跑完头一条变异就停
    （GATE_MUTATION_STOP_AFTER_ROWS=1，基线全在变异之前）：用时表里 7 行都有了数（变异只判了一行，别的 6 行的数只能是基线记的）；
    库换一棵树、用时表拷过去再跑，7 条测试都有用时而短，回到按个数切的块（3 块），不再各占一块；
    弄坏开关 baseline-timing-unwritten 下基线不记，用时表里只有变异判过的那一行。"""
    cases = [("tests::amber_holds", 101, True), ("tests::birch_holds", 111, True), ("tests::cedar_holds", 121, True), ("tests::dune_holds", 131, True),
             ("tests::fern_holds", 141, True), ("tests::grove_holds", 151, True), ("extra::kiln_holds", 161, True)]
    first_repository = os.path.join(work, "baseline-timing-first")
    selftest_heavy_chunk_repository(first_repository, cases)
    second_repository = selftest_new_library_copy(work, first_repository, "baseline-timing-second")
    table_rows, _malformed = read_table(os.path.join(first_repository, TABLE))
    settings = dict(GATE_MUTATION_BASELINE_CHUNK_FILTERS="3", GATE_MUTATION_STOP_AFTER_ROWS="1")
    selftest.seeds_short_timings = False   # 没有用时的行要保持没有用时
    try:
        first = selftest.run(first_repository, **settings)
        first_timings = os.path.join(run_records_directory(first_repository, dict(selftest.base_environment)), "timings")
        written = [row for row in table_rows if read_timing(first_timings, row_content_hash(row)) is not None]
        second_timings = os.path.join(run_records_directory(second_repository, dict(selftest.base_environment)), "timings")
        for row in written:
            selftest_write_timings(second_timings, [row], read_timing(first_timings, row_content_hash(row)))
        second = selftest.run(second_repository, **settings)
    finally:
        selftest.seeds_short_timings = True
    selftest.check("基线一块判完记下点名测试的用时：7 行都没有用时（7 块、没有用时独占 7 批），跑完头一条变异就停，用时表里 7 行都有了数（别的 6 行的数只能是基线记的）；"
                   "拷到新库再跑，7 条测试都有用时而短，回到按个数切的 3 块、没有用时独占 0 批",
                   "现跑基线 7 批 7 行（其中长测试独占 0 批、没有用时独占 7 批）" in first[2] and len(written) == 7
                   and "现跑基线 3 批 7 行（其中长测试独占 0 批、没有用时独占 0 批）" in second[2],
                   f"头一趟 退 {first[0]}：{first[2][:400]}；用时表里有数的 {len(written)} 行；第二趟 退 {second[0]}：{second[2][:400]}")


def selftest_heavy_chunk_resume_cell(selftest, work):
    """被打断重跑只补没判的块（文件头「基线」的「重基线批切块」；做法同 selftest_killed_run_resume_cell）：6 行切成 3 块，一个工作进程，每次 cargo 睡 2 秒；
    下一块的 cargo 起来（头一块判完、记录写完）就对整组发 SIGKILL，等它的假 cargo 自己退，再重跑同一条命令：头一块的基线复用，只跑剩下两块。
    下一块起来时头一块没恰好判完（机器忙），这一格记本次未跑。"""
    label = "重基线批被整组硬杀：判完的块已入库，重跑同一条命令只补没判的块"
    child_script = ("import importlib.util, os, sys\n"
                    "specification = importlib.util.spec_from_file_location('crates_mutation_rows_chunk_resume_child', sys.argv[1])\n"
                    "module = importlib.util.module_from_spec(specification)\n"
                    "sys.modules[specification.name] = module\n"
                    "specification.loader.exec_module(module)\n"
                    "sys.exit(module.run_stage(sys.argv[2], {name: value for name, value in os.environ.items() if name != 'SINGLEFS_MULTI_HOST_CONFIG'}))\n")
    repository = os.path.join(work, "heavy-chunk-resume")
    selftest_heavy_chunk_repository(repository, [("tests::amber_holds", 101, True), ("tests::birch_holds", 111, True), ("tests::cedar_holds", 121, True),
                                                 ("tests::dune_holds", 131, True), ("tests::fern_holds", 141, True), ("tests::grove_holds", 151, True)])
    pristine = source_hash(repository)
    temporary = os.path.join(work, "heavy-chunk-resume-tmp")
    os.makedirs(temporary)
    target = os.path.join(work, "heavy-chunk-resume-target")
    span_log = os.path.join(work, "heavy-chunk-resume-spans.log")
    killed_environment = dict(selftest.base_environment, TMPDIR=temporary, GATE_MUTATION_WORKERS="1", GATE_MUTATION_TARGET_DIR=target, GATE_MUTATION_BASELINE_CHUNK_FILTERS="2",
                              FAKE_CARGO_SPAN_LOG=span_log, FAKE_CARGO_SPAN_SLEEP="2")
    selftest_seed_short_timings(repository, killed_environment)   # 每一行都有用时（短的）：切成 3 块，不是没有用时的 6 块

    def span_events():
        if not os.path.exists(span_log):
            return []
        with open(span_log, encoding="utf-8") as handle:
            return [line.split("\t") for line in handle.read().splitlines() if line.count("\t") == 2]

    def baseline_events(kind):
        return [event for event in span_events() if event[0] == kind and event[1].startswith("baseline-")]

    def start_child(child_environment, stdout_handle, stderr_handle):
        return subprocess.Popen([sys.executable, "-c", child_script, MODULE_PATH, repository], env=selftest.pool_process_environment(child_environment), start_new_session=True,
                                stdout=stdout_handle, stderr=stderr_handle)
    stderr_path = os.path.join(work, "heavy-chunk-resume-stderr.txt")
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = start_child(killed_environment, subprocess.DEVNULL, stderr_handle)
    deadline = time.monotonic() + 300
    while len(baseline_events("start")) < 2 and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.02)
    started, ended = len(baseline_events("start")), len(baseline_events("end"))
    exited_early = child.poll() is not None
    if not exited_early:
        os.killpg(child.pid, signal.SIGKILL)   # 整组：子进程自己是会话首进程，工作进程在同一个进程组里；只对自证自己起的这一组发
    child.wait(timeout=60)
    settle_deadline = time.monotonic() + 60
    while time.monotonic() < settle_deadline and len([event for event in span_events() if event[0] == "start"]) != len([event for event in span_events() if event[0] == "end"]):
        time.sleep(0.05)
    with open(stderr_path, encoding="utf-8", errors="replace") as handle:
        killed_stderr = handle.read()
    if exited_early:
        selftest.check(label, False, f"被杀的那一趟在被杀之前自己退了（退 {child.returncode}，基线起 {started} 块、判完 {ended} 块，只有一块就是没切）；stderr 尾：{killed_stderr[-400:]}")
        return
    if (started, ended) != (2, 1):
        selftest.skip(label, f"下一块起来时头一块没恰好判完（起 {started} 块、判完 {ended} 块）")
        return
    library = selftest_library(library_of(repository))
    recorded = sum(1 for name in library.stored_names("baselines") if "verdict=ok" in (library.read_text("baselines", name) or ""))
    with contextlib.suppress(FileNotFoundError):
        os.unlink(selftest.log)
    rerun_environment = dict(selftest.base_environment, GATE_MUTATION_WORKERS="1", GATE_MUTATION_TARGET_DIR=target, GATE_MUTATION_BASELINE_CHUNK_FILTERS="2")
    rerun_stdout_path, rerun_stderr_path = os.path.join(work, "heavy-chunk-resume-rerun-stdout.txt"), os.path.join(work, "heavy-chunk-resume-rerun-stderr.txt")
    with open(rerun_stdout_path, "w", encoding="utf-8") as stdout_handle, open(rerun_stderr_path, "w", encoding="utf-8") as stderr_handle:
        rerun_child = start_child(rerun_environment, stdout_handle, stderr_handle)
    try:
        rerun_exit = rerun_child.wait(timeout=600)
    except subprocess.TimeoutExpired:
        os.killpg(rerun_child.pid, signal.SIGKILL)   # 只对自证自己起的这一组发
        rerun_child.wait(timeout=60)
        rerun_exit = f"600 秒没跑完、整组杀掉（退 {rerun_child.returncode}）"
    with open(rerun_stderr_path, encoding="utf-8", errors="replace") as handle:
        rerun_stderr = handle.read()
    rerun_invocations = []
    if os.path.exists(selftest.log):
        with open(selftest.log, encoding="utf-8") as handle:
            rerun_invocations = handle.read().splitlines()
    rerun_baselines = [line for line in plain_invocations(rerun_invocations) if line.endswith("\t" + pristine)]
    selftest.check(label,
                   child.returncode == -signal.SIGKILL and recorded == 2 and rerun_exit == 0 and "现跑基线 2 批 4 行" in rerun_stderr and len(rerun_baselines) == 2
                   and not any("amber_holds" in line or "birch_holds" in line for line in rerun_baselines)
                   and len(mutation_invocations(plain_invocations(rerun_invocations), pristine)) == 6,
                   f"被杀的那一趟退 {child.returncode}，库里基线 ok {recorded} 条；重跑退 {rerun_exit}，基线 {rerun_baselines}；重跑 stderr 头 {rerun_stderr[:400]}")
