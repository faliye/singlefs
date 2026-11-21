"""crates-mutation-rows.py 拆出的一份。
一道的整个跑法：按项机读行、run_stage、复用哨兵、记覆盖的排程（CoveragePlanner）、llvm 工具、汇总报告，连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import contextlib
import io
import os
import re
import signal
import subprocess
import sys
import tempfile
import time
from crates_mutation_rows_base import (BUILD_PROFILES_DIRECTORY_NAME, CARGO_CONFIGURATION_DIRECTORY, CGROUP_LISTING, CGROUP_ROOT,
    CLOSURE_REFUSED_HOWTO, COLLISION_REFUSAL_PREFIX, COLUMN_TITLES, DEFAULT_LONG_ROW_SECONDS, Decision, EXIT_SELECTION_ONLY, HeavyThreadDefaultError,
    ITEM_DONE_PREFIX, ITEM_EXIT_BY_VERDICT, ITEM_PLAN_PREFIX, LINK_FAILURE_LINKER, LLVM_TOOLS, LLVM_TOOLS_VARIABLE, MODULE_PATH,
    PRUNE_SERVER_BINARY_VARIABLE, REASONS_SHOWN, RELEASE_REGISTRY_HOWTO, REUSABLE_VERDICT, REUSE_COLUMNS, ROCKSDB_PREBUILT_PATH,
    RUN_BUILD_REMOVAL_FAILURES, Row, SCRIPT_KEY, SEED_SOURCE_SUFFIX, SHARD_ROOT_PREFIX, STAGE, STAGE_CHILD_PROGRAM, STAGE_REPOSITORY, TABLE,
    TARGET_DIRECTORY_CAP_VARIABLE, VERDICTS_COUNTED, VERDICTS_UNCOUNTED, admission_module, break_is, cargo_debug_overrides, copied_paths,
    format_size, merge_progress_variable, merge_step_close, merge_step_open, merge_step_row_done, mutation_units, note, print_rejection, read_table,
    reuse_checks, say)
from crates_mutation_rows_judging import (anchor_broken_text, caught_text, executed_set_is_current, given_states_gpu_counts, named_test_stdout,
    row_libtest_selection, verdict_of_run)
from crates_mutation_rows_support import (build_selftest_repository, coverage_invocations, mutation_invocations, plain_invocations, selftest_write,
    source_hash)
from crates_mutation_rows_static import (release_only_tests_module)
from crates_mutation_rows_units import (index_entry, library_of, release_key_context, row_identity,
    run_records_directory, selftest_library, snapshot_identity, snapshot_of)
from crates_mutation_rows_baseline import (baseline_batches, baseline_chunks, batch_key, dispatch_order)
from crates_mutation_rows_targets import (StopRequested, crash_domain_narrowing, default_shared_target_directory, disk_bound_for,
    environment_records, measured_target_peaks, narrowed_outcome_text, narrowed_sentinel_off_reason, narrowed_sharding_refusal,
    planned_run_build_target_directory, reclaim_extra_target_directories, selftest_run_target_directories, size_setting_problems,
    target_directory_cap_bytes, target_reclaim_lines)
from crates_mutation_rows_memory import (CARGO_JOBS_VARIABLE, cargo_jobs_for, compile_thread_budget, host_memory_capacity, memory_bound_probing, worker_count_for,
    worker_peak_keys, worker_peaks_recorded_for)
from crates_mutation_rows_runs import (test_runs_multi_host_configuration)
from crates_mutation_rows_pool import (run_pool, run_pool_reporting_dead_worker)
from crates_mutation_rows_plan import (baseline_check, decide_rows, impact_summary, light_components, plan_rows, prechecks, read_selection)


def item_exit_code(verdicts):
    """一项（同名的几行合成一项）的退出码（文件头「按项机读行」）：有一行不是抓到也不是本次未跑的 1，没有而有本次未跑的 77，其余 0。"""
    codes = [ITEM_EXIT_BY_VERDICT.get(verdict, 1) for verdict in verdicts]
    return 1 if 1 in codes else 77 if 77 in codes else 0


class ItemProgress:
    """真跑时每项跑完打一行 ITEM_DONE（stderr，文件头「按项机读行」）：同名的几行合成一项，一项的行全有了结局（复用的、现判的、基线不绿的、锚点失效的）就打一次。
    seconds 是这一项各行在工作进程里判一行从头到尾的秒数相加（几项并行时各记各的），不是整趟。弄坏开关 item-done-reused-silent、item-done-seconds-whole-run。"""

    def __init__(self, rows, environment):
        self.environment = environment
        self.started = time.monotonic()
        self.pending = collections.OrderedDict()   # 项名 -> 还没有结局的行号
        self.name_of = {}
        for row in rows:
            self.pending.setdefault(row.name, set()).add(row.line_number)
            self.name_of[row.line_number] = row.name
        self.verdicts = collections.defaultdict(list)
        self.judged = collections.Counter()
        self.reused = collections.Counter()
        self.seconds = collections.defaultdict(float)
        self.printed = set()

    def settle(self, line_number, verdict, reused=False, seconds=0.0):
        """一行有了结局：verdict 是 caught / failure / invalid / linkfailed / baseline / notrun / anchor；reused 是复用的那一行。最后一行到了就打这一项。"""
        name = self.name_of.get(line_number)
        if name is None or line_number not in self.pending[name]:
            return
        self.pending[name].discard(line_number)
        self.verdicts[name].append(verdict)
        if reused:
            self.reused[name] += 1
        elif verdict != "anchor":
            self.judged[name] += 1
        self.seconds[name] += seconds
        if not self.pending[name]:
            self.print_done(name)

    def print_done(self, name):
        if name in self.printed:
            return
        self.printed.add(name)
        if break_is(self.environment, "item-done-reused-silent") and self.reused[name] and not self.judged[name]:
            return
        seconds = time.monotonic() - self.started if break_is(self.environment, "item-done-seconds-whole-run") else self.seconds[name]
        note(f"{ITEM_DONE_PREFIX} item={name} seconds={seconds:.3f} judged={self.judged[name]} reused={self.reused[name]} exit={item_exit_code(self.verdicts[name])}")

    def finish(self):
        """收尾：还有行没有结局的项（被停没判到）补一行，这些行按没判到记（exit=1）。跑到的每一项恰好一行。"""
        for name, lines in self.pending.items():
            if name not in self.printed:
                self.verdicts[name] += ["unrun"] * len(lines)
                lines.clear()
                self.print_done(name)


def item_plan_lines(names, context, environment):
    """--list-items --with-keys 的 ITEM_PLAN 行（文件头「按项机读行」）：names 是逐行列出的变异名（表的次序），同名的几行合成一项；
    要现判 = 键对不上与拒算，能复用 = 键对得上，锚点失效的行（不在 context 的行里）两样都不算。弄坏开关 item-plan-reusable-zero：reusable 一律 0。"""
    counts = collections.OrderedDict((name, [0, 0]) for name in names)
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        entry = counts.setdefault(row.name, [0, 0])
        if decision.reuse:
            entry[1] += 0 if break_is(environment, "item-plan-reusable-zero") else 1
        else:
            entry[0] += 1
    return [f"{ITEM_PLAN_PREFIX} item={name} to_judge={to_judge} reusable={reusable}" for name, (to_judge, reusable) in counts.items()]


def run_stage(root, environment=None):
    """门禁 crates-mutation-replay 的整道：判定打 stdout（按表的行号排序，与进程数、复用了多少都无关），进度与复用几条打 stderr。回退出码。
    服务分片的合并那一步（环境里 TASK_SERVER_SHARD_PHASE=merge）里这一道就是核账：开头与退出前各打一行（范围「合并·核账」），其间逐行做的几步照 merge_step_open 打进度。"""
    environment = dict(os.environ if environment is None else environment)
    if environment.get("TASK_SERVER_SHARD_PHASE") == "merge":
        environment[merge_progress_variable()] = "合并·核账"
    progress = merge_step_open(environment, None, None)
    exit_code = run_stage_rows(root, environment)
    merge_step_close(progress, f"退出码 {exit_code}")
    return exit_code


def run_stage_rows(root, environment):
    """run_stage 的正文：环境已由 run_stage 备好。"""
    import gc
    admission = admission_module()
    table_path = os.path.join(root, TABLE)
    if not os.path.isfile(table_path):
        print_rejection(f"没有 {TABLE}", [], ["建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"])
        return 1
    try:
        rows, malformed = read_table(table_path, environment)
    except (OSError, UnicodeDecodeError) as error:
        print_rejection(f"读不了 {TABLE}（{type(error).__name__}：{error}）", [], ["转成 UTF-8 文本再跑；读不了的表一条都判不了。"])
        return 1
    # 不成形的行（read_table 判的）在 prechecks 里与别的几样一起报：整张表的不成形行不因选行而放过
    if not rows and not malformed:
        print_rejection(f"{TABLE} 里一条成形的变异都没有", [], ["至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。"])
        return 1
    size_problems = size_setting_problems(environment)
    for headline, howto_line in size_problems:
        print_rejection(headline, [], [howto_line])
    if size_problems:
        return 1
    numeric_settings = {}
    for variable, default in (("GATE_MUTATION_LONG_ROW_SECONDS", DEFAULT_LONG_ROW_SECONDS), ("GATE_MUTATION_STOP_AFTER_ROWS", ""), (CARGO_JOBS_VARIABLE, ""),
                              ("GATE_MUTATION_BASELINE_CHUNK_FILTERS", ""), ("GATE_MUTATION_PROGRESS_SECONDS", ""), ("GATE_MUTATION_COMPUTING_LINE_SECONDS", "")):
        text = environment.get(variable, "").strip() or default
        if text and not re.fullmatch(r"[1-9][0-9]*", text):
            print_rejection(f"{variable} 写成了「{text}」，不是正整数", [], [f"写成正整数（{variable} 的意思见 scripts/mutation/crates-mutation-rows.py 文件头），或者不设它。"])
            return 1
        numeric_settings[variable] = int(text) if text else None
    # 缩过域的一趟（文件头「缩过域的一趟」）：现判的不入库、复用照库里全域下判的抓到、不抽复用哨兵、不记覆盖，各行结局与成功句写缩过域；
    #   服务分片的各片与合并那一步（TASK_SERVER_SHARD_PHASE）不收：各片不入库，合并按库核账核不到。弄坏开关 narrowed-sharding-allowed：照收
    narrowing = crash_domain_narrowing(environment)
    narrowed_unrecorded = bool(narrowing) and not break_is(environment, "narrowed-verdicts-recorded")
    if narrowing and environment.get("TASK_SERVER_SHARD_PHASE") in ("shard", "merge") and not break_is(environment, "narrowed-sharding-allowed"):
        headline, howto_line = narrowed_sharding_refusal(narrowing)
        print_rejection(headline, [], [howto_line])
        return 2
    selection_path = environment.get("GATE_MUTATION_ROW_SELECTION", "").strip()
    table_rows = rows   # 整张表成形的行：选行时撞键照整张表判（plan_rows 的 table_rows）
    if selection_path and rows:
        selected, selection_problems = read_selection(selection_path, rows)
        if selection_problems:
            print_rejection(f"GATE_MUTATION_ROW_SELECTION 指的那份选行文件不对，一行都没判：", selection_problems,
                            [f"一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）；行号照被判的那一份表现数。"])
            return 1
        rows = [row for row in rows if row.line_number in selected]
    progress = merge_step_open(environment, "预检", None)
    copied, refused, anchor_broken = prechecks(root, rows, malformed, environment)
    if refused:
        return 1
    merge_step_close(progress, f"锚点失效 {len(anchor_broken)} 行")
    # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：起任何 cargo 之前（算键、基线、变异、插桩编译都在这之后）判这台装了、
    #   此刻进程环境的取值对得上；判红一条都不跑、退 1，红句照 entry 原样转打。树里没有依赖 rocksdb 的清单、PATH 上是带标记的假 cargo，entry 报不判，
    #   这时什么都不打（stderr 头一行仍是「判定库记录」那一行）。弄坏开关 rocksdb-entry-unchecked 下不判（自证「RocksDB 预编库的取值缺了」那一格判错）
    if not break_is(environment, "rocksdb-entry-unchecked"):
        entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT_PATH, "entry", "--root", root], env=environment, capture_output=True, text=True,
                               stdin=subprocess.DEVNULL)
        if entry.returncode != 0:
            say(entry.stdout.rstrip("\n") + ("\n" + entry.stderr.rstrip("\n") if entry.stderr.strip() else ""))
            print_rejection("起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条都没跑", [],
                            ['照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再跑同一条命令。'])
            return 1
    # 锚点失效的行（文件头「锚点失效」）单记一档：不算键、不派活、不写库，别的行照跑
    anchor_judgements = [(row.line_number, "anchor", anchor_broken_text(row, anchor_broken[row.line_number]), None)
                         for row in rows if row.line_number in anchor_broken]
    selected_rows = list(rows)   # 锚点失效的行也是一项（按项机读行）
    rows = [row for row in rows if row.line_number not in anchor_broken]
    try:
        key_context, rows, components, library = plan_rows(root, rows, environment, table_rows)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来：一条都没跑", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return 1
    except release_only_tests_module().RegistryUnreadable as error:
        print_rejection(f"判不了哪几行是重测试目标（{error}），补不了 --release：一条都没跑", [], [RELEASE_REGISTRY_HOWTO])
        return 1
    except (mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print_rejection(f"算不出行键或判定库的位置（{error}），判定库没处对：一条都没跑", [],
                        ["被判的要是 git 工作树（rsync 拷的副本先在副本根 git init；只给自证换的几个变量只在临时 prune-server 底下认）、根 Cargo.toml 读得懂；",
                         f"单跑 python3 scripts/mutation/crates-mutation-rows.py plan {root} 看它报什么。"])
        return 1
    # 用时表与占盘记录是运行记录（文件头「判定库」末段）：放共享根下，不进判定库、不落到 git common-dir
    try:
        run_records = run_records_directory(root, environment)
        timings_directory = os.path.join(run_records, "timings")
        os.makedirs(timings_directory, exist_ok=True)
    except (admission.InputManifestError, OSError) as error:
        print_rejection(f"用时表与占盘记录没处放（{error}），一条都没跑", [],
                        ["多机配置（仓根 multi-host.env，或 SINGLEFS_MULTI_HOST_CONFIG 指的那一份）写上 NFS_SHARED_ROOTS（两台同一绝对路径的共享根）；不补缺省。"])
        return 2
    # 真跑测试的每一次都要把多机配置交给测试（文件头「真跑测试的格把多机配置交给测试」）：那份文件不在，起任何 cargo 之前就拒（配置错，不当本次未跑）。
    #   弄坏开关 test-runs-configuration-unpassed 下不判
    configuration_path, configuration_present = test_runs_multi_host_configuration(environment)
    if not configuration_present and not break_is(environment, "test-runs-configuration-unpassed"):
        print_rejection(f"多机配置 {configuration_path} 不在：真跑测试的每一次都要把它交给测试（副本不是 git 工作树，测试自己找不到，crates 那侧找不到就报错），一条都没跑", [],
                        ["照仓根 multi-host.env.example 在主工作树根写一份 multi-host.env，或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；不补缺省。"])
        return 2
    skip_baseline = break_is(environment, "nobaseline")
    snapshot = snapshot_identity(snapshot_of(root, copied)) if break_is(environment, "baseline-by-tree-snapshot") else None
    decided = decide_rows(library, rows, components, key_context, environment)
    decisions = {line_number: decision for line_number, (decision, _record) in decided.items()}
    sentinel = plan_reuse_sentinel(library, rows, components, environment)
    reused, baseline_ok_lines, reused_baseline_count = {}, set(), 0
    progress = merge_step_open(environment, "核复用与基线", len(rows))
    for done, row in enumerate(rows, 1):
        merge_step_row_done(progress, done)
        decision, record = decided[row.line_number]
        if decision.reuse and break_is(environment, "reuse-lookup-skipped"):
            decision = decisions[row.line_number] = Decision(False, "unexplained", "弄坏开关 reuse-lookup-skipped：没读库")
        if decision.reuse and sentinel.forced(sentinel_item(row)):
            decision = decisions[row.line_number] = Decision(False, "sentinel", "复用哨兵抽中：键相同，这一趟不读库")
        if decision.reuse:
            try:
                removal = (int(record.get("removal_files", "")), int(record.get("removal_allocated_bytes", "")),
                           int(record.get("removal_apparent_bytes", "")), record.get("removal_left") == "1")
            except ValueError:
                decision = decisions[row.line_number] = Decision(False, "unexplained", "记录里删临时目录的那几个数读不出")
        if decision.reuse:
            reused[row.line_number] = (row.line_number, "caught", caught_text(row, "（复用）"), removal)
            baseline_ok_lines.add(row.line_number)
            continue
        ok, _problem = baseline_check(library, row, components[row.line_number], key_context, snapshot, environment)
        if ok:
            baseline_ok_lines.add(row.line_number)
            reused_baseline_count += 1
    merge_step_close(progress, f"复用抓到 {len(reused)} 行、复用基线 ok {reused_baseline_count} 行")
    refused_lines = {row.line_number for row in rows if components[row.line_number].row_key is None}
    # 键对不上的行只在分片里现判（文件头「分片」）：门禁不是分片的一片时设 GATE_MUTATION_SHARDED_ONLY=1，库里没有键相同记录的行这一趟不现判、整道判红；
    #   拒算的（没有键）、复用哨兵抽中的、键相同而上次没判成抓到的照判。弄坏开关 unsharded-rows-judged 下照现判
    if environment.get("GATE_MUTATION_SHARDED_ONLY") == "1" and not break_is(environment, "unsharded-rows-judged"):
        unsharded = [row for row in rows if row.line_number not in reused and decisions[row.line_number].column not in ("refused", "sentinel", "not-caught")]
        if unsharded:
            print_rejection(f"{len(unsharded)} 行在判定库里没有键相同的记录，这一趟不现判（键对不上的行只在分片里判），一条都没跑：",
                            [f"{TABLE}:{row.line_number} {row.name}：{COLUMN_TITLES[decisions[row.line_number].column]}" for row in unsharded[:REASONS_SHOWN]]
                            + ([f"……另 {len(unsharded) - REASONS_SHOWN} 行（逐行：python3 scripts/mutation/crates-mutation-rows.py plan {root}）"] if len(unsharded) > REASONS_SHOWN else []),
                            ["经统一任务服务交这一道：task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh [--item <行号或变异名>]，服务按分片协议拆成计划、两片、合并，"
                             "两片现判、合并那一步照键复用判；",
                             "合并那一步报这一句的，是分片判的行没写进服务里那一份库：看合并那一步上面「双机分片」那几行点名的片。"])
            return 1
    candidates = [row for row in rows if row.line_number not in reused]
    if skip_baseline:
        baseline_ok_lines |= {row.line_number for row in candidates}
    needs_baseline = [row for row in candidates if row.line_number not in baseline_ok_lines]
    batches, unknown_baseline_notes, long_baseline_notes = baseline_chunks(needs_baseline, key_context.release_registered, environment, timings_directory)
    ordered_candidates, moved_count = dispatch_order(candidates, timings_directory, numeric_settings["GATE_MUTATION_LONG_ROW_SECONDS"], environment)
    planner = CoveragePlanner(components, library, key_context)
    if narrowed_unrecorded:   # 缩过域的一趟不记覆盖（文件头「缩过域的一趟」）：缩域下执行过的单元偏少，记了也不入库
        planner.predicted_rows = lambda _rows: []
        planner.job_after = lambda _row, _judgement, _info: None
    # 组成清单先进库（点名测试的闭包）：这一趟中途停了，下一趟照样报得出变了哪几个单元
    key_context.persist(library, components)
    # stderr 头一行：复用几条、现跑几条、前移几条（逐栏的数在 stdout 的「复用：」那一行）
    note(f"  … 判定库记录：复用抓到 {len(reused)} 条、复用基线 ok {reused_baseline_count} 行；现跑基线 {len(batches)} 批 {len(needs_baseline)} 行（其中长测试独占 {len(long_baseline_notes)} 批、没有用时独占 {len(unknown_baseline_notes)} 批）、"
         f"现跑变异至多 {len(candidates)} 条（其中拒算 {len(refused_lines)} 条）、锚点失效不跑 {len(anchor_judgements)} 行、补记覆盖 0 份；前移 {moved_count} 条（上一次用时 ≥ {numeric_settings['GATE_MUTATION_LONG_ROW_SECONDS']} 秒"
         f"的先派，按用时从长到短）；库在 {library.directory}"
         + ("；基线没跑：GATE_MUTATION_BREAK=nobaseline（只给证红用）" if skip_baseline else ""))
    if narrowing:
        note(f"  … 缩过域（{narrowing}）：这一趟不算门禁通过；现判的判定、基线与覆盖"
             + ("都不写进判定库、不记覆盖" if narrowed_unrecorded else "照写进判定库（弄坏开关 narrowed-verdicts-recorded）")
             + "，复用照库里全域下判的抓到")
    for line in sentinel.plan_lines:
        note(line)
    # 按项机读行（文件头同名一段）：复用的与锚点失效的项开跑时就有了结局，现判的项在 run_pool 里一行行判完时报
    item_progress = ItemProgress(selected_rows, environment)
    for judgement in anchor_judgements:
        item_progress.settle(judgement[0], "anchor")
    for line_number in reused:
        item_progress.settle(line_number, "caught", reused=True)
    # 多机配置开了 GPU、点名测试走崩溃判段的行补了 gpu 特性（文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：逐行点名
    if key_context.gpu_route_lines:
        note(f"  … 多机配置 {key_context.gpu_route_configuration} 开了 GPU（ENABLE_GPU=1）：{len(key_context.gpu_route_lines)} 行点名的测试走崩溃判段、cargo 参数没开 gpu，"
             "这一趟补上 gpu 特性起（键照补了之后的特性算；真跑测试的那几次交多机配置）："
             + "、".join(f"{TABLE}:{line_number}（--features {feature}）" for line_number, feature in key_context.gpu_route_lines))
    for line_number, why in key_context.gpu_route_unbuilt:   # 补了 gpu 特性点名的测试就不编（判的是没编 gpu 的二进制）：照原样跑
        note(f"  … 多机配置开了 GPU，{TABLE}:{line_number} 点名的测试走崩溃判段，补 gpu 特性之后不编（{why}），照原样跑")
    # 命中重测试目标而没带 release 的行补了 --release（文件头「重测试目标只许 release」）：报行数，点名前 REASONS_SHOWN 行与依据
    if key_context.release_lines:
        shown = key_context.release_lines[:REASONS_SHOWN]
        note(f"  … 重测试目标只许 release：{len(key_context.release_lines)} 行命中而 cargo 参数没带 release，这一趟补上 --release 起（键照补了之后的参数算）："
             + "、".join(f"{TABLE}:{line_number}（{'；'.join(basis)}）" for line_number, basis in shown)
             + (f"，另 {len(key_context.release_lines) - len(shown)} 行没列" if len(key_context.release_lines) > len(shown) else "")
             + "；变异表那几行的第五段补上 --release 之后不再补（static-check 报 RELEASE 的就是它们）")
    # 现判与拒算的原因逐行进 stderr（作废时报出变了哪几个单元，文件头「复用判据」）；多了只列前 REASONS_SHOWN 行，全部的看 plan
    fresh_reasons = [row for row in rows if row.line_number not in reused and row.line_number not in refused_lines]
    for row in fresh_reasons[:REASONS_SHOWN]:
        decision = decisions[row.line_number]
        note(f"  · 现判：{TABLE}:{row.line_number} {row.name}：{COLUMN_TITLES[decision.column]}——{decision.detail}")
    refused_rows = [row for row in rows if row.line_number in refused_lines]
    for row in refused_rows[:REASONS_SHOWN]:
        refused_reason = components[row.line_number].refusal or ""
        if refused_reason.startswith(COLLISION_REFUSAL_PREFIX):   # 撞键（refuse_colliding_row_keys）：「拒算（行键撞键：与第 N 行同键）」
            note(f"  · 拒算（{refused_reason}）：{TABLE}:{row.line_number} {row.name}；照跑、不复用")
        else:
            note(f"  · 拒算（照跑、不复用）：{TABLE}:{row.line_number} {row.name}：{refused_reason}")
    if len(fresh_reasons) > REASONS_SHOWN or len(refused_rows) > REASONS_SHOWN:
        note(f"  · 现判 {len(fresh_reasons)} 行、拒算 {len(refused_rows)} 行，各只列了前 {REASONS_SHOWN} 行；逐行的栏与变了的单元：python3 scripts/mutation/crates-mutation-rows.py plan {root}")
    outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
    worker_count, cargo_jobs, memory_text = 0, "-", "没有要现跑的行，没开工作进程"
    impact_text = impact_summary(rows, components, key_context, environment, len(anchor_judgements))   # 要闭包，在派工作进程之前算好（文件头「算键的内存」）
    if candidates:
        tools, tools_text, tools_problem = llvm_tools(root, environment)
        if tools_problem is not None:
            print_rejection(f"记覆盖要的 llvm 工具不对（{tools_problem[0]}），一条变异都没跑：", [],
                            [tools_problem[1], "不退回不记覆盖去跑：没有覆盖记录，基线就只能按点名测试的闭包复用。"])
            return 1
        note(f"  … 记覆盖用的 llvm 工具：{tools_text}")
        # 编译目录所在的盘放得下几个工作进程的两份编译目录（文件头「磁盘余量」）：算在核数、条数两项之后，取最小。
        # 调用方没给 GATE_MUTATION_TARGET_DIR 时编译目录由 run_pool 在这一趟的 shard_root 里建，这里按它将来所在的盘（$TMPDIR）算
        shared_target = default_shared_target_directory(environment)
        disk_probe = shared_target if shared_target is not None else planned_run_build_target_directory()
        # 每份编译目录的涨幅按运行记录目录里近 20 次每格判完量到的占盘峰值算（文件头「磁盘余量」），没有读数的那一种按上限
        target_peaks = measured_target_peaks(run_records, 20)
        # 开几个工作进程按这一条拿到的内存上限 ÷ 那个编译并行度下每个工作进程的实测峰值收（文件头「内存上限」）：撞内存上限由服务放大重排、从头跑，开满线程数只会一次次撞顶。
        #   排在另外三项之后（天花板含磁盘那一项）；编译并行度与下面设 CARGO_BUILD_JOBS 同用 cargo_jobs_for
        #   每个工作进程的峰值只取这一趟要现跑的行点名的包里最大的（worker_peak_packages）
        #   这一趟写峰值记录时从没量过的包不进这一项，派活时由探针收住（文件头「内存上限」的「探针」）；不写的时候（门禁样本根）照保守值进这一项
        peak_packages = worker_peak_keys([row.arguments for row in candidates])
        peaks_recorded = worker_peaks_recorded_for(root, environment)
        memory_results = []
        # 按这一台能给的内存除（文件头「内存上限」）：取不到这一台那一项时按这一条拿到的上限除。
        #   起这一道的父进程自己的用量同在一个 scope 里：先从这一台能给的内存里减掉（量不了按 0），撞内存上限那一步再加回去比
        import task_memory
        parent_bytes = task_memory.process_tree_usage(os.getpid(), self_only=True) or 0
        capacity = host_memory_capacity(environment)
        if capacity[0] is not None:
            capacity = (max(0, capacity[0] - parent_bytes), f"{capacity[1]}，减去起这一道的父进程自己 {format_size(parent_bytes)}")

        # 另外三项（线程数、条数、磁盘余量）的小者：跑着加开工作进程的上界（文件头「内存上限」的「跑着加开」）
        growth_ceiling = []

        def memory_bound(ceiling):
            growth_ceiling.append(ceiling)
            memory_results.append(memory_bound_probing(environment, ceiling, thread_budget, run_records, peak_packages, peaks_recorded,
                                                       CGROUP_LISTING, CGROUP_ROOT, None, capacity))
            return memory_results[-1][:2]
        try:
            thread_budget = compile_thread_budget(environment)
            worker_count, worker_count_head_line = worker_count_for(len(candidates), environment,
                                                                    disk=lambda ceiling: disk_bound_for(disk_probe, environment, ceiling, target_peaks),
                                                                    memory=memory_bound)
        except HeavyThreadDefaultError as error:
            print_rejection(f"取不到每台默认重活线程数（{error}），定不了开几个工作进程与每个 cargo 的编译并行度，一条变异都没跑：", [],
                            [error.howto, "或给 GATE_MUTATION_WORKERS=<正整数> 与 GATE_MUTATION_CARGO_JOBS=<正整数> 指定工作进程数与编译并行度；不退回按核数算。"])
            return 2
        # 撞内存上限（文件头「内存上限」的「撞内存上限」）：要开的工作进程 × 量到的峰值比这一条拿到的上限大（一个都放不下，或按这一台能给的内存开满而服务给的上限装不下）。
        #   先往下减：编译并行度留在要开的个数那一档（量过的那一档；个数减了编译并行度跟着涨，涨上去的那一档常常没量过），从要开的个数往下找装得下的最大个数，
        #   减到 1 个为止；找到就照那个数开、照跑，头一行写因为内存从几个减到了几个。弄坏开关 memory-cap-exits-before-fewer-workers：不往下减（照 1 个也装不下办）。
        #   1 个也装不下：照开 1 个起跑，头一行写估值与上限；真撞了上限由统一任务服务判（memory-cap-hit，乘 1.5 重排），不自己先退。
        #   弄坏开关 memory-cap-short-self-exit：1 个也装不下就按 memory-cap-hit 的退出码 250 自己退。
        #   250 写在这里、不做模块级常量
        shortfall_text, held_jobs = "", None
        if memory_results and memory_results[-1][2] is not None and memory_results[-1][3] is not None:
            hard_cap, requirement = memory_results[-1][2], memory_results[-1][3]
            wanted_bytes, wanted_text = requirement(worker_count)
            if wanted_bytes is not None:
                wanted_jobs = cargo_jobs_for(environment, worker_count, thread_budget)
                peak = wanted_bytes // worker_count

                def needed_for(count):
                    needed, needed_text = count * peak, (wanted_text if count == worker_count else
                                                         f"{count} × 每个工作进程的峰值 {format_size(peak)}（编译并行度留在 {wanted_jobs}）= {format_size(count * peak)}")
                    # 按这一台能给的内存除时，起这一道的父进程（这一层 python）自己的用量同在这一个 scope 里，一并算进去（除的时候已从这一台能给的内存里减掉）
                    if capacity[0] is not None:
                        needed, needed_text = needed + parent_bytes, f"{needed_text}，加起这一道的父进程自己 {format_size(parent_bytes)}"
                    return needed, needed_text

                needed, needed_text = needed_for(worker_count)
                if needed > hard_cap:
                    fitting = 0 if break_is(environment, "memory-cap-exits-before-fewer-workers") else next(
                        (count for count in range(worker_count - 1, 0, -1) if needed_for(count)[0] <= hard_cap), 0)
                    if fitting:
                        # 减到 fitting 个之后编译并行度照 fitting 个重取（整台线程数 ÷ fitting，cargo_jobs_for）：重取的那一档（或更高的）有实测、
                        # fitting × 那一档的峰值（加父进程自己）装得下才用，不然留在减之前那一档（那一档没量过，借不到）。弄坏开关 worker-jobs-held-after-memory-cut：一律留在减之前那一档
                        rederived_jobs = cargo_jobs_for(environment, fitting, thread_budget)
                        rederived_bytes, rederived_text = requirement(fitting) if rederived_jobs != wanted_jobs else (None, "")
                        if rederived_bytes is not None and capacity[0] is not None:
                            rederived_bytes, rederived_text = rederived_bytes + parent_bytes, f"{rederived_text}，加起这一道的父进程自己 {format_size(parent_bytes)}"
                        rederived_fits = (rederived_bytes is not None and rederived_bytes <= hard_cap
                                          and not break_is(environment, "worker-jobs-held-after-memory-cut"))
                        fitting_text = (f"{rederived_text}；编译并行度照 {fitting} 个重取成 {rederived_jobs}，不留在减之前的 {wanted_jobs}" if rederived_fits
                                        else needed_for(fitting)[1])
                        reduced_text = (f"因为内存从 {worker_count} 个减到了 {fitting} 个：这一条拿到的上限 {format_size(hard_cap)} 装不下 {worker_count} 个（{needed_text}），"
                                        f"装得下 {fitting} 个（{fitting_text}），照 {fitting} 个开、照跑")
                        worker_count_head_line = worker_count_head_line.replace(
                            f"这一轮开 {worker_count} 个工作进程（", f"这一轮开 {fitting} 个工作进程（{reduced_text}；减之前", 1)
                        shortfall_text = f"；{reduced_text}"
                        worker_count, held_jobs = fitting, (None if rederived_fits else wanted_jobs)
                    elif break_is(environment, "memory-cap-short-self-exit"):
                        # 弄坏开关 memory-cap-short-self-exit：1 个也装不下就自己按撞内存上限退 250、不开工
                        if worker_count == 1:
                            headline = f"内存上限装不下一个工作进程的峰值（{memory_results[-1][1]}）"
                        else:
                            headline = (f"这一条拿到的上限 {format_size(hard_cap)} 装不下要开的 {worker_count} 个工作进程（{needed_text}；{memory_results[-1][1]}），"
                                        f"减到 1 个也装不下（{needed_for(1)[1]}）")
                        note(worker_count_head_line)
                        print_rejection(f"{headline}，按撞内存上限退出（退 250，统一任务服务错误类型表里 memory-cap-hit 那一行），一条变异都没跑：", [],
                                        ["经 task-server/task submit 交的，服务乘 1.5 重排，不用改命令；到顶的由服务打回 memory-cap-at-maximum，照它的出路收小范围或分片；手跑的给更大的内存上限再跑。"])
                        return 250
                    else:
                        # 1 个也装不下：照开 1 个起跑，真撞了上限由统一任务服务判（memory-cap-hit 乘 1.5 重排）。
                        # 脚本永远不报自己要多少内存（用户 2026-11-13 定） history-kept: 用户指定的注释原文
                        #   这一行只写估值与这一条拿到的上限，不是向服务要内存；编译并行度留在要开的个数那一档（量过的那一档）
                        short_text = ((f"内存上限装不下一个工作进程的估值：这一条拿到的上限 {format_size(hard_cap)}、{needed_text}" if worker_count == 1 else
                                       f"内存从 {worker_count} 个减到 1 个也装不下：这一条拿到的上限 {format_size(hard_cap)} 装不下 {worker_count} 个（{needed_text}），"
                                       f"减到 1 个也装不下（{needed_for(1)[1]}）")
                                      + "；照开 1 个起跑、不自己退 250，真撞了上限由统一任务服务判（memory-cap-hit，乘 1.5 重排）")
                        worker_count_head_line = worker_count_head_line.replace(
                            f"这一轮开 {worker_count} 个工作进程（", f"这一轮开 1 个工作进程（{short_text}；{'' if worker_count == 1 else '减之前'}", 1)
                        shortfall_text = f"；{short_text}"
                        worker_count, held_jobs = 1, wanted_jobs
        note(worker_count_head_line)
        memory_text = (memory_results[-1][1] if memory_results[-1][0] is not None
                       else f"没有内存上限这一项（{memory_results[-1][1]}），照线程数") + shortfall_text
        # 每个 cargo 的编译并行度（文件头「编译并行度」）：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢；与内存那一项试进程数时同一个函数。
        #   因为内存往下减了个数的，编译并行度留在减之前那一档（上面「撞内存上限」）
        cargo_jobs = str(held_jobs if held_jobs is not None else cargo_jobs_for(environment, worker_count, thread_budget))
        environment["CARGO_BUILD_JOBS"] = cargo_jobs
        # 起 cargo 时关调试信息、release 档开增量（文件头「调试信息与 release 增量」）：只设在起 cargo 的这一份环境里；这两项的取值不进行键
        release_incremental_of_the_caller = environment.get("CARGO_PROFILE_RELEASE_INCREMENTAL")   # 基线起 cargo 时放回去的（baseline_cargo_environment）
        debug_overrides = cargo_debug_overrides(environment)
        environment.update(debug_overrides)
        note("  … 起 cargo 时的构建档：" + ("、".join(f"{name}={value}" for name, value in sorted(debug_overrides.items())) + "（关调试信息、release 开增量，取值不进行键；基线不带 release 增量这一项）"
                                       if debug_overrides else "照调用方的环境与 Cargo.toml（弄坏开关 debuginfo-kept、release-incremental-unset 都开着）"))
        note("  … 起 cargo 时的崩溃判定：基线连调用方的那一份判定库服务（读写，判过的块按键复用）；变异与记覆盖的插桩跑各起自己的临时 prune-server，判出的结论不进那一份"
             "（文件头「基线连调用方的判定库服务」「变异复跑的崩溃判定」）")
        # 这一趟自己建的由 run_pool 建好之后打那一行
        if shared_target is not None:
            for line in target_reclaim_lines(shared_target, worker_count, reclaim_extra_target_directories(shared_target, worker_count, environment), owned=False):
                note(line)
        context = dict(root=root, copied=copied, environment=environment, release_incremental_of_the_caller=release_incremental_of_the_caller,
                       llvm_tools=tools,
                       library_directory=library.directory, run_records_directory=run_records, timings_directory=timings_directory,
                       row_records={row.line_number: light_components(components[row.line_number]) for row in rows},
                       shared_target_directory=shared_target, worker_count=worker_count, stage_process=os.getpid(),
                       worker_peaks_recorded=peaks_recorded, narrowing=narrowing, long_baseline_notes=long_baseline_notes, unknown_baseline_notes=unknown_baseline_notes,
                       target_directory_cap_bytes=target_directory_cap_bytes(environment),
                       # 探针（文件头「内存上限」的「探针」）：有内存上限这一项、这一趟写峰值记录时，run_pool 派每一格前按实测峰值判放不放得下
                       memory_admission=(dict(cap=memory_results[-1][2], jobs=int(cargo_jobs), packages=peak_packages)
                                         if peaks_recorded and memory_results and memory_results[-1][2] is not None else None),
                       # 跑着加开（文件头「内存上限」的「跑着加开」）：同探针一个前提，另要开的个数没到另外三项的小者；上界是那三项的小者，比的是这一条拿到的上限、加父进程自己。
                       #   弄坏开关 workers-not-grown-after-measuring：不加开
                       worker_growth=(dict(cap=memory_results[-1][2], parent_bytes=parent_bytes, ceiling=growth_ceiling[-1], thread_budget=thread_budget)
                                      if (peaks_recorded and memory_results and memory_results[-1][2] is not None and growth_ceiling
                                          and growth_ceiling[-1] > worker_count and not break_is(environment, "workers-not-grown-after-measuring")) else None))
        RUN_BUILD_REMOVAL_FAILURES.clear()   # 这一趟 run_pool 收尾没删成的编译目录（同一进程里跑几趟时不带上一趟的）
        note(release_key_context(key_context, components, environment))
        try:
            outcome = run_pool_reporting_dead_worker(context, worker_count, batches, ordered_candidates, baseline_ok_lines,
                                                     numeric_settings["GATE_MUTATION_STOP_AFTER_ROWS"], planner, item_progress)
        finally:
            gc.unfreeze()
        if outcome is None:
            return 1
        # 跑着加开过的（文件头「内存上限」的「跑着加开」）：收尾一行的「按内存」接上从几个加到几个；「这一轮 N 个工作进程」写的仍是开跑时的个数
        if outcome.get("worker_growth"):
            memory_text += f"；{outcome['worker_growth']}"
        if RUN_BUILD_REMOVAL_FAILURES:
            print_rejection(f"这一趟自己建的编译目录收尾没删成（{'、'.join(RUN_BUILD_REMOVAL_FAILURES)}），整道判红", [],
                            ["照上面那几行报的进程逐个停掉（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>），再删那个目录；",
                             "是这一趟的工作进程或 cargo 没收干净：去看 scripts/mutation/crates-mutation-rows.py 的 run_pool 收尾。"])
            return 1
        if outcome.get("seed_problems"):
            print_rejection(f"先编一份依赖：源码副本里摸过的包有 {len(outcome['seed_problems'])} 个产物 cargo 当成没变（fresh），修改时刻规矩坏了，一条都没跑",
                            outcome["seed_problems"],
                            ["去看 scripts/mutation/crates-mutation-rows.py 的 sync_seed_source（内容变了的文件要写成此刻的修改时刻、所在的包整包摸过）；",
                             f"修好之前删掉 {os.path.normpath(context['shared_target_directory'] or planned_run_build_target_directory()) + SEED_SOURCE_SUFFIX} 再跑（整份重拷，修改时刻都是此刻）。"])
            return 1
        if outcome.get("prune_server_problem"):
            print_rejection(f"变异复跑要各起一份临时 prune-server，服务的二进制取不到（{outcome['prune_server_problem']}），一条都没跑", [],
                            [f"设 {PRUNE_SERVER_BINARY_VARIABLE}=<编好的 prune-server 的绝对路径> 再跑；或修好 prune-server/ 的编译"
                             "（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server，经 task-server/task submit 交）。"])
            return 1
    item_progress.finish()
    for problem in outcome["record_problems"]:
        note(f"  ! 判定库记录或用时表没写成（下一趟这一条照跑，不会复用）：{problem}")
    # 派出去多少格就要收回来多少格：对不上整道红，不许少跑一条还报绿（command-safety.md）
    if outcome["submitted"] != outcome["received"]:
        print_rejection(f"派出去 {outcome['submitted']} 格（基线的批、变异与覆盖），只收回 {outcome['received']} 格", [],
                        ["这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，",
                         "是就去查领活与收束那一段（scripts/mutation/crates-mutation-rows.py 的 run_pool）。"])
        return 1
    if outcome.get("over_cap"):
        print_rejection(f"编译目录清掉增量产物与 deps 里同名产物的旧版本之后仍超上限 {TARGET_DIRECTORY_CAP_VARIABLE}，停派：在跑的那几格判完就没再派活，"
                        f"这一轮没跑全（现判的变异判完 {len(outcome['mutation'])} 条）", outcome["over_cap"],
                        [f"剩下的是每个依赖与测试二进制的最新一版：把 {TARGET_DIRECTORY_CAP_VARIABLE} 调大到放得下这一趟的一份编译目录（文件头「编译目录上限」），",
                         "或给编译目录换一块余量大的盘（GATE_MUTATION_TARGET_DIR）；判完的那几格已经写了判定库记录，再跑一趟只跑剩下的。"])
        return 1
    if outcome["stopped"]:
        print_rejection(f"只供测试的开关 GATE_MUTATION_STOP_AFTER_ROWS={numeric_settings['GATE_MUTATION_STOP_AFTER_ROWS']}："
                        f"现跑的变异判完 {len(outcome['mutation'])} 条就没再派活，这一轮没跑全", [],
                        ["这个开关只给自证造「跑到一半被杀」用，门禁里别设它；判完的那几条已经写了判定库记录，去掉它再跑一趟只跑剩下的。"])
        return 1
    # 索引：这一趟复用的与现判出结果的每一行，记下它此刻的键与几样组成的号（下一趟键对不上时照它报变了哪几个单元）
    judged_lines = {judgement[0] for judgement in outcome["mutation"] if judgement[1] != "anchor"} | set(reused)
    for row in rows:
        if row.line_number in judged_lines and components[row.line_number].row_key and not narrowed_unrecorded:   # 缩过域的一趟索引也不写（文件头「缩过域的一趟」）
            library.write("index", row_identity(row), index_entry(components[row.line_number], row))
    rows_by_line = {row.line_number: row for row in rows}
    fresh = []
    for line_number, verdict, text, removal in outcome["mutation"]:
        if verdict == "caught":
            how = "（拒算）" if line_number in refused_lines else f"（现判：{COLUMN_TITLES[decisions[line_number].column]}）"
            # 缩过域的一趟里现判出的结局（文件头「缩过域的一趟」）：抓到那一句的缩过域接在「（现判：…）」之前，那个标照旧在行尾
            text = narrowed_outcome_text(caught_text(rows_by_line[line_number], ""), narrowing) + how if narrowing else caught_text(rows_by_line[line_number], how)
        elif narrowing and verdict != "anchor":
            text = narrowed_outcome_text(text, narrowing)
        fresh.append((line_number, verdict, text, removal))
    judgements = list(reused.values()) + fresh + anchor_judgements
    for row in candidates:
        problem, tail = outcome["baseline"].get(row.line_number, (None, ""))
        if problem is not None:
            baseline_text = f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}\n{tail}"
            judgements.append((row.line_number, "baseline", narrowed_outcome_text(baseline_text, narrowing) if narrowing else baseline_text, None))
    judged_lines = {judgement[0] for judgement in judgements}
    missing = [row for row in rows if row.line_number not in judged_lines]
    if missing:
        print_rejection(f"有 {len(missing)} 行一个判定都没有（没有作数的记录、基线也没判出它来）", [f"{TABLE}:{row.line_number} {row.name}" for row in missing],
                        ["这是判定库记录与派活自己的完整性闸红了，不是变异的问题；再跑一遍，还缺就去查 run_stage 与 run_pool 那一段。"])
        return 1
    fresh_columns = collections.Counter(decisions[row.line_number].column for row in rows if row.line_number not in reused and row.line_number not in refused_lines)
    coverage_failures = [result for result in outcome["coverage"] if result["problem"] is not None]
    for result in coverage_failures:
        note(f"  ! 覆盖没记成（组首行 {TABLE}:{result['line']}；下一趟这一条测试的基线只按它的闭包复用）：{result['problem']}")
    coverage_recorded = sum(1 for result in outcome["coverage"] if result["coverage"] and not result.get("existing"))
    coverage_existing = sum(1 for result in outcome["coverage"] if result.get("existing"))
    note(f"  … 覆盖：这一趟记了 {coverage_recorded} 份、已有沿用 {coverage_existing} 份、没记成 {len(coverage_failures)} 份")
    for line_number in reused:
        sentinel.observe(sentinel_item(rows_by_line[line_number]), reuse_checks.REUSED, "抓到")
    observe_fresh_outcomes(sentinel, outcome["mutation"], rows_by_line, environment)
    sentinel_problems = sentinel.finish(printer=note)
    exit_code = report(rows, judgements, outcome, selection_path, skip_baseline, dict(
        worker_count=worker_count, cargo_jobs=cargo_jobs, memory_text=memory_text, reused=len(reused), reused_baseline=reused_baseline_count,
        fresh_columns=fresh_columns, fresh_rows=len(candidates) - len(refused_lines), refused_rows=len(refused_lines),
        collision_rows=[(row.line_number, components[row.line_number].refusal) for row in rows
                        if (components[row.line_number].refusal or "").startswith(COLLISION_REFUSAL_PREFIX)],
        coverage_recorded=coverage_recorded, coverage_existing=coverage_existing, coverage_failed=len(coverage_failures),
        fresh_mutations=len(outcome["mutation"]), fresh_batches=len(batches), fresh_long_batches=len(long_baseline_notes), fresh_unknown_batches=len(unknown_baseline_notes), records_directory=library.directory,
        not_run_record=environment.get("GATE_MUTATION_NOT_RUN_RECORD", "").strip(),
        impact=impact_text, narrowing=narrowing))
    for message, howto in sentinel_problems:
        print_rejection(message, [], [howto])
    return 1 if sentinel_problems else exit_code


def sentinel_item(row):
    """复用哨兵里一行的名字：行号与变异名（变异名可能重名）。"""
    return f"{row.line_number} {row.name}"


def observe_fresh_outcomes(sentinel, mutation_outcomes, rows_by_line, environment):
    """现判那几行的结局交复用哨兵。派活之后才锚点失效的那一行没跑（judge_row），不交；强制现判那一条落在不算数的结局上
    （VERDICTS_UNCOUNTED：本次未跑）也不交，哨兵照「这一趟没报结局」打一行不判（lib_reuse_checks.ReuseSentinel.finish）——
    没跑成的那一次不是现判结论，拿它去比库里的「抓到」会判成不稳。别的行不算数的照交：哨兵被派去现判就已是复用失灵，哨兵看的是状态、不看结论。
    弄坏开关 uncounted-forced-observed 下强制现判那一条不算数的也照交。"""
    for line_number, verdict, _text, _removal in mutation_outcomes:
        if verdict == "anchor":
            continue
        item = sentinel_item(rows_by_line[line_number])
        forced = sentinel.forced(item)
        if forced and verdict in VERDICTS_UNCOUNTED and not break_is(environment, "uncounted-forced-observed"):
            continue
        sentinel.observe(item, reuse_checks.FORCED if forced else reuse_checks.JUDGED, "抓到" if verdict == "caught" else verdict)


def plan_reuse_sentinel(library, rows, components, environment):
    """复用哨兵（文件头「复用三问与复用哨兵」）：候选是此刻行键在 records/ 里有「抓到」记录的行，直接读记录（不经 row_decision）。
    环境里开了分步进度（merge_progress_variable）就把「抽哨兵候选」这一步逐行打进度。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "crates 变异表", environment=environment)
    # 缩过域的一趟不抽（文件头「缩过域的一趟」）：复用照库里全域下判的抓到，强制现判出的缩域结论比不出复用对不对。弄坏开关 narrowed-sentinel-drawn：照抽
    narrowing = crash_domain_narrowing(environment)
    if narrowing and not break_is(environment, "narrowed-sentinel-drawn"):
        sentinel.plan_lines = []
        sentinel.plan({}, reuse=False, off_reason=narrowed_sentinel_off_reason(narrowing), printer=sentinel.plan_lines.append)
        return sentinel
    candidates = {}
    progress = merge_step_open(environment, "抽哨兵候选", len(rows))
    for done, row in enumerate(rows, 1):
        key = components[row.line_number].row_key
        record = library.read("records", key) if key else None
        if record is not None and record.get("row_key") == key and record.get("verdict") == REUSABLE_VERDICT["mutation"]:
            candidates[sentinel_item(row)] = "抓到"
        merge_step_row_done(progress, done)
    merge_step_close(progress, f"候选 {len(candidates)} 行")
    # 抽中的几条先攒着，等 stderr 头一行（「… 判定库记录：」）打完再打（别的定义与自证照头一行读复用几条）
    sentinel.plan_lines = []
    sentinel.plan(candidates, printer=sentinel.plan_lines.append)
    return sentinel


