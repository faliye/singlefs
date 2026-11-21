"""crates-mutation-rows.py 拆出的一份。
派活之前：读选行、核表（prechecks）、分行（plan_rows）、开 GPU 时补 gpu 特性、点名测试够不着被改坏的函数（static-check 可达性）、判复用、牵连面，连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import contextlib
import hashlib
import multiprocessing
import os
import re
import shutil
import sys
import time
import traceback
from crates_mutation_rows_base import (CARGO_CONFIGURATION_DIRECTORY, CARGO_DEBUG_PROFILES, CARGO_DEBUG_SETTING, CARGO_RELEASE_INCREMENTAL_SETTING,
    COLLISION_REFUSAL_PREFIX, COPY_REBUILD_HOWTO, FIELD_NAMES, GPU_ROUTE_FEATURE, GPU_ROUTE_UNITS, GPU_TEST_THREADS_RELEASED_TARGETS,
    REACHABILITY_CPU_CONFIGURATION_TEXT, REACHABILITY_CPU_FALLBACK_ROOTS, REACHABILITY_GPU_ROUTE_STOPS, RELEASE_REGISTRY_ROOT, ROW_PROBLEM_KINDS,
    Row, STAGE, STAGE_REPOSITORY, TABLE, _REACHABILITY_PARALLEL, admission_module, baseline_cargo_environment, break_is, copied_paths,
    inserted_before_double_dash, merge_step_close, merge_step_open, merge_step_part, merge_step_row_done, mutation_units, note, print_rejection,
    read_table)
from crates_mutation_rows_judging import (baseline_problem, baseline_reuse_problem, row_baseline_output)
from crates_mutation_rows_support import (build_selftest_repository, coverage_invocations, per_row_how, plain_invocations, replace_in_file,
    selftest_forced_row_name, selftest_new_library_copy, selftest_one_forced, selftest_verdict_view, selftest_write, selftest_write_stage_registration_table)
from crates_mutation_rows_static import (copied_with_manifest_directories, copied_with_path_dependencies, default_features_off_row, forked_parts,
    release_forced_row, release_registered_targets, row_problems, static_check_slices)
from crates_mutation_rows_units import (KeyContext, key_workers, new_key_pool_facts, open_library,
    precompute_closures_from_shared_parent, release_key_context, row_decision, row_identity, row_unit_keys, row_view,
    selftest_uncounted_keeps_counted_cells)
from crates_mutation_rows_targets import (environment_records)
from crates_mutation_rows_runs import (test_runs_multi_host_configuration)
from crates_mutation_rows_pool import (with_break)


def read_selection(path, rows):
    """GATE_MUTATION_ROW_SELECTION 那份文件：一行一个变异表行号（# 起头的行与空行不看）。回 (选中的行号集合, [问题])。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        return set(), [f"读不了 {path}：{error}"]
    selected, problems = set(), []
    row_lines = {row.line_number for row in rows}
    for index, line in enumerate(lines, 1):
        text = line.strip()
        if not text or text.startswith("#"):
            continue
        if not text.isdigit():
            problems.append(f"{path} 第 {index} 行「{text}」不是一个行号")
        elif int(text) not in row_lines:
            problems.append(f"{path} 第 {index} 行：{TABLE} 第 {text} 行不是一条成形的变异")
        else:
            selected.add(int(text))
    if not selected and not problems:
        problems.append(f"{path} 里一个行号都没有")
    return selected, problems


def prechecks(root, rows, malformed, environment):
    """派活之前先判、一个工作进程都不起的那几样（row_problems，与门禁 code-source-discipline 经 static-check 判的是同一份）。回 (拷贝范围, 整道判红了没有, {行号: 锚点失效的原因})。
    锚点失效按行判（文件头「锚点失效」）：只有它的时候不判整道，交回那几行由 run_stage 单记一档；别的几样有一样就整道判红，锚点失效的行一起列出。
    弄坏开关 anchor-stops-table：锚点失效也让整道判红。"""
    admission = admission_module()
    try:
        copied = copied_paths()
    except admission.RegistrationError as error:
        copied, reason = [], str(error)
    else:
        reason = "那一行一条路径都没登记"
    if not copied:
        print_rejection(f"读不到 .claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行登记的路径（{reason}）：判不出每片的源码副本该拷哪些", [],
                        [f"单独执行 python3 scripts/gate/admission.py paths \"{STAGE_REPOSITORY}\" {STAGE} 看它报什么；那一行不在就照别的行给本阶段登记它读的路径。"])
        return None, True, {}
    try:
        copied = copied_with_path_dependencies(root, copied, environment)
    except (ValueError, OSError) as error:
        print_rejection(f"拷贝范围跟不全 path 依赖（{error}）：判不出每片的源码副本该拷哪些，一个工作进程都没起", [],
                        ["path 依赖指到被判那棵树之外的，改成树里的目录（判定库服务 prune-server/ 这一类放仓根下）；Cargo.toml 读不懂的照报错修，再跑。",
                         COPY_REBUILD_HOWTO + "。"])
        return None, True, {}
    try:
        copied = copied_with_manifest_directories(root, copied, rows, environment)
    except (ValueError, OSError) as error:
        print_rejection(f"拷贝范围补不上变异行 --manifest-path 指的包目录（{error}）：判不出每片的源码副本该拷哪些，一个工作进程都没起", [],
                        ["那一行的 --manifest-path 写成仓内一份 Cargo.toml 相对仓根的路径（例 --manifest-path prune-server/Cargo.toml），只写一次；",
                         "那个包的 path 依赖指到树之外的改成树里的目录；改完先跑 python3 scripts/mutation/crates-mutation-rows.py static-check <根>。"])
        return None, True, {}
    # 锚点只在真仓上核：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，所以拷贝范围也在派活之前判。
    # 锚点失效之外的几样有一样红就一个工作进程都不起；只有锚点失效的，那几行单记一档、别的行照跑。
    problems = row_problems(root, rows, malformed, copied)
    whole_table = [kind for kind in ROW_PROBLEM_KINDS if problems[kind] and (kind != "ANCHOR" or break_is(environment, "anchor-stops-table"))]
    if not whole_table:
        return copied, False, {line_number: why for line_number, _name, why in problems["ANCHOR"]}
    not_started = "这一轮一个工作进程都没起；门禁 code-source-discipline 静态判同样几样（不跑变异），改完先跑它。"
    if problems["MALFORMED"]:
        print_rejection(f"{TABLE} 有几行不成形（六段制表符分隔：{'、'.join(FIELD_NAMES)}；替换文可以空，那是删掉原文，其余五段不许空）：",
                        [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["MALFORMED"]],
                        ["补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。", not_started])
    if problems["ANCHOR"]:
        print_rejection("变异表有几行锚点失效：", [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["ANCHOR"]],
                        ["改代码时把变异表里的原文一起改到今天的写法、让它恰好命中一次（锚点失效的那一行等于没跑过，.claude/rules/mutation-sampling.md 第七类）。", not_started])
    if problems["NAME"]:
        print_rejection("变异表有几行的「必须红的测试名」不是一个用例名（带了正则符号或空白），按字面在测试输出里永远找不到：",
                        [f"{TABLE}:{line_number} {name}：「{expected}」" for line_number, name, expected in problems["NAME"]],
                        ["那一列写成用例名本身（例 new_pool_file_creation_self_release_is_one_slot，或带模块路径 tests::new_pool_file_creation_self_release_is_one_slot），",
                         "去掉 $、^ 这类符号；只写名字时本阶段已经按「在输出里只许对上一个用例」判，不用再靠 $ 锚定。", not_started])
    if problems["ROW"]:
        print_rejection("变异表有几行的文件不在每片拷的范围里（锚点在真仓上核得过，工作进程在自己的源码副本里却打不开它）：",
                        [f"{TABLE}:{line_number} {name}：{path}" for line_number, name, path in problems["ROW"]],
                        [f"把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 本阶段那一行（现在是 {'、'.join(copied)}），"
                         "或者把这条变异改到拷贝范围之内的文件上。", not_started])
    if problems["STRAY"]:
        print_rejection("变异表有几行的原文或替换文里有 \\n 以外的反斜杠转义（这里只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑）：",
                        [f"{TABLE}:{line_number} {name}：{segment}里的 {escape}" for line_number, name, segment, escape in problems["STRAY"]],
                        ["去掉那个反斜杠（例 \\&report.outcome 写成 &report.outcome），改完单跑那一行证明点名的测试红。", not_started])
    return None, True, {}



def light_components(components):
    """交给工作进程的那一份（进程池按 pickle 传）：只有键与几样组成的号，不带单元对象。"""
    return dict(row_key=components.row_key or "", baseline_key=components.baseline_key or "", coverage_id=components.coverage_id or "",
                test_closure=components.test_closure or "",
                mutated=",".join(mutation_units.unit_id(reference) for reference in components.mutated or []))


def baseline_check(library, row, components, key_context, snapshot, environment):
    """(基线 ok 复用得上没有, 不复用的那一句)。拒算的行（没有基线键）照跑。
    覆盖记录里执行过的单元照这一行的单元键核（KeyContext.keys_for：--manifest-path 指的包用它自己那一份，文件头「--manifest-path」）。"""
    if not components.baseline_key:
        return False, components.refusal or "没有基线键"
    keys, _refusal = key_context.keys_for(row)
    record = library.read("baselines", components.baseline_key)
    coverage = library.read_json("coverage", components.coverage_id) if components.coverage_id else None
    problem = baseline_reuse_problem(record, components, keys, coverage, snapshot, environment)
    return problem is None, problem


def plan_rows(root, rows, environment, table_rows=()):
    """(KeyContext, 这一趟的行, {行号: Components}, 判定库)：run_stage、plan、row-plan、--list-items、import 共用这一份算法；只读（不写库）。
    交回的行：验证包只点名集成测试目标的先换成补了 --no-default-features 的（default_features_off_row，文件头「验证包的测试目标不编装置二进制」），
    命中重测试目标而没带 release 的换成补了 --release 的（release_forced_row，文件头「重测试目标只许 release」），
    多机配置开了 GPU 时点名测试走崩溃判段的那几行换成补了 gpu 特性的（gpu_route_rows），别的原样；键都照换了之后的参数算。
    table_rows 是整张表成形的行（选行时给）：没选中、而原文与替换文都与选中的某一行相同的行（行键撞得上的只有这几行）一起算键，撞键照整张表判（refuse_colliding_row_keys）。
    重测试目标的登记表读不了抛 lib_release_only_tests.py 的 RegistryUnreadable（调用方拒、一条都不跑）。
    环境里开了分步进度（merge_progress_variable）就把「现算行键」这一步分段、逐行打进度。"""
    planned_lines = {row.line_number for row in rows}
    texts = {(row.old, row.new) for row in rows}
    peers = [row for row in table_rows if row.line_number not in planned_lines and (row.old, row.new) in texts]
    progress = merge_step_open(environment, "现算行键", len(rows) + len(peers))
    key_context = KeyContext(root, environment)
    key_context.release_registered = release_registered_targets()
    forced_rows = [release_forced_row(default_features_off_row(row, environment), key_context.release_registered, environment) for row in rows]
    forced_peers = [release_forced_row(default_features_off_row(row, environment), key_context.release_registered, environment) for row in peers]
    # 报出来的只是这一趟判的行：一起算键的别的行（peers）补没补 --release 不报
    key_context.release_lines = [(row.line_number, basis) for row, basis in forced_rows if basis is not None]
    rows, peers = [row for row, _basis in forced_rows], [row for row, _basis in forced_peers]
    closures_computed = key_context.prepare(list(rows) + peers)
    merge_step_part(progress, f"点名测试的闭包算完（{closures_computed} 个）")
    components = {}
    for done, row in enumerate(list(rows) + peers, 1):
        components[row.line_number] = key_context.components(row)
        merge_step_row_done(progress, done)
    every_row, components = gpu_route_rows(key_context, list(rows) + peers, components, environment)
    key_context.collisions = refuse_colliding_row_keys(components, environment)
    merge_step_close(progress, f"算了 {len(components)} 行的键")
    # 报出来的只是这一趟判的行：一起算键的别的行（peers）补没补特性不报
    key_context.gpu_route_lines = [item for item in key_context.gpu_route_lines if item[0] in planned_lines]
    key_context.gpu_route_unbuilt = [item for item in key_context.gpu_route_unbuilt if item[0] in planned_lines]
    key_context.release_lines = sorted(item for item in key_context.release_lines if item[0] in planned_lines)
    rows = [row for row in every_row if row.line_number in planned_lines]
    return key_context, rows, {row.line_number: components[row.line_number] for row in rows}, open_library(root, environment)


def refuse_colliding_row_keys(components, environment):
    """表里两行以上算出同一把行键时这几行拒算（文件头「行键」）：行键不带路径与名字（D13（验证路线） 已定项 17），
    两份内容相同的函数各改一处、点名同一条测试的两行算得出同一把键，谁的判定都不能替另一行作保；拒算的照跑、不复用、不写记录。
    就地换掉 components 里撞键的那几份（row_key 置空、refusal 写「行键撞键：与第 N 行同键」，基线键与覆盖号不动：它们不带被改坏的单元，两行各自的基线照常复用），
    交撞键的几组行号（每组从小到大，组按头一个行号排）。弄坏开关 row-key-collision-unrefused：不判撞键（撞键的两行共用一把键、互相复用）。"""
    if break_is(environment, "row-key-collision-unrefused"):
        return []
    lines_by_key = collections.defaultdict(list)
    for line_number, comp in components.items():
        if comp.row_key is not None:
            lines_by_key[comp.row_key].append(line_number)
    groups = sorted(sorted(lines) for lines in lines_by_key.values() if len(lines) > 1)
    for lines in groups:
        for line_number in lines:
            others = "、".join(str(other) for other in lines if other != line_number)
            components[line_number] = components[line_number]._replace(row_key=None, refusal=f"{COLLISION_REFUSAL_PREFIX}与第 {others} 行同键")
    return groups


