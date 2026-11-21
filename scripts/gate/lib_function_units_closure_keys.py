"""键与摘要：闭包的键（closure_key）、entry 行、与 unit-graph 对拍的键（unit_graph_key）、dump 与 explain 的输出行、变异控制组，
Rust 的 rust_units、rust_closure，跨轮缓存的登记与现取位置。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import copy
import importlib.util
import os
import posixpath
import re
import shutil
import sys
import tempfile
import time
from lib_function_units_lexer import active_breaks, rust_lex_cache_directory
from lib_function_units_base import (
    FORMAL_VERIFICATION, FunctionUnitsError, KEY_FORM, LIBRARY_ROOT, Refusal, Unit, data_directory_unit, data_file_unit, declared_checks,
    sha256_text, snippet_of,
)
from lib_function_units_shell_text import shell_assignment_row, shell_assignment_value
from lib_function_units_shell_index import ShellIndex, inline_refusals, shell_units
from lib_function_units_python_module import PythonIndex, python_units
from lib_function_units_unit_graph import unit_graph_product_view_cache_directory
from lib_function_units_rust_workspace import DEFAULT_RUST_CFG, KNOWN_CFG_NAMES, RustWorkspace, cfg_entries
from lib_function_units_wgsl import wgsl_units
from lib_function_units_rust_closure import rust_closure_of
from lib_function_units_script_closure import parse_entry
from lib_function_units_registry import is_test_service_code


# ── 跨轮缓存的登记（文件头「跨轮缓存的登记」）──

TEST_ENVIRONMENT_STAGE_SCRIPT = ".claude/gate.d/test-environment.sh"
TEST_ENVIRONMENT_STAGE_SHARED_ENTRY = re.compile(r'python3 "\$STAGE_DIR/\.\./hooks/lib_work_root\.py" shared ([a-z][a-z0-9-]*)\)"')


def work_root_library():
    """这份库所在仓里的 .claude/hooks/lib_work_root.py（数据根的算法只在它那一处）：按路径载入、不进 sys.modules。"""
    specification = importlib.util.spec_from_file_location("lib_work_root_for_cross_run_caches",
                                                           os.path.join(LIBRARY_ROOT, ".claude", "hooks", "lib_work_root.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def test_environment_stage_run_times_file():
    """test-environment 记的上一趟各阶段用时放哪：照 TEST_ENVIRONMENT_STAGE_SCRIPT 此刻的算法逐段取，位置的每一段都读自那份脚本：
    「local run_times_file cross_run_cache=」那一行读 GATE_CROSS_RUN_TMPDIR（设了取它）；没设时那份脚本里恰好一处 `python3 "$STAGE_DIR/../hooks/lib_work_root.py" shared <名>)"`
    给出名字，目录交 work_root_library().shared_data_path(<名>)（脚本是调同一个库的同名命令行，.claude/gate.d 的上一层是 .claude，所以是这份库所在仓的 .claude/hooks）；
    「run_times_file=」那一行照这个目录展开成文件。任何一段在脚本里找不到恰好一处就抛 ValueError（脚本改了写法，照新写法改这个函数）。
    弄坏开关 cross-run-cache-run-times-old-spelling：只认「local run_times_file=」起头的写法（脚本里一行都没有，抛 ValueError）。"""
    if "cross-run-cache-run-times-old-spelling" in active_breaks():
        return shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "local run_times_file=")
    cross_run_cache = shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "local run_times_file cross_run_cache=")
    if not cross_run_cache:
        with open(os.path.join(LIBRARY_ROOT, TEST_ENVIRONMENT_STAGE_SCRIPT), encoding="utf-8") as handle:
            entries = TEST_ENVIRONMENT_STAGE_SHARED_ENTRY.findall(handle.read())
        if len(entries) != 1:
            raise ValueError(f"{TEST_ENVIRONMENT_STAGE_SCRIPT} 里调 lib_work_root.py shared <名> 取目录的写法有 {len(entries)} 处（要恰好一处）："
                             "那份脚本改了写法，照新写法改 test_environment_stage_run_times_file")
        cross_run_cache = work_root_library().shared_data_path(entries[0])
    return shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "run_times_file=", {"cross_run_cache": cross_run_cache})


def cross_run_cache_registry():
    """仓里有意跨轮复用的缓存（文件头「跨轮缓存的登记」）：[(名字, 放它的脚本, 现取它的位置的函数)]；现取的是那份脚本自己算位置的那一段，不在这里另抄一份位置。
    写成函数、不写成模块级的表：表里现取位置的那几个函数（载入 lib_work_root.py、读别的脚本）不随 import 时执行的模块级语句进别的闭包（文件头「Python 闭包」）。"""
    return (
        ("Rust 词法单元缓存", "scripts/gate/lib_function_units_lexer.py", rust_lex_cache_directory),
        ("产品行为视图编出的 unit-graph 程序", "scripts/gate/lib_function_units.py", unit_graph_product_view_cache_directory),
        ("test-environment 记的上一趟各阶段用时", TEST_ENVIRONMENT_STAGE_SCRIPT, test_environment_stage_run_times_file),
        shell_assignment_row("LKMM 判法取的内核内存模型", ".claude/scripts/lkmm.sh", "CACHE="),
        shell_assignment_row("取依赖脚本的下载缓存", ".claude/scripts/fetch-deps.sh", "CACHE="),
    )


def cross_run_cache_rows():
    """→ [(名字, 脚本, 路径或 None, 算不出的原因或 None)]，照 cross_run_cache_registry 的次序、照此刻的环境现取。"""
    rows = []
    for name, script, locate in cross_run_cache_registry():
        try:
            path, problem = locate(), None
        except Exception as error:  # 现取的那一段在别的脚本里：载入或调用抛什么都照「算不出」报，不让一份带垮别的几份
            path, problem = None, f"{type(error).__name__}：{error}"
        if path is None and problem is None:
            problem = "取到 None（缓存关着：FUNCTION_UNITS_LEX_CACHE_DIRECTORY=off 这一类改放处变量设着）"
        rows.append((name, script, path, problem))
    return rows


def print_cross_run_cache_rows():
    """命令行 cross-run-caches（自证 CROSS_RUN_CACHE_CELL 在拷贝里起它）：先打 tempdir 一行（此刻的 tempfile.gettempdir()），再每份缓存一行，交退出码（有算不出的交 3）。"""
    print(f"tempdir\t{tempfile.gettempdir()}")
    rows = cross_run_cache_rows()
    for name, script, path, problem in rows:
        print(f"cache\t{name}\t{script}\t{path}" if problem is None else f"refused-cache\t{name}\t{script}\t{problem}")
    unresolved = [name for name, _script, _path, problem in rows if problem is not None]
    if unresolved:
        print(f"  ✗ {len(unresolved)} 份跨轮缓存的位置算不出：{'、'.join(unresolved)}", file=sys.stderr)
        print("    → 怎么办：看 refused-cache 那几行的原因，照那份脚本此刻的写法改 cross_run_cache_registry 里它那一行", file=sys.stderr)
        return 3
    print(f"  ✓ 现取 {len(rows)} 份跨轮缓存的位置（登记表 cross_run_cache_registry）", file=sys.stderr)
    return 0


RustClosureResult = collections.namedtuple("RustClosureResult", "members key refusals cfg_lines closure")


def rust_units(root):
    """→ [(种类, 限定名, 文件, (起, 止), 指纹, cfg 链)]，cfg 链是这个单元在它那份文件里的条件（外层模块与内属性的也在里面），
    原文写法、由外到里；mod 声明带进来的那一截按目标定，不在这里（closure 时按入口的目标求值）。"""
    workspace = RustWorkspace(root)
    return [(unit.kind, unit.name, unit.file, (unit.first_line, unit.last_line), unit.fingerprint, [raw for _tree, raw, _line in unit.conditions or ()])
            for rust_file in workspace.all_files() for unit in sorted(rust_file.units, key=Unit.sort_key)]


def rust_closure(root, entry, target=None, cfg=None):
    """入口（<文件>::<限定名> 或 fn 路径）在目标（默认按文件挑）与 cfg 集（加进默认全局 cfg 的几项）下 → 成员单元行、键（拒算时 None）、拒算。"""
    workspace = RustWorkspace(root)
    _language, path, qualified = parse_entry(root, entry)
    unit, target_id = workspace.find_entry(path, qualified, target)
    closure = rust_closure_of(workspace, unit, target_id, cfg)
    key, refusals = closure_key(closure)
    return RustClosureResult([member.text() for member in closure.units()], key, refusals, closure.cfg_lines, closure)


# 按行进键的登记表（文件头「按行进键的登记表」）：表（相对根）→ (读整张表、交解析结果的函数名, {只按其中几行取用的函数名: 它看的那几行的正则})；
# 正则里的 {entry} 换成这一条判定的入口文件（re.escape 过）
ROW_KEYED_TABLES = {
    ".claude/hooks/selftest-registry.tsv": (frozenset(("read_selftest_registry", "selftest_registry_at")), {
        "registered_selftest": r"^{entry}\s",                                    # 按被起的那份脚本的键查它自己那一行
        "stage_samples_row": r"^\.claude/gate\.d/[^/\s]+\s",                     # 按门禁阶段查样本档那一行
        "registered_sample_tool_directories": r"^\.claude/gate\.d/[^/\s]+\s",    # 样本档行里写的假工具目录
    }),
}


def row_keyed_closure(closure, root, entry_file):
    """按行进键的登记表（文件头「按行进键的登记表」）：闭包里 ROW_KEYED_TABLES 登记的那张表的 data-file 单元，闭包里调读表函数的每个单元
    （读表函数自己除外）都是登记的取用函数时，换成只含那几个取用函数看的行的 data-rows 单元；有一个调用者不在登记里、entry_file 是 None、
    弄坏开关 row-keyed-tables-whole 下照旧整份。交回一份新的 Closure（原来那一份不动；没换的交原来那一份）。"""
    if entry_file is None or "row-keyed-tables-whole" in active_breaks():
        return closure
    replaced = {}
    for unit in closure.order:
        if not (unit.data and unit.kind == "data-file" and unit.name in ROW_KEYED_TABLES):
            continue
        readers, consumers = ROW_KEYED_TABLES[unit.name]
        patterns = set()
        for member in closure.order:
            if member.data or member.language != "python" or member.module is None or member.name.split(".")[-1] in readers:
                continue
            facts = member.module.facts(member)
            if not (readers & (set(facts.names) | {attribute for _base, attribute in facts.attributes})):
                continue
            pattern = consumers.get(member.name.split(".")[-1])
            if pattern is None:
                patterns = None
                break
            patterns.add(pattern.replace("{entry}", re.escape(entry_file)))
        if not patterns:
            continue
        expressions = [re.compile(pattern) for pattern in sorted(patterns)]
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="surrogateescape") as handle:
            selected = [line for line in handle.read().split("\n") if any(expression.search(line) for expression in expressions)]
        fingerprint = sha256_text("\n".join(sorted(patterns)) + "\0" + "\n".join(selected))
        replaced[id(unit)] = Unit("data", "data-rows", unit.name, unit.file, 1, max(len(selected), 1), fingerprint, data=True)
    if not replaced:
        return closure
    projected = copy.copy(closure)
    projected.order = [replaced.get(id(unit), unit) for unit in closure.order]
    projected.members = {id(unit): unit for unit in projected.order}
    projected.via = dict(closure.via)
    for old, new in replaced.items():
        projected.via[id(new)] = closure.via.get(old)
    return projected


def closure_key(closure, toolchain=(), dependencies=(), externals=None, allow_refused=False):
    """→ (键或 None, 拒算列表)。拒算不空时键是 None（allow_refused 只给对照用：照算出键，拒算照样交回）。
    版本不进键（文件头「版本不进键」）：这份库与 admission.py 都不给 toolchain、dependencies、externals，闭包里的工具链询问不出工具链单元，
    闭包点名外部程序（closure.programs）不拒算；三样只在调用方给了时照出（scripts/mutation 与 lib_verdict_library.py 还给）。
    弄坏开关 external-programs-refused 下闭包点名了外部程序而调用方没给 externals 的拒算 external-program-unkeyed。
    拒算记在 closure.refusals 里，别处并起来的闭包（lib_verdict_library.py 的 FunctionUnitIndex.key）照拒算并过去，不悄悄少收。
    闭包里测试服务代码的单元（is_test_service_code 认的文件里的，数据单元同样）不出 unit 行与 data 行（文件头「测试服务代码不进键」）：
    闭包照旧经它们往下收，entry 行照出，落在测试服务代码里的入口单元是服务代码自己的（脚本体、非测试的函数）在 entry 行里只记是哪一个、不记指纹，
    是测试的（Rust 的测试函数、测试目标里的单元、文档测试）照记指纹（entry_key_lines 的 service_code_unkeyed）。
    弄坏开关 test-service-code-keyed 下 unit 行与 data 行照出。"""
    externals = dict(externals or {})
    refusals = set(closure.refusals)
    if "external-programs-refused" in active_breaks():
        for program, (file, line) in sorted(closure.programs.items()):
            if program not in externals:
                refusals.add(Refusal("external-program-unkeyed", file, line, program))
    ordered = sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    if ordered and not allow_refused:
        return None, ordered
    lines = [f"form\t{KEY_FORM}"]
    lines += sorted(f"cfg\t{item}" for item in key_cfg_items(closure, closure.cfg_lines))
    lines += sorted(f"toolchain\t{item}" for item in toolchain)
    lines += sorted(f"dependency\t{item}" for item in dependencies)
    lines += sorted(f"external\t{name}\t{sha}\t{version}" for name, (sha, version) in externals.items())
    service_code_keyed = "test-service-code-keyed" in active_breaks()
    keyed = [unit for unit in closure.order if service_code_keyed or not is_test_service_code(unit.file)]
    lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in keyed if not unit.data)
    lines += sorted(f"data\t{unit.kind}\t{unit.fingerprint}" for unit in keyed if unit.data)
    lines += entry_key_lines(closure, service_code_unkeyed=True)
    return sha256_text("".join(line + "\n" for line in lines)), ordered


def entry_key_lines(closure, service_code_unkeyed=False):
    """键里的 entry 行（文件头「键」）：closure.entry_units 里每个入口单元连同包它的 impl / trait 头（Unit.container）一层层往外，各取（种类，指纹），
    一个入口一行、排序去重；不带名字、路径与行号（测试服务代码里的单元除外，见下）。闭包是单元的多重集，几个入口互相收进对方的闭包（一个 trait impl 里的几个方法经 R-结构、R-隐式，
    python 里互相调用的两个 def）时闭包一模一样，没有这几行就共用一把键、一份判定（scripts/gate/verus-proof-check.py 的 entry_unit_component 同一种写法）。
    文档测试取到它自己就停：它的 container 是所挂的条目，rust_closure_of 的 R-结构不连它（另编一个 crate），键里也不带它的正文。
    service_code_unkeyed（closure_key 给；文件头「测试服务代码不进键」）：落在测试服务代码里的单元（is_test_service_code）不取指纹，取（种类，<文件>::<限定名>）：
    改它的正文键不变，几个这样的入口照旧各是各的键；入口是测试的（closure.test_code_entry_units：rust_closure_of 记的测试函数、测试目标里的单元、
    文档测试，连同包它的 impl / trait 头）照取指纹，改测试自己的正文键变。unit_graph_key 不给（与 crates/singlefs-unit-graph 对拍，那一份不读测试服务代码登记表）。
    没记入口的闭包（别处自己拼的、并起来的）不出这几行。弄坏开关 key-entry-units-dropped：一行都不出；
    key-entry-doctest-owner-chained：文档测试照别的单元往外取所挂的条目；test-service-code-entry-keyed：测试服务代码里的入口单元照取指纹；
    test-service-code-test-entry-unkeyed：测试服务代码里是测试的入口单元也只记是哪一个。"""
    if "key-entry-units-dropped" in active_breaks():
        return []
    unkeyed = service_code_unkeyed and "test-service-code-entry-keyed" not in active_breaks()
    test_entries = frozenset() if "test-service-code-test-entry-unkeyed" in active_breaks() else frozenset(getattr(closure, "test_code_entry_units", ()))
    lines = set()
    for entry in getattr(closure, "entry_units", ()):
        chain, owner, visited = [], entry, set()
        while owner is not None and id(owner) not in visited:
            visited.add(id(owner))
            if unkeyed and is_test_service_code(owner.file) and id(owner) not in test_entries:
                chain.append(f"{owner.kind}\t{owner.file}::{owner.name}")
            else:
                chain.append(f"{owner.kind}\t{owner.fingerprint}")
            if owner.kind == "doctest" and "key-entry-doctest-owner-chained" not in active_breaks():
                break
            owner = owner.container
        lines.add("entry\t" + "\t".join(chain))
    return sorted(lines)


def key_cfg_items(closure, unnarrowed):
    """键里的 cfg 那几项：闭包记了真引到的原子（cfg_atoms，「<原子>\\t<true|false>」）就只取它们；没记的（别的脚本自己拼的闭包、
    并起来的闭包没并 cfg_atoms）退回 unnarrowed（整套生效 cfg：只多不少，键照样对，只是带不带特性的两趟分开存）。
    弄坏开关 rust-cfg-atoms-unnarrowed 下一律退回 unnarrowed。"""
    narrowed = getattr(closure, "cfg_atoms", None)
    if narrowed is None or "rust-cfg-atoms-unnarrowed" in active_breaks():
        return list(unnarrowed)
    return list(narrowed)


def unit_graph_key_lines(closure, toolchain=(), lock_lines=()):
    """unit_graph_key 取 sha256 的那几行（次序就是 crates/singlefs-unit-graph 的 reuse_key 拼的次序）：entry-target-kind\\t<目标种类>、
    cfg\\t<原子>\\t<true|false>（闭包里各单元真引到的 cfg 原子与它在收那个单元的目标下的真值，排好序；key_cfg_items）、
    unit\\t<种类>\\t<指纹>（闭包里每个 Rust 与 WGSL 单元，排序）、data\\t<摘要>（每一处读数据文件，排序）、
    entry\\t<种类>\\t<指纹>[\\t<种类>\\t<指纹>…]（入口单元自己，entry_key_lines：一个入口一行、排序去重；Rust 那一份的 entry_key_lines 同一种行、同一个位置）、
    toolchain\\t<行>、lock\\t<行>（调用方给的次序）。弄坏开关 unit-graph-key-entry-lines-dropped：不出 entry 行。"""
    lines = [f"entry-target-kind\t{closure.entry_target_kind}"]
    lines += [f"cfg\t{atom}" for atom in sorted(key_cfg_items(closure, closure.entry_cfg_atoms))]
    lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)
    lines += sorted(f"data\t{site[3]}" for site in closure.data_sites)
    if "unit-graph-key-entry-lines-dropped" not in active_breaks():
        lines += entry_key_lines(closure)
    lines += [f"toolchain\t{line}" for line in toolchain]
    lines += [f"lock\t{line}" for line in lock_lines]
    return lines


def unit_graph_key(closure, toolchain=(), lock_lines=()):
    """照 crates/singlefs-unit-graph 的 reuse_key 算 Rust 闭包的键（两份实现逐入口对拍用；python 这一份自己的复用键仍是 closure_key）→ (键或 None, 拒算)。
    unit_graph_key_lines 那几行以 \\n 连、末尾不带，取 sha256。闭包里有拒算不交键。"""
    ordered = sorted(closure.refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    if ordered:
        return None, ordered
    return sha256_text("\n".join(unit_graph_key_lines(closure, toolchain, lock_lines))), []


def dump_lines(root, language=None, under=None, judgment_kind=FORMAL_VERIFICATION, product_dump=None):
    """→ (单元行, 拒算行, 各语言单元数)。判定种类是产品行为时 Rust 单元照产品行为视图（文件头「判定种类」），只给 Verus 看的不列。"""
    units, refusals, counts = [], set(), collections.Counter()
    if language in (None, "rust"):
        workspace = RustWorkspace(root, judgment_kind, product_dump)
        names, pairs = cfg_entries(list(DEFAULT_RUST_CFG))
        known = set(KNOWN_CFG_NAMES)
        for crate in workspace.crates:
            known |= crate.check_cfg
            if crate.proc_macro:
                refusals.add(Refusal("proc-macro-crate", posixpath.join(crate.directory, "Cargo.toml"), 1, f"proc-macro = true（{crate.package}）"))
        for target in workspace.targets.values():
            refusals.update(target.refusals)
        data_digests = {}
        for rust_file in workspace.all_files():
            refusals.update(rust_file.refusals)
            for unit in rust_file.units:
                if unit.ghost_only:
                    continue
                units.append(unit)
                counts["rust"] += 1
                refusals.update(unit.refusals)
                if "rust-data-dump-unlisted" not in active_breaks():
                    # 数据单元照 crates/singlefs-unit-graph 的 dump：每一处读（include_str! 一类与运行期拼得出字面路径的读，.wgsl 不算）一行
                    # data\t<数据文件>\t<读它的文件>:<行>-<行>\t<字节 sha256>
                    # 读的是目录的（折出 crates/ 之下的目录）写 data-directory\t<目录>\t<读它的文件>:<行>-<行>\t<目录下全部文件的指纹>，指纹见 directory_fingerprint
                    for kind, value, index in unit.data_references or ():
                        if kind not in ("file", "directory"):
                            continue
                        if (kind, value) not in data_digests:
                            data_digests[(kind, value)] = (data_file_unit(root, value) if kind == "file" else data_directory_unit(root, value)).fingerprint
                        line = rust_file.lines[index]
                        site = Unit("data", "data" if kind == "file" else "data-directory", value, unit.file, line, line, data_digests[(kind, value)], data=True)
                        units.append(site)
                        counts["data"] += 1
                if unit.kind == "extern_block" and not unit.listed_c_library_calls:
                    refusals.add(Refusal("extern-block", unit.file, unit.first_line, rust_file.row(unit.first_line)))
                for tree, raw, line in unit.conditions or ():
                    if tree is None or not cfg_tree_known(tree, known):
                        refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
    if language in (None, "wgsl"):
        for item in wgsl_units(root):
            refusals.update(item.refusals)
            units += item.units
            counts["wgsl"] += len(item.units)
    if language in (None, "python"):
        python_index, modules = python_units(root, under)
        empty_shell = ShellIndex(root, [])
        for module in modules:
            refusals.update(module.refusals)
            refusals |= inline_refusals(root, python_index, empty_shell, declared_checks(root, module.path, module.rows))
            for unit in module.units:
                units.append(unit)
                counts["python"] += 1
                facts = module.facts(unit)
                refusals.update(facts.refusals)
                refusals |= inline_refusals(root, python_index, empty_shell, facts.inline_scripts)
    if language in (None, "shell"):
        index = shell_units(root, under)
        inline_python = PythonIndex(root)
        for path in sorted(index.files):
            shell_file = index.files[path]
            refusals.update(shell_file.refusals)
            refusals |= inline_refusals(root, inline_python, index, declared_checks(root, path, shell_file.rows))
            for unit in shell_file.units:
                units.append(unit)
                counts["shell"] += 1
                if not unit.data:
                    for kind, files, refusal in index.links(unit):
                        if refusal is not None:
                            refusals.add(refusal)
                        if kind == "python-inline":
                            refusals |= inline_refusals(root, inline_python, index, files)
    unit_lines = [unit.dump_text() for unit in sorted(units, key=dump_sort_key)]
    refusal_lines = [refusal.text() for refusal in sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))]
    return unit_lines, refusal_lines, counts


def dump_sort_key(unit):
    """dump 的次序：Rust、WGSL 与数据单元照 crates/singlefs-unit-graph 的 sorted_texts（文件、起行、止行、整行）；python 与 shell 照 Unit.sort_key。
    同一份文件里只有一种语言，两种排法不会比到一起。"""
    if unit.language in ("rust", "wgsl", "data"):
        return (unit.file, unit.first_line, unit.last_line, unit.dump_text(), "")
    return (unit.file, unit.first_line, unit.kind, unit.name, "")


def cfg_tree_known(tree, known):
    if tree[0] in ("all", "any", "not"):
        return all(cfg_tree_known(item, known) for item in tree[1])
    return tree[1] in known


def explain_lines(closure):
    lines = []
    for unit in closure.units():
        via, rule = closure.via.get(id(unit), (None, ""))
        source = f"{via.file}:{via.name}" if via is not None else "-"
        lines.append(f"via\t{unit.file}:{unit.name}\t{source}\t{rule}")
    lines += [f"round\t{number}\t{size}" for number, size in enumerate(closure.round_sizes, 1)]
    lines += [f"table-launch\t{file}\t{outcome}\t{' '.join(words) if words else '-'}\t{f'{via.file}:{via.name}' if via is not None else '-'}"
              for file, words, via, outcome in closure.table_launches]
    return lines


# ── V5 / V11 对照（design 报告 2.2 节） ──────────────────────────────────────

CONTROL_ENTRIES = (
    ("v5_test", "crates/singlefs-verification/tests/new_pool_file_creation_publish.rs::root_slots_system_configurations_and_journal_ring_hold_the_published_state"),
    ("v11_test", "crates/singlefs-format/src/lib.rs::tests::pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for"),
    ("cpu_judge", "crates/singlefs-verification/src/crash.rs::judge_crash_image"),
)
CONTROL_VARIANTS = (
    ("V5", "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;", {"v5_test": "changed", "v11_test": "same", "cpu_judge": "changed"}),
    ("V11", "pub const DATA_POINTER_BYTES: u64 = 88;", {"v5_test": "changed", "v11_test": "changed", "cpu_judge": "changed"}),
)


def mutation_row(root, original):
    with open(os.path.join(root, "crates", "mutations.tsv"), encoding="utf-8") as handle:
        for number, row in enumerate(handle, 1):
            fields = row.rstrip("\n").split("\t")
            if len(fields) >= 4 and fields[2].replace("\\n", "\n") == original:
                return number, fields[1], fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n")
    raise FunctionUnitsError(f"crates/mutations.tsv 里找不到原文是「{original}」的一行")


def control_keys(copy_root):
    workspace = RustWorkspace(copy_root)
    out = {}
    for label, entry in CONTROL_ENTRIES:
        _language, path, qualified = parse_entry(copy_root, entry)
        unit, target_id = workspace.find_entry(path, qualified, None)
        closure = rust_closure_of(workspace, unit, target_id)
        key, refusals = closure_key(closure, allow_refused=True)
        out[label] = (key, len(closure.order), len(refusals))
    return out


def run_controls(root):
    breaks = active_breaks()
    with tempfile.TemporaryDirectory(prefix="function-units-controls-") as work:
        copy_root = os.path.join(work, "copy")
        os.makedirs(copy_root)
        shutil.copytree(os.path.join(root, "crates"), os.path.join(copy_root, "crates"), ignore=shutil.ignore_patterns("target"))
        for name in ("Cargo.toml", "Cargo.lock"):
            if os.path.isfile(os.path.join(root, name)):
                shutil.copy2(os.path.join(root, name), os.path.join(copy_root, name))
        started = time.monotonic()
        baseline = control_keys(copy_root)
        for label, (key, members, refused) in baseline.items():
            print(f"base\t{label}\tkey={key[:16]}\tunits={members}\trefused={refused}")
        failures = 0
        for variant, original, expected in CONTROL_VARIANTS:
            number, file, before, after = mutation_row(root, original)
            path = os.path.join(copy_root, file)
            with open(path, encoding="utf-8") as handle:
                text = handle.read()
            if text.count(before) != 1:
                raise FunctionUnitsError(f"{file} 里「{before}」命中 {text.count(before)} 次（要恰好一次）")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text.replace(before, after))
            now = control_keys(copy_root)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            for label, _entry in CONTROL_ENTRIES:
                got = "changed" if now[label][0] != baseline[label][0] else "same"
                want = "same" if "rust-constants-dropped" in breaks else expected[label]
                mark = "✓" if got == want else "✗"
                failures += got != want
                print(f"  {mark} {variant}（crates/mutations.tsv:{number}）\t{label}\t{got}\t期望 {want}")  # gate-lint:detail
        elapsed = time.monotonic() - started
    if failures:
        print(f"  ✗ 对照 {failures} 格与 design 报告 2.2 节的表不符（共 {len(CONTROL_VARIANTS) * len(CONTROL_ENTRIES)} 格，{elapsed:.1f} 秒）")
        print("    → 怎么办：看不符那一格的入口闭包（closure <根> <入口> --explain）里有没有那个常量，补第三节规则里漏的那一条")
        return 1
    print(f"  ✓ 对照 {len(CONTROL_VARIANTS) * len(CONTROL_ENTRIES)} 格与 design 报告 2.2 节的表一致（{elapsed:.1f} 秒）")
    return 0


# ── 自证 ────────────────────────────────────────────────────────────────────




def write_tree(base, files):
    for path, text in files.items():
        absolute = os.path.join(base, path)
        os.makedirs(os.path.dirname(absolute), exist_ok=True)
        with open(absolute, "w", encoding="utf-8") as handle:
            handle.write(text)


def edit_file(base, path, old, new):
    absolute = os.path.join(base, path)
    with open(absolute, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        raise AssertionError(f"自证样本 {path} 里「{old}」命中 {text.count(old)} 次（要恰好一次）")
    with open(absolute, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