class CoveragePlanner:
    """这一趟要记哪几份覆盖（文件头「判定库」覆盖记录）：一行现判成抓到、它点名的测试此刻还没有一份执行过的单元都没变的覆盖记录，就派一格（同一份覆盖号只派一次）。"""

    def __init__(self, components, library, key_context):
        """执行过的单元照每一行自己的单元键核（key_context.keys_for，文件头「--manifest-path」）。"""
        self.components, self.library, self.key_context = components, library, key_context
        self.scheduled = set()

    def initial_jobs(self):
        return []

    def predicted_rows(self, rows):
        """这一趟现跑之后可能要记覆盖的行（先编一份依赖的插桩那一份按它们的组编，文件头「先编一份依赖」）：与 job_after 同一条判据，只是不看判定、不记 scheduled，
        同一份覆盖号只交头一行。"""
        picked, seen = [], set()
        for row in rows:
            components = self.components.get(row.line_number)
            if components is None or not components.coverage_id or not components.test_full_name or components.coverage_id in seen:
                continue
            seen.add(components.coverage_id)
            if not executed_set_is_current(self.key_context.keys_for(row)[0], self.library.read_json("coverage", components.coverage_id)):
                picked.append(row)
        return picked

    def job_after(self, row, judgement, info):
        if judgement[1] != "caught":
            return None
        components = self.components.get(row.line_number)
        if components is None or not components.coverage_id or components.coverage_id in self.scheduled or not components.test_full_name:
            return None
        coverage = self.library.read_json("coverage", components.coverage_id)
        if executed_set_is_current(self.key_context.keys_for(row)[0], coverage):
            return None
        self.scheduled.add(components.coverage_id)
        before, switches = batch_key(row.arguments)
        return dict(coverage_id=components.coverage_id, before=list(before), switches=list(switches), test=components.test_full_name,
                    arguments=row.arguments, expected=row.expected, line=row.line_number, stale=coverage is not None)