def gpu_route_enabled(environment):
    """多机配置开没开 GPU（文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：配置照 test_runs_multi_host_configuration 找（真跑测试的格交给测试的就是这一份），
    写了 ENABLE_GPU=1 交 (True, 路径)，别的（不在、没写、写的不是 1）交 (False, 路径)。在而读不了抛 admission.py 的 InputManifestError。"""
    path, present = test_runs_multi_host_configuration(environment)
    return present and admission_module().configuration_value(path, "ENABLE_GPU") == "1", path


def gpu_route_feature_word(row, components, key_context):
    """这一行要补的特性写法（点名测试的单元闭包够得着 GPU_ROUTE_UNITS 里的一处、cargo 参数里还没开 gpu 时）：cargo 参数的 -p 点名了那一处的包写 gpu，
    没点名写 <那一处的包>/gpu；不用补（闭包够不着、已开了 gpu 或 --all-features、闭包算不出）交 None。"""
    closure = key_context.closures.get(components.test_closure) if components.test_closure else None
    return None if closure is None else gpu_route_feature_word_of(row.arguments, closure)


def gpu_route_package(closure):
    """点名测试的单元闭包（ClosureInfo）够得着 GPU_ROUTE_UNITS 里的哪一处：交那一处的包，够不着交 None。"""
    wanted = {(file, name): package for package, file, name in GPU_ROUTE_UNITS[0]}
    for member in closure.members:
        package = wanted.get((member.file, member.name.split("::")[-1]))
        if package is not None:
            return package
    return None


def gpu_feature_on(arguments):
    """解析过的 cargo 参数里已开了 gpu（--features 里有 gpu 或 <包>/gpu，或 --all-features）。"""
    return arguments.all_features or any(feature.split("/")[-1] == GPU_ROUTE_FEATURE for feature in arguments.features)


def gpu_route_feature_word_of(arguments_text, closure):
    """同 gpu_route_feature_word，闭包由调用方给（ClosureInfo）：cargo 参数里没开 gpu、闭包够得着选路函数时交要补的特性写法，否则交 None。"""
    arguments = mutation_units.parse_cargo_arguments(arguments_text)
    if gpu_feature_on(arguments):
        return None
    package = gpu_route_package(closure)
    if package is None:
        return None
    return GPU_ROUTE_FEATURE if package in arguments.packages else f"{package}/{GPU_ROUTE_FEATURE}"


def with_feature(arguments, feature):
    """cargo 参数里补一个 --features <特性>：插在第一个 `--` 之前（libtest 的参数不动），没有 `--` 接在末尾。"""
    return inserted_before_double_dash(arguments, ["--features", feature])


def gpu_route_rows(key_context, rows, components, environment):
    """(这一趟的行, {行号: Components})：多机配置开了 GPU（gpu_route_enabled）时，点名测试走崩溃判段、cargo 参数没开 gpu 的那几行换成补了 gpu 特性的
    （配 GPU 必用 GPU，不降级：不补的话二进制没编进 gpu，测试判段不跑、只录就返回 ok，改坏的那一处没判过），按补了之后的参数重算键
    （闭包照这一组特性收，特性进键）；之后起的 cargo 都照这份参数，真跑测试的那几次照旧交多机配置、带 gpu 特性的收尾补 --test-threads（with_gpu_test_threads、gpu_test_run）。
    原样的六段（Row.fields）不动：用时表与索引照它取名字。换了哪几行记在 key_context.gpu_route_lines（行号, 补的特性）。
    补了之后点名的测试不编的（#[cfg(not(feature = "gpu"))] 的用例，判的就是没编 gpu 的二进制）照原样留着，记在 key_context.gpu_route_unbuilt（行号, 为什么）。
    弄坏开关 gpu-route-feature-unadded：不补。"""
    if break_is(environment, "gpu-route-feature-unadded"):
        return rows, components
    enabled, configuration = gpu_route_enabled(environment)
    if not enabled:
        return rows, components
    key_context.gpu_route_configuration = configuration
    replaced, release_bases = {}, {}
    for row in rows:
        feature = gpu_route_feature_word(row, components[row.line_number], key_context)
        if feature is not None:
            # 补了 gpu 特性就命中重测试目标（文件头「重测试目标只许 release」）：补完特性再照判一次，没带 release 的补 --release
            featured, basis = release_forced_row(row._replace(arguments=with_feature(row.arguments, feature)), key_context.release_registered, environment)
            if basis is not None:
                release_bases[row.line_number] = basis
            replaced[row.line_number] = (featured, feature)
    try:
        key_context.prepare([row for row, _feature in replaced.values()])
    except mutation_units.ClosureRefused as error:
        labels = {f"{TABLE}:{row.line_number} {row.name}": line_number for line_number, (row, _feature) in replaced.items()}
        for _class_name, message, named in error.failures:
            for label in named:
                if labels.get(label) in replaced:
                    del replaced[labels[label]]
                    key_context.gpu_route_unbuilt.append((labels[label], message))
        key_context.prepare([row for row, _feature in replaced.values()])   # 剩下的照旧算；再有算不成的照常抛（不是补特性带出来的）
    if not replaced:
        return rows, components
    already_released = {line_number for line_number, _basis in key_context.release_lines}
    for line_number, (row, feature) in replaced.items():
        components[line_number] = key_context.components(row)
        key_context.gpu_route_lines.append((line_number, feature))
        if line_number in release_bases and line_number not in already_released:
            key_context.release_lines.append((line_number, release_bases[line_number]))
    return [replaced[row.line_number][0] if row.line_number in replaced else row for row in rows], components


class ReachabilityConfigurationError(Exception):
    """static-check 的截断表、豁免表或选路表里有函数在 <根> 的源码里找不到（文件头「点名测试够不着被改坏的函数」）。"""


class ReachabilityNotJudged(Exception):
    """static-check 的选路表、截断表、豁免表里的函数在 <根> 的源码里一个都没有（门禁样本那种小根）：这一趟没有判可达性，原因是异常的文字（文件头「点名测试够不着被改坏的函数」）。"""


def reachability_units_named(workspace, path, name):
    """工作区里 path 那份 .rs 里短名是 name 的 fn 单元；文件不在工作区里交空。"""
    rust_file = workspace.files.get(path)
    return [] if rust_file is None else [unit for unit in rust_file.units if unit.kind == "fn" and unit.short == name]


def reachability_table_names():
    """GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、REACHABILITY_CPU_FALLBACK_ROOTS 点名的全部函数：[(文件, 函数名)]。"""
    names = [(file, name) for _package, file, name in GPU_ROUTE_UNITS[0]] + list(REACHABILITY_CPU_FALLBACK_ROOTS[0])
    for source, target in REACHABILITY_GPU_ROUTE_STOPS[0]:
        names += ([source] if source is not None else []) + [target]
    return names


def reachability_names_missing(workspace, environment):
    """reachability_table_names 里在这一份工作区里找不到的函数（「文件::函数名」）。
    弄坏开关 reachability-stops-unverified：一个都不报（不核的样子：函数改了名，截断与豁免悄悄不起作用）。"""
    if break_is(environment, "reachability-stops-unverified"):
        return []
    return [f"{file}::{name}" for file, name in reachability_table_names() if not reachability_units_named(workspace, file, name)]


def reachability_stop_pairs(workspace, environment):
    """REACHABILITY_GPU_ROUTE_STOPS 在这一份工作区里落到的 (来处单元或 None, 单元)，交给 lib_function_units.py 的 rust_closure_of 的 stop_at。
    弄坏开关 reachability-stops-ignored：交空（不截）；reachability-edges-as-units：按边停的也按单元停（测试直接调的 CPU 判器入口也截掉）。"""
    if break_is(environment, "reachability-stops-ignored"):
        return ()
    edges_as_units = break_is(environment, "reachability-edges-as-units")
    pairs = []
    for source, target in REACHABILITY_GPU_ROUTE_STOPS[0]:
        targets = reachability_units_named(workspace, *target)
        if source is None or edges_as_units:
            pairs += [(None, unit) for unit in targets]
        else:
            pairs += [(origin, unit) for origin in reachability_units_named(workspace, *source) for unit in targets]
    return tuple(pairs)


def cpu_configuration_writer(workspace, closure, environment):
    """截过的闭包里写 CPU 配置的那一个单元（文件头「点名测试够不着被改坏的函数」豁免那一段）：交「文件::限定名」，没有交 None。
    串字面量含 ENABLE_GPU=0 的单元，测试代码与库代码都认（测试经库函数写只开 CPU 的配置，库函数在截过的闭包里）；空串加 multi-host 起头 .env 收尾的串只认测试代码。
    每个单元的字面量只扫一次，结果记在工作区对象上（vars(workspace)），同一份工作区上的几百行共用。
    弄坏开关 reachability-cpu-configuration-ignored：一律交 None；reachability-cpu-configuration-test-code-only：库代码里的 ENABLE_GPU=0 不认，只认测试代码。"""
    if break_is(environment, "reachability-cpu-configuration-ignored"):
        return None
    test_code_only = break_is(environment, "reachability-cpu-configuration-test-code-only")
    lfu = mutation_units.function_units()
    flags_of_units = vars(workspace).setdefault("cpu_configuration_flags_of_units", {})
    for unit in closure.order:
        target_id = closure.unit_targets.get(id(unit))
        if (unit.language != "rust" or unit.first_token is None or unit.kind == "doctest" or unit.kind in lfu.RUST_STRUCTURE_ONLY_KINDS
                or target_id is None):
            continue
        flags = flags_of_units.get(id(unit))
        if flags is None:
            rust_file = workspace.files[unit.file]
            values = [lfu.rust_string_value(rust_file.texts[index]) for index in range(unit.first_token, unit.last_token) if rust_file.kinds[index] == lfu.LITERAL]
            values = [value for value in values if value is not None]
            flags = flags_of_units[id(unit)] = (any(REACHABILITY_CPU_CONFIGURATION_TEXT in value for value in values),
                                                "" in values and any(value.startswith("multi-host") and value.endswith(".env") for value in values))
        mentions_text, empty_configuration = flags
        if not (mentions_text or empty_configuration):
            continue
        if (test_code_only or not mentions_text) and not lfu.rust_unit_is_test(workspace, unit, target_id):
            continue
        return f"{unit.file}::{unit.name}"
    return None


def cpu_fallback_root_in(workspace, closure, environment):
    """截过的闭包里有 REACHABILITY_CPU_FALLBACK_ROOTS 的哪一个：交「文件::函数名」，没有交 None。弄坏开关 reachability-fallback-roots-ignored：一律交 None。"""
    if break_is(environment, "reachability-fallback-roots-ignored"):
        return None
    for path, name in REACHABILITY_CPU_FALLBACK_ROOTS[0]:
        if any(id(unit) in closure.members for unit in reachability_units_named(workspace, path, name)):
            return f"{path}::{name}"
    return None


def reachability_cut_answer(request, environment):
    """一份截过的闭包（文件头「点名测试够不着被改坏的函数」）：request = (单元键, 工作区, 测试单元, 目标, 解析过的 cargo 参数, 要问的单元, stop_at)。
    交 (截过的闭包里有的那几个要问的单元, 豁免的 (哪一种, 凭据) 或 None)。cfg 与特性照 lib_mutation_units.py 的 UnitKeys.closure 那一份取（门禁给这一行算键用的同一组）。"""
    keys, workspace, unit, target_id, parsed, probes, stop_at = request
    lfu = mutation_units.function_units()
    cfg = [item for item in lfu.DEFAULT_RUST_CFG if not (parsed.release and item == "debug_assertions")]
    closure = lfu.rust_closure_of(workspace, unit, target_id, cfg, True, keys.features_of(parsed, target_id), parsed.no_default_features, stop_at=stop_at)
    members = {(member.kind, member.file, member.name) for member in closure.order}
    writer = cpu_configuration_writer(workspace, closure, environment)
    fallback = None if writer is not None else cpu_fallback_root_in(workspace, closure, environment)
    exempt = ("cpu-configuration", writer) if writer is not None else (("gpu-tables-refused", fallback) if fallback is not None else None)
    return frozenset(probe for probe in probes if probe in members), exempt


def _reachability_cut_task(index):
    """算截过的闭包的一格（并行时在 fork 出来的子进程里跑，单元与工作区是父进程那一份的拷贝）：交 (下标, 有的那几个要问的单元, 豁免, 算不成的那一句或 None, 回溯或 None)。
    lib_function_units.py 的 UsageError（入口在这一组 cfg 下不编）交那一句，那一行记没判；别的异常（算法自己的错）交回溯，父进程照抛。异常对象不交回父进程（不一定 pickle 得动）。"""
    try:
        present, exempt = reachability_cut_answer(_REACHABILITY_PARALLEL["requests"][index], _REACHABILITY_PARALLEL["environment"])
        return index, present, exempt, None, None
    except Exception as error:
        known = isinstance(error, mutation_units.function_units().UsageError)
        return index, frozenset(), None, f"{type(error).__qualname__}：{error}", None if known else traceback.format_exc()


