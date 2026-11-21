"""crates-mutation-rows.py 拆出的一份。
几个子进程按内存配速同时跑的个数、判定库与运行记录目录、行键（KeyContext）、复用判据（baseline_reuse_problem 在 judging），连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import contextlib
import fnmatch
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from crates_mutation_rows_base import (BASELINE_KEY_FORM, COLUMN_TITLES, COVERAGE_KEY_FORM, Components, Decision,
    IGNORED_WHEN_COPYING_INTO_EACH_SHARD, LIBRARY_DIRECTORY_NAME, MODULE_PATH, RECORD_HEADER, REUSABLE_VERDICT, ROW_KEY_FORM, SCRIPT_KEY,
    STAGE_REPOSITORY, TABLE, admission_module, break_is, build_environment_lines_for_break, default_heavy_threads, merge_step_close, merge_step_open,
    merge_step_row_done, mutation_units, note, process_usage_bytes, read_table, reuse_checks, under_temporary_prune_server, write_atomically)
from crates_mutation_rows_support import (build_selftest_repository, per_row_how, replace_in_file, selftest_one_forced, selftest_write)
from crates_mutation_rows_static import (manifest_directory_of, read_bytes, unit_keys_for_directory, unit_keys_under_manifest)


def memory_bounded_jobs(threads, cap_bytes, base_bytes, extra_bytes):
    """几个子进程同时跑：threads 与内存上限容得下的个数两者的小者，至少 1。容得下的个数 = (上限 − base_bytes) ÷ extra_bytes，向下取整（extra_bytes 不到 1 按 1 除）；
    cap_bytes 是 None（没有内存上限这一项）不收。自证的格函数（selftest_cell_jobs_for）交推的账（selftest_cell_limits）。"""
    jobs = threads
    if cap_bytes is not None:
        jobs = min(jobs, (cap_bytes - base_bytes) // max(1, extra_bytes))
    return max(1, jobs)


def row_content_hash(row):
    """六段原样的 sha256：只给用时表与索引当名字，不进任何键。"""
    return hashlib.sha256("\t".join(row.fields).encode("utf-8", "surrogateescape")).hexdigest()


def driver_and_library_digests(root):
    """弄坏开关 driver-and-library-keyed 用：root 下驱动（scripts/mutation/ 的 crates_mutation_rows_*.py、crates-mutation-rows.py、lib_mutation_units.py）
    与算键库（scripts/gate/lib_function_units*.py）各份文件的 sha256，按路径排。平常不读它们：驱动与算键库的代码不进键（文件头「行键」）。"""
    digests = []
    for pattern in ("scripts/mutation/crates_mutation_rows_*.py", "scripts/mutation/crates-mutation-rows.py", "scripts/mutation/lib_mutation_units.py",
                    "scripts/gate/lib_function_units*.py"):
        for path in sorted(glob.glob(os.path.join(root, pattern))):
            with open(path, "rb") as handle:
                digests.append(hashlib.sha256(handle.read()).hexdigest())
    return digests


def row_identity(row):
    """索引（index/）里这一行的名字：变异名与文件。只用来说清现判的原因，不进键。"""
    return hashlib.sha256(f"{row.name}\t{row.path}".encode("utf-8", "surrogateescape")).hexdigest()


def utc_now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def refuse_overrides_outside_service(environment):
    """只给自证换的变量（文件头「判定库」）不在临时 prune-server 底下设了：抛 InputManifestError、点名设了的几个，一条都不跑。
    弄坏开关 run-overrides-honored-outside-service 下放过这两个。"""
    admission = admission_module()
    if under_temporary_prune_server(environment):
        return
    names = ([] if break_is(environment, "run-overrides-honored-outside-service")
             else ["GATE_MUTATION_RUN_RECORDS_DIRECTORY", "GATE_MUTATION_MULTI_HOST_CONFIG"])
    found = [f"{name}（「{environment[name].strip()}」）" for name in names if environment.get(name, "").strip()]
    if found:
        raise admission.InputManifestError(f"不在临时 prune-server 底下设了 {'、'.join(found)}：这几个只给自证换（运行记录、多机配置），"
                                           "平常跑 unset 它们（库根是被判那棵树的 git common-dir）")


def records_home(root, environment):
    """判定库放在哪个目录底下（文件头「判定库」）：被判那棵树的 git common-dir；
    取不到抛 InputManifestError（弄坏开关 library-outside-git-tolerated 下拿被判的根当库根）。先过 refuse_overrides_outside_service。"""
    refuse_overrides_outside_service(environment)
    admission = admission_module()
    common_directory = admission.git_common_directory(root)
    if not common_directory:
        if break_is(environment, "library-outside-git-tolerated"):
            return os.path.abspath(root)
        raise admission.InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），判定库没有名字")
    return common_directory


def library_directory(root, environment):
    return os.path.join(records_home(root, environment), LIBRARY_DIRECTORY_NAME)


def open_library(root, environment):
    return mutation_units.Library(library_directory(root, environment), RECORD_HEADER)


def run_records_directory(root, environment):
    """用时表与占盘记录放哪（文件头「判定库」末段）：GATE_MUTATION_RUN_RECORDS_DIRECTORY 设了取它，没设取多机配置 NFS_SHARED_ROOTS 里
    不含本仓、也不在本仓之内的几段里排在最前的一段；底下再接 crates-mutation-run-records<库的命名空间>（库根在临时目录底下的自证一趟一份，别处两台同一份）。
    配置里没有这一项抛 InputManifestError（不补缺省、不落到 git common-dir）。目录名写在函数体里、不做模块级常量。"""
    admission = admission_module()
    shared_root = environment.get("GATE_MUTATION_RUN_RECORDS_DIRECTORY", "").strip()
    if not shared_root:
        configuration = admission.multi_host_configuration_file(STAGE_REPOSITORY, environment)
        repositories = [os.path.realpath(path) for path in (root, STAGE_REPOSITORY)]

        def overlaps(candidate):
            real = os.path.realpath(candidate)
            return any(real == repository or real.startswith(repository.rstrip(os.sep) + os.sep) or repository.startswith(real.rstrip(os.sep) + os.sep)
                       for repository in repositories)

        candidates = [path for path in admission.configuration_value(configuration, "NFS_SHARED_ROOTS").split(":") if os.path.isabs(path) and not overlaps(path)]
        if not candidates:
            raise admission.InputManifestError(f"多机配置 {configuration} 里没有不含本仓的 NFS_SHARED_ROOTS（两台同一绝对路径的共享根）：用时表与占盘记录没处放")
        shared_root = candidates[0]
    namespace = mutation_units.Library.client().namespace_of(library_directory(root, environment), environment)
    return os.path.join(shared_root, "crates-mutation-run-records" + namespace)


def snapshot_of(base, copied):
    """base 底下拷贝范围里（跳过 target 与 crates/mutations.tsv）每个文件的 {路径: sha256}：只给弄坏开关 baseline-by-tree-snapshot 用（基线按整树快照比），
    基线记录里照记一份当属性，不当复用的依据。"""
    files = {}
    for item in copied:
        path = os.path.join(base, item)
        if os.path.isfile(path):
            if item != TABLE:
                files[item] = hashlib.sha256(read_bytes(path)).hexdigest()
            continue
        for directory, names, file_names in os.walk(path, followlinks=True):
            names[:] = sorted(name for name in names if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
            for name in sorted(file_names):
                full = os.path.join(directory, name)
                relative = os.path.relpath(full, base)
                if relative == TABLE or not os.path.isfile(full):
                    continue
                files[relative] = hashlib.sha256(read_bytes(full)).hexdigest()
    return files


def snapshot_identity(files):
    return hashlib.sha256("".join(f"{files[path]}  {path}\n" for path in sorted(files)).encode("utf-8", "surrogateescape")).hexdigest()


def key_workers(environment):
    """算闭包开几个子进程：GATE_MUTATION_KEY_WORKERS 给了（正整数）照它；不然取每台默认重活线程数（scripts/hosts/heavy_threads.py）。
    登记文件读不了抛 HeavyThreadDefaultError。"""
    requested = environment.get("GATE_MUTATION_KEY_WORKERS", "").strip()
    if re.fullmatch(r"[1-9][0-9]*", requested):
        return int(requested)
    return int(default_heavy_threads(environment))


def row_view(keys, row):
    """这一行的被改坏的单元与点名测试的闭包取哪个视图（lib_mutation_units.py 文件头「视图」）：lib_function_units.py 的 FORMAL_VERIFICATION 或 PRODUCT_BEHAVIOR。
    原文里有产品行为视图剥掉的字（UnitKeys.touches_stripped_text）的取形式化视图，别的（原文不是 .rs、命中不是恰好 1 次、读不了、没有被剥掉的字）取产品行为视图；
    弄坏开关 specification-lines-product-view 下一律取产品行为视图。自己按原文判，不靠 mutated_units 与 resolve_test 谁先谁后
    （resolve_test 的 view 不给，是照紧挨着的上一次 mutated_units：调用次序一调换，视图就悄悄变）。"""
    lfu = mutation_units.function_units()
    if not row.path.endswith(".rs") or mutation_units.break_is("specification-lines-product-view"):
        return lfu.PRODUCT_BEHAVIOR
    try:
        # 单元键记着读过的那一份（UnitKeys.source_bytes：每份被改坏的文件只读一遍，mutated_units 取同一份）；没有这个方法的单元键照原样现读
        reader = getattr(keys, "source_bytes", None)
        if reader is not None:
            text = reader(row.path).decode("utf-8", "surrogateescape")
        else:
            with open(os.path.join(keys.root, row.path), "rb") as handle:
                text = handle.read().decode("utf-8", "surrogateescape")
    except OSError:
        return lfu.PRODUCT_BEHAVIOR
    if text.count(row.old) != 1:
        return lfu.PRODUCT_BEHAVIOR
    start = text.find(row.old)
    return lfu.FORMAL_VERIFICATION if keys.touches_stripped_text(row.path, text, start, start + len(row.old)) else lfu.PRODUCT_BEHAVIOR


def row_unit_keys(root, row, environment, keys_by_directory):
    """(这一行的单元键, None) 或 (None, 拒算那一句)：没写 --manifest-path 的是 <根> 那一份，写了的是那个包自己的一份（unit_keys_under_manifest）；
    --manifest-path 写坏了（manifest_directory_of 交的为什么）这一行拒算。keys_by_directory 是调用方的缓存（包目录 → 单元键，<根> 那一份的包目录是 ""），没有就现建。"""
    directory, why = manifest_directory_of(root, row.arguments, environment)
    if why is not None:
        return None, f"cargo 参数的 --manifest-path 认不出：{why}"
    if directory not in keys_by_directory:
        keys_by_directory[directory] = mutation_units.UnitKeys(root) if directory == "" else unit_keys_under_manifest(root, directory)
    return keys_by_directory[directory], None


def built_workspaces_of(keys):
    """这一份单元键已经切好的工作区（产品行为视图、形式化视图，只列建过的，这里不触发现建）。--manifest-path 指的包取 ManifestUnitKeys 包着的那一份。"""
    inner = getattr(keys, "wrapped_unit_keys", keys)
    return [workspace for workspace in (inner._workspace, inner._formal_workspace) if workspace is not None]


def share_lexed_tokens(workspace, environment):
    """一份工作区里每个文件的词串换成驻留的串，行号与括号配对的下标同一个数共用一个对象（文件头「算键的内存」）：取值不变。回换了几个词串。弄坏开关 key-token-values-unshared：不换。"""
    if break_is(environment, "key-token-values-unshared"):
        return 0
    numbers, tokens = {}, 0
    for rust_file in workspace.files.values():
        rust_file.texts[:] = [sys.intern(text) for text in rust_file.texts]
        rust_file.lines[:] = [numbers.setdefault(line, line) for line in rust_file.lines]
        rust_file.pairs = {numbers.setdefault(opening, opening): numbers.setdefault(closing, closing) for opening, closing in rust_file.pairs.items()}
        tokens += len(rust_file.texts)
    return tokens


def new_key_pool_facts():
    """这一趟算键的进程池 fork 之前做了什么（KeyContext.pool_facts，自证读它）：共用过词法单元的工作区、换了几个词串、父进程先算了几条、冻了多少对象。"""
    return dict(shared_workspaces=set(), tokens_shared=0, prewarmed=0, frozen_objects=0)


def precompute_closures_from_shared_parent(keys, requests, labels, workers, environment, pool_facts):
    """keys.precompute_closures 连同 fork 之前那三样（文件头「算键的内存」）：不到真开子进程的条数或只给 1 个子进程的就地算，那三样不做。回算了几个闭包。
    弄坏开关 key-parent-not-prewarmed：父进程不先算一条；key-pool-not-frozen：不冻。"""
    import gc
    if workers <= 1 or len(requests) < mutation_units.PARALLEL_MINIMUM:
        return keys.precompute_closures(requests, workers, labels)
    for workspace in built_workspaces_of(keys):
        if id(workspace) not in pool_facts["shared_workspaces"]:
            pool_facts["shared_workspaces"].add(id(workspace))
            pool_facts["tokens_shared"] += share_lexed_tokens(workspace, environment)
    computed = 0
    if not break_is(environment, "key-parent-not-prewarmed"):
        with contextlib.suppress(mutation_units.ClosureRefused):
            computed = keys.precompute_closures(requests[:1], 1, labels[:1])
        pool_facts["prewarmed"] += computed
    frozen = not break_is(environment, "key-pool-not-frozen")
    if frozen:
        gc.collect()
        gc.freeze()
        pool_facts["frozen_objects"] = gc.get_freeze_count()
    try:
        return computed + keys.precompute_closures(requests, workers, labels)
    finally:
        if frozen:
            gc.unfreeze()


def released_unit_keys(root, directory, keys):
    """放掉工作区之后留给父进程的那一份单元键（文件头「算键的内存」）：一份新的单元键，工作区换成每个文件的单元指纹表，只回答 current_fingerprints。
    keys 没切过工作区的（那个包里没有一行要它）留空，要用时照常现建。"""
    import types
    released = unit_keys_for_directory(root, directory)
    built = getattr(keys, "wrapped_unit_keys", keys)._workspace
    if built is not None:
        table = {path: types.SimpleNamespace(units=[mutation_units.reference_of(unit) for unit in rust_file.units]) for path, rust_file in built.files.items()}
        absent = types.SimpleNamespace(units=[])
        getattr(released, "wrapped_unit_keys", released)._workspace = types.SimpleNamespace(file=lambda path: table.get(path, absent))
    return released


def release_key_context(key_context, components, environment):
    """派工作进程之前放掉算键的上下文（文件头「算键的内存」）：就地换掉 key_context 的单元键、清闭包，就地把 components 每一份的 test_unit 置空，
    再把父进程的对象冻进永久代（调用方在进程池收完之后 gc.unfreeze）。交 MUTATION_KEY_CONTEXT_RELEASE 那一行。
    弄坏开关 key-context-kept-for-workers：不放；key-pool-not-frozen：不冻。"""
    import ctypes
    import gc
    import weakref
    before = process_usage_bytes(os.getpid())
    released, workspaces = "no", []
    if not break_is(environment, "key-context-kept-for-workers"):
        released = "yes"
        workspaces = [weakref.ref(workspace) for keys in key_context._manifest_keys.values() for workspace in built_workspaces_of(keys)]
        key_context.drop_unit_keys()
        components.update({line_number: comp._replace(test_unit=None) for line_number, comp in components.items()})
    gc.collect()
    with contextlib.suppress(OSError, AttributeError):   # 只有 glibc 有 malloc_trim：别的 libc 上这一步不做，只是还回去的少一点
        ctypes.CDLL(None).malloc_trim(0)
    after = process_usage_bytes(os.getpid())
    left = sum(1 for reference in workspaces if reference() is not None)
    frozen = 0
    if not break_is(environment, "key-pool-not-frozen"):
        gc.freeze()
        frozen = gc.get_freeze_count()
    return (f"MUTATION_KEY_CONTEXT_RELEASE released={released} parent_bytes_before={'none' if before is None else before} "
            f"parent_bytes_after={'none' if after is None else after} workspaces_released={len(workspaces) - left} workspaces_left={left} frozen_objects={frozen}")


class KeyContext:
    """一棵树这一趟算键要的几样（文件头「行键」）：单元键（UnitKeys）；弄坏开关下另有构建环境的行（build_environment_lines）。"""

    def __init__(self, root, environment):
        self.root, self.environment = root, environment
        self.build_environment_lines = build_environment_lines_for_break(root, environment)   # 平常是空表：构建环境不进键（文件头「行键」）
        self.keys = mutation_units.UnitKeys(root)
        self._manifest_keys = {"": self.keys}   # 包目录 → 单元键（row_unit_keys：<根> 那一份是 ""，别的是 unit_keys_under_manifest）
        self.closures = {}
        self.pool_facts = new_key_pool_facts()   # 算键的进程池 fork 之前做了什么（precompute_closures_from_shared_parent）
        self.collisions = []   # 撞键的几组行号（refuse_colliding_row_keys）
        self.gpu_route_lines, self.gpu_route_configuration = [], None   # 补了 gpu 特性的行（行号, 补的特性）与那份多机配置（gpu_route_rows）
        self.gpu_route_unbuilt = []   # 补了 gpu 特性点名的测试就不编、照原样留着的行（行号, 为什么）
        # 命中重测试目标、补了 --release 的行（行号, 依据）与那一份登记的测试目标（plan_rows、gpu_route_rows；文件头「重测试目标只许 release」）
        self.release_lines, self.release_registered = [], None

    def keys_for(self, row):
        """(这一行的单元键, None) 或 (None, 拒算那一句)：单元键按行的包目录取（文件头「--manifest-path」）。
        没写 --manifest-path 的是 <根> 那一份（self.keys），写了的是那个包自己的一份（unit_keys_under_manifest，按包目录缓存）；
        --manifest-path 写坏了（manifest_directory_of 交的为什么）这一行拒算。判法在 row_unit_keys（static-check 可达性用同一份）。"""
        return row_unit_keys(self.root, row, self.environment, self._manifest_keys)

    def prepare(self, rows):
        """先把这一趟每一行点名测试的闭包算好（并行，进程数见 key_workers）；components 之后照缓存取。回算了几个闭包。
        按行的单元键分组（keys_for：<根> 那一份与各个 --manifest-path 指的包各一组）各算各的；几组里闭包算不成的并成一个 ClosureRefused，别的组照算完再抛。"""
        groups = collections.OrderedDict()   # id(单元键) → (单元键, requests, labels)
        for row in rows:
            keys, keys_refusal = self.keys_for(row)
            if keys_refusal is not None:
                continue
            _mutated, refusal = keys.mutated_units(row.path, row.old)
            test, test_refusal = ((None, refusal) if refusal is not None
                                  else keys.resolve_test(row.arguments, row.expected, view=row_view(keys, row)))
            if test_refusal is None:
                _keys, requests, labels = groups.setdefault(id(keys), (keys, [], []))
                requests.append((test.unit, test.target, mutation_units.parse_cargo_arguments(row.arguments)))
                labels.append(f"{TABLE}:{row.line_number} {row.name}")
        computed, failures = 0, []
        for keys, requests, labels in groups.values():
            try:
                computed += precompute_closures_from_shared_parent(keys, requests, labels, key_workers(self.environment), self.environment, self.pool_facts)
            except mutation_units.ClosureRefused as error:
                failures += error.failures
        if failures:
            raise mutation_units.ClosureRefused(failures)
        return computed

    def components(self, row):
        """这一行的 Components：键算得出交齐全的一份，算不出（拒算）row_key 是 None、refusal 写为什么。"""
        keys, refusal = self.keys_for(row)
        if refusal is not None:
            return Components(None, refusal, None, None, None, None, None, None, None)
        mutated, refusal = keys.mutated_units(row.path, row.old)
        if refusal is not None:
            return Components(None, f"被改坏的单元：{refusal}", None, None, None, None, None, None, None)
        test, refusal = keys.resolve_test(row.arguments, row.expected, view=row_view(keys, row))
        if refusal is not None:
            return Components(None, refusal, mutated, None, None, None, None, None, None)
        arguments = mutation_units.parse_cargo_arguments(row.arguments)
        closure = keys.closure(test.unit, test.target, arguments)
        if closure.key is None:
            return Components(None, f"点名测试 {test.full_name} 的闭包拒算（{closure.refusals[0]}）", mutated, test.unit, test.full_name,
                              None, None, None, None)
        self.closures[closure.key] = closure
        # 被改坏的单元只带种类与指纹：路径、名字、行号不进键（D13（验证路线） 已定项 17）；两份内容相同的函数各改一处、点名同一条测试的两行
        # 算出同一把键，由 refuse_colliding_row_keys 判成拒算（文件头「行键」）。落在测试服务代码里的单元连指纹也不带（keyed_mutated_references，
        # lib_mutation_units.py 文件头「测试服务代码不进键」）：改它不让这一行重判；弄坏开关 mutated-test-service-code-keyed：带
        keyed_mutated = mutation_units.keyed_mutated_references(mutated, break_is(self.environment, "mutated-test-service-code-keyed"))
        mutated_lines = [] if break_is(self.environment, "mutated-unit-ignored") else [f"{reference.kind}\t{reference.fingerprint}" for reference in keyed_mutated]
        # 基线键与覆盖号带点名测试所在的包与测试目标：两个目标里各有一条字面相同的测试，基线与覆盖不互相顶替（文件头「判定库」）。
        # 弄坏开关 baseline-key-without-target：不带
        test_target = [] if break_is(self.environment, "baseline-key-without-target") else [
            ("test-target", [f"{keys.workspace.targets[test.target].crate.package}\t{test.target}"])]
        test_key = "" if break_is(self.environment, "test-closure-ignored") else closure.key
        # 弄坏开关 driver-and-library-keyed：驱动与算键库那几份文件的内容并进三把键
        driver_groups = [("driver-and-library", driver_and_library_digests(self.root))] if break_is(self.environment, "driver-and-library-keyed") else []
        # 构建环境（编译变量、cargo 配置、清单的 lints）、编译档、线程数、rustc 与 cargo 的版本、依赖版本都不进键；弄坏开关 build-environment-keyed、profile-keyed、build-jobs-keyed、
        # toolchain-version-keyed 下 build_environment_lines 交这些行，三把键各收一份
        environment_groups = [("build-environment", self.build_environment_lines)] if self.build_environment_lines else []
        # 点名测试的开关与特性：cargo 参数里选测试、定 cfg 的那几样（--release、--target、过滤词、libtest 参数、特性），闭包照它们算（文件头「行键」）
        test_groups = [("switches", arguments.switches), ("features", sorted(keys.features_of(arguments, test.target)))]
        groups = [("form", [ROW_KEY_FORM]), ("mutated", mutated_lines),
                  ("original", [mutation_units.escape_field(row.old)]), ("replacement", [mutation_units.escape_field(row.new)]),
                  ("test", [test_key])] + test_groups + environment_groups + driver_groups
        row_key = mutation_units.key_of_lines(groups)
        baseline_key = mutation_units.key_of_lines([("form", [BASELINE_KEY_FORM]), ("test-unit", [test.unit.fingerprint])] + test_target
                                                   + test_groups + environment_groups + driver_groups)
        # 覆盖号：点名的测试（单元指纹、所在的包与目标）、开关与特性；驱动与算键库的代码不进，覆盖记录里每个执行过的单元的指纹照 baseline_reuse_problem 逐个对现在的源码
        coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM]), ("test-unit", [test.unit.fingerprint])] + test_target
                                                  + test_groups + environment_groups + driver_groups)
        return Components(row_key, None, mutated, test.unit, test.full_name, closure.key, arguments.switches, baseline_key, coverage_id)

    def persist(self, library, components_by_line):
        """组成清单进库（每一份点名测试的闭包），新见到的单元并进 units/。只在 run_stage 里调（plan 只读）。
        环境里开了分步进度（merge_progress_variable）就把「写组成清单」这一步逐行打进度。"""
        progress = merge_step_open(self.environment, "写组成清单", len(components_by_line))
        for done, components in enumerate(components_by_line.values(), 1):
            if components.test_closure in self.closures:
                library.put_composition(components.test_closure, self.closures[components.test_closure].members)
            merge_step_row_done(progress, done)
        library.flush_units()
        merge_step_close(progress, "组成清单写完")

    def drop_unit_keys(self):
        """放掉单元键里的工作区与闭包（release_key_context）。之后还回答 keys_for 与它交的 current_fingerprints，别的算键的方法不再用。"""
        self._manifest_keys = {directory: released_unit_keys(self.root, directory, keys) for directory, keys in self._manifest_keys.items()}
        self.keys = self._manifest_keys[""]
        self.closures = {}


def index_entry(components, row):
    """索引里这一行的那一份（只用来说清现判的原因）：键与几样组成的号。"""
    return collections.OrderedDict(
        row_key=components.row_key or "",
        test_closure=components.test_closure or "", text=hashlib.sha256(f"{row.old}\0{row.new}".encode("utf-8", "surrogateescape")).hexdigest(),
        mutated=",".join(mutation_units.unit_id(reference) for reference in components.mutated or []), recorded_date=mutation_units.today_text())


def explain_fresh(library, components, row, previous, context):
    """键对不上的一行：(栏, 细节)。照索引里这一行上一次的那一份逐样比（被改坏的单元 → 点名测试的闭包 → 原文与替换文），
    变了的那几个单元列进细节（文件头「判定库」作废时报变了哪几个单元）。"""
    if previous is None:
        return "row-new", "判定库里没有这一行的记录（新加的行，或改了变异名、文件）"
    if previous.get("row_key") == components.row_key:
        return "not-caught", "上一趟算过这把键而没留下判定（被打断、记录没写成，或判出的不是抓到）"
    current_ids = ",".join(mutation_units.unit_id(reference) for reference in components.mutated)
    if previous.get("mutated") != current_ids:
        names = "、".join(f"{reference.file}::{reference.name}" for reference in components.mutated)
        return "mutated", f"被改坏的单元 {names} 变了"
    if previous.get("test_closure") != components.test_closure:
        closure = context.closures.get(components.test_closure)
        changes = mutation_units.changed_units(library, previous.get("test_closure"), closure.members if closure else [])
        return "test", f"点名测试 {components.test_full_name} 的闭包：{mutation_units.summarize_changes(changes)}"
    if previous.get("text") != hashlib.sha256(f"{row.old}\0{row.new}".encode("utf-8", "surrogateescape")).hexdigest():
        return "text", "这一行的原文或替换文改了"
    return "unexplained", "键对不上而几样组成的号都与上一次相同（cargo 参数里的开关或特性改了）"


def row_decision(library, components, row, context, previous=None, record=None):
    """这一行这一趟复用还是现判（文件头「复用判据」）：(Decision, 记录)。拒算的 column 是 refused。"""
    if components.row_key is None:
        return Decision(False, "refused", components.refusal), None
    if record is None:
        record = library.read("records", components.row_key)
    if record is not None and record.get("row_key") == components.row_key:
        if record.get("verdict") == REUSABLE_VERDICT["mutation"]:
            return Decision(True, None, f"键相同（{record.get('judged_date', '日期没记')} {record.get('how', '')} 判的抓到）"), record
        return Decision(False, "not-caught", f"上次判的是 {record.get('verdict') or '（没写）'}，只有抓到的复用"), record
    column, detail = explain_fresh(library, components, row, previous, context)
    return Decision(False, column, detail), None


def read_timing(timings_directory, content_hash):
    """上一次跑的秒数；没有记过、读不了交 None。用时表是运行记录（文件头「判定库」末段），timings_directory 是运行记录目录底下的 timings/。"""
    try:
        with open(os.path.join(timings_directory, content_hash), encoding="utf-8", errors="surrogateescape") as handle:
            record = mutation_units.parse_record(handle.read())
    except OSError:
        return None
    try:
        return float(record.get("seconds_running", ""))
    except ValueError:
        return None


def selftest_seed_short_timings(root, environment):
    """自证里 run_stage 起之前：root 的变异表里每一行在用时表里还没有数的，记成 10 秒（短于长测试的门槛，也短于派活前移的门槛）：别的格比的是判定与起 cargo 的次数，
    不随「没有用时的测试各占一块」的切法改数；那条切法自己的格（selftest_long_baseline_chunk_run_cell、selftest_baseline_timing_write_cell）
    把 Selftest.seeds_short_timings 关掉。没有变异表的 root 不记。"""
    table = os.path.join(root, TABLE)
    if not os.path.isfile(table):
        return
    try:
        timings_directory = os.path.join(run_records_directory(root, environment), "timings")
    except admission_module().InputManifestError:
        return   # 取不到运行记录目录（不在 git 里、不该设的变量设了）：run_stage 自己判红，那几格比的就是它的拒绝
    os.makedirs(timings_directory, exist_ok=True)
    for row in read_table(table, environment)[0]:
        path = os.path.join(timings_directory, row_content_hash(row))
        if not os.path.exists(path):
            write_atomically(path, mutation_units.record_text(RECORD_HEADER, dict(seconds_running="10")))


def library_of(repository):
    """自证里那棵仓的判定库目录：只当 prune-server（临时服务）里的名字与命名空间，本地没有文件（文件头「判定库」）。"""
    return os.path.join(repository, ".git", LIBRARY_DIRECTORY_NAME)


def selftest_library(directory):
    """自证读写库目录名是 directory 的那一份库（经 prune-server；库目录在临时目录底下，命名空间一趟一份）。"""
    return mutation_units.Library(directory, RECORD_HEADER)


def selftest_child_module_loader(name):
    """子进程正文的开头：按路径（argv[1]）载入本模块，绑定到 module。name 只写标识符，不带引号。"""
    return ("import importlib.util, sys\n"
            f"specification = importlib.util.spec_from_file_location('{name}', sys.argv[1])\n"
            "module = importlib.util.module_from_spec(specification)\n"
            "sys.modules[specification.name] = module\n"
            "specification.loader.exec_module(module)\n")


def selftest_key_pool_child_program():
    """假驱动的子进程正文：FakeKeys 的 precompute_closures 代替真算闭包。父进程有一大片容器当工作区，就地算那一趟（workers 是 1）建一份缓存；
    进程池那一趟 fork 两个子进程，每个先回收一次、再照「缓存不在就自己建一份」做一次，交两段各涨了多少私有脏页（smaps_rollup 的 Private_Dirty）。"""
    return selftest_child_module_loader("crates_mutation_rows_key_pool_child") + (
        "import gc, json, multiprocessing, os\n"
        "heap = [[index] for index in range(2000000)]\n"
        "answers = []\n"
        "def private_dirty():\n"
        "    with open('/proc/self/smaps_rollup', encoding='ascii') as handle:\n"
        "        return next(int(line.split()[1]) * 1024 for line in handle if line.startswith('Private_Dirty:'))\n"
        "def worker(_index):\n"
        "    started = private_dirty()\n"
        "    gc.collect()\n"
        "    collected = private_dirty()\n"
        "    built = None if FakeKeys.memo is not None else [[index] for index in range(1000000)]\n"
        "    finished = private_dirty()\n"
        "    del built\n"
        "    return dict(collecting=collected - started, caching=finished - collected)\n"
        "class FakeKeys:\n"
        "    memo = None\n"
        "    _workspace = None\n"
        "    _formal_workspace = None\n"
        "    def precompute_closures(self, requests, workers, labels=None):\n"
        "        if workers <= 1:\n"
        "            FakeKeys.memo = [[index] for index in range(1000000)]\n"
        "            return len(requests)\n"
        "        with multiprocessing.get_context('fork').Pool(processes=2) as pool:\n"
        "            answers.extend(pool.map(worker, range(2), chunksize=1))\n"
        "        return len(requests) - 1\n"
        "facts = module.new_key_pool_facts()\n"
        "count = module.mutation_units.PARALLEL_MINIMUM\n"
        "computed = module.precompute_closures_from_shared_parent(FakeKeys(), list(range(count)), [''] * count, 2, dict(os.environ), facts)\n"
        "json.dump(dict(answers=answers, computed=computed, requested=count, prewarmed=facts['prewarmed'], frozen=facts['frozen_objects'], frozen_after=gc.get_freeze_count()), sys.stdout)\n")


def selftest_key_pool_fake_cells(selftest):
    """算键的进程池内存有界（文件头「算键的内存」）：假驱动造 200 万个容器的父进程，不起真工具。回收一次、用一次缓存都不许把父进程的页抄成子进程私有的。"""
    bound = 32 << 20
    child = subprocess.run([sys.executable, "-c", selftest_key_pool_child_program(), MODULE_PATH], env=dict(selftest.base_environment),
                           capture_output=True, text=True, errors="replace", timeout=600, check=False)
    try:
        answer = json.loads(child.stdout)
    except ValueError:
        answer = None
    detail = f"退 {child.returncode}；{answer}；子进程 stderr 尾 {child.stderr[-300:]!r}"
    measured = answer is not None and len(answer["answers"]) == 2
    selftest.check("算键的进程池：假驱动上 fork 之前冻进永久代：子进程回收一次，私有脏页涨幅不超过 32 MiB，池收完永久代是空的",
                   measured and max(item["collecting"] for item in answer["answers"]) <= bound and answer["frozen"] >= 2000000 and answer["frozen_after"] == 0, detail)
    selftest.check("算键的进程池：假驱动上父进程先算一条：子进程用的缓存建在父进程，私有脏页涨幅不超过 32 MiB，这一条算进回的条数",
                   measured and max(item["caching"] for item in answer["answers"]) <= bound and answer["prewarmed"] == 1 and answer["computed"] == answer["requested"], detail)


def selftest_key_pool_closure_keys(repository, rows, environment, workers):
    """真跑的小仓上照 KeyContext.prepare 同一份路子算每一行的闭包：(单元键, [每一行的闭包键], 这一趟的 pool_facts)。"""
    keys = mutation_units.UnitKeys(repository)
    requests, labels = [], []
    for row in rows:
        keys.mutated_units(row.path, row.old)
        test, _refusal = keys.resolve_test(row.arguments, row.expected, view=row_view(keys, row))
        requests.append((test.unit, test.target, mutation_units.parse_cargo_arguments(row.arguments)))
        labels.append(row.name)
    facts = new_key_pool_facts()
    precompute_closures_from_shared_parent(keys, requests, labels, workers, environment, facts)
    return keys, [keys.closure(*request).key for request in requests], facts


def selftest_key_release_child_program():
    """假驱动的子进程正文：一份假工作区（文件表与一大片容器）、一份 test_unit 也攥着一大片容器的 Components，调 release_key_context，交放之前、放之后的用量与放之后的回答。"""
    return selftest_child_module_loader("crates_mutation_rows_key_release_child") + (
        "import gc, json, os, types\n"
        "class FakeWorkspace:\n"
        "    pass\n"
        "def usage():\n"
        "    return module.process_usage_bytes(os.getpid())\n"
        "root = sys.argv[2]\n"
        "environment = dict(os.environ)\n"
        "baseline = usage()\n"
        "fingerprint = 'f' * 64\n"
        "workspace = FakeWorkspace()\n"
        "workspace.ballast = [[index] for index in range(1000000)]\n"
        "workspace.files = {'a.rs': types.SimpleNamespace(units=[types.SimpleNamespace(kind='fn', file='a.rs', name='kept', first_line=1, last_line=1, fingerprint=fingerprint)])}\n"
        "keys = module.mutation_units.UnitKeys(root)\n"
        "keys._workspace = workspace\n"
        "context = module.KeyContext.__new__(module.KeyContext)\n"
        "context.root, context.environment, context.keys, context._manifest_keys = root, environment, keys, {'': keys}\n"
        "context.closures = {'closure': [[index] for index in range(200000)]}\n"
        "components = {1: module.Components(*[None] * 9)._replace(test_unit=[[index] for index in range(1000000)], row_key='row')}\n"
        "del workspace, keys\n"
        "gc.collect()\n"
        "before = usage()\n"
        "line = module.release_key_context(context, components, environment)\n"
        "gc.unfreeze()\n"
        "after = usage()\n"
        "reference = module.mutation_units.UnitReference('fn', 'a.rs', 'kept', '1-1', fingerprint)\n"
        "current = context.keys_for(module.Row(1, 'row', 'a.rs', 'a', 'b', '-p tiny --lib', 'kept', ()))[0].current_fingerprints([reference])\n"
        "json.dump(dict(baseline=baseline, before=before, after=after, line=line, current=sorted(current[('fn', 'a.rs', 'kept')]), test_unit=components[1].test_unit is None,\n"
        "               row_key=components[1].row_key, closures=len(context.closures)), sys.stdout)\n")


def selftest_key_release_fake_cell(selftest, work):
    """放掉算键的上下文，父进程的用量真的下来（文件头「算键的内存」）：假驱动造的工作区、闭包与 test_unit 各攥一大片，不起真工具。弄坏开关 key-context-kept-for-workers 下判错。"""
    directory = os.path.join(work, "key-release-fake")
    selftest_write(os.path.join(directory, "a.rs"), "fn kept() {}\n")
    child = subprocess.run([sys.executable, "-c", selftest_key_release_child_program(), MODULE_PATH, directory], env=dict(selftest.base_environment),
                           capture_output=True, text=True, errors="replace", timeout=600, check=False)
    try:
        answer = json.loads(child.stdout)
    except ValueError:
        answer = None
    selftest.check("派工作进程之前：假驱动上放掉工作区、闭包与 test_unit：放之前比底座多 100 MiB 以上，放之后至多多 32 MiB，单元指纹照答，别的字段不动",
                   answer is not None and answer["before"] - answer["baseline"] >= 100 << 20 and answer["after"] - answer["baseline"] <= 32 << 20
                   and "released=yes" in answer["line"] and "workspaces_released=1 workspaces_left=0" in answer["line"] and answer["current"] == ["f" * 64]
                   and answer["test_unit"] and answer["row_key"] == "row" and answer["closures"] == 0,
                   f"退 {child.returncode}；{answer}；子进程 stderr 尾 {child.stderr[-300:]!r}")


def selftest_write_timings(directory, rows, seconds):
    """rows 每一行在用时表目录 directory 里记 seconds 秒（直接写文件，不经 run_stage）。"""
    os.makedirs(directory, exist_ok=True)
    for row in rows:
        write_atomically(os.path.join(directory, row_content_hash(row)), mutation_units.record_text(RECORD_HEADER, dict(seconds_running=str(seconds))))


def selftest_reuse_cells(selftest, work):
    """复用三问（reuse-three-questions）与复用哨兵（reuse-sentinel）两格，写法在文件头「复用三问与复用哨兵」。"""
    repository = os.path.join(work, "three-questions")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    sentinel_environment = {reuse_checks.SENTINEL_COUNT_VARIABLE: "1", reuse_checks.SENTINEL_SEED_VARIABLE: "5"}
    forced_how = "现判：" + COLUMN_TITLES["sentinel"]

    def states(run):
        found = {}
        for name, how in per_row_how(run[1]).items():
            found[name] = (reuse_checks.FORCED if how == forced_how else reuse_checks.JUDGED if how.startswith("现判")
                           else reuse_checks.REUSED if how == "复用" else how)
        return found

    runs = [selftest.run(repository, **sentinel_environment)]
    runs.append(selftest.run(repository, **sentinel_environment))
    replace_in_file(source, "pub fn pad(", "pub fn unused_in_three_questions() -> u32 { 72 }\npub fn pad(")
    runs.append(selftest.run(repository, **sentinel_environment))
    replace_in_file(source, "base.max(OFFSET)", "base.max(OFFSET + 0)")
    runs.append(selftest.run(repository, **sentinel_environment))
    passed, problems = reuse_checks.report_three_questions(SCRIPT_KEY, *[states(run) for run in runs], ["root-457", "root-double"])
    selftest.check(reuse_checks.CELL_NAME, passed and all(run[0] == 0 for run in runs),
                   f"四趟退 {[run[0] for run in runs]}；{'；'.join(problems)}；{runs[-1][2][-600:]}")
    green = selftest.run(repository, **sentinel_environment)
    skipped = selftest.run(repository, GATE_MUTATION_BREAK="reuse-lookup-skipped", **sentinel_environment)
    forced = [name for name, how in per_row_how(green[1]).items() if how == forced_how]
    selftest.check(reuse_checks.SENTINEL_CELL_NAME, green[0] == 0 and "复用哨兵（crates 变异表）：种子 5" in green[2] and len(forced) == 1
                   and skipped[0] == 1 and "复用失灵：" in skipped[1],
                   f"照常那一趟退 {green[0]}、强制现判 {forced}：{green[2][-400:]}；不读库那一趟退 {skipped[0]}：{skipped[1][-400:]}")


def selftest_multiple_packages_cell(selftest, work):
    """cargo 参数里有多个 -p（lib_mutation_units.py 文件头「点名的测试」）：自证函数在 lib_mutation_units.py 的 multiple_packages_cell，这里登记、跑它。
    弄坏开关 multiple-packages-last-only（只认最后一个 -p）下这一格必须判错。"""
    passed, note = mutation_units.multiple_packages_cell(work)
    selftest.check("多个 -p：键按并集特性算、与 -p 的先后无关、改别的包的无关函数键不变、改共有包里点名测试够得着的函数键变", passed, note)


def selftest_specification_line_cell(selftest, work):
    """verus! 里的规格行（lib_mutation_units.py 文件头「视图」）：自证函数在 lib_mutation_units.py 的 specification_line_cell，这里登记、跑它。
    弄坏开关 specification-lines-product-view（规格行也取产品行为视图）下这一格必须判错。"""
    passed, note = mutation_units.specification_line_cell(work)
    selftest.check(mutation_units.SPECIFICATION_LINE_CELL, passed, note)


def selftest_mutate_plan_test_service_code_cell(selftest, work):
    """mutate.sh 的键（lib_mutation_units.py 文件头「测试服务代码不进键」）：自证函数在 lib_mutation_units.py 的 mutate_plan_test_service_code_cell，这里登记、跑它。
    弄坏开关 mutated-test-service-code-keyed（被改坏的单元落在测试服务代码里也把指纹放进键）下这一格必须判错。"""
    passed, note = mutation_units.mutate_plan_test_service_code_cell(work)
    selftest.check(mutation_units.MUTATE_PLAN_SERVICE_CELL, passed, note)


def selftest_uncounted_keeps_counted_cells(selftest, work):
    """不算数的结局不盖算数的（文件头「复用判据」记红撤绿那一段）：先抓到入库；同一把键经复用哨兵强制现判一行、记本次未跑（假 cargo 强令），
    库里每一行仍是抓到、下一趟照复用；另一棵没有记录的小仓同样本次未跑，照写本次未跑。
    弄坏开关 uncounted-overwrites-counted 下「库里一行抓到…现判出本次未跑之后」那一格判错。"""
    row_count = len(SELFTEST_TABLE_ROWS)

    def verdicts_in(repository):
        library = selftest_library(library_of(repository))
        return [library.read("records", name).get("verdict") for name in library.stored_names("records")]

    kept = os.path.join(work, "reuse-fix-uncounted-keeps-counted")
    build_selftest_repository(kept, SELFTEST_TABLE_ROWS)
    caught_first = selftest.run(kept)
    not_run_over = selftest.run(kept, FAKE_CARGO_FORCED_OUTCOME="notrun", **selftest_one_forced(row_count))
    verdicts_after_not_run = verdicts_in(kept)
    after_not_run = selftest.run(kept)
    selftest.check("不算数的结局不盖算数的：库里一行抓到，同一把键经哨兵强制现判出本次未跑之后，库里仍是抓到、下一趟照复用（现判 0 条）",
                   caught_first[0] == 0 and not_run_over[0] == 0 and "本次未跑：1 条" in not_run_over[1]
                   and verdicts_after_not_run == ["caught"] * row_count and after_not_run[0] == 0
                   and f"现判 0 条、复用 {row_count} 条" in after_not_run[1],
                   f"头一趟退 {caught_first[0]}；本次未跑那一趟退 {not_run_over[0]}；之后库里的判定 {verdicts_after_not_run}；"
                   f"下一趟退 {after_not_run[0]}：{after_not_run[1][-300:]}{not_run_over[1][-300:]}")
    fresh = os.path.join(work, "reuse-fix-uncounted-fresh")
    build_selftest_repository(fresh, SELFTEST_TABLE_ROWS)
    fresh_not_run = selftest.run(fresh, FAKE_CARGO_FORCED_OUTCOME="notrun")
    selftest.check(f"不算数的结局不盖算数的：库里没有这把键的记录时，本次未跑照行键写（{row_count} 条本次未跑的记录）",
                   fresh_not_run[0] == 77 and verdicts_in(fresh) == ["notrun"] * row_count,
                   f"退 {fresh_not_run[0]}；库里的判定 {verdicts_in(fresh)}")


