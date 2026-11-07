"""判定库：每条判定按它的单元键存，下一趟键相同就复用，不再起重活。
判法照 .claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」；口径照 .claude/kb/decisions/13-验证路线.md 已定项 17
（判定复用的单元是函数，键只由函数单元、工具链、依赖版本与这条判定自己的内容输入组成）。

Python 一侧这一个概念只叫「判定库」（类名 VerdictLibrary）。Rust 一侧 crates/singlefs-verification 的判定存储（verdict_store 模块）是另一份存法，不在这里。
用它的：
  门禁 formal-verification 的两份判法（scripts/gate/verus-proof-check.py、scripts/experiment-apparatus/e165-pin-red-examples.py）、
    门禁 lkmm（.claude/scripts/lkmm.sh 每份 litmus 两条判定）与门禁 qemu-device-streams（每档一条判定）用整套：
    函数单元闭包键（FunctionUnitIndex）、VerdictLibrary 存放与按键读、Tally 现判与复用分开报；两道 shell 写的门禁经下面的命令行用
    （lookup、record、record-red 记红撤绿）。
  库目录取法与下面「存放」相同（被判那棵树的 git common-dir 下那一份固定目录，只当名字）、判定另经别的库类存进 prune-server 的：
    门禁 crates-mutation-replay 的判法（scripts/mutation/crates-mutation-rows.py，调 library_directory）与 scripts/mutation/mutate.sh
    （判定经 scripts/mutation/lib_mutation_units.py 的 Library 存进 prune-server，mutate.sh 判失败的档记 fails=1、不当绿复用），
    与 scripts/experiment-apparatus/replay.sh（族 unit-verdicts、种类 replay-row，经 scripts/gate/admission.py library-files 读写服务，对不上的记 drift、不当绿复用）；
    记红撤绿都是照同一把键盖掉原有的绿，存法各自的文件头写。
  经 scripts/gate/admission.py 自己的 VerdictLibrary 存放（库目录取法与这里相同：它文件头「判定库」）、经它的 verdict-record --red-names 记红撤绿的：
    门禁 crash-point-replay 按条复用的几格、model-differential-and-scenarios 各模型对拍格、research-build-and-replay 的 research-unit-tests 格、
    code-tooling 的 research-script-selftests 格，与入口 scripts/crash/crash-point-replay.sh 按条复用的那一段。
不用它、各自留一份的（gate-overlap:copy-kept 写在它们那里）：广谱变异的结论表（scripts/mutation/broad-mutation.py，族 broad-mutation，只追加的行；
  同一把键先抓到、后判没红的翻面条不沿用、现判一次，见它文件头「判定行」末段）。

共用库给判定算键用函数单元摘要（与 scripts/gate/lib_function_units.py 同一份切法，这里只 import 它，不另切）：
  FunctionUnitIndex(根).key(入口…) 交一把 UnitKey：从入口（<文件>::<限定名>、<文件>，写法同 lib_function_units.py 文件头「入口」）沿调用与引用收的
    函数单元闭包，各单元（种类，去注释指纹）的多重集，加入口的生效 cfg、调用方给的工具链行、依赖版本行与外部程序（名、sha256、--version 那一行），
    照 lib_function_units.closure_key 取 sha256。几个入口的闭包并起来算一把；同一个根上 Rust 的工作区只读一次（几十个 Rust 入口共用）。
    不带路径、名字、行号；组成清单（单元行：种类、限定名、文件:起止、指纹）随键交回，只当索引。
  闭包里有拒算（lib_function_units.py 文件头「拒算」）：UnitKey.key 是 None，带它的那一条判定不复用、每趟现判，lookup 的原因写「拒算」并点名第一处。
  judging_unit_key(根, 入口…)：判法单元指纹，判结局那几个函数的闭包键；这份库与 lib_function_units.py 两份（NOT_JUDGING_FILES）的单元不进它：
    它们管怎么存、怎么算键，不管判对错，改它们不让任何判定作废。
  locked_dependency_lines(根, 包名)：Cargo.lock 里从这个工作区包出发传递引到的外部包，一包一行「名 版本 来源 checksum」（工作区里的 path 包不列：它们的单元已在闭包里）；
    dev-dependencies 与只在别的平台上进依赖图的包照收（lock 里分不出），方向是多作废。
  rust_toolchain_lines(根)：cargo -V 与 rustc -V 各一行，给 Rust 入口的键当工具链行（调用方另加构建配置那一行：--release、--target、rustflags）。

键：
  components：[(名字, 摘要或 UnitKey)]，名字是给人看的「装置单元」「这一行」这类（不是路径），按名字排好，「名字 NUL 摘要 换行」接起来取 sha256 是输入指纹；
  判法单元指纹（judging_units，摘要或 UnitKey）一个判定库一个，建库时给。键 = sha256（「输入指纹 NUL 判法单元指纹」）。
  两列有一样变了就现判；现判的原因点名是哪一列：输入里「<哪几样> 变了」，或「判法单元指纹 变了」；detail(项) 交变了的单元清单
  （上一份判定的组成清单与这一趟的比，按（种类，限定名，文件）对：改了、新进、出去的各列名），门禁照抄进它那一项的红句或现判行。
  记红、撤绿、下趟现判：判绿的调 record、判红的调 record_red，两样都照同一把键原子写 keys/<键>.json，后写的盖掉先写的（判红撤掉这把键原有的绿，
  再判绿盖掉红）；lookup 遇到这把键记着红，答现判、原因是「上次判红」（RED_RECORD_REASON）。没跑成、被信号停了的两样都不记，下一趟照判。
  键相同就复用，不看记下来的是哪一项的名字（输入里要有这一项自己的那一段，不然两项会共用一份判定）。
  一项判完当场写进 prune-server（服务回确认才算数）；被打断之后同一条命令重跑，判过的复用、没判的接着判。

存放（directory 由调用方照 library_directory(被判的根, <这一道的固定目录名>) 取：被判那棵树的 git common-dir 下那一份固定目录的路径；本地不建、不读、不写）：
  键里只用固定目录名当库名，哪棵仓不进键：主仓与仓副本（git init 过的）读写同一份。被判的根不是 git 工作树（取不到 common-dir）时
    library_directory 退 2（LibraryUnavailable，是 SystemExit；标准错误点名那棵树与出路），一项都不判；没给库目录、这棵树里没有客户端
    （拷进不带 scripts/prune-server/ 的小树里跑的这一份）同样退 2。没有「不读不写、全部现判」的那条路。
  每份判定库（<directory>/<kind>）的三样都在 prune-server 结论表里，键 = verdict-library[@命名空间]/<固定目录名>/<kind>/<相对路径>
  （scripts/prune-server/prune_server_client.py 文件头「键空间」，library_service_store；命名空间只在临时服务底下给，自证与样本各给各的临时树，不串）：
  keys/<键>.json      这一份判定：item、key、input_fingerprint、judging_units、components（名字 → 摘要）、compositions（名字 → 组成清单的摘要）、
                      verdict（green 或 red）、summary、details（调用方给的几样数，例 verified 条数）、recorded（判定日期）、run（跑法）
  items/<项名的 sha256>.json   这一项最近一次判绿的那一份（与上面同内容；判红不写它），只用来说清「现判」的原因
  compositions/<组成清单的摘要>.json   组成清单本身（单元行），几条判定共用一份；只给 detail 报变了的单元，不进任何键
  一份判定与它的 items/ 那一份放同一个写帧（原子）。服务连不上客户端退 75（PruneServerFailure，是 SystemExit）：库没有本地文件那条路，
  调用方（门禁、命令行 lookup / record / record-red）照 75 整道退出，不当「无记录」现判。

用法（python）：
    from lib_verdict_library import FunctionUnitIndex, VerdictLibrary, Tally, digest_text, judging_unit_key
    index = FunctionUnitIndex(根)
    units = index.key(["crates/x/src/bin/y.rs::main"], toolchain=[...], dependencies=locked_dependency_lines(根, "x"))
    library = VerdictLibrary(library_directory(根, "<这一道的固定目录名>"), "proof-function", judging_unit_key(仓根, ["scripts/gate/z.py::check"]))
    reused, reason = library.lookup(项名, [("被验函数单元", units), ("这一行", digest_text(...))])
    ... 现判 ... library.record(项名, components, "一句判定", run="跑法", details={...})       # 判绿时调
                 library.record_red(项名, components, "一句判定", run="跑法", details={...})   # 判红时调（撤掉同一把键的绿）
    tally.count(reused, reason); print(tally.text())

命令行（shell 写的门禁用；退出码 0 成、1 现判 / 写不成、2 用法错、3 算不出、4 拒算、75 prune-server 不可用（调用方整道退 75，不当现判））：
  lib_verdict_library.py unit-key <根> <入口>… [--target 目标] [--cfg 谓词]… [--toolchain 行]… [--dependency 行]… [--external 名=sha256=版本行]…
                                    [--exclude-file 相对根的路径]… [--exclude-unit <文件>::<限定名>]…
      打「key<TAB><键>」与组成清单（单元行），有拒算打拒算行、不打键行、退 4。--dependency-of <包名> 照 locked_dependency_lines 补依赖行；
      --programs-keyed-elsewhere 照 judging_unit_key 那样不要求外部程序（判法单元指纹用：外部程序由每条判定自己的那几样管）。
  lib_verdict_library.py judging-units <根> <入口>…        同 unit-key，去掉 NOT_JUDGING_FILES 两份的单元
  lib_verdict_library.py lookup <目录> <种类> <项> <组成表>      <目录> 照「存放」：<被判那棵树的 git common-dir>/<固定目录名>
      组成表一行一样「名字<TAB>摘要」或「名字<TAB>@<unit-key 的输出文件>」，名字写 判法单元指纹 的那一行是判法单元指纹那一列；
      打一行「复用<TAB>原因」退 0，或「现判<TAB>原因<TAB>变了的单元」退 1
  lib_verdict_library.py record <目录> <种类> <项> <组成表> --summary <一句判定> --run <跑法> [--detail 名=值]…   判绿之后调；写不成退 1
  lib_verdict_library.py record-red <目录> <种类> <项> <组成表> --summary <一句判定> --run <跑法> [--detail 名=值]…   判红之后调（同 record_red）；写不成退 1
  lib_verdict_library.py --selftest   只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）；
                                      不在那底下退 2、一格都不跑，不写真服务

弄坏开关 VERDICT_LIBRARY_BREAK=<项>[,<项>…]（只在 --selftest 里认：命令行第一个参数不是 --selftest、又不是自证起的子进程（自证设 VERDICT_LIBRARY_SELFTEST=1）时设了它，
参数解析之前退 2、点名它只在自证里认；自证末尾逐个打开，核点名的那一格转红）。逐项写开关名、弄坏哪一处，下一行起写打开后自证红的格
（「」里是失败行 「✗ lib_verdict_library 自证 N 格里 M 格判错：<格标签、…>」 里的标签）：
  VERDICT_LIBRARY_BREAK=unit-key-from-raw-file  单元键改取入口文件整份原文的 sha256（不切单元、不去注释）
      「只改注释单元键不变」「改没被调到的函数键不变」「作废时点名变了的单元」
  VERDICT_LIBRARY_BREAK=refusal-reused  拒算的键照一个固定的摘要算、照样复用
      「拒算不复用、不记」
  VERDICT_LIBRARY_BREAK=composition-not-recorded  不存组成清单，detail 报不出变了的单元
      「作废时点名变了的单元」「判法单元指纹变了整项现判」「点名判法的单元」
  VERDICT_LIBRARY_BREAK=judging-files-included  判法单元指纹里留着这份库与 lib_function_units.py 的单元
      「判法单元指纹不含库自己」
  VERDICT_LIBRARY_BREAK=red-not-recorded  record_red 什么都不写（只记绿，判红不撤同一把键的绿）
      「判绿入库、同一把键判红之后，下一趟这一项现判、不复用」
  VERDICT_LIBRARY_BREAK=library-directory-outside-git-tolerated  library_directory 取不到 git common-dir 时交 None、不退 2（「不读不写、全部现判」）
      「不在 git 工作树里取库目录退 2」
  VERDICT_LIBRARY_BREAK=missing-client-tolerated  这棵树里没有客户端时 library_service_store 交 None、不退 2（「照没给判定库办」）
      「树里没有客户端的命令行 lookup 退 2」
"""
import collections
import datetime
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import tomllib