def reachability_cut_answers(requests, environment):
    """每份 request 的 (有的那几个要问的单元, 豁免, 算不成的那一句或 None)，次序同 requests。不到 lib_mutation_units.py 的 PARALLEL_MINIMUM 份
    或 key_workers 只给 1 个时就地算，别的开 fork 出来的子进程并行算（同 lib_mutation_units.py 的 precompute_closures）。收回的份数对不上抛 RuntimeError。
    fork 之前父进程的对象冻进永久代（同 precompute_closures_from_shared_parent，文件头「算键的内存」）：子进程回收时不逐页碰父进程的工作区与闭包，不各抄一份。"""
    import gc
    workers = key_workers(environment)
    _REACHABILITY_PARALLEL.clear()
    _REACHABILITY_PARALLEL.update(requests=requests, environment=environment)
    frozen = False
    try:
        if workers <= 1 or len(requests) < mutation_units.PARALLEL_MINIMUM or multiprocessing.current_process().daemon:
            results = [_reachability_cut_task(index) for index in range(len(requests))]
        else:
            gc.collect()
            gc.freeze()
            frozen = True
            # 不用进程池（它按「模块名.函数名」pickle _reachability_cut_task，本模块被按文件路径装成别的名字时 pickle 不了）：forked_parts 不 pickle 函数
            results = [result for part in forked_parts(_reachability_cut_task, list(range(len(requests))), workers) for result in part]
    finally:
        if frozen:
            gc.unfreeze()
        _REACHABILITY_PARALLEL.clear()
    unexpected = [detail for _index, _present, _exempt, _failure, detail in results if detail]
    if unexpected:
        raise RuntimeError(f"算截过的闭包时出了算法自己的错（{len(unexpected)} 处）：\n" + "\n".join(unexpected))
    answers = {index: (present, exempt, failure) for index, present, exempt, failure, _detail in results}
    if sorted(answers) != list(range(len(requests))):
        raise RuntimeError(f"截过的闭包派出 {len(requests)} 份、收回 {len(answers)} 份")
    return [answers[index] for index in range(len(requests))]


def reachability_precompute(plans, arguments_field, environment, pool_facts):
    """plans 里每一份照 plan[arguments_field] 的 cargo 参数先把点名测试的闭包算好（按单元键分组，同 KeyContext.prepare，进程数见 key_workers）：
    与 KeyContext 同一份 precompute_closures_from_shared_parent（fork 之前共用词法单元、先算一条、冻进永久代，文件头「算键的内存」），子进程不各抄父进程的工作区。
    pool_facts 是这一趟 static-check 的一份 new_key_pool_facts（同一份工作区只换一次词串）。
    算不成的（ClosureRefused）这里不报：别的照算完，调用方逐行再问 reachability_closure 时照原话记没判。"""
    groups = collections.OrderedDict()
    for plan in plans:
        _keys, requests, labels = groups.setdefault(id(plan["keys"]), (plan["keys"], [], []))
        requests.append((plan["test"].unit, plan["test"].target, mutation_units.parse_cargo_arguments(plan[arguments_field])))
        labels.append(f"{TABLE}:{plan['row'].line_number} {plan['row'].name}")
    for keys, requests, labels in groups.values():
        with contextlib.suppress(mutation_units.ClosureRefused):
            precompute_closures_from_shared_parent(keys, requests, labels, key_workers(environment), environment, pool_facts)


def reachability_closure(plan, arguments_text):
    """(点名测试照这组 cargo 参数的 ClosureInfo, None) 或 (None, 算不成那一句)。"""
    try:
        return plan["keys"].closure(plan["test"].unit, plan["test"].target, mutation_units.parse_cargo_arguments(arguments_text)), None
    except mutation_units.function_units().UsageError as error:
        return None, f"点名测试 {plan['test'].full_name} 的闭包算不成：{error}"


def source_text_reader(data_units, path, environment):
    """点名测试的闭包里罩住被改坏的文件 path 的那个数据单元（文件头「点名测试够不着被改坏的函数」closure 那一条）：data-file 的路径就是 path，
    或 data-directory 的目录在 path 之上；交「data-file <路径>」这一类，没有交 None。data_units 是闭包里的 (种类, 路径) 集合。
    弄坏开关 reachability-source-text-ignored：一律交 None，只拿函数单元比，读源码文本的测试报够不着。"""
    if break_is(environment, "reachability-source-text-ignored"):
        return None
    if ("data-file", path) in data_units:
        return f"data-file {path}"
    for kind, directory in sorted(data_units):
        if kind == "data-directory" and path.startswith(directory.rstrip("/") + "/"):
            return f"data-directory {directory}"
    return None


def reachability_refused_reason(plan, where, probes):
    """闭包拒算的行判不了时的原因句：where 是「已收部分」（全闭包）或「截过的闭包已收部分」，probes 是被改坏的单元（种类、文件、限定名）。
    拒算的闭包不全，没收进来的那一段可能调得到被改坏的单元，所以没落在已收部分里的不报够不着，只报判不了。"""
    mutated = "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes))
    return (f"点名测试 {plan['test'].full_name} 闭包拒算、{where}里没有被改坏的单元（{mutated}）：拒算原因 {plan['refusal']}；"
            f"拒算的闭包不全，够不够得着判不了")


def reachability_plans_by_closure(plans):
    """逐个交 (plan, 成员集合, 数据单元集合)：几行点名同一份闭包的排在一起（组按头一次出现的次序，组内照原次序），成员集合（种类、文件、限定名）
    每份闭包只摊开一次、这一组判完就放掉，不给几千份闭包同时各留一份（文件头「点名测试够不着被改坏的函数」内存那一句）。"""
    groups = collections.OrderedDict()
    for plan in plans:
        groups.setdefault(id(plan["closure"]), []).append(plan)
    for identity in list(groups):
        grouped = groups.pop(identity)
        member_set = frozenset((member.kind, member.file, member.name) for member in grouped[0]["closure"].members)
        data_set = frozenset((kind, name) for kind, _file, name in member_set if kind in ("data-file", "data-directory"))
        for plan in grouped:
            yield plan, member_set, data_set


