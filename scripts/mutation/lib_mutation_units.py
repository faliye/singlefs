#!/usr/bin/env python3
"""变异判定的函数单元键与按条判定库：crates 变异表（scripts/mutation/crates-mutation-rows.py，门禁 crates-mutation-replay）
与实验变异表（scripts/mutation/mutate.sh）共用这一份。只被 import，没有命令行入口；判法由两边的 --selftest 覆盖
（crates-mutation-rows.py「单元键」那几格，mutate.sh 的 unit-library 那几例）。

切分与闭包不在这里：Rust 的函数单元、去注释指纹与从入口收的闭包都调 scripts/gate/lib_function_units.py（按文件路径 import，不另写一份，
它的文件头「种类与限定名」「键」「拒算」）。这里做三件事：
  ① 一条变异落在哪几个单元上（UnitKeys.mutated_units）、点名的测试是哪一个单元（UnitKeys.resolve_test）、一个单元的闭包键与组成清单（UnitKeys.closure）；
  ② 工具链行与依赖版本行（toolchain_lines、dependency_lines）；
  ③ 按条的判定库（Library）：每条判定按键寻址存进 prune-server，组成清单另按内容寻址存。

单元：
  被改坏的单元 = 原文在那份 .rs 里占的行与之重叠的单元里最里层的那几个：fn 项（连同函数体里的嵌套条目），函数外的项按项单元
    （const、static、struct、enum、union、type、trait 与 impl 头、宏定义、use……）。原文不落在任何单元上（只在注释、空白或文件头的内属性里）
    拒算。文件是 .wgsl 的按 fn 切（UnitKeys.wgsl_mutated_units，切法只在 lib_function_units.py 的 wgsl_file 一处）：与原文所占的行重叠的顶层单元，
    原文在 fn 里取那个 wgsl_fn（带 @compute 的是 wgsl_compute），不在 fn 里的取它自己的单元（wgsl_const、wgsl_struct、wgsl_var 这类）；
    文件切不开、原文不落在任何单元上（只在注释或空白里）拒算，不退回整份文件。别的不是 .rs 的文件（litmus 这类测试读的数据文件）按数据单元取整份字节的 sha256。
  视图：变异行键与点名测试的闭包平常取产品行为视图（UnitKeys.workspace：verus! 里的规格、证明与 #[verifier::…] 外属性不进键）；原文里有产品行为视图剥掉的字
    （lib_function_units.py 的 verus_ghost_code_blanked 剥过之后与原样不同，例如 verus! 里 assume_specification 与 spec fn 那几行）的这一行，被改坏的单元与点名测试的闭包
    改取形式化视图（UnitKeys.formal_workspace，规格与证明进键；产品行为视图里这几行不在任何单元上，会拒算）：mutated_units 判出来记下这一行用哪个视图，
    紧接着的 resolve_test 与 closure 照它取（resolve_test 的 view 参数另可点名）；原文里没有被剥掉的字的行照旧取产品行为视图。
  弄坏开关 specification-lines-product-view（GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK，只给证红用）：一律取产品行为视图（改前的样子）；
    自证是 specification_line_cell（本库的函数，crates-mutation-rows.py 的 --selftest 跑它）；这个开关下它必须判错。
  弄坏开关 wgsl-whole-file（GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK，只给证红用）：.wgsl 照改前当数据单元、取整份字节的 sha256；
  crates-mutation-rows.py --selftest 的「WGSL 按 fn 取键」那一格必须判错。
  点名的测试 = cargo test 参数选中的编译目标（--lib、--bin <名>、--test <名>；都没写是这个包的 lib、bin 与 tests/ 目标）里，
    带 #[test]（或 #[<路径>::test]）的 fn 单元里 libtest 全名（文件在目标里的模块路径加单元的限定名）等于点名、或以「::<点名>」收尾的恰好一个；
    一个都没有、对上几个都拒算（测试由宏展开出来的就是一个都没有）。cargo 参数里写了几个 -p（-p a -p b）：每个包的选中目标一起找，对上恰好一个才算；
    同一趟编的几个包，依赖图上共有的包按几个包请求的特性并集编（UnitKeys.features_of：别的 -p 包的缺省特性、命令行里的特性、它们依赖边上请求的特性并进
    「包名/特性」交给 lib_function_units.py；crates-mutation-rows.py 行键里的特性行照这个并集写，只写一个 -p 的行不变）。
    弄坏开关 multiple-packages-last-only（GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK）：只留最后一个 -p（改前的样子）；
    自证是 multiple_packages_cell（本库的函数，crates-mutation-rows.py 的 --selftest 跑它）；这个开关下它必须判错。
  闭包 = lib_function_units.py 的 rust_closure_of 从那个单元收的（测试单元按 cfg(test) 编；cargo 参数里有 --release 时 debug_assertions 不进 cfg），
    带上 cargo 参数里的特性；闭包里有拒算，这一个闭包就没有键。
闭包算不成（precompute_closures 抛 ClosureRefused）：入口在这一组 cfg 下不编（点名的测试带 #[cfg(feature = "gpu")]、cargo 参数没开那个特性）、没有叫这个名字的包这一类
  lib_function_units.py 的 UsageError，不是拒算（拒算是一个闭包没有键、这一行照跑），是这一行的参数与源码对不上。并行算（fork 出来的子进程）与就地算都把别的闭包算完，
  最后抛一个 ClosureRefused，逐处带类名、原消息与点名它的调用方标签；工作进程只交类名与消息，不交异常对象（按路径载入的模块没登记进 sys.modules 时它 pickle 不回父进程，
  进程池就把真错换成 PicklingError，改前 --list-items 末尾就是这一句）。
  弄坏开关 closure-error-masked（GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK，只给证红用）：模块不登记进 sys.modules、工作进程把异常原样抛（改前的样子）；
  crates-mutation-rows.py --selftest 的「点名的测试在这一组特性下不编」一格必须判错。

工具链行（toolchain_lines）：rustc -vV 与 cargo -V 原样各行；环境里影响编出什么的变量（RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、RUSTDOCFLAGS、
  CARGO_BUILD_RUSTFLAGS、CARGO_BUILD_TARGET、RUSTC_WRAPPER、RUSTC_WORKSPACE_WRAPPER、CARGO_PROFILE_*、CARGO_TARGET_<三元组>_{RUSTFLAGS,RUNNER,LINKER}）；
  从根往上每一层与 CARGO_HOME 的 .cargo/config.toml（或 config）解析之后的 build（除 jobs、target-dir、incremental）、target、profile、env 各键；
  根 Cargo.toml 解析之后的 profile、lints、workspace.lints 各键。只取解析后的值，不取原文（注释与排版不进）；文件路径不进（按层级号排）。
依赖版本行（dependency_lines，与 crates/singlefs-unit-graph/src/lock_file.rs 的 narrowed_lock_lines 同一条规则）：闭包里每个 Rust 单元真点名的外部 crate
  （单元的词法单元里 use <外部 crate>::…、路径 <外部 crate>::…、extern crate <外部 crate>；外部 crate 是单元所在 crate 清单里不带 path 的依赖，
  代码名是依赖的键把 - 换成 _，包名取 package = 写的、没写取键），在根 Cargo.lock 里那个工作区 crate 自己的包（同名、没有 source）的 dependencies 里
  找到它、按名与版本落到恰好一个 [[package]]，再顺 dependencies 收传递依赖；一包一行「名<TAB>版本<TAB>来源<TAB>checksum」（没有的写 -）。
  落不到一个包的拒算（lock-dependency-unresolved），不静默跳过。闭包用到的工作区包里别的外部依赖不收。
  与 Rust 那一份的差别只有一处：那边按依赖的键找包，不认 package = 改名。

判定库（Library）：在 prune-server 结论表里，键 = mutation-units[@命名空间]/<库目录名>/<相对路径>（scripts/prune-server/prune_server_client.py 文件头「键空间」）。
  库目录由调用方给（<库根的 git common-dir>/singlefs-crates-mutation-library、…/singlefs-mutate-library 这类），只当名字与命名空间：本地不建、不读、不写，
  库目录落在临时目录底下的另带命名空间（自证一趟一个库，不串）。服务连不上客户端退 75（PruneServerFailure，是 SystemExit），调用方不当「库里没有」现判。
  records/<键>            一条判定（key=value 文本，一个字段一行）：判定、要打的那一行、判定日期 judged_date、跑法 how、各样组成的号
  baselines/<键>          基线 ok 的判定（crates 变异表用）
  coverage/<号>.json.gz   覆盖记录：一条测试执行过的单元（crates 变异表用）
  compositions/<号>.json.gz  一份组成清单（一个闭包的单元号列表，或一组工具链行、依赖版本行），号是它的内容键
  units/<单元号>          单元号 → [种类, 文件, 限定名, 起-止, 指纹]（组成清单里出现过的单元；作废时按它报变了哪几个单元）；每个单元一条，
                          内容只由单元号定，几个进程同时写不丢
  index/<名字与文件的 sha256>  这一行上一次的键与几样组成的号：只用来说清「现判」的原因，不当复用的依据
  服务没有删的操作：库不按天数、个数、版本回收。用时表、占盘这类运行记录不进库（crates-mutation-rows.py 文件头「判定库」末段）。
  单元号 = sha256（种类 NUL 文件 NUL 限定名 NUL 指纹）的前 16 位十六进制：只当组成清单里的索引，不进任何键。
"""
import collections
import contextlib
import datetime
import gzip
import hashlib
import importlib.util
import json
import multiprocessing
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib
import traceback
import zlib

