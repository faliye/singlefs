"""crates-mutation-rows.py 拆出的一份。
命令行的各个子命令（static-check、library-directory、row-plan、plan、--list-items、--item、sample-tools、llvm-tools），连同直接调子命令的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import contextlib
import io
import os
import re
import signal
import sys
import time
from crates_mutation_rows_base import (CLOSURE_REFUSED_HOWTO, COLUMN_TITLES, COPY_REBUILD_HOWTO, FAKE_CARGO, FAKE_LLVM_TOOL, GPU_ROUTE_UNITS,
    HeavyThreadDefaultError, ITEM_DONE_LINE, ITEM_PLAN_LINE, LLVM_TOOLS, REACHABILITY_CPU_FALLBACK_ROOTS, REACHABILITY_GPU_ROUTE_STOPS,
    REACHABILITY_HOWTO, RELEASE_HOWTO, RELEASE_REGISTRY_HOWTO, REUSE_COLUMNS, ROW_PROBLEM_KINDS, Row, STAGE, STAGE_REPOSITORY,
    STAGE_SCRIPT, STATIC_CHECK_REGISTRY_HOWTO, TABLE, admission_module, break_is, copied_paths, merge_progress_variable, merge_step_close, merge_step_open,
    merge_step_row_done, mutation_units, note, print_rejection, read_table)
from crates_mutation_rows_support import (build_selftest_repository, coverage_invocations, mutation_invocations, per_row_how, plain_invocations,
    replace_in_file, run_command_capturing, selftest_new_library_copy, selftest_one_forced, selftest_write, selftest_write_stage_registration_table,
    source_hash)
from crates_mutation_rows_static import (anchor_broken_rows, anchor_problems_by_line, copied_with_manifest_directories,
    copied_with_path_dependencies, escaping_literals, judged_registry_root, release_only_tests_module, release_unwritten_rows,
    replacement_undefined_identifiers_of_rows,
    row_problems, selftest_static_check_parallel_compare, selftest_static_check_parallel_repository)
from crates_mutation_rows_units import (key_workers, library_directory, library_of, read_timing, row_content_hash, run_records_directory, selftest_library)
from crates_mutation_rows_targets import (crash_domain_narrowing, narrowed_sharding_refusal)
from crates_mutation_rows_plan import (ReachabilityConfigurationError, ReachabilityNotJudged, decide_rows, impact_summary, plan_rows,
    prebuilt_workspace, reachability_of_rows, reachability_resolved_rows, read_selection, selftest_release_repository)
from crates_mutation_rows_stage import (ItemProgress, item_plan_lines, llvm_tools, unfiltered_rows, unfiltered_rows_judged)
import crates_mutation_rows_units
import crates_mutation_rows_plan


def static_check_usage_problem(arguments):
    """static-check 的参数哪里不对：→ None（对）或 (原因, 下一步)。"""
    if not arguments:
        return "缺 <根>（被判那棵树的仓根）", 'python3 scripts/mutation/crates-mutation-rows.py static-check "$PWD"（在仓根跑）'
    if arguments[0].startswith("-"):
        return f"第一个参数要是 <根>，给的是选项 {arguments[0]}", 'python3 scripts/mutation/crates-mutation-rows.py static-check "$PWD" [--table <变异表>]'
    if len(arguments) == 2 and arguments[1] == "--table":
        return "--table 后面没给变异表的路径", f"python3 scripts/mutation/crates-mutation-rows.py static-check {arguments[0]} --table <变异表>"
    if len(arguments) not in (1, 3) or (len(arguments) == 3 and arguments[1] != "--table"):
        return (f"参数个数或写法不对（给了 {len(arguments)} 个：{' '.join(arguments)}；只认 <根> 与可选的 --table <变异表>）",
                f"python3 scripts/mutation/crates-mutation-rows.py static-check {arguments[0]} [--table <变异表>]")
    if not os.path.isdir(arguments[0]):
        return f"<根> {arguments[0]} 不是一个目录", "<根> 写被判那棵树的仓根（git 工作树），例如 \"$PWD\""
    return None


def command_static_check(arguments):
    problem = None if break_is(os.environ, "staticusagebare") else static_check_usage_problem(arguments)
    if problem is not None:
        print(f"ERROR\t{problem[0]}；用法：crates-mutation-rows.py static-check <根> [--table <变异表>]")
        print(f"     → 怎么办：{problem[1]}")
        return 2
    if not arguments or len(arguments) not in (1, 3) or (len(arguments) == 3 and arguments[1] != "--table"):
        print("ERROR\t用法：crates-mutation-rows.py static-check <根> [--table <变异表>]")
        return 2
    root = arguments[0]
    table_path = arguments[2] if len(arguments) == 3 else os.path.join(root, TABLE)
    if len(arguments) == 3 and not os.path.isfile(table_path) and not break_is(os.environ, "tablemissingok"):
        print(f"ERROR\t--table 指的变异表不在：{table_path}（不判，也不报「查了 0 行」）")
        print("     → 怎么办：核对路径（相对路径相对当前工作目录）；判仓里那一张就去掉 --table（默认读 <根>/crates/mutations.tsv）")
        return 2
    admission = admission_module()
    registry_root = judged_registry_root(root, os.environ)   # 拷贝范围与重测试目标都读被判那棵树的登记表（文件头「拷贝范围」）
    registry_table = os.path.join(registry_root, admission.REGISTRATION_TABLE)
    stage_marks = [("", time.monotonic())]   # 分段用时（只打到标准错误，不进标准输出）：(这一段的名字, 段尾的时刻)
    if not os.path.isfile(registry_table):
        print(f"ERROR\t被判的树里没有登记表 {registry_table}：拷贝范围与重测试目标都读它，判不了")
        print(f"     → 怎么办：{STATIC_CHECK_REGISTRY_HOWTO}")
        return 2
    try:
        copied = copied_paths(registry_root)
    except (admission.RegistrationError, OSError) as error:
        print(f"ERROR\t读不到 {registry_table} 里 {STAGE} 那一行：{error}")
        print(f"     → 怎么办：{STATIC_CHECK_REGISTRY_HOWTO}")
        return 2
    if not copied:
        print(f"ERROR\t.claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行一条路径都没登记")
        return 2
    try:
        copied = copied_with_path_dependencies(root, copied, os.environ)
    except (ValueError, OSError) as error:
        print(f"ERROR\t拷贝范围跟不全 path 依赖：{error}")
        print(f"     → 怎么办：path 依赖指到被判那棵树之外的，改成树里的目录；Cargo.toml 读不懂的照报错修，再跑；{COPY_REBUILD_HOWTO}")
        return 2
    rows, malformed = [], []
    if os.path.isfile(table_path):
        try:
            rows, malformed = read_table(table_path)
        except (OSError, UnicodeDecodeError) as error:
            print(f"ERROR\t读不了 {table_path}：{error}")
            return 2
    try:
        copied = copied_with_manifest_directories(root, copied, rows, os.environ)
    except (ValueError, OSError) as error:
        print(f"ERROR\t拷贝范围补不上变异行 --manifest-path 指的包目录：{error}")
        print("     → 怎么办：那一行的 --manifest-path 写成仓内一份 Cargo.toml 相对仓根的路径（例 --manifest-path prune-server/Cargo.toml），只写一次；"
              "那个包的 path 依赖指到树之外的改成树里的目录，再跑")
        return 2
    try:
        workers = key_workers(os.environ)   # 逐行、逐份互不依赖的几段切片并行的路数（文件头 static-check「并行」）
    except HeavyThreadDefaultError as error:
        print(f"ERROR\t取不到每台默认重活线程数（static-check 切片并行的路数）：{error}")
        print("     → 怎么办：.claude/heavy-thread-default 照它的报错修（SINGLEFS_THREAD_CAP 设了要是正整数），再跑")
        return 2
    stage_marks.append(("拷贝范围与读表", time.monotonic()))
    problems = row_problems(root, rows, malformed, copied)
    if unfiltered_rows_judged(root, os.environ):
        problems["MALFORMED"] += unfiltered_rows(rows)
    stage_marks.append(("row_problems", time.monotonic()))
    literals, scanned = escaping_literals(root, copied, workers, os.environ)
    stage_marks.append(("逃出 crates/ 的字面量", time.monotonic()))
    outside_literals = [literal for literal in literals if not literal[4]]
    print("COPIED\t" + " ".join(copied))
    for kind in ROW_PROBLEM_KINDS:
        for detail in problems[kind]:
            print("\t".join([kind] + [str(item) for item in detail]))
    for relative, line_number, literal, resolved, _inside in outside_literals:
        print(f"LITERAL\t{relative}\t{line_number}\t{literal}\t{resolved}")
    # 替换文里新引的标识符找不到定义（replacement_undefined_identifiers）：只 static-check 判，crates-mutation-replay 派活之前不判（那边套用后编不过记无效）
    anchors = anchor_problems_by_line(root, rows)   # 每份文件读一遍，标识符与可达性两段共用
    identifier_rows = [row for row in rows if row.path.endswith(".rs") and anchors[row.line_number] is None]
    identifiers_unjudged = len(rows) - len(identifier_rows)
    undefined_rows = 0
    for row, missing in zip(identifier_rows, replacement_undefined_identifiers_of_rows(root, identifier_rows, workers, os.environ)):
        if missing:
            undefined_rows += 1
            print(f"UNDEFINED\t{row.line_number}\t{row.name}\t{row.path}\t{','.join(missing)}")
    print(f"IDENTIFIERS_CHECKED\t{len(rows) - identifiers_unjudged}\t{identifiers_unjudged}")
    stage_marks.append(("锚点与标识符", time.monotonic()))
    # 命中重测试目标而第五段没写 release（文件头「重测试目标只许 release」）：只 static-check 判，crates-mutation-replay 派活之前不判（那边照补 --release 起）
    try:
        release_unwritten = release_unwritten_rows(rows, os.environ, registry_root)
    except release_only_tests_module().RegistryUnreadable as error:
        print(f"ERROR\t判不了哪几行是重测试目标：{error}")
        print(f"     → 怎么办：{RELEASE_REGISTRY_HOWTO}")
        return 2
    for line_number, name, basis in release_unwritten:
        print(f"RELEASE\t{line_number}\t{name}\t{'；'.join(basis)}")
    if release_unwritten:
        print(f"     → 怎么办：{RELEASE_HOWTO}")
    unreachable = []
    try:
        unreachable, exempted, unjudged, cut, configuration, enabled = reachability_of_rows(root, rows, anchors, os.environ)
    except ReachabilityNotJudged as error:
        print(f"REACHABILITY_NOT_JUDGED\t{error}")
    except ReachabilityConfigurationError as error:
        print(f"ERROR\t点名测试够不着被改坏的函数那一道的选路表、截断表或豁免表里有函数在 {root} 的源码里找不到：{error}")
        print("     → 怎么办：函数改了名或搬了文件，照今天的源码改 scripts/mutation/crates_mutation_rows_base.py 的 GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、"
              "REACHABILITY_CPU_FALLBACK_ROOTS（判法见 scripts/mutation/crates-mutation-rows.py 文件头「点名测试够不着被改坏的函数」），再跑")
        return 2
    except (admission.InputManifestError, mutation_units.MutationUnitsError, mutation_units.function_units().FunctionUnitsError,
            HeavyThreadDefaultError, OSError, RuntimeError) as error:
        # RuntimeError 是算截过的闭包时算法自己的错（reachability_cut_answers）：照 ERROR 退 2，不让未捕获的异常退 1 冒充「有问题行」
        print(f"ERROR\t点名测试够不着被改坏的函数那一道判不了：{error}")
        print("     → 怎么办：多机配置读不了的修那份文件（SINGLEFS_MULTI_HOST_CONFIG 指的，没设是本仓的 multi-host.env）；"
              ".claude/heavy-thread-default 读不了的照它的报错修；工作区切不开的照 scripts/gate/lib_function_units.py 的报错修，再跑")
        return 2
    else:
        for line_number, name, route, mutated, test, cargo_arguments in unreachable:
            print(f"UNREACHABLE\t{line_number}\t{name}\t{route}\t{mutated}\t{test}\t{cargo_arguments}")
        if unreachable:
            print(f"     → 怎么办：{REACHABILITY_HOWTO}")
        for line_number, name, kind, evidence in exempted:
            print(f"REACHABILITY_EXEMPT\t{line_number}\t{name}\t{kind}\t{evidence}")
        for line_number, name, why in unjudged:
            print(f"REACHABILITY_UNJUDGED\t{line_number}\t{name}\t{why}")
        print(f"REACHABILITY_CHECKED\t{len(rows) - len(unjudged)}\t{len(unjudged)}\t{len(exempted)}\t{cut}\t{configuration}\t"
              f"{'ENABLE_GPU=1' if enabled else 'GPU 没开'}")
    stage_marks.append(("RELEASE 与可达性", time.monotonic()))
    reachability_seconds = getattr(reachability_of_rows, "seconds", None) or {}
    note("  · static-check 分段用时（挂钟秒，只看哪一段串行、哪一段慢）："
         + "、".join(f"{name} {mark - earlier:.1f}" for (_previous, earlier), (name, mark) in zip(stage_marks, stage_marks[1:]))
         + (f"；可达性里：{'、'.join(f'{name} {seconds:.1f}' for name, seconds in reachability_seconds.items())}" if reachability_seconds else ""))
    print(f"CHECKED\t{len(rows)}\t{scanned}\t{len(literals)}")
    return 1 if any(problems.values()) or outside_literals or undefined_rows or release_unwritten or unreachable else 0



def command_library_directory(arguments):
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py library-directory <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    admission = admission_module()
    try:
        print(library_directory(arguments[0], dict(os.environ)))
    except (admission.InputManifestError, OSError) as error:
        print(f"  ✗ 取不到判定库的位置：{error}")
        print("     → 怎么办：被判的要是 git 工作树（rsync 拷的副本先在副本根 git init）；只给自证换的几个变量只在临时 prune-server 底下认")
        return 2
    return 0


def plan_context(root, environment, selection_path=""):
    """plan、row-plan、--list-items 共用：读表、算每一行的键与判定（与 run_stage 同一份 plan_rows、decide_rows）；不写库。
    selection_path 给了（row-plan --rows）就只对选中的行算键、判定，别的行放进 unselected_rows、不算键；弄坏开关 rowplanselectionignored 下照整张表算。
    交 dict；不成就打两行（✗ 与 →）交 None。"""
    admission = admission_module()
    try:
        rows, _malformed = read_table(os.path.join(root, TABLE), environment)
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {os.path.join(root, TABLE)}：{error}")
        print("     → 怎么办：先让这张表在、是 UTF-8；读不了就一行都判不了")
        return None
    selected_lines = None
    if selection_path and not break_is(environment, "rowplanselectionignored"):
        selected_lines, selection_problems = read_selection(selection_path, rows)
        if selection_problems:
            print(f"  ✗ row-plan 的选行文件不对：{'；'.join(selection_problems)}")
            print(f"     → 怎么办：一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）；行号照被判那一份表现数")
            return None
    # 锚点失效的行（文件头「锚点失效」）先剔出去、单列：不算键（run_stage 不跑它们，row-plan 不让双机分片的驱动把它们分给第二台）
    anchor_broken = {} if break_is(environment, "anchor-rows-planned") else anchor_broken_rows(root, rows, environment)
    anchor_rows = [(row, anchor_broken[row.line_number]) for row in rows if row.line_number in anchor_broken]
    rows = [row for row in rows if row.line_number not in anchor_broken]
    unselected_rows = []
    if selected_lines is not None:   # 选行：只对选中的行算键；别的行只给用时（row-plan 的第四列读用时表，不要键）
        unselected_rows = [row for row in rows if row.line_number not in selected_lines]
        rows = [row for row in rows if row.line_number in selected_lines]
    try:
        key_context, rows, components, library = plan_rows(root, rows, environment, table_rows=rows + unselected_rows)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return None
    except release_only_tests_module().RegistryUnreadable as error:
        print_rejection(f"判不了哪几行是重测试目标（{error}），键算不出来", [], [RELEASE_REGISTRY_HOWTO])
        return None
    except (mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print(f"  ✗ 算不出行键或判定库的位置：{error}")
        print("     → 怎么办：被判的要是 git 工作树（rsync 拷的副本先在副本根 git init）、根 Cargo.toml 读得懂")
        return None
    return dict(rows=rows, components=components, key_context=key_context, library=library,
                decided=decide_rows(library, rows, components, key_context, environment),
                anchor_rows=anchor_rows, unselected_rows=unselected_rows)


def command_row_plan(arguments):
    """每一条成形的变异一行：行号、行键（拒算写 -）、这一趟会不会复用、上一次现跑的秒数（见文件头 row-plan）；带 --rows 只对选中的行算键。
    末尾带 --progress（合并那一步用）就把核锚点、现算行键、读库、读用时表分步打进度（merge_step_open，范围「合并」）。"""
    progress_requested = arguments[-1:] == ["--progress"]
    if progress_requested:
        arguments = arguments[:-1]
    if len(arguments) not in (1, 3) or (len(arguments) == 3 and (arguments[1] != "--rows" or not arguments[2])):
        print("  ✗ 用法：crates-mutation-rows.py row-plan <根> [--rows <选行文件>] [--progress]")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）；--rows 后面是一份选行文件（一行一个变异表行号）；--progress 只能写在最后")
        return 2
    environment = dict(os.environ)
    # 缩过域的一趟不分片（文件头「缩过域的一趟」）：row-plan 只给服务分片的驱动用，计划与合并那一步都在这里拒。弄坏开关 narrowed-sharding-allowed：照列
    narrowing = crash_domain_narrowing(environment)
    if narrowing and not break_is(environment, "narrowed-sharding-allowed"):
        headline, howto_line = narrowed_sharding_refusal(narrowing)
        print(f"  ✗ {headline}")
        print(f"     → 怎么办：{howto_line}")
        return 2
    if progress_requested:
        environment[merge_progress_variable()] = "合并"
    context = plan_context(arguments[0], environment, arguments[2] if len(arguments) == 3 else "")
    if context is None:
        return 2
    try:
        timings_directory = os.path.join(run_records_directory(arguments[0], environment), "timings")
    except admission_module().InputManifestError as error:
        print(f"  ✗ 用时表在哪取不到：{error}")
        print("     → 怎么办：多机配置（仓根 multi-host.env，或 SINGLEFS_MULTI_HOST_CONFIG 指的那一份）写上 NFS_SHARED_ROOTS（文件头「判定库」末段）")
        return 2
    lines = []
    timed = 0
    progress = merge_step_open(environment, "读用时表", len(context["rows"]) + len(context["unselected_rows"]))
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        timing = read_timing(timings_directory, row_content_hash(row))
        key = context["components"][row.line_number].row_key or "-"
        lines.append((row.line_number, f"{row.line_number}\t{key}\t{1 if decision.reuse else 0}\t{'-' if timing is None else f'{timing:.3f}'}\t-"))
        timed += 1
        merge_step_row_done(progress, timed)
    # 选行之外的行：只给用时（驱动分行取没记过用时的中位数仍取全表），键写 -、不复用，不算键
    for row in context["unselected_rows"]:
        timing = read_timing(timings_directory, row_content_hash(row))
        lines.append((row.line_number, f"{row.line_number}\t-\t0\t{'-' if timing is None else f'{timing:.3f}'}\t-"))
        timed += 1
        merge_step_row_done(progress, timed)
    merge_step_close(progress, f"读了 {timed} 行的用时")
    # 锚点失效的行：没有键、不复用，第五列 anchor-broken（双机分片的驱动不把它们分给第二台）
    lines += [(row.line_number, f"{row.line_number}\t-\t0\t-\tanchor-broken") for row, _why in context["anchor_rows"]]
    for _line_number, text in sorted(lines):
        print(text)
    table_rows = len(context["rows"]) + len(context["unselected_rows"]) + len(context["anchor_rows"])
    selection_note = f"，选行文件点名了 {len(context['rows'])} 行、别的行只给用时" if len(arguments) == 3 and not break_is(environment, "rowplanselectionignored") else ""
    note(f"  · row-plan：算了 {len(context['components'])} 行的键（表里成形的 {table_rows} 行{selection_note}）")
    return 0


def command_plan(arguments):
    """只读：每一行会复用、现判还是拒算，栏与细节（变了哪几个单元），末行总数（文件头 plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）；只读，不编、不跑、不写")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment)
    if context is None:
        return 2
    columns, reused, refused = collections.Counter(), 0, 0
    lines = []
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        if decision.reuse:
            reused += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t复用\t键相同\t{decision.detail}"))
        elif decision.column == "refused":
            refused += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t拒算\t拒算\t{decision.detail}"))
        else:
            columns[decision.column] += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t现判\t{COLUMN_TITLES[decision.column]}\t{decision.detail}"))
    # 锚点失效的行（文件头「锚点失效」）：不算键、这一趟不跑
    lines += [(row.line_number, f"{row.line_number}\t{row.name}\t锚点失效\t锚点失效\t{why}") for row, why in context["anchor_rows"]]
    for _line_number, text in sorted(lines):
        print(text)
    print(f"合计 {len(context['rows']) + len(context['anchor_rows'])} 行：复用 {reused} 行、现判 {sum(columns.values())} 行（"
          + "、".join(f"{title} {columns.get(name, 0)}" for name, title in REUSE_COLUMNS) + f"）、拒算 {refused} 行、锚点失效 {len(context['anchor_rows'])} 行")
    return 0