def workspace_file_task(arguments):
    """建工作区按文件切片并行的一份（prebuilt_workspace 在 fork 出来的子进程里调）：(根, 相对根的路径, 包目录) → (路径, 文本的 sha256, 包目录, RustFile 或 None)。
    读法与切法同 lib_function_units.py 的 RustWorkspace.file（utf-8，解不了的字节照 surrogateescape 留着；RustFile(根, 路径, 文本, 包目录)）；读不了交 None，本进程照原样现读。"""
    root, path, directory = arguments
    try:
        with open(os.path.join(root, path), encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
    except OSError:
        return path, None, directory, None
    digest = hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()
    return path, digest, directory, mutation_units.function_units().RustFile(root, path, text, directory)


def workspace_file_items(root):
    """建工作区要切的每份 .rs：[(根, 相对根的路径, 包目录)]，照 lib_function_units.py 的 RustWorkspace.all_files 列（每个包目录下的 .rs，根上那个包不收别的包目录下的），
    包目录照 RustWorkspace.crate_of_path 取（路径前缀最长的那个包）。"""
    lfu = mutation_units.function_units()
    crates = lfu.discover_rust_crates(root)
    items = {}
    for crate in crates:
        for path in lfu.files_under(root, (".rs",), [crate.directory or "."]):
            if crate.directory == "" and any(path.startswith(other.directory + "/") for other in crates if other.directory):
                continue
            owner = None
            for candidate in crates:
                prefix = candidate.directory + "/" if candidate.directory else ""
                if path.startswith(prefix) and (owner is None or len(candidate.directory) > len(owner.directory)):
                    owner = candidate
            items.setdefault(path, (root, path, owner.directory if owner is not None else os.path.dirname(path)))
    return [items[path] for path in sorted(items)]


def prebuilt_workspace(keys, view, workers, environment):
    """static-check 建工作区按文件切片并行（文件头 static-check「并行」）：keys 那份单元键的 view 视图（lib_function_units.py 的 PRODUCT_BEHAVIOR 或 FORMAL_VERIFICATION）
    还没建时，每份 .rs 的 RustFile 在 fork 出来的子进程里按文件切片切好（workspace_file_task、static_check_slices），本进程照原样建 RustWorkspace，载入每份文件时
    文本的 sha256 与包目录都与子进程那一份相同就取子进程切好的，不同（中途被改了）或子进程没有的照原样现切；产品行为视图的剥法、限定名、索引照旧在本进程里做。
    建好放进单元键（_workspace 或 _formal_workspace），之后取 workspace / formal_workspace 就是它。路数不到 2、份数不到 PARALLEL_MINIMUM、本进程是守护进程时不切，
    由单元键头一次取时照原样建；子进程出错打一行说明、照原样建。这一趟取了几份记在 prebuilt_workspace.files（{视图: 份数}，自证核它）。
    弄坏开关 workspace-files-unsliced：不切，照原样建；rust-file-replaced-on-entry-module：替身换在 lib_function_units.py 上、不换在 RustWorkspace 所在的模块上，一份都取不到。"""
    import gc
    lfu = mutation_units.function_units()
    inner = getattr(keys, "wrapped_unit_keys", keys)
    formal = view == lfu.FORMAL_VERIFICATION
    already_built = inner._formal_workspace if formal else inner._workspace
    if already_built is not None or break_is(environment, "workspace-files-unsliced") or workers <= 1 or multiprocessing.current_process().daemon:
        return
    items = workspace_file_items(inner.root)
    if len(items) < mutation_units.PARALLEL_MINIMUM:
        return
    gc.collect()
    gc.freeze()
    try:
        answers = static_check_slices(workspace_file_task, items, workers, environment)
    except RuntimeError as error:
        note(f"  ! 建工作区按文件切片并行没成（{error}），照原样在本进程里逐份切")
        return
    finally:
        gc.unfreeze()
    ready = {path: (digest, directory, rust_file) for path, digest, directory, rust_file in answers if rust_file is not None}
    # RustWorkspace 载入每份文件读的 RustFile 是它自己所在模块里那一份（lfu 只是再导出），替身要换在那个模块的全局名上；
    # 弄坏开关 rust-file-replaced-on-entry-module：换在 lfu 上，RustWorkspace 读不到它，子进程切好的一份都取不到
    holder = lfu if break_is(environment, "rust-file-replaced-on-entry-module") else sys.modules[lfu.RustWorkspace.__module__]
    real_rust_file, taken = holder.RustFile, []

    def rust_file_from_slices(root, path, text, crate_directory):
        found = ready.pop(path, None)
        if (found is not None and root == inner.root and found[1] == crate_directory
                and found[0] == hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()):
            taken.append(path)
            return found[2]
        return real_rust_file(root, path, text, crate_directory)
    holder.RustFile = rust_file_from_slices
    try:
        workspace = lfu.RustWorkspace(inner.root, view)
    finally:
        holder.RustFile = real_rust_file
    if formal:
        inner._formal_workspace = workspace
    else:
        inner._workspace = workspace
    prebuilt_workspace.files = dict(getattr(prebuilt_workspace, "files", {}))
    prebuilt_workspace.files[view] = len(taken)


def reachability_row_answer(item):
    """一行的被改坏的单元与点名测试（reachability_resolved_rows 按行切片时在 fork 出来的子进程里调）：(行, 单元键, 视图) →
    (被改坏的单元或 None, 这一行在形式化视图上新记下的被改坏的单元, 点名测试的（目标, 全名）或 None, 拒算那一句或 None)。只交串与 UnitReference，不交单元对象。"""
    row, keys, view = item
    inner = getattr(keys, "wrapped_unit_keys", keys)
    before = set(inner._formal_references)
    mutated, refusal = keys.mutated_units(row.path, row.old)
    added = sorted(set(inner._formal_references) - before)
    test = None
    if refusal is None:
        test, refusal = keys.resolve_test(row.arguments, row.expected, view=view)
    return mutated, added, (test.target, test.full_name) if test is not None else None, refusal


def reachability_resolved_rows(items, workers, environment):
    """static-check 可达性逐行那一段（文件头「点名测试够不着被改坏的函数」的「并行」）：items 是 [(行, 单元键, 视图)]（视图照 row_view），
    → 逐行 (被改坏的单元或 None, 点名测试 TestUnit 或 None, 拒算那一句或 None)，次序同 items。
    fork 之前在本进程里把要用的工作区建好（各单元键的产品行为视图，有一行取形式化视图就连同形式化视图；还没建的照 prebuilt_workspace 按文件切片并行切），
    冻进永久代；按行切片并行（static_check_slices），
    子进程只交串与 UnitReference。并回之后本进程补上子进程里的两样：这一行在形式化视图上被改坏的单元记进单元键（_formal_references 与它们的单元，unit_of 记），
    点名测试的单元对象照（目标, 全名）从本进程的 test_units 取（闭包缓存按单元对象认，要本进程那一份）。切了几行记在 reachability_resolved_rows.forked_rows（自证核它）。
    弄坏开关 reachability-rows-unsliced：逐行在本进程里算，不切片。"""
    import gc
    lfu = mutation_units.function_units()
    built = set()
    for _row, keys, view in items:
        inner = getattr(keys, "wrapped_unit_keys", keys)
        prebuilt_workspace(inner, lfu.PRODUCT_BEHAVIOR, workers, environment)
        built.add(id(inner.workspace))   # property：没切过的头一次取时照原样切，这里在 fork 之前切好
        if view == lfu.FORMAL_VERIFICATION:
            prebuilt_workspace(inner, lfu.FORMAL_VERIFICATION, workers, environment)
            built.add(id(inner.formal_workspace))
    if break_is(environment, "reachability-rows-unsliced"):
        workers = 1
    sliced = workers > 1 and len(items) >= mutation_units.PARALLEL_MINIMUM and not multiprocessing.current_process().daemon
    reachability_resolved_rows.forked_rows = len(items) if sliced else 0
    if sliced:
        gc.collect()
        gc.freeze()
    try:
        answers = static_check_slices(reachability_row_answer, items, workers, environment)
    finally:
        if sliced:
            gc.unfreeze()
    resolved = []
    for (row, keys, view), (mutated, added, test_name, refusal) in zip(items, answers):
        inner = getattr(keys, "wrapped_unit_keys", keys)
        if added:
            inner._formal_references.update(added)
            for reference in added:
                inner.unit_of(reference)
        test = None
        if test_name is not None:
            target_id, full_name = test_name
            target = inner.workspace.targets.get(target_id)
            found = next((unit for name, unit in inner.test_units(target, view == lfu.FORMAL_VERIFICATION) if name == full_name), None) if target is not None else None
            if found is None:
                refusal = f"点名的测试 {full_name}（目标 {target_id}）在本进程的工作区里对不回（切片并行并回时错了位）"
            else:
                test = mutation_units.TestUnit(found, target_id, full_name)
        resolved.append((mutated, test, refusal))
    return resolved


def reachability_of_rows(root, rows, anchors, environment):
    """static-check 的可达性判法（文件头「点名测试够不着被改坏的函数」）：→ (够不着的行, 豁免的行, 没判的行, 截过闭包的行数, 多机配置, 开没开 GPU)。
      anchors 是 anchor_problems_by_line 给这几行判过的锚点（{行号: 为什么或 None}）。
      够不着的行 [(行号, 变异名, 哪一路 gpu-route 或 closure, 被改坏的单元, 点名测试, 门禁给这一行用的 cargo 参数)]；豁免的行 [(行号, 变异名, 哪一种, 凭据)]；
      没判的行 [(行号, 变异名, 为什么)]；三样都按行号排。截断表、豁免表或选路表里有函数在 <根> 的源码里找不到抛 ReachabilityConfigurationError；
      这些函数在 <根> 的源码里一个都没有（门禁样本那种小根）抛 ReachabilityNotJudged，一行都不判；
      多机配置在而读不了抛 admission.py 的 InputManifestError。弄坏开关 reachability-closure-unchecked：不判「闭包里根本没有」那一路。
      闭包拒算（ClosureInfo.key 是 None）的行照已收的成员同一套判：被改坏的单元或读到的源码文本落在已收部分里算够得着；不在的报没判（reachability_refused_reason），
      不报够不着（全闭包与截过的闭包两处一样）。弄坏开关 reachability-refused-unjudged：拒算的一律报没判。"""
    lfu = mutation_units.function_units()
    seconds, clock = {}, [time.monotonic()]
    reachability_of_rows.seconds = seconds   # 分段用时（command_static_check 打到标准错误）

    def lap(name):
        now = time.monotonic()
        seconds[name] = seconds.get(name, 0.0) + now - clock[0]
        clock[0] = now
    keys_by_directory = {"": mutation_units.UnitKeys(root)}
    root_keys = keys_by_directory[""]
    prebuilt_workspace.files = {}
    prebuilt_workspace(root_keys, lfu.PRODUCT_BEHAVIOR, key_workers(environment), environment)   # 全仓 .rs 按文件切片并行切（文件头 static-check「并行」）
    table_names = reachability_table_names()
    if not any(reachability_units_named(root_keys.workspace, file, name) for file, name in table_names):
        raise ReachabilityNotJudged(f"选路表、截断表、豁免表点名的 {len(table_names)} 处函数在 {root} 的源码里一个都没有（{table_names[0][0]}::{table_names[0][1]} 等），没有可判的路")
    missing = reachability_names_missing(root_keys.workspace, environment)
    if missing:
        raise ReachabilityConfigurationError("、".join(missing))
    lap("建工作区")
    enabled, configuration = gpu_route_enabled(environment)
    unreachable, exempt, unjudged, plans, resolving = [], [], [], [], []
    for row in rows:
        why = anchors[row.line_number]
        keys, refusal = (None, f"锚点失效：{why}") if why is not None else row_unit_keys(root, row, environment, keys_by_directory)
        if refusal is not None:
            unjudged.append((row.line_number, row.name, refusal))
            continue
        resolving.append((row, keys, row_view(keys, row)))   # row_view 读的那一份记在单元键里（source_bytes），mutated_units 不再读
    for (row, keys, view), (mutated, test, refusal) in zip(resolving, reachability_resolved_rows(resolving, key_workers(environment), environment)):
        if refusal is not None:
            unjudged.append((row.line_number, row.name, refusal))
            continue
        plans.append(dict(row=row, keys=keys, mutated=mutated, test=test, view=view, arguments=row.arguments))
    lap("逐行被改坏的单元与点名测试")
    pool_facts = new_key_pool_facts()
    reachability_precompute(plans, "arguments", environment, pool_facts)
    judged = []
    for plan in plans:
        closure, why = reachability_closure(plan, plan["arguments"])
        if closure is None:
            unjudged.append((plan["row"].line_number, plan["row"].name, why))
            continue
        plan["closure"] = closure
        word = gpu_route_feature_word_of(plan["arguments"], closure) if enabled else None
        if word is not None:
            plan["gpu_arguments"] = with_feature(plan["arguments"], word)
        judged.append(plan)
    # 开 GPU 时门禁给这几行补 gpu 特性（gpu_route_rows），闭包照补了之后的参数算；补了就不编的（cfg(not(feature = "gpu")) 的用例）门禁照原样跑，
    # 判段不跑、CPU 判器同样不跑，照原样参数的闭包截
    reachability_precompute([plan for plan in judged if "gpu_arguments" in plan], "gpu_arguments", environment, pool_facts)
    for plan in judged:
        if "gpu_arguments" in plan:
            closure, _why = reachability_closure(plan, plan["gpu_arguments"])
            if closure is not None:
                plan["closure"], plan["arguments"] = closure, plan["gpu_arguments"]
    lap("点名测试的闭包")
    stop_pairs, cut_requests, cut_plans = {}, collections.OrderedDict(), []
    for plan, member_set, data_set in reachability_plans_by_closure(judged):
        row, closure = plan["row"], plan["closure"]
        if closure.key is None:   # 闭包拒算：键算不出，但已收的成员照用（文件头「点名测试够不着被改坏的函数」拒算那一条）
            plan["refusal"] = closure.refusals[0] if closure.refusals else "没写原因"
            if break_is(environment, "reachability-refused-unjudged"):
                unjudged.append((row.line_number, row.name, f"点名测试 {plan['test'].full_name} 的闭包拒算（{plan['refusal']}），够不够得着判不了"))
                continue
        probes = frozenset((reference.kind, reference.file, reference.name) for reference in plan["mutated"])
        present = probes & member_set
        if source_text_reader(data_set, row.path, environment) is not None:
            continue   # 点名测试读被改坏那份文件的源码文本（include_str! 或运行期读文件、读目录）：读到的文本就看得见这一处改动，够得着，也不看 GPU 那一路
        if not present and "refusal" in plan:   # 拒算的闭包不全：已收部分里没有，说不准够不够得着，不报够不着
            unjudged.append((row.line_number, row.name, reachability_refused_reason(plan, "已收部分", probes)))
            continue
        if not present and not break_is(environment, "reachability-closure-unchecked"):
            unreachable.append((row.line_number, row.name, "closure", "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes)),
                                plan["test"].full_name, plan["arguments"]))
            continue
        if not enabled or plan["keys"] is not root_keys or gpu_route_package(closure) is None:
            continue   # 没开 GPU、或判段不经选路：闭包里有它就算够得着
        workspace = root_keys.formal_workspace if plan["view"] == lfu.FORMAL_VERIFICATION else root_keys.workspace
        if id(workspace) not in stop_pairs:
            stop_pairs[id(workspace)] = reachability_stop_pairs(workspace, environment)
        parsed = mutation_units.parse_cargo_arguments(plan["arguments"])
        request_key = (id(workspace),) + root_keys.cache_key(plan["test"].unit, plan["test"].target, parsed)
        if request_key not in cut_requests:
            cut_requests[request_key] = [workspace, plan["test"].unit, plan["test"].target, parsed, set()]
        cut_requests[request_key][4] |= present or probes
        cut_plans.append((plan, present or probes, request_key))
    lap("按闭包成员判够不够得着")
    request_keys = list(cut_requests)
    answers = dict(zip(request_keys, reachability_cut_answers(
        [(root_keys, workspace, unit, target_id, parsed, frozenset(probes), stop_pairs[id(workspace)])
         for workspace, unit, target_id, parsed, probes in (cut_requests[key] for key in request_keys)], environment)))
    for plan, probes, request_key in cut_plans:
        row = plan["row"]
        present, exemption, failure = answers[request_key]
        if failure is not None:
            unjudged.append((row.line_number, row.name, f"点名测试 {plan['test'].full_name} 截过的闭包算不成：{failure}"))
        elif exemption is not None:
            # 认得出豁免的行不判 gpu-route（够不够得着都打 REACHABILITY_EXEMPT：这一行的判段不走或不只走 GPU 路，截过的闭包不代表它）
            exempt.append((row.line_number, row.name) + exemption)
        elif not present & probes and "refusal" in plan:   # 全闭包拒算：截过的闭包已收部分里没有，也说不准（拒算的那一段可能经 CPU 判器连到）
            unjudged.append((row.line_number, row.name, reachability_refused_reason(plan, "截过的闭包已收部分", probes)))
        elif not present & probes:
            unreachable.append((row.line_number, row.name, "gpu-route", "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes)),
                                plan["test"].full_name, plan["arguments"]))
    lap("截过的闭包")
    return sorted(unreachable), sorted(exempt), sorted(unjudged), len(cut_plans), configuration, enabled


def decide_rows(library, rows, components, key_context, environment):
    """每一行 (Decision, 记录)：与 run_stage 同一份 row_decision。环境里开了分步进度（merge_progress_variable）就把「读库」这一步逐行打进度。"""
    progress = merge_step_open(environment, "读库", len(rows))
    decided = {}
    for done, row in enumerate(rows, 1):
        comp = components[row.line_number]
        previous = library.read("index", row_identity(row)) if comp.row_key else None
        decided[row.line_number] = row_decision(library, comp, row, key_context, previous)
        merge_step_row_done(progress, done)
    reusable = sum(1 for decision, _record in decided.values() if decision.reuse)
    merge_step_close(progress, f"读到键相同的判定 {reusable} 行、读不到 {len(decided) - reusable} 行")
    return decided


def impact_unit_name(reference):
    """牵连面那一行里一个单元的名字：<文件>::<限定名>；数据文件的限定名就是路径，只写路径。"""
    return reference.file if reference.name == reference.file else f"{reference.file}::{reference.name}"


def impact_summary(rows, components, key_context, environment, anchor_broken_count=0):
    """牵连面那一行（文件头「牵连面」）：每一行的键依赖的单元（KeyContext 为它算好的点名测试闭包，连同被改坏的单元）按（种类、文件、限定名）计数，
    报进的行数最多的那个单元、前 5 个与一次改动最坏牵连几行。只报数，不判红。弄坏开关 impact-row-dropped：算进的头一行不数。"""
    counts, names, counted = collections.Counter(), {}, 0
    for row in rows:
        comp = components.get(row.line_number)
        closure = key_context.closures.get(comp.test_closure) if comp is not None and comp.test_closure else None
        if closure is None:
            continue
        counted += 1
        if counted == 1 and break_is(environment, "impact-row-dropped"):
            continue
        units = {}
        for reference in list(closure.members) + list(comp.mutated or []):
            units[(reference.kind, reference.file, reference.name)] = impact_unit_name(reference)
        for identity, name in units.items():
            counts[identity] += 1
            names[identity] = name
    tail = f"（算了 {counted} 行的闭包；拒算与锚点失效的 {len(rows) - counted + anchor_broken_count} 行没有闭包、没算进）"
    if not counts:
        return f"牵连最大的函数单元：没有（一行的闭包都没算出来）；一次改动最坏牵连 0 行{tail}"
    ranked = sorted(counts.items(), key=lambda item: (-item[1], names[item[0]], item[0]))
    top_identity, top_count = ranked[0]
    top_five = "、".join(f"{names[identity]} {count}" for identity, count in ranked[:5])
    return f"牵连最大的函数单元：{names[top_identity]} 进了 {top_count} 行的闭包（前 5：{top_five}）；一次改动最坏牵连 {top_count} 行{tail}"


def selftest_gpu_route_feature_cell(selftest, work):
    """开 GPU 时崩溃判段那几行补 gpu 特性（gpu_route_rows，文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：选路函数换成样本 crate 里的
    judge_not_run_without_the_gpu_feature（GPU_ROUTE_UNITS[0] 临时换掉），多机配置 ENABLE_GPU=1、两行都不带特性：点名测试够得着选路函数的那一行
    起的每一次 cargo 都带 --features gpu（基线、变异的先编与跑），够不着的那一行一次都不带；
    点名 #[cfg(not(feature = "gpu"))] 用例的那一行（也够得着选路函数）补了就不编，照原样跑、一次都不带；stderr 点名补了哪一行、哪一行补不上；
    行键照补了之后的特性算：配置换成 ENABLE_GPU=0 时那一行的键不同、参数不带特性，另一行的键相同。弄坏开关 gpu-route-feature-unadded（不补）下这一格判错。"""
    repository = os.path.join(work, "gpu-route-feature")
    build_selftest_repository(repository, SELFTEST_GPU_ROUTE_ROWS, source=SELFTEST_GPU_ROUTE_SOURCE)
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    gpu_off = os.path.join(work, "gpu-route-off", "multi-host.env")
    selftest_write(gpu_off, "# 自证用的多机配置：GPU 关\nENABLE_GPU=0\n")
    saved = GPU_ROUTE_UNITS[0]
    try:
        GPU_ROUTE_UNITS[0] = (("tiny", "crates/tiny/src/lib.rs", "judge_not_run_without_the_gpu_feature"),)
        exit_code, stdout, stderr, invocations = selftest.run(repository)
        table_rows = read_table(os.path.join(repository, TABLE), selftest.base_environment)[0]
        _context, on_rows, on_components, _library = plan_rows(repository, table_rows, dict(selftest.base_environment))
        _context, off_rows, off_components, _library = plan_rows(repository, table_rows, dict(selftest.base_environment, GATE_MUTATION_MULTI_HOST_CONFIG=gpu_off))
    finally:
        GPU_ROUTE_UNITS[0] = saved
    enumeration, plain, unbuilt = (row.line_number for row in table_rows)
    calls = [line.split("\t")[0] for line in invocations]
    enumeration_calls = [line for line in calls if "enumeration_judges" in line]
    gpu_calls = [line for line in calls if "--features gpu" in line]
    route_note = next((line for line in stderr.splitlines() if "补上 gpu 特性起" in line), "")
    selftest.check("开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性：配置 ENABLE_GPU=1、行不带特性，起的 cargo 带 --features gpu，够不着选路函数的那一行不带；"
                   "带 gpu 特性就不编的用例（cfg(not(feature = \"gpu\"))）那一行照原样跑；stderr 点名补了哪一行、哪一行补不上；"
                   "行键照补了之后的特性算（GPU 关时那一行的键不同、另一行相同）",
                   exit_code == 0 and "计数：抓到 3 条、没红 0 条" in stdout
                   and any("--show-output" in line and "enumeration_judges" in line for line in gpu_calls)
                   and enumeration_calls and all("--features gpu" in line for line in enumeration_calls)
                   and not any("plain_sum" in line or "without_the_gpu_feature_routes_to_not_run" in line for line in gpu_calls)
                   and f"{TABLE}:{enumeration}（--features gpu）" in route_note and f"{TABLE}:{plain}（" not in route_note
                   and f"{TABLE}:{unbuilt} 点名的测试走崩溃判段，补 gpu 特性之后不编" in stderr
                   and "--features gpu" in {row.line_number: row for row in on_rows}[enumeration].arguments
                   and "--features" not in {row.line_number: row for row in off_rows}[enumeration].arguments
                   and on_components[enumeration].row_key and on_components[enumeration].row_key != off_components[enumeration].row_key
                   and on_components[plain].row_key == off_components[plain].row_key,
                   f"退 {exit_code}；带 --features gpu 的 {gpu_calls}；点名 enumeration_judges 的 {enumeration_calls}；"
                   f"键 开 {on_components[enumeration].row_key} / 关 {off_components[enumeration].row_key}；{stdout[-300:]}{stderr[-500:]}")


def selftest_gpu_test_threads_cell(selftest, work):
    """带 gpu 特性的格测试参数收尾带 --test-threads（with_gpu_test_threads，文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：
    五行同跑一趟（SINGLEFS_THREAD_CAP=7 当登记值）：点名放开名单里的测试目标（本模块的 GPU_TEST_THREADS_RELEASED_TARGETS）的 gpu 行，
    基线、变异、插桩跑真跑测试的每一次都收尾带 `--test-threads 7`；--lib 的 gpu 行带 `--test-threads 1`；不带 gpu 的行一次都不带；
    gpu 行自己带 --exact 的，--exact 与 --test-threads 各只出现一次（点名用例在前、--test-threads 收尾）；gpu 行自己写了 --test-threads=3 的原样（不再补）；
    只编不跑的（--no-run）一次都不带。线程参数不进行键：登记值 7 换成 9，五行的键都不变。弄坏开关 gpu-test-threads-unadded（不补）下这一格判错。"""
    released = GPU_TEST_THREADS_RELEASED_TARGETS[0]
    root = os.path.join(work, "gpu-test-threads")
    build_selftest_repository(root, [row.replace("{target}", released) for row in SELFTEST_GPU_RELEASED_TARGETS_ROWS], extra_source=SELFTEST_GPU_RELEASED_TARGETS_SOURCE)
    selftest_write(os.path.join(root, "crates", "tiny", "tests", f"{released}.rs"), "mod tests {\n    #[test]\n    fn released_agrees() { assert_eq!(tiny::shared(), 2); }\n}\n")
    selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    exit_code, stdout, stderr, invocations = selftest.run(root, SINGLEFS_THREAD_CAP="7")
    table_rows = read_table(os.path.join(root, TABLE), selftest.base_environment)[0]
    _context, _rows, seven_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment, SINGLEFS_THREAD_CAP="7"))
    _context, _rows, nine_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment, SINGLEFS_THREAD_CAP="9"))

    def libtest_tail(line):
        words = line.split("\t")[0].split()
        return words[words.index("--") + 1:] if "--" in words else []

    def real_runs(test):
        """起了点名这条测试、真跑测试的每一次（--no-run 的先编不算）的 libtest 参数。"""
        return [libtest_tail(line) for line in invocations if "--no-run" not in line.split("\t")[0].split()
                and any(word in (test, "tests::" + test) for word in libtest_tail(line))]

    def thread_words(tail):
        """libtest 参数里的 --test-threads 开关词（--test-threads 或 --test-threads=N，不含分开写时它的值）。"""
        return [word for word in tail if word == "--test-threads" or word.startswith("--test-threads=")]
    released_runs, lib_runs, plain_runs = real_runs("released_agrees"), real_runs("doubles"), real_runs("adds")
    exact_runs, own_runs = [tail for tail in real_runs("tests::halves") if "--exact" in tail], real_runs("negates")
    compile_runs = [line for line in invocations if "--no-run" in line.split("\t")[0].split()]
    selftest.check("带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带：基线、变异、插桩跑每一次都照办，自己带 --exact 或 --test-threads=N 的不重复、各只一个，"
                   "只编不跑的不带，登记值换了行键不变",
                   exit_code == 0 and len(released_runs) >= 3 and all(tail[-2:] == ["--test-threads", "7"] for tail in released_runs)
                   and len(lib_runs) >= 3 and all(tail[-2:] == ["--test-threads", "1"] for tail in lib_runs)
                   and plain_runs and all(not thread_words(tail) for tail in plain_runs)
                   and len(exact_runs) >= 1 and all(tail.count("--exact") == 1 and thread_words(tail) == ["--test-threads"] and tail[-2:] == ["--test-threads", "1"]
                                                    and tail.index("--exact") < tail.index("--test-threads") for tail in exact_runs)
                   and len(own_runs) >= 3 and all(thread_words(tail) == ["--test-threads=3"] for tail in own_runs)
                   and compile_runs and all("--test-threads" not in line for line in compile_runs)
                   and all(seven_components[number].row_key and seven_components[number].row_key == nine_components[number].row_key for number in seven_components),
                   f"退 {exit_code}；放开目标 {released_runs}；--lib {lib_runs}；不带 gpu {plain_runs}；带 --exact {exact_runs}；自带 --test-threads=3 {own_runs}；"
                   f"先编 {compile_runs}；{stdout[-300:]}{stderr[-300:]}")


