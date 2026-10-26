#!/usr/bin/env python3
# admission: always 判的是这一次给的根下此刻的 Rust 源码与入口登记表，几秒算完；上一次算出来的交集不替这一次作保
# run-condition: none 只读根下的 Rust 源码与 .claude/gate.d/checker-direction-entries.tsv，除了 python3 之外没有环境要求
# gate-similar: checker-independence-and-sync.sh 它的 checker-implementation-disjoint 格判 crate 粒度的依赖闭包交集（C12（增量语义共用）），罩不到同一个 crate 里的两个方向；这里判 singlefs-checker 里两个方向的函数单元闭包交集，作为同一道的另一格接进去
# gate-similar: lib_function_units.py 闭包由它收（Rust 八条闭包规则），这里不另写解析器，只在它收出的两份闭包上求交、按登记表分类
"""池级 checker 两个方向共享什么（C410（checker 两个方向共享什么没有检查））：扫描式方向调用的函数单元集合与遍历式方向调用的函数单元集合，
交集只许落在格式解析、报告通道与常量里；越界、或取不到调用图即判红。门禁 checker-independence-and-sync 的 checker-direction-overlap 格调它。
依据：D13（验证路线） 已定项 12「两者对同一条不变量独立判断，共享的只有格式解析」。扫描式方向的边界照用户 2026-10-26 定的
「连 walk.rs 扫单元头那几个函数一起算，另建调用图」：扫描重建模块 crates/singlefs-checker/src/rebuild.rs，连同 walk.rs 里扫单元头、
按已发布谓词滤、喂 I-1.8（归并后版本全序） 与扫描重建的那几个函数。

做法：
  调用图取 scripts/gate/lib_function_units.py 的 Rust 闭包（RustWorkspace、parse_entry、find_entry、rust_closure_of；闭包规则在它文件头），
  不另写解析器。两个方向的入口、切点、局部绑定与允许共享的格式解析单元登记在 .claude/gate.d/checker-direction-entries.tsv（写法在它文件头）。
  ① 扫描式方向的闭包是各 scan-entry 单元闭包的并。
  ② 遍历式方向的闭包是各 traversal-entry 单元闭包的并；收之前把 traversal-cut 登记的（单元, 名字）从那个单元的引用里摘掉
     （编排函数一次跑两个方向、又调比对点：这几处调扫描式方向与比对点不算遍历式在调）。摘的只是那一个单元里的那个名字，
     遍历式别的单元点到同一个名字照连。local-binding 登记的（单元, 名字）两个方向都摘：那个名字在那个单元里是 let 绑定的局部量，
     库按名字把它连到了同名函数。
  比对点（comparison，今天是 I-1.13（扫描重建与遍历方向的索引相同） 那一格）拿两个方向的结果比，不属哪一个方向：编排函数调它那一处照 traversal-cut 切掉，
     它调两个方向的那几处（名字是某个 traversal-entry 或 scan-entry 单元的名字）自动摘掉，再收它自己的闭包；比对点那一侧要的遍历式读法
     （读出最新根的三张表）登记成 traversal-entry。比对点自己的闭包不判，逐个列出与扫描式共用的、只在比对点里的函数单元（与遍历式共用的只报个数）。
  ③ 交集里逐个单元分：
     函数单元（fn、macro_rules、item_macro）  只许格式解析与报告通道：落在 crates/singlefs-format/ 下的（整个包），或登记表 shared-format 点名的
                                             crates/singlefs-checker/src/lib.rs 单元，或登记表 reporting-channel 点名的
                                             crates/singlefs-checker/src/image.rs 里 Judgements 的方法；别的判越界
                                             （报告通道：两个方向往同一个判定累加器报判定，传的是判定结果，不是判定逻辑，不算判定共享；
                                             用户 2026-10-26 定「Judgements::judge 条款写一句报告通道不算判定共享」）
     没有函数体的 trait 方法声明               不判：只是签名，调到的是实现那一边（今天是读盘接口 ImageReader 那几个）
     常量（const、static）                    许
     别的（struct、enum、union、type、trait 头、impl 头、use、mod、mod_head、extern_crate、doctest、数据单元）  不判：不是被调用的函数，只报个数
  取不到调用图（判红）：登记表不在或读不出、哪一个方向一个入口都没有、入口找不到或命中不止一个、库报算不出、闭包里有拒算行（认不出的写法）。
  登记表核法（判红）：四列、角色认得、理由至少 8 个字、同一行不登两次；traversal-cut 的名字要是某个 scan-entry 或 comparison 单元的名字、
  那个单元里真点到它、那个单元在遍历式方向的闭包里；comparison 单元至少被一行 traversal-cut 切到；local-binding 的单元里要有 let [mut] <名字>；
  shared-format 只许登 crates/singlefs-checker/src/lib.rs 里的单元；reporting-channel 只许登 crates/singlefs-checker/src/image.rs 里
  `impl Judgements`（自身 impl，不是 trait impl）的方法；每一行的单元都要在。
管不到的：数据流（遍历式读出的实例表行经 PublishedPredicate 交给扫描式用，是传值不是调用，这里看不见，交集里只见得到 InstanceTableRow 这个类型）；
  库按名字连边的过近似（同名方法、同名局部量连上就算调用；误连的照 local-binding 登记）；trait 对象背后的实现落在 crate 外的（读盘接口的实现在 harness 里）。

用法：
  checker-direction-overlap.py check [根] [--show-all]   判；--show-all 另把交集里的常量、类型与结构逐个列出
  checker-direction-overlap.py --selftest                 造出来的小仓里判每一格；CHECKER_DIRECTION_OVERLAP_BREAK=<项> 时点名的格必须红
退出码：0 交集都在格式解析与常量里；1 越界、取不到调用图或登记表有问题（逐个列）；2 用法错；77 根下没有 crates/（本次无对象可判）。

弄坏开关（只给 --selftest 证红用；平常跑设了它退 2、点名它只在自证里认；值认不得退 2）。逐项写开关名、弄坏哪一处、打开后自证哪几格红
（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  CHECKER_DIRECTION_OVERLAP_BREAK=overlap-unjudged  classify() 不判越界：交集里格式解析之外的函数单元照许
      「✗ 自证 scan-calls-a-traversal-judge：」「✗ 自证 shared-macro：」「✗ 自证 cut-elsewhere-still-counts：」「✗ 自证 local-binding-unregistered：」
      「✗ 自证 reporting-channel-unregistered：」
  CHECKER_DIRECTION_OVERLAP_BREAK=macros-unjudged  classify() 把 macro_rules、item_macro 当结构不判
      「✗ 自证 shared-macro：」
  CHECKER_DIRECTION_OVERLAP_BREAK=closure-failures-accepted  direction_closure() 找不到的入口跳过、拒算行不当问题
      「✗ 自证 missing-entry：」「✗ 自证 closure-refusal：」
  CHECKER_DIRECTION_OVERLAP_BREAK=table-unchecked  read_table() 与 shared_format_units() 不核登记表（列数、角色、理由、shared-format 落在哪、指不指得到）
      「✗ 自证 table-malformed：」「✗ 自证 stale-shared-format：」「✗ 自证 reporting-channel-missing：」
  CHECKER_DIRECTION_OVERLAP_BREAK=cuts-unchecked  apply_cuts() 与 check() 不核切点（名字是不是扫描式入口或比对点、单元里点没点到、单元在不在遍历式闭包里、
      比对点有没有被切到）
      「✗ 自证 cut-rows-checked：」「✗ 自证 comparison-uncut：」
  CHECKER_DIRECTION_OVERLAP_BREAK=cuts-applied-everywhere  apply_cuts() 把切点的名字从每个单元里摘掉，不只摘登记的那一个单元
      「✗ 自证 cut-rows-checked：」（先摘掉的名字让后一行切点核成「没点到」）「✗ 自证 cut-elsewhere-still-counts：」
  CHECKER_DIRECTION_OVERLAP_BREAK=local-bindings-unverified  apply_local_bindings() 不核单元里有没有 let <名字>
      「✗ 自证 local-binding-unverified：」
  CHECKER_DIRECTION_OVERLAP_BREAK=local-bindings-ignored  apply_local_bindings() 登记了也不摘
      「✗ 自证 local-binding-registered：」
  CHECKER_DIRECTION_OVERLAP_BREAK=declarations-judged  classify() 没有函数体的 trait 方法声明照函数单元判
      「✗ 自证 format-only-green：」「✗ 自证 scan-calls-a-traversal-judge：」「✗ 自证 shared-macro：」「✗ 自证 comparison-helper-shared-with-scan：」
      「✗ 自证 local-binding-registered：」「✗ 自证 reporting-channel-green：」「✗ 自证 reporting-channel-unregistered：」
      （scan-calls-a-traversal-judge 与 shared-macro 是越界的个数多出读盘接口那一个声明）
  CHECKER_DIRECTION_OVERLAP_BREAK=comparison-closure-unlisted  check() 不列比对点自己的闭包（与扫描式共用的、只在比对点里的函数单元）
      「✗ 自证 comparison-helper-shared-with-scan：」
  CHECKER_DIRECTION_OVERLAP_BREAK=reporting-channel-anywhere  reporting_channel_units() 不核 reporting-channel 登的单元落在哪（不看文件、不看是不是 Judgements 的方法）
      「✗ 自证 reporting-channel-elsewhere：」
  CHECKER_DIRECTION_OVERLAP_BREAK=reporting-channel-missing-accepted  check() 对 reporting-channel 登的单元找不到不记问题
      「✗ 自证 reporting-channel-missing：」
"""
import contextlib
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
sys.path.insert(0, SCRIPT_DIRECTORY)
from preflight import preflight  # noqa: E402
import lib_function_units as function_units  # noqa: E402