def command_list_items(arguments):
    """可点名的变异，一行一条（门禁 --list-items 调的就是这一份），每行末尾一段写这一行的键对不对得上；末行合计键对不上的条数。"""
    with_keys = "--with-keys" in arguments
    arguments = [argument for argument in arguments if argument != "--with-keys"]
    selection_path = ""
    if "--rows" in arguments:
        index = arguments.index("--rows")
        selection_path = arguments[index + 1] if index + 1 < len(arguments) else ""
        arguments = arguments[:index] + arguments[index + 2:]
        if not selection_path or not with_keys:
            print("  ✗ 用法：crates-mutation-rows.py --list-items --with-keys --rows <选行文件> <根>")
            print("     → 怎么办：--rows 只和 --with-keys 一起用，后面是一份选行文件（一行一个变异表行号，门禁把 --item 点名的几行换成它）")
            return 2
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py --list-items [--with-keys [--rows <选行文件>]] <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根；--with-keys 另给每项一行 ITEM_PLAN（文件头「按项机读行」），再加 --rows 只列、只算点名那几行")
        return 2
    if not break_is(os.environ, "listitemspipeunhandled"):
        # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    path = os.path.join(arguments[0], TABLE)
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {path}（{error}）：列不出可点名的变异")
        print("     → 怎么办：在项目根跑，或把项目根作为参数传进来。")
        return 1
    items = [(number, line.split("\t")) for number, line in enumerate(lines, 1) if not line.startswith("#") and line.strip()]
    # 锚点失效的那几行（文件头「锚点失效」）在逐行那一段末尾接一句；判法与 run_stage 同一份（anchor_problem），成形的行才判
    environment = dict(os.environ)
    try:
        table_rows = read_table(path, environment)[0]
    except (OSError, UnicodeDecodeError):
        table_rows = []
    anchor_broken = {} if break_is(environment, "anchor-rows-planned") else anchor_broken_rows(arguments[0], table_rows, environment)
    selected = None
    if selection_path and not break_is(environment, "item-plan-selection-ignored"):
        selected, selection_problems = read_selection(selection_path, table_rows)
        if selection_problems:
            print(f"  ✗ --rows 的选行文件不对：{'；'.join(selection_problems)}")
            print(f"     → 怎么办：一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）")
            return 2
        items = [(number, fields) for number, fields in items if number in selected]
    else:
        selection_path = ""   # 弄坏开关 item-plan-selection-ignored：点名也算整表
    for number, fields in items:
        anchor_note = f"；锚点失效：{anchor_broken[number]}" if number in anchor_broken else ""
        print(f"{fields[0]}\t第 {number} 行，改 {fields[1] if len(fields) > 1 else ''}{anchor_note}", flush=True)
    status_output = io.StringIO()
    with contextlib.redirect_stdout(status_output):
        context = plan_context(arguments[0], environment, selection_path)
    if context is None:
        print("  · 键对不对得上没算出来：" + " ".join(line.strip() for line in status_output.getvalue().splitlines()))
        if with_keys:
            print("  ✗ --with-keys 要每项的键，上面这一行写着键为什么没算出来")
            print("     → 怎么办：照上一行修了再跑；不退回只列项名（没有键的 ITEM_PLAN 排不了活）。")
            return 1
        return 0
    if with_keys:
        for line in item_plan_lines([fields[0] for _number, fields in items], context, environment):
            print(line, flush=True)
    counter = collections.Counter()
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        state = "复用" if decision.reuse else "拒算" if decision.column == "refused" else "键对不上"
        counter[state] += 1
    anchor_rows = [(row, why) for row, why in context["anchor_rows"] if selected is None or row.line_number in selected]
    anchor_named = "、".join(f"第 {row.line_number} 行 {row.name}" for row, _why in anchor_rows)
    print(f"  · 键对不上 {counter['键对不上']} 条（这一趟要现判）、键对得上 {counter['复用']} 条（复用）、拒算 {counter['拒算']} 条（照跑）、"
          f"锚点失效 {len(anchor_rows)} 行{'（不跑、整道判红：' + anchor_named + '）' if anchor_named else ''}；"
          f"逐行的栏与变了哪几个单元：python3 scripts/mutation/crates-mutation-rows.py plan {arguments[0]}")
    # 牵连面（文件头「牵连面」）：末了另起一行，只报数
    print(f"  · {impact_summary(context['rows'], context['components'], context['key_context'], environment, len(anchor_rows))}")
    return 0


def command_item(arguments):
    """--item <行号或变异名> [--item …] <根>：交给门禁 crates-mutation-replay.sh --item（同一份选行模式）。"""
    items, rest, index = [], [], 0
    while index < len(arguments):
        if arguments[index] == "--item":
            if index + 1 >= len(arguments) or not arguments[index + 1]:
                print("  ✗ --item 后面缺行号或变异名")
                print("     → 怎么办：写成 --item <行号或变异名>（可给几次）<根>；可点的用 crates-mutation-rows.py --list-items <根> 看")
                return 2
            items.append(arguments[index + 1])
            index += 2
        else:
            rest.append(arguments[index])
            index += 1
    if not items or len(rest) != 1:
        print("  ✗ 用法：crates-mutation-rows.py --item <行号或变异名> [--item …] <根>")
        print("     → 怎么办：至少点一行、最后给被判那棵树的仓根；可点的用 crates-mutation-rows.py --list-items <根> 看")
        return 2
    command = ["bash", STAGE_SCRIPT] + [word for item in items for word in ("--item", item)] + rest
    sys.stdout.flush()
    os.execvp("bash", command)
    return 2   # execvp 不返回