def llvm_tools(root, environment):
    """记覆盖用的 llvm-profdata、llvm-cov：(目录, 「目录与版本」那一句, None)，不对交 (None, None, (卡在哪, 下一步))（文件头「判定库」llvm 工具）。"""
    def run(command):
        try:
            return subprocess.run(command, capture_output=True, text=True, errors="replace", cwd=root, env=environment, stdin=subprocess.DEVNULL)
        except OSError:
            return None
    rustc = run(["rustc", "-vV"])
    if rustc is None or rustc.returncode != 0:
        return None, None, ("rustc -vV 跑不起来", "装好 Rust 工具链（bash .claude/scripts/env.sh 会报缺什么）")
    fields = dict(line.split(": ", 1) for line in rustc.stdout.splitlines() if ": " in line)
    wanted, host = fields.get("LLVM version", ""), fields.get("host", "")
    directory = environment.get(LLVM_TOOLS_VARIABLE, "").strip()
    if not directory:
        sysroot = run(["rustc", "--print", "sysroot"])
        if sysroot is None or sysroot.returncode != 0 or not host:
            return None, None, ("rustc --print sysroot 或 rustc -vV 的 host 取不到", "装好 Rust 工具链再跑")
        directory = os.path.join(sysroot.stdout.strip(), "lib", "rustlib", host, "bin")
    versions = []
    for tool in LLVM_TOOLS:
        path = os.path.join(directory, tool)
        if not os.access(path, os.X_OK):
            return None, None, (f"{path} 不在或不能执行", f"装 rustup 组件：rustup component add llvm-tools（与 rustc 同一份 LLVM）；装在别处就设 {LLVM_TOOLS_VARIABLE}=<目录>")
        answered = run([path, "--version"])
        found = re.search(r"LLVM version (\S+)", answered.stdout if answered is not None else "")
        version = found.group(1) if found else ""
        if not break_is(environment, "llvmversionunchecked") and (not wanted or not (version == wanted or version.startswith(wanted + "-"))):
            return None, None, (f"{tool} 的 LLVM 版本是「{version or '读不出'}」，rustc 的是「{wanted or '读不出'}」",
                                "装与 rustc 同一份的 rustup 组件：rustup component add llvm-tools（rustup update 之后要重装）；"
                                f"{LLVM_TOOLS_VARIABLE} 指的目录换成那一份")
        versions.append(f"{tool} {version}")
    return directory, f"{directory}（{'、'.join(versions)}；rustc 的 LLVM {wanted}）", None