def selftest_release_repository(work, name, rows):
    """→ 小仓：带 tiny 的集成测试目标 tiny_crash_stream（.cargo/ 里有 gpu 那几行要的 [env]）；小仓自己的登记表（.claude/gate.d/stage-inputs.tsv）只登记一条点名那个目标的
    崩溃枚举用例，加本模块所在那一份仓的登记表里本阶段那一行（拷贝范围）。static-check 的两格读它；run_stage 的那一格（selftest_release_forced_cell）把 RELEASE_REGISTRY_ROOT 换成它。"""
    repository = os.path.join(work, name)
    build_selftest_repository(repository, rows, extra_source=SELFTEST_RELEASE_SOURCE)
    selftest_write(os.path.join(repository, "crates", "tiny", "tests", "tiny_crash_stream.rs"),
                   "mod tests {\n    #[test]\n    fn streams_agree() { assert_eq!(tiny::stream_total(), 7); }\n}\n")
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    selftest_write_stage_registration_table(repository, SELFTEST_RELEASE_REGISTRY)
    return repository


def selftest_release_forced_cell(selftest, work):
    """门禁补 --release（release_forced_row，文件头「重测试目标只许 release」）：三行同跑一趟——点名登记的崩溃枚举用例目标的一行、带 gpu 特性的一行、
    两样都不中的一行；前两行起的每一次 cargo（基线、变异、插桩编译与插桩跑）`--` 之前都带 --release、只带一个，第三行一次都不带；stderr 那一行点名前两行、不点第三行。
    登记表在样本仓里；run_stage 读 RELEASE_REGISTRY_ROOT，这一格把它指到样本仓、跑完换回。弄坏开关 release-unforced（不补）下这一格判错。"""
    repository = selftest_release_repository(work, "release-forced", [SELFTEST_RELEASE_CRASH_ROW, SELFTEST_GPU_FEATURE_ROW, SELFTEST_TABLE_ROWS[0]])
    saved = RELEASE_REGISTRY_ROOT[0]
    try:
        RELEASE_REGISTRY_ROOT[0] = repository
        exit_code, stdout, stderr, invocations = selftest.run(repository)
    finally:
        RELEASE_REGISTRY_ROOT[0] = saved

    def cargo_words(line):
        words = line.split("\t")[0].split()
        return words[:words.index("--")] if "--" in words else words
    calls = [cargo_words(line) for line in invocations]
    hit_calls = [words for words in calls if "tiny_crash_stream" in words or "gpu" in words]
    plain_calls = [words for words in calls if "tiny_crash_stream" not in words and "gpu" not in words and "-p" in words]
    release_note = next((line for line in stderr.splitlines() if "重测试目标只许 release" in line), "")
    selftest.check("重测试目标只许 release：命中的两行（crash-case 目标、gpu 特性）起的每一次 cargo 都带 --release、只带一个，不命中的那一行一次都不带，stderr 点名补了哪两行",
                   exit_code == 0 and len(hit_calls) >= 4 and all(words.count("--release") == 1 for words in hit_calls)
                   and plain_calls and not any("--release" in words for words in plain_calls)
                   and f"{TABLE}:2（crash-case 目标 tiny:tiny_crash_stream" in release_note and f"{TABLE}:3（gpu 特性 gpu）" in release_note
                   and f"{TABLE}:4（" not in release_note,
                   f"退 {exit_code}；命中的 {hit_calls}；不命中的 {plain_calls}；stderr 那一行 {release_note!r}；{stdout[-300:]}{stderr[-300:]}")


def selftest_default_features_off_cell(selftest, work):
    """验证包的测试目标不编装置二进制（default_features_off_row，文件头「验证包的测试目标不编装置二进制」）：
    ① 判法本身（不起 cargo）：-p 只点名 singlefs-verification、点名 --test、没有别的选目标的选项、没自己写 --no-default-features 的参数，在第一个 `--` 之前补上
       --no-default-features（没有 `--` 接在末尾，带 --release、--features gpu、几个 --test 的照样补）；--lib、--lib --test、--bin、--bins、--tests、--all-targets、
       没点名目标、自己已写、点名别的包或几个包、没点名包的参数一个字不变；
    ② 一趟真跑（假 cargo，包名换成 singlefs-verification 的样本 crate）：点名 --test 的一行起的每一次 cargo（基线、变异、插桩编译与插桩跑）`--` 之前都带一个 --no-default-features，
       点名 --lib 的一行一次都不带；plan_rows 交回的参数同样如此，补了的那一行的键与不补的不同、--lib 那一行的键不变（--no-default-features 进行键的开关）。
    弄坏开关 no-default-features-unadded（不补）下这一格判错。"""
    environment = dict(selftest.base_environment)

    def planned(arguments):
        return default_features_off_row(Row(1, "m", "crates/singlefs-verification/src/lib.rs", "a", "b", arguments, "t", ("m", "p", "a", "b", arguments, "t")), environment).arguments
    package = "-p singlefs-verification"
    changed = [(f"{package} --test crash_target -- --include-ignored --exact the_case", f"{package} --test crash_target --no-default-features -- --include-ignored --exact the_case"),
               (f"{package} --release --features gpu --test crash_target -- x", f"{package} --release --features gpu --test crash_target --no-default-features -- x"),
               (f"{package} --test one --test two -- x", f"{package} --test one --test two --no-default-features -- x"),
               (f"{package} --test=one -- x", f"{package} --test=one --no-default-features -- x"),
               (f"{package} --test crash_target", f"{package} --test crash_target --no-default-features"),
               (f"{package} --test crash_target -- --lib --bin", f"{package} --test crash_target --no-default-features -- --lib --bin")]
    unchanged = [f"{package} --lib -- x", f"{package} --lib --test crash_target -- x", f"{package} --bin apparatus -- x", f"{package} --bins -- x", f"{package} --tests -- x",
                 f"{package} --all-targets -- x", f"{package} -- x", f"{package} --test crash_target --no-default-features -- x", f"{package} --release --lib -- x",
                 "-p tiny --test crash_target -- x", f"{package} -p tiny --test crash_target -- x", "--test crash_target -- x"]
    wrong_changed = [(before, planned(before), after) for before, after in changed if planned(before) != after]
    wrong_unchanged = [(before, planned(before)) for before in unchanged if planned(before) != before]
    selftest.check("验证包的测试目标不编装置二进制（判法）：-p 只点名 singlefs-verification、点名 --test、没有别的选目标的选项的参数，在第一个 -- 之前补 --no-default-features（带 --release、--features gpu、"
                   "几个 --test、--test=名、没有 -- 的照样补）；--lib、--lib --test、--bin、--bins、--tests、--all-targets、没点名目标、自己已写、别的包或几个包、没点名包的一个字不变",
                   not wrong_changed and not wrong_unchanged, f"该补的没补对 {wrong_changed}；不该动的动了 {wrong_unchanged}")
    rows = ["test-target-one-plus-one\tcrates/tiny/src/lib.rs\t{ 1 + 1 }\t{ 1 + 2 }\t-p singlefs-verification --test sample_target -- released_agrees\treleased_agrees",
            SELFTEST_TABLE_ROWS[0].replace("-p tiny", "-p singlefs-verification")]
    root = os.path.join(work, "default-features-off")
    build_selftest_repository(root, rows, extra_source=SELFTEST_GPU_RELEASED_TARGETS_SOURCE)
    selftest_write(os.path.join(root, "crates", "tiny", "Cargo.toml"), '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(root, "crates", "tiny", "tests", "sample_target.rs"), "mod tests {\n    #[test]\n    fn released_agrees() { assert_eq!(singlefs_verification::shared(), 2); }\n}\n")
    exit_code, stdout, stderr, invocations = selftest.run(root)
    table_rows = read_table(os.path.join(root, TABLE), selftest.base_environment)[0]
    _context, planned_rows, on_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment))
    _context, _rows, off_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment, CRATES_MUTATION_ROWS_BREAK="no-default-features-unadded"))
    target_line, library_line = (row.line_number for row in table_rows)
    planned_arguments = {row.line_number: row.arguments for row in planned_rows}

    def cargo_words(line):
        words = line.split("\t")[0].split()
        return words[:words.index("--")] if "--" in words else words
    calls = [cargo_words(line) for line in invocations]
    target_calls = [words for words in calls if "sample_target" in words]
    library_calls = [words for words in calls if "--lib" in words]
    selftest.check("验证包的测试目标不编装置二进制（一趟真跑）：点名 --test 的一行起的每一次 cargo 都带一个 --no-default-features，点名 --lib 的一行一次都不带；plan_rows 交回的参数同样，"
                   "补了的那一行键与不补的不同、--lib 那一行键不变",
                   exit_code == 0 and len(target_calls) >= 3 and all(words.count("--no-default-features") == 1 for words in target_calls)
                   and library_calls and all("--no-default-features" not in words for words in library_calls)
                   and "--no-default-features" in planned_arguments[target_line].split() and "--no-default-features" not in planned_arguments[library_line].split()
                   and on_components[target_line].row_key and on_components[target_line].row_key != off_components[target_line].row_key
                   and on_components[library_line].row_key == off_components[library_line].row_key,
                   f"退 {exit_code}；点名 --test 的 {target_calls}；点名 --lib 的 {library_calls}；plan_rows 的参数 {planned_arguments}；"
                   f"键 补 {on_components[target_line].row_key} / 不补 {off_components[target_line].row_key}；{stdout[-300:]}{stderr[-500:]}")