def command_sample_tools(arguments):
    """sample-tools <目录>：写假 cargo 与两个假 llvm 工具（文件头同名子命令）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py sample-tools <目录>")
        print("     → 怎么办：<目录> 是放假工具的目录（门禁样本档在自己的临时目录里建）")
        return 2
    try:
        selftest_write(os.path.join(arguments[0], "cargo"), FAKE_CARGO, executable=True)
        for tool in LLVM_TOOLS:
            selftest_write(os.path.join(arguments[0], tool), FAKE_LLVM_TOOL, executable=True)
    except OSError as error:
        print(f"  ✗ 写不了 {arguments[0]}（{error}）：样本档的假工具放不下")
        print("     → 怎么办：看 ${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。")
        return 1
    print(f"  · 假 cargo 与假 llvm 工具（{'、'.join(LLVM_TOOLS)}）写进了 {arguments[0]}")
    return 0


def command_llvm_tools(arguments):
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py llvm-tools <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（rustc 在那里起，认得它的 rust-toolchain 文件）")
        return 2
    _directory, text, problem = llvm_tools(arguments[0], dict(os.environ))
    if problem is not None:
        print(f"  ✗ {problem[0]}")
        print(f"     → 怎么办：{problem[1]}")
        return 1
    print(f"  ✓ {text}")
    return 0


def selftest_undefined_identifier_cell(selftest, work):
    """static-check 的 UNDEFINED（replacement_undefined_identifiers）：替换文引了被改文件与包里都没有的标识符（照第六十一节那样被删掉的）的那一行报一行；
    包里别的文件定义的、替换文自己绑定的（let、闭包参数、match 臂）、方法与路径里的段、宏、字符串与注释里的都不报。弄坏开关 undefinedidentifiersunchecked 下一行都不报，这一格判错。"""
    root = os.path.join(work, "undefined-identifier")
    rows = [
        "deleted-identifier\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ halved_before / 2 }\t-p tiny --lib -- halves\thalves",
        "defined-in-package\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ helper_elsewhere(value) * 2 }\t-p tiny --lib -- doubles\tdoubles",
        "bound-in-replacement\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ let flipped = 0 - value; let pick = |input: i32| input; "
        "match Some(flipped) { Some(inner) => pick(inner).max(i32::MIN), None => unreachable!() } }\t-p tiny --lib -- negates\tnegates",
        "only-in-text\tcrates/tiny/src/lib.rs\t{ left + right }\t{ let _ = \"ghost_in_string\"; /* ghost_in_comment */ left - right }\t-p tiny --lib -- adds\tadds",
    ]
    build_selftest_repository(root, rows)
    selftest_write(os.path.join(root, "crates", "tiny", "src", "other.rs"), "pub fn helper_elsewhere(value: u32) -> u32 { value }\n")
    exit_code, output = run_command_capturing(command_static_check, [root])
    undefined = [line for line in output.splitlines() if line.startswith("UNDEFINED\t")]
    selftest.check("static-check 的 UNDEFINED：替换文引了已删的标识符 halved_before 的那一行报一行、退 1；包里别的文件定义的、替换文自己绑定的、方法、路径、宏、字符串与注释里的都不报",
                   exit_code == 1 and undefined == ["UNDEFINED\t2\tdeleted-identifier\tcrates/tiny/src/lib.rs\thalved_before"]
                   and "IDENTIFIERS_CHECKED\t4\t0" in output, output)


def selftest_reachability_static_check(root, stops, environment_changes=None):
    """在样本仓上跑一趟 static-check 可达性：选路表、截断表、豁免表换成样本 crate 里的那几处（REACHABILITY_GPU_ROUTE_STOPS 换成 stops），跑完换回。
    environment_changes 临时并进 os.environ（多机配置换成 GPU 关那一份这类）。→ (退出码, stdout, {变异名: 哪一路}, {变异名: 豁免的种类})。"""
    saved = (GPU_ROUTE_UNITS[0], REACHABILITY_GPU_ROUTE_STOPS[0], REACHABILITY_CPU_FALLBACK_ROOTS[0])
    saved_environment = {name: os.environ.get(name) for name in (environment_changes or {})}
    try:
        GPU_ROUTE_UNITS[0] = (("tiny", SELFTEST_REACHABILITY_FILE, "judge_not_run_without_the_gpu_feature"),)
        REACHABILITY_GPU_ROUTE_STOPS[0] = stops
        REACHABILITY_CPU_FALLBACK_ROOTS[0] = ((SELFTEST_REACHABILITY_FILE, "hand_out_letting_the_gpu_refuse"),)
        os.environ.update(environment_changes or {})
        exit_code, output = run_command_capturing(command_static_check, [root])
    finally:
        GPU_ROUTE_UNITS[0], REACHABILITY_GPU_ROUTE_STOPS[0], REACHABILITY_CPU_FALLBACK_ROOTS[0] = saved
        for name, value in saved_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    routes = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "UNREACHABLE" and len(fields) > 3}
    exempted = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "REACHABILITY_EXEMPT" and len(fields) > 3}
    return exit_code, output, routes, exempted


def selftest_reachability_cells(selftest, work):
    """static-check 可达性（文件头「点名测试够不着被改坏的函数」）在样本 crate 上：多机配置 ENABLE_GPU=1（自证的那一份），截断表 =
    run_enumeration 连到 cpu_judge_entry 的那一条边（按边停）与 lax_base_recovery（按单元停），豁免表 = hand_out_letting_the_gpu_refuse。
    每格配一个弄坏开关（文件头弄坏开关那一节 reachability-*）。"""
    root = os.path.join(work, "reachability")
    build_selftest_repository(root, SELFTEST_REACHABILITY_ROWS, source=SELFTEST_REACHABILITY_SOURCE)
    stops = SELFTEST_REACHABILITY_STOPS
    exit_code, output, routes, exempted = selftest_reachability_static_check(root, stops)
    gpu_off = os.path.join(work, "reachability-gpu-off", "multi-host.env")
    selftest_write(gpu_off, "# 自证用的多机配置：GPU 关\nENABLE_GPU=0\n")
    off_code, off_output, off_routes, _off_exempted = selftest_reachability_static_check(root, stops, {"GATE_MUTATION_MULTI_HOST_CONFIG": gpu_off})
    selftest.check("static-check 可达性的 gpu-route：开 GPU 时点名测试走选路、被改坏的函数只在 CPU 判器入口（按边停）或宽松的恢复（按单元停）后面的两行报 gpu-route、退 1；"
                   "多机配置换成 GPU 关时这两行不报",
                   exit_code == 1 and routes.get("cpu-check-blind") == "gpu-route" and routes.get("recovery-three") == "gpu-route"
                   and "cpu-check-blind" not in off_routes and "recovery-three" not in off_routes and off_code in (0, 1),
                   f"开：退 {exit_code}、{routes}；关：退 {off_code}、{off_routes}；{output[-800:]}{off_output[-400:]}")
    selftest.check("static-check 可达性的 closure：被改坏的函数 never_called 不在点名测试 plain_sum 的闭包里（开不开 GPU 都调不到）的那一行报 closure",
                   routes.get("never-called-seven") == "closure", f"{routes}；{output[-600:]}")
    selftest.check("static-check 可达性的 cpu-configuration 豁免：测试自己写 ENABLE_GPU=0 配置的那一行不报 UNREACHABLE、打 REACHABILITY_EXEMPT cpu-configuration",
                   "cpu-check-on-the-cpu-configuration" not in routes and exempted.get("cpu-check-on-the-cpu-configuration") == "cpu-configuration",
                   f"{routes}；豁免 {exempted}")
    # 测试经库代码里的函数写 ENABLE_GPU=0（样本：非测试代码的 configuration_text_on_the_cpu，测试只调它）：弄坏开关 reachability-cpu-configuration-test-code-only 下这一行报 gpu-route
    library_root = os.path.join(work, "reachability-library-configuration")
    library_source = SELFTEST_REACHABILITY_SOURCE.replace(
        "#[cfg(test)]\nmod tests {\n",
        'pub fn configuration_text_on_the_cpu() -> &\'static str { "ENABLE_GPU=0\\n" }\n\n#[cfg(test)]\nmod tests {\n'
        "    #[test]\n    fn enumeration_on_the_cpu_through_the_library() { assert!(!super::configuration_text_on_the_cpu().is_empty()); assert!(super::run_enumeration(1)); }\n", 1)
    build_selftest_repository(library_root, ["cpu-check-on-the-cpu-through-the-library\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t"
                                             "-p tiny --lib -- enumeration_on_the_cpu_through_the_library\tenumeration_on_the_cpu_through_the_library"], source=library_source)
    _library_code, library_output, library_routes, library_exempted = selftest_reachability_static_check(library_root, stops)
    selftest.check("static-check 可达性的 cpu-configuration 豁免，库函数写配置：测试经库代码里的函数（非测试代码）写 ENABLE_GPU=0 的那一行不报 UNREACHABLE、"
                   "打 REACHABILITY_EXEMPT cpu-configuration，凭据点名那个库函数",
                   "cpu-check-on-the-cpu-through-the-library" not in library_routes
                   and library_exempted.get("cpu-check-on-the-cpu-through-the-library") == "cpu-configuration"
                   and any(line.startswith("REACHABILITY_EXEMPT\t") and "cpu-check-on-the-cpu-through-the-library" in line and "::configuration_text_on_the_cpu" in line
                           for line in library_output.splitlines()),
                   f"{library_routes}；豁免 {library_exempted}；{library_output[-600:]}")
    selftest.check("static-check 可达性放行够得着的：测试自己直接调 CPU 判器入口（不经选路那一条边）的那一行、不经选路的 plain-four 都不报",
                   "cpu-check-observed" not in routes and "plain-four" not in routes and "cpu-check-observed" not in exempted, f"{routes}；豁免 {exempted}")
    selftest.check("static-check 可达性的 gpu-tables-refused 豁免：截过的闭包里有 GPU 许建不出判表、回到 CPU 判器的那个函数的那一行不报、打 REACHABILITY_EXEMPT gpu-tables-refused",
                   "cpu-check-refused-tables" not in routes and exempted.get("cpu-check-refused-tables") == "gpu-tables-refused", f"{routes}；豁免 {exempted}")
    selftest.check("static-check 可达性放行读源码文本的：点名测试 include_str! 读被改坏的那份文件（闭包里有它的 data-file 单元）、不调被改坏的函数 never_called 的那一行不报",
                   "never-called-read-as-text" not in routes and "never-called-read-as-text" not in exempted, f"{routes}；豁免 {exempted}；{output[-600:]}")
    checked = next((line.split("\t") for line in output.splitlines() if line.startswith("REACHABILITY_CHECKED\t")), [])
    selftest.check("static-check 可达性报数与出路：REACHABILITY_CHECKED 判了 8 行、没判 0 行、截过闭包 5 行；有 UNREACHABLE 就跟一行「→ 怎么办」写换用例或删行",
                   checked[1:3] == ["8", "0"] and checked[4:5] == ["5"] and f"→ 怎么办：{REACHABILITY_HOWTO}" in output,
                   f"{checked}；{output[-600:]}")
    missing_stops = stops + ((None, (SELFTEST_REACHABILITY_FILE, "judge_that_is_not_in_the_source")),)
    missing_code, missing_output, _routes, _exempted = selftest_reachability_static_check(root, missing_stops)
    selftest.check("static-check 可达性起跑先核截断表：表里的函数在源码里找不到打 ERROR 点名它、退 2",
                   missing_code == 2 and any(line.startswith("ERROR\t") and "judge_that_is_not_in_the_source" in line for line in missing_output.splitlines()),
                   f"退 {missing_code}；{missing_output[-400:]}")
    small_root = os.path.join(work, "reachability-small")
    build_selftest_repository(small_root, SELFTEST_TABLE_ROWS)
    small_code, small_output = run_command_capturing(command_static_check, [small_root])
    selftest.check("static-check 可达性：选路表、截断表、豁免表点名的函数在这棵树里一个都没有（门禁样本那种小根）时打 REACHABILITY_NOT_JUDGED 点名原因、不打 REACHABILITY_CHECKED、不报 ERROR，退出码不受它影响",
                   small_code == 0 and any(line.startswith("REACHABILITY_NOT_JUDGED\t") for line in small_output.splitlines())
                   and "REACHABILITY_CHECKED" not in small_output and "ERROR\t" not in small_output,
                   f"退 {small_code}；{small_output[-500:]}")
    unknown_code, unknown_output = run_command_capturing(command_static_check, [small_root, "--option-never-used-before"])
    selftest.check("static-check 认不出的选项：打 ERROR 与「→ 怎么办」、退 2，一行都不判",
                   unknown_code == 2 and "ERROR\t" in unknown_output and "→ 怎么办" in unknown_output and "CHECKED" not in unknown_output,
                   f"退 {unknown_code}；{unknown_output[-400:]}")


def selftest_reachability_refused_cells(selftest, work):
    """static-check 可达性遇到点名测试的闭包拒算（文件头「点名测试够不着被改坏的函数」闭包拒算那一条）：样本里 refused_* 三条测试调了读 CARGO_MANIFEST_DIR 的函数
    （闭包算不出键），whole_* 两行点名的测试闭包不拒算作对照。截断表、多机配置与 selftest_reachability_cells 同一份。
    弄坏开关 reachability-refused-unjudged（拒算的一律报没判）下头一格必须判错。"""
    root = os.path.join(work, "reachability-refused")
    source = SELFTEST_REACHABILITY_SOURCE.replace("mod tests {\n", "mod tests {\n" + SELFTEST_REACHABILITY_REFUSED_TESTS, 1)
    build_selftest_repository(root, SELFTEST_REACHABILITY_REFUSED_ROWS, source=source)
    exit_code, output, routes, exempted = selftest_reachability_static_check(root, SELFTEST_REACHABILITY_STOPS)
    unjudged = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "REACHABILITY_UNJUDGED" and len(fields) > 3}
    selftest.check("static-check 可达性的闭包拒算：点名测试的闭包拒算（调了读 CARGO_MANIFEST_DIR 的函数）、被改坏的函数 never_called 落在已收部分里的那一行不报够不着、不报没判",
                   "refused-reaches-never-called" not in routes and "refused-reaches-never-called" not in unjudged and "refused-reaches-never-called" not in exempted,
                   f"{routes}；没判 {unjudged}；{output[-800:]}")
    selftest.check("static-check 可达性的闭包拒算：被改坏的函数不在已收部分里的那一行不报 UNREACHABLE closure、报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、已收部分里没有被改坏的单元」与拒算原因 runtime-path",
                   "refused-misses-never-called" not in routes and "闭包拒算、已收部分里没有被改坏的单元" in unjudged.get("refused-misses-never-called", "")
                   and "runtime-path" in unjudged.get("refused-misses-never-called", ""), f"{routes}；没判 {unjudged}；{output[-800:]}")
    selftest.check("static-check 可达性的闭包拒算：开 GPU、被改坏的 CPU 判器只在截过的闭包之外的那一行不报 gpu-route、报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、截过的闭包已收部分里没有被改坏的单元」",
                   "refused-behind-the-cpu-judge" not in routes
                   and "闭包拒算、截过的闭包已收部分里没有被改坏的单元" in unjudged.get("refused-behind-the-cpu-judge", ""), f"{routes}；没判 {unjudged}；{output[-800:]}")
    checked = next((line.split("\t") for line in output.splitlines() if line.startswith("REACHABILITY_CHECKED\t")), [])
    selftest.check("static-check 可达性的闭包拒算，不拒算的行判法不变：点名测试闭包不拒算的两行照报（never_called 不在 plain_sum 的闭包里报 closure、CPU 判器只在 CPU 那一臂后面报 gpu-route），"
                   "退 1，REACHABILITY_CHECKED 判了 3 行、没判 2 行",
                   exit_code == 1 and routes.get("whole-misses-never-called") == "closure" and routes.get("whole-behind-the-cpu-judge") == "gpu-route"
                   and checked[1:3] == ["3", "2"], f"退 {exit_code}；{routes}；{checked}；{output[-800:]}")


# row_view（算键）与 reachability_row_answer（可达性）各在自己那一份模块里按名字被调：计数的包装要放进每一份有这个名字的模块
WRAPPED_FUNCTION_MODULES = (crates_mutation_rows_units, crates_mutation_rows_plan)


def selftest_static_check_parallel_cell(selftest, work):
    """static-check 切片并行（文件头 static-check「并行」）对拍：同一份样本仓上 GATE_MUTATION_KEY_WORKERS=1（逐份逐行算；UNDEFINED 那一段就是逐行调 replacement_undefined_identifiers）
    与 =3（切片并行：逃出 crates/ 的字面量按 .rs 份，UNDEFINED 先按被改文件与包切词、再按行细判，点名测试的闭包经 precompute_closures_from_shared_parent）的输出逐行相同；
    ANCHOR、UNDEFINED、IDENTIFIERS_CHECKED 几行另与逐行调 anchor_problem、replacement_undefined_identifiers 拼出的逐行相同（逐行调的那一路）。
    样本：可达性那份样本 crate 加 8 份 extra_<n>.rs（各一个函数、一条点名测试、一处逃出 crates/ 的字面量），8 行替换文引了没定义的 ghost_<n>，另有锚点命中 0 次、文件不在的各一行；
    要并行的几段份数都不少于 PARALLEL_MINIMUM（9 份 .rs、10 份标识符集合、8 行细判、14 份点名测试的闭包、可达性逐行那一段 16 行），格里核 UNDEFINED 与 LITERAL 的行数，不是空跑；
    另核可达性逐行那一段（reachability_resolved_rows）并行那一趟真切了片、单线程那一趟没切，单线程那一趟里每份被改坏的 .rs 只打开一次（row_view 与 reachability_row_answer 里数）；
    建工作区（prebuilt_workspace）并行那一趟取了子进程按文件切好的 .rs、单线程那一趟没切，按文件切片建的工作区与逐份切的两个视图逐项相同。
    弄坏开关 static-check-slices-misordered（并回时片的次序倒过来）下头一格判错，reachability-rows-unsliced（逐行那一段不切片）下「按行切片并行」那一格判错，
    workspace-files-unsliced（建工作区不按文件切片）与 rust-file-replaced-on-entry-module（子进程切好的文件的替身换在入口模块上、RustWorkspace 读不到）下「建工作区按文件切片并行：」那一格判错，
    mutated-source-reread（lib_mutation_units.py：被改坏的文件不记、每次现读）下「只读一遍」那一格判错。"""
    import builtins
    root = os.path.join(work, "static-check-parallel")
    selftest_static_check_parallel_repository(root)
    # 单线程那一趟另数可达性逐行那一段（row_view 与 reachability_row_answer 里）打开每份 .rs 的次数：每份被改坏的文件只读一遍（UnitKeys.source_bytes）
    opened, real_open = collections.Counter(), builtins.open
    real_functions = {"row_view": crates_mutation_rows_units.row_view, "reachability_row_answer": crates_mutation_rows_plan.reachability_row_answer}

    def counting_open(file, mode="r", *rest, **options):
        if isinstance(file, (str, os.PathLike)) and os.fspath(file).endswith(".rs"):
            opened[os.path.relpath(os.path.abspath(os.fspath(file)), root)] += 1
        return real_open(file, mode, *rest, **options)

    def counted(name):
        def wrapper(*arguments):
            builtins.open = counting_open
            try:
                return real_functions[name](*arguments)
            finally:
                builtins.open = real_open
        return wrapper
    for part in WRAPPED_FUNCTION_MODULES:
        vars(part).update({name: counted(name) for name in real_functions if name in vars(part)})
    try:
        single_code, single_output, _routes, _exempted = selftest_reachability_static_check(root, SELFTEST_REACHABILITY_STOPS, {"GATE_MUTATION_KEY_WORKERS": "1"})
    finally:
        for part in WRAPPED_FUNCTION_MODULES:
            vars(part).update({name: function for name, function in real_functions.items() if name in vars(part)})
    single_forked = getattr(reachability_resolved_rows, "forked_rows", None)
    single_prebuilt = dict(getattr(prebuilt_workspace, "files", None) or {})
    parallel_code, parallel_output, _routes, _exempted = selftest_reachability_static_check(root, SELFTEST_REACHABILITY_STOPS, {"GATE_MUTATION_KEY_WORKERS": "3"})
    parallel_forked = getattr(reachability_resolved_rows, "forked_rows", None)
    parallel_prebuilt = dict(getattr(prebuilt_workspace, "files", None) or {})
    selftest_static_check_parallel_compare(selftest, root, single_code, single_output, parallel_code, parallel_output)
    lfu = mutation_units.function_units()
    selftest.check(f"static-check 建工作区按文件切片并行：并行那一趟本进程取了子进程切好的 .rs 不少于 {mutation_units.PARALLEL_MINIMUM} 份，单线程那一趟一份都没切",
                   not single_prebuilt.get(lfu.PRODUCT_BEHAVIOR) and (parallel_prebuilt.get(lfu.PRODUCT_BEHAVIOR) or 0) >= mutation_units.PARALLEL_MINIMUM,
                   f"单线程那一趟 {single_prebuilt}、并行那一趟 {parallel_prebuilt}（{{视图: 取了几份}}）")

    def workspace_view(workspace):
        """对拍用的工作区摘要：每份文件的单元（text() 那一行、限定名、只给 Verus 看）与拒算，各目标收的文件与模块路径。"""
        files = {path: ([(unit.text(), unit.full, unit.ghost_only) for unit in rust_file.units], sorted(tuple(refusal) for refusal in rust_file.refusals))
                 for path, rust_file in workspace.files.items()}
        targets = {target_id: sorted((path, tuple(place[0])) for path, place in target.files.items()) for target_id, target in workspace.targets.items()}
        return files, targets
    differing = []
    for view in (lfu.PRODUCT_BEHAVIOR, lfu.FORMAL_VERIFICATION):
        serial_keys, sliced_keys = mutation_units.UnitKeys(root), mutation_units.UnitKeys(root)
        prebuilt_workspace(sliced_keys, view, 3, os.environ)
        if view == lfu.FORMAL_VERIFICATION:
            serial_view, sliced_view = workspace_view(serial_keys.formal_workspace), workspace_view(sliced_keys.formal_workspace)
        else:
            serial_view, sliced_view = workspace_view(serial_keys.workspace), workspace_view(sliced_keys.workspace)
        if serial_view != sliced_view or not serial_view[0]:
            differing.append(f"{view}：逐份切 {len(serial_view[0])} 份、按文件切片 {len(sliced_view[0])} 份，"
                             f"不同的文件 {sorted(path for path in set(serial_view[0]) | set(sliced_view[0]) if serial_view[0].get(path) != sliced_view[0].get(path))[:5]}")
    selftest.check("static-check 建工作区按文件切片并行与逐份切的工作区逐项相同（产品行为与形式化两个视图：每份文件的单元、限定名、拒算，各目标收的文件与模块路径）",
                   not differing, "；".join(differing))
    selftest.check(f"static-check 可达性逐行那一段（被改坏的单元与点名测试）按行切片并行：并行那一趟切了不少于 {mutation_units.PARALLEL_MINIMUM} 行，单线程那一趟一行都没切",
                   single_forked == 0 and isinstance(parallel_forked, int) and parallel_forked >= mutation_units.PARALLEL_MINIMUM,
                   f"单线程切了 {single_forked} 行、并行切了 {parallel_forked} 行")
    reread = {path: count for path, count in opened.items() if count != 1}
    selftest.check("static-check 可达性那一段每份被改坏的文件只读一遍（row_view 与 mutated_units 共用单元键记着的那一份）",
                   bool(opened) and not reread, f"读了几遍 {dict(opened)}；不是一遍的 {reread}")


def selftest_closure_refused_cell(selftest, work):
    """点名的测试在这一组特性下不编（#[cfg(feature = "gpu")] 的测试、cargo 参数没开 gpu）：闭包算不成时真错照原话报出、点名它的变异表的行、带出路，
    不被进程池换成 PicklingError。10 个闭包要算，GATE_MUTATION_KEY_WORKERS=2 走并行（子进程里抛的），=1 走就地；run_stage 与 --list-items 各核。
    弄坏开关 closure-error-masked（模块不登记进 sys.modules、子进程原样抛）下并行那一趟必须判错。"""
    tests = "".join(f"    #[test]\n    fn adds{index}() {{}}\n" for index in range(9))
    source = ("pub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n" + tests
              + '    #[cfg(feature = "gpu")]\n    #[test]\n    fn gpu_only() {}\n}\n')
    rows = [f"plain-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * right }}\t-p tiny --lib -- adds{index}\tadds{index}" for index in range(9)]
    rows.append("gated-without-feature\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- gpu_only\tgpu_only")
    repository = os.path.join(work, "closure-refused")
    build_selftest_repository(repository, rows, source=source)
    for key_workers, how in (("2", "并行算（子进程里抛）"), ("1", "就地算")):
        try:
            exit_code, stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_KEY_WORKERS=key_workers)
        except Exception as error:   # 弄坏开关下进程池把真错换成 MaybeEncodingError（PicklingError）抛出来
            exit_code, stdout = None, f"{type(error).__name__}：{str(error)[:300]}"
        selftest.check(f"闭包算不成（{how}）：run_stage 打真错（入口在这一组 cfg 下不编）、点名它的变异表第 11 行、带出路，退 1，没有 PicklingError",
                       exit_code == 1 and "UsageError：入口 crates/tiny/src/lib.rs::tests::gpu_only 在这一组 cfg 下不编" in stdout
                       and "点名它的：crates/mutations.tsv:11 gated-without-feature" in stdout and "给那几行的 cargo 参数补上编它要的特性" in stdout
                       and "PicklingError" not in stdout, f"退 {exit_code}；{stdout[-600:]}")
    saved_handler = signal.getsignal(signal.SIGPIPE)
    saved_workers = os.environ.get("GATE_MUTATION_KEY_WORKERS")
    os.environ["GATE_MUTATION_KEY_WORKERS"] = "2"
    listing = io.StringIO()
    try:
        with contextlib.redirect_stdout(listing):
            list_exit = command_list_items([repository])
    except Exception as error:
        list_exit = None
        listing.write(f"{type(error).__name__}：{str(error)[:300]}")
    finally:
        signal.signal(signal.SIGPIPE, saved_handler)
        if saved_workers is None:
            os.environ.pop("GATE_MUTATION_KEY_WORKERS", None)
        else:
            os.environ["GATE_MUTATION_KEY_WORKERS"] = saved_workers
    text = listing.getvalue()
    selftest.check("--list-items 遇到闭包算不成：变异名照列，末尾一行写键没算出来与真错（入口在这一组 cfg 下不编、点名它的行），没有回溯",
                   list_exit == 0 and "gated-without-feature\t第 11 行" in text and "键对不对得上没算出来" in text
                   and "在这一组 cfg 下不编" in text and "点名它的：crates/mutations.tsv:11 gated-without-feature" in text and "PicklingError" not in text,
                   f"退 {list_exit}；{text[-600:]}")


def selftest_release_static_check_cell(selftest, work):
    """static-check 的 RELEASE（文件头「重测试目标只许 release」）：一张表四行——带 gpu 特性没写 --release、点名登记的目标没写、命中而写了 --release、两样都不中：
    前两行各报一行 RELEASE（带依据），跟一行「→ 怎么办」写「第五段补上 --release」，退 1；另一张表把前两行写上 --release，一行都不报、退 0。
    登记表（崩溃枚举用例 tiny-stream）放在样本树里，static-check 读被判那棵树的。弄坏开关 release-unchecked（不判）下这一格判错，
    static-check-registry-from-script-repository（读本模块所在那一份仓的登记表，样本树自己登记的那条崩溃枚举用例看不见）下同样判错。"""
    rows = [SELFTEST_GPU_FEATURE_ROW, SELFTEST_RELEASE_CRASH_ROW, SELFTEST_RELEASE_WRITTEN_ROW, SELFTEST_TABLE_ROWS[0]]
    repository = selftest_release_repository(work, "release-static", rows)
    written = [row.replace("--lib -- doubles", "--lib --release -- doubles").replace("--test tiny_crash_stream --", "--test tiny_crash_stream --release --") for row in rows]
    written_repository = selftest_release_repository(work, "release-static-written", written)
    exit_code, output = run_command_capturing(command_static_check, [repository])
    written_code, written_output = run_command_capturing(command_static_check, [written_repository])
    release_lines = [line for line in output.splitlines() if line.startswith("RELEASE\t")]
    selftest.check("static-check 的 RELEASE：命中而第五段没写 --release 的两行（gpu 特性、crash-case 目标）各报一行、带依据、跟一行写出路，退 1；命中写了的与不命中的不报，两行都写上之后退 0",
                   exit_code == 1 and len(release_lines) == 2
                   and release_lines[0] == "RELEASE\t2\tgpu-double-becomes-triple\tgpu 特性 gpu"
                   and release_lines[1].startswith("RELEASE\t3\tstream-seven-becomes-eight\tcrash-case 目标 tiny:tiny_crash_stream（--test 点名；登记 crash-case:tiny-stream）")
                   and "     → 怎么办：第五段补上 --release" in output
                   and written_code == 0 and "RELEASE\t" not in written_output,
                   f"退 {exit_code}：{output}；都写上之后退 {written_code}：{written_output}")


def selftest_unfiltered_static_check_cell(selftest, work):
    """static-check 的「第五段 `--` 之后没有测试名筛选词」（unfiltered_rows、unfiltered_rows_judged）：三行表——带筛选词的、第五段没有 `--` 的（第 3 行）、带筛选词的：
    样本根放了 .mutation-filter-words-judged 的那一份报一行 MALFORMED（第 3 行，写没有测试名筛选词）、退 1；同一张表、没放标记的样本根不判、不报这一句、退 0。
    弄坏开关 unfiltered-rows-unchecked（一律不判）下这一格判错。"""
    rows = [SELFTEST_TABLE_ROWS[0], SELFTEST_TABLE_ROWS[1].replace("--lib -- doubles", "--lib"), SELFTEST_TABLE_ROWS[2]]
    repository = selftest_release_repository(work, "unfiltered-static", rows)
    selftest_write(os.path.join(repository, ".mutation-filter-words-judged"), "这份样本根按第五段筛选词判\n")
    bare_repository = selftest_release_repository(work, "unfiltered-static-bare", rows)
    saved_staged = os.environ.pop("GATE_STAGED_FROM", None)
    try:
        exit_code, output = run_command_capturing(command_static_check, [repository])
        bare_code, bare_output = run_command_capturing(command_static_check, [bare_repository])
    finally:
        if saved_staged is not None:
            os.environ["GATE_STAGED_FROM"] = saved_staged
    lines = [line for line in output.splitlines() if line.startswith("MALFORMED\t")]
    selftest.check("static-check 判「第五段 `--` 之后没有测试名筛选词」：放了标记的样本根里第 3 行（第五段没有 --）报一行 MALFORMED、写没有测试名筛选词，退 1；同一张表没放标记的样本根不判、退 0",
                   exit_code == 1 and len(lines) == 1 and lines[0].startswith("MALFORMED\t3\tdouble-becomes-triple\t第五段 `--` 之后没有测试名筛选词")
                   and bare_code == 0 and "没有测试名筛选词" not in bare_output, f"退 {exit_code}：{output}；没放标记的退 {bare_code}：{bare_output}")


def selftest_release_judged_tree_registry_cell(selftest, work):
    """static-check 的登记表读被判的那棵树（文件头「拷贝范围」「重测试目标只许 release」）：样本树自己的登记表（.claude/gate.d/stage-inputs.tsv）登记了
    本模块所在那一份仓的登记表里没有的一条崩溃枚举用例（tiny 的 tiny_crash_stream）与一处多出来的拷贝范围（docs/outside.rs），少登记了本模块所在那一份仓的登记表里有的 litmus/；
    四行表，static-check 在样本树上：
      点名样本树登记的目标、第五段没写 --release：报一行 RELEASE（带依据）；
      点名本模块所在那一份仓的登记表里有、样本树的登记表里没有的崩溃枚举用例目标、第五段没写 --release：不报；
      改 docs/outside.rs（样本树的拷贝范围里有、本模块所在那一份仓的没有）：不报 ROW；
      改 litmus/outside.rs（本模块所在那一份仓的拷贝范围里有、样本树的没有）：报 ROW。
    另一棵样本树里没有登记表：打 ERROR 点名那张表与出路、退 2，不回落到本模块所在那一份仓的登记表、不打 CHECKED。
    弄坏开关 static-check-registry-from-script-repository（读本模块所在那一份仓的登记表）下这一格判错。"""
    label = "static-check 读被判那棵树的登记表"
    script_targets = release_only_tests_module().crash_case_targets(STAGE_REPOSITORY)
    if not script_targets:
        selftest.skip(label, f"{STAGE_REPOSITORY} 的登记表里没有崩溃枚举用例，没有「本模块所在那一份仓有、样本树没有」的目标可点名")
        return
    script_target = script_targets[0]
    rows = [SELFTEST_RELEASE_CRASH_ROW.replace("stream-seven-becomes-eight", "tree-registered-target"),
            f"script-registered-target\tcrates/tiny/src/lib.rs\t{{ 3 + 4 }}\t{{ 3 + 6 }}\t-p {script_target.package} --test {script_target.target} -- streams_agree\tstreams_agree",
            "tree-registered-file\tdocs/outside.rs\tpub fn outside() {}\tpub fn outside() { }\t-p tiny --lib -- adds\tadds",
            "script-registered-file\tlitmus/outside.rs\tpub fn outside() {}\tpub fn outside() { }\t-p tiny --lib -- adds\tadds"]
    repository = os.path.join(work, "release-judged-tree-registry")
    build_selftest_repository(repository, rows, extra_source=SELFTEST_RELEASE_SOURCE)
    for outside_directory in ("docs", "litmus"):
        selftest_write(os.path.join(repository, outside_directory, "outside.rs"), "pub fn outside() {}\n")
    selftest_write_stage_registration_table(repository, SELFTEST_RELEASE_REGISTRY, copy_ranges_added=("docs/outside.rs",), copy_ranges_dropped=("litmus/",))
    exit_code, output = run_command_capturing(command_static_check, [repository])
    release_lines = [line for line in output.splitlines() if line.startswith("RELEASE\t")]
    row_lines = [line for line in output.splitlines() if line.startswith("ROW\t")]
    selftest.check(f"{label}：样本树登记的崩溃枚举用例目标那一行报一行 RELEASE，只有本模块所在那一份仓登记的目标那一行不报；样本树多登记的拷贝范围里的文件不报 ROW，"
                   "只有本模块所在那一份仓登记的拷贝范围里的文件报 ROW",
                   exit_code == 1 and "ERROR\t" not in output and script_target.package != "tiny" and len(release_lines) == 1
                   and release_lines[0].startswith("RELEASE\t2\ttree-registered-target\tcrash-case 目标 tiny:tiny_crash_stream（--test 点名；登记 crash-case:tiny-stream）")
                   and row_lines == ["ROW\t5\tscript-registered-file\tlitmus/outside.rs"],
                   f"退 {exit_code}；RELEASE {release_lines}；ROW {row_lines}；本模块所在那一份仓的目标 {script_target.package}:{script_target.target}；{output[-600:]}")
    bare_repository = os.path.join(work, "release-judged-tree-bare")
    build_selftest_repository(bare_repository, rows[:1], extra_source=SELFTEST_RELEASE_SOURCE)
    registry_table = os.path.join(bare_repository, admission_module().REGISTRATION_TABLE)
    os.remove(registry_table)
    bare_code, bare_output = run_command_capturing(command_static_check, [bare_repository])
    selftest.check(f"{label}：样本树里没有登记表，打 ERROR 点名那张表、给出路、退 2，不回落到本模块所在那一份仓的登记表、不打 CHECKED",
                   bare_code == 2 and f"ERROR\t被判的树里没有登记表 {registry_table}" in bare_output and f"     → 怎么办：{STATIC_CHECK_REGISTRY_HOWTO}" in bare_output
                   and "CHECKED" not in bare_output, f"退 {bare_code}；{bare_output[-600:]}")


def selftest_path_dependency_copy_cells(selftest, work):
    """拷贝范围跟 path 依赖（文件头「拷贝范围」）：tiny 经 path 依赖仓根下拷贝范围之外的 sidecar/（sidecar 再经 path 依赖 sidecar-inner/），
    static-check 的 COPIED 里有这两处（run_stage 的 prechecks 调同一个 copied_with_path_dependencies）；path 指到树之外的，static-check 打 ERROR、退 2。
    弄坏开关 path-dependencies-not-copied 下两格判错。"""
    root = os.path.join(work, "path-dependency-copy")
    build_selftest_repository(root, SELFTEST_TABLE_ROWS)
    with open(os.path.join(root, "crates", "tiny", "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write('\n[dependencies]\nsidecar = { path = "../../sidecar" }\n')
    selftest_write(os.path.join(root, "sidecar", "Cargo.toml"), '[package]\nname = "sidecar"\nversion = "0.1.0"\n\n[dependencies]\nsidecar-inner = { path = "../sidecar-inner" }\n')
    selftest_write(os.path.join(root, "sidecar", "src", "lib.rs"), "")
    selftest_write(os.path.join(root, "sidecar-inner", "Cargo.toml"), '[package]\nname = "sidecar-inner"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(root, "sidecar-inner", "src", "lib.rs"), "")
    exit_code, output = run_command_capturing(command_static_check, [root])
    copied_line = next((line.split("\t", 1)[1].split(" ") for line in output.splitlines() if line.startswith("COPIED\t")), [])
    selftest.check("拷贝范围跟 path 依赖：tiny 依赖的 sidecar/ 与 sidecar 再依赖的 sidecar-inner/ 都进 static-check 的 COPIED",
                   exit_code in (0, 1) and "sidecar" in copied_line and "sidecar-inner" in copied_line, f"退 {exit_code}，COPIED {copied_line}；{output[-400:]}")
    outside = os.path.join(work, "path-dependency-outside", "tree")
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS)
    with open(os.path.join(outside, "crates", "tiny", "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write('\n[dependencies]\nstray = { path = "../../../stray" }\n')
    selftest_write(os.path.join(work, "path-dependency-outside", "stray", "Cargo.toml"), '[package]\nname = "stray"\nversion = "0.1.0"\n')
    exit_code, output = run_command_capturing(command_static_check, [outside])
    selftest.check("拷贝范围跟 path 依赖指到树之外：static-check 打 ERROR（拷贝范围跟不全 path 依赖）、退 2，出路连副本拷不全那一种指到 minimal-copy.py",
                   exit_code == 2 and "ERROR\t拷贝范围跟不全 path 依赖" in output and "minimal-copy.py" in output, f"退 {exit_code}；{output[-400:]}")


def selftest_manifest_path_cells(selftest, work):
    """变异行带 --manifest-path（文件头「--manifest-path」）：样本仓根下放一份工作区之外的小工程 standalone-project/（自己是工作区根，仓根工作区不列它、
    也没有谁经 path 依赖它、不在登记的拷贝范围里），变异表是样本表头一行（-p tiny）加两行各点它的一条测试（--manifest-path <路径> 与 --manifest-path=<路径> 两种写法）。
    假 cargo 只读 crates/tiny/src/lib.rs 里的「// TEST」行：那两条照 FILES 读副本里 standalone-project/src/lib.rs 的原文，改坏了就红。
      静态核：static-check 退 0、COPIED 里有 standalone-project、没有 ROW 行；
      基线与判红：头一趟退 0、三行都现判抓到、拒算 0 条，基线与变异的 cargo 照原样带两种写法的 --manifest-path 起；
      单元键照那个包的源码算：第二趟输入不变、三行都复用、cargo 一次都不起；改 standalone-project 里 scale 的函数体另一处，下一趟只现判点 scales 的那一行；
      不在仓内：另一棵样本仓里一行 --manifest-path 指到仓根之外的一份 Cargo.toml，static-check 打 ERROR、退 2，run_stage 退 1、一条都不跑。
    弄坏开关 manifest-path-ignored 下四格判错（拷贝范围不补，static-check 报 ROW、run_stage 判红；指到仓外的那一行不拒）。"""
    standalone_source = ("// 工作区之外的小工程（自证「--manifest-path」那几格）\n"
                         "pub fn scale(value: u32) -> u32 {\n    let margin = 0;\n    value * 5 + margin\n}\n"
                         "pub fn shift(value: u32) -> u32 { value + 11 }\n\n"
                         "#[cfg(test)]\nmod tests {\n    #[test]\n    fn scales() { assert_eq!(super::scale(2), 10); }\n"
                         "    #[test]\n    fn shifts() { assert_eq!(super::shift(1), 12); }\n}\n")
    tiny_tests = ("// TEST scales FILES ../../standalone-project/src/lib.rs value * 5 + margin\n"
                  "// TEST shifts FILES ../../standalone-project/src/lib.rs { value + 11 }\n")
    manifest_rows = [
        "standalone-scale-becomes-six\tstandalone-project/src/lib.rs\tvalue * 5 + margin\tvalue * 6 + margin\t"
        "--manifest-path standalone-project/Cargo.toml --lib -- scales\tscales",
        "standalone-shift-becomes-twelve\tstandalone-project/src/lib.rs\t{ value + 11 }\t{ value + 12 }\t"
        "--manifest-path=standalone-project/Cargo.toml --lib -- shifts\tshifts",
    ]
    repository = os.path.join(work, "manifest-path")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS[:1] + manifest_rows, extra_source=tiny_tests)
    selftest_write(os.path.join(repository, "standalone-project", "Cargo.toml"), '[package]\nname = "standalone"\nversion = "0.1.0"\n\n[workspace]\n')
    selftest_write(os.path.join(repository, "standalone-project", "src", "lib.rs"), standalone_source)
    exit_code, output = run_command_capturing(command_static_check, [repository])
    copied_line = next((line.split("\t", 1)[1].split(" ") for line in output.splitlines() if line.startswith("COPIED\t")), [])
    row_lines = [line for line in output.splitlines() if line.startswith("ROW\t")]
    selftest.check("--manifest-path 静态核：static-check 退 0，拷贝范围照 --manifest-path 补进工作区之外的 standalone-project、没有 ROW 行",
                   exit_code == 0 and "standalone-project" in copied_line and not row_lines, f"退 {exit_code}，COPIED {copied_line}，ROW {row_lines}；{output[-400:]}")
    first = selftest.run(repository)
    first_how = per_row_how(first[1])
    plain = plain_invocations(first[3])
    selftest.check("--manifest-path 基线与判红：头一趟退 0，-p tiny 那一行与两行 --manifest-path 都现判抓到、拒算 0 条，基线与变异的 cargo 照原样带两种写法的 --manifest-path 起",
                   first[0] == 0 and "现判 3 条、复用 0 条、拒算 0 条" in first[1]
                   and all(first_how.get(name, "").startswith("现判") for name in ("add-becomes-sub", "standalone-scale-becomes-six", "standalone-shift-becomes-twelve"))
                   and any("--manifest-path standalone-project/Cargo.toml" in line for line in plain)
                   and any("--manifest-path=standalone-project/Cargo.toml" in line for line in plain),
                   f"退 {first[0]}；{first_how}；cargo {plain}；{first[1][-500:]}{first[2][-300:]}")
    second = selftest.run(repository)
    replace_in_file(os.path.join(repository, "standalone-project", "src", "lib.rs"), "let margin = 0;", "let margin = 0 * value;")
    third = selftest.run(repository)
    third_how = per_row_how(third[1])
    selftest.check("--manifest-path 单元键照那个包的源码算：第二趟三行都复用、cargo 一次都不起；改 scale 的函数体另一处之后只现判点 scales 的那一行，别的两行复用",
                   second[0] == 0 and second[3] == [] and "现判 0 条、复用 3 条、拒算 0 条" in second[1]
                   and third[0] == 0 and "现判 1 条、复用 2 条、拒算 0 条" in third[1] and third_how.get("standalone-scale-becomes-six", "").startswith("现判")
                   and third_how.get("standalone-shift-becomes-twelve") == "复用" and third_how.get("add-becomes-sub") == "复用",
                   f"第二趟退 {second[0]}、cargo {second[3]}；第三趟退 {third[0]}、{third_how}；{second[1][-300:]}{third[1][-300:]}")
    outside = os.path.join(work, "manifest-path-outside", "tree")
    stray_row = "stray-manifest\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 4 }\t--manifest-path ../stray-project/Cargo.toml --lib -- doubles\tdoubles"
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS[:1] + [stray_row])
    selftest_write(os.path.join(work, "manifest-path-outside", "stray-project", "Cargo.toml"), '[package]\nname = "stray"\nversion = "0.1.0"\n\n[workspace]\n')
    exit_code, output = run_command_capturing(command_static_check, [outside])
    refused = selftest.run(outside)
    selftest.check("--manifest-path 不在仓内：指到仓根之外的一份 Cargo.toml，static-check 打 ERROR、点名那一行、退 2，run_stage 退 1、一条都不跑",
                   exit_code == 2 and "ERROR\t拷贝范围补不上变异行 --manifest-path 指的包目录" in output and f"{TABLE}:3 stray-manifest" in output
                   and "不在仓内" in output and refused[0] == 1 and refused[3] == [] and "拷贝范围补不上变异行 --manifest-path 指的包目录" in refused[1],
                   f"static-check 退 {exit_code}；{output[-300:]}；run_stage 退 {refused[0]}、cargo {refused[3]}；{refused[1][-300:]}")


def selftest_crash_domain_narrowing_cells(selftest, work):
    """缩过域的一趟（文件头「缩过域的一趟」），同一份小仓（SELFTEST_TABLE_ROWS 四行）上依次：
    ① 设了 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3：四行都现判、都抓到、退 0，每一行抓到那一句与成功句写缩过域（变量=值）、不算门禁通过、没入库，覆盖一份都不记；
       同一份输入不设变量再跑一趟：四行仍现判（缩域那一趟一条都没入库）、不复用。弄坏开关 narrowed-verdicts-recorded 下第二趟四行都复用，这一格判错；
    ② 库里有了不缩域那一趟的抓到，设 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5、复用哨兵取 3 条强制现判 1 条（selftest_one_forced）：四行都复用、假 cargo 基线与变异一次都没起，
       stderr 写复用哨兵不抽与缩过域的原因，成功句写缩过域。弄坏开关 narrowed-sentinel-drawn 下强制现判一条，这一格判错；
    ③ 缩过域不分片：设了缩域变量的 row-plan 退 2、红句写不分片、一行都不列；run_stage 在 TASK_SERVER_SHARD_PHASE=shard 下退 2、假 cargo 一次都没起。
       弄坏开关 narrowed-sharding-allowed 下 row-plan 照列、run_stage 照判，这一格判错。"""
    repository = os.path.join(work, "narrowing")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    narrowed_note = "缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、没入库"
    narrowed = selftest.run(repository, SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO="3")
    plain = selftest.run(repository)
    selftest.check("设了缩域变量：现判的行不入库、成功句写缩过域——SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3 的一趟四行都现判、都抓到、退 0，抓到那四句与成功句写缩过域（变量=值）、"
                   "不算门禁通过、没入库，不记覆盖；同一份输入不设变量再跑一趟，四行仍现判、不复用",
                   narrowed[0] == 0 and narrowed[1].count(f"红了；{narrowed_note}（现判：") == 4
                   and "；缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、现判的没入库（复用的是库里全域下判的）；判定库在 " in narrowed[1]
                   and "这一趟现判 4 条、复用 0 条" in narrowed[1] and not coverage_invocations(narrowed[3])
                   and plain[0] == 0 and "这一趟现判 4 条、复用 0 条" in plain[1] and "缩过域" not in plain[1],
                   f"缩域那一趟退 {narrowed[0]}、记覆盖 {len(coverage_invocations(narrowed[3]))} 次、stdout「{narrowed[1][-900:]}」；"
                   f"不设变量那一趟退 {plain[0]}、stdout「{plain[1][-500:]}」")
    reused = selftest.run(repository, SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO="5", **selftest_one_forced(4))
    selftest.check("缩过域的一趟照复用库里全域下判的抓到、不抽复用哨兵：SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5、哨兵取 3 条强制现判 1 条的一趟四行都复用、"
                   "假 cargo 基线与变异一次都没起，stderr 写不抽哨兵与缩过域的原因，成功句写缩过域、不算门禁通过",
                   reused[0] == 0 and "这一趟现判 0 条、复用 4 条" in reused[1] and not plain_invocations(reused[3])
                   and "：缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）：强制现判出的是缩域下的结论" in reused[2] and "不抽哨兵" in reused[2]
                   and "；缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）、不算门禁通过、现判的没入库" in reused[1],
                   f"退 {reused[0]}；假 cargo 起了 {len(plain_invocations(reused[3]))} 次基线与变异；stdout「{reused[1][-700:]}」；stderr「{reused[2][-900:]}」")
    # row-plan 照 os.environ 读环境：这一格里换成自证的环境（假 cargo、临时运行记录）加缩域变量，跑完原样换回
    saved = dict(os.environ)
    os.environ.update(selftest.base_environment, SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO="3")
    listing = io.StringIO()
    try:
        with contextlib.redirect_stdout(listing), contextlib.redirect_stderr(io.StringIO()):
            row_plan_exit = command_row_plan([repository])
    finally:
        os.environ.clear()
        os.environ.update(saved)
    sharded = selftest.run(repository, SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO="3", TASK_SERVER_SHARD_PHASE="shard")
    selftest.check("缩过域不分片：设了缩域变量的 row-plan 退 2、红句写不分片、出路指到 prove-red.sh、一行都不列；run_stage 在 TASK_SERVER_SHARD_PHASE=shard 下退 2、假 cargo 一次都没起",
                   row_plan_exit == 2 and "缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）的变异不分片" in listing.getvalue()
                   and "scripts/mutation/prove-red.sh" in listing.getvalue() and not re.search(r"^\d+\t", listing.getvalue(), re.M)
                   and sharded[0] == 2 and "的变异不分片" in sharded[1] and not sharded[3],
                   f"row-plan 退 {row_plan_exit}「{listing.getvalue()[-500:]}」；分片的一片退 {sharded[0]}、假 cargo 起了 {len(sharded[3])} 次「{sharded[1][-400:]}」")


def selftest_unit_key_cells(selftest, work):
    """单元键几格（controls 路第 5 节 crates 变异那一行：S34、S30、N33、N34、N45；另加被改坏的单元、点名测试的闭包、V5 那一类常量、
    覆盖复用基线、拒算、plan、改 rustc 与 cargo 的版本和编译器环境变量、llvm 工具几格）：每格施加一处改动，看哪几行现判、栏对不对；弄坏开关各让点名的那一格判错。"""
    repository = os.path.join(work, "units")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    table_path = os.path.join(repository, TABLE)
    names = [row.split("\t")[0] for row in SELFTEST_UNIT_ROWS]

    def fresh_and_reused(run):
        how = per_row_how(run[1])
        return sorted(name for name in how if how[name].startswith("现判")), sorted(name for name in how if how[name] == "复用"), how

    first = selftest.run(repository)
    coverage_files = selftest_library(library_of(repository)).stored_names("coverage")
    selftest.check("头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份（5 行 4 条测试：两行点名 checks_root）",
                   first[0] == 0 and len(coverage_files) == 4 and len(coverage_invocations(first[3])) == 8 and "行新加 5" in first[1]
                   and "覆盖：这一趟记了 4 份" in first[1], f"覆盖记录 {coverage_files}；{first[1]}")
    second = selftest.run(repository)
    selftest.check("输入不变的第二趟：5 行都复用、cargo 一次都不起", second[0] == 0 and second[3] == [] and fresh_and_reused(second)[1] == sorted(names), second[1])
    # plan：只读、逐行打、末行报总数
    library_before = selftest_library(library_of(repository)).service.scan("")
    plan = run_command_capturing(command_plan, [repository])
    library_after = selftest_library(library_of(repository)).service.scan("")
    selftest.check("plan：只读（库里一条都没多、没改）、逐行打复用还是现判、末行报总数",
                   plan[0] == 0 and library_after == library_before and plan[1].count("\t复用\t键相同\t") == 5
                   and plan[1].splitlines()[-1].startswith("合计 5 行：复用 5 行、现判 0 行"), plan[1])
    listed = run_command_capturing(command_list_items, [repository])
    selftest.check("--list-items：逐行列可点名的变异，末行报键对不上 0 条、对得上 5 条、拒算 0 条",
                   listed[0] == 0 and listed[1].count("\t第 ") == 5 and "键对不上 0 条（这一趟要现判）、键对得上 5 条（复用）、拒算 0 条" in listed[1], listed[1])
    # S34：被改坏的函数 check_root 的别一行变了：原文落在它里面的两行（root-457、root-double）现判，栏是被改坏的单元变了；别的三行复用
    replace_in_file(source, "base.max(OFFSET)", "base.max(OFFSET + 0)")
    s34 = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(s34)
    selftest.check("被改坏函数的别一行变了（S34）：原文在它里面的两行现判、栏是被改坏的单元变了，别的三行复用",
                   s34[0] == 0 and fresh == ["root-457", "root-double"] and how.get("root-457") == "现判：被改坏的单元变了"
                   and reused == ["helper-eight", "sum-sub", "taken-eleven"]
                   and "root-457：被改坏的单元变了——被改坏的单元 crates/tiny/src/lib.rs::check_root 变了" in s34[2], f"{how}；{s34[1]}{s34[2][-600:]}")
    # N45：同一个文件里加一个没人调的函数：一行都不现判
    replace_in_file(source, "pub fn pad(", "pub fn unused_helper() -> u32 { 70 }\npub fn pad(")
    n45 = selftest.run(repository)
    selftest.check("同一个文件里加一个没人调的函数（N45）：一行都不现判、cargo 一次都不起", n45[0] == 0 and n45[3] == [] and fresh_and_reused(n45)[0] == [],
                   n45[1] + "\n".join(n45[3]))
    # N34：给一条变异改名：一行都不现判（变异名不进行键）
    replace_in_file(table_path, "sum-sub\t", "sum-becomes-sub\t")
    n34 = selftest.run(repository)
    selftest.check("给一条变异改名（N34）：一行都不现判、改了名的那一行复用", n34[0] == 0 and n34[3] == [] and per_row_how(n34[1]).get("sum-becomes-sub") == "复用",
                   n34[1])
    # 改了点名测试调到、不是被改坏的那个函数（sums 调的 pad）：点名它的那一行现判，栏是点名测试的闭包变了；弄坏开关 test-closure-ignored 下复用、这一格判错
    replace_in_file(source, "{ value + 0 }", "{ value + 0 + 0 }")
    padded = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(padded)
    selftest.check("改了点名测试调到、不是被改坏的函数：点名它的那一行现判（点名测试的闭包变了），别的复用",
                   padded[0] == 0 and fresh == ["sum-becomes-sub"] and how.get("sum-becomes-sub") == "现判：点名测试的闭包变了" and len(reused) == 4, f"{how}；{padded[1]}")
    # 改了被改坏、点名测试够不着的函数（helps 只调 add，helper 是被改坏的那个）：只现判那一行，栏是被改坏的单元变了；弄坏开关 mutated-unit-ignored 下复用、这一格判错
    replace_in_file(source, "pub fn helper(value: u32) -> u32 { value + 7 }", "pub fn helper(value: u32) -> u32 { let _seven = 7; { value + 7 } }")
    helper = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(helper)
    selftest.check("改了被改坏、点名测试够不着的函数：只现判那一行（被改坏的单元变了）",
                   helper[0] == 0 and fresh == ["helper-eight"] and how.get("helper-eight") == "现判：被改坏的单元变了" and len(reused) == 4, f"{how}；{helper[1]}")
    # V5 那一类：改了被改坏函数用的常量 OFFSET（check_root 自己的指纹不变）：原文在 check_root 里的两行现判，栏是点名测试的闭包变了
    replace_in_file(source, "OFFSET: u32 = 95", "OFFSET: u32 = 91")
    constant = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(constant)
    selftest.check("改了被改坏函数用的常量（V5 那一类）：原文在那个函数里的两行现判，别的复用，现判的原因点名那个常量",
                   constant[0] == 0 and fresh == ["root-457", "root-double"] and how.get("root-457") == "现判：点名测试的闭包变了" and len(reused) == 3
                   and "改了 crates/tiny/src/lib.rs::OFFSET（const）" in constant[2], f"{how}；{constant[1]}{constant[2][-600:]}")
    # 覆盖复用基线：改了 branches 静态够得着、没执行到的 untaken：那一行现判（闭包变了），它的基线按覆盖复用、不跑基线；
    # 弄坏开关 coveragenotrecorded 下没有覆盖记录、基线只按闭包比、要跑一批基线，这一格判错
    replace_in_file(source, "{ value + 10 }", "{ value + 12 }")
    untaken = selftest.run(repository)
    fresh, _reused, how = fresh_and_reused(untaken)
    selftest.check("改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用（只起那一条变异、不跑基线）",
                   untaken[0] == 0 and fresh == ["taken-eleven"] and len(plain_invocations(untaken[3])) == 1 and "复用基线 ok 1 行" in untaken[2],
                   f"{how}；{untaken[2][:400]}" + "\n".join(untaken[3]))
    # 基线不按整树快照：加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用；弄坏开关 baseline-by-tree-snapshot 下基线现跑、这一格判错
    replace_in_file(source, "pub fn pad(", "pub fn unused_too() -> u32 { 71 }\npub fn pad(")
    replace_in_file(table_path, "{ left - right }", "{ left * right }")
    snapshot = selftest.run(repository)
    fresh, _reused, how = fresh_and_reused(snapshot)
    selftest.check("加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用（不按整树快照）",
                   snapshot[0] == 0 and fresh == ["sum-becomes-sub"] and how.get("sum-becomes-sub") == "现判：原文或替换文变了" and len(plain_invocations(snapshot[3])) == 1,
                   f"{how}；" + "\n".join(snapshot[3]))
    # 改 cargo 的版本（假 cargo 的 -V）：行键不变，每一行都复用、cargo 一次不起；弄坏开关 toolchain-version-keyed 下 KeyContext 收 rustc -vV、cargo -V，每一行现判、这一格判错
    versioned = selftest.run(repository, FAKE_CARGO_VERSION="cargo 0.0.1 (crates-mutation-rows selftest：换了 cargo 的版本)")
    selftest.check("改 rustc、cargo 版本之后行键不变：假 cargo 的 -V 换了一行，每一行都复用、cargo 一次都不起",
                   versioned[0] == 0 and per_row_how(versioned[1]) and set(per_row_how(versioned[1]).values()) == {"复用"} and versioned[3] == [], versioned[1])
    # 改编译器与链接器相关的环境变量：行键不变，每一行都复用、cargo 一次不起；弄坏开关 build-environment-keyed 下 KeyContext 收环境变量，每一行现判、这一格判错（toolchain-version-keyed 下同样判错：它收的行里有环境变量）
    flagged = selftest.run(repository, RUSTFLAGS="--cfg crates_mutation_rows_selftest")
    selftest.check("改编译器的环境变量 RUSTFLAGS 之后行键不变：每一行都复用、cargo 一次都不起",
                   flagged[0] == 0 and per_row_how(flagged[1]) and set(per_row_how(flagged[1]).values()) == {"复用"} and flagged[3] == [], flagged[1])
    # 拒算：点名的测试是宏展开出来的、认不出单元：那一行每趟照跑、不复用，成功句报拒算 1 条，stderr 写为什么
    refused_root = os.path.join(work, "refused")
    build_selftest_repository(refused_root, SELFTEST_TABLE_ROWS[:1] + ["unknown-test\tcrates/tiny/src/lib.rs\tu32) -> u32 { value * 2 }\tu32) -> u32 { value * 5 }\t-p tiny --lib -- doubled\tdoubled"],
                              "// TEST doubled NEEDS { value * 2 }\n")
    refused_first = selftest.run(refused_root)
    refused_second = selftest.run(refused_root)
    selftest.check("点名的测试认不出单元：那一行拒算、每趟照跑，成功句报拒算 1 条，stderr 写为什么",
                   refused_first[0] == 0 and refused_second[0] == 0 and per_row_how(refused_second[1]).get("unknown-test") == "拒算"
                   and len(mutation_invocations(plain_invocations(refused_second[3]), source_hash(refused_root))) == 1
                   and "现判 0 条、复用 1 条、拒算 1 条" in refused_second[1] and "拒算（照跑、不复用）" in refused_second[2],
                   refused_second[1] + refused_second[2][-400:])
    mismatched = selftest.run(selftest_new_library_copy(work, repository, "units-llvm-mismatched"), FAKE_LLVM_VERSION="0.0.0")   # 换一棵树：要现跑的行才查 llvm 工具
    selftest.check("llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑", mismatched[0] == 1 and mismatched[3] == []
                   and "记覆盖要的 llvm 工具不对" in mismatched[1], mismatched[1] + mismatched[2])
    empty_tools = os.path.join(work, "no-llvm-tools")
    os.makedirs(empty_tools)
    missing = selftest.run(selftest_new_library_copy(work, repository, "units-llvm-missing"), GATE_MUTATION_LLVM_TOOLS=empty_tools)
    selftest.check("llvm 工具不在：判红、一条都不跑、出路写 rustup component add llvm-tools", missing[0] == 1 and missing[3] == []
                   and "rustup component add llvm-tools" in missing[1], missing[1])


def selftest_parallel_key_cell(selftest, work):
    """算闭包的子进程数不同（就地算与 fork 4 个子进程），每一行的键逐字相同（并行之后重新证它与串行一致）；行数要够 PARALLEL_MINIMUM 才真起子进程。"""
    count = mutation_units.PARALLEL_MINIMUM + 2
    functions = "".join(f"pub fn step_{index}(value: u32) -> u32 {{ value + {index + 100} }}\n" for index in range(count))
    tests = "".join(f"    #[test]\n    fn steps_{index}() {{ assert_eq!(super::step_{index}(1), {index + 101}); }}\n" for index in range(count))
    source = "// 并行算闭包那一格的样本 crate\n" + functions + "\n#[cfg(test)]\nmod tests {\n" + tests + "}\n"
    rows = [f"step-{index}\tcrates/tiny/src/lib.rs\t{{ value + {index + 100} }}\t{{ value + {index + 200} }}\t-p tiny --lib -- steps_{index}\tsteps_{index}"
            for index in range(count)]
    repository = os.path.join(work, "parallel-keys")
    build_selftest_repository(repository, rows, source=source)
    saved = os.environ.get("GATE_MUTATION_KEY_WORKERS")
    outputs = {}
    try:
        for workers in ("1", "4"):
            os.environ["GATE_MUTATION_KEY_WORKERS"] = workers
            outputs[workers] = run_command_capturing(command_row_plan, [repository])
    finally:
        if saved is None:
            os.environ.pop("GATE_MUTATION_KEY_WORKERS", None)
        else:
            os.environ["GATE_MUTATION_KEY_WORKERS"] = saved
    keys = [line.split("\t")[1] for line in outputs["4"][1].splitlines() if "\t" in line]
    selftest.check(f"算闭包的子进程数不同（就地算与 4 个子进程）：{count} 行的键逐字相同、没有一行拒算",
                   outputs["1"][0] == 0 and outputs["4"] == outputs["1"] and len(keys) == count and "-" not in keys,
                   f"1 个：{outputs['1'][1][:400]}；4 个：{outputs['4'][1][:400]}")


def selftest_wgsl_unit_key_cell(selftest, work):
    """WGSL 按 fn 取键（lib_mutation_units.py 文件头「单元」）：一份 .wgsl 两个 fn（scaled、untouched）加一个模块级常量 SCALE，
    改坏 scaled 的一行与改坏常量的一行键算得出、原文只在注释里的一行拒算；改 untouched 之后两行的键都不变，改 scaled 的别一行之后只有改坏 scaled 的那一行键变。
    键从 row-plan 读（只读，不起 cargo）。弄坏开关 wgsl-whole-file（.wgsl 取整份字节的 sha256）下这一格必须判错。"""
    repository = os.path.join(work, "wgsl-units")
    build_selftest_repository(repository, SELFTEST_WGSL_ROWS)
    kernel = os.path.join(repository, "crates", "tiny", "src", "kernel.wgsl")
    selftest_write(kernel, SELFTEST_WGSL_KERNEL)

    def keys():
        exit_code, output = run_command_capturing(command_row_plan, [repository])
        found = {}
        for line in output.splitlines():
            fields = line.split("\t")
            if len(fields) == 5 and fields[0].isdigit():
                found[int(fields[0])] = fields[1]
        return exit_code, [found.get(line_number) for line_number in (2, 3, 4)], output

    first_code, first, first_output = keys()
    replace_in_file(kernel, "return value + 5u;", "return value + 6u;")
    other_code, other, other_output = keys()
    replace_in_file(kernel, "let base = value + 1u;", "let base = value + 2u;")
    mutated_code, mutated, mutated_output = keys()
    keyed = all(key is not None and re.fullmatch(r"[0-9a-f]{64}", key) for key in first[:2])
    selftest.check("WGSL 按 fn 取键：改坏 fn 与改坏模块级常量的两行键算得出、原文只在注释里的一行拒算；改另一个 fn 之后两行键都不变，改被改坏的 fn 的别一行之后只有那一行键变",
                   (first_code, other_code, mutated_code) == (0, 0, 0) and keyed and first[2] == "-"
                   and other[:2] == first[:2] and mutated[0] != other[0] and mutated[1] == other[1],
                   f"头一趟 {first}；改 untouched 之后 {other}；改 scaled 之后 {mutated}：{first_output[-300:]}{other_output[-300:]}{mutated_output[-300:]}")


def selftest_item_lines_cells(selftest, work):
    """按项机读行（文件头「按项机读行」）：
    ① 真跑：头一趟 4 项全现判，stderr 里每项恰好一行 ITEM_DONE（项名与表里的一一对上、judged=1 reused=0 exit=0），stdout 里没有这一行（stdout 与进程数无关）；
    ② 真跑：输入不变的第二趟全复用，仍是每项一行（judged=0 reused=1 exit=0 seconds=0.000）——弄坏开关 item-done-reused-silent 下一行都没有，这一格判错；
    ③ --list-items --with-keys：改了一行的替换文，4 项各一行 ITEM_PLAN，那一项 to_judge=1 reusable=0、别的 to_judge=0 reusable=1，条数之和等于末行合计的「键对不上」与「键对得上」，
       行数等于逐行列出的项数，不起 cargo——弄坏开关 item-plan-reusable-zero 下 reusable 一律 0，这一格判错；
    ④ 真跑：每项各记各的挂钟：假 cargo 每条改坏了的变异睡 1 秒（基线与覆盖不睡），1 个工作进程串行判 4 条，每项 seconds 不小于 1、各项之和不超过整趟挂钟
       （各项判一行的起止互不包含；机器忙只会让每项更长，两条都不随忙闲变）；
    ⑤ 各项的 seconds 是这一项各行自己的秒数相加：直接给 ItemProgress 各行的秒数，中间隔 0.05 秒，不看挂钟——
       弄坏开关 item-done-seconds-whole-run 下记自开跑起的整趟，⑤ 这一格判错（打出来的至少是隔的那一会儿、不是各行秒数之和）。"""
    names = [row.split("\t")[0] for row in SELFTEST_TABLE_ROWS]

    def done_lines(run):
        return [match.groupdict() for match in map(ITEM_DONE_LINE.match, run[2].splitlines()) if match]

    repository = os.path.join(work, "item-lines")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    first = selftest.run(repository)
    first_done = done_lines(first)
    selftest.check("真跑：头一趟 4 项全现判，stderr 里每项恰好一行 ITEM_DONE（judged=1 reused=0 exit=0），stdout 里没有",
                   first[0] == 0 and sorted(line["item"] for line in first_done) == sorted(names)
                   and all(line["judged"] == "1" and line["reused"] == "0" and line["exit"] == "0" for line in first_done)
                   and "ITEM_DONE" not in first[1], f"退 {first[0]}；ITEM_DONE {first_done}；{first[2][-600:]}")
    second = selftest.run(repository)
    second_done = done_lines(second)
    selftest.check("真跑：全复用的第二趟每项一行 ITEM_DONE（judged=0 reused=1 exit=0 seconds=0.000）",
                   second[0] == 0 and sorted(line["item"] for line in second_done) == sorted(names)
                   and all(line["judged"] == "0" and line["reused"] == "1" and line["exit"] == "0" and line["seconds"] == "0.000" for line in second_done),
                   f"退 {second[0]}；ITEM_DONE {second_done}；{second[2][-600:]}")
    replace_in_file(os.path.join(repository, TABLE), "{ value * 3 }", "{ value * 4 }")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(selftest.log)
    listed = run_command_capturing(command_list_items, ["--with-keys", repository])
    plan_text = [line for line in listed[1].splitlines() if line.startswith("ITEM_PLAN ")]
    plan = {match["item"]: (int(match["to_judge"]), int(match["reusable"])) for match in map(ITEM_PLAN_LINE.match, plan_text) if match}
    listed_items = [line for line in listed[1].splitlines() if "\t第 " in line]
    totals = re.search(r"键对不上 ([0-9]+) 条（这一趟要现判）、键对得上 ([0-9]+) 条（复用）、拒算 ([0-9]+) 条", listed[1])
    cargo_runs = open(selftest.log, encoding="utf-8").read().splitlines() if os.path.exists(selftest.log) else []
    selftest.check("--list-items --with-keys：每项一行 ITEM_PLAN（行数等于逐行列出的项数），改了替换文的那一项 to_judge=1、别的 reusable=1，条数之和等于末行合计，不起 cargo",
                   listed[0] == 0 and len(plan_text) == len(listed_items) == len(names) and len(plan) == len(names)
                   and plan == {name: ((1, 0) if name == "double-becomes-triple" else (0, 1)) for name in names}
                   and totals is not None and (sum(value[0] for value in plan.values()), sum(value[1] for value in plan.values())) == (int(totals[1]) + int(totals[3]), int(totals[2]))
                   and not cargo_runs, f"退 {listed[0]}；ITEM_PLAN {plan_text}；逐行 {len(listed_items)} 项；合计 {totals and totals.group(0)}；假 cargo 起过 {cargo_runs}；{listed[1][-500:]}")
    table_rows_now = read_table(os.path.join(repository, TABLE))[0]
    named_line = next(row.line_number for row in table_rows_now if row.name == "double-becomes-triple")
    selection_file = os.path.join(work, "item-lines-selection.txt")
    selftest_write(selection_file, f"{named_line}\n")
    only_one = run_command_capturing(command_list_items, ["--with-keys", "--rows", selection_file, repository])
    only_one_plan = [line for line in only_one[1].splitlines() if line.startswith("ITEM_PLAN ")]
    only_one_context = plan_context(repository, dict(os.environ), selection_file)
    selftest.check("--list-items --with-keys --rows：只打点名那一行的 ITEM_PLAN、只算那一行的键（别的行的单元闭包不读），合计只数这一行",
                   only_one[0] == 0 and only_one_plan == ["ITEM_PLAN item=double-becomes-triple to_judge=1 reusable=0"]
                   and len([line for line in only_one[1].splitlines() if "\t第 " in line]) == 1
                   and only_one_context is not None and len(only_one_context["components"]) == 1
                   and "键对不上 1 条（这一趟要现判）、键对得上 0 条（复用）" in only_one[1],
                   f"退 {only_one[0]}；ITEM_PLAN {only_one_plan}；算了键的行 {len(only_one_context['components']) if only_one_context else None}；{only_one[1][-400:]}")
    sleeping_repository = selftest_new_library_copy(work, repository, "item-lines-sleeping")   # 换一棵树：库是新的一份，4 条变异都要现判
    sleeping_started = time.monotonic()
    sleeping = selftest.run(sleeping_repository, FAKE_CARGO_MUTATION_SLEEP="1.0", FAKE_CARGO_BARRIER_PRISTINE=source_hash(sleeping_repository))
    sleeping_wall = time.monotonic() - sleeping_started
    sleeping_seconds = [float(line["seconds"]) for line in done_lines(sleeping)]
    selftest.check("真跑：每项各记各的挂钟（每条改坏了的变异睡 1 秒、串行判 4 条，每项 seconds 不小于 1、各项之和不超过整趟挂钟，不是每项都记整趟）",
                   sleeping[0] == 0 and len(sleeping_seconds) == len(names) and all(seconds >= 0.99 for seconds in sleeping_seconds)
                   and sum(sleeping_seconds) <= sleeping_wall + 0.05,
                   f"退 {sleeping[0]}；每项的 seconds {sleeping_seconds}，之和 {sum(sleeping_seconds):.3f}，整趟挂钟 {sleeping_wall:.3f}；{sleeping[2][-400:]}")
    # 各项的 seconds 是这一项各行自己的秒数相加：不看挂钟，直接给 ItemProgress 各行的秒数，中间隔一小会儿；记整趟的话 seconds 是自开跑起的挂钟（不小于隔的那一会儿）
    progress_rows = [Row(1, "first-item", "crates/tiny/src/lib.rs", "a", "b", "-p tiny --lib", "adds", ()),
                     Row(2, "first-item", "crates/tiny/src/lib.rs", "c", "d", "-p tiny --lib", "adds", ()),
                     Row(3, "second-item", "crates/tiny/src/lib.rs", "e", "f", "-p tiny --lib", "doubles", ())]
    progress = ItemProgress(progress_rows, dict(selftest.base_environment))
    progress_stderr = io.StringIO()
    with contextlib.redirect_stderr(progress_stderr):
        progress.settle(1, "caught", seconds=0.001)
        time.sleep(0.05)
        progress.settle(3, "caught", seconds=0.004)
        progress.settle(2, "caught", seconds=0.002)
    progress_done = {match["item"]: match["seconds"] for match in map(ITEM_DONE_LINE.match, progress_stderr.getvalue().splitlines()) if match}
    selftest.check("各项的 seconds 是这一项各行自己的秒数相加（first-item 两行 0.001 与 0.002 得 0.003，second-item 一行 0.004 得 0.004），不是自开跑起的整趟",
                   progress_done == {"first-item": "0.003", "second-item": "0.004"}, f"打出来的 seconds {progress_done}；{progress_stderr.getvalue()[-400:]}")


def selftest_anchor_and_impact_cells(selftest, work):
    """锚点失效按行判（文件头「锚点失效」）与牵连面（文件头「牵连面」）。"""
    # 锚点失效按行判：第 6 行原文命中 0 次，记锚点失效、整道判红；别的 4 行照跑、照入库；第二趟那 4 行复用、锚点失效那一行照旧红、不入库
    anchor_root = os.path.join(work, "anchor-per-row")
    build_selftest_repository(anchor_root, SELFTEST_TABLE_ROWS + [SELFTEST_ANCHOR_MISSING_ROW])
    anchor_pristine = source_hash(anchor_root)
    anchored = selftest.run(anchor_root)
    records_after_first = selftest_library(library_of(anchor_root)).stored_names("records")
    anchored_again = selftest.run(anchor_root)
    anchor_line = f"{TABLE}:6 anchor-missing：锚点失效——原文在 crates/tiny/src/lib.rs 里命中 0 次，要恰好 1 次（这一行没跑、不入库、不复用）"
    selftest.check("锚点失效按行判：那一行记锚点失效、别的行照跑照入库（计数报锚点失效 1 行、整道判红；第二趟那 4 行复用、锚点失效那一行照旧红）",
                   anchored[0] == 1 and len(mutation_invocations(plain_invocations(anchored[3]), anchor_pristine)) == 4
                   and "计数：抓到 4 条、没红 0 条" in anchored[1] and "无效 0 条、锚点失效 1 行（共 5 条）" in anchored[1]
                   and anchor_line in anchored[1] and "有几行锚点失效（锚点失效 1 行" in anchored[1] and len(records_after_first) == 4
                   and anchored_again[0] == 1 and anchored_again[3] == [] and "复用 4 条" in anchored_again[1] and anchor_line in anchored_again[1],
                   anchored[1] + anchored[2][-800:] + "\n".join(anchored[3]) + f"\n记录 {len(records_after_first)} 条\n" + anchored_again[1])
    # 派活之后锚点失效：基线那一趟跑完用例之后，假 cargo 把工作副本里 halves 那一行的原文改掉（造「开跑之后别的会话改了源码」）；
    # 那一行记锚点失效、不入库，别的 3 行照判照入库，整道判红，进程池不炸
    mid_root = os.path.join(work, "anchor-mid-run")
    build_selftest_repository(mid_root, SELFTEST_TABLE_ROWS)
    try:
        mid = selftest.run(mid_root, GATE_MUTATION_WORKERS="1", FAKE_CARGO_EDIT_SOURCE="{ value / 2 }\t{ value >> 1 }")
    except AssertionError as error:
        mid = (None, f"抛了 AssertionError：{error}", "", [])
    mid_record_count = len(selftest_library(library_of(mid_root)).stored_names("records"))
    selftest.check("派活之后锚点失效：那一行记锚点失效，别的行照判照入库",
                   mid[0] == 1 and "计数：抓到 3 条、没红 0 条" in mid[1] and "锚点失效 1 行（共 4 条）" in mid[1]
                   and "halve-becomes-third：锚点失效——派活之后源码被改了" in mid[1] and mid_record_count == 3,
                   mid[1] + mid[2][-800:] + f"\n记录 {mid_record_count} 条")
    # row-plan、plan、--list-items 把锚点失效的行单列：第 6 行（.rs）与第 7 行（数据文件，单元键不核锚点）都写「-」键与 anchor-broken，不算进现判
    listed_root = os.path.join(work, "anchor-listed")
    build_selftest_repository(listed_root, SELFTEST_TABLE_ROWS + [SELFTEST_ANCHOR_MISSING_ROW, SELFTEST_DATA_FILE_ANCHOR_ROW])
    row_plan = run_command_capturing(command_row_plan, [listed_root])
    plan_lines = {line.split("\t")[0]: line.split("\t") for line in row_plan[1].splitlines() if "\t" in line}
    plan = run_command_capturing(command_plan, [listed_root])
    listed = run_command_capturing(command_list_items, [listed_root])
    selftest.check("row-plan、plan、--list-items 把锚点失效的行单列（数据文件那一行也不算键）",
                   row_plan[0] == 0 and plan_lines.get("6") == ["6", "-", "0", "-", "anchor-broken"] and plan_lines.get("7") == ["7", "-", "0", "-", "anchor-broken"]
                   and all(plan_lines.get(str(number), ["", "-"])[1] != "-" and plan_lines[str(number)][4] == "-" for number in range(2, 6))
                   and plan[0] == 0 and "6\tanchor-missing\t锚点失效\t锚点失效\t原文在 crates/tiny/src/lib.rs 里命中 0 次" in plan[1]
                   and plan[1].splitlines()[-1].startswith("合计 6 行：") and plan[1].splitlines()[-1].endswith("拒算 0 行、锚点失效 2 行")
                   and listed[0] == 0 and listed[1].count("；锚点失效：") == 2
                   and "锚点失效 2 行（不跑、整道判红：第 6 行 anchor-missing、第 7 行 data-file-anchor-missing）" in listed[1],
                   row_plan[1] + plan[1] + listed[1])
    # 牵连面：单元键样本 crate 上 6 行，checks_root 点名三行（root-457、root-double、offset-plus-one），它的闭包（tests::checks_root、check_root、OFFSET）各进 3 行；
    # 弄坏开关 impact-row-dropped 少算头一行（root-457），数就对不上
    impact_root = os.path.join(work, "impact")
    build_selftest_repository(impact_root, SELFTEST_UNIT_ROWS + [SELFTEST_IMPACT_ROW], SELFTEST_IMPACT_TEST, source=SELFTEST_UNIT_SOURCE)
    impact_listed = run_command_capturing(command_list_items, [impact_root])
    impact_lines = [line for line in impact_listed[1].splitlines() if "牵连最大的函数单元：" in line]
    impact_run = selftest.run(impact_root)
    run_impact_lines = [line for line in impact_run[1].splitlines() if "牵连最大的函数单元：" in line]
    selftest.check("牵连面：样本 crate 上牵连最大的单元与行数（--list-items 末了与成功句之后各一行，数一样）",
                   len(impact_lines) == 1 and impact_lines[0] == "  · " + SELFTEST_IMPACT_EXPECTED
                   and impact_run[0] == 0 and len(run_impact_lines) == 1 and run_impact_lines[0] == "  " + SELFTEST_IMPACT_EXPECTED
                   and impact_run[1].splitlines()[-1] == run_impact_lines[0] and impact_run[1].splitlines()[-2].startswith("  ✓ crates 变异表复跑：6 条"),
                   impact_listed[1] + impact_run[1])