def report(rows, judgements, outcome, selection_path, skip_baseline, counts):
    """按变异表的行号排序打印判定、计数、基线与临时目录，判红的逐栏列出；回退出码。"""
    judgements.sort(key=lambda judgement: judgement[0])
    by_verdict = collections.defaultdict(list)
    for _line_number, verdict, text, _removal in judgements:
        by_verdict[verdict].append(text)
    caught, failures, invalid, baseline_bad = by_verdict["caught"], by_verdict["failure"], by_verdict["invalid"], by_verdict["baseline"]
    not_run = by_verdict["notrun"]
    link_failed = by_verdict["linkfailed"]   # 替换之后失败在链接（verdict_of_run）：单列一档，不记无效、不记没红，这一趟判红（出路是删编译目录重跑）
    anchor_broken = by_verdict["anchor"]   # 锚点失效（文件头「锚点失效」）：rows 里没有这几行，「共」那个数把它们加回来
    for text in caught:
        say(text)
    # 各栏各数各的；「计数：」这一行的格式别的定义与规则照栏读（mutation-triage），基线另起一行报
    say(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、锚点失效 {len(anchor_broken)} 行"
        f"（共 {len({row.line_number for row in rows} | {judgement[0] for judgement in judgements if judgement[1] == 'anchor'})} 条）")
    if skip_baseline:
        say("  基线：没跑（GATE_MUTATION_BREAK=nobaseline，只给证红用）")
    else:
        say(f"  基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 {len(rows) - len(baseline_bad)} 行，基线不绿 {len(baseline_bad)} 行"
            f"（按 cargo 参数分 {len(baseline_batches(rows))} 批；共 {len(rows)} 行）")
    # 复用与覆盖各另起一行（「计数：」那一行的格式不动）；判定本身与进程数无关，这两行随记录变（比对两趟的判定时去掉，selftest_verdict_view）
    counts["not_run_rows"] = len(not_run)
    reuse_text = reuse_summary_text(counts)
    # 成功句点名判定库在哪（文件头「判定库」）：接在「；这一趟现判 …」之后，比对两趟的判定时连它一起去掉（selftest_verdict_view）
    library_text = f"；判定库在 {counts['records_directory']}"
    if counts.get("narrowing"):   # 缩过域的一趟（文件头「缩过域的一趟」）：成功句写实际值、不算门禁通过、现判的没入库
        library_text = f"；缩过域（{counts['narrowing']}）、不算门禁通过、现判的没入库（复用的是库里全域下判的）{library_text}"
    say(f"  复用：{reuse_text}")
    say(f"  覆盖：这一趟记了 {counts['coverage_recorded']} 份（点名测试执行过的单元，基线按它复用），已有、沿用 {counts['coverage_existing']} 份，没记成 {counts['coverage_failed']} 份"
        + ("（逐份的原因在 stderr；那几条测试的基线下一趟只按闭包复用）" if counts["coverage_failed"] else ""))
    # 本次未跑的逐条列出来（文件头「本次未跑」）：不算抓到、不算没红；「计数：」那一行的格式不动，另起一行报
    not_run_record_problem = None
    if not_run:
        say(f"  本次未跑：{len(not_run)} 条（点名的测试在本机打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功，或带 gpu 特性而多机配置不在、给定状态全交 CPU 判，"
            "没跑到被改坏的那一处；或点名的测试红了而它自己那一段输出里、或所有段之外有行首 GPU_DEVICE_LOST 的行，红的是设备丢失、不是断言："
            "不算抓到、不算没红，这一趟不是整张表的结论）")
        for text in not_run:
            say(f"    · {text}")
        not_run_record_problem = write_not_run_record(counts.get("not_run_record", ""), not_run)
    removals = [removal for _line, _verdict, _text, removal in judgements if removal is not None]
    left_behind = outcome["left_behind"]
    removal_state = "每条结束时不论退出码都整个删掉了" if not left_behind else f"跑完有 {len(left_behind)} 个没删掉"
    say(f"  临时目录：{len(removals)} 条变异各一个自己的 TMPDIR，{removal_state}；删前合计 {sum(removal[0] for removal in removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in removals)} 字节（表观 {sum(removal[2] for removal in removals)} 字节）")
    sections = [
        (anchor_broken, f"有几行锚点失效（锚点失效 {len(anchor_broken)} 行；原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了：这几行没跑、不入库、不复用，别的行照判）：",
         ["把那一行的原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），文件挪了就改「文件」那一段；",
          "改完单跑那几行：bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名>（.claude/rules/mutation-sampling.md 第七类）。"]),
        (baseline_bad, f"有几行基线不绿（基线不绿 {len(baseline_bad)} 行；点名的测试在没改坏的副本上就不是恰好一个 ok，改坏之后红了也证明不了什么，这几行的变异没跑）：",
         ["先看点名的测试读的文件在不在拷贝范围里（.claude/gate.d/stage-inputs.tsv 本阶段那一行；bash .claude/gate.d/code-source-discipline.sh 静态判这一样），",
          "不在就登记进那一行；再在真仓上单跑它（照那一行的 cargo test 参数），真仓上也红就是测试本来就坏，先修测试；对上多个用例的把点名写全到模块路径。"]),
        (invalid, f"有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：",
         ["先看尾巴分来源，不是去补用例。有 process didn't exit successfully 与信号的，是测试进程被杀，先查是谁杀的；点名的名字里带 $ 这类字符的，改那一行的测试名；其余是替换文编不过——",
          "先看反斜杠：表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；",
          "别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。"]),
        (link_failed, f"有变异链接失败（链接失败 {len(link_failed)} 条；替换之后编过了、失败在链接，没判到被改坏的那一处，不是替换文编不过，也不是没红）：",
         ["多半是编译目录里别的命令编出的旧产物与这一趟对不上（链接器报未定义的符号）：换一份干净的编译目录重跑这一行：经服务交的编译目录按树给、task clean 不删它（task-server/README.md「编译目录」），在一份新建的仓副本里交（另一棵树拿另一组编译目录），本机手跑的删掉 CARGO_TARGET_DIR 指的目录再跑；",
          "删掉重跑还是链接失败，才是替换文改出来的链接问题，照无效那一类去看替换文。"]),
        (failures, f"有变异没红（没红 {len(failures)} 条）：",
         ["没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。"]),
    ]
    for items, headline, howto_lines in sections:
        if items:
            print_rejection(headline, items, howto_lines)
    if left_behind:
        print_rejection(f"有 {len(left_behind)} 个临时目录跑完没删掉（{outcome['per_mutation_temporary_root']} 底下还剩这几个；line-<行号> 是变异、baseline-<行号> 是那一批基线，行号是 {TABLE} 的）：",
                        [f"{name}：里面 {len(outcome['left_behind_contents'][name])} 项，前几项 {'、'.join(outcome['left_behind_contents'][name][:5]) or '（空）'}"
                         for name in left_behind],
                        ["看 judge_row / judge_batch 末尾 measure_and_remove_temporary_directory 那一步跑到没有、删不删得掉（只读的目录、没退干净的进程又把它建回来）；",
                         "这一道收尾时连同源码副本一起删掉，现场只剩上面列的名字。别拿掉这一步：被杀的测试 Drop 守卫不跑，它建的镜像全堆在这一轮的 TMPDIR 里，一轮能堆几百 G。"])
    if not_run_record_problem:
        print_rejection(f"本次未跑的 {len(not_run)} 条没写进 GATE_MUTATION_NOT_RUN_RECORD 那份记录（{not_run_record_problem}），门禁汇总就看不见它们", [],
                        ["看那份文件所在的目录满没满、有没有写权限；记不下本次未跑的这一趟按红记，免得被当成全判过。"])
    red = bool(invalid or failures or baseline_bad or left_behind or not_run_record_problem or anchor_broken
               or (link_failed and not break_is(os.environ, "link-failure-not-red")))
    note(f"  … 这一轮 {counts['worker_count']} 个工作进程，每个 cargo 编译并行度 {counts['cargo_jobs']}，不设上限与限时（内存与卡住归统一任务服务）；"
         f"按内存：{counts['memory_text']}；"
         f"复用抓到 {counts['reused']} 条、复用基线 ok {counts['reused_baseline']} 行，现跑变异 {counts['fresh_mutations']} 条、现跑基线 {counts['fresh_batches']} 批（其中长测试独占 {counts['fresh_long_batches']} 批、没有用时独占 {counts['fresh_unknown_batches']} 批）")
    if selection_path:
        # 只判了一部分：都抓到也退 3（门禁汇总记失败，不会被当成通过），有红退 1
        say(f"  … 只判了选中的 {len(rows)} 行，不是整道结论（GATE_MUTATION_ROW_SELECTION={selection_path}；"
            f"{'有红，见上面' if red else (f'选中的里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条' if not_run else '选中的都红在点名的测试上')}；"
            + (f"缩过域（{counts['narrowing']}）、不算门禁通过、现判的没入库（判定库在 {counts['records_directory']}））" if counts.get("narrowing")
               else f"判定库记录写进了 {counts['records_directory']}）") + f"；这一趟{reuse_text}")
        return 1 if red else EXIT_SELECTION_ONLY
    if red:
        return 1
    if not_run:
        # 有本次未跑的：不说全抓到；一条抓到的都没有就退 77（无对象判到）
        if not caught:
            say(f"  · crates 变异表复跑：{len(rows)} 条全是本次未跑（见上），一条都没判到被改坏的那一处：这一道记本次未跑，不是通过；这一趟{reuse_text}{library_text}")
            return 77
        say(f"  ✓ crates 变异表复跑：{len(rows)} 条里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条（本次未跑的逐条见上，不算抓到；这一趟不是整张表都红在点名的测试上）"
            f"；没红、无效 0 条，锚点失效 0 行，每条一个自己的 TMPDIR、结束时都删掉了；这一趟{reuse_text}{library_text}")
        say(f"  {counts['impact']}")
        return 0
    # ⚠️ 进程数不写进 stdout：写进去就让「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」当场为假；复用几条接在成功句末尾「；这一趟现判」之后
    say(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次、锚点失效 0 行；点名的测试在没改坏的副本上都恰好一个 ok；"
        f"每条一个自己的 TMPDIR、结束时都删掉了；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、判定与进程数无关）"
        f"；这一趟{reuse_text}{library_text}")
    # 牵连面（文件头「牵连面」）：成功句之后另起一行，只报数
    say(f"  {counts['impact']}")
    return 0