sys.dont_write_bytecode = True
LIBRARY_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
FUNCTION_UNITS_PATH = os.path.join(LIBRARY_DIRECTORY, "..", "gate", "lib_function_units.py")
_FUNCTION_UNITS = []

UNIT_ID_LENGTH = 16
PARALLEL_MINIMUM = 8   # 要算的闭包不到这么多就在本进程里算（fork 子进程的开销比算还大）
BUILD_VARIABLES = frozenset(("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS", "CARGO_BUILD_RUSTFLAGS",
                             "CARGO_BUILD_TARGET", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"))
BUILD_KEYS_NOT_AFFECTING_OUTPUT = frozenset(("jobs", "target-dir", "incremental"))
CONFIGURATION_SECTIONS = ("build", "target", "profile", "env")
MANIFEST_SECTIONS = ("profile", "lints")
NAME_TAKING_SWITCHES = frozenset(("-p", "--package", "--bin", "--test", "--example", "--bench", "--features", "-F", "--manifest-path", "--skip"))
SWITCHES_NOT_AFFECTING_OUTCOME = frozenset(("-j", "--jobs", "--offline", "--quiet", "-q", "--color", "--message-format"))
TEST_TARGET_KINDS = ("lib", "bin", "test")


class MutationUnitsError(Exception):
    """算不出键（工具链起不来、Cargo.lock 读不懂这一类）：调用方报错、这一趟不写库。"""


class ClosureRefused(MutationUnitsError):
    """点名测试的闭包没算成（lib_function_units.py 的 UsageError：入口在这一组 cfg 下不编、没有叫这个名字的包这一类）。
    failures：[(类名, 消息, [点名它的那几行的标签])]，同一类名同一消息的并成一条；精确到哪一行由调用方在 precompute_closures 的 labels 里给。
    进程池的工作进程里抛的异常对象不一定能 pickle 回父进程（按路径载入的模块没登记进 sys.modules 时就不能），所以工作进程只交类名与消息，
    父进程在这里照原话报出（文件头「闭包算不成」）。"""

    def __init__(self, failures):
        self.failures = failures
        super().__init__(f"{len(failures)} 处点名测试的闭包算不成：" + "；".join(
            f"{class_name}：{message}" + (f"（点名它的：{'、'.join(labels)}）" if labels else "") for class_name, message, labels in failures))

    def detail_lines(self):
        """给人看的逐处明细：一处两行（原话、点名它的行）。"""
        lines = []
        for class_name, message, labels in self.failures:
            lines.append(f"{class_name}：{message}")
            if labels:
                lines.append(f"  点名它的：{'、'.join(labels)}")
        return lines


def break_is(name):
    """弄坏开关（只给证红用）：GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK（逗号分隔）里有 name。本库没有环境参数，读本进程的环境。"""
    return any(name in os.environ.get(variable, "").split(",") for variable in ("GATE_MUTATION_BREAK", "CRATES_MUTATION_ROWS_BREAK"))


def function_units():
    """scripts/gate/lib_function_units.py，第一次用到时按路径载入（调用方的 sys.path 不一定有它的目录）。"""
    if not _FUNCTION_UNITS:
        specification = importlib.util.spec_from_file_location("lib_function_units_for_mutation_units", FUNCTION_UNITS_PATH)
        module = importlib.util.module_from_spec(specification)
        if not break_is("closure-error-masked"):
            # 登记进 sys.modules：模块里的异常类（UsageError）pickle 时按「模块名 + 类名」找类，找不到进程池就把真错换成 PicklingError（文件头「闭包算不成」）
            sys.modules[specification.name] = module
        specification.loader.exec_module(module)
        _FUNCTION_UNITS.append(module)
    return _FUNCTION_UNITS[0]


def sha256_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def escape_field(text):
    return text.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t")


def key_of_lines(groups):
    """[(标签, [值…])…] → sha256：每样一组，组内排序，「标签\\t值\\n」接起来。值是空列表的那一样只留标签一行（区分缺与空）。"""
    lines = []
    for label, values in groups:
        values = sorted(values)
        lines += [f"{label}\t{value}" for value in values] if values else [f"{label}\t-"]
    return sha256_text("".join(line + "\n" for line in lines))


def today_text():
    return datetime.date.today().isoformat()


# ── 单元 ──────────────────────────────────────────────────────────────────────

UnitReference = collections.namedtuple("UnitReference", "kind file name lines fingerprint")


def reference_of(unit):
    return UnitReference(unit.kind, unit.file, unit.name, f"{unit.first_line}-{unit.last_line}", unit.fingerprint)


def unit_id(reference):
    return hashlib.sha256("\0".join((reference.kind, reference.file, reference.name, reference.fingerprint)).encode("utf-8", "surrogateescape")).hexdigest()[:UNIT_ID_LENGTH]


ClosureInfo = collections.namedtuple("ClosureInfo", "key members dependencies refusals")


class CompactMembers:
    """一个闭包的成员：单元号串（每个 UNIT_ID_LENGTH 位十六进制，接起来）加一张几个闭包共用的单元表，要的时候才还原成 UnitReference。
    几千个闭包、每个上千个单元时，全还原成元组要几百 M 内存；组成清单进库只要单元号（Library.put_composition 认它）。"""

    def __init__(self, identifiers, table):
        self.identifiers_text, self.table = identifiers, table

    def identifiers(self):
        text = self.identifiers_text
        return [text[index:index + UNIT_ID_LENGTH] for index in range(0, len(text), UNIT_ID_LENGTH)]

    def __iter__(self):
        for identifier in self.identifiers():
            yield UnitReference(*self.table[identifier])

    def __len__(self):
        return len(self.identifiers_text) // UNIT_ID_LENGTH


_PARALLEL = {}


def _parallel_closure(task):
    """并行算闭包的一格（在 fork 出来的子进程里跑，单元对象与工作区都是父进程那一份的拷贝）：交 (下标, 键, 依赖版本行, 拒算, 单元号串, 这个子进程头一回交的单元)。"""
    index, unit_index, target_id, arguments = task
    keys = _PARALLEL["keys"]
    sent = _PARALLEL.setdefault("sent", set())
    try:
        info = keys.closure(_PARALLEL["units"][unit_index], target_id, arguments)
    except Exception as error:   # 异常对象不交回父进程（不一定 pickle 得动）：只交类名与消息，不认得的类另带回溯，父进程照原话报（ClosureRefused）
        if break_is("closure-error-masked"):
            raise
        known = isinstance(error, function_units().UsageError)
        return index, None, None, None, None, None, (type(error).__qualname__, str(error), "" if known else traceback.format_exc())
    keys._closures.clear()   # 子进程里不留：每格只算一次，留着只占内存
    identifiers = [unit_id(reference) for reference in info.members]
    fresh = {identifier: tuple(reference) for identifier, reference in zip(identifiers, info.members) if identifier not in sent}
    sent.update(fresh)
    return index, info.key, info.dependencies, info.refusals, "".join(identifiers), fresh, None
TestUnit = collections.namedtuple("TestUnit", "unit target full_name")
CargoArguments = collections.namedtuple("CargoArguments", "packages targets features no_default_features all_features switches release filters")


def parse_cargo_arguments(arguments):
    """cargo test 参数 → CargoArguments：包（每个 -p / --package 一个，按出现次序去重）、目标、特性、开关、是否 release、过滤词。
    `--` 前后的测试过滤词都进 switches 的 filter: 项，避免筛掉点名测试后沿用旧结论。
    弄坏开关 multiple-packages-last-only：只留最后一个 -p（改前的样子）。"""
    words = arguments.split() if isinstance(arguments, str) else list(arguments)
    before, after = (words[:words.index("--")], words[words.index("--") + 1:]) if "--" in words else (words, [])
    packages, targets, features, switches, filters = [], [], [], [], []
    no_default = all_features = release = False
    index = 0
    while index < len(before):
        word = before[index]
        name, equals, inline_value = word.partition("=")
        value = inline_value if equals else (before[index + 1] if index + 1 < len(before) else "")
        takes_value = name in NAME_TAKING_SWITCHES or name in ("--profile", "--target", "-j", "--jobs", "--color", "--message-format")
        step = 1 if equals or not takes_value else 2
        if name in ("-p", "--package"):
            if break_is("multiple-packages-last-only"):
                packages = [value]
            elif value not in packages:
                packages.append(value)
        elif name in ("--bin", "--test", "--example", "--bench"):
            targets.append((name[2:], value))
        elif name == "--lib":
            targets.append(("lib", None))
        elif name in ("--bins", "--tests"):
            targets.append((name[2:-1], "*"))
        elif name in ("--features", "-F"):
            features += [item for item in value.replace(",", " ").split() if item]
        elif name == "--no-default-features":
            no_default = True
        elif name == "--all-features":
            all_features = True
        elif name == "--release" or (name == "--profile" and value in ("release", "bench")):
            release = True
            switches.append(word if step == 1 else f"{name}={value}")
        elif name not in SWITCHES_NOT_AFFECTING_OUTCOME and name.startswith("-"):
            switches.append(word if step == 1 else f"{name}={value}")
        elif not name.startswith("-") and not (index == 0 and word in ("test", "check")):
            filters.append(word)
        index += step
    libtest_switches, index = [], 0
    while index < len(after):
        word = after[index]
        if word == "--skip":
            libtest_switches.append("--skip=" + (after[index + 1] if index + 1 < len(after) else ""))
            index += 2
            continue
        if word.startswith("-"):
            libtest_switches.append(word)
        else:
            filters.append(word)
        index += 1
    switches += [f"libtest:{word}" for word in libtest_switches]
    switches += [f"filter:{escape_field(word)}" for word in filters]
    if no_default:
        switches.append("--no-default-features")
    if all_features:
        switches.append("--all-features")
    return CargoArguments(tuple(packages), targets, features, no_default, all_features, sorted(switches), release, filters)


class UnitKeys:
    """一棵树（Rust 工作区的根）上的单元：被改坏的单元、点名的测试、闭包键，都在这一份 RustWorkspace 上算，闭包按入口缓存。"""

    def __init__(self, root):
        self.root = os.path.abspath(root)
        self._workspace = None
        self._formal_workspace = None   # 形式化视图的工作区：第一次有一行要用才建（文件头「视图」）
        self._formal_unit_ids = set()   # 取自形式化视图工作区的单元的 id（closure 凭它选视图）
        self._formal_references = set()   # mutated_units 在形式化视图上交出去的 UnitReference
        self._last_mutation_formal = False   # 紧挨着的上一次 mutated_units 判的是不是形式化视图（resolve_test 默认照它）
        self._blanked = {}   # 文件 → 剥过规格与证明的正文（verus_ghost_code_blanked，剥不了的是 None）
        self._closures = {}
        self._tests = {}
        self._lock = None
        self.unit_table = {}   # 单元号 → UnitReference 的各段（precompute_closures 并行算回来的闭包共用）
        self.dependencies_by_whole_crate = False   # 弄坏开关「按整包收」（mutate.sh 的 dependenciesbycrate）：依赖版本行不看点没点名

    @property
    def workspace(self):
        if self._workspace is None:
            # 变异行键与点名测试的闭包判产品行为：Rust 单元取产品行为视图，verus! 里的规格与证明不进键（lib_function_units.py 文件头「判定种类」）
            self._workspace = function_units().RustWorkspace(self.root, function_units().PRODUCT_BEHAVIOR)
        return self._workspace

    @property
    def formal_workspace(self):
        if self._formal_workspace is None:
            lfu = function_units()
            self._formal_workspace = lfu.RustWorkspace(self.root, lfu.FORMAL_VERIFICATION)
        return self._formal_workspace

    def rust_file(self, path):
        return self.workspace.file(path)

    def unit_of(self, reference):
        """UnitReference 对回的单元（种类、名字、起-止都对得上的那一个）；找不回交 None。被改坏的单元取自哪个视图，这里就在哪个视图的文件里找。"""
        formal = reference in self._formal_references
        rust_file = self.formal_workspace.file(reference.file) if formal else self.rust_file(reference.file)
        found = next((unit for unit in rust_file.units
                      if unit.kind == reference.kind and unit.name == reference.name and f"{unit.first_line}-{unit.last_line}" == reference.lines), None)
        if found is not None and formal:
            self._formal_unit_ids.add(id(found))
        return found

    def touches_stripped_text(self, path, text, start, end):
        """text[start:end]（原文）里有没有产品行为视图剥掉的字：lib_function_units.py 的 verus_ghost_code_blanked 剥过（换成空格、长度不变）与原样不同。
        文件里没有 verus、verifier 两个词的不剥；剥不了（括号配不上）的当没有，照产品行为视图走。"""
        if "verus" not in text and "verifier" not in text:
            return False
        if path not in self._blanked:
            self._blanked[path] = function_units().verus_ghost_code_blanked(text)
        blanked = self._blanked[path]
        return blanked is not None and blanked[start:end] != text[start:end]

    def lock_packages(self):
        if self._lock is None:
            self._lock = read_lock_packages(self.root)
        return self._lock

    def dependencies_of(self, units, workspace=None):
        """一组 lib_function_units.py 的单元的依赖版本行与拒算（文件头「依赖版本行」）；单元取自哪个视图的工作区就给哪个（不给是产品行为视图）。"""
        return dependency_lines(workspace or self.workspace, units, self.lock_packages(), self.dependencies_by_whole_crate)

    def reference_dependencies(self, references):
        """一组 UnitReference（被改坏的单元这一类）各自点名的外部 crate 的依赖版本行与拒算；数据单元与找不回单元的跳过。
        形式化视图上交出去的那几个在形式化视图的文件里找，依赖版本行也照那个视图的词法单元取。"""
        units, formal_units = [], []
        for reference in references:
            if reference.kind == "data-file" or not reference.file.endswith(".rs"):
                continue
            formal = reference in self._formal_references
            rust_file = self.formal_workspace.file(reference.file) if formal else self.rust_file(reference.file)
            (formal_units if formal else units).extend(
                unit for unit in rust_file.units
                if unit.kind == reference.kind and unit.name == reference.name and f"{unit.first_line}-{unit.last_line}" == reference.lines)
        lines, refusals = self.dependencies_of(units)
        if formal_units:
            formal_lines, formal_refusals = self.dependencies_of(formal_units, self.formal_workspace)
            lines, refusals = sorted(set(lines) | set(formal_lines)), sorted(set(refusals) | set(formal_refusals))
        return lines, refusals

    def mutated_units(self, path, original):
        """(被改坏的单元的 UnitReference 列表, None) 或 (None, 拒算那一句)。见文件头「单元」「视图」。"""
        self._last_mutation_formal = False
        try:
            with open(os.path.join(self.root, path), "rb") as handle:
                data = handle.read()
        except OSError as error:
            return None, f"被改坏的文件 {path} 读不了（{error}）"
        if path.endswith(".wgsl") and not break_is("wgsl-whole-file"):
            return self.wgsl_mutated_units(path, data, original)
        if not path.endswith(".rs"):
            return [UnitReference("data-file", path, path, "1-" + str(data.count(b"\n") + 1), hashlib.sha256(data).hexdigest())], None
        text = data.decode("utf-8", "surrogateescape")
        if text.count(original) != 1:
            return None, f"原文在 {path} 里命中 {text.count(original)} 次，要恰好 1 次"
        start = text.find(original)
        first = text.count("\n", 0, start) + 1
        last = max(first, first + original.count("\n") - (1 if original.endswith("\n") else 0))
        formal = not break_is("specification-lines-product-view") and self.touches_stripped_text(path, text, start, start + len(original))
        self._last_mutation_formal = formal
        rust_file = self.formal_workspace.file(path) if formal else self.rust_file(path)
        if rust_file.refusals:
            refusal = rust_file.refusals[0]
            return None, f"{path} 切不开（{refusal.form}，第 {refusal.line} 行）"
        overlapping = [unit for unit in rust_file.units if not unit.data and unit.kind != "doctest" and unit.first_line <= last and unit.last_line >= first]
        innermost = [unit for unit in overlapping
                     if not any(other is not unit and unit.first_line <= other.first_line and other.last_line <= unit.last_line
                                and (other.first_line, other.last_line) != (unit.first_line, unit.last_line) for other in overlapping)]
        if not innermost:
            return None, f"原文（{path} 第 {first}–{last} 行）不落在任何单元上（只在注释、空白或内属性里）"
        references = sorted((reference_of(unit) for unit in innermost), key=lambda reference: (reference.lines, reference.kind, reference.name))
        if formal:
            self._formal_references.update(references)
            self._formal_unit_ids.update(id(unit) for unit in innermost)
        return references, None

    def wgsl_mutated_units(self, path, data, original):
        """.wgsl 的被改坏的单元（文件头「单元」）：按 lib_function_units.py 的 wgsl_file 切出的顶层单元（wgsl_fn、wgsl_compute、wgsl_const、wgsl_struct、
        wgsl_var……），取与原文所占的行重叠的那几个；WGSL 的单元不嵌套，重叠的都是最里层。切不开、原文不落在任何单元上拒算，不退回整份文件。"""
        text = data.decode("utf-8", "surrogateescape")
        if text.count(original) != 1:
            return None, f"原文在 {path} 里命中 {text.count(original)} 次，要恰好 1 次"
        start = text.find(original)
        first = text.count("\n", 0, start) + 1
        last = max(first, first + original.count("\n") - (1 if original.endswith("\n") else 0))
        wgsl_file = function_units().wgsl_file(self.root, path)
        if wgsl_file.refusals:
            refusal = wgsl_file.refusals[0]
            return None, f"{path} 切不开（{refusal.form}，第 {refusal.line} 行）"
        overlapping = [unit for unit in wgsl_file.units if unit.first_line <= last and unit.last_line >= first]
        if not overlapping:
            return None, f"原文（{path} 第 {first}–{last} 行）不落在任何 WGSL 单元上（只在注释或空白里）"
        return sorted((reference_of(unit) for unit in overlapping), key=lambda reference: (reference.lines, reference.kind, reference.name)), None

    def is_test_function(self, rust_file, unit):
        """单元开头的外属性里有 #[test] 或 #[<路径>::test]。"""
        texts, index = rust_file.texts, unit.first_token
        while index is not None and index + 1 < unit.last_token and texts[index] == "#":
            opening = index + 1
            if texts[opening] == "!":
                opening += 1
            close = rust_file.pairs.get(opening) if texts[opening] == "[" else None
            if close is None:
                return False
            inner = texts[opening + 1:close]
            if inner and inner[-1] == "test" and (len(inner) == 1 or inner[-2] == ":"):
                return True
            index = close + 1
        return False

    def candidate_targets(self, arguments):
        """cargo 参数选中的编译目标（RustTarget 列表）；包不在工作区里交 None。"""
        lfu = function_units()
        workspace = self.workspace
        if not arguments.packages:
            crates = [crate for crate in workspace.crates if crate.directory == ""] or list(workspace.crates)
        else:
            try:
                crates = [workspace.crate_by_package(package) for package in arguments.packages]
            except lfu.UsageError:
                return None
        chosen = []
        for crate in crates:
            owned = [target for target in workspace.targets.values() if target.crate is crate and target.kind in TEST_TARGET_KINDS]
            if not arguments.targets:
                chosen += owned
                continue
            for kind, name in arguments.targets:
                chosen += [target for target in owned if target.kind == kind and (name in (None, "*") or target.name == name)]
        return sorted(set(chosen), key=lambda target: target.id)

    def test_units(self, target, formal=False):
        """一个编译目标里的 #[test] fn 单元：[(libtest 全名, 单元)]；formal 是取形式化视图的单元（文件头「视图」）。"""
        if (target.id, formal) not in self._tests:
            found = []
            workspace = self.formal_workspace if formal else self.workspace
            for path, (module_path, _chain) in sorted(target.files.items()):
                rust_file = workspace.files.get(path) or workspace.file(path)
                for unit in rust_file.units:
                    if unit.kind == "fn" and self.is_test_function(rust_file, unit):
                        found.append(("::".join(list(module_path) + [unit.name]), unit))
                        if formal:
                            self._formal_unit_ids.add(id(unit))
            self._tests[(target.id, formal)] = found
        return self._tests[(target.id, formal)]

    def resolve_test(self, arguments, expected, view=None):
        """(TestUnit, None) 或 (None, 拒算那一句)。见文件头「单元」「视图」。view 不给，照紧挨着的上一次 mutated_units 判的视图
        （同一行先问被改坏的单元、再问点名的测试）；给了 lib_function_units.py 的 FORMAL_VERIFICATION 取形式化视图，给 PRODUCT_BEHAVIOR 取产品行为视图。"""
        formal = self._last_mutation_formal if view is None else view == function_units().FORMAL_VERIFICATION
        parsed = parse_cargo_arguments(arguments) if not isinstance(arguments, CargoArguments) else arguments
        targets = self.candidate_targets(parsed)
        if targets is None:
            return None, f"cargo 参数里的包 {'、'.join(parsed.packages)} 有不在工作区里的"
        if not targets:
            return None, "cargo 参数选不中一个编译目标（lib、bin、tests/）"
        hits = []
        for target in targets:
            for full_name, unit in self.test_units(target, formal):
                if full_name == expected or full_name.endswith("::" + expected):
                    hits.append(TestUnit(unit, target.id, full_name))
        if len(hits) != 1:
            names = "、".join(sorted({hit.full_name for hit in hits}))
            return None, (f"点名的测试 {expected} 在选中的目标里对上 {len(hits)} 个 #[test] 单元" + (f"（{names}）" if names else "（测试由宏展开出来、或名字写错）"))
        return hits[0], None

    def features_of(self, arguments, target_id):
        """cargo 参数里的特性 → lib_function_units.py 要的「包名/特性」写法。"""
        crate = self.workspace.targets[target_id].crate
        features = []
        for item in arguments.features:
            features.append(item if "/" in item else f"{crate.package}/{item}")
        if arguments.all_features:
            features += [f"{crate.package}/{name}" for name in crate.features]
        if len(arguments.packages) > 1 and not break_is("multiple-packages-last-only"):
            features += self.features_unified_from_the_other_packages(arguments, crate)
        return tuple(sorted(set(features)))

    def features_unified_from_the_other_packages(self, arguments, entry_crate):
        """同一趟 cargo 编的别的 -p 包（不含入口 crate 自己）往依赖图上共有的包请求的特性，写成「包名/特性」（文件头「多个 -p」）：
        cargo 把一趟里每个包对依赖开的特性并起来，共有的包按并集编。那几个包各自的缺省特性（没给 --no-default-features 时）、命令行给的特性（裸写的只算它们自己有的）、
        --all-features 全开，加上依赖边上请求的特性与缺省特性：那几个包自己顺普通与 dev 两段依赖（按测试编），它们依赖的工作区包只顺普通依赖，一路顺下去。
        别的 -p 包不是入口所在的包，不进 rust_closure_of 的链接集；这些请求只在共有的依赖包上起作用（enable 只加特性、顺特性里的 dep/子特性往下开）。"""
        workspace, requests = self.workspace, []
        for name in arguments.packages:
            other = workspace.crate_by_package(name)
            if other is entry_crate:
                continue
            if not arguments.no_default_features and "default" in other.features:
                requests.append(f"{other.package}/default")
            for item in arguments.features:
                if "/" not in item and item in other.features:
                    requests.append(f"{other.package}/{item}")
            if arguments.all_features:
                requests += [f"{other.package}/{feature}" for feature in other.features]
            pending, seen = [(other, ("normal", "dev"))], set()
            while pending:
                crate, sections = pending.pop()
                if crate.directory in seen:
                    continue
                seen.add(crate.directory)
                for section in sections:
                    for dependency in crate.dependencies[section]:
                        dependency_crate = workspace.crate_by_directory.get(dependency.directory)
                        if dependency_crate is None:
                            continue
                        if dependency.default_features and "default" in dependency_crate.features:
                            requests.append(f"{dependency_crate.package}/default")
                        requests += [f"{dependency_crate.package}/{feature}" for feature in dependency.features]
                        pending.append((dependency_crate, ("normal",)))
        return requests

    def cache_key(self, unit, target_id, parsed):
        return id(unit), target_id, self.features_of(parsed, target_id), parsed.no_default_features, parsed.release

    def precompute_closures(self, requests, workers, labels=None):
        """requests：[(单元, 目标, cargo 参数)]。按（单元、目标、特性、--no-default-features、--release）去重，workers 个 fork 出来的子进程并行算
        （子进程拿的是父进程那一份已经切好的工作区），结果进闭包缓存，成员存成 CompactMembers。workers ≤ 1 或不到 PARALLEL_MINIMUM 格时就地算。
        回算了几个闭包。次序与进程数无关：每个闭包各算各的，键只由它自己的成员定。
        闭包算不成（lib_function_units.py 的 UsageError）的不停在头一处：就地算与并行算都把别的闭包算完，最后抛一个 ClosureRefused，
        逐处带类名、原消息与点名它的 labels（与 requests 一一对应的标签，例变异表的行号与名字；不给就没有）。其余类的异常（算法自己的错）
        在并行算时连回溯一起抛 RuntimeError，不当作「算不成」。"""
        pending, seen = [], {}
        for position, (unit, target_id, arguments) in enumerate(requests):
            parsed = arguments if isinstance(arguments, CargoArguments) else parse_cargo_arguments(arguments or "")
            cache_key = self.cache_key(unit, target_id, parsed)
            if cache_key in self._closures:
                continue
            if cache_key in seen:
                seen[cache_key].append(labels[position] if labels else "")
                continue
            seen[cache_key] = [labels[position] if labels else ""]
            pending.append((unit, target_id, parsed, cache_key))
        failures = []   # [(类名, 消息, 点名它的标签)]，按 pending 的次序

        def refused(position, class_name, message):
            failures.append((class_name, message, [label for label in seen[pending[position][3]] if label]))

        if workers <= 1 or len(pending) < PARALLEL_MINIMUM:
            for position, (unit, target_id, parsed, _cache_key) in enumerate(pending):
                try:
                    self.closure(unit, target_id, parsed)
                except function_units().UsageError as error:
                    refused(position, type(error).__qualname__, str(error))
            self.raise_closure_refused(failures)
            return len(pending)
        self.lock_packages()   # fork 之前读一次，子进程共用
        _PARALLEL.clear()
        _PARALLEL.update(keys=self, units=[item[0] for item in pending])
        tasks = [(index, index, item[1], item[2]) for index, item in enumerate(pending)]
        unexpected = []
        try:
            with multiprocessing.get_context("fork").Pool(processes=min(workers, len(pending))) as pool:
                for index, key, dependencies, refusals, identifiers, fresh, failed in pool.imap_unordered(_parallel_closure, tasks, chunksize=4):
                    if failed is not None:
                        class_name, message, detail = failed
                        if detail:
                            unexpected.append(f"{class_name}：{message}\n{detail}")
                        else:
                            refused(index, class_name, message)
                        continue
                    self.unit_table.update(fresh)
                    self._closures[pending[index][3]] = ClosureInfo(key, CompactMembers(identifiers, self.unit_table), dependencies, refusals)
        finally:
            _PARALLEL.clear()
        if unexpected:
            raise RuntimeError(f"并行算闭包时工作进程里出了算法自己的错（{len(unexpected)} 处）：\n" + "\n".join(unexpected))
        self.raise_closure_refused(failures)
        return len(pending)

    @staticmethod
    def raise_closure_refused(failures):
        """failures 非空就抛 ClosureRefused：同一类名同一消息的并成一条（标签并起来），次序按头一次出现。"""
        if not failures:
            return
        merged = collections.OrderedDict()
        for class_name, message, labels in failures:
            merged.setdefault((class_name, message), []).extend(label for label in labels if label not in merged.get((class_name, message), []))
        raise ClosureRefused([(class_name, message, labels) for (class_name, message), labels in merged.items()])

    def closure(self, unit, target_id, arguments=None):
        """ClosureInfo：从 unit 收的闭包的键（lib_function_units.py 的 closure_key，不带工具链与依赖）、成员、依赖版本行（文件头「依赖版本行」）、拒算。
        依赖版本行有拒算的，键也是 None。"""
        lfu = function_units()
        parsed = arguments if isinstance(arguments, CargoArguments) else parse_cargo_arguments(arguments or "")
        features = self.features_of(parsed, target_id)
        cache_key = self.cache_key(unit, target_id, parsed)
        if cache_key not in self._closures:
            cfg = [item for item in lfu.DEFAULT_RUST_CFG if not (parsed.release and item == "debug_assertions")]
            workspace = self.formal_workspace if id(unit) in self._formal_unit_ids else self.workspace
            closure = lfu.rust_closure_of(workspace, unit, target_id, cfg, True, features, parsed.no_default_features)
            key, refusals = lfu.closure_key(closure)
            dependencies, dependency_refusals = self.dependencies_of(closure.order, workspace)
            self._closures[cache_key] = ClosureInfo(None if dependency_refusals else key, [reference_of(member) for member in closure.order], dependencies,
                                                    [f"{refusal.form} {refusal.file}:{refusal.line}" for refusal in refusals] + dependency_refusals)
        return self._closures[cache_key]

    def current_fingerprints(self, references):
        """一组 UnitReference（种类、文件、限定名）此刻各自的指纹：{(种类, 文件, 限定名): {指纹…}}；文件不在或切不开的那几个交空集合。"""
        current = {}
        for reference in references:
            identity = (reference.kind, reference.file, reference.name)
            if identity in current:
                continue
            if reference.kind == "data-directory":
                directory = os.path.join(self.root, reference.file)
                current[identity] = {function_units().directory_fingerprint(self.root, reference.file)[0]} if os.path.isdir(directory) else set()
                continue
            if not os.path.isfile(os.path.join(self.root, reference.file)):
                current[identity] = set()
                continue
            if reference.kind == "data-file":
                with open(os.path.join(self.root, reference.file), "rb") as handle:
                    current[identity] = {hashlib.sha256(handle.read()).hexdigest()}
                continue
            units = self.rust_file(reference.file).units
            current[identity] = {unit.fingerprint for unit in units if unit.kind == reference.kind and unit.name == reference.name}
        return current

    def executed_units(self, export, work_root):
        """llvm-cov export 的 JSON → 执行过的单元（计数 > 0 的函数，按它第一段代码区域的行落进最里层那个单元：多半是 fn 单元，
        derive 与宏生成的函数落在 struct、enum、宏定义这一类单元上）。回 (UnitReference 列表, 落不进任何单元的函数个数)。
        路径相对 work_root（那一份源码副本）认；落在副本之外的（依赖包、标准库）不算。"""
        executed, unplaced = {}, 0
        root = os.path.normpath(work_root)
        for block in export.get("data", []):
            for function in block.get("functions", []):
                if not function.get("count") or not function.get("regions") or not function.get("filenames"):
                    continue
                full = os.path.normpath(function["filenames"][0])
                if not full.startswith(root + os.sep) or not full.endswith(".rs"):
                    continue
                path = os.path.relpath(full, root).replace(os.sep, "/")
                line = function["regions"][0][0]
                try:
                    units = [unit for unit in self.rust_file(path).units
                             if not unit.data and unit.kind != "doctest" and unit.first_line <= line <= unit.last_line]
                except OSError:
                    units = []
                if not units:
                    unplaced += 1
                    continue
                innermost = min(units, key=lambda unit: unit.last_line - unit.first_line)
                executed[(innermost.file, innermost.name, innermost.first_line)] = reference_of(innermost)
        return sorted(executed.values()), unplaced


# ── 工具链与依赖 ──────────────────────────────────────────────────────────────

def flattened(prefix, value):
    """嵌套的 TOML 表 → [「a.b.c=<JSON>」…]（数组与标量整个取值）。"""
    if isinstance(value, dict):
        lines = []
        for name in sorted(value):
            lines += flattened(f"{prefix}.{name}" if prefix else name, value[name])
        return lines
    return [f"{prefix}={json.dumps(value, sort_keys=True, ensure_ascii=False, default=str)}"]


def cargo_configuration_files(root, environment):
    """cargo 读的配置文件（从根往上每一层的 .cargo/config.toml 或 .cargo/config，再加 CARGO_HOME 那一份），近的在前。"""
    found, directory = [], os.path.abspath(root)
    while True:
        for name in ("config", "config.toml"):
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path):
                found.append(path)
                break
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    cargo_home = os.path.join(os.path.abspath(root), cargo_home)
    for name in ("config", "config.toml"):
        path = os.path.join(cargo_home, name)
        if os.path.isfile(path):
            if os.path.realpath(path) not in {os.path.realpath(item) for item in found}:
                found.append(path)
            break
    return found


def toolchain_lines(root, environment):
    """见文件头「工具链行」。rustc / cargo 起不来抛 MutationUnitsError。"""
    lines = []
    for command in (["rustc", "-vV"], ["cargo", "-V"]):
        try:
            completed = subprocess.run(command, cwd=root, env=environment, capture_output=True, text=True, errors="surrogateescape", stdin=subprocess.DEVNULL)
        except OSError as error:
            raise MutationUnitsError(f"{' '.join(command)} 起不来：{error}") from error
        if completed.returncode != 0:
            raise MutationUnitsError(f"{' '.join(command)} 退 {completed.returncode}")
        lines += [f"{command[0]}\t{line.strip()}" for line in completed.stdout.splitlines() if line.strip()]
    for name in sorted(environment):
        target_variable = name.startswith("CARGO_TARGET_") and name.endswith(("_RUSTFLAGS", "_RUNNER", "_LINKER"))
        if name in BUILD_VARIABLES or name.startswith("CARGO_PROFILE_") or target_variable:
            lines.append(f"environment\t{name}={environment[name]}")
    for rank, path in enumerate(cargo_configuration_files(root, environment)):
        try:
            with open(path, "rb") as handle:
                configuration = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise MutationUnitsError(f"{path} 读不懂：{error}") from error
        for section in CONFIGURATION_SECTIONS:
            value = configuration.get(section)
            if section == "build" and isinstance(value, dict):
                value = {name: item for name, item in value.items() if name not in BUILD_KEYS_NOT_AFFECTING_OUTPUT}
            if value:
                lines += [f"configuration\t{rank}\t{line}" for line in flattened(section, value)]
    manifest_path = os.path.join(root, "Cargo.toml")
    if os.path.isfile(manifest_path):
        try:
            with open(manifest_path, "rb") as handle:
                manifest = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise MutationUnitsError(f"{manifest_path} 读不懂：{error}") from error
        sections = [(section, manifest.get(section)) for section in MANIFEST_SECTIONS]
        sections.append(("workspace.lints", manifest.get("workspace", {}).get("lints")))
        for section, value in sections:
            if value:
                lines += [f"manifest\t{line}" for line in flattened(section, value)]
    return lines


def read_lock_packages(root):
    """根下 Cargo.lock 的全部 [[package]]（出现次序）；没有 Cargo.lock 交空列表（点名了外部 crate 的闭包因此拒算）。
    读不懂、某个包缺 name 或 version 抛 MutationUnitsError。"""
    path = os.path.join(root, "Cargo.lock")
    if not os.path.isfile(path):
        return []
    try:
        with open(path, "rb") as handle:
            packages = tomllib.load(handle).get("package", [])
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise MutationUnitsError(f"{path} 读不懂：{error}") from error
    for position, package in enumerate(packages, 1):
        if not isinstance(package.get("name"), str) or not isinstance(package.get("version"), str):
            raise MutationUnitsError(f"{path} 里从上往下数第 {position} 个 [[package]] 缺 name 或 version")
    return packages


def lock_line(package):
    """一个包进键的那一行：名、版本、来源、checksum（没有的写 -），制表符隔开。"""
    return "\t".join((package["name"], package["version"], package.get("source") or "-", package.get("checksum") or "-"))


def external_dependencies_of(crate):
    """一个工作区 crate 的外部依赖（清单里不带 path 的，[dependencies]、[dev-dependencies]、[build-dependencies] 与 [target.*] 下的同名几节）：
    代码里点它用的名字（依赖的键，- 换成 _）→ Cargo.lock 里的包名（写了 package = 的取它，没写的取键）。"""
    manifest = crate.manifest
    tables = [manifest] + [table for table in (manifest.get("target") or {}).values() if isinstance(table, dict)]
    external = {}
    for table in tables:
        for section in ("dependencies", "dev-dependencies", "dev_dependencies", "build-dependencies", "build_dependencies"):
            for key, specification in (table.get(section) or {}).items():
                if isinstance(specification, dict) and "path" in specification:
                    continue
                package = specification.get("package", key) if isinstance(specification, dict) else key
                external[key.replace("-", "_")] = package
    return external


def path_roots_of(rust_file, unit):
    """一个 Rust 单元里点名的路径根：「<名字> ::」里的名字，前面不是「::」（或前面的 :: 是整条路径开头的那个）；外加 extern crate <名字>。
    交 [(名字, 行)]。注释与串里的不算（按词法单元认）。"""
    if unit.first_token is None or unit.last_token is None:
        return []
    identifier = function_units().IDENT
    kinds = rust_file.kinds[unit.first_token:unit.last_token]
    texts = rust_file.texts[unit.first_token:unit.last_token]
    lines = rust_file.lines[unit.first_token:unit.last_token]

    def text_at(position):
        return texts[position] if 0 <= position < len(texts) else ""

    roots = []
    for position, kind in enumerate(kinds):
        if kind != identifier:
            continue
        followed_by_separator = text_at(position + 1) == ":" and text_at(position + 2) == ":"
        after_separator = position >= 2 and text_at(position - 1) == ":" and text_at(position - 2) == ":"
        separator_starts_path = after_separator and not (position >= 3 and (kinds[position - 3] == identifier or text_at(position - 3) in (">", ")", "]")))
        extern_crate_name = position >= 2 and text_at(position - 1) == "crate" and text_at(position - 2) == "extern"
        if (followed_by_separator and (not after_separator or separator_starts_path)) or extern_crate_name:
            roots.append((texts[position], lines[position]))
    return roots


def resolve_lock_item(packages, item):
    """dependencies 里的一项（「名」「名 版本」「名 版本 (来源)」）落到哪个包：按名找、写了版本的按版本挑，恰好一个才算；否则 None。"""
    parts = item.split()
    if not parts:
        return None
    candidates = [package for package in packages if package["name"] == parts[0] and (len(parts) < 2 or package["version"] == parts[1])]
    return candidates[0] if len(candidates) == 1 else None


def dependency_lines(workspace, units, packages, by_whole_crate=False):
    """见文件头「依赖版本行」：→ (依赖版本行，排好序；拒算，一条一句「lock-dependency-unresolved <文件>:<行>（<包> → <依赖>）」)。
    workspace 是 lib_function_units.py 的 RustWorkspace，units 是闭包里的单元（不是 Rust 的、没有词法单元的跳过），packages 是 read_lock_packages 交的。
    by_whole_crate（弄坏开关「按整包收」）：不看点没点名，单元所在 crate 的外部依赖全收。"""
    refusals, pending = set(), []
    for unit in units:
        if getattr(unit, "language", None) != "rust" or getattr(unit, "data", False):
            continue
        crate = workspace.crate_of_path(unit.file)
        external = external_dependencies_of(crate) if crate is not None else {}
        if not external:
            continue
        own = next((package for package in packages if package["name"] == crate.package and not package.get("source")), None)
        if by_whole_crate:
            named = [(code_name, unit.first_line) for code_name in sorted(external)]
        else:
            named = path_roots_of(workspace.file(unit.file), unit)
        for root, line in named:
            dependency = external.get(root)
            if dependency is None:
                continue
            item = next((item for item in (own or {}).get("dependencies", []) if item.split()[:1] == [dependency]), None)
            package = resolve_lock_item(packages, item) if item is not None else None
            if package is None:
                refusals.add(f"lock-dependency-unresolved {unit.file}:{line}（{crate.package} → {dependency}）")
            else:
                pending.append((package, unit.file, line))
    lines = set()
    while pending:
        package, path, line = pending.pop()
        text = lock_line(package)
        if text in lines:
            continue
        lines.add(text)
        for item in package.get("dependencies", []):
            found = resolve_lock_item(packages, item)
            if found is None:
                refusals.add(f"lock-dependency-unresolved {path}:{line}（{package['name']} → {item}）")
            else:
                pending.append((found, path, line))
    return sorted(lines), sorted(refusals)


# ── 判法单元（python 写的判法：从几个入口函数收的闭包） ────────────────────────

def judging_units(repository, entries, whole_file=False):
    """(键或 None, UnitReference 列表, 拒算列表)：从 entries（lib_function_units.py 的入口写法 <文件>::<限定名>）各收一个闭包、合起来，
    键 = 合起来的单元（种类、指纹）多重集。whole_file（弄坏开关用）：键换成闭包碰到的那几份文件的整份原文。"""
    lfu = function_units()
    members, refusals = {}, set()
    for entry in entries:
        closure = lfu.closure_of(repository, entry)
        for unit in closure.order:
            members[(unit.file, unit.kind, unit.name, unit.first_line)] = unit
        refusals |= {f"{refusal.form} {refusal.file}:{refusal.line}" for refusal in closure.refusals}
    references = sorted((reference_of(unit) for unit in members.values()), key=lambda reference: (reference.file, reference.lines, reference.kind, reference.name))
    if refusals:
        return None, references, sorted(refusals)
    if whole_file:
        texts = []
        for path in sorted({reference.file for reference in references}):
            with open(os.path.join(repository, path), "rb") as handle:
                texts.append(hashlib.sha256(handle.read()).hexdigest())
        return key_of_lines([("form", ["whole-file"]), ("file", texts)]), references, []
    return key_of_lines([("form", ["singlefs-judging-units-v1"]), ("unit", [f"{reference.kind}\t{reference.fingerprint}" for reference in references])]), references, []


# ── 判定库 ────────────────────────────────────────────────────────────────────

def write_atomically(path, data):
    """同目录排他建临时文件、写完 fsync、改名换上（不在同一个 inode 上就地改）。"""
    directory = os.path.dirname(path)
    os.makedirs(directory, exist_ok=True)
    handle, temporary = tempfile.mkstemp(prefix=".partial-", dir=directory)
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(data.encode("utf-8", "surrogateescape") if isinstance(data, str) else data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except BaseException:
        with contextlib.suppress(OSError):
            os.unlink(temporary)
        raise


def encode_value(value):
    return str(value).replace("\\", "\\\\").replace("\n", "\\n")


def decode_value(value):
    out, index = [], 0
    while index < len(value):
        if value[index] == "\\" and index + 1 < len(value):
            out.append("\n" if value[index + 1] == "n" else value[index + 1])
            index += 2
        else:
            out.append(value[index])
            index += 1
    return "".join(out)


def record_text(header, fields):
    return header + "".join(f"{name}={encode_value(value)}\n" for name, value in fields.items())


def parse_record(text):
    fields = {}
    for line in text.split("\n"):
        if not line or line.startswith("#") or "=" not in line:
            continue
        name, value = line.split("=", 1)
        fields[name] = decode_value(value)
    return fields


class Library:
    """按条的判定库（文件头「判定库」）：读写都经 prune-server，directory 只当库的名字与命名空间，本地不建。"""

    def __init__(self, directory, header="# 变异判定库的一条记录（scripts/mutation/lib_mutation_units.py 的 Library 写）\n"):
        self.directory = directory
        self.header = header
        self._units = {}
        self._new_units = collections.defaultdict(dict)
        self.service = self.service_store(directory)

    @staticmethod
    def client():
        """scripts/prune-server/prune_server_client.py（判定库服务的客户端），照它的名字只载入一次；载入不连服务。"""
        name = "singlefs_prune_server_client"
        if name not in sys.modules:
            specification = importlib.util.spec_from_file_location(name, os.path.join(LIBRARY_DIRECTORY, "..", "prune-server", "prune_server_client.py"))
            module = importlib.util.module_from_spec(specification)
            sys.modules[name] = module
            specification.loader.exec_module(module)
        return sys.modules[name]

    @staticmethod
    def service_store(directory):
        """这份库在 prune-server 里的 LibraryStore（文件头「判定库」）；只有客户端的弄坏开关 unreachable-treated-as-no-library 下交 None。"""
        client = Library.client()
        return client.library_store(client.FAMILY_MUTATION_UNITS, os.path.basename(os.path.normpath(directory)), directory)

    def path(self, kind, name):
        """这一条在 prune-server 里的位置（报错与给人看的那一行用；不是本地路径）。"""
        return f"{self.service.location()}{kind}/{name}"

    def read_text(self, kind, name):
        """一份记录的原文（不解）；不在交 None，服务不可用客户端退 75。mutate.sh 的复用哨兵按它自己的读法解。"""
        data = self.service.read(f"{kind}/{name}")
        return None if data is None else data.decode("utf-8", "surrogateescape")

    def read(self, kind, name):
        text = self.read_text(kind, name)
        return None if text is None else parse_record(text)

    def write(self, kind, name, fields):
        self.service.write(f"{kind}/{name}", record_text(self.header, fields))

    def stored_names(self, kind):
        """这一种记录在库里的名字（按名字排好）：自证数「写进了几条」用。"""
        return [relative[len(kind) + 1:] for relative, _value in self.service.scan(f"{kind}/")]

    def read_json(self, kind, name):
        data = self.service.read(f"{kind}/{name}.json.gz")
        if data is None:
            return None
        try:
            return json.loads(gzip.decompress(data).decode("utf-8", "surrogateescape"))
        except (OSError, ValueError, EOFError, zlib.error):
            return None

    def write_json(self, kind, name, value):
        self.service.write(f"{kind}/{name}.json.gz", gzip.compress(json.dumps(value, sort_keys=True, ensure_ascii=False).encode("utf-8", "surrogateescape"), mtime=0))

    def put_composition(self, key, references=None, lines=None):
        """一份组成清单按它的键存（已有就不写）：references 是单元（记进 units/），lines 是工具链行、依赖版本行这一类。"""
        if self.service.read(f"compositions/{key}.json.gz") is not None:
            return
        value = {}
        if isinstance(references, CompactMembers):
            identifiers = references.identifiers()
            for identifier in identifiers:
                self._new_units[identifier[:2]][identifier] = list(references.table[identifier])
            value["units"] = sorted(identifiers)
        elif references is not None:
            identifiers = []
            for reference in references:
                identifier = unit_id(reference)
                identifiers.append(identifier)
                self._new_units[identifier[:2]][identifier] = list(reference)
            value["units"] = sorted(identifiers)
        if lines is not None:
            value["lines"] = sorted(lines)
        self.write_json("compositions", key, value)

    def flush_units(self):
        """这一趟新见到的单元写进 units/<单元号>（每个单元一条，内容只由单元号定，几个进程同时写不丢）。"""
        for bucket, additions in sorted(self._new_units.items()):
            self.merge_units(bucket, additions)
        self._new_units.clear()

    def merge_units(self, bucket, additions):
        self._units.pop(bucket, None)
        self.service.write_many([(f"units/{identifier}", json.dumps(value, ensure_ascii=False)) for identifier, value in sorted(additions.items())])

    def unit(self, identifier):
        bucket = identifier[:2]
        if bucket not in self._units:
            self._units[bucket] = {relative[len("units/"):]: json.loads(data.decode("utf-8", "surrogateescape"))
                                   for relative, data in self.service.scan(f"units/{bucket}")}
        value = self._units[bucket].get(identifier) or self._new_units.get(bucket, {}).get(identifier)
        return UnitReference(*value) if value else None

    def composition(self, key):
        return self.read_json("compositions", key) if key else None


def changed_units(library, old_key, new_references):
    """一份旧组成清单（old_key）与此刻的单元比：[「改了 / 新进 / 出了 <文件>::<限定名>」…]；旧清单读不到交 None。"""
    old = library.composition(old_key)
    if old is None or "units" not in old:
        return None
    old_by_identity, new_by_identity = collections.defaultdict(set), collections.defaultdict(set)
    for identifier in old["units"]:
        reference = library.unit(identifier)
        if reference is not None:
            old_by_identity[(reference.kind, reference.file, reference.name)].add(reference.fingerprint)
    for reference in new_references:
        new_by_identity[(reference.kind, reference.file, reference.name)].add(reference.fingerprint)
    changes = []
    for identity in sorted(set(old_by_identity) | set(new_by_identity)):
        kind, path, name = identity
        if identity not in old_by_identity:
            changes.append(f"新进 {path}::{name}（{kind}）")
        elif identity not in new_by_identity:
            changes.append(f"出了 {path}::{name}（{kind}）")
        elif old_by_identity[identity] != new_by_identity[identity]:
            changes.append(f"改了 {path}::{name}（{kind}）")
    return changes


def changed_lines(library, old_key, new_lines):
    old = library.composition(old_key)
    if old is None or "lines" not in old:
        return None
    old_lines, current = set(old["lines"]), set(new_lines)
    return [f"没了「{line}」" for line in sorted(old_lines - current)] + [f"多了「{line}」" for line in sorted(current - old_lines)]


def summarize_changes(changes, limit=4):
    if changes is None:
        return "上一次的组成清单读不到"
    if not changes:
        return "组成清单逐个相同"
    head = "、".join(changes[:limit])
    return head + (f" 等 {len(changes)} 处" if len(changes) > limit else "")


# ── 实验变异表（mutate.sh）：按条入库 ───────────────────────────────────────────

# v1 丢过 --skip 且可能取错 Cargo 配置；旧记录即使表面键相同也不能沿用。
MUTATE_KEY_FORM = "singlefs-mutate-row-v2"
MUTATE_REUSABLE = ("caught", "notred", "invalid")


def read_mutate_table(path):
    """mutate.sh 的变异表：一行三段（变异名、原文、替换文），空行与 # 开头的跳过，原文 / 替换文里的 \\n 是换行。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            while len(fields) < 3:
                fields.append("")
            rows.append((fields[0], fields[1].replace("\\n", "\n"), fields[2].replace("\\n", "\n")))
    return rows


def mutate_plan(workspace_root, source, binary, table, cargo_words, environment, library_directory, plan_path, break_switches=()):
    """mutate.sh 开跑前调：逐条算键、查库，写 plan_path（JSON：每条的键与组成、测试的闭包键），回 [(下标, 变异名, 复用不复用, 档, 要不要判失败, 那一行, 现判的原因)]。
    键（文件头「键」）：原文、替换文、包住原文那几个单元的闭包键（scope）、cargo 参数与特性（switches，bin 名不进）、工具链行、依赖版本行。
    复用另要：抓到的，红了的那几条测试此刻的闭包键与记录里的相同；没红、无效的，这个 bin 全部测试的闭包键合起来与记录里的相同。
    依赖版本行每条各算：包住原文那几个单元的闭包里点名的外部 crate（文件头「依赖版本行」），落不到包的那一条拒算。break_switches 是 mutate.sh 的弄坏开关：
      libraryunread   不读库（一律现判）；calleesignored  scope 只取包住原文那几个单元自己的指纹、不收它们调到的单元（依赖版本行也只看它们自己）；
      dependenciesbycrate  依赖版本行按整包收（单元所在 crate 的外部依赖不看点没点名全收）。"""
    keys = UnitKeys(workspace_root)
    keys.dependencies_by_whole_crate = "dependenciesbycrate" in break_switches
    library = Library(library_directory, "# 实验变异表的一条判定（scripts/mutation/mutate.sh 经 scripts/mutation/lib_mutation_units.py 写）\n")
    toolchain = toolchain_lines(workspace_root, environment)
    parsed = parse_cargo_arguments(cargo_words)
    target_id = None
    for target in keys.workspace.targets.values():
        if target.kind == "bin" and target.name == binary and source in target.files:
            target_id = target.id
    tests, all_tests_key, test_refusal = {}, None, None
    if target_id is not None:
        test_keys = []
        for full_name, unit in keys.test_units(keys.workspace.targets[target_id]):
            info = keys.closure(unit, target_id, parsed)
            if info.key is None:
                test_refusal = f"测试 {full_name} 的闭包拒算（{info.refusals[0]}）"
                continue
            tests[full_name] = info.key
            test_keys.append(info.key)
        all_tests_key = key_of_lines([("form", ["all-tests"]), ("test", test_keys)]) if test_refusal is None else None
    rows = read_mutate_table(table)
    plan = {"rows": [], "tests": tests, "all_tests": all_tests_key, "library": library_directory,
            "toolchain": None, "how": f"mutate.sh（{' '.join(cargo_words)}）"}
    toolchain_key = key_of_lines([("toolchain", toolchain)])
    plan["toolchain"] = toolchain_key
    decisions = []
    for name, original, replacement in rows:
        entry = {"name": name, "key": None, "refusal": None}
        units, refusal = keys.mutated_units(source, original)
        if refusal is None and target_id is None:
            refusal = f"{source} 不在 bin {binary} 的模块树里"
        scope_keys, scope_members, dependencies = [], [], set()
        if refusal is None:
            for reference in units:
                unit = keys.unit_of(reference)
                if "calleesignored" in break_switches:
                    scope_keys.append(f"{reference.kind}\t{reference.fingerprint}")
                    own_lines, own_refusals = keys.dependencies_of([unit])
                    if own_refusals:
                        refusal = f"包住原文的 {reference.name} 的依赖版本行拒算（{own_refusals[0]}）"
                        break
                    dependencies |= set(own_lines)
                    continue
                info = keys.closure(unit, target_id, parsed)
                if info.key is None:
                    refusal = f"包住原文的 {reference.name} 的闭包拒算（{info.refusals[0]}）"
                    break
                scope_keys.append(info.key)
                scope_members += info.members
                dependencies |= set(info.dependencies)
        entry["refusal"] = refusal
        if refusal is None:
            scope_key = key_of_lines([("scope", scope_keys)])
            dependency_key = key_of_lines([("dependency", sorted(dependencies))])
            entry["scope"], entry["dependency"] = scope_key, dependency_key
            entry["key"] = key_of_lines([("form", [MUTATE_KEY_FORM]), ("original", [escape_field(original)]),
                                         ("replacement", [escape_field(replacement)]), ("scope", [scope_key]), ("switches", parsed.switches),
                                         ("features", list(parsed.features)), ("toolchain", [toolchain_key]), ("dependency", [dependency_key])])
            library.put_composition(scope_key, scope_members)
            library.put_composition(dependency_key, lines=sorted(dependencies))
        plan["rows"].append(entry)
        decisions.append(mutate_decision(library, plan, entry, break_switches))
    library.put_composition(toolchain_key, lines=toolchain)
    library.flush_units()
    plan["test_refusal"] = test_refusal
    write_atomically(plan_path, json.dumps(plan, ensure_ascii=False, sort_keys=True))
    return [(index, plan["rows"][index]["name"]) + decision for index, decision in enumerate(decisions)]


def mutate_decision(library, plan, entry, break_switches):
    """(复用不复用, 档, 要不要判失败, 那一行, 现判的原因)。"""
    if entry["key"] is None:
        return False, "", "0", "", f"拒算：{entry['refusal']}"
    if "libraryunread" in break_switches:
        return False, "", "0", "", "弄坏开关 libraryunread：不读库"
    record = library.read("records", entry["key"])
    if record is None:
        return False, "", "0", "", "库里没有这把键（新加的行，或原文、替换文、包住原文的单元与它调到的单元、cargo 参数、工具链、依赖有一样变了）"
    verdict = record.get("verdict", "")
    if verdict not in MUTATE_REUSABLE:
        return False, "", "0", "", f"记录里的判定「{verdict}」不复用"
    if verdict == "caught":
        red = [name for name in record.get("red_tests", "").split(",") if name]
        recorded = dict(item.split("=", 1) for item in record.get("red_test_keys", "").split(",") if "=" in item)
        if not red or any(plan["tests"].get(name) is None or plan["tests"].get(name) != recorded.get(name) for name in red):
            return False, "", "0", "", "红了的那几条测试此刻对不上记录里的闭包键（改了、没了，或这一趟算不出）"
    else:
        if plan["all_tests"] is None or plan["all_tests"] != record.get("all_tests"):
            return False, "", "0", "", "这个 bin 的测试此刻合起来的闭包键与记录里的不同"
    return True, verdict, record.get("fails", "0"), record.get("line", ""), ""


MULTIPLE_PACKAGES_FIXTURE = {
    "crates/fshared/Cargo.toml": '[package]\nname = "fshared"\n\n[features]\ntuned = []\n',
    "crates/fshared/src/lib.rs": '''#[cfg(feature = "tuned")]
pub fn mode() -> u32 {
    2
}
#[cfg(not(feature = "tuned"))]
pub fn mode() -> u32 {
    1
}
''',
    "crates/fleft/Cargo.toml": '[package]\nname = "fleft"\n\n[dependencies]\nfshared = { path = "../fshared" }\n',
    "crates/fleft/src/lib.rs": '''pub fn left() -> u32 {
    fshared::mode()
}
#[cfg(test)]
mod tests {
    #[test]
    fn the_left_mode_is_read_from_the_shared_package() {
        assert!(super::left() > 0);
    }
}
''',
    "crates/fright/Cargo.toml": '[package]\nname = "fright"\n\n[dependencies]\nfshared = { path = "../fshared", features = ["tuned"] }\n',
    "crates/fright/src/lib.rs": '''pub fn right() -> u32 {
    fshared::mode()
}
#[cfg(test)]
mod tests {
    #[test]
    fn the_right_mode_is_read_from_the_shared_package() {
        assert!(super::right() > 0);
    }
}
''',
}
MULTIPLE_PACKAGES_CELL = ("多个 -p：点名的测试在前一个包里照样找得到；共有的依赖包按几个包请求的特性并集编（键随并集变、与 -p 的次序无关）；"
                          "改被点名的包没调到的函数键不变，改共有依赖包里并集特性下那一份的函数体键变")


def multiple_packages_cell(work):
    """自证一格（crates-mutation-rows.py 的 --selftest 登记，弄坏开关 multiple-packages-last-only）：两个 -p（fleft、fright），点名的测试在 fleft；
    fright 对共有的 fshared 开了 tuned。→ (过没过, 说明)。"""
    root = os.path.join(work, "multiple-packages")
    for path, text in MULTIPLE_PACKAGES_FIXTURE.items():
        os.makedirs(os.path.dirname(os.path.join(root, path)), exist_ok=True)
        with open(os.path.join(root, path), "w", encoding="utf-8") as handle:
            handle.write(text)
    test_name = "the_left_mode_is_read_from_the_shared_package"

    def key_of(arguments):
        keys = UnitKeys(root)   # 工作区与闭包按入口缓存：改了源码之后另起一份
        parsed = parse_cargo_arguments(arguments)
        hit, why = keys.resolve_test(parsed, test_name)
        if hit is None:
            return None, why
        info = keys.closure(hit.unit, hit.target, parsed)
        return info.key, "; ".join(info.refusals)

    single, single_note = key_of("-p fleft --lib")
    both, both_note = key_of("-p fleft -p fright --lib")
    swapped, swapped_note = key_of("-p fright -p fleft --lib")
    parsed = parse_cargo_arguments("-p fleft -p fright --lib")
    parsed_single = parse_cargo_arguments("-p fleft --lib")
    keys = UnitKeys(root)
    unified = keys.features_of(parsed, keys.resolve_test(parsed, test_name)[0].target) if both is not None else None
    with open(os.path.join(root, "crates/fright/src/lib.rs"), "a", encoding="utf-8") as handle:
        handle.write("pub fn unrelated() -> u32 {\n    7\n}\n")
    outside, _note = key_of("-p fleft -p fright --lib")
    shared = os.path.join(root, "crates/fshared/src/lib.rs")
    with open(shared, encoding="utf-8") as handle:
        text = handle.read()
    with open(shared + ".new", "w", encoding="utf-8") as handle:
        handle.write(text.replace("    2\n}", "    3\n}"))
    os.replace(shared + ".new", shared)
    tuned, _note = key_of("-p fleft -p fright --lib")
    ok = (parsed.packages == ("fleft", "fright") and parsed_single.packages == ("fleft",) and single is not None and both is not None and swapped is not None
          and both != single and swapped == both and outside == both and tuned is not None and tuned != both and unified == ("fshared/tuned",))
    return ok, (f"包 {parsed.packages}；单个 -p 的键 {single}（{single_note}）、两个 -p 的 {both}（{both_note}）、换序的 {swapped}（{swapped_note}）"
                f"（要后两个相同、与单个的不同）；并集特性 {unified}（要 ('fshared/tuned',)）；改 fright 里没调到的函数后 {outside}（要同前）；改 tuned 那一份的函数体后 {tuned}（要不同）")


SPECIFICATION_LINE_FILE = "crates/fspec/src/lib.rs"
SPECIFICATION_LINE_SOURCE = """verus! {
pub open spec fn limit() -> int {
    5 + 0
}
pub fn doubled(x: u64) -> u64 {
    x + x
}
} // verus!
#[cfg(test)]
mod tests {
    #[test]
    fn doubled_is_even() {
        assert_eq!(super::doubled(3) % 2, 0);
    }
}
"""
SPECIFICATION_LINE_GHOST_PIECE = "pub open spec fn limit() -> int {\n    5 + 0\n}"   # 产品行为视图剥掉的那一截（spec fn 整条）
SPECIFICATION_LINE_CELL = ("verus! 里的规格行：原文落在产品行为视图剥掉的字上，被改坏的单元与点名测试的闭包取形式化视图、算得出键，改那条规格键变；"
                           "原文在执行代码上的行照旧取产品行为视图，改规格键不变")


def specification_line_cell(work):
    """自证一格（crates-mutation-rows.py 的 --selftest 登记，弄坏开关 specification-lines-product-view）：verus! 里一条 spec fn 与一个执行函数、
    一个点名的测试；产品行为视图用的 unit-graph 由假程序替身（剥过的那一份照原样切出的 dump，照 lib_function_units.py 的 product_view_program_cell 的做法）。
    → (过没过, 说明)。"""
    lfu = function_units()
    root = os.path.join(work, "specification-lines")
    arguments, expected = "-p fspec --lib -- doubled_is_even", "doubled_is_even"

    def key_of(original, edit_specification):
        """(这一行的键或 None, 取的视图或拒算那一句)；键 = 被改坏的单元的指纹加点名测试的闭包键。"""
        source, piece = SPECIFICATION_LINE_SOURCE, SPECIFICATION_LINE_GHOST_PIECE
        if edit_specification:
            source, piece = source.replace("5 + 0", "5 + 0 + 0"), piece.replace("5 + 0", "5 + 0 + 0")
        blanked = source.replace(piece, "".join(character if character == "\n" else " " for character in piece))
        manifest = {"crates/fspec/Cargo.toml": '[package]\nname = "fspec"\n'}
        shutil.rmtree(root, ignore_errors=True)
        lfu.write_tree(root, dict(manifest, **{SPECIFICATION_LINE_FILE: source}))
        dump = lfu.product_view_dump_of(work, dict(manifest, **{SPECIFICATION_LINE_FILE: blanked}))
        program_directory = os.path.join(work, "specification-lines-bin")
        shutil.rmtree(program_directory, ignore_errors=True)
        os.makedirs(program_directory)
        dump_file = os.path.join(work, "specification-lines-dump.txt")
        with open(dump_file, "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in dump))
        program = os.path.join(program_directory, "unit-graph")
        with open(program, "w", encoding="utf-8") as handle:
            handle.write(f'#!/bin/bash\n[[ "$1 $3 $4" == "dump --judgment-kind {lfu.PRODUCT_BEHAVIOR}" && $# == 4 ]] || exit 2\ncat {shlex.quote(dump_file)}\n')
        os.chmod(program, 0o755)

        def compute():
            keys = UnitKeys(root)
            references, why = keys.mutated_units(SPECIFICATION_LINE_FILE, original)
            if references is None:
                return None, why
            view = "形式化视图" if keys._last_mutation_formal else "产品行为视图"
            hit, why = keys.resolve_test(arguments, expected)
            if hit is None:
                return None, why
            info = keys.closure(hit.unit, hit.target, parse_cargo_arguments(arguments))
            if info.key is None:
                return None, "闭包拒算 " + "; ".join(info.refusals)
            return key_of_lines([("mutated", [f"{reference.kind}\t{reference.fingerprint}" for reference in references]), ("test", [info.key])]), view
        return lfu.with_unit_graph_directory(program_directory, compute)

    specification, specification_note = key_of("5 + 0", False)
    specification_edited, _note = key_of("5 + 0", True)
    executable, executable_note = key_of("x + x", False)
    executable_after_specification_edit, _note = key_of("x + x", True)
    ok = (specification is not None and specification_note == "形式化视图" and specification_edited is not None and specification_edited != specification
          and executable is not None and executable_note == "产品行为视图" and executable_after_specification_edit == executable)
    return ok, (f"规格行的键 {specification}（{specification_note}，要取形式化视图、有键）、改了那条规格之后 {specification_edited}（要有键、与前者不同）；"
                f"执行代码行的键 {executable}（{executable_note}，要取产品行为视图）、改了那条规格之后 {executable_after_specification_edit}（要与前者相同）")


def mutate_record(plan_path, index, verdict, fails, line, red_names=""):
    """mutate.sh 判完一条（现判的）调：档在可复用的那几种里、这一条有键，就写一条记录；回写了没有。"""
    with open(plan_path, encoding="utf-8") as handle:
        plan = json.load(handle)
    entry = plan["rows"][int(index)]
    if entry["key"] is None or verdict not in MUTATE_REUSABLE:
        return False
    red = [name for name in red_names.replace(",", " ").split() if name]
    fields = collections.OrderedDict(kind="mutate", key=entry["key"], verdict=verdict, fails=fails, line=line, judged_date=today_text(), how=plan["how"],
                                     scope=entry.get("scope", ""), toolchain=plan["toolchain"], dependency=entry.get("dependency", ""),
                                     all_tests=plan.get("all_tests") or "", red_tests=",".join(red),
                                     red_test_keys=",".join(f"{name}={plan['tests'].get(name, '')}" for name in red))
    Library(plan["library"], "# 实验变异表的一条判定（scripts/mutation/mutate.sh 经 scripts/mutation/lib_mutation_units.py 写）\n").write("records", entry["key"], fields)
    return True