NO_RECORD_REASON = "无记录"
BROKEN_RECORD_REASON = "记录读不出"
REFUSED_REASON = "拒算"
RED_RECORD_REASON = "上次判红"
JUDGING_UNITS_COLUMN = "判法单元指纹"
LIBRARY_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(LIBRARY_DIRECTORY))
FUNCTION_UNITS_PATH = os.path.join(REPOSITORY_ROOT, "scripts", "gate", "lib_function_units.py")
# 管怎么存、怎么算键的两份：它们的单元不进任何判法单元指纹（judging_unit_key）
NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")
BREAK = {item for item in os.environ.get("VERDICT_LIBRARY_BREAK", "").split(",") if item}
DETAIL_UNIT_LIMIT = 6
_FUNCTION_UNITS = []


def digest_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape") if isinstance(text, str) else text).hexdigest()


def digest_file(path):
    """文件内容的 sha256；不在写成「没有」的摘要（不在也是一种取值）。"""
    if not os.path.isfile(path):
        return digest_text(f"没有 {os.path.basename(path)}")
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def digest_tree(directory, skipped_names=("target",), skipped_files=()):
    """目录下全部普通文件逐个「sha256  相对路径」按路径字节序接起来再取 sha256；目录不在写成「没有」的摘要。
    只给弄坏开关退回旧判法用（verus-proof-check.py 的 reuse-key-whole-crate）；判定的键不许用它（整目录原文连路径）。"""
    if not os.path.isdir(directory):
        return digest_text(f"没有目录 {os.path.basename(directory)}")
    skipped = {os.path.normpath(path) for path in skipped_files}
    lines = []
    for walked, subdirectories, names in os.walk(directory):
        subdirectories[:] = sorted(name for name in subdirectories if name not in skipped_names)
        for name in names:
            full = os.path.join(walked, name)
            if os.path.isfile(full) and os.path.relpath(full, directory) not in skipped:
                lines.append((os.path.relpath(full, directory).encode("utf-8", "surrogateescape"), digest_file(full)))
    return digest_text(b"".join(digest.encode() + b"  " + relative + b"\n" for relative, digest in sorted(lines)))


# ── 函数单元闭包键 ──

class UnitKey(collections.namedtuple("UnitKey", "key units refusals")):
    """一把单元键：key（sha256，有拒算时 None）、units（组成清单：单元行，排好序）、refusals（拒算行）。"""
    __slots__ = ()

    def first_refusal(self):
        return self.refusals[0] if self.refusals else ""

    def composition_digest(self):
        return digest_text("\n".join(self.units))