def reuse_summary_text(counts):
    """「现判 N 条、复用 M 条、拒算 R 条、本次未跑 K 条（现判的按栏：…）」：成功句末尾与「复用：」那一行用同一句。
    拒算的行照跑、不复用，不算进现判；本次未跑是判出来的一档（点名的测试打了 NOT_RUN_NO_GPU_ADAPTER），算在现判或拒算里。
    拒算里有撞键的（refuse_colliding_row_keys）末尾另接「；其中撞键拒算 N 条：第 <行> 行拒算（行键撞键：与第 <行> 行同键）、…」，没有不接。"""
    columns = "、".join(f"{title} {counts['fresh_columns'].get(name, 0)}" for name, title in REUSE_COLUMNS)
    collisions = counts.get("collision_rows", [])
    collision_text = (f"；其中撞键拒算 {len(collisions)} 条：" + "、".join(f"第 {line_number} 行拒算（{reason}）" for line_number, reason in collisions)
                      if collisions else "")
    return (f"现判 {counts['fresh_rows']} 条、复用 {counts['reused']} 条、拒算 {counts['refused_rows']} 条、本次未跑 {counts.get('not_run_rows', 0)} 条"
            f"（现判的按栏：{columns}）{collision_text}")


def write_not_run_record(path, texts):
    """每条本次未跑往 GATE_MUTATION_NOT_RUN_RECORD 那份文件追加一行（阶段转给 gate.sh 的 GATE_NOT_RUN_FILE）；没设不写。写不进去交那一句，写成交 None。"""
    if not path:
        return None
    try:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write("".join(f"{STAGE} 本次未跑：{text}\n" for text in texts))
    except OSError as error:
        return f"{path}：{error}"
    return None