def selftest_debuginfo_cells(selftest, work):
    """起 cargo 时关调试信息、release 档开增量（文件头「调试信息与 release 增量」）：一趟跑下来假 cargo 收到的每一次 test（基线、变异、插桩编译与插桩跑）环境里四档的
    CARGO_PROFILE_<档>_DEBUG 都是 false、CARGO_PROFILE_RELEASE_INCREMENTAL 除基线外都是 true，调用方设的同名变量也被顶掉；基线（假 cargo 按 TMPDIR 的名字 baseline-… 认）收到的
    CARGO_PROFILE_RELEASE_INCREMENTAL 是调用方设的值、不是门禁加的 true（文件头「基线连调用方的判定库服务」），另一格直接核 baseline_cargo_environment 的取法。
    编译档与线程数都不进行键（lib_mutation_units.py 的 toolchain_lines）：门禁覆盖的 debug 换成 line-tables-only、调用方另设 CARGO_PROFILE_TEST_DEBUG 的那一趟 4 行都复用；
    门禁覆盖的 release 增量换成 false、调用方另设 CARGO_PROFILE_TEST_INCREMENTAL 的那一趟 4 行都复用；
    调用方设 CARGO_PROFILE_RELEASE_OPT_LEVEL、CARGO_PROFILE_TEST_DEBUG_ASSERTIONS、CARGO_PROFILE_TEST_CODEGEN_UNITS 的三趟各自 4 行都复用；
    调用方设 CARGO_BUILD_JOBS、GATE_MUTATION_CARGO_JOBS、GATE_MUTATION_WORKERS、RUST_TEST_THREADS 的四趟各自 4 行都复用；
    cargo 配置的 [build] jobs 与 [profile.test]、根 Cargo.toml 的 [profile.dev] 改了之后 4 行都复用。
    弄坏开关 debuginfo-kept 下头一格判错，release-incremental-unset 下第二格判错，baseline-release-incremental-kept 下「基线起的 cargo 不带门禁加的 release 增量覆盖」
    「基线环境的取法」两格判错，profile-keyed（lib_mutation_units.py 的：编译档的原文行回到行键里）下「改编译档的 debug 之后行键不变」「改编译档的 incremental 之后行键不变」
    「改编译档的 opt-level、debug-assertions、codegen-units 之后行键不变」「改配置文件与清单里的编译档、线程数之后行键不变」判错，
    build-jobs-keyed（lib_mutation_units.py 的：CARGO_BUILD_JOBS 与 [build] jobs 回到行键里）下「改线程数之后行键不变」「改配置文件与清单里的编译档、线程数之后行键不变」判错。"""
    repository = os.path.join(work, "debuginfo")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "debuginfo-environment.jsonl")
    exit_code, _stdout, stderr, _ = selftest.run(repository, FAKE_CARGO_ENVIRONMENT_LOG=environment_log, CARGO_PROFILE_RELEASE_DEBUG="true",
                                                 CARGO_PROFILE_RELEASE_INCREMENTAL="false")
    records = environment_records(environment_log)
    wanted = {f"CARGO_PROFILE_{profile}_DEBUG": "false" for profile in CARGO_DEBUG_PROFILES}
    off = [record for record in records if all(record.get(name) == value for name, value in wanted.items())]
    forms = {("--no-run" in record["ARGUMENTS"].split(), any(word.startswith("--message-format") for word in record["ARGUMENTS"].split())) for record in records}
    selftest.check(f"门禁起的每条 cargo 都关了调试信息：{len(records)} 次（基线、变异、插桩编译与插桩跑）每一次四档的 CARGO_PROFILE_<档>_DEBUG 都是 false，调用方设的 true 也顶掉",
                   exit_code == 0 and records and len(off) == len(records) and {(False, False), (True, True)} <= forms,
                   f"退 {exit_code}；{len(off)}/{len(records)} 次关了；收到的 {[{name: record.get(name) for name in wanted} for record in records[:3]]}；{stderr[-300:]}")
    baseline_records = [record for record in records if record.get("RUN_NAME", "").startswith("baseline-")]
    other_records = [record for record in records if not record.get("RUN_NAME", "").startswith("baseline-")]
    incremental = [record for record in other_records if record.get("CARGO_PROFILE_RELEASE_INCREMENTAL") == "true"]
    selftest.check(f"门禁起的每条 cargo 都开了 release 档的增量编译：{len(other_records)} 次（基线不算，基线那一格另判）每一次 CARGO_PROFILE_RELEASE_INCREMENTAL 都是 true，调用方设的 false 也顶掉",
                   exit_code == 0 and other_records and len(incremental) == len(other_records),
                   f"退 {exit_code}；{len(incremental)}/{len(other_records)} 次开了；收到的 {[record.get('CARGO_PROFILE_RELEASE_INCREMENTAL') for record in other_records[:3]]}")
    baseline_overridden = [record for record in baseline_records if record.get("CARGO_PROFILE_RELEASE_INCREMENTAL") != "false"]
    baseline_debug_off = [record for record in baseline_records if all(record.get(name) == value for name, value in wanted.items())]
    selftest.check(f"基线起的 cargo 不带门禁加的 release 增量覆盖：{len(baseline_records)} 次基线每一次 CARGO_PROFILE_RELEASE_INCREMENTAL 都是调用方设的 false（不是门禁加的 true），调试信息那四项照关",
                   exit_code == 0 and baseline_records and not baseline_overridden and len(baseline_debug_off) == len(baseline_records),
                   f"退 {exit_code}；基线 {len(baseline_records)} 次、被覆盖的 {len(baseline_overridden)} 次、调试信息关着的 {len(baseline_debug_off)} 次；"
                   f"基线收到的 {[record.get('CARGO_PROFILE_RELEASE_INCREMENTAL') for record in baseline_records[:3]]}；{stderr[-300:]}")
    carried = dict(selftest.base_environment, CARGO_PROFILE_RELEASE_INCREMENTAL="true", CARGO_PROFILE_TEST_DEBUG="false")
    carried_before = dict(carried)
    without_caller_value = baseline_cargo_environment(carried, None)
    with_caller_value = baseline_cargo_environment(carried, "false")
    selftest.check("基线环境的取法：调用方没设 release 增量就不放这一项、设了照放回它的值，别的变量原样留着，传进去的那一份不改",
                   "CARGO_PROFILE_RELEASE_INCREMENTAL" not in without_caller_value and with_caller_value.get("CARGO_PROFILE_RELEASE_INCREMENTAL") == "false"
                   and without_caller_value.get("CARGO_PROFILE_TEST_DEBUG") == "false" and with_caller_value.get("CARGO_PROFILE_TEST_DEBUG") == "false"
                   and carried == carried_before,
                   f"没设时 {without_caller_value.get('CARGO_PROFILE_RELEASE_INCREMENTAL')!r}、设了 false 时 {with_caller_value.get('CARGO_PROFILE_RELEASE_INCREMENTAL')!r}；"
                   f"传进去的那一份改了 {carried != carried_before}")
    reused = selftest.run(repository)
    saved = CARGO_DEBUG_SETTING[0], CARGO_RELEASE_INCREMENTAL_SETTING[0]
    try:
        CARGO_DEBUG_SETTING[0] = "line-tables-only"
        unkeyed = selftest.run(repository, CARGO_PROFILE_TEST_DEBUG="2")
        CARGO_DEBUG_SETTING[0], CARGO_RELEASE_INCREMENTAL_SETTING[0] = saved[0], "false"
        incremental_changed = selftest.run(repository, CARGO_PROFILE_TEST_INCREMENTAL="false")
    finally:
        CARGO_DEBUG_SETTING[0], CARGO_RELEASE_INCREMENTAL_SETTING[0] = saved
    selftest.check("改编译档的 debug 之后行键不变：头一趟之后同一取值 4 行都复用；门禁覆盖的 debug 换成 line-tables-only、"
                   "调用方另设 CARGO_PROFILE_TEST_DEBUG=2 的那一趟 4 行照样都复用",
                   reused[0] == 0 and "现判 0 条、复用 4 条" in reused[1] and unkeyed[0] == 0 and "现判 0 条、复用 4 条" in unkeyed[1],
                   f"同一取值 {per_row_how(reused[1])}；换了 debug {per_row_how(unkeyed[1])}")
    selftest.check("改编译档的 incremental 之后行键不变：门禁覆盖的 release 增量换成 false、调用方另设 CARGO_PROFILE_TEST_INCREMENTAL=false 的那一趟 4 行都复用",
                   incremental_changed[0] == 0 and "现判 0 条、复用 4 条" in incremental_changed[1], f"换了 incremental {per_row_how(incremental_changed[1])}")
    profile_runs = {name: selftest.run(repository, **{name: value}) for name, value in (("CARGO_PROFILE_RELEASE_OPT_LEVEL", "2"), ("CARGO_PROFILE_TEST_DEBUG_ASSERTIONS", "false"),
                                                                                         ("CARGO_PROFILE_TEST_CODEGEN_UNITS", "1"))}
    selftest.check("改编译档的 opt-level、debug-assertions、codegen-units 之后行键不变：调用方各设一项的三趟各自 4 行都复用",
                   all(run[0] == 0 and "现判 0 条、复用 4 条" in run[1] for run in profile_runs.values()),
                   "；".join(f"设 {name} {per_row_how(run[1])}" for name, run in profile_runs.items()))
    thread_settings = (dict(CARGO_BUILD_JOBS="3"), dict(GATE_MUTATION_CARGO_JOBS="5"), dict(GATE_MUTATION_WORKERS="2"), dict(RUST_TEST_THREADS="1"))
    thread_runs = {next(iter(setting)): selftest.run(repository, **setting) for setting in thread_settings}
    selftest.check("改线程数之后行键不变：调用方各设 CARGO_BUILD_JOBS=3、GATE_MUTATION_CARGO_JOBS=5、GATE_MUTATION_WORKERS=2、RUST_TEST_THREADS=1 的四趟各自 4 行都复用",
                   all(run[0] == 0 and "现判 0 条、复用 4 条" in run[1] for run in thread_runs.values()),
                   "；".join(f"设 {name} {per_row_how(run[1])}" for name, run in thread_runs.items()))
    files = os.path.join(work, "debuginfo-files")
    build_selftest_repository(files, SELFTEST_TABLE_ROWS)
    recorded = selftest.run(files)
    selftest_write(os.path.join(files, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), "[build]\njobs = 3\n\n[profile.test]\nopt-level = 1\nincremental = false\n")
    with open(os.path.join(files, "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write("\n[profile.dev]\ndebug-assertions = false\ncodegen-units = 7\n")
    edited_files = selftest.run(files)
    selftest.check("改配置文件与清单里的编译档、线程数之后行键不变：cargo 配置加 [build] jobs 与 [profile.test]（opt-level、incremental）、根 Cargo.toml 加 [profile.dev]（debug-assertions、codegen-units）之后，"
                   "4 行都复用",
                   recorded[0] == 0 and edited_files[0] == 0 and "现判 0 条、复用 4 条" in edited_files[1], f"头一趟 {recorded[1][-200:]}；改了之后 {per_row_how(edited_files[1])}")


def selftest_key_release_cells(selftest, work):
    """真跑的小仓（假 cargo）：放之前与放之后 current_fingerprints 的回答相同；run_stage 派工作进程之前放掉工作区，判定、记覆盖与不放时逐字相同，覆盖此刻的那一份不重记。"""
    import gc
    repository = os.path.join(work, "key-release")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment = dict(selftest.base_environment)
    table_rows, _malformed = read_table(os.path.join(repository, TABLE), environment)
    key_context, rows, components, _library = plan_rows(repository, table_rows, environment)
    sample_row = rows[0]
    unit_reference = mutation_units.UnitReference
    references = (list(key_context.closures[components[sample_row.line_number].test_closure].members) + list(components[sample_row.line_number].mutated)
                  + [unit_reference("data-file", "fixture-data/answer.txt", "fixture-data/answer.txt", "1-1", "0" * 64),
                     unit_reference("fn", "crates/tiny/src/lib.rs", "no_such_function", "1-1", "0" * 64),
                     unit_reference("fn", "crates/tiny/src/absent.rs", "gone", "1-1", "0" * 64)])
    before = key_context.keys_for(sample_row)[0].current_fingerprints(references)
    line = release_key_context(key_context, components, environment)
    gc.unfreeze()
    after = key_context.keys_for(sample_row)[0].current_fingerprints(references)
    selftest.check("派工作进程之前：放掉工作区之后 current_fingerprints 的回答与放之前逐字相同（闭包成员、被改坏的单元、数据文件、没有的函数、没有的文件）",
                   before == after and any(before.values()), f"{line}；放之前 {before}；放之后 {after}")
    released_run = selftest.run(repository, GATE_MUTATION_WORKERS="2")
    reading = re.search(r"^MUTATION_KEY_CONTEXT_RELEASE .*$", released_run[2], re.M)
    selftest.check("派工作进程之前：run_stage 放掉了工作区（一个都没剩）、冻了对象，打一行 MUTATION_KEY_CONTEXT_RELEASE，收完永久代是空的",
                   released_run[0] == 0 and reading is not None and "released=yes" in reading.group(0) and "workspaces_released=1 workspaces_left=0" in reading.group(0)
                   and re.search(r"frozen_objects=[1-9]", reading.group(0)) is not None and gc.get_freeze_count() == 0, f"退 {released_run[0]}；{reading and reading.group(0)}；永久代 {gc.get_freeze_count()}")
    kept_run = selftest.run(selftest_new_library_copy(work, repository, "key-release-kept"), GATE_MUTATION_WORKERS="2",
                            GATE_MUTATION_BREAK=with_break(selftest, "key-context-kept-for-workers"))
    selftest.check("派工作进程之前：放与不放，判定逐字相同，起 cargo 与记覆盖的次数相同",
                   kept_run[0] == released_run[0] and selftest_verdict_view(kept_run[1]) == selftest_verdict_view(released_run[1])
                   and len(plain_invocations(kept_run[3])) == len(plain_invocations(released_run[3])) and len(coverage_invocations(kept_run[3])) == len(coverage_invocations(released_run[3])),
                   f"放 {released_run[1]}；不放 {kept_run[1]}")
    replace_in_file(os.path.join(repository, TABLE), "{ value * 3 }", "{ value * 4 }")
    changed_row = selftest.run(repository, GATE_MUTATION_WORKERS="2")
    selftest.check("派工作进程之前：放掉工作区之后，覆盖记录还是此刻的那一行照指纹表认：只改一行的替换文，现判那一条，一份覆盖都不重记",
                   changed_row[0] == 0 and len(plain_invocations(changed_row[3])) == 1 and coverage_invocations(changed_row[3]) == [], f"退 {changed_row[0]}；" + "\n".join(changed_row[3]))


def selftest_row_key_collision_cell(selftest, work, environment):
    """撞键拒算（refuse_colliding_row_keys，文件头「行键」）：left.rs 与 right.rs 里内容相同的 value 各改一处、点名同一条测试的两行算出同一把行键，
    头一趟两行都拒算、照跑（各起一次变异）、成功句与 stderr 逐行报「拒算（行键撞键：与第 N 行同键）」，不撞键的 three-becomes-four 现判；
    第二趟两行照旧拒算、照跑，three-becomes-four 复用；只选 left 那一行（选行，别的行不算键）时照整张表判、仍拒算。
    弄坏开关 row-key-collision-unrefused（不判撞键）下这一格判错。"""
    twin = os.path.join(work, "reuse-fix-twin-units")
    build_selftest_repository(twin, SELFTEST_TWIN_UNIT_ROWS, source=SELFTEST_TWIN_UNIT_SOURCE)
    for name in ("left", "right"):
        selftest_write(os.path.join(twin, "crates", "tiny", "src", f"{name}.rs"), "pub fn value() -> u32 { 1 }\n")
    twin_rows = read_table(os.path.join(twin, TABLE), environment)[0]
    left, right = (row.line_number for row in twin_rows[:2])
    _context, _rows, alone, _library = plan_rows(twin, twin_rows[:1], environment, table_rows=twin_rows)
    first = selftest.run(twin)
    second = selftest.run(twin)

    def mutation_runs(invocations):
        """这一趟起的变异那几次（带 --show-output 的只有变异那一趟），按点名的测试数。"""
        return collections.Counter(word for line in invocations if "--show-output" in line.split("\t")[0].split()
                                   for word in ("values_are_one", "three_is_three") if word in line)
    wanted_reasons = [f"拒算（行键撞键：与第 {right} 行同键）：{TABLE}:{left} left-one-to-two", f"拒算（行键撞键：与第 {left} 行同键）：{TABLE}:{right} right-one-to-two"]
    first_how, second_how = per_row_how(first[1]), per_row_how(second[1])
    selftest.check("撞键拒算：left.rs 与 right.rs 里内容相同的函数各改一处、点名同一条测试的两行同键，两趟都拒算、照跑，成功句与 stderr 逐行报"
                   "「拒算（行键撞键：与第 N 行同键）」；不撞键的一行头一趟现判、第二趟复用；只选 left 那一行也照整张表判撞键",
                   first[0] == 0 and second[0] == 0
                   and first_how.get("left-one-to-two") == "拒算" and first_how.get("right-one-to-two") == "拒算" and first_how.get("three-becomes-four", "").startswith("现判")
                   and second_how.get("left-one-to-two") == "拒算" and second_how.get("right-one-to-two") == "拒算" and second_how.get("three-becomes-four") == "复用"
                   and mutation_runs(first[3]) == {"values_are_one": 2, "three_is_three": 1} and mutation_runs(second[3]) == {"values_are_one": 2}
                   and all(reason in first[2] and reason in second[2] for reason in wanted_reasons)
                   and f"其中撞键拒算 2 条：第 {left} 行拒算（行键撞键：与第 {right} 行同键）" in second[1] and "现判 0 条、复用 1 条、拒算 2 条" in second[1]
                   and alone[left].row_key is None and alone[left].refusal == f"{COLLISION_REFUSAL_PREFIX}与第 {right} 行同键",
                   f"头一趟退 {first[0]}、{first_how}、变异 {dict(mutation_runs(first[3]))}；第二趟退 {second[0]}、{second_how}、变异 {dict(mutation_runs(second[3]))}；"
                   f"只选 left 那一行：键 {alone[left].row_key}、{alone[left].refusal}；第二趟 {second[1][-400:]}{second[2][-600:]}")


def selftest_reuse_fix_cells(selftest, work):
    """复用的几处修补（文件头「行键」「判定库」「基线」「复用判据」）：
    撞键拒算（selftest_row_key_collision_cell；弄坏开关 row-key-collision-unrefused 下判错）；
    基线键与覆盖号带包与测试目标（两个目标里字面相同的测试，基线键与覆盖号不同；baseline-key-without-target 下相同）；
    一行的基线结局不随这一批拼进了哪几行而变（核查报告那个复现：只选一行、两行一批，同一行同一个结论；baseline-batch-composition 下不同）；
    记红撤绿（先抓到入库，同一把键改判没红、基线不绿各一趟，下一趟现判；red-not-recorded、baseline-red-keeps-caught 下照复用）；
    不算数的结局不盖算数的（selftest_uncounted_keeps_counted_cells；uncounted-overwrites-counted 下判错）；
    库根（被判的树不是 git 工作树：退 1、点名它、一条都不跑；library-outside-git-tolerated 下照跑）。"""
    environment = dict(selftest.base_environment)
    selftest_row_key_collision_cell(selftest, work, environment)
    # 基线键与覆盖号带包与测试目标
    targets = os.path.join(work, "reuse-fix-test-targets")
    build_selftest_repository(targets, SELFTEST_TARGET_ROWS, source="pub fn shared() -> u32 { 1 }\n")
    for name, value in (("alpha", 1), ("beta", 2)):
        selftest_write(os.path.join(targets, "crates", "tiny", "tests", f"{name}.rs"), SELFTEST_TARGET_TEST.format(value))
    target_rows = read_table(os.path.join(targets, TABLE), environment)[0]
    _context, _rows, target_components, _library = plan_rows(targets, target_rows, environment)
    pairs = [(target_components[row.line_number].baseline_key, target_components[row.line_number].coverage_id) for row in target_rows]
    selftest.check("基线键与覆盖号带包与测试目标：tests/alpha.rs 与 tests/beta.rs 里字面相同的 smoke，两行的基线键、覆盖号都不同",
                   len(pairs) == 2 and all(all(pair) for pair in pairs) and pairs[0][0] != pairs[1][0] and pairs[0][1] != pairs[1][1], f"（基线键, 覆盖号）{pairs}")
    # 一行的基线结局不随这一批拼进了哪几行而变（核查报告的复现：两个模块各有一条 foo，一行点 a::foo、短名 foo，一行点 b::foo）
    one_to_three = Row(1, "one_to_three", "crates/demo/src/lib.rs", "1", "3", "-p demo --lib -- --exact a::foo", "foo", ())
    two_to_three = Row(2, "two_to_three", "crates/demo/src/lib.rs", "2", "3", "-p demo --lib -- --exact b::foo", "b::foo", ())
    single_row_output = "running 1 test\ntest a::foo ... ok\n\ntest result: ok. 1 passed; 0 failed\n"
    two_row_output = "running 2 tests\ntest a::foo ... ok\ntest b::foo ... ok\n\ntest result: ok. 2 passed; 0 failed\n"
    single_row = baseline_problem(one_to_three, 0, row_baseline_output(one_to_three, single_row_output, environment))
    two_rows = baseline_problem(one_to_three, 0, row_baseline_output(one_to_three, two_row_output, environment))
    other = baseline_problem(two_to_three, 0, row_baseline_output(two_to_three, two_row_output, environment))
    selftest.check("一行的基线结局不随这一批拼进了哪几行而变：one_to_three 一行一批与两行一批都判基线 ok，two_to_three 也 ok",
                   single_row is None and two_rows is None and other is None, f"一行一批 {single_row!r}、两行一批 {two_rows!r}、two_to_three {other!r}")
    # 记红撤绿：先抓到入库；同一把键（不改源码、不改表）经复用哨兵强制现判一行（哨兵取行数少一条，剩下那一条强制现判，其余照复用），判一趟没红，下一趟那一行现判；
    #   另一棵树先抓到入库（不跑基线，库里没有基线记录），同一把键强制现判一行、判一趟基线不绿，下一趟那一行现判（先跑基线）
    row_count = len(SELFTEST_TABLE_ROWS)
    one_forced = selftest_one_forced(row_count)
    withdrawn = os.path.join(work, "reuse-fix-red-withdraws")
    build_selftest_repository(withdrawn, SELFTEST_TABLE_ROWS)
    first = selftest.run(withdrawn)
    survived = selftest.run(withdrawn, FAKE_CARGO_FORCED_OUTCOME="ok", **one_forced)
    survived_name = selftest_forced_row_name(survived)
    after_survived = selftest.run(withdrawn)
    selftest.check("记红撤绿：先抓到入库，同一把键经哨兵强制现判一行、改判没红之后，下一趟那一行现判（栏是上次没判成抓到，不复用上次的抓到），别的行照复用",
                   first[0] == 0 and survived[0] == 1 and survived_name is not None and after_survived[0] == 0
                   and per_row_how(after_survived[1]).get(survived_name) == "现判：上次没判成抓到"
                   and f"现判 1 条、复用 {row_count - 1} 条" in after_survived[1], survived[2][-400:] + after_survived[1] + after_survived[2])
    baseline_withdrawn = os.path.join(work, "reuse-fix-baseline-red-withdraws")
    build_selftest_repository(baseline_withdrawn, SELFTEST_TABLE_ROWS)
    without_baseline = selftest.run(baseline_withdrawn, GATE_MUTATION_BREAK=with_break(selftest, "nobaseline"))
    baseline_red = selftest.run(baseline_withdrawn, FAKE_CARGO_FORCED_OUTCOME="FAILED", **one_forced)
    baseline_red_name = selftest_forced_row_name(baseline_red)
    after_baseline_red = selftest.run(baseline_withdrawn)
    selftest.check("记红撤绿：先抓到入库，同一把键经哨兵强制现判一行、判一趟基线不绿之后，下一趟那一行现判（先跑基线，不复用上次的抓到），别的行照复用",
                   without_baseline[0] == 0 and baseline_red[0] == 1 and "基线不绿 1 行" in baseline_red[1] and baseline_red_name is not None
                   and after_baseline_red[0] == 0 and per_row_how(after_baseline_red[1]).get(baseline_red_name) == "现判：上次没判成抓到"
                   and f"现判 1 条、复用 {row_count - 1} 条" in after_baseline_red[1] and len(plain_invocations(after_baseline_red[3])) == 2,
                   baseline_red[1] + after_baseline_red[1] + "\n".join(after_baseline_red[3]))
    selftest_uncounted_keeps_counted_cells(selftest, work)
    # 库根：被判的树没有 .git（rsync 拷的副本那样）：一条都不跑、点名它，不退回别处
    copy = os.path.join(work, "reuse-fix-library-copy")
    build_selftest_repository(copy, SELFTEST_TABLE_ROWS)
    shutil.rmtree(os.path.join(copy, ".git"))
    rooted = selftest.run(copy)
    selftest.check("库根：被判的树不是 git 工作树：退 1、点名它、一条都不跑",
                   rooted[0] == 1 and "不是 git 工作树" in rooted[1] + rooted[2] and copy in rooted[1] + rooted[2] and not plain_invocations(rooted[3]),
                   f"退 {rooted[0]}，假 cargo 真跑 {len(plain_invocations(rooted[3]))} 次：{rooted[1][-300:]}{rooted[2][-400:]}")




def selftest_driver_and_library_not_in_key_cell(selftest, work):
    """驱动与算键库的代码不进键（文件头「行键」）：样本仓根下放驱动（scripts/mutation/crates_mutation_rows_judging.py、lib_mutation_units.py）与算键库
    （scripts/gate/lib_function_units_rust_file.py）三份替身，改其中每一份的一个函数体，样本每一行的行键、基线键、覆盖号都不变；
    对照：改样本 crate 里被改坏的函数 check_root 的别一行，原文落在它里面的行的行键变（键不是常量）。
    弄坏开关 driver-and-library-keyed（KeyContext.components：仓根下驱动与算键库那几份文件的 sha256 并进三把键）下这一格判错。"""
    repository = os.path.join(work, "keys-without-driver")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    environment = dict(selftest.base_environment)
    table_rows = read_table(os.path.join(repository, TABLE), environment)[0]

    def keys_now():
        _context, _rows, found, _library = plan_rows(repository, table_rows, environment)
        return {row.name: (found[row.line_number].row_key, found[row.line_number].baseline_key, found[row.line_number].coverage_id) for row in table_rows}

    stand_ins = {"scripts/mutation/crates_mutation_rows_judging.py": ("def verdict_of_run(output):\n    return output\n", "def verdict_of_run(output):\n    return (output)\n"),
                 "scripts/mutation/lib_mutation_units.py": ("def key_of_lines(groups):\n    return sorted(groups)\n", "def key_of_lines(groups):\n    return list(sorted(groups))\n"),
                 "scripts/gate/lib_function_units_rust_file.py": ("class RustFile:\n    def row(self, line):\n        return line\n",
                                                                 "class RustFile:\n    def row(self, line):\n        return line + 1\n")}
    for relative, (before_text, _after_text) in stand_ins.items():
        selftest_write(os.path.join(repository, relative), before_text)
    with_stand_ins = keys_now()
    for relative, (_before_text, after_text) in stand_ins.items():
        selftest_write(os.path.join(repository, relative), after_text)
    edited = keys_now()
    selftest.check("改驱动与算键库的函数体：样本仓根下三份替身（驱动两份、算键库一份）各改一个函数体，每一行的行键、基线键、覆盖号都不变",
                   len(edited) == len(table_rows) and all(all(keys) for keys in edited.values()) and edited == with_stand_ins,
                   f"改前 {[(name, keys[0][:12]) for name, keys in with_stand_ins.items()]}；改后 {[(name, keys[0][:12]) for name, keys in edited.items()]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "base.max(OFFSET)", "base.max(OFFSET + 0)")
    changed = keys_now()
    moved = sorted(name for name in changed if changed[name][0] != edited[name][0])
    selftest.check("对照：改被改坏的函数 check_root 的别一行，原文落在它里面的 root-457、root-double 行键变，别的行不变",
                   moved == ["root-457", "root-double"], f"行键变了的行 {moved}")


def selftest_test_service_code_key_cell(selftest, work):
    """测试服务代码不进键（lib_mutation_units.py 文件头「测试服务代码不进键」）：样本仓里 crates/tiny/src/lib.rs 临时登成测试服务代码（lib_function_units.py 的登记表读进来的那一份里并进一行，
    跑完去掉），样本里的函数与点名测试都在这一份里；每一格拿样本每一行的（行键、基线键、覆盖号）对上一格的：
    ① 改被改坏、点名测试够不着的函数 helper：每一行的三把键都不变（行键里 mutated 那一组不放测试服务代码里的单元指纹）；
    ② 改点名测试 sums 调到的函数 pad：三把键都不变（闭包里测试服务代码的单元不出 unit 行，closure_key 排）；
    ③ 改点名测试 sums 自己（入口单元的指纹照进键）：只有点名它的 sum-sub 一行的行键、基线键、覆盖号变，别的行不变。
    弄坏开关 mutated-test-service-code-keyed（KeyContext.components：被改坏的单元落在测试服务代码里也放进 mutated 那一组）下 ① 判错。
    ② 没有自己的开关（排闭包里测试服务代码的单元是 lib_function_units.py 的 closure_key 做的）：环境里设 FUNCTION_UNITS_BREAK=test-service-code-keyed（closure_key 照收这些单元）下 ② 判错。"""
    repository = os.path.join(work, "keys-test-service-code")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    source_path = "crates/tiny/src/lib.rs"
    environment = dict(selftest.base_environment)
    table_rows = read_table(os.path.join(repository, TABLE), environment)[0]

    def keys_now():
        _context, _rows, found, _library = plan_rows(repository, table_rows, environment)
        return {row.name: (found[row.line_number].row_key, found[row.line_number].baseline_key, found[row.line_number].coverage_id) for row in table_rows}

    registered = mutation_units.function_units().test_service_code_rows()
    registered[source_path] = "自证：只在 selftest_test_service_code_key_cell 跑的那一会儿登记"
    try:
        before = keys_now()
        replace_in_file(os.path.join(repository, source_path), "pub fn helper(value: u32) -> u32 { value + 7 }", "pub fn helper(value: u32) -> u32 { let _seven = 7; { value + 7 } }")
        helped = keys_now()
        replace_in_file(os.path.join(repository, source_path), "pub fn pad(value: u32) -> u32 { value + 0 }", "pub fn pad(value: u32) -> u32 { value + 0 + 0 }")
        padded = keys_now()
        replace_in_file(os.path.join(repository, source_path), "assert_eq!(super::sum(super::pad(2), 3), 5);", "assert_eq!(super::sum(super::pad(2), 3), 5 + 0);")
        tested = keys_now()
    finally:
        registered.pop(source_path, None)
    names = [row.name for row in table_rows]
    selftest.check("测试服务代码：被改坏的函数在测试服务代码里，改它之后一行都不现判（样本每一行的行键、基线键、覆盖号都不变）",
                   len(before) == len(table_rows) and all(all(keys) for keys in before.values()) and helped == before,
                   f"改 helper 之后键变了的行 {sorted(name for name in names if helped.get(name) != before.get(name))}（要没有）")
    selftest.check("测试服务代码：点名测试调到的函数在测试服务代码里，改它之后一行都不现判（闭包里测试服务代码的单元不出 unit 行）", padded == helped,
                   f"改 pad 之后键变了的行 {sorted(name for name in names if padded.get(name) != helped.get(name))}（要没有）")
    moved = sorted(name for name in names if tested[name] != padded[name])
    selftest.check("测试服务代码：改点名测试自己（入口单元的指纹照进键），只有点名它的那一行的三把键变",
                   all(all(keys) for keys in tested.values()) and moved == ["sum-sub"] and all(tested["sum-sub"][index] != padded["sum-sub"][index] for index in range(3)),
                   f"键变了的行 {moved}（要只有 sum-sub）")


def selftest_build_environment_and_version_cells(selftest, work):
    """构建环境、rustc 与 cargo 的版本、Cargo.lock 里的依赖版本都不进键（文件头「行键」）：样本 crate 的 pad 点名外部包 fakedep（清单与 Cargo.lock 里都有它），
    点名 pad 的测试 sums 的闭包里就有这一处外部引用；每一格拿样本每一行的（行键、基线键、覆盖号）对上一格的：
    ① 假 cargo 的 -V 换一行；
    ② 编译器与链接器相关的环境变量（RUSTFLAGS、RUSTDOCFLAGS、CARGO_BUILD_TARGET、RUSTC_WRAPPER、CARGO_TARGET_<三元组>_LINKER）换了；
    ③ cargo 配置加 [build] 的 rustflags、[target.<三元组>] 的 linker 与 rustflags、[env]；
    ④ 根 Cargo.toml 加 [workspace.lints]；
    ⑤ Cargo.lock 里 fakedep 的版本换一个；
    ⑥ PATH 指向空目录（没有 rustc 与 cargo），照样算得出行键、与有它们时逐把相同；
    ⑦ 对照：改 pad 的函数体（在 sums 的闭包里），sum-sub 的行键变、别的行不变（键不是常量）。
    ①到⑥都是「三把键都不变」。弄坏开关 build-environment-keyed（KeyContext 另收 toolchain_lines 交的行）下 ②③④ 判错，
    toolchain-version-keyed（toolchain_lines 另收 rustc -vV、cargo -V，KeyContext 收 toolchain_lines 交的全部行）下 ①②③④ 判错、⑥ 判错（rustc 起不来），
    dependency-versions-keyed（lib_mutation_units.py 的 UnitKeys：依赖版本行并进闭包键）下 ⑤ 判错。"""
    repository = os.path.join(work, "keys-without-environment")
    source = SELFTEST_UNIT_SOURCE.replace("pub fn pad(value: u32) -> u32 { value + 0 }", "pub fn pad(value: u32) -> u32 { value + 0 + fakedep::zero() }")
    assert source != SELFTEST_UNIT_SOURCE
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=source)
    selftest_write(os.path.join(repository, "crates", "tiny", "Cargo.toml"), '[package]\nname = "tiny"\nversion = "0.1.0"\n\n[dependencies]\nfakedep = "1.2.3"\n')
    lock_path = os.path.join(repository, "Cargo.lock")
    selftest_write(lock_path, SELFTEST_EXTERNAL_LOCK)
    environment = dict(selftest.base_environment)
    table_rows = read_table(os.path.join(repository, TABLE), environment)[0]

    def keys_now(changes=None):
        _context, _rows, found, _library = plan_rows(repository, table_rows, dict(environment, **(changes or {})))
        return {row.name: (found[row.line_number].row_key, found[row.line_number].baseline_key, found[row.line_number].coverage_id) for row in table_rows}

    def shown(keys):
        return [(name, triple[0][:12] if triple[0] else None) for name, triple in keys.items()]

    before = keys_now()
    cargo_changed = keys_now({"FAKE_CARGO_VERSION": "cargo 9.9.9 (crates-mutation-rows selftest：换了 cargo 的版本)"})
    selftest.check("假 cargo 的 -V 换一行之后样本每一行的行键、基线键、覆盖号都不变",
                   len(before) == len(table_rows) and all(all(keys) for keys in before.values()) and cargo_changed == before,
                   f"改前 {shown(before)}；改后 {shown(cargo_changed)}")
    compiler_environment = {"RUSTFLAGS": "--cfg selftest_flag", "RUSTDOCFLAGS": "--cfg selftest_doc_flag", "CARGO_BUILD_TARGET": "x86_64-unknown-linux-gnu",
                            "RUSTC_WRAPPER": "selftest-wrapper", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER": "selftest-linker"}
    environment_changed = keys_now(compiler_environment)
    selftest.check("改编译器与链接器相关的环境变量之后样本每一行的行键、基线键、覆盖号都不变：RUSTFLAGS、RUSTDOCFLAGS、CARGO_BUILD_TARGET、RUSTC_WRAPPER、CARGO_TARGET_<三元组>_LINKER 换了",
                   environment_changed == before, f"改前 {shown(before)}；改后 {shown(environment_changed)}")
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"),
                   '[build]\nrustflags = ["--cfg", "selftest_flag"]\n\n[target.x86_64-unknown-linux-gnu]\nlinker = "selftest-linker"\nrustflags = ["-C", "link-arg=-s"]\n\n'
                   '[env]\nSELFTEST_VALUE = "2"\n')
    configuration_changed = keys_now()
    selftest.check("改 cargo 配置的 build、target、env 之后样本每一行的行键、基线键、覆盖号都不变：cargo 配置加 [build] rustflags、[target.<三元组>] linker 与 rustflags、[env]",
                   configuration_changed == before, f"改前 {shown(before)}；改后 {shown(configuration_changed)}")
    with open(os.path.join(repository, "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write('\n[workspace.lints.rust]\nunused_variables = "warn"\n')
    lints_changed = keys_now()
    selftest.check("改根清单的 lints 之后样本每一行的行键、基线键、覆盖号都不变：根 Cargo.toml 加 [workspace.lints.rust]",
                   lints_changed == configuration_changed, f"改前 {shown(configuration_changed)}；改后 {shown(lints_changed)}")
    replace_in_file(lock_path, 'name = "fakedep"\nversion = "1.2.3"', 'name = "fakedep"\nversion = "1.2.4"')
    lock_changed = keys_now()
    selftest.check("改 Cargo.lock 里的依赖版本之后行键不变：点名外部包 fakedep 的闭包（sums 经 pad）在 Cargo.lock 里的版本换了，样本每一行的行键、基线键、覆盖号都不变",
                   lock_changed == lints_changed, f"改前 {shown(lints_changed)}；改后 {shown(lock_changed)}")
    empty_path = os.path.join(work, "keys-without-environment-empty-path")
    os.makedirs(empty_path, exist_ok=True)
    try:
        without_compilers, problem = keys_now({"PATH": empty_path}), None
    except mutation_units.MutationUnitsError as error:
        without_compilers, problem = None, str(error)
    selftest.check("没有 rustc 与 cargo 也算得出行键：PATH 指向空目录，样本每一行的行键、基线键、覆盖号与有 rustc 与 cargo 时逐把相同",
                   without_compilers is not None and without_compilers == lock_changed, f"算不出：{problem}；有编译器时 {shown(lock_changed)}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "value + 0 + fakedep::zero()", "value + 1 + fakedep::zero()")
    changed = keys_now()
    moved = sorted(name for name in changed if changed[name][0] != lock_changed[name][0])
    selftest.check("对照：改点名测试 sums 的闭包里的函数 pad 的函数体，sum-sub 的行键变，别的行不变", moved == ["sum-sub"], f"行键变了的行 {moved}")
