"""判定库：每条判定按它的单元键存，下一趟键相同就复用，不再起重活。
判法照 .claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」；口径照 .claude/kb/decisions/13-验证路线.md 已定项 17
（判定复用的单元是函数，键只由函数单元、工具链、依赖版本与这条判定自己的内容输入组成）。

Python 一侧这一个概念只叫「判定库」（类名 VerdictLibrary）。Rust 一侧 crates/singlefs-checker-tier 的判定存储（verdict_store 模块）是另一份存法，不在这里。
用它的：
  门禁 checker-tier-formal-verification 的两份判法（scripts/gate/verus-proof-check.py、scripts/experiments/e165-pin-red-examples.py）、
    门禁 checker-tier-lkmm（.claude/scripts/lkmm.sh 每份 litmus 两条判定）与门禁 checker-tier-qemu-device-streams（每档一条判定）用整套：
    函数单元闭包键（FunctionUnitIndex）、VerdictLibrary 存放与按键读、write_atomically 原子写、Tally 现判与复用分开报；两道 shell 写的门禁经下面的命令行用。
  门禁 checker-tier-crates-mutation-replay 的判法（scripts/mutation/crates-mutation-rows.py）用 write_atomically、read_text 与 Tally。
不用它、各自留一份的（gate-overlap:copy-kept 写在它们那里）：广谱变异的结论表（scripts/mutation/broad-mutation.py，只追加的 TSV）。

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
  只记绿：红、没跑成、被信号停了的一律不记，下一趟照判。键相同就复用，不看记下来的是哪一项的名字（输入里要有这一项自己的那一段，不然两项会共用一份判定）。
  一项判完当场落盘（write_atomically：同目录排他建临时文件、写完 fsync、改名换上）；被打断之后同一条命令重跑，判过的复用、没判的接着判。

存放（directory 由调用方给，几道门禁各给 git common-dir 下自己那一份固定目录，跨轮留着，不放 /tmp）：
  <directory>/<kind>/keys/<键>.json      这一份判定：item、key、input_fingerprint、judging_units、components（名字 → 摘要）、compositions（名字 → 组成清单的摘要）、
                                         verdict（只有 green）、summary、details（调用方给的几样数，例 verified 条数）、recorded（判定日期）、run（跑法）
  <directory>/<kind>/items/<项名的 sha256>.json   这一项最近一次判绿的那一份（与上面同内容），只用来说清「现判」的原因
  <directory>/<kind>/compositions/<组成清单的摘要>.json   组成清单本身（单元行），几条判定共用一份；只给 detail 报变了的单元，不进任何键

用法（python）：
    from lib_verdict_library import FunctionUnitIndex, VerdictLibrary, Tally, digest_text, judging_unit_key
    index = FunctionUnitIndex(根)
    units = index.key(["crates/x/src/bin/y.rs::main"], toolchain=[...], dependencies=locked_dependency_lines(根, "x"))
    library = VerdictLibrary(目录或 None, "proof-function", judging_unit_key(仓根, ["scripts/gate/z.py::check"]), reuse=True)
    reused, reason = library.lookup(项名, [("被验函数单元", units), ("这一行", digest_text(...))])
    ... 现判 ... library.record(项名, components, "一句判定", run="跑法", details={...})   # 只在判绿时调
    tally.count(reused, reason); print(tally.text())
目录给 None：不读不写（reason 写「没给判定库」），判法照旧全部现判。

命令行（shell 写的门禁用；退出码 0 成、1 现判 / 写不成、2 用法错、3 算不出、4 拒算）：
  lib_verdict_library.py unit-key <根> <入口>… [--target 目标] [--cfg 谓词]… [--toolchain 行]… [--dependency 行]… [--external 名=sha256=版本行]…
                                    [--exclude-file 相对根的路径]… [--exclude-unit <文件>::<限定名>]…
      打「key<TAB><键>」与组成清单（单元行），有拒算打拒算行、不打键行、退 4。--dependency-of <包名> 照 locked_dependency_lines 补依赖行；
      --programs-keyed-elsewhere 照 judging_unit_key 那样不要求外部程序（判法单元指纹用：外部程序由每条判定自己的那几样管）。
  lib_verdict_library.py judging-units <根> <入口>…        同 unit-key，去掉 NOT_JUDGING_FILES 两份的单元
  lib_verdict_library.py lookup <目录> <种类> <项> <组成表> [--no-reuse]
      组成表一行一样「名字<TAB>摘要」或「名字<TAB>@<unit-key 的输出文件>」，名字写 判法单元指纹 的那一行是判法单元指纹那一列；
      打一行「复用<TAB>原因」退 0，或「现判<TAB>原因<TAB>变了的单元」退 1
  lib_verdict_library.py record <目录> <种类> <项> <组成表> --summary <一句判定> --run <跑法> [--detail 名=值]…   判绿之后调；写不成退 1
  lib_verdict_library.py --selftest

弄坏开关 VERDICT_LIBRARY_BREAK=<项>[,<项>…]（只给证红用；自证逐个打开，核点名的那一格转红）：
  unit-key-from-raw-file   单元键改取入口文件整份原文的 sha256（不切单元、不去注释）      「只改注释单元键不变」那一格红
  refusal-reused           拒算的键照一个固定的摘要算、照样复用                           「拒算不复用」那一格红
  composition-not-recorded 不存组成清单，detail 报不出变了的单元                           「作废时点名变了的单元」那一格红
  judging-files-included   判法单元指纹里留着这份库与 lib_function_units.py 的单元        「判法单元指纹不含库自己」那一格红
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

NO_LIBRARY_REASON = "没给判定库"
REUSE_OFF_REASON = "关了复用"
NO_RECORD_REASON = "无记录"
BROKEN_RECORD_REASON = "记录读不出"
REFUSED_REASON = "拒算"
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


def write_atomically(path, data):
    """同目录排他建临时文件、写完 fsync、改名换上；data 是 str 或 bytes，所在目录不在先建。失败时删掉临时文件，原文件不动，异常照抛。"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    handle_number, temporary = tempfile.mkstemp(dir=os.path.dirname(path), prefix=os.path.basename(path) + ".partial.")
    try:
        if isinstance(data, bytes):
            with os.fdopen(handle_number, "wb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
        else:
            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


def read_text(path):
    """按键读一份记录的原文：不在、读不了交 None（下一趟照判，不抛）。"""
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            return handle.read()
    except OSError:
        return None


def read_json(path):
    text = read_text(path)
    if text is None:
        return None
    try:
        return json.loads(text)
    except ValueError:
        return None


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
    def __init__(self, directory, kind, judging_units, reuse=True):
        self.directory = os.path.join(directory, kind) if directory else None
        self.judging_units = judging_units
        self.reuse = reuse
        self.details = {}
        self.records = {}

    def _key_path(self, key):
        return os.path.join(self.directory, "keys", key + ".json")

    def _item_path(self, item):
        return os.path.join(self.directory, "items", digest_text(item) + ".json")

    def _composition_path(self, digest):
        return os.path.join(self.directory, "compositions", digest + ".json")

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
            stored = read_json(self._composition_path(compositions[name]))
            if not isinstance(stored, list):
                continue
            changes = changed_units(stored, value.units)
            if changes:
                shown = "、".join(changes[:DETAIL_UNIT_LIMIT]) + (f" 等 {len(changes)} 个" if len(changes) > DETAIL_UNIT_LIMIT else "")
                notes.append(f"{name}：{shown}")
        if notes:
            self.details[item] = "；".join(notes)

    def lookup(self, item, components):
        """交 (复用?, 原因)：复用时原因是记下那一份的日期与一句判定；现判时原因是没给判定库、关了复用、无记录、记录读不出、拒算，
        或「<输入里哪几样>、判法单元指纹 变了」（变了的输入按名字排，判法单元指纹变了排在最后）；变了的单元另由 detail(项) 交。"""
        self.details.pop(item, None)
        self.records.pop(item, None)
        refused = refused_component(components, self.judging_units)
        if refused is not None:
            name, value = refused
            self.details[item] = f"{name}：{value.first_refusal()}"
            return False, f"{REFUSED_REASON}（{name}）"
        if self.directory is None:
            return False, NO_LIBRARY_REASON
        if not self.reuse:
            return False, REUSE_OFF_REASON
        key = key_of(components, self.judging_units)
        if os.path.exists(self._key_path(key)):
            record = read_json(self._key_path(key))
            if record is None or record.get("verdict") != "green" or record.get("key") != key:
                return False, BROKEN_RECORD_REASON
            latest = read_json(self._item_path(item))
            if latest is None or latest.get("key") != key:
                # 这一项最近那一份换成复用的这一份：下一趟「现判」的原因比的是这一趟的输入
                try:
                    write_atomically(self._item_path(item), json.dumps(dict(record, item=item), ensure_ascii=False, sort_keys=True) + "\n")
                except OSError:
                    pass
            self.records[item] = record
            return True, f"{record.get('recorded', '?')} 判绿：{record.get('summary', '')}"
        latest = read_json(self._item_path(item))
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
        latest = read_json(self._item_path(item))
        if latest is None:
            return False, NO_RECORD_REASON
        self.records[item] = latest
        return True, f"{latest.get('recorded', '?')} 判绿（没比键）：{latest.get('summary', '')}"

    def record(self, item, components, summary, run="", details=None):
        """判绿之后调：组成清单、键那一份与这一项最近那一份各原子写一次。写不成交一句原因（不抛：后果只是下一趟这一项照判），写成交 None。
        有拒算的不写（交原因）。run 是跑法（判出这一份的那条命令或那几样参数），details 是调用方要随判定存的几样数。"""
        if self.directory is None:
            return None
        refused = refused_component(components, self.judging_units)
        if refused is not None:
            return f"拒算（{refused[0]}），不记"
        key = key_of(components, self.judging_units)
        compositions = {}
        try:
            if "composition-not-recorded" not in BREAK:
                for name, value in list(components) + [(JUDGING_UNITS_COLUMN, self.judging_units)]:
                    if isinstance(value, UnitKey) and value.units:
                        digest = value.composition_digest()
                        compositions[name] = digest
                        if not os.path.exists(self._composition_path(digest)):
                            write_atomically(self._composition_path(digest), json.dumps(list(value.units), ensure_ascii=False) + "\n")
            text = json.dumps({"item": item, "key": key, "input_fingerprint": input_fingerprint(components),
                               "judging_units": stable_digest(self.judging_units),
                               "components": {name: stable_digest(value) for name, value in components}, "compositions": compositions,
                               "verdict": "green", "summary": summary, "details": dict(details or {}), "run": run,
                               "recorded": datetime.date.today().isoformat()}, ensure_ascii=False, sort_keys=True) + "\n"
            write_atomically(self._key_path(key), text)
            write_atomically(self._item_path(item), text)
        except OSError as error:
            return f"判定库 {self.directory} 写不进：{error}"
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


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if not arguments or arguments[0] not in ("unit-key", "judging-units", "lookup", "record"):
        print("  ✗ 用法：lib_verdict_library.py unit-key|judging-units <根> <入口>… | lookup|record <目录> <种类> <项> <组成表> … | --selftest")
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
            library = VerdictLibrary(directory, kind, judge, reuse="--no-reuse" not in extra)
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
            raise ValueError(f"record 认不出 {extra[position]}")
        failed = VerdictLibrary(directory, kind, judge).record(item, components, summary, run=run, details=details)
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
            library_directory = os.path.join(work, "library")
            judge_key = UnitKey(digest_text("判法"), ("fn\tjudge\tx.py:1-2\taaa",), ())
            store = VerdictLibrary(library_directory, "sample", judge_key)
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
            other_judge = VerdictLibrary(library_directory, "sample", UnitKey(digest_text("另一份判法"), ("fn\tjudge\tx.py:1-2\tbbb",), ()))
            reused, reason = other_judge.lookup("一项", components)
            cell("判法单元指纹变了整项现判、点名判法的单元", not reused and reason == f"{JUDGING_UNITS_COLUMN} 变了" and "改了 fn judge" in other_judge.detail("一项"),
                 f"{reused} {reason} 「{other_judge.detail('一项')}」")
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
            before = subprocess.run(command + ["lookup", library_directory, "shell", "direct", table], capture_output=True, text=True)
            recorded = subprocess.run(command + ["record", library_directory, "shell", "direct", table, "--summary", "判绿", "--run", "自证"], capture_output=True, text=True)
            after = subprocess.run(command + ["lookup", library_directory, "shell", "direct", table], capture_output=True, text=True)
            cell("命令行 unit-key、lookup、record 一个来回", done.returncode == 0 and before.returncode == 1 and before.stdout.startswith("现判\t无记录")
                 and recorded.returncode == 0 and after.returncode == 0 and after.stdout.startswith("复用\t"),
                 f"unit-key 退 {done.returncode}（{done.stderr.strip()[-200:]}），lookup 退 {before.returncode}「{before.stdout.strip()}」，"
                 f"record 退 {recorded.returncode}「{recorded.stdout.strip()}」，再 lookup 退 {after.returncode}「{after.stdout.strip()}」")
        except (AssertionError, ValueError) as error:
            cell("自证的现场", False, str(error))
        shutil.rmtree(root, ignore_errors=True)

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        for item, cell_label in (("unit-key-from-raw-file", "只改注释单元键不变"), ("refusal-reused", "拒算不复用、不记"),
                                 ("composition-not-recorded", "作废时点名变了的单元"), ("judging-files-included", "判法单元指纹不含库自己")):
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