# ── 命令行 ────────────────────────────────────────────────────────────────────

def unfiltered_rows_judged(root, environment):
    """static-check 判不判「第五段 `--` 之后没有测试名筛选词」（unfiltered_rows）：被判的是本模块所在的那份仓（门禁判的就是它的 crates/mutations.tsv）、--staged 的临时树（GATE_STAGED_FROM 设着）、
    样本根里放了 .mutation-filter-words-judged 的才判；别的样本根的变异表不判（它们的表只造别的红绿，行不写筛选词）。弄坏开关 unfiltered-rows-unchecked：一律不判。"""
    if break_is(environment, "unfiltered-rows-unchecked"):
        return False
    return (os.path.realpath(root) == os.path.realpath(STAGE_REPOSITORY) or bool(environment.get("GATE_STAGED_FROM", "").strip())
            or os.path.isfile(os.path.join(root, ".mutation-filter-words-judged")))


def unfiltered_rows(rows):
    """[(行号, 变异名, 为什么)]：第五段 `--` 之后没有测试名筛选词的行（row_libtest_selection 取不到过滤词：没有 `--`、或 `--` 后面只有以 - 起头的开关）。
    没有筛选词，基线与每一次变异都把点名的测试目标整个跑一遍（目标里用例越多越慢、越占内存），判法只看点名的那一条用例，其余白跑。以 MALFORMED 一类打出来（门禁 code-source-discipline 读这一类）。"""
    return [(row.line_number, row.name, "第五段 `--` 之后没有测试名筛选词：基线与每一次变异都把点名的测试目标整个跑一遍；在 `--` 后面写点名那条用例的名字（子串，不带 --exact）")
            for row in rows if not row_libtest_selection(row.arguments)[0]]


def selftest_signal_and_pipe_cells(selftest, work):
    """被 TERM 停时（双机分片的驱动停这一份）$TMPDIR 下的 singlefs-mutation-replay-* 删掉、退 143；--list-items 接 | head 时照 SIGPIPE 惯例安静退。
    子进程只对自证自己起的那一个发信号（Popen 的 send_signal）。"""
    repository = os.path.join(work, "signalled")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    temporary = os.path.join(work, "signalled-tmp")
    os.makedirs(temporary)
    hang, released, done = (os.path.join(work, name) for name in ("cargo-hanging", "cargo-released", "cargo-done"))
    environment = dict(os.environ, TMPDIR=temporary, FAKE_CARGO_HANG_FILE=hang, FAKE_CARGO_RELEASE_FILE=released,
                       FAKE_CARGO_DONE_FILE=done, GATE_MUTATION_TARGET_DIR="")   # 编译目录建在这一趟的 shard_root 里（文件头「编译目录」），停完连同它删
    stderr_path = os.path.join(work, "signalled-stderr.txt")
    # stderr 落文件、不接管道：没接住信号时工作进程成了孤儿、还开着它，接管道就等不到头
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = subprocess.Popen([sys.executable, "-c", STAGE_CHILD_PROGRAM, MODULE_PATH, repository], env=selftest.pool_process_environment(environment),
                                 stdout=subprocess.DEVNULL, stderr=stderr_handle)
    # TERM 发在「$TMPDIR 下的临时目录建出来了、假 cargo 在工作进程里挂住了（标记文件里有工作进程的进程号）」之后：run_pool 装好信号处理、建了临时目录、
    # 起了工作进程，那时收到 TERM 才走「收尾再退」；在那之前（解释器还在载入、run_stage 还在算键）收到的就是默认处理，退 -15、什么也没建。
    # 机器忙时头一步可以慢到分钟，所以按条件等（轮询，带超时），不靠定长；等不到就不发（子进程自证自己停掉），这一格判错并写明没等到什么。
    # 弄坏开关 term-sent-unwaited：不等条件，起了子进程立刻发 TERM（碰上忙机器的样子），这一格必须判错
    ready_seconds = 300
    ready_deadline = time.monotonic() + ready_seconds

    def running_state():
        shards = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
        marker = open(hang, encoding="utf-8").read().strip() if os.path.exists(hang) else ""
        return shards, marker

    while_running, hanging_marker = running_state()
    if not break_is(os.environ, "term-sent-unwaited"):
        while not (len(while_running) == 1 and hanging_marker.isdigit()) and child.poll() is None and time.monotonic() < ready_deadline:
            time.sleep(0.05)
            while_running, hanging_marker = running_state()
    not_ready = "" if len(while_running) == 1 and hanging_marker.isdigit() else \
        f"没等到临时目录和挂住的假 cargo 都在（{ready_seconds} 秒内；临时目录 {while_running}，标记文件里 {hanging_marker!r}，子进程已退 {child.poll()}）；"
    if child.poll() is None and (not not_ready or break_is(os.environ, "term-sent-unwaited")):
        child.send_signal(signal.SIGTERM)
    elif child.poll() is None:
        child.kill()   # 没到「该发 TERM」的时候就不发，也不留着它：只停这一个子进程
    try:
        child.wait(timeout=180)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()
    open(released, "w").close()
    deadline = time.monotonic() + 130
    while os.path.exists(hang) and not os.path.exists(done) and time.monotonic() < deadline:
        time.sleep(0.05)

    def process_is_alive(pid):
        """/proc/<pid> 在、而且不是僵尸（已退、等着被收的不算活着）。"""
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
                return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
        except OSError:
            return False

    # 挂住的假 cargo 记下的父进程就是那个工作进程：照理已经被停了；还活着（没接住信号的样子）就只对这一个进程号发 TERM，不留孤儿
    worker = int(open(hang, encoding="utf-8").read().strip() or 0) if os.path.exists(hang) else 0
    worker_alive = bool(worker) and process_is_alive(worker)
    if worker_alive:
        with contextlib.suppress(OSError):
            os.kill(worker, signal.SIGTERM)
    left = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
    stderr = open(stderr_path, encoding="utf-8", errors="replace").read()
    selftest.check("被 TERM 停（双机分片的驱动停这一份的样子）：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143，工作进程停了，挂住的假 cargo 也退了",
                   not not_ready and len(while_running) == 1 and left == [] and child.returncode == 128 + signal.SIGTERM and not worker_alive and os.path.exists(done),
                   f"{not_ready}跑的时候 {while_running}，停完 {left}，退 {child.returncode}，工作进程 {worker} 还活着 {worker_alive}，假 cargo 退了 {os.path.exists(done)}；"
                   f"stderr 尾：{stderr.strip()[-300:]}")
    term_targets = selftest_run_target_directories(stderr)
    selftest.check("跑完运行目录里不剩它建的编译目录（被 TERM 停）：编译目录建在 $TMPDIR 下这一趟的 singlefs-mutation-replay-* 的 build 里，停完连同它删掉",
                   not not_ready and len(term_targets) == 1 and term_targets[0].startswith(os.path.join(temporary, SHARD_ROOT_PREFIX)) and not os.path.exists(term_targets[0])
                   and left == [] and f"删了这一趟的编译目录 {os.path.dirname(term_targets[0])}" in stderr,
                   f"{not_ready}编译目录 {term_targets}，停完 $TMPDIR 下剩 {left}；{[line for line in stderr.splitlines() if '编译目录' in line][:3]}")
    read_end, write_end = os.pipe()
    os.close(read_end)
    closed_pipe = subprocess.run([sys.executable, MODULE_PATH, "--list-items", repository], stdout=write_end, stderr=subprocess.PIPE, text=True,
                                 errors="replace", env=dict(os.environ))
    os.close(write_end)
    selftest.check("--list-items 接 | head（读的一头先关了）：被 SIGPIPE 安静停掉，stderr 一个字都没有",
                   closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "", f"退 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-300:]!r}")


def selftest_killed_run_resume_cell(selftest, work):
    """判过的格被整组硬杀也留下（文件头「判定库」：每条判完立刻写、写入原子）：一趟 run_stage 在子进程里起（自己一个会话），一个工作进程，每次 cargo 睡 2 秒（FAKE_CARGO_SPAN_SLEEP）；
    等到第三条变异的 cargo 起来（前两条判完、记录写完，第三条在睡）就对整组发 SIGKILL（进程池的工作进程连同父进程一起死，与内存包装杀整组一样不给任何收尾的机会），
    等第三条的假 cargo 自己退出，再重跑同一条命令：判完的 2 条照键复用，只现判剩下的 2 条（基线记录也留着，基线不再跑）。
    重跑同样起子进程、现载入本仓的代码，不在自证进程里调 run_stage。驱动与算键库的代码不进键，两趟之间别的会话改了它们也不改两趟算出的键。
    前提没造出来（第三条起来时前两条没恰好判完：机器忙到 2 秒内没轮询到）也记本次未跑。
    弄坏开关 records-written-at-end（记录攒在工作进程内存里、进程池正常收尾才写）下被杀之后库里一条都没有，这一格必须判错。"""
    child_script = ("import importlib.util, os, sys\n"
                    "specification = importlib.util.spec_from_file_location('crates_mutation_rows_selftest_child', sys.argv[1])\n"
                    "module = importlib.util.module_from_spec(specification)\n"
                    "sys.modules[specification.name] = module\n"
                    "specification.loader.exec_module(module)\n"
                    "sys.exit(module.run_stage(sys.argv[2], {name: value for name, value in os.environ.items() if name != 'SINGLEFS_MULTI_HOST_CONFIG'}))\n")
    repository = os.path.join(work, "killed-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    temporary = os.path.join(work, "killed-run-tmp")
    os.makedirs(temporary)
    target = os.path.join(work, "killed-run-target")
    span_log = os.path.join(work, "killed-run-spans.log")
    environment = dict(selftest.base_environment, TMPDIR=temporary, GATE_MUTATION_WORKERS="1", GATE_MUTATION_TARGET_DIR=target,
                       FAKE_CARGO_SPAN_LOG=span_log, FAKE_CARGO_SPAN_SLEEP="2")

    def span_events():
        if not os.path.exists(span_log):
            return []
        with open(span_log, encoding="utf-8") as handle:
            return [line.split("\t") for line in handle.read().splitlines() if line.count("\t") == 2]

    def mutation_events(kind):
        return [event for event in span_events() if event[0] == kind and event[1].startswith("line-")]

    def start_child(child_environment, stdout_handle, stderr_handle):
        # 两趟同一条命令：子进程现载入本仓的这一份模块
        return subprocess.Popen([sys.executable, "-c", child_script, MODULE_PATH, repository], env=selftest.pool_process_environment(child_environment), start_new_session=True,
                                stdout=stdout_handle, stderr=stderr_handle)

    stderr_path = os.path.join(work, "killed-run-stderr.txt")
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = start_child(environment, subprocess.DEVNULL, stderr_handle)
    deadline = time.monotonic() + 300
    while len(mutation_events("start")) < 3 and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.02)
    started, ended = len(mutation_events("start")), len(mutation_events("end"))
    exited_early = child.poll() is not None
    if not exited_early:
        os.killpg(child.pid, signal.SIGKILL)   # 整组：子进程自己是会话首进程，进程池的工作进程在同一个进程组里；只对自证自己起的这一组发
    child.wait(timeout=60)
    # 被杀的那一格的假 cargo 在自己的会话里，睡完自己退：等它的 end 记下来，免得它晚到把一行写进重跑那一趟的 cargo 记录
    settle_deadline = time.monotonic() + 60
    while time.monotonic() < settle_deadline and len([event for event in span_events() if event[0] == "start"]) != len([event for event in span_events() if event[0] == "end"]):
        time.sleep(0.05)
    if exited_early:
        selftest.check("一趟跑到一半整组 SIGKILL 之后重跑同一条命令：第一趟子进程在被杀之前没有自己退出", False,
                       f"子进程退 {child.returncode}，起 {started} 条变异、判完 {ended} 条；stderr 尾：{open(stderr_path, encoding='utf-8', errors='replace').read()[-400:]}")
        return
    if (started, ended) != (3, 2):
        selftest.skip("被整组硬杀的一趟判过的格留下，重跑同一条命令照键复用", f"第三条变异的 cargo 起来时前两条没恰好判完（起 {started} 条、判完 {ended} 条）")
        return
    library = selftest_library(library_of(repository))
    recorded = 0
    for name in library.stored_names("records"):
        text = library.read_text("records", name) or ""
        recorded += 1 if "kind=mutation" in text and "verdict=caught" in text else 0
    rerun_environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=target)
    with contextlib.suppress(FileNotFoundError):
        os.unlink(selftest.log)
    rerun_stdout_path, rerun_stderr_path = os.path.join(work, "killed-run-rerun-stdout.txt"), os.path.join(work, "killed-run-rerun-stderr.txt")
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
    rerun_head = (rerun_stderr.split("判定库记录：", 1) + [""])[1].splitlines()[0]
    rerun_reasons = [line.strip() for line in rerun_stderr.splitlines() if line.startswith("  · 现判：")]
    resumed = len(mutation_invocations(rerun_invocations, pristine))
    selftest.check("一趟跑到一半整组 SIGKILL（前两条变异判完、第三条在跑），重跑同一条命令：判完的 2 条每条都已原子写进判定库、照键复用，只现判剩下的 2 条（基线记录也留着，基线不再跑）",
                   child.returncode == -signal.SIGKILL and recorded == ended and rerun_exit == 0 and f"复用抓到 {ended} 条、" in rerun_head and "现跑基线 0 批 0 行" in rerun_head
                   and resumed == len(SELFTEST_TABLE_ROWS) - ended and len(plain_invocations(rerun_invocations)) == resumed,
                   f"子进程退 {child.returncode}；被杀时判完 {ended} 条、库里抓到的记录 {recorded} 条；重跑退 {rerun_exit}，头一行「{rerun_head[:300]}」，"
                   f"现判的原因「{'；'.join(rerun_reasons[:2])[:400]}」，"
                   f"重跑起的变异 {resumed} 次、基线与变异合计 {len(plain_invocations(rerun_invocations))} 次；第一趟 stderr 尾：{open(stderr_path, encoding='utf-8', errors='replace').read()[-300:]}")