ENTRIES_TABLE = ".claude/gate.d/checker-direction-entries.tsv"
FORMAT_PACKAGE_DIRECTORY = "crates/singlefs-format/"
FORMAT_PARSING_FILE = "crates/singlefs-checker/src/lib.rs"
REPORTING_CHANNEL_FILE = "crates/singlefs-checker/src/image.rs"
REPORTING_CHANNEL_TYPE = "Judgements"
ROLES = ("traversal-entry", "scan-entry", "comparison", "traversal-cut", "local-binding", "shared-format", "reporting-channel")
ROLES_NAMING_A_NAME = ("traversal-cut", "local-binding")
FUNCTION_KINDS = ("fn", "macro_rules", "item_macro")
MACRO_KINDS = ("macro_rules", "item_macro")
CONSTANT_KINDS = ("const", "static")
MINIMUM_REASON_CHARACTERS = 8
IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
USAGE_EXIT = 2
NOT_RUN_EXIT = 77
BREAK_VARIABLE = "CHECKER_DIRECTION_OVERLAP_BREAK"
KNOWN_BREAK_ITEMS = ("overlap-unjudged", "macros-unjudged", "closure-failures-accepted", "table-unchecked", "cuts-unchecked",
                     "cuts-applied-everywhere", "local-bindings-unverified", "local-bindings-ignored", "declarations-judged",
                     "comparison-closure-unlisted", "reporting-channel-anywhere", "reporting-channel-missing-accepted")
HOWTO = ("越界的单元：属格式解析的搬进 crates/singlefs-checker/src/lib.rs、登记表加一行 shared-format（认不认它属格式解析由用户定，D13（验证路线） 已定项 12 只许共享格式解析），"
         "是往判定累加器 Judgements 报判定的方法登 reporting-channel，不属的在扫描式那一侧自己写一份；库按名字误连的（同名局部量）照 local-binding 登记；取不到调用图与登记表的问题照点名的那一行改；"
         "看一条边从哪来：python3 scripts/gate/lib_function_units.py closure <根> <入口> --explain")


class UsageError(Exception):
    pass