def function_units():
    """scripts/gate/lib_function_units.py，第一次用到时才读进来（它切单元、收闭包、算键；这里只调它）。"""
    if not _FUNCTION_UNITS:
        sys.dont_write_bytecode = True
        specification = importlib.util.spec_from_file_location("lib_function_units_for_verdict_library", FUNCTION_UNITS_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _FUNCTION_UNITS.append(module)
    return _FUNCTION_UNITS[0]


def parse_external(text):
    """「名=sha256=--version 那一行」→ (名, (sha256, 版本行))；版本行里可以有 =。"""
    name, _first, rest = text.partition("=")
    digest, _second, version = rest.partition("=")
    if not name or not digest:
        raise ValueError(f"外部程序要写成 名=sha256=版本行，读到 {text!r}")
    return name, (digest, version)


class FunctionUnitIndex:
    """一个根上的单元键：Rust 工作区读一次、几十个入口共用；python 与 shell 每个入口各收一次（它们快）。"""

    def __init__(self, root):
        self.root = os.path.abspath(root)
        self._workspace = None

    def workspace(self):
        if self._workspace is None:
            self._workspace = function_units().RustWorkspace(self.root)
        return self._workspace

    def closure(self, entry, target=None, cfg=(), features=()):
        units = function_units()
        language, path, qualified = units.parse_entry(self.root, entry)
        if language == "rust":
            workspace = self.workspace()
            unit, target_id = workspace.find_entry(path, qualified, target)
            return units.rust_closure_of(workspace, unit, target_id, list(cfg), False, list(features), False)
        options = units.Options()
        options.target = target
        options.cfg = list(cfg)
        return units.closure_of(self.root, entry, options)

    def key(self, entries, target=None, cfg=(), features=(), toolchain=(), dependencies=(), externals=None,
            excluded_files=(), excluded_units=(), programs_keyed_elsewhere=False):
        """几个入口的闭包并起来算一把 UnitKey。excluded_files / excluded_units（<文件>::<限定名>）里的单元不进键（调用方另把它们的取值放进键时用）。
        闭包里点名的外部程序（qemu-system-*、herd7、busybox、cargo）要调用方在 externals 里给（名、sha256、版本行），没给的拒算；
        programs_keyed_elsewhere 只给判法单元指纹用：判法里起的外部程序由每条判定自己的工具链那一样管，这里记一个固定的占位、不拒算。
        入口找不到、根不在抛 ValueError（调用方按算不出办：这一条照判，不复用）。弄坏开关 unit-key-from-raw-file 下键取入口文件整份原文。"""
        units = function_units()
        if "unit-key-from-raw-file" in BREAK:
            pieces = [digest_file(os.path.join(self.root, units.parse_entry(self.root, entry)[1] or "")) for entry in entries]
            return UnitKey(digest_text("\n".join(pieces + list(toolchain) + list(dependencies))), (), ())
        merged = units.Closure()
        seen = set()
        excluded_files, excluded_units = set(excluded_files), set(excluded_units)
        try:
            for entry in entries:
                closure = self.closure(entry, target, cfg, features)
                for unit in closure.order:
                    identity = (unit.language, unit.kind, unit.name, unit.file, unit.first_line, unit.fingerprint)
                    if identity in seen or unit.file in excluded_files or f"{unit.file}::{unit.name}" in excluded_units:
                        continue
                    seen.add(identity)
                    merged.order.append(unit)
                    merged.members[id(unit)] = unit
                merged.refusals.update(refusal for refusal in closure.refusals if refusal.file not in excluded_files)
                merged.cfg_lines = sorted(set(merged.cfg_lines) | set(closure.cfg_lines))
                for program, place in closure.programs.items():
                    if place[0] not in excluded_files:
                        merged.programs.setdefault(program, place)
        except (units.UsageError, units.FunctionUnitsError) as error:
            raise ValueError(str(error)) from error
        externals = dict(externals or {})
        if programs_keyed_elsewhere:
            for program in merged.programs:
                externals.setdefault(program, ("-", "由每条判定自己的工具链那一样管"))
        key, refusals = units.closure_key(merged, list(toolchain), list(dependencies), externals)
        return UnitKey(key, tuple(sorted(unit.text() for unit in merged.order)), tuple(refusal.text() for refusal in refusals))


def judging_unit_key(root, entries, index=None):
    """判法单元指纹：判结局那几个函数（entries）的闭包键，NOT_JUDGING_FILES 两份的单元不进（弄坏开关 judging-files-included 下照进）。"""
    index = index or FunctionUnitIndex(root)
    excluded = () if "judging-files-included" in BREAK else NOT_JUDGING_FILES
    return index.key(entries, excluded_files=excluded, programs_keyed_elsewhere=True)


def locked_dependency_lines(root, package):
    """Cargo.lock 里从工作区包 package 出发传递引到的外部包（有 source 的），一包一行「名 版本 来源 checksum」，排好序。
    lock 里依赖写「名」「名 版本」「名 版本 (来源)」三种；同名几个版本时按版本对。lock 不在或包不在交一行说明（不在也是一种取值）。"""
    path = os.path.join(root, "Cargo.lock")
    if not os.path.isfile(path):
        return ["没有 Cargo.lock"]
    with open(path, "rb") as handle:
        packages = tomllib.load(handle).get("package", [])
    by_name = collections.defaultdict(list)
    for entry in packages:
        by_name[entry.get("name")].append(entry)
    roots = [entry for entry in by_name.get(package, []) if "source" not in entry]
    if not roots:
        return [f"Cargo.lock 里没有工作区包 {package}"]
    lines, seen, todo = set(), set(), list(roots)
    while todo:
        entry = todo.pop()
        identity = (entry.get("name"), entry.get("version"), entry.get("source"))
        if identity in seen:
            continue
        seen.add(identity)
        if "source" in entry:
            lines.add(f"{entry.get('name')} {entry.get('version')} {entry.get('source')} {entry.get('checksum', '-')}")
        for dependency in entry.get("dependencies", []):
            name, _space, rest = dependency.partition(" ")
            version = rest.split(" ", 1)[0] if rest else None
            candidates = by_name.get(name, [])
            matched = [candidate for candidate in candidates if version is None or candidate.get("version") == version]
            todo.extend(matched or candidates)
    return sorted(lines)


def rust_toolchain_lines(root):
    """cargo -V 与 rustc -V 各一行（只问版本，在 root 里起：rust-toolchain 文件按目录定工具链）；起不来写「没有 cargo」这类一行（不在也是一种取值）。"""
    lines = []
    for program in ("cargo", "rustc"):
        try:
            completed = subprocess.run([program, "-V"], cwd=root, capture_output=True, text=True, timeout=60)
            lines.append(f"{program}: {completed.stdout.strip() or completed.stderr.strip()[:200] or '退 ' + str(completed.returncode)}")
        except (OSError, subprocess.TimeoutExpired) as error:
            lines.append(f"{program}: 没有（{error}）")
    return lines


def git_common_directory(tree):
    """tree 所在 git 仓的 common-dir（绝对路径）；不是 git 工作树、git 起不来交 None。"""
    try:
        completed = subprocess.run(["git", "-C", tree, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                   capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return (completed.stdout.strip() or None) if completed.returncode == 0 else None


class LibraryUnavailable(SystemExit):
    """判定库没处放（文件头「存放」）：被判的根不是 git 工作树、没给库目录、这棵树里没有客户端。退 2；抛之前在标准错误打 ✗ 与出路。
    是 SystemExit：调用方的 except Exception 拦不住，进程照 2 退。"""

    def __init__(self, message, remedy):
        print(f"  ✗ 判定库没处放：{message}（退 {EXIT_USAGE}）", file=sys.stderr)
        print(f"     → 怎么办：{remedy}", file=sys.stderr)
        super().__init__(EXIT_USAGE)
        self.message = message

    def __str__(self):
        return self.message


def library_directory(judged_root, name):
    """这一道判定库的库目录（文件头「存放」）：被判那棵树的 git common-dir 下的 <name>，只当名字；取不到 common-dir 退 2（LibraryUnavailable）。
    弄坏开关 library-directory-outside-git-tolerated 下取不到交 None（「不读不写、全部现判」）。"""
    common_directory = git_common_directory(judged_root)
    if common_directory is None:
        if "library-directory-outside-git-tolerated" in BREAK:
            return None
        raise LibraryUnavailable(f"{os.path.abspath(judged_root)} 不是 git 工作树（取不到 git common-dir）",
                                 "在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（判定库按固定库名存，哪棵仓不进键，副本与主仓读写同一份）")
    return os.path.join(common_directory, name)


def component_digest(value):
    return value.key if isinstance(value, UnitKey) else value


def refused_component(components, judging_units):
    """交 (名字, UnitKey) 里第一样拒算的（判法单元指纹拒算也算），没有交 None。弄坏开关 refusal-reused 下一律交 None。"""
    if "refusal-reused" in BREAK:
        return None
    for name, value in list(components) + [(JUDGING_UNITS_COLUMN, judging_units)]:
        if isinstance(value, UnitKey) and value.key is None:
            return name, value
    return None


def stable_digest(value):
    """进键的摘要：UnitKey 取它的键；拒算的（只在弄坏开关 refusal-reused 下走到这里）取一个固定串的摘要。"""
    digest = component_digest(value)
    return digest if digest is not None else digest_text("拒算")


def input_fingerprint(components):
    """输入指纹：components 按名字排好，「名字 NUL 摘要 换行」接起来取 sha256。"""
    return digest_text("".join(f"{name}\0{stable_digest(value)}\n" for name, value in sorted(components, key=lambda pair: pair[0])))


def key_of(components, judging_units):
    """键 = sha256（「输入指纹 NUL 判法单元指纹」）。"""
    return digest_text(f"{input_fingerprint(components)}\0{stable_digest(judging_units)}")


def prune_server_client():
    """scripts/prune-server/prune_server_client.py（判定库服务的客户端），第一次用到时载入（文件头「存放」）；载入不连服务。
    拷进不带它的小树里跑的这一份交 None（library_service_store 照它退 2，自证照它拒跑）。"""
    name = "singlefs_prune_server_client"
    if name not in sys.modules:
        path = os.path.join(REPOSITORY_ROOT, "scripts", "prune-server", "prune_server_client.py")
        if not os.path.isfile(path):
            return None
        specification = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name]


def library_service_store(kind_directory):
    """这一份库（<固定目录>/<kind>）在 prune-server 里的 LibraryStore（文件头「存放」）；这棵树里没有客户端退 2（LibraryUnavailable）。
    弄坏开关 missing-client-tolerated 下没有客户端交 None（「照没给判定库办」），客户端的弄坏开关 unreachable-treated-as-no-library 下客户端交 None。"""
    client = prune_server_client()
    if client is None:
        if "missing-client-tolerated" in BREAK:
            return None
        raise LibraryUnavailable(f"这棵树（{REPOSITORY_ROOT}）里没有 scripts/prune-server/prune_server_client.py",
                                 "在带着 scripts/prune-server/ 的整棵仓里跑：判定库只经 prune-server 的客户端读写")
    kind = f"{os.path.basename(os.path.dirname(kind_directory))}/{os.path.basename(kind_directory)}"
    return client.library_store(client.FAMILY_VERDICT_LIBRARY, kind, kind_directory)


def stored_json(store, relative):
    """(在不在, 读出来的 dict 或 None)：从 prune-server 读这一份库里的 <relative>。在而解不开交 (True, None)；服务不可用客户端退 75。"""
    data = store.read(relative)
    if data is None:
        return False, None
    try:
        return True, json.loads(data.decode("utf-8", "surrogateescape"))
    except ValueError:
        return True, None


def read_library_json(kind_directory, relative):
    """一份 VerdictLibrary 形态的库（<目录>/<kind>）里按相对路径读一份记录（复用哨兵直接读键文件用，lib_reuse_checks.py）；记录不在、解不开交 None。"""
    return stored_json(library_service_store(kind_directory), relative)[1]


def unit_identity(line):
    """单元行「种类<TAB>限定名<TAB>文件:起-止<TAB>指纹」→ ((种类, 限定名, 文件), 指纹)。"""
    fields = line.split("\t")
    if len(fields) != 4:
        return (line, "", ""), ""
    return (fields[0], fields[1], fields[2].rsplit(":", 1)[0]), fields[3]


def changed_units(before, after):
    """两份组成清单比：交 [「改了 <种类> <限定名>」「新进 …」「出去 …」]，按（种类，限定名，文件）对。"""
    old = dict(unit_identity(line) for line in before)
    new = dict(unit_identity(line) for line in after)
    changed = [f"改了 {kind} {name}（{file}）" for (kind, name, file), digest in sorted(new.items()) if (kind, name, file) in old and old[(kind, name, file)] != digest]
    changed += [f"新进 {kind} {name}（{file}）" for (kind, name, file) in sorted(new) if (kind, name, file) not in old]
    changed += [f"出去 {kind} {name}（{file}）" for (kind, name, file) in sorted(old) if (kind, name, file) not in new]
    return changed


class VerdictLibrary:
    def __init__(self, directory, kind, judging_units):
        # directory 只当 prune-server 里的名字（文件头「存放」），本地不建；没给退 2，没有不读不写的形态
        if not directory:
            raise LibraryUnavailable("没给库目录", "照文件头「存放」：library_directory(被判的根, <这一道的固定目录名>) 取")
        self.directory = os.path.join(directory, kind)
        self.judging_units = judging_units
        self.details = {}
        self.records = {}
        self.store = library_service_store(self.directory)

    @staticmethod
    def _key_relative(key):
        return f"keys/{key}.json"

    @staticmethod
    def _item_relative(item):
        return f"items/{digest_text(item)}.json"

    @staticmethod
    def _composition_relative(digest):
        return f"compositions/{digest}.json"

    def _stored(self, relative):
        """(在不在, 读出来的 dict 或 None)，读 prune-server（文件头「存放」）。"""
        return stored_json(self.store, relative)

    def key_exists(self, key):
        """这把键在库里有没有一份记录（不管红绿、读不读得出）：只数键的那一趟用（verus-proof-check.py 的 peek）。"""
        return self._stored(self._key_relative(key))[0]

    def item_record(self, item):
        """这一项最近判绿的那一份（dict），没有交 None。"""
        return self._stored(self._item_relative(item))[1]

    def stored_keys(self):
        """库里有记录的键（排好序）：自证核「写进了哪一份库」用。"""
        names = [relative[len("keys/"):] for relative, _value in self.store.scan("keys/")]
        return sorted(name[:-len(".json")] for name in names if name.endswith(".json"))

    def detail(self, item):
        """上一次 lookup 这一项时算出的变了的单元（「；」接起来，至多 DETAIL_UNIT_LIMIT 个，多的报个数）；没有交空串。"""
        return self.details.get(item, "")

    def record_of(self, item):
        """上一次 lookup 这一项复用到的那一份记录（dict）；没复用交 None。"""
        return self.records.get(item)

    def _explain(self, item, latest, components):
        """latest 是这一项最近判绿的那一份：逐样比组成清单，记下变了的单元。"""
        if "composition-not-recorded" in BREAK:
            return
        notes = []
        compositions = latest.get("compositions") or {}
        for name, value in sorted(list(components) + [(JUDGING_UNITS_COLUMN, self.judging_units)], key=lambda pair: pair[0]):
            if not isinstance(value, UnitKey) or name not in compositions:
                continue
            stored_digest = latest.get("judging_units") if name == JUDGING_UNITS_COLUMN else (latest.get("components") or {}).get(name)
            if stored_digest == value.key:
                continue
            stored = self._stored(self._composition_relative(compositions[name]))[1]
            if not isinstance(stored, list):
                continue
            changes = changed_units(stored, value.units)
            if changes:
                shown = "、".join(changes[:DETAIL_UNIT_LIMIT]) + (f" 等 {len(changes)} 个" if len(changes) > DETAIL_UNIT_LIMIT else "")
                notes.append(f"{name}：{shown}")
        if notes:
            self.details[item] = "；".join(notes)

    def lookup(self, item, components):
        """交 (复用?, 原因)：复用时原因是记下那一份的日期与一句判定；现判时原因是无记录、记录读不出、拒算、
        上次判红（这把键记着红：record_red 写的，那一句与日期另由 detail(项) 交），
        或「<输入里哪几样>、判法单元指纹 变了」（变了的输入按名字排，判法单元指纹变了排在最后）；变了的单元另由 detail(项) 交。"""
        self.details.pop(item, None)
        self.records.pop(item, None)
        refused = refused_component(components, self.judging_units)
        if refused is not None:
            name, value = refused
            self.details[item] = f"{name}：{value.first_refusal()}"
            return False, f"{REFUSED_REASON}（{name}）"
        key = key_of(components, self.judging_units)
        key_present, record = self._stored(self._key_relative(key))
        if key_present:
            if record is not None and record.get("verdict") == "red" and record.get("key") == key:
                self.details[item] = f"{record.get('recorded', '?')} 判红：{record.get('summary', '')}"
                return False, RED_RECORD_REASON
            if record is None or record.get("verdict") != "green" or record.get("key") != key:
                return False, BROKEN_RECORD_REASON
            latest = self._stored(self._item_relative(item))[1]
            if latest is None or latest.get("key") != key:
                # 这一项最近那一份换成复用的这一份：下一趟「现判」的原因比的是这一趟的输入
                self.store.write_many([(self._item_relative(item), json.dumps(dict(record, item=item), ensure_ascii=False, sort_keys=True) + "\n")])
            self.records[item] = record
            return True, f"{record.get('recorded', '?')} 判绿：{record.get('summary', '')}"
        latest = self._stored(self._item_relative(item))[1]
        if latest is None or not isinstance(latest.get("components"), dict):
            return False, NO_RECORD_REASON
        digests = {name: stable_digest(value) for name, value in components}
        changed = [name for name in sorted(digests) if latest["components"].get(name) != digests[name]]
        changed += [name for name in sorted(latest["components"]) if name not in digests]
        if latest.get("judging_units") != stable_digest(self.judging_units):
            changed.append(JUDGING_UNITS_COLUMN)
        self._explain(item, latest, components)
        return False, ("、".join(changed) + " 变了") if changed else BROKEN_RECORD_REASON

    def lookup_by_item_only(self, item):
        """弄坏开关用（几份判法的 reuse-ignores-key）：这一项最近判绿过就复用，不比键。正常判法不调它。"""
        latest = self._stored(self._item_relative(item))[1]
        if latest is None:
            return False, NO_RECORD_REASON
        self.records[item] = latest
        return True, f"{latest.get('recorded', '?')} 判绿（没比键）：{latest.get('summary', '')}"

    def record(self, item, components, summary, run="", details=None):
        """判绿之后调：组成清单、键那一份与这一项最近那一份放一个写帧写进 prune-server（键那一份盖掉这把键原有的红）。写成交 None；
        有拒算的不写（交原因）；服务不可用客户端退 75。run 是跑法（判出这一份的那条命令或那几样参数），details 是调用方要随判定存的几样数。"""
        return self._write(item, components, summary, run, details, "green")

    def record_red(self, item, components, summary, run=None, details=None):
        """判红之后调：照同一把键写一条 verdict=red 的记录，盖掉这把键原有的绿；下一趟 lookup 答现判、原因「上次判红」。
        这一项最近判绿的那一份（items/）不动。参数、返回同 record。弄坏开关 red-not-recorded 下什么都不写。"""
        if "red-not-recorded" in BREAK:
            return None
        return self._write(item, components, summary, run or "", details, "red")

    def _write(self, item, components, summary, run, details, verdict):
        refused = refused_component(components, self.judging_units)
        if refused is not None:
            return f"拒算（{refused[0]}），不记"
        key = key_of(components, self.judging_units)
        compositions = {}
        pairs = []
        if "composition-not-recorded" not in BREAK:
            for name, value in list(components) + [(JUDGING_UNITS_COLUMN, self.judging_units)]:
                if isinstance(value, UnitKey) and value.units:
                    digest = value.composition_digest()
                    compositions[name] = digest
                    if not self._stored(self._composition_relative(digest))[0]:
                        pairs.append((self._composition_relative(digest), json.dumps(list(value.units), ensure_ascii=False) + "\n"))
        text = json.dumps({"item": item, "key": key, "input_fingerprint": input_fingerprint(components),
                           "judging_units": stable_digest(self.judging_units),
                           "components": {name: stable_digest(value) for name, value in components}, "compositions": compositions,
                           "verdict": verdict, "summary": summary, "details": dict(details or {}), "run": run,
                           "recorded": datetime.date.today().isoformat()}, ensure_ascii=False, sort_keys=True) + "\n"
        pairs.append((self._key_relative(key), text))
        if verdict == "green":
            pairs.append((self._item_relative(item), text))
        self.store.write_many(pairs)
        return None


class Tally:
    """一格里现判几项、复用几项，现判的按原因分栏（原因是 lookup 交的那一句，crates 变异表的是复用判据的栏名）。"""

    def __init__(self):
        self.reused = 0
        self.reasons = {}

    def count(self, reused, reason):
        if reused:
            self.reused += 1
        else:
            self.reasons[reason] = self.reasons.get(reason, 0) + 1

    @property
    def judged(self):
        return sum(self.reasons.values())

    def text(self):
        columns = "、".join(f"{reason} {count}" for reason, count in sorted(self.reasons.items(), key=lambda pair: (-pair[1], pair[0])))
        return f"现判 {self.judged}、复用 {self.reused}" + (f"（现判的原因：{columns}）" if columns else "")

    def append_to(self, path, cell):
        """门禁给的 FORMAL_ITEM_TALLY_FILE：追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」，门禁汇总时整道相加。"""
        if not path:
            return
        columns = "、".join(f"{reason} {count}" for reason, count in sorted(self.reasons.items()))
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{cell}\t{self.judged}\t{self.reused}\t{columns}\n")


# ── 命令行（shell 写的门禁用） ──

EXIT_DONE, EXIT_JUDGE, EXIT_USAGE, EXIT_CANNOT, EXIT_REFUSED = 0, 1, 2, 3, 4


def read_unit_key_file(path):
    """unit-key 的输出文件 → UnitKey（有拒算行时键是 None）。"""
    key, units, refusals = None, [], []
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        for line in handle.read().split("\n"):
            if line.startswith("key\t"):
                key = line.split("\t", 1)[1]
            elif line.startswith("refused\t"):
                refusals.append(line)
            elif line:
                units.append(line)
    return UnitKey(None if refusals else key, tuple(units), tuple(refusals))


def read_components(path):
    """组成表 → (components, 判法单元指纹)。一行「名字<TAB>摘要」或「名字<TAB>@<unit-key 输出文件>」；空行与 # 开头的不算。"""
    components, judge = [], ""
    base = os.path.dirname(os.path.abspath(path))
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.startswith("#"):
                continue
            name, separator, value = line.partition("\t")
            if not separator or not name or not value:
                raise ValueError(f"{path} 第 {number} 行不是「名字<TAB>摘要」")
            if value.startswith("@"):
                target = value[1:] if os.path.isabs(value[1:]) else os.path.join(base, value[1:])
                value = read_unit_key_file(target)
            if name == JUDGING_UNITS_COLUMN:
                judge = value
            else:
                components.append((name, value))
    return components, judge


def unit_key_command(arguments, judging):
    root, entries, options = None, [], collections.defaultdict(list)
    valued = ("--target", "--cfg", "--toolchain", "--dependency", "--external", "--exclude-file", "--exclude-unit", "--dependency-of")
    programs_keyed_elsewhere = False
    position = 0
    while position < len(arguments):
        word = arguments[position]
        if word == "--programs-keyed-elsewhere":
            programs_keyed_elsewhere = True
            position += 1
            continue
        if word in valued:
            if position + 1 >= len(arguments):
                raise ValueError(f"{word} 后面缺值")
            options[word].append(arguments[position + 1])
            position += 2
            continue
        if word.startswith("--"):
            raise ValueError(f"认不出的选项 {word}")
        if root is None:
            root = word
        else:
            entries.append(word)
        position += 1
    if root is None or not entries:
        raise ValueError("要一个根与至少一个入口")
    dependencies = list(options["--dependency"])
    for package in options["--dependency-of"]:
        dependencies += locked_dependency_lines(root, package)
    externals = dict(parse_external(text) for text in options["--external"])
    index = FunctionUnitIndex(root)
    if judging:
        unit_key = judging_unit_key(root, entries, index)
    else:
        unit_key = index.key(entries, target=(options["--target"] or [None])[-1], cfg=options["--cfg"], toolchain=options["--toolchain"],
                             dependencies=dependencies, externals=externals, excluded_files=options["--exclude-file"],
                             excluded_units=options["--exclude-unit"], programs_keyed_elsewhere=programs_keyed_elsewhere)
    for line in unit_key.units:
        print(line)
    if unit_key.key is None:
        for line in unit_key.refusals:
            print(line)
        return EXIT_REFUSED
    print(f"key\t{unit_key.key}")
    return EXIT_DONE


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程（自证设了 VERDICT_LIBRARY_SELFTEST=1）而 VERDICT_LIBRARY_BREAK 非空
    （不管值是什么），退 2 并点名它只在自证里认；在参数解析之前判。"""
    if os.environ.get("VERDICT_LIBRARY_BREAK") and arguments[:1] != ["--selftest"] and os.environ.get("VERDICT_LIBRARY_SELFTEST") != "1":
        print("  ✗ VERDICT_LIBRARY_BREAK 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
        print("     → 怎么办：去掉 VERDICT_LIBRARY_BREAK 再跑；要证红就跑 VERDICT_LIBRARY_BREAK=<项> python3 scripts/crash/lib_verdict_library.py --selftest", file=sys.stderr)
        return EXIT_USAGE
    return None


def main(arguments):
    refusal = refuse_break_outside_selftest(arguments)
    if refusal is not None:
        return refusal
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if not arguments or arguments[0] not in ("unit-key", "judging-units", "lookup", "record", "record-red"):
        print("  ✗ 用法：lib_verdict_library.py unit-key|judging-units <根> <入口>… | lookup|record|record-red <目录> <种类> <项> <组成表> … | --selftest")
        print("     → 怎么办：看文件头「命令行」一段")
        return EXIT_USAGE
    command, rest = arguments[0], arguments[1:]
    try:
        if command in ("unit-key", "judging-units"):
            return unit_key_command(rest, command == "judging-units")
        if len(rest) < 4:
            raise ValueError(f"{command} 要 <目录> <种类> <项> <组成表>")
        directory, kind, item, table = rest[:4]
        extra = rest[4:]
        components, judge = read_components(table)
        if command == "lookup":
            if extra:
                raise ValueError(f"lookup 认不出 {extra[0]}")
            library = VerdictLibrary(directory, kind, judge)
            reused, reason = library.lookup(item, components)
            print(("复用\t" if reused else "现判\t") + reason + ("" if reused else "\t" + library.detail(item)))
            return EXIT_DONE if reused else EXIT_JUDGE
        summary, run, details = "", "", {}
        position = 0
        while position < len(extra):
            if extra[position] in ("--summary", "--run", "--detail") and position + 1 < len(extra):
                value = extra[position + 1]
                if extra[position] == "--summary":
                    summary = value
                elif extra[position] == "--run":
                    run = value
                else:
                    name, _equals, detail_value = value.partition("=")
                    details[name] = detail_value
                position += 2
                continue
            raise ValueError(f"{command} 认不出 {extra[position]}")
        library = VerdictLibrary(directory, kind, judge)
        failed = (library.record_red if command == "record-red" else library.record)(item, components, summary, run=run, details=details)
        if failed:
            print(f"  ! {failed}")
            return EXIT_JUDGE
        return EXIT_DONE
    except ValueError as error:
        print(f"  ✗ {command}：{error}")
        print("     → 怎么办：看文件头「命令行」一段的写法")
        return EXIT_USAGE if "要" in str(error) or "认不出" in str(error) or "缺值" in str(error) else EXIT_CANNOT


# ── 自证 ──

SELFTEST_PYTHON = '''"""自证用的小判法。"""
LIMIT = 3


def helper(value):
    return value + LIMIT


def judge(value):
    return helper(value) > 5


def unrelated(value):
    return value * 2
'''

SELFTEST_SHELL = '''#!/usr/bin/env bash
helper() {
  printf '%s\\n' "$1"
}
judge() {
  helper "$1"
}
'''

SELFTEST_RUST_MANIFEST = '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n'
SELFTEST_RUST = '''pub const LIMIT: u32 = 3;

pub fn helper(value: u32) -> u32 {
    value + LIMIT
}

pub fn judge(value: u32) -> bool {
    helper(value) > 5
}

pub fn unrelated(value: u32) -> u32 {
    value * 2
}
'''


def selftest():
    import shutil
    # 判定库的格写进 prune-server：只在临时服务上跑（文件头「命令行」--selftest），不是就一格都不跑，不写真服务
    client = prune_server_client()
    service_problem = client.selftest_service_problem() if client is not None else "这棵树里没有 scripts/prune-server/prune_server_client.py"
    if service_problem:
        print(f"  ✗ lib_verdict_library 自证没跑：{service_problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY if client is not None else '在带着 scripts/prune-server/ 的仓里跑'}")
        return EXIT_USAGE
    os.environ["VERDICT_LIBRARY_SELFTEST"] = "1"   # 自证起的子进程（命令行 unit-key、lookup、record 与逐个开关的 --selftest）凭它认弄坏开关
    cases, wrong = 0, []

    def cell(label, passed, detail):
        nonlocal cases
        cases += 1
        if not passed:
            wrong.append(label)
            print(f"    {label} 这一格判错：{detail}")  # gate-lint:detail

    with tempfile.TemporaryDirectory(prefix="lib-verdict-library-selftest-") as work:
        root = os.path.join(work, "root")
        os.makedirs(os.path.join(root, "scripts", "gate"))
        os.makedirs(os.path.join(root, "crates", "sample", "src"))
        with open(os.path.join(root, "Cargo.toml"), "w", encoding="utf-8") as handle:
            handle.write('[workspace]\nmembers = ["crates/sample"]\nresolver = "2"\n')
        with open(os.path.join(root, "Cargo.lock"), "w", encoding="utf-8") as handle:
            handle.write('version = 4\n\n[[package]]\nname = "sample"\nversion = "0.1.0"\ndependencies = ["outside"]\n\n'
                         '[[package]]\nname = "outside"\nversion = "1.2.3"\nsource = "registry+https://example.invalid"\nchecksum = "abc"\n\n'
                         '[[package]]\nname = "unrelated-outside"\nversion = "9.9.9"\nsource = "registry+https://example.invalid"\nchecksum = "def"\n')
        files = {"scripts/gate/judge.py": SELFTEST_PYTHON, "scripts/gate/judge.sh": SELFTEST_SHELL,
                 "crates/sample/Cargo.toml": SELFTEST_RUST_MANIFEST, "crates/sample/src/lib.rs": SELFTEST_RUST}

        def write_all(changes=None):
            for relative, text in files.items():
                for old, new in (changes or {}).get(relative, ()):
                    if text.count(old) != 1:
                        raise AssertionError(f"自证的改动原文在 {relative} 里要恰好命中一次：{old!r}")
                    text = text.replace(old, new)
                with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                    handle.write(text)

        entries = {"python": "scripts/gate/judge.py::judge", "shell": "scripts/gate/judge.sh::judge", "rust": "crates/sample/src/lib.rs::judge"}

        def keys(changes=None):
            write_all(changes)
            index = FunctionUnitIndex(root)
            return {language: index.key([entry]) for language, entry in entries.items()}

        try:
            base = keys()
            cell("三种语言都算得出键", all(value.key for value in base.values()), {language: value.refusals for language, value in base.items()})
            comment_only = keys({"scripts/gate/judge.py": [("    return helper(value) > 5\n", "    # 只是注释\n    return helper(value) > 5  # 行尾注释\n")],
                                 "scripts/gate/judge.sh": [('  helper "$1"\n', '  # 只是注释\n  helper "$1"\n')],
                                 "crates/sample/src/lib.rs": [("    helper(value) > 5\n", "    // 只是注释\n    helper(value) > 5 /* 行尾 */\n")]})
            cell("只改注释单元键不变", all(comment_only[language].key == base[language].key for language in entries),
                 {language: (base[language].key, comment_only[language].key) for language in entries})
            body = keys({"scripts/gate/judge.py": [("LIMIT = 3", "LIMIT = 4")], "scripts/gate/judge.sh": [("printf '%s\\n'", "printf '%s:\\n'")],
                         "crates/sample/src/lib.rs": [("LIMIT: u32 = 3", "LIMIT: u32 = 4")]})
            cell("改被调到的单元（常量、下游函数）键变", all(body[language].key != base[language].key for language in entries),
                 {language: (base[language].key, body[language].key) for language in entries})
            unrelated = keys({"scripts/gate/judge.py": [("value * 2", "value * 3")], "crates/sample/src/lib.rs": [("value * 2", "value * 3")]})
            cell("改没被调到的函数键不变", unrelated["python"].key == base["python"].key and unrelated["rust"].key == base["rust"].key,
                 {language: (base[language].key, unrelated[language].key) for language in ("python", "rust")})
            write_all()
            dependencies = locked_dependency_lines(root, "sample")
            cell("依赖版本行只收引到的外部包", dependencies == ["outside 1.2.3 registry+https://example.invalid abc"], dependencies)
            # 判定库：记、复用、作废时点名变了的单元、拒算不复用
            selftest_library = os.path.join(work, "library")
            judge_key = UnitKey(digest_text("判法"), ("fn\tjudge\tx.py:1-2\taaa",), ())
            store = VerdictLibrary(selftest_library, "sample", judge_key)
            components = [("被判函数", base["rust"]), ("这一行", digest_text("第一行"))]
            first = store.lookup("一项", components)
            failed = store.record("一项", components, "判绿", run="自证", details={"verified": "3"})
            second = store.lookup("一项", components)
            cell("记下之后同一把键复用、记录带跑法与日期与数", first == (False, NO_RECORD_REASON) and failed is None and second[0]
                 and (store.record_of("一项") or {}).get("details", {}).get("verified") == "3" and (store.record_of("一项") or {}).get("run") == "自证",
                 f"第一次 {first}，写 {failed}，第二次 {second}，记录 {store.record_of('一项')}")
            changed = keys({"crates/sample/src/lib.rs": [("LIMIT: u32 = 3", "LIMIT: u32 = 4")]})
            reused, reason = store.lookup("一项", [("被判函数", changed["rust"]), ("这一行", digest_text("第一行"))])
            cell("作废时点名变了的单元", not reused and reason == "被判函数 变了" and "改了 const LIMIT" in store.detail("一项"),
                 f"{reused} {reason} 「{store.detail('一项')}」")
            refused = UnitKey(None, (), ("refused\tgetattr-variable\tx.py:3\tgetattr(x, name)",))
            store.record("拒算项", [("被判函数", UnitKey(digest_text("拒算"), (), ()))], "判绿")
            reused, reason = store.lookup("拒算项", [("被判函数", refused)])
            wrote = store.record("拒算项", [("被判函数", refused)], "判绿")
            cell("拒算不复用、不记", not reused and reason.startswith(REFUSED_REASON) and "getattr-variable" in store.detail("拒算项") and wrote,
                 f"{reused} {reason} 「{store.detail('拒算项')}」 写 {wrote}")
            other_judge = VerdictLibrary(selftest_library, "sample", UnitKey(digest_text("另一份判法"), ("fn\tjudge\tx.py:1-2\tbbb",), ()))
            reused, reason = other_judge.lookup("一项", components)
            cell("判法单元指纹变了整项现判、点名判法的单元", not reused and reason == f"{JUDGING_UNITS_COLUMN} 变了" and "改了 fn judge" in other_judge.detail("一项"),
                 f"{reused} {reason} 「{other_judge.detail('一项')}」")
            # 记红、撤绿、下趟现判（文件头「键」）：同一把键先判绿入库、再判红 ⇒ 下一趟现判、原因「上次判红」；红之后再判绿 ⇒ 下一趟复用
            red_store = VerdictLibrary(selftest_library, "red-sample", judge_key)
            red_components = [("被判函数", base["rust"]), ("这一行", digest_text("红的那一行"))]
            red_store.record("红项", red_components, "判绿", run="自证")
            green_first = red_store.lookup("红项", red_components)
            wrote_red = red_store.record_red("红项", red_components, "判红：断言没过", run="自证")
            after_red = red_store.lookup("红项", red_components)
            cell("判绿入库、同一把键判红之后，下一趟这一项现判、不复用",
                 green_first[0] and wrote_red is None and after_red == (False, RED_RECORD_REASON) and "判红：断言没过" in red_store.detail("红项"),
                 f"判绿之后 {green_first}，写红 {wrote_red}，判红之后 {after_red} 「{red_store.detail('红项')}」")
            red_store.record("红项", red_components, "又判绿", run="自证")
            green_again = red_store.lookup("红项", red_components)
            cell("红之后再判绿，下一趟复用", green_again[0] and (red_store.record_of("红项") or {}).get("summary") == "又判绿",
                 f"{green_again}，记录 {red_store.record_of('红项')}")
            # 库目录（文件头「存放」）：git 工作树取 <common-dir>/<库名>；不是 git 工作树的退 2、点名那棵树，不交「没有库」
            main_tree, copy_tree = os.path.join(work, "main-tree"), os.path.join(work, "copy-tree")
            os.makedirs(main_tree)
            os.makedirs(copy_tree)
            initialized = subprocess.run(["git", "init", "-q", main_tree], capture_output=True, text=True)
            main_common = git_common_directory(main_tree)
            located = library_directory(main_tree, "selftest-verdicts")
            try:
                outside = ("交了", library_directory(copy_tree, "selftest-verdicts"))
            except SystemExit as error:
                outside = ("退出", error.code)
            cell("不在 git 工作树里取库目录退 2",
                 initialized.returncode == 0 and main_common is not None and located == os.path.join(main_common, "selftest-verdicts") and outside == ("退出", EXIT_USAGE),
                 f"git init 退 {initialized.returncode}（{initialized.stderr.strip()[-200:]}），工作树取到 {located}（要 {main_common}/selftest-verdicts），"
                 f"不是工作树的 {copy_tree} {outside[0]} {outside[1]}（要退出 {EXIT_USAGE}）")
            # 这棵树里没有客户端（拷进不带 scripts/prune-server/ 的小树）：命令行 lookup 退 2、点名缺客户端，不答现判
            small_tree = os.path.join(work, "small-tree")
            os.makedirs(os.path.join(small_tree, "scripts", "crash"))
            shutil.copy2(os.path.realpath(__file__), os.path.join(small_tree, "scripts", "crash", "lib_verdict_library.py"))
            small_table = os.path.join(work, "small-components.tsv")
            with open(small_table, "w", encoding="utf-8") as handle:
                handle.write(f"这一档\t{digest_text('direct')}\n{JUDGING_UNITS_COLUMN}\t{digest_text('判法')}\n")
            small = subprocess.run([sys.executable, os.path.join(small_tree, "scripts", "crash", "lib_verdict_library.py"), "lookup",
                                    os.path.join(main_common or work, "selftest-verdicts"), "small", "一项", small_table], capture_output=True, text=True)
            cell("树里没有客户端的命令行 lookup 退 2", small.returncode == EXIT_USAGE and "prune_server_client.py" in small.stderr and not small.stdout.startswith("现判"),
                 f"退 {small.returncode}（要 {EXIT_USAGE}），标准输出「{small.stdout.strip()[:200]}」，标准错误「{small.stderr.strip()[-300:]}」")
            # 判法单元指纹不含库自己：仓里的 verus-proof-check.py 的 check_proof 闭包里，判定库与 lib_function_units 的单元一个都没有
            repository_index = FunctionUnitIndex(REPOSITORY_ROOT)
            judging = judging_unit_key(REPOSITORY_ROOT, ["scripts/gate/verus-proof-check.py::check_proof"], repository_index)
            leaked = [line for line in judging.units if any(f"\t{path}:" in line for path in NOT_JUDGING_FILES)]
            cell("判法单元指纹不含库自己", judging.key is not None and judging.units and not leaked, f"键 {judging.key}，混进来的 {leaked[:3]}，拒算 {judging.refusals[:2]}")
            # 命令行：unit-key 写文件、lookup 现判、record、再 lookup 复用
            unit_file = os.path.join(work, "unit-key.txt")
            with open(unit_file, "w", encoding="utf-8") as handle:
                done = subprocess.run([sys.executable, os.path.realpath(__file__), "unit-key", root, entries["shell"]], stdout=handle, stderr=subprocess.PIPE, text=True)
            table = os.path.join(work, "components.tsv")
            with open(table, "w", encoding="utf-8") as handle:
                handle.write(f"宿主脚本单元\t@{unit_file}\n这一档\t{digest_text('direct')}\n{JUDGING_UNITS_COLUMN}\t{digest_text('判法')}\n")
            command = [sys.executable, os.path.realpath(__file__)]
            before = subprocess.run(command + ["lookup", selftest_library, "shell", "direct", table], capture_output=True, text=True)
            recorded = subprocess.run(command + ["record", selftest_library, "shell", "direct", table, "--summary", "判绿", "--run", "自证"], capture_output=True, text=True)
            after = subprocess.run(command + ["lookup", selftest_library, "shell", "direct", table], capture_output=True, text=True)
            cell("命令行 unit-key、lookup、record 一个来回", done.returncode == 0 and before.returncode == 1 and before.stdout.startswith("现判\t无记录")
                 and recorded.returncode == 0 and after.returncode == 0 and after.stdout.startswith("复用\t"),
                 f"unit-key 退 {done.returncode}（{done.stderr.strip()[-200:]}），lookup 退 {before.returncode}「{before.stdout.strip()}」，"
                 f"record 退 {recorded.returncode}「{recorded.stdout.strip()}」，再 lookup 退 {after.returncode}「{after.stdout.strip()}」")
            recorded_red = subprocess.run(command + ["record-red", selftest_library, "shell", "direct", table, "--summary", "判红", "--run", "自证"], capture_output=True, text=True)
            after_red_line = subprocess.run(command + ["lookup", selftest_library, "shell", "direct", table], capture_output=True, text=True)
            cell("命令行 record-red 之后 lookup 现判、原因上次判红", recorded_red.returncode == 0 and after_red_line.returncode == 1
                 and after_red_line.stdout.startswith(f"现判\t{RED_RECORD_REASON}\t"),
                 f"record-red 退 {recorded_red.returncode}「{recorded_red.stdout.strip()}」，再 lookup 退 {after_red_line.returncode}「{after_red_line.stdout.strip()}」")
            # 判定只进 prune-server（文件头「存放」）：上面几格记过、复用过的那几份库，库目录底下一个判定文件都没有
            local_library_files = [os.path.relpath(os.path.join(walked, name), work) for base in (selftest_library, os.path.join(main_common or work, "selftest-verdicts"))
                                   for walked, _directories, names in os.walk(base) for name in names]
            cell("判定只进服务、库目录本地不建", not os.path.exists(selftest_library) and not local_library_files and bool(store.stored_keys()),
                 f"库目录 {selftest_library} 在不在 {os.path.exists(selftest_library)}，本地判定文件 {local_library_files[:3]}，服务里这份库的键 {len(store.stored_keys())} 个")
        except (AssertionError, ValueError) as error:
            cell("自证的现场", False, str(error))
        shutil.rmtree(root, ignore_errors=True)

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        for item, cell_label in (("unit-key-from-raw-file", "只改注释单元键不变"), ("refusal-reused", "拒算不复用、不记"),
                                 ("composition-not-recorded", "作废时点名变了的单元"), ("judging-files-included", "判法单元指纹不含库自己"),
                                 ("red-not-recorded", "判绿入库、同一把键判红之后，下一趟这一项现判、不复用"),
                                 ("library-directory-outside-git-tolerated", "不在 git 工作树里取库目录退 2"),
                                 ("missing-client-tolerated", "树里没有客户端的命令行 lookup 退 2")):
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                       env=dict(os.environ, VERDICT_LIBRARY_BREAK=item))
            if completed.returncode == 0 or f"{cell_label} 这一格判错" not in completed.stdout:
                breaks_missed.append(f"{item}（要红在「{cell_label}」）")
            else:
                breaks_proven += 1
    if wrong:
        print(f"  ✗ lib_verdict_library 自证 {cases} 格里 {len(wrong)} 格判错：{'、'.join(wrong)}")
        print("     → 怎么办：按判错那几格的标签找对应的函数修（单元键在 FunctionUnitIndex，存放与原因在 VerdictLibrary）；判据写在文件头")
        return 1
    if breaks_missed:
        print(f"  ✗ 弄坏开关打开之后没红在点名那一格：{'；'.join(breaks_missed)}")
        print("     → 怎么办：那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return 1
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都红在点名那一格" if not BREAK else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ lib_verdict_library 自证 {cases} 格都判对{suffix}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