def selftest_shard_root_window_cell(selftest, work):
    """TERM 落在 run_pool 建 shard_root 的 mkdir 之后、名字交到调用方手里之前（tempfile.mkdtemp 的窗口）：在进程内把 os.mkdir 换成「真建出
    SHARD_ROOT_PREFIX 起头的目录之后抛 StopRequested」（等同 TERM 恰好落在那一瞬），跑 run_pool，退出码是 143、$TMPDIR 下一个目录都不剩。
    弄坏开关 shardrootnamedlate（用 tempfile.mkdtemp，名字丢了）下这一格必须判错。"""
    temporary = os.path.join(work, "shard-root-window-tmp")
    os.makedirs(temporary)
    real_mkdir, saved_tempdir = os.mkdir, tempfile.tempdir
    fired = []

    def mkdir_then_stop(path, *arguments, **keywords):
        real_mkdir(path, *arguments, **keywords)
        if os.path.basename(os.fspath(path)).startswith(SHARD_ROOT_PREFIX) and not fired:
            fired.append(os.fspath(path))
            raise StopRequested(128 + signal.SIGTERM)

    raised = None
    tempfile.tempdir = temporary
    os.mkdir = mkdir_then_stop
    try:
        run_pool(dict(environment=dict(os.environ)), 1, [], [], set(), None, None)
    except StopRequested as stop:
        raised = stop.code
    finally:
        os.mkdir = real_mkdir
        tempfile.tempdir = saved_tempdir
    left = sorted(name for name in os.listdir(temporary) if name.startswith(SHARD_ROOT_PREFIX))
    selftest.check("TERM 落在建 shard_root 的 mkdir 之后、名字交到调用方手里之前：run_pool 照样退 143，$TMPDIR 下的 singlefs-mutation-replay-* 一个都不剩",
                   len(fired) == 1 and raised == 128 + signal.SIGTERM and left == [],
                   f"mkdir 之后抛停的信号 {len(fired)} 次（要 1 次），run_pool 退出码 {raised}（要 {128 + signal.SIGTERM}），停完还剩 {left}")


def selftest_cargo_configuration_cell(selftest, work):
    """拷贝范围含 .cargo/（拷贝范围取真登记表里本阶段那一行）：点名的测试要编 gpu 特性的那一行，副本里的假 cargo 照真 cargo 的找法读得到
    .cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS，基线绿、变异抓到；读不到就照 librocksdb-sys 的构建脚本退 101，记「基线不绿」。"""
    repository = os.path.join(work, "cargo-configuration")
    build_selftest_repository(repository, [SELFTEST_TABLE_ROWS[0], SELFTEST_GPU_FEATURE_ROW])
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    exit_code, stdout, stderr, invocations = selftest.run(repository)
    feature_runs = [line for line in plain_invocations(invocations) if "--features gpu" in line]
    selftest.check("拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到",
                   exit_code == 0 and "基线不绿 0 行" in stdout and "计数：抓到 2 条、没红 0 条" in stdout and len(feature_runs) == 2,
                   f"退 {exit_code}；假 cargo 带 --features gpu 起了 {len(feature_runs)} 次（要基线 1 次、变异 1 次）；拷贝范围 {copied_paths()}；{stdout}{stderr[-400:]}")


def selftest_product_gpu_feature_cells(selftest, work):
    """产品 GPU 包（singlefs-background-gpu）真显卡对拍开的 tests-on-a-real-gpu 与 gpu 同样当上卡的特性认（GPU_TEST_FEATURES、gpu_test_run）。
    这一道不另套清场（文件头「带 gpu 特性的格不另套清场」）：两格的假 cargo 记录里起的就是 cargo 本身。两格：
    ① 带 tests-on-a-real-gpu 的那一行真跑测试的几次（基线、变异、插桩跑）各收尾带 --test-threads 1，只编不跑的与不带的那一行不带，照抓到；
    ② 点名的测试在没卡的机器上打 NOT_RUN_NO_GPU_ADAPTER 后返回成功：记本次未跑，不算抓到也不算没红，整道不判红，真跑的那几次同样收尾带 --test-threads 1。
    弄坏开关 product-gpu-feature-unquiesced 下这一特性不当上卡的特性认（两格判错）。"""
    def product_world(name, product_row, extra_source=""):
        """一棵带一行不带上卡特性、一行带 tests-on-a-real-gpu（product_row）的小仓。"""
        root = os.path.join(work, name)
        build_selftest_repository(root, [SELFTEST_TABLE_ROWS[0], product_row], extra_source=extra_source)
        selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        return root

    def split_runs(invocations):
        """(带 tests-on-a-real-gpu 真跑的每一次的参数, 不带它真跑的每一次的参数, 只编不跑的每一次的参数)。"""
        runs = [line.split("\t")[0].split() for line in invocations]
        real = [words for words in runs if "--no-run" not in words]
        return ([words for words in real if "tests-on-a-real-gpu" in words], [words for words in real if "tests-on-a-real-gpu" not in words],
                [words for words in runs if "--no-run" in words])

    root = product_world("product-gpu-feature", SELFTEST_PRODUCT_GPU_FEATURE_ROW)
    exit_code, stdout, stderr, invocations = selftest.run(root)
    product_runs, plain_runs, compile_runs = split_runs(invocations)
    selftest.check("产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各收尾带 --test-threads 1，只编不跑的与不带的不带，照抓到",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in stdout and len(product_runs) == 3
                   and all(words[-2:] == ["--test-threads", "1"] for words in product_runs)
                   and plain_runs and all("--test-threads" not in words for words in plain_runs)
                   and compile_runs and all("--test-threads" not in words for words in compile_runs),
                   f"退 {exit_code}；带 tests-on-a-real-gpu 真跑 {product_runs}（要基线、变异、插桩跑各 1 次，收尾都是 --test-threads 1）；不带的 {plain_runs}；"
                   f"只编不跑 {compile_runs}；{stdout[-600:]}{stderr[-400:]}")
    root = product_world("product-gpu-card-absent", SELFTEST_PRODUCT_GPU_NOT_RUN_ROW, SELFTEST_NOT_RUN_TEST)
    not_run_record = os.path.join(work, "product-gpu-not-run-record.txt")
    exit_code, stdout, stderr, invocations = selftest.run(root, GATE_MUTATION_NOT_RUN_RECORD=not_run_record)
    recorded_lines = open(not_run_record, encoding="utf-8").read().splitlines() if os.path.exists(not_run_record) else []
    product_runs, _plain_runs, _compile_runs = split_runs(invocations)
    selftest.check("产品 GPU 包（--features tests-on-a-real-gpu）的行：点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER 后返回成功，记本次未跑（不算抓到、不算没红），"
                   "整道不判红，记录里一行，真跑的那几次收尾带 --test-threads 1",
                   exit_code == 0 and "本次未跑：1 条" in stdout and "计数：抓到 1 条、没红 0 条" in stdout
                   and f"{TABLE}:3 product-card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in stdout
                   and len(recorded_lines) == 1 and "product-card-path-changed" in recorded_lines[0]
                   and len(product_runs) >= 2 and all(words[-2:] == ["--test-threads", "1"] for words in product_runs),
                   f"退 {exit_code}；带 tests-on-a-real-gpu 真跑 {product_runs}（基线、变异至少 2 次，收尾都是 --test-threads 1）；记录 {recorded_lines}；"
                   f"{stdout[-900:]}{stderr[-400:]}")