def break_items():
    items = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in items if item not in KNOWN_BREAK_ITEMS]
    if unknown:
        raise UsageError(f"{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
    return items


def broken(item):
    return item in break_items()


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，交回退出码 2（参数解析之前判）；否则交 None。"""
    value = os.environ.get(BREAK_VARIABLE, "")
    if not value or arguments == ["--selftest"]:
        return None
    print(f"  ✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没判")
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/gate/checker-direction-overlap.py --selftest")
    return USAGE_EXIT


class Row:
    def __init__(self, line, role, entry, name, reason):
        self.line, self.role, self.entry, self.name, self.reason = line, role, entry, name, reason
        self.unit, self.target = None, None

    def where(self):
        return f"{ENTRIES_TABLE}:{self.line}"


def read_table(root, problems):
    """登记表 → [Row]；表不在或读不出交 None。核法见文件头「登记表核法」。"""
    path = os.path.join(root, ENTRIES_TABLE)
    if not os.path.isfile(path):
        problems.append(f"取不到调用图：{ENTRIES_TABLE} 不在，两个方向的入口没有登记")
        return None
    unchecked = broken("table-unchecked")
    rows, seen = [], set()
    try:
        with open(path, encoding="utf-8") as handle:
            text_rows = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        problems.append(f"取不到调用图：{ENTRIES_TABLE} 读不出（{error}）")
        return None
    for number, text in enumerate(text_rows, 1):
        if not text.strip() or text.lstrip().startswith("#"):
            continue
        fields = text.split("\t")
        if len(fields) != 4:
            if not unchecked:
                problems.append(f"登记表 {ENTRIES_TABLE}:{number} 是 {len(fields)} 列，要四列：角色、单元、点名的名字（不用的写 -）、# 理由，制表符分隔")
            continue
        role, entry, name, reason = fields
        if role not in ROLES:
            if not unchecked:
                problems.append(f"登记表 {ENTRIES_TABLE}:{number} 的角色「{role}」认不得，只认 {'、'.join(ROLES)}")
            continue
        reason_text = reason[1:].strip() if reason.startswith("#") else ""
        if not unchecked and len(reason_text) < MINIMUM_REASON_CHARACTERS:
            problems.append(f"登记表 {ENTRIES_TABLE}:{number} 的理由要写成「# <理由>」、至少 {MINIMUM_REASON_CHARACTERS} 个字")
        if not unchecked and role in ROLES_NAMING_A_NAME and not IDENTIFIER.match(name):
            problems.append(f"登记表 {ENTRIES_TABLE}:{number} 是 {role}，第三列要写一个名字（标识符），现在是「{name}」")
        if not unchecked and role not in ROLES_NAMING_A_NAME and name != "-":
            problems.append(f"登记表 {ENTRIES_TABLE}:{number} 是 {role}，第三列写 -，现在是「{name}」")
        key = (role, entry, name)
        if key in seen and not unchecked:
            problems.append(f"登记表 {ENTRIES_TABLE}:{number} 与前面一行相同（{role} {entry} {name}），同一行只登一次")
        seen.add(key)
        rows.append(Row(number, role, entry, name, reason_text))
    return rows


def resolve(workspace, root, row, problems):
    """登记行的单元 → 填进 row.unit、row.target；找不到的记一条问题。"""
    try:
        language, path, qualified = function_units.parse_entry(root, row.entry)
        if language != "rust" or path is None or not qualified:
            raise function_units.UsageError(f"要写 <.rs 文件>::<限定名>，现在是 {row.entry}")
        row.unit, row.target = workspace.find_entry(path, qualified, None)
    except (function_units.UsageError, function_units.FunctionUnitsError) as error:
        row.unit = None
        return f"{row.where()} {row.role} {row.entry}：{error}"
    return None


def unit_place(unit):
    return f"{unit.kind} {unit.file}:{unit.first_line}-{unit.last_line} {unit.output_name()}"


def short_place(unit):
    return f"{os.path.basename(unit.file)}:{unit.first_line} {unit.name}"


def referrers_of(members, targets):
    """→ {id(目标): 闭包里点到它名字的单元，按 sort_key 取最前一个}：报「经哪个单元收进」用。不取库的 via（先到的那一条边随集合的遍历次序变，
    两趟打出来的不一样）；摘名字（切点、局部绑定）之后、还回去之前调。"""
    found = {}
    for unit in sorted(members.values(), key=function_units.Unit.sort_key):
        if unit.kind not in FUNCTION_KINDS + CONSTANT_KINDS:
            continue
        paths = {pair[1] for pair in (unit.qualified or ())}
        for target in targets:
            # 方法（挂在 impl、trait 下的）按 .名( 与 路径::名 认，自由函数按裸名与路径认：结构体里同名的字段不算点到它
            names = (set(unit.methods or ()) if target.container is not None else set(unit.plain or ())) | paths
            if id(target) not in found and target is not unit and target.short in names:
                found[id(target)] = unit
    return found


def direction_closure(workspace, rows, what, problems):
    """几条入口闭包的并 → ({id: 单元}, {id: 收进它的第一条入口行})；找不到的入口与拒算记问题（「取不到调用图」）。"""
    members, origins = {}, {}
    accepted = broken("closure-failures-accepted")
    for row in rows:
        if row.unit is None:
            continue
        try:
            closure = function_units.rust_closure_of(workspace, row.unit, row.target)
        except (function_units.UsageError, function_units.FunctionUnitsError) as error:
            problems.append(f"取不到调用图：{what}入口 {row.entry}（{row.where()}）的闭包算不出：{error}")
            continue
        if not accepted:
            for refusal in closure.refusal_list():
                problems.append(f"取不到调用图：{what}入口 {row.entry} 的闭包里有拒算 {refusal.form} {refusal.file}:{refusal.line} {refusal.snippet}")
        for unit in closure.order:
            if id(unit) not in members:
                members[id(unit)] = unit
                origins[id(unit)] = row
    return members, origins


def unit_references_name(unit, name):
    return name in (unit.plain or ()) or name in (unit.methods or ()) or any(pair[1] == name for pair in (unit.qualified or ()))


def strip_name(unit, name, saved):
    """把名字从单元的引用里摘掉（裸名、方法名、限定路径的末段），原样记进 saved 以便摘完还回去。"""
    if id(unit) not in saved:
        saved[id(unit)] = (unit, unit.plain, unit.methods, unit.qualified, unit.references)
    unit.plain = set(unit.plain or ()) - {name}
    unit.methods = set(unit.methods or ()) - {name}
    unit.qualified = {pair for pair in (unit.qualified or ()) if pair[1] != name}
    unit.references = unit.plain | unit.methods


def restore(saved):
    for unit, plain, methods, qualified, references in saved.values():
        unit.plain, unit.methods, unit.qualified, unit.references = plain, methods, qualified, references


def has_let_binding(workspace, unit, name):
    texts = workspace.files[unit.file].texts
    for index in range(unit.first_token, max(unit.first_token, unit.last_token - 1)):
        if texts[index] != "let":
            continue
        following = texts[index + 1] if index + 1 < unit.last_token else ""
        if following == "mut" and index + 2 < unit.last_token:
            following = texts[index + 2]
        if following == name:
            return True
    return False


def apply_local_bindings(workspace, rows, problems, saved):
    applied = 0
    for row in rows:
        if row.unit is None:
            continue
        if not broken("local-bindings-unverified") and not has_let_binding(workspace, row.unit, row.name):
            problems.append(f"登记表 {row.where()} local-binding {row.entry} 里没有 let {row.name}：它不是局部绑定，照调用算，这一行删掉")
            continue
        if broken("local-bindings-ignored"):
            continue
        strip_name(row.unit, row.name, saved)
        applied += 1
    return applied


def apply_cuts(workspace, rows, names_that_may_be_cut, problems, saved):
    unchecked = broken("cuts-unchecked")
    applied = []
    for row in rows:
        if row.unit is None:
            continue
        if not unchecked and row.name not in names_that_may_be_cut:
            problems.append(f"登记表 {row.where()} traversal-cut 的名字 {row.name} 不是哪个 scan-entry 或 comparison 单元的名字：切点只许切编排函数调扫描式方向与比对点的那几处")
            continue
        if not unchecked and not unit_references_name(row.unit, row.name):
            problems.append(f"登记表 {row.where()} traversal-cut：{row.entry} 里没点到 {row.name}，这一行切不到什么，删掉")
            continue
        if broken("cuts-applied-everywhere"):
            for rust_file in workspace.all_files():
                for unit in rust_file.units:
                    if unit_references_name(unit, row.name):
                        strip_name(unit, row.name, saved)
        else:
            strip_name(row.unit, row.name, saved)
        applied.append(row)
    return applied


def shared_format_units(rows, problems):
    """shared-format 那几行 → {id: 行}；只许 lib.rs 里的单元。"""
    allowed = {}
    for row in rows:
        if row.unit is None:
            continue
        if not broken("table-unchecked") and row.unit.file != FORMAT_PARSING_FILE:
            problems.append(f"登记表 {row.where()} shared-format {row.entry} 不在 {FORMAT_PARSING_FILE}：允许共享的格式解析只登那一份里的单元")
            continue
        allowed[id(row.unit)] = row
    return allowed


def is_method_of_the_reporting_channel(unit):
    """判定累加器 Judgements 自身 impl 里的方法（不是 trait impl，也不是自由函数）。"""
    owner = unit.container
    return (unit.kind == "fn" and unit.file == REPORTING_CHANNEL_FILE and owner is not None and owner.kind == "impl"
            and not owner.is_trait_impl and owner.self_types == {REPORTING_CHANNEL_TYPE})


def reporting_channel_units(rows, problems):
    """reporting-channel 那几行 → {id: 行}；只许 image.rs 里 Judgements 的方法。"""
    allowed = {}
    for row in rows:
        if row.unit is None:
            continue
        if not broken("reporting-channel-anywhere") and not is_method_of_the_reporting_channel(row.unit):
            problems.append(f"登记表 {row.where()} reporting-channel {row.entry} 不是 {REPORTING_CHANNEL_FILE} 里 {REPORTING_CHANNEL_TYPE} 的方法："
                            f"报告通道只登往判定累加器报判定的那几个方法")
            continue
        allowed[id(row.unit)] = row
    return allowed


def bodiless_trait_declaration(workspace, unit):
    if unit.kind != "fn" or unit.container is None or unit.container.kind != "trait":
        return False
    return workspace.files[unit.file].texts[unit.last_token - 1] == ";"


def classify(workspace, unit, allowed, channel):
    """交集里的一个单元 → format-package / format-registered / reporting-channel / declaration / constant / structure / outside。"""
    function_kinds = tuple(kind for kind in FUNCTION_KINDS if not (broken("macros-unjudged") and kind in MACRO_KINDS))
    if unit.kind in function_kinds:
        if unit.file.startswith(FORMAT_PACKAGE_DIRECTORY):
            return "format-package"
        if id(unit) in allowed:
            return "format-registered"
        if id(unit) in channel:
            return "reporting-channel"
        if not broken("declarations-judged") and bodiless_trait_declaration(workspace, unit):
            return "declaration"
        if broken("overlap-unjudged"):
            return "format-registered"
        return "outside"
    if unit.kind in CONSTANT_KINDS:
        return "constant"
    return "structure"


def check(root, show_all=False):
    """→ 退出码；判定逐行打到 stdout。"""
    if not os.path.isdir(os.path.join(root, "crates")):
        print(f"  ! {root} 下没有 crates/：两个方向的调用图这一格本次无对象可判")
        return NOT_RUN_EXIT
    problems = []
    rows = read_table(root, problems)
    if rows is None:
        return report(problems, None)
    try:
        workspace = function_units.RustWorkspace(root)
    except function_units.FunctionUnitsError as error:
        problems.append(f"取不到调用图：函数单元库建不起 {root} 的 Rust 工作区（{error}）")
        return report(problems, None)
    for row in rows:
        problem = resolve(workspace, root, row, problems)
        if problem is None:
            continue
        if row.role in ("traversal-entry", "scan-entry"):
            if not broken("closure-failures-accepted"):
                problems.append(f"取不到调用图：{problem}")
        elif not broken("table-unchecked") and not (row.role == "reporting-channel" and broken("reporting-channel-missing-accepted")):
            problems.append(f"登记表指不到单元：{problem}")
    by_role = {role: [row for row in rows if row.role == role] for role in ROLES}
    if not broken("table-unchecked"):
        for role, what in (("traversal-entry", "遍历式"), ("scan-entry", "扫描式")):
            if not by_role[role]:
                problems.append(f"取不到调用图：登记表里一行 {role} 都没有，{what}方向没有入口")
    saved = {}
    try:
        bindings = apply_local_bindings(workspace, by_role["local-binding"], problems, saved)
        scan, scan_origins = direction_closure(workspace, by_role["scan-entry"], "扫描式", problems)
        scan_names = {row.unit.short for row in by_role["scan-entry"] if row.unit is not None}
        entry_names = scan_names | {row.unit.short for row in by_role["traversal-entry"] if row.unit is not None}
        for row in by_role["comparison"]:
            if row.unit is not None:
                for name in entry_names:
                    if unit_references_name(row.unit, name):
                        strip_name(row.unit, name, saved)
        comparison, _comparison_origins = direction_closure(workspace, by_role["comparison"], "比对点", problems)
        comparison_names = {row.unit.short for row in by_role["comparison"] if row.unit is not None}
        cuts = apply_cuts(workspace, by_role["traversal-cut"], scan_names | comparison_names, problems, saved)
        traversal, traversal_origins = direction_closure(workspace, by_role["traversal-entry"], "遍历式", problems)
        shared = sorted((traversal[key] for key in traversal if key in scan), key=function_units.Unit.sort_key)
        scan_referrers, traversal_referrers = referrers_of(scan, shared), referrers_of(traversal, shared)
    finally:
        restore(saved)
    if not broken("cuts-unchecked"):
        for row in cuts:
            if id(row.unit) not in traversal:
                problems.append(f"登记表 {row.where()} traversal-cut：{row.entry} 不在遍历式方向的闭包里，切点只登遍历式方向里调扫描式或比对点的单元")
        cut_names = {row.name for row in cuts}
        for row in by_role["comparison"]:
            if row.unit is not None and row.unit.short not in cut_names:
                problems.append(f"登记表 {row.where()} comparison {row.entry} 没有一行 traversal-cut 切到它：比对点要从遍历式方向里切出来，不然它照遍历式算")
    allowed = shared_format_units(by_role["shared-format"], problems)
    channel = reporting_channel_units(by_role["reporting-channel"], problems)
    groups = {}
    for unit in shared:
        groups.setdefault(classify(workspace, unit, allowed, channel), []).append(unit)
    summary = {"scan_entries": len(by_role["scan-entry"]), "scan": len(scan), "traversal_entries": len(by_role["traversal-entry"]),
               "traversal": len(traversal), "cuts": len(cuts), "bindings": bindings, "shared": len(shared)}
    allowed_functions = groups.get("format-package", []) + groups.get("format-registered", [])
    print(f"  · 扫描式 {summary['scan_entries']} 个入口、闭包 {summary['scan']} 个单元；遍历式 {summary['traversal_entries']} 个入口"
          f"（切 {summary['cuts']} 处、局部绑定 {summary['bindings']} 处）、闭包 {summary['traversal']} 个单元；交集 {summary['shared']} 个单元")
    print(f"  · 交集里格式解析的函数单元 {len(allowed_functions)} 个：")
    for unit in sorted(allowed_functions, key=function_units.Unit.sort_key):
        how = "singlefs-format 包" if unit.file.startswith(FORMAT_PACKAGE_DIRECTORY) else f"登记表 {allowed[id(unit)].where()}" if id(unit) in allowed else "弄坏开关照许"
        print(f"      {unit_place(unit)}（{how}）")
    reporting = groups.get("reporting-channel", [])
    if by_role["reporting-channel"]:
        print(f"  · 交集里报告通道的函数单元 {len(reporting)} 个（往同一个判定累加器报判定，不算判定共享）：")
        for unit in sorted(reporting, key=function_units.Unit.sort_key):
            print(f"      {unit_place(unit)}（登记表 {channel[id(unit)].where()}）")
    declarations = groups.get("declaration", [])
    print(f"  · 交集里没有函数体的 trait 方法声明 {len(declarations)} 个（不判）：")
    for unit in declarations:
        print(f"      {unit_place(unit)}")
    constants, structures = groups.get("constant", []), groups.get("structure", [])
    kinds = {}
    for unit in structures:
        kinds[unit.kind] = kinds.get(unit.kind, 0) + 1
    print(f"  · 交集里的常量 {len(constants)} 个（许）、类型与结构 {len(structures)} 个（不判：{'、'.join(f'{kind} {count}' for kind, count in sorted(kinds.items())) or '无'}）")
    if show_all:
        for unit in constants + structures:
            print(f"      {unit_place(unit)}")
    unused = [row for key, row in list(allowed.items()) + list(channel.items()) if not any(id(unit) == key for unit in shared)]
    if unused:
        print(f"  · 登记了允许共享、今天不在交集里的 {len(unused)} 个：{'、'.join(row.entry.rsplit('::', 1)[-1] for row in unused)}")
    if by_role["comparison"] and not broken("comparison-closure-unlisted"):
        own = [unit for key, unit in comparison.items()
               if unit.kind in FUNCTION_KINDS and not unit.file.startswith(FORMAT_PACKAGE_DIRECTORY) and key not in allowed]
        with_scan = sorted((unit for unit in own if id(unit) in scan and id(unit) not in traversal), key=function_units.Unit.sort_key)
        with_traversal = [unit for unit in own if id(unit) in traversal]
        only = sorted((unit for unit in own if id(unit) not in scan and id(unit) not in traversal), key=function_units.Unit.sort_key)
        names = "、".join(row.unit.name for row in by_role["comparison"] if row.unit is not None)
        print(f"  · 比对点（{names}）自己的闭包 {len(comparison)} 个单元（不判）：与扫描式共用、遍历式没有的函数单元 {len(with_scan)} 个，"
              f"与遍历式共用的 {len(with_traversal)} 个，只在比对点里的 {len(only)} 个")
        for unit in with_scan:
            print(f"      与扫描式共用 {unit_place(unit)}")
        for unit in only:
            print(f"      只在比对点里 {unit_place(unit)}")
    outside = groups.get("outside", [])
    for unit in outside:
        scan_via, traversal_via = scan_referrers.get(id(unit)), traversal_referrers.get(id(unit))
        no_referrer = "没有函数点它的名字，库按结构或类型连进来"
        print(f"  ✗ 越界 {unit_place(unit)}（扫描式从 {scan_origins[id(unit)].unit.name} 收进、经 {short_place(scan_via) if scan_via else no_referrer}；"
              f"遍历式从 {traversal_origins[id(unit)].unit.name} 收进、经 {short_place(traversal_via) if traversal_via else no_referrer}）")  # gate-lint:detail
    return report(problems, summary, len(outside), len(allowed_functions), len(declarations), len(constants), len(structures), len(reporting))


def report(problems, summary, outside=0, allowed_functions=0, declarations=0, constants=0, structures=0, reporting=0):
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems or outside:
        scope = "" if summary is None else f"；扫描式闭包 {summary['scan']} 个单元、遍历式闭包 {summary['traversal']} 个、交集 {summary['shared']} 个"
        print(f"  ✗ 两个方向的函数单元交集越出格式解析与常量 {outside} 个，取不到调用图与登记表的问题 {len(problems)} 处（逐个列在上面{scope}）")  # gate-lint:summary
        print(f"     → 怎么办：{HOWTO}")
        return 1
    print(f"  ✓ 两个方向的函数单元交集都落在格式解析与常量里：交集 {summary['shared']} 个单元，函数单元 {allowed_functions} 个都是格式解析，"
          f"没有函数体的 trait 方法声明 {declarations} 个，常量 {constants} 个，类型与结构 {structures} 个（扫描式闭包 {summary['scan']} 个单元、遍历式闭包 {summary['traversal']} 个）"
          + (f"；另有报告通道的函数单元 {reporting} 个（往判定累加器报判定，不算判定共享）" if reporting else ""))
    return 0


# ── 自证 ────────────────────────────────────────────────────────────────────

FIXTURE_FORMAT_LIBRARY = """pub const SLOT_BYTES: u64 = 4096;

pub fn slot_offset(slot: u64) -> u64 {
    slot * SLOT_BYTES
}
"""

FIXTURE_CHECKER_LIBRARY = """pub mod walk;
mod rebuild;

pub(crate) const HEADER_MAGIC: &[u8] = b"SFSU";

pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

pub trait ImageReader {
    fn read(&self, offset: u64) -> Option<Vec<u8>>;
}
"""

FIXTURE_WALK = """use singlefs_format::{slot_offset, SLOT_BYTES};

use crate::rebuild::rebuild_index;
use crate::{read_u64, ImageReader, HEADER_MAGIC};

pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<String> {
    judge_pool_image(reader)
}

fn judge_pool_image(reader: &dyn ImageReader) -> Vec<String> {
    let mut verdicts = vec![judge_root_record(reader)];
    let scanned = scanned_units(reader);
    verdicts.push(judge_rebuild_agrees_with_the_walked_index(reader, &scanned));
    verdicts
}

fn root_record_is_well_formed(root: &[u8]) -> bool {
    root.len() >= 8 && read_u64(root, 0) != 0
}

fn judge_root_record(reader: &dyn ImageReader) -> String {
    let root = reader.read(slot_offset(0)).unwrap_or_default();
    // fixture:root-record-extra
    if root_record_is_well_formed(&root) {
        String::from("I-7.2 成立")
    } else {
        String::from("I-7.2 违例")
    }
}

fn walked_index(reader: &dyn ImageReader) -> Vec<u64> {
    let root = reader.read(SLOT_BYTES).unwrap_or_default();
    vec![read_u64(&root, 0)]
}

fn judge_rebuild_agrees_with_the_walked_index(reader: &dyn ImageReader, scanned: &[Vec<u8>]) -> String {
    // fixture:comparison-extra
    if walked_index(reader) == rebuild_index(scanned) {
        String::from("I-1.13 成立")
    } else {
        String::from("I-1.13 违例")
    }
}

fn unit_of_header(header: &[u8]) -> Option<Vec<u8>> {
    if header.len() < 8 || &header[..4] != HEADER_MAGIC {
        return None;
    }
    // fixture:unit-of-header-extra
    Some(header.to_vec())
}

fn scanned_units(reader: &dyn ImageReader) -> Vec<Vec<u8>> {
    (0..4)
        .filter_map(|slot| reader.read(slot_offset(slot)))
        .filter_map(|header| unit_of_header(&header))
        .collect()
}
"""

FIXTURE_REBUILD = """use crate::read_u64;

pub(crate) fn rebuild_index(scanned: &[Vec<u8>]) -> Vec<u64> {
    scanned.iter().map(|unit| read_u64(unit, 0)).collect()
}
"""

FIXTURE_TABLE_ROWS = [
    "traversal-entry\tcrates/singlefs-checker/src/walk.rs::check_pool_image\t-\t# 池级 checker 对外的入口",
    "traversal-entry\tcrates/singlefs-checker/src/walk.rs::walked_index\t-\t# 比对点那一侧遍历式读出的索引",
    "scan-entry\tcrates/singlefs-checker/src/walk.rs::scanned_units\t-\t# 扫单元头、按魔数滤",
    "scan-entry\tcrates/singlefs-checker/src/rebuild.rs::rebuild_index\t-\t# 扫描重建三张表那一步",
    "comparison\tcrates/singlefs-checker/src/walk.rs::judge_rebuild_agrees_with_the_walked_index\t-\t# 拿两个方向的结果比",
    "traversal-cut\tcrates/singlefs-checker/src/walk.rs::judge_pool_image\tscanned_units\t# 编排函数调扫描式方向",
    "traversal-cut\tcrates/singlefs-checker/src/walk.rs::judge_pool_image\tjudge_rebuild_agrees_with_the_walked_index\t# 编排函数调比对点",
    "shared-format\tcrates/singlefs-checker/src/lib.rs::read_u64\t-\t# 按小端读 8 字节",
]


def fixture_files(walk=FIXTURE_WALK, table_rows=None, extra=None):
    files = {
        "crates/singlefs-format/Cargo.toml": '[package]\nname = "singlefs-format"\n',
        "crates/singlefs-format/src/lib.rs": FIXTURE_FORMAT_LIBRARY,
        "crates/singlefs-checker/Cargo.toml": '[package]\nname = "singlefs-checker"\n\n[dependencies]\nsinglefs-format = { path = "../singlefs-format" }\n',
        "crates/singlefs-checker/src/lib.rs": FIXTURE_CHECKER_LIBRARY,
        "crates/singlefs-checker/src/walk.rs": walk,
        "crates/singlefs-checker/src/rebuild.rs": FIXTURE_REBUILD,
        ENTRIES_TABLE: "# 自证的小仓\n" + "\n".join(FIXTURE_TABLE_ROWS if table_rows is None else table_rows) + "\n",
    }
    files.update(extra or {})
    return files


def walk_with(root_record_extra="", unit_of_header_extra="", before_unit_of_header="", comparison_extra=""):
    text = FIXTURE_WALK.replace("    // fixture:root-record-extra\n", root_record_extra)
    text = text.replace("    // fixture:unit-of-header-extra\n", unit_of_header_extra)
    text = text.replace("    // fixture:comparison-extra\n", comparison_extra)
    return text.replace("fn unit_of_header(", before_unit_of_header + "fn unit_of_header(", 1)


FIXTURE_IMAGE = """pub struct Judgements {
    pub failed: u64,
}

impl Judgements {
    pub fn judge(&mut self, holds: bool) {
        if !holds {
            self.failed += 1;
        }
    }
}

pub struct Tally {
    pub count: u64,
}

impl Tally {
    pub fn count_once(&mut self) {
        self.count += 1;
    }
}
"""

REPORTING_CHANNEL_ROW = "reporting-channel\tcrates/singlefs-checker/src/image.rs::<impl Judgements>::judge\t-\t# 两个方向往同一个累加器报判定"


def reporting_walk():
    """两个方向都往判定累加器 Judgements 报判定的 walk.rs：遍历式判根记录、扫描式扫单元头，各调一次 judge。"""
    report_from_root = "    let mut sink = Judgements { failed: 0 };\n    sink.judge(root.len() >= 8);\n"
    report_from_header = "    let mut sink = Judgements { failed: 0 };\n    sink.judge(header.len() >= 8);\n"
    text = walk_with(root_record_extra=report_from_root, unit_of_header_extra=report_from_header)
    return "use crate::image::Judgements;\n" + text


def reporting_files(table_rows):
    return fixture_files(walk=reporting_walk(), table_rows=table_rows, extra={
        "crates/singlefs-checker/src/lib.rs": FIXTURE_CHECKER_LIBRARY.replace("mod rebuild;", "mod rebuild;\npub mod image;"),
        "crates/singlefs-checker/src/image.rs": FIXTURE_IMAGE,
    })


SHARED_MACRO = """macro_rules! verdict_text {
    ($name:expr, $holds:expr) => {
        if $holds {
            format!("{} 成立", $name)
        } else {
            format!("{} 违例", $name)
        }
    };
}

"""


def selftest_cases():
    """→ [(格名, 文件, 期望退出码, 输出里要有的几段, 输出里不许有的几段)]。"""
    rows = list(FIXTURE_TABLE_ROWS)
    return [
        ("format-only-green", fixture_files(), 0,
         ["✓ 两个方向的函数单元交集都落在格式解析与常量里", "singlefs_checker::read_u64（登记表", "singlefs_format::slot_offset（singlefs-format 包）",
          "没有函数体的 trait 方法声明 1 个", "<trait ImageReader>::read"], ["✗"]),
        ("scan-calls-a-traversal-judge",
         fixture_files(walk=walk_with(unit_of_header_extra="    if !root_record_is_well_formed(header) {\n        return None;\n    }\n")), 1,
         ["✗ 越界 fn crates/singlefs-checker/src/walk.rs:", "singlefs_checker::walk::root_record_is_well_formed（扫描式从 scanned_units 收进", "越出格式解析与常量 1 个"], []),
        ("shared-macro",
         fixture_files(walk=walk_with(root_record_extra="    let _verdict = verdict_text!(\"I-7.2\", root.len() >= 8);\n",
                                      unit_of_header_extra="    let _verdict = verdict_text!(\"单元头\", header.len() >= 8);\n",
                                      before_unit_of_header=SHARED_MACRO)), 1,
         ["✗ 越界 macro_rules", "verdict_text", "越出格式解析与常量 1 个"], []),
        ("missing-entry",
         fixture_files(table_rows=rows + ["scan-entry\tcrates/singlefs-checker/src/walk.rs::scanned_unit_headers\t-\t# 改了名而没跟着改的入口"]), 1,
         ["✗ 取不到调用图", "scanned_unit_headers"], []),
        ("closure-refusal",
         fixture_files(walk=walk_with(unit_of_header_extra="    let _note = SCAN_NOTE;\n",
                                      before_unit_of_header="const SCAN_NOTE: &str = include_str!(\"absent-scan-note.txt\");\n\n")), 1,
         ["✗ 取不到调用图：扫描式入口", "拒算 include-missing"], []),
        ("table-malformed",
         fixture_files(table_rows=rows + ["scan-entrance\tcrates/singlefs-checker/src/walk.rs::scanned_units\t-\t# 写错了的角色名字",
                                          "shared-format\tcrates/singlefs-checker/src/lib.rs::HEADER_MAGIC\t-\t# 短",
                                          "shared-format\tcrates/singlefs-checker/src/walk.rs::root_record_is_well_formed\t-\t# 走树判根记录的函数，不是格式解析"]), 1,
         ["角色「scan-entrance」认不得", "至少 8 个字", "root_record_is_well_formed 不在 crates/singlefs-checker/src/lib.rs"], []),
        ("stale-shared-format",
         fixture_files(table_rows=rows + ["shared-format\tcrates/singlefs-checker/src/lib.rs::read_u32\t-\t# 按小端读 4 字节"]), 1,
         ["登记表指不到单元", "read_u32"], []),
        ("cut-rows-checked",
         fixture_files(walk=walk_with(before_unit_of_header="fn rescan_everything(reader: &dyn ImageReader) -> Vec<Vec<u8>> {\n    scanned_units(reader)\n}\n\n"),
                       table_rows=rows + ["traversal-cut\tcrates/singlefs-checker/src/walk.rs::judge_rebuild_agrees_with_the_walked_index\twalked_index\t# 想把遍历式自己的函数也切掉",
                                          "traversal-cut\tcrates/singlefs-checker/src/walk.rs::judge_root_record\tscanned_units\t# 这个单元里根本没调它",
                                          "traversal-cut\tcrates/singlefs-checker/src/walk.rs::rescan_everything\tscanned_units\t# 没人调的单元"]), 1,
         ["名字 walked_index 不是哪个 scan-entry 或 comparison 单元的名字", "judge_root_record 里没点到 scanned_units", "rescan_everything 不在遍历式方向的闭包里"], []),
        ("comparison-uncut",
         fixture_files(table_rows=[row for row in rows if "\tjudge_rebuild_agrees_with_the_walked_index\t#" not in row]), 1,
         ["comparison crates/singlefs-checker/src/walk.rs::judge_rebuild_agrees_with_the_walked_index 没有一行 traversal-cut 切到它"], []),
        ("comparison-helper-shared-with-scan",
         fixture_files(walk=walk_with(before_unit_of_header="fn copy_is_readable(unit: &[u8]) -> bool {\n    unit.len() >= 8\n}\n\n",
                                      unit_of_header_extra="    if !copy_is_readable(header) {\n        return None;\n    }\n",
                                      comparison_extra="    let _readable = scanned.iter().filter(|unit| copy_is_readable(unit)).count();\n")), 0,
         ["✓ 两个方向的函数单元交集都落在格式解析与常量里", "与扫描式共用、遍历式没有的函数单元 1 个",
          "与扫描式共用 fn crates/singlefs-checker/src/walk.rs:", "singlefs_checker::walk::copy_is_readable",
          "只在比对点里 fn crates/singlefs-checker/src/walk.rs:", "singlefs_checker::walk::judge_rebuild_agrees_with_the_walked_index"], ["✗"]),
        ("cut-elsewhere-still-counts",
         fixture_files(walk=walk_with(root_record_extra="    let _count = scanned_units(reader).len();\n")), 1,
         ["✗ 越界 fn", "singlefs_checker::walk::scanned_units（", "singlefs_checker::walk::unit_of_header（"], []),
        ("local-binding-unregistered",
         fixture_files(walk=walk_with(root_record_extra="    let unit_of_header = |bytes: &[u8]| bytes.len();\n    let _width = unit_of_header(&root);\n")), 1,
         ["✗ 越界 fn", "singlefs_checker::walk::unit_of_header（"], []),
        ("local-binding-registered",
         fixture_files(walk=walk_with(root_record_extra="    let unit_of_header = |bytes: &[u8]| bytes.len();\n    let _width = unit_of_header(&root);\n"),
                       table_rows=rows + ["local-binding\tcrates/singlefs-checker/src/walk.rs::judge_root_record\tunit_of_header\t# 局部闭包与扫描式的函数同名"]), 0,
         ["✓ 两个方向的函数单元交集都落在格式解析与常量里", "局部绑定 1 处"], ["✗"]),
        ("local-binding-unverified",
         fixture_files(table_rows=rows + ["local-binding\tcrates/singlefs-checker/src/walk.rs::judge_pool_image\tread_u64\t# 其实没有这样一个局部量"]), 1,
         ["judge_pool_image 里没有 let read_u64"], []),
        ("reporting-channel-green",
         reporting_files(rows + [REPORTING_CHANNEL_ROW]), 0,
         ["✓ 两个方向的函数单元交集都落在格式解析与常量里", "交集里报告通道的函数单元 1 个",
          "singlefs_checker::image::<impl Judgements>::judge（登记表", "另有报告通道的函数单元 1 个"], ["✗"]),
        ("reporting-channel-unregistered",
         reporting_files(rows), 1,
         ["✗ 越界 fn crates/singlefs-checker/src/image.rs:", "singlefs_checker::image::<impl Judgements>::judge（扫描式从 scanned_units 收进",
          "越出格式解析与常量 1 个"], []),
        ("reporting-channel-elsewhere",
         reporting_files(rows + [REPORTING_CHANNEL_ROW,
                                 "reporting-channel\tcrates/singlefs-checker/src/walk.rs::root_record_is_well_formed\t-\t# 走树判根记录的函数，不是往累加器报判定",
                                 "reporting-channel\tcrates/singlefs-checker/src/image.rs::<impl Tally>::count_once\t-\t# image.rs 里的方法，可不是 Judgements 的"]), 1,
         ["reporting-channel crates/singlefs-checker/src/walk.rs::root_record_is_well_formed 不是 crates/singlefs-checker/src/image.rs 里 Judgements 的方法",
          "reporting-channel crates/singlefs-checker/src/image.rs::<impl Tally>::count_once 不是 crates/singlefs-checker/src/image.rs 里 Judgements 的方法"], []),
        ("reporting-channel-missing",
         reporting_files(rows + [REPORTING_CHANNEL_ROW,
                                 "reporting-channel\tcrates/singlefs-checker/src/image.rs::<impl Judgements>::judge_naming_the_slots\t-\t# 改了名而没跟着改的登记"]), 1,
         ["登记表指不到单元", "judge_naming_the_slots"], []),
    ]


def write_tree(root, files):
    for relative, text in files.items():
        path = os.path.join(root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)


def run_in_process(root):
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = check(root)
    return code, captured.getvalue()


def selftest():
    try:
        break_items()
    except UsageError as error:
        print(f"  ✗ {error}")
        print(f"     → 怎么办：{BREAK_VARIABLE} 只写文件头开关表里的项，逗号分隔")
        return USAGE_EXIT
    failures, cells = [], []
    work = tempfile.mkdtemp(prefix="checker-direction-overlap-selftest-")
    try:
        for name, files, expected_exit, wanted, unwanted in selftest_cases():
            cells.append(name)
            root = os.path.join(work, name)
            write_tree(root, files)
            try:
                code, output = run_in_process(root)
            except Exception as error:  # 自证里库抛出来的也算这一格判错，接着判别的格
                failures.append((name, f"抛了 {type(error).__name__}：{error}"))
                continue
            missing = [piece for piece in wanted if piece not in output]
            present = [piece for piece in unwanted if piece in output]
            if code != expected_exit or missing or present:
                failures.append((name, f"期望退 {expected_exit}、实测 {code}；缺 {missing or '无'}；多 {present or '无'}；输出：{output.strip()[:600]}"))
        cells.append("break-refused-outside-selftest")
        root = os.path.join(work, "format-only-green")
        probe = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "check", root, "--force", "--break-guard-probe-unknown-option"],
                               env=dict(os.environ, **{BREAK_VARIABLE: "overlap-unjudged"}), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=120)
        named = any(BREAK_VARIABLE in row and "自证" in row for row in (probe.stdout + probe.stderr).split("\n"))
        if probe.returncode != USAGE_EXIT or not named:
            failures.append(("break-refused-outside-selftest", f"期望退 {USAGE_EXIT} 并点名 {BREAK_VARIABLE} 只在自证里认，实测退 {probe.returncode}"))
        cells.append("no-crates-not-run")
        empty = os.path.join(work, "no-crates")
        os.makedirs(empty)
        code, output = run_in_process(empty)
        if code != NOT_RUN_EXIT:
            failures.append(("no-crates-not-run", f"根下没有 crates/ 期望退 {NOT_RUN_EXIT}，实测 {code}"))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for name, failure in failures:
        print(f"  ✗ 自证 {name}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("     → 怎么办：按上面判错的那一格修 check() 里对应的那一段；开着弄坏开关时点名的格判错是该有的，别的格判错是开关弄坏得太宽")
        return 1
    print(f"  ✓ 自证 {len(cells)} 格都判对（交集只在格式解析、报告通道、扫描式调遍历式判定函数、取不到闭包、登记表、切点、比对点与局部绑定、平常跑拒弄坏开关）")
    return 0


def main(arguments):
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    try:
        if arguments == ["--selftest"]:
            return selftest()
        if not arguments or arguments[0] != "check":
            raise UsageError("要写 check [根] [--show-all] 或 --selftest")
        rest = arguments[1:]
        show_all = "--show-all" in rest
        rest = [argument for argument in rest if argument != "--show-all"]
        unknown = [argument for argument in rest if argument.startswith("-")]
        if unknown or len(rest) > 1:
            raise UsageError(f"认不出的参数 {' '.join(unknown or rest)}")
        root = os.path.abspath(rest[0]) if rest else REPOSITORY_ROOT
        return check(root, show_all)
    except UsageError as error:
        print(f"  ✗ 用法错：{error}")
        print("     → 怎么办：checker-direction-overlap.py check [根] [--show-all]，或 --selftest；写法在文件头")
        return USAGE_EXIT


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