def selftest_gpu_configuration_cells(selftest, work):
    """真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡（文件头同名一段）。PATH 上换一份包着假 cargo 的假 cargo（这一格自己写，FAKE_CARGO 不动）：
    每起一次往记录里写一行 JSON（参数、起在哪个目录、收到的 SINGLEFS_MULTI_HOST_CONFIG），点名 gpu_parity、带 --show-output、它是 ok 的那一次
    照 libtest 的样子在末尾交出 $GIVEN_STATES_TALLIES（<GPU 判的>/<CPU 判的>，逗号分开几组）那几行 CRASH_POINT_REPLAY_GIVEN_STATES。五格：
    ① 交多机配置：一行不带 gpu、一行带 gpu（都抓到），两行真跑测试的每一次（基线、变异、插桩跑）在副本里起的 cargo 都收到
       SINGLEFS_MULTI_HOST_CONFIG=那份配置的绝对路径，只编不跑的收不到；
    ② 多机配置不在：run_stage 起 cargo 之前就拒、退 2、点名那份配置，假 cargo 一次都没起；
    ③ 给定状态有 GPU 判过的（两行里一行 states_judged_by_the_gpu_here 大于 0）：照常判，没红就记没红、整道判红；
    ④ 给定状态全交 CPU 判（两行都是 0）：记本次未跑、不进没红，整道不判红；
    ⑤ named_test_stdout 只读点名的测试那一段（别的用例那一段里 GPU 判过的、同前缀的拒算行都不算）。"""
    gpu_row = "gpu-halve-quartered\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t-p tiny --features gpu --lib -- gpu_parity\tgpu_parity"
    wrapper_directory = os.path.join(work, "gpu-configuration-bin")
    selftest_write(os.path.join(wrapper_directory, "cargo"), r'''#!/usr/bin/env python3
# 包着自证的假 cargo（$INNER_FAKE_CARGO）：记一行 JSON 进 $GPU_CONFIGURATION_LOG，原样起它；点名 gpu_parity、带 --show-output、它是 ok 的那一次
# 照 libtest 的样子在末尾交出 $GIVEN_STATES_TALLIES 那几行 CRASH_POINT_REPLAY_GIVEN_STATES
import json, os, subprocess, sys
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:   # 插桩编译与插桩跑在 test 前面带 --config build.rustflags=…（同假 cargo 的认法）
    arguments = arguments[2:]
if os.environ.get("GPU_CONFIGURATION_LOG") and arguments[:1] == ["test"]:
    with open(os.environ["GPU_CONFIGURATION_LOG"], "a", encoding="utf-8") as log:
        log.write(json.dumps({"ARGUMENTS": " ".join(arguments[1:]), "WORKING_DIRECTORY": os.getcwd(),
                              "SINGLEFS_MULTI_HOST_CONFIG": os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")}) + "\n")
inner = subprocess.run([sys.executable, os.environ["INNER_FAKE_CARGO"]] + sys.argv[1:], capture_output=True, text=True)
sys.stdout.write(inner.stdout)
sys.stderr.write(inner.stderr)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
if "--show-output" in after and "gpu_parity" in after and "test tests::gpu_parity ... ok" in inner.stdout:
    lines = "".join(f"CRASH_POINT_REPLAY_GIVEN_STATES item=selftest-{index} stage=judge mode=judge judged_by=selftest tables_judged_here=1 "
                    f"states_judged_by_the_gpu_here={gpu} states_judged_by_the_cpu_here={cpu} states_reused=0 states_the_gpu_could_not_judge=0\n"
                    for index, (gpu, cpu) in enumerate(tally.split("/") for tally in os.environ["GIVEN_STATES_TALLIES"].split(",")))
    print("\nsuccesses:\n\n---- tests::gpu_parity stdout ----\n" + lines)
sys.exit(inner.returncode)
''', executable=True)

    def configuration_world(name, tallies, configuration, needed):
        """一棵带一行不带 gpu（adds，抓到）、一行带 gpu（gpu_parity 只看 needed：给 { value * 2 } 时改 halve 的那一条没红，给 value / 2 } 时抓到；带收尾的花括号才只命中代码那一行、不命中别的用例的说明行，也不让变异的原文 { value / 2 } 在说明行里再命中一次）的小仓，跑一趟。
        交 (仓, 退出码, stdout 与 stderr 末尾, 包着的假 cargo 记的每一次)。configuration 是交给本模块的那一份多机配置的路径（GATE_MUTATION_MULTI_HOST_CONFIG）。"""
        root = os.path.join(work, name)
        build_selftest_repository(root, [SELFTEST_TABLE_ROWS[0], gpu_row], extra_source=f"// TEST gpu_parity NEEDS {needed}\n")
        selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        log = os.path.join(work, name + "-cargo-environment.jsonl")
        exit_code, stdout, stderr, _ = selftest.run(
            root, GATE_MUTATION_MULTI_HOST_CONFIG=configuration, GPU_CONFIGURATION_LOG=log, GIVEN_STATES_TALLIES=tallies,
            INNER_FAKE_CARGO=os.path.join(selftest.bin_directory, "cargo"), PATH=wrapper_directory + os.pathsep + selftest.base_environment["PATH"])
        return root, exit_code, stdout + stderr[-600:], environment_records(log)

    def test_runs(records, with_gpu):
        return [record for record in records if ("--features gpu" in record["ARGUMENTS"]) == with_gpu and "--no-run" not in record["ARGUMENTS"].split()]

    configuration = os.path.join(work, "gpu-configuration-present", "multi-host.env")
    selftest_write(configuration, "# 自证用的多机配置：假 cargo 不读它，这一格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
    root, exit_code, output, records = configuration_world("gpu-configuration-passed", "3/0", configuration, "value / 2 }")
    with_gpu, without_gpu = test_runs(records, True), test_runs(records, False)
    compiles = [record for record in records if "--no-run" in record["ARGUMENTS"].split()]
    received = [record.get("SINGLEFS_MULTI_HOST_CONFIG") for record in with_gpu + without_gpu]
    selftest.check("真跑测试的每一次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG 指向本模块所在仓的多机配置（绝对路径，带不带 gpu 都有），只编不跑的没有",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in output and len(with_gpu) == 3 and without_gpu and received == [configuration] * len(received)
                   and all(record["WORKING_DIRECTORY"] != root for record in with_gpu + without_gpu) and compiles
                   and all(record.get("SINGLEFS_MULTI_HOST_CONFIG") is None for record in compiles),
                   f"退 {exit_code}；带 gpu 真跑 {len(with_gpu)} 次（要基线、变异、插桩跑各 1 次）、不带 gpu 真跑 {len(without_gpu)} 次，收到 {received}（要都是 {configuration}），"
                   f"起在 {[record['WORKING_DIRECTORY'] for record in with_gpu + without_gpu]}（不许是 {root}）；只编不跑的收到 "
                   f"{[record.get('SINGLEFS_MULTI_HOST_CONFIG') for record in compiles]}（要都没有）；{output[-700:]}")
    absent = os.path.join(work, "gpu-configuration-absent", "multi-host.env")
    _root, exit_code, output, records = configuration_world("gpu-configuration-missing", "3/0", absent, "{ value * 2 }")
    selftest.check("多机配置不在：run_stage 退 2、点名那份配置，起 cargo 之前就拒（假 cargo 一次都没起）",
                   exit_code == 2 and f"多机配置 {absent} 不在" in output and not records,
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次（要 0 次）；{output[-900:]}")
    _root, exit_code, output, _records = configuration_world("gpu-given-states-on-the-card", "0/12,7/0", configuration, "{ value * 2 }")
    selftest.check("带 gpu 特性的行：给定状态有 GPU 判过的（两行里一行 states_judged_by_the_gpu_here 大于 0）照常判：没红记没红、整道判红",
                   exit_code == 1 and "计数：抓到 1 条、没红 1 条" in output and "gpu-halve-quartered：gpu_parity 没红" in output and "本次未跑：" not in output,
                   f"退 {exit_code}；{output[-900:]}")
    _root, exit_code, output, _records = configuration_world("gpu-given-states-on-the-cpu", "0/12,0/3", configuration, "{ value * 2 }")
    selftest.check("带 gpu 特性的行：给定状态全交 CPU 判（每一行 states_judged_by_the_gpu_here 都是 0）记本次未跑、不进没红，整道不判红",
                   exit_code == 0 and "计数：抓到 1 条、没红 0 条" in output and "本次未跑：1 条" in output
                   and "gpu-halve-quartered：本次未跑——点名的测试 gpu_parity 带 gpu 特性，它打的 2 行 CRASH_POINT_REPLAY_GIVEN_STATES 里 states_judged_by_the_gpu_here 全是 0" in output,
                   f"退 {exit_code}；{output[-900:]}")
    sample = ("running 2 tests\ntest tests::gpu_parity ... ok\ntest tests::other ... ok\n\nsuccesses:\n\n"
              "---- tests::other stdout ----\nCRASH_POINT_REPLAY_GIVEN_STATES item=a states_judged_by_the_gpu_here=9 states_judged_by_the_cpu_here=0\n\n"
              "---- tests::gpu_parity stdout ----\nCRASH_POINT_REPLAY_GIVEN_STATES item=b states_judged_by_the_gpu_here=0 states_judged_by_the_cpu_here=4\n"
              "CRASH_POINT_REPLAY_GIVEN_STATES_KEY_REFUSED item=b states_judged_by_the_gpu_here=5\n\n"
              "successes:\n    tests::gpu_parity\n    tests::other\n\ntest result: ok. 2 passed; 0 failed\n")
    named = given_states_gpu_counts(named_test_stdout("tests::gpu_parity", sample))
    selftest.check("named_test_stdout 只读点名的测试那一段：别的用例那一段里 GPU 判过的、同前缀的拒算行都不算",
                   named == [0] and named_test_stdout("tests::absent", sample) == "" and named_test_stdout("tests::gpu_parity", "no sections\n") == "no sections\n",
                   f"读到 {named}（要 [0]）")


def selftest_link_failure_cell(selftest, work):
    """链接失败（verdict_of_run 的 linkfailed，文件头「复用判据」）：点名的测试一行都没出现、输出里有 error: linking with 与 rust-lld、没有 error[E<码>]，判 linkfailed
    （写明失败在链接、出路是删编译目录重跑这一行），不判 invalid；同一份输出加一行 error[E0308]、或链接器不是 rust-lld 的，仍判 invalid；
    报告里（report）有链接失败的一趟判红退 1、单列一节写出路，不并进「无效」的计数。弄坏开关 link-failure-as-invalid（照无效记）下头一格判错，
    link-failure-not-red（报告里不算红）下报告那一格判错。"""
    environment = dict(selftest.base_environment)
    row = Row(9, "answer-changed", "crates/tiny/src/lib.rs", "{ 1 }", "{ 2 }", "-p tiny --lib -- answer", "answer", ())
    linked = verdict_of_run(row, 101, SELFTEST_LINK_FAILURE_OUTPUT, environment)
    compile_error = verdict_of_run(row, 101, SELFTEST_LINK_FAILURE_OUTPUT + "error[E0308]: mismatched types\n", environment)
    other_linker = verdict_of_run(row, 101, SELFTEST_LINK_FAILURE_OUTPUT.replace(LINK_FAILURE_LINKER, "ld"), environment)
    selftest.check("链接失败：编过了、输出里有 error: linking with 与 rust-lld、没有 error[E<码>]，判 linkfailed、写明失败在链接与出路（删编译目录重跑这一行），不判 invalid；"
                   "同一份输出多一行 error[E0308]、或链接器不是 rust-lld，仍判 invalid",
                   linked[0] == "linkfailed" and "失败在链接" in linked[1] and "删掉这一趟用的编译目录，重跑这一行" in linked[1]
                   and compile_error[0] == "invalid" and other_linker[0] == "invalid",
                   f"链接失败的输出判 {linked[0]}（{linked[1][:120]}）；多一行 error[E0308] 判 {compile_error[0]}；链接器不是 rust-lld 判 {other_linker[0]}")
    selftest.check("链接失败不盖算数的结论：linkfailed 在 VERDICTS_UNCOUNTED 里、不在 VERDICTS_COUNTED 里",
                   "linkfailed" in VERDICTS_UNCOUNTED and "linkfailed" not in VERDICTS_COUNTED, f"不算数 {VERDICTS_UNCOUNTED}；算数 {VERDICTS_COUNTED}")
    shown = io.StringIO()
    counts = dict(fresh_columns={}, fresh_rows=1, reused=0, refused_rows=0, records_directory="", coverage_recorded=0, coverage_existing=0, coverage_failed=0,
                  worker_count=1, cargo_jobs=1, memory_text="", reused_baseline=0, fresh_mutations=1, fresh_batches=0, fresh_long_batches=0,
                  fresh_unknown_batches=0, impact="", not_run_record="", narrowing="")
    outcome = dict(left_behind=[], per_mutation_temporary_root="", left_behind_contents={})
    with contextlib.redirect_stdout(shown), contextlib.redirect_stderr(io.StringIO()):
        exit_code = report([row], [(row.line_number, "linkfailed", linked[1], None)], outcome, "", True, counts)
    printed = shown.getvalue()
    selftest.check("报告里有链接失败的一趟：退 1，单列「有变异链接失败」一节写出路，「计数：」那一行的无效仍是 0 条",
                   exit_code == 1 and "有变异链接失败（链接失败 1 条" in printed and "删掉这一趟用的编译目录" in printed and "无效 0 条" in printed,
                   f"退 {exit_code}；输出 {printed[-400:]}")


def selftest_sentinel_uncounted_cell(selftest):
    """强制现判那一条本次未跑（notrun）：observe_fresh_outcomes 不把它当现判结论交哨兵，哨兵照「这一趟没报结局」打一行不判、不判成不稳。
    两条候选、哨兵 1 条，另一条就是强制现判；只喂结局，不起 run_stage。弄坏开关 uncounted-forced-observed 下照交，哨兵判「判定不稳」，这一格必须判错。"""
    rows = [SelftestSentinelRow(2, "root-457"), SelftestSentinelRow(3, "root-double")]
    rows_by_line = {row.line_number: row for row in rows}
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "crates 变异表", count=1, seed=5)
    planned = []
    sentinel.plan({sentinel_item(row): "抓到" for row in rows}, printer=planned.append)
    forced = [row for row in rows if sentinel.forced(sentinel_item(row))]
    for row in rows:
        if row not in forced:
            sentinel.observe(sentinel_item(row), reuse_checks.REUSED, "抓到")
    observe_fresh_outcomes(sentinel, [(row.line_number, "notrun", "", None) for row in forced], rows_by_line, selftest.base_environment)
    printed = []
    problems = sentinel.finish(printer=printed.append)
    expected = f"强制现判的 {sentinel_item(forced[0])} 这一趟没报结局（没跑到），不判" if forced else "（没抽出强制现判）"
    selftest.check("复用哨兵：强制现判那一条本次未跑（notrun）：不当现判结论交哨兵，打一行不判、不判成不稳",
                   len(forced) == 1 and not problems and any(expected in line for line in printed),
                   f"强制现判 {[sentinel_item(row) for row in forced]}；哨兵判 {problems}；打的 {printed}；抽的 {planned}")


def build_coverage_repository(directory):
    """插桩编译那一格的样本仓：crates/tiny 用 SELFTEST_COVERAGE_SOURCE，另有一个 -p tiny 编不进去的包 crates/other。"""
    build_selftest_repository(directory, SELFTEST_COVERAGE_ROWS, source=SELFTEST_COVERAGE_SOURCE)
    selftest_write(os.path.join(directory, "crates", "other", "Cargo.toml"), '[package]\nname = "other"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(directory, "crates", "other", "src", "lib.rs"), "pub fn other() -> u32 { 1 }\n")


def selftest_build_profile_cell(selftest, work):
    """插桩编译那一步（假 cargo 照构建脚本的样子写一份剖析文件，没设 LLVM_PROFILE_FILE 就写当前目录）的剖析文件只落在这一格的临时目录。"""
    repository = os.path.join(work, "build-profiles")
    build_coverage_repository(repository)
    profile_log = os.path.join(work, "cargo-profiles.log")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(profile_log)
    run = selftest.run(repository, FAKE_CARGO_PROFILE_LOG=profile_log)
    written = []
    if os.path.exists(profile_log):
        with open(profile_log, encoding="utf-8") as handle:
            written = [line.split("\t") for line in handle.read().splitlines()]
    builds = [line for line in coverage_invocations(run[3]) if "--no-run" in line]
    # 这一格的 TMPDIR 是 coverage-<行号>-<随机> 下的 build-profiles/：落点的目录要叫这个名字、父目录要叫 coverage-…，且不在编译所在的工作目录里
    outside = [path for work_directory, path in written
               if os.path.dirname(path).endswith(os.sep + BUILD_PROFILES_DIRECTORY_NAME)
               and os.path.basename(os.path.dirname(os.path.dirname(path))).startswith("coverage-")
               and not (path == work_directory or path.startswith(work_directory + os.sep))]
    selftest.check("插桩编译那一步写出的剖析文件只落在这一格的临时目录、工作目录里一个都没有（假 cargo 没设 LLVM_PROFILE_FILE 就写当前目录）",
                   run[0] == 0 and len(builds) == 3 and len(written) == len(builds) and len(outside) == len(written),
                   f"退 {run[0]}；插桩编译 {len(builds)} 次；假 cargo 写了 {len(written)} 份：{written}；落对的 {len(outside)} 份")
