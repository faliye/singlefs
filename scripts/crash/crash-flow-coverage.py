#!/usr/bin/env python3
# admission: always 判的是这一趟崩溃点重放的日志与此刻的登记表、StepKind 源码；门禁与入口每一趟现判，几秒跑完
# run-condition: none 只读日志、登记表与 Rust 源码；collect 的覆盖行存在 prune-server 里（连不上退 75，被判的根不是 git 工作树退 2）
"""崩溃流覆盖：覆盖表、ignore 开着的节点、评估 0 次的（流，不变量）对。判法只在这一份里，
门禁 crash-point-replay 的 crash-flow-coverage、crash-ignored-nodes、invariant-zero-evaluations 三格调它，
入口 scripts/crash/crash-point-replay.sh 每趟跑完拿它把覆盖表落进这一趟的日志目录。出口与条款：.claude/kb/milestone/03-third-txn.md「出口」的出口 4、
六·1、六·3；用户 2026-10-20 定的 K1–K7 原话在 records/2026-10-18-里程碑三收尾与近期欠账.md「用户 2026-10-19 弹窗定出口 4 七条」那一条。

读的两种行（crates 那一侧打，这里只读）：
  覆盖行  CRASH_FLOW_COVERAGE entry=<入口名> step=<StepKind 成员名> segment=<段序号> expansion=<expanded|ignored_point|registered_segment|not_expanded>
          四个字段的名字与次序逐字如上，多一个少一个、值不认得都记成「写法不对」。step 认成员名（UnitWrite），也认它 name() 的串（unit_write），换成成员名再判。
          行首可以粘着 libtest 的「test <名> ... 」。
  计数行  一行里带 I-x.y=<评估过>/<判违例>/<不适用>（crates/singlefs-verification/src/crash.rs 的 checker_counts_by_invariant）。
          只认这三段：带 I-x.y=<数>/<数> 的写法但不是三段的（两段、四段）整行记成「写法不对」，不按段数分别认。
          流 = <来源>:<这一行在第一个 I- 之前头一个全大写的词>（例 crash_enumeration_fixed_script_stream:CRASH_POINT_REPLAY_FIXED_SCRIPT；没有这样的词记 -；
          那之前另有 stream=<名> 或 flow=<名> 的接成 <词>/<名>，例 crash_enumeration_tree_split_streams:CRASH_POINT_REPLAY_TREE_SPLIT/AccountingLeafSplit），
          来源是测试目标名（tests/<名>.rs 去掉 .rs；库的单测记 lib，装置二进制记 bin/<名>），放量那一段记 crash-amplification。同一个流打了几行，次数相加。

记录（这份脚本自己的中间格式，一行一条，制表符隔开，行首可带空白）：
  CRASH_FLOW_RECORD coverage <来源> <入口> <步骤> <段> <展开方式>
  CRASH_FLOW_RECORD counts <来源> <标签> <I-x.y=a/b/c 用空格连起来>
  CRASH_FLOW_RECORD malformed <来源> <那一行>
  CRASH_FLOW_RECORD unknown <来源> <为什么这个测试二进制的行这一趟不全>
  日志里另有一行 CRASH_FLOW_RECORDS_FILE <路径> 的（门禁打的），run-directory 照路径把那份记录读进来。

子命令：
  extract --source <来源> <日志>…                              日志里的覆盖行与计数行转成记录；日志里本来就有的记录原样带上
  collect <根> <种类> <计划输出|-> <日志> [--pass <名>]
        门禁里按条复用的一趟（不带特性那一趟或带特性那一趟）的日志按测试二进制（「Running <目标根文件> (」那一行起的一段）转成记录。
        复用：按测试函数的单元键复用，复用的测试这一趟不跑、不打行。一个测试二进制里编得进的测试函数（计划输出里 judge、reuse、refused 的那几条）
        的（名字，键）排好序取 sha256 是它的组成键：这一趟整份现判（一条复用都没有）的，打出来的覆盖行与计数行照组成键存进 prune-server 里
        那个种类的判定库（scripts/gate/admission.py 的 VerdictLibrary(<根>, <种类>).store，族 unit-verdicts，相对路径 flow-lines/<组成键>.lines；
        键空间在 scripts/prune-server/prune_server_client.py 文件头「键空间」），本地不建、不读、不写；
        有复用的，库里有同一个组成键就取回存着的行（连同这一趟现判的那几条打的行）；没有就只交这一趟打的行，另交一条 unknown（这个测试二进制的行不全）。
        计划输出给 - 或没有一条计划行（按条复用算不出、全部现判）：日志里每一段都算整份现判、不存。<根> 不是 git 工作树（判定库没有名字）：admission.py 退 2，这里照它退。
        服务连不上退 75（客户端的退出码），不当「库里没有」交 unknown（弄坏开关 unreachable-treated-as-missing 下照库里没有办，改错的样子）。
  run-directory <目录>        崩溃点重放入口一趟的日志目录（test-functions.<目标>.log、test-functions.<目标>.two-hosts.log、
        pipeline.log、enumeration.<项>.log）转成记录
  check <coverage|ignored|zero-evaluations> <根> [--subset] <记录文件>…    判一格（下面「判法」），退出码见下
  table <根> (<记录文件>… | --run-directory <目录>)  打覆盖表、ignore 开着的节点、评估 0 次的对，Markdown
  baseline <ignored|zero-evaluations> <记录文件>…              照这一趟的记录打基准表的候选行（填 crash-ignored-nodes.tsv、invariant-zero-evaluations.tsv 用；
        0 次那张的理由列打成 <填理由>，要人逐行写，门禁不认这个占位）
  --selftest

判法：
  coverage（K1 乙）：行是入口登记表 .claude/gate.d/crash-entries.tsv（一行「<入口名><TAB><说明>」），列是 StepKind 的成员
    （crates/singlefs-verification/src/segments.rs 的 pub enum StepKind 静态读）。一格罩住 = 有一条覆盖行、步骤是它、expansion=expanded。
    判红：覆盖行写法不对；有覆盖行的入口不在登记表里；步骤不是 StepKind 的成员；登记的入口一条覆盖行都没有；StepKind 有成员没被任何一条流罩住；
    登记表不在、行写错（不是两列、名字重了、说明空着）、StepKind 读不出。后两样（入口没有行、成员没被罩住）只在行齐的时候判：
    带了 --subset（只跑了点名的几条用例）或有 unknown 记录（有测试二进制的行不全）时这两样记本次未跑、逐个列名，退 77（别的照判，有红照红）。
  ignored（K3 丙）：节点 = 一条 expansion 是 ignored_point、registered_segment、not_expanded 的覆盖行，身份是（去向、来源、入口、步骤、段）。
    基准 .claude/gate.d/crash-ignored-nodes.tsv 一行一个节点，五列照这个次序；这一趟出现了基准里没有的节点就红（逐条列）；比基准少的报出来、不红（基准可以删那几行）。
    基准不在、行写错判红；基准里有一行「# 初值：未填」（还没跑过一趟崩溃点重放来填）整格退 77，同时打这一趟的候选行。
  zero-evaluations（K4 甲）：计数行按（流，不变量）相加，评估过 0 次的对不在基准 .claude/gate.d/invariant-zero-evaluations.tsv
    （一行「<流><TAB><不变量><TAB><理由>」）里就红（逐条列）；基准每行理由不许空、不许是 <填理由>。基准里的对这一趟不是 0 了、
    或这一趟没出现，报出来、不红。一条计数行都没有先退 77（不读基准）；基准有「# 初值：未填」一行整格退 77、打候选行。
退出码：0 判绿；1 判红（逐处列、带出路）；2 用法错、读不了记录文件；77 本次未跑（上面各条写了哪几种）；75 collect 连不上 prune-server。
自证只在临时 prune-server 上跑（覆盖行存取那几格写服务）：开跑先调客户端 selftest_service_problem，不是临时服务就退 2、一格都不跑，不写真服务。起法：
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/crash/crash-flow-coverage.py --selftest

弄坏开关（只给 --selftest 证红用）CRASH_FLOW_COVERAGE_BREAK=<项>（几个用逗号连）。逐项写开关名、弄坏哪一处，下一行写打开后自证哪几格红
（格名是 selftest_cases() 里那一格的名字；门禁 crash-point-replay 的样本哪一份判错写在那道门禁文件头）：
  CRASH_FLOW_COVERAGE_BREAK=unregistered-entry-accepted  coverage 不判登记表外的入口
      「✗ 自检 coverage-unregistered-entry：」、「✗ 自检 coverage-flow-entry-judged-against-the-table：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=entry-absence-unchecked  coverage 不判登记的入口一条行都没有
      「✗ 自检 coverage-entry-without-lines：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=step-kind-coverage-unchecked  coverage 不判 StepKind 成员没被罩住
      「✗ 自检 coverage-new-step-kind-member：」、「✗ 自检 coverage-only-ignored-does-not-cover：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=any-expansion-covers  罩住照「出现」算，不看 expansion=expanded
      「✗ 自检 coverage-only-ignored-does-not-cover：」、「✗ 自检 table：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=incomplete-judged-as-complete  有 unknown 记录或 --subset 时照样判入口没有行与成员没被罩住
      「✗ 自检 coverage-incomplete-is-not-run：」、「✗ 自检 coverage-subset-is-not-run：」
      「✗ StepKind 成员 Barrier 没被任何一条流罩住（没有一条 step=Barrier expansion=ex」
      「✗ crash-flow-coverage：覆盖行 2 条，2 处不对（逐处列在上面）」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=ignored-growth-accepted  ignored 不判比基准多出来的节点
      「✗ 自检 ignored-one-more-node：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=unseeded-marker-ignored  两张基准不认「# 初值：未填」，照空基准判
      「✗ 自检 baselines-unseeded：」
      「✗ crash-ignored-nodes：这一趟 ignore 开着的节点 1 个（ignored_point 0、registered_se」
      「✗ …sample_stream:CHECKER…I-2.1 评估过 0 次，基准里没有这一行」
      「✗ …sample_stream:CRASH_POINT_REPLAY_SAMPLE_FAST…I-2.1 评估过 0 次，基准里没有」
      「✗ invariant-zero-evaluations：2 条流，评估 0 次的对 2 个，其中 2 个不在基准里」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=zero-growth-accepted  zero-evaluations 不判基准外的 0 次对
      「✗ 自检 zero-evaluations-new-zero：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=empty-reason-accepted  zero-evaluations 不判基准里理由空着的行
      「✗ 自检 zero-evaluations-empty-reason：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=reuse-without-recall  collect 照改前：复用的测试二进制不取回存着的行、也不报 unknown
      「✗ 自检 collect-recalls-reused-binary：」、「✗ 自检 collect-partial-reuse-is-unknown：」
      「✗ 自检 collect-unreachable-exits-75：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=unreachable-treated-as-missing  collect 连不上 prune-server 时照「库里没有存着的行」办（交 unknown、退 0），不退 75
      「✗ 自检 collect-unreachable-exits-75：」
      「✗ 自检 …格里有…格判错」
  CRASH_FLOW_COVERAGE_BREAK=count-shape-unchecked  计数行不核段数：不是三段的行不记成写法不对（两段的行被当成没有计数）
      「✗ 自检 extract-count-shape：」
      「✗ 自检 …格里有…格判错」
"""
import glob
import hashlib
import importlib.util
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
from preflight import preflight  # noqa: E402

BROKEN = set(filter(None, os.environ.get("CRASH_FLOW_COVERAGE_BREAK", "").split(",")))
ENTRIES_TABLE = ".claude/gate.d/crash-entries.tsv"
IGNORED_TABLE = ".claude/gate.d/crash-ignored-nodes.tsv"
ZERO_TABLE = ".claude/gate.d/invariant-zero-evaluations.tsv"
SEGMENTS_SOURCE = "crates/singlefs-verification/src/segments.rs"
ADMISSION_MODULE = os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py")
UNSEEDED_MARKER = "# 初值：未填"
REASON_PLACEHOLDER = "<填理由>"
EXPANSIONS = ("expanded", "ignored_point", "registered_segment", "not_expanded")
IGNORED_KINDS = EXPANSIONS[1:]
RECORD_PREFIX = "CRASH_FLOW_RECORD"
RECORDS_FILE_LINE = "CRASH_FLOW_RECORDS_FILE"
COVERAGE_MARK = re.compile(r"(?<![A-Za-z0-9_])CRASH_FLOW_COVERAGE(?=\s|$)")
COVERAGE_FIELDS = ("entry", "step", "segment", "expansion")
NAME_VALUE = re.compile(r"^[A-Za-z0-9_.:-]+$")
COUNT_TOKEN = re.compile(r"(?<![\w.-])(I-\d+(?:\.\d+)*)=(\d+)/(\d+)/(\d+)(?![\w/])")
COUNT_SHAPE = re.compile(r"(?<![\w.-])I-\d+(?:\.\d+)*=\d+(?:/\d+)+(?![\w/])")   # 看着像计数、写了几段不论；段数不是三段的整行记成写法不对
LABEL_TOKEN = re.compile(r"^[A-Z][A-Z0-9_]{2,}$")
RUNNING_LINE = re.compile(r"^\s*Running (?:unittests )?(\S+) \(")
PLAN_DECISIONS = ("judge", "reuse", "refused")
FLOW_LINES_DIRECTORY = "flow-lines"
SHOWN_AT_MOST = 20
CANDIDATES_SHOWN = 40
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77


# ── 行与记录 ─────────────────────────────────────────

def clean(text):
    return text.replace("\t", " ").replace("\n", " ").strip()


def record_line(fields):
    return RECORD_PREFIX + "\t" + "\t".join(clean(str(field)) for field in fields)


def parse_coverage(text):
    """覆盖行 → (入口, 步骤, 段, 展开方式)；写法不对 → None；不是覆盖行 → False。"""
    found = COVERAGE_MARK.search(text)
    if found is None:
        return False
    pairs = text[found.end():].split()
    if len(pairs) != len(COVERAGE_FIELDS):
        return None
    values = []
    for expected, pair in zip(COVERAGE_FIELDS, pairs):
        key, separator, value = pair.partition("=")
        if key != expected or not separator or not NAME_VALUE.match(value):
            return None
        values.append(value)
    entry, step, segment, expansion = values
    if not segment.isdigit() or expansion not in EXPANSIONS:
        return None
    return entry, step, segment, expansion


def count_tokens(text):
    return list(COUNT_TOKEN.finditer(text))


def counts_label(text, first_match):
    """第一个 I- 之前头一个全大写的词；那之前另有 stream=<名> 或 flow=<名> 的，接成 <词>/<名>（一个测试目标里几条流共用一个词时分得开）。"""
    words = text[:first_match.start()].split()
    label = next((word for word in words if LABEL_TOKEN.match(word)), "-")
    named = next((word.split("=", 1)[1] for word in words if re.match(r"^(stream|flow)=[A-Za-z0-9_.:-]+$", word)), "")
    return f"{label}/{named}" if named else label


def is_relevant(text):
    return parse_coverage(text) is not False or bool(COUNT_SHAPE.search(text))


def records_of_lines(lines, source):
    """日志行 → 记录（元组）。日志里本来就有的记录行原样带上。"""
    records = []
    for raw in lines:
        text = raw.rstrip("\n")
        stripped = text.lstrip(" ")
        if stripped.startswith(RECORD_PREFIX + "\t"):
            parsed = parse_record(stripped)
            if parsed is not None:
                records.append(parsed)
            continue
        coverage = parse_coverage(text)
        if coverage is None:
            records.append(("malformed", source, clean(text)[:300]))
            continue
        if coverage is not False:
            records.append(("coverage", source, *coverage))
            continue
        if "count-shape-unchecked" not in BROKEN and any(not COUNT_TOKEN.fullmatch(shape.group(0)) for shape in COUNT_SHAPE.finditer(text)):
            records.append(("malformed", source, clean(text)[:300]))
            continue
        matches = count_tokens(text)
        if matches:
            tokens = " ".join(match.group(0) for match in matches)
            records.append(("counts", source, counts_label(text, matches[0]), tokens))
    return records


RECORD_WIDTHS = {"coverage": 6, "counts": 4, "malformed": 3, "unknown": 3}


def parse_record(text):
    fields = text.rstrip("\n").split("\t")[1:]
    if not fields or fields[0] not in RECORD_WIDTHS or len(fields) != RECORD_WIDTHS[fields[0]]:
        return None
    return tuple(fields)


def read_records(paths):
    """记录文件 → 记录（元组）；读不了的文件抛 OSError。"""
    records = []
    for path in paths:
        with open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                stripped = line.lstrip(" ")
                if stripped.startswith(RECORD_PREFIX + "\t"):
                    parsed = parse_record(stripped)
                    if parsed is not None:
                        records.append(parsed)
    return records


def source_of_target_root(target_root):
    if target_root.startswith("tests/") and target_root.endswith(".rs"):
        return target_root[len("tests/"):-len(".rs")]
    if target_root == "src/lib.rs":
        return "lib"
    match = re.match(r"^src/bin/([^/]+?)(?:/main)?\.rs$", target_root)
    if match:
        return "bin/" + match.group(1)
    return target_root


def log_sections(lines):
    """cargo test 的输出按「Running <目标根文件> (」切段：{目标根文件: [行…]}，次序照出现。"""
    sections = {}
    current = None
    for raw in lines:
        match = RUNNING_LINE.match(raw)
        if match:
            current = match.group(1)
            sections.setdefault(current, [])
            continue
        if current is not None:
            sections[current].append(raw.rstrip("\n"))
    return sections


# ── 判定库里存的行（collect） ─────────────────────────

def flow_lines_store(root, kind):
    """这个种类在 prune-server 里的那一份（admission.py 的 VerdictLibrary(根, 种类).store，覆盖行放它的 flow-lines/ 底下）；
    <根> 不是 git 工作树、这棵树里没有客户端，admission.py 退 2（LibraryUnavailable）。建它不连服务。"""
    specification = importlib.util.spec_from_file_location("admission_for_flow_lines", ADMISSION_MODULE)
    admission = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(admission)
    return admission.VerdictLibrary(root, kind).store


def flow_lines_relative(key):
    return f"{FLOW_LINES_DIRECTORY}/{key}.lines"


def composition_key(target_root, rows):
    lines = sorted(f"{name}\t{key}\n" for name, _decision, key in rows)
    return hashlib.sha256(("flow-lines-composition\n" + target_root + "\n" + "".join(lines)).encode("utf-8", "surrogateescape")).hexdigest()


def read_plan(path):
    """计划输出（admission.py test-plan 那一种）→ {目标根文件: [(名字, 决定, 键)…]}，不编的那几条不算；一条计划行都没有交 None。"""
    if path == "-" or not os.path.isfile(path):
        return None
    by_root, any_row = {}, False
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if len(columns) < 7 or columns[1] not in PLAN_DECISIONS + ("not-compiled",):
                continue
            any_row = True
            if columns[1] in PLAN_DECISIONS:
                by_root.setdefault(columns[4], []).append((columns[0], columns[1], columns[2]))
    return by_root if any_row else None


def store_lines(store, key, lines):
    """覆盖行照组成键写进服务（一条一写帧，服务回确认才算数）；服务连不上客户端退 75。"""
    store.write(flow_lines_relative(key), "".join(line + "\n" for line in lines))


def stored_lines(store, key):
    """服务里这个组成键存着的行；没有交 None。服务连不上客户端退 75（弄坏开关 unreachable-treated-as-missing 下交 None）。"""
    if store is None:
        return None
    try:
        data = store.read(flow_lines_relative(key))
    except SystemExit as failure:   # 客户端的 PruneServerFailure 是 SystemExit：只在弄坏开关下拦
        if "unreachable-treated-as-missing" not in BROKEN or failure.code != 75:
            raise
        return None
    return None if data is None else data.decode("utf-8", "replace").splitlines()


def collect(root, kind, plan_path, log_path, pass_name):
    """交 (记录, 一句汇总)。"""
    with open(log_path, encoding="utf-8", errors="replace") as handle:
        sections = log_sections(handle)
    plan = read_plan(plan_path)
    records, fresh_complete, recalled, stored, incomplete = [], 0, 0, 0, []
    store = flow_lines_store(root, kind) if plan is not None else None
    roots = sorted(set(sections) | set(plan or {}))
    for target_root in roots:
        source = source_of_target_root(target_root)
        fresh = [line for line in sections.get(target_root, []) if is_relevant(line)]
        rows = (plan or {}).get(target_root, [])
        if plan is None or not rows:
            records += records_of_lines(fresh, source)
            fresh_complete += 1
            continue
        key = composition_key(target_root, rows)
        reused = [row for row in rows if row[1] == "reuse"]
        if not reused:
            if target_root not in sections:
                records.append(("unknown", source, f"{pass_name}：要现判 {len(rows)} 条，日志里却没有这个测试二进制那一段"))
                incomplete.append(source)
                continue
            records += records_of_lines(fresh, source)
            fresh_complete += 1
            if store is not None:
                store_lines(store, key, fresh)
                stored += 1
            continue
        if "reuse-without-recall" in BROKEN:
            records += records_of_lines(fresh, source)
            continue
        kept = stored_lines(store, key)
        if kept is not None:
            merged = list(dict.fromkeys(kept + fresh))
            records += records_of_lines(merged, source)
            recalled += 1
            continue
        records += records_of_lines(fresh, source)
        records.append(("unknown", source, f"{pass_name}：{len(reused)} 条测试函数复用、{len(rows) - len(reused)} 条现判，判定库里没有这一组键整份现判时存下的行"))
        incomplete.append(source)
    summary = (f"{pass_name}：{len(roots)} 个测试二进制，整份现判 {fresh_complete} 个（存进判定库 {stored} 个）、取回存着的行 {recalled} 个、"
               f"行不全 {len(incomplete)} 个{('（' + '、'.join(incomplete[:SHOWN_AT_MOST]) + '）') if incomplete else ''}"
               + ("" if plan is not None else "；没有计划行（按条复用算不出、全部现判），日志里每一段都算整份、不存"))
    return records, summary


def run_directory_records(directory):
    records = []
    for path in sorted(glob.glob(os.path.join(directory, "*.log"))):
        base = os.path.basename(path)
        if base == "test-functions-build.log":
            continue
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.readlines()
        by_test_functions = re.match(r"^test-functions\.(.+?)(?:\.two-hosts)?\.log$", base)
        if by_test_functions:
            records += records_of_lines(lines, by_test_functions.group(1))
        elif base == "pipeline.log":
            records += records_of_lines(lines, "crash-amplification")
        else:
            # enumeration.<项>.log 这一类（门禁的输出）只带它点名的记录文件与原样的记录行，别的行不当覆盖行读
            records += passed_through(lines)
        for line in lines:
            stripped = line.strip()
            if stripped.startswith(RECORDS_FILE_LINE + " "):
                referenced = stripped[len(RECORDS_FILE_LINE) + 1:].strip()
                if os.path.isfile(referenced):
                    records += read_records([referenced])
    return records


def passed_through(lines):
    found = []
    for line in lines:
        stripped = line.lstrip(" ")
        if stripped.startswith(RECORD_PREFIX + "\t"):
            parsed = parse_record(stripped)
            if parsed is not None:
                found.append(parsed)
    return found


# ── 登记表与 StepKind ───────────────────────────────────────

def table_rows(path):
    """表 → (行：[(行号, [列…])], 有没有「# 初值：未填」)；表不在抛 OSError。# 开头的行与空行不算；行尾「\t# 注释」去掉。"""
    rows, unseeded = [], False
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            text = line.rstrip("\n")
            if text.strip() == UNSEEDED_MARKER:
                unseeded = True
            if not text.strip() or text.lstrip().startswith("#"):
                continue
            columns = text.split("\t")
            while columns and columns[-1].lstrip().startswith("#"):
                columns.pop()
            rows.append((number, columns))
    return rows, unseeded


def read_entries(root):
    """入口登记表 → ({入口名: 说明}, 问题, 在不在)。"""
    path = os.path.join(root, ENTRIES_TABLE)
    try:
        rows, _unseeded = table_rows(path)
    except OSError:
        return {}, [], False
    entries, problems = {}, []
    for number, columns in rows:
        if len(columns) != 2 or not NAME_VALUE.match(columns[0].strip()) or not columns[1].strip():
            problems.append(f"{ENTRIES_TABLE}:{number} 不是「<入口名><TAB><说明>」（入口名只用字母、数字、_.:-，说明不许空）")
            continue
        name = columns[0].strip()
        if name in entries:
            problems.append(f"{ENTRIES_TABLE}:{number} 入口 {name} 登记了两回")
            continue
        entries[name] = columns[1].strip()
    if not entries and not problems:
        problems.append(f"{ENTRIES_TABLE} 一个入口都没登记")
    return entries, problems, True


def without_comments(text):
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    return re.sub(r"//[^\n]*", " ", text)


def read_step_kinds(root):
    """StepKind 的成员（声明序）与 {成员: name() 的串}；读不出交 ([], {}, 原因)。"""
    path = os.path.join(root, SEGMENTS_SOURCE)
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError as error:
        return [], {}, f"读不了 {SEGMENTS_SOURCE}（{error.strerror}）：StepKind 的成员读不出"
    code = without_comments(text)
    match = re.search(r"\benum\s+StepKind\s*\{", code)
    if match is None:
        return [], {}, f"{SEGMENTS_SOURCE} 里找不到 enum StepKind"
    depth, index = 1, match.end()
    while index < len(code) and depth:
        depth += {"{": 1, "}": -1}.get(code[index], 0)
        index += 1
    body = re.sub(r"#\[[^\]]*\]", " ", code[match.end():index - 1])
    members, depth, current = [], 0, ""
    for character in body + ",":
        if character in "({":
            depth += 1
        elif character in ")}":
            depth -= 1
        if character == "," and depth == 0:
            name = re.match(r"\s*([A-Z][A-Za-z0-9_]*)", current)
            if name:
                members.append(name.group(1))
            current = ""
        else:
            current += character
    if not members:
        return [], {}, f"{SEGMENTS_SOURCE} 的 enum StepKind 里一个成员都读不出"
    names = dict(re.findall(r"StepKind::([A-Za-z0-9_]+)\s*=>\s*\"([a-z0-9_]+)\"", code))
    return members, names, ""


# ── 判法 ─────────────────────────────────────────────

def report(problems, summary, remedy_lines):
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ {summary}")  # gate-lint:summary
    print("  → 怎么办：" + remedy_lines[0])
    for line in remedy_lines[1:]:
        print("            " + line)


def coverage_matrix(records, members, name_to_member):
    """{入口: {成员: 'expanded' | 'seen'}}。"""
    matrix = {}
    for record in records:
        if record[0] != "coverage":
            continue
        _kind, _source, entry, step, _segment, expansion = record
        member = step if step in members else name_to_member.get(step, step)
        cells = matrix.setdefault(entry, {})
        if expansion == "expanded" or "any-expansion-covers" in BROKEN:
            cells[member] = "expanded"
        else:
            cells.setdefault(member, "seen")
    return matrix


def check_coverage(root, records, subset):
    entries, problems, present = read_entries(root)
    if not present:
        problems.append(f"入口登记表 {ENTRIES_TABLE} 不在")
    members, names, step_problem = read_step_kinds(root)
    if step_problem:
        problems.append(step_problem)
    name_to_member = {name: member for member, name in names.items()}
    coverage = [record for record in records if record[0] == "coverage"]
    unknown = [record for record in records if record[0] == "unknown"]
    for record in records:
        if record[0] == "malformed":
            problems.append(f"{record[1]}：覆盖行或计数行写法不对（计数行要 I-x.y=<评估过>/<判违例>/<不适用> 三段）：{record[2]}")
    outside, bad_steps = {}, {}
    for record in coverage:
        _kind, source, entry, step, _segment, _expansion = record
        if entry not in entries:
            outside.setdefault(entry, set()).add(source)
        if members and step not in members and step not in name_to_member:
            bad_steps.setdefault(step, set()).add(source)
    if "unregistered-entry-accepted" not in BROKEN:
        for entry, sources in sorted(outside.items()):
            problems.append(f"入口 {entry} 不在登记表 {ENTRIES_TABLE} 里（打它的：{'、'.join(sorted(sources))}）")
    for step, sources in sorted(bad_steps.items()):
        problems.append(f"步骤 {step} 不是 StepKind 的成员（打它的：{'、'.join(sorted(sources))}；成员 {'、'.join(members)}）")
    matrix = coverage_matrix(records, members, name_to_member)
    missing_entries = [entry for entry in entries if entry not in matrix]
    uncovered = [member for member in members if not any(cells.get(member) == "expanded" for cells in matrix.values())]
    complete = (not subset and not unknown) or "incomplete-judged-as-complete" in BROKEN
    not_judged = []
    if complete:
        if "entry-absence-unchecked" not in BROKEN:
            problems += [f"登记的入口 {entry} 一条覆盖行都没有" for entry in missing_entries]
        if "step-kind-coverage-unchecked" not in BROKEN:
            problems += [f"StepKind 成员 {member} 没被任何一条流罩住（没有一条 step={member} expansion=expanded 的覆盖行）" for member in uncovered]
    else:
        not_judged = [f"入口 {entry} 没有行" for entry in missing_entries] + [f"成员 {member} 没被罩住" for member in uncovered]
    if problems:
        report(problems, f"crash-flow-coverage：覆盖行 {len(coverage)} 条，{len(problems)} 处不对（逐处列在上面）",
               [f"登记表外的入口：确是新入口就在 {ENTRIES_TABLE} 加一行「<入口名><TAB><说明>」，不是就改打行那一处的入口名；",
                "入口没有行、StepKind 成员没被罩住：给那个入口或那种提交步骤补一条崩溃流（入口一律经 CrashPointRecorder 设点、打覆盖行），或把那一段展开；",
                "写法不对的照 scripts/crash/crash-flow-coverage.py 文件头「覆盖行」改 crates 里打行那一处。"])
        return EXIT_RED
    if not complete:
        reasons = (["只跑了点名的几条（--subset）"] if subset else []) + [f"{record[1]} 的行不全（{record[2]}）" for record in unknown[:SHOWN_AT_MOST]]
        print(f"  ! 本次未跑（这不是通过）：crash-flow-coverage 的「入口没有行、成员没被罩住」两样没判：{'；'.join(reasons)}；"
              f"这两样里眼下看得到的 {len(not_judged)} 处：{'、'.join(not_judged[:SHOWN_AT_MOST]) or '无'}")
        print("     → 补全：把行不全的测试二进制整份现判一趟（它们的测试函数键对不上的那几条跑过、判绿记库之后，下一趟整份复用时就取得回），或不带 --item 跑全部用例")
        return EXIT_NOT_RUN
    print(f"  ✓ crash-flow-coverage：登记的 {len(entries)} 个入口都有覆盖行，StepKind {len(members)} 个成员都被罩住（覆盖行 {len(coverage)} 条）")
    return EXIT_GREEN


def node_of(record):
    _kind, source, entry, step, segment, expansion = record
    return (expansion, source, entry, step, segment)


def read_ignored_baseline(root):
    """→ (节点集合, 问题, 有没有初值未填的标记)；表不在抛 OSError。"""
    rows, unseeded = table_rows(os.path.join(root, IGNORED_TABLE))
    nodes, problems = set(), []
    for number, columns in rows:
        stripped = [column.strip() for column in columns]
        if len(stripped) != 5 or stripped[0] not in IGNORED_KINDS or not stripped[4].isdigit() or not all(stripped):
            problems.append(f"{IGNORED_TABLE}:{number} 不是「<去向><TAB><来源><TAB><入口><TAB><步骤><TAB><段>」"
                            f"（去向是 {'、'.join(IGNORED_KINDS)} 之一，段是数）")
            continue
        node = tuple(stripped)
        if node in nodes:
            problems.append(f"{IGNORED_TABLE}:{number} 这个节点登记了两回")
        nodes.add(node)
    return nodes, problems, unseeded


def ignored_nodes(records):
    return sorted({node_of(record) for record in records if record[0] == "coverage" and record[5] in IGNORED_KINDS})


def check_ignored(root, records):
    try:
        baseline, problems, unseeded = read_ignored_baseline(root)
    except OSError:
        report([], f"crash-ignored-nodes：基准 {IGNORED_TABLE} 不在",
               [f"建这张表（表头照 scripts/crash/crash-flow-coverage.py 文件头「判法」ignored 那一条），初值用 crash-flow-coverage.py baseline ignored <这一趟的记录> 打"])
        return EXIT_RED
    nodes = ignored_nodes(records)
    if unseeded and "unseeded-marker-ignored" not in BROKEN:
        print(f"  ! 本次未跑（这不是通过）：基准 {IGNORED_TABLE} 里还有「{UNSEEDED_MARKER}」：初值等一趟崩溃点重放之后填；这一趟的 {len(nodes)} 个节点照下面的行填进去（头 {CANDIDATES_SHOWN} 行；全部用 crash-flow-coverage.py baseline ignored <记录文件> 打）、删掉那一行")
        for node in nodes[:CANDIDATES_SHOWN]:
            print("    " + "\t".join(node))
        return EXIT_NOT_RUN
    if problems:
        report(problems, f"crash-ignored-nodes：基准 {IGNORED_TABLE} 有 {len(problems)} 行写错", ["照文件头那一句的写法改那几行"])
        return EXIT_RED
    extra = [node for node in nodes if node not in baseline]
    fewer = sorted(baseline - set(nodes))
    counts = {kind: sum(1 for node in nodes if node[0] == kind) for kind in IGNORED_KINDS}
    counts_text = "、".join(f"{kind} {count}" for kind, count in counts.items())
    if extra and "ignored-growth-accepted" not in BROKEN:
        report([f"基准里没有的节点：{chr(9).join(node)}" for node in extra],
               f"crash-ignored-nodes：这一趟 ignore 开着的节点 {len(nodes)} 个（{counts_text}），比基准 {len(baseline)} 行多出 {len(extra)} 个",
               ["ignore 开着的节点只减不增（出口 4）：把多出来的那几段展开（或别把点标成 ignore）；",
                f"确要新开一个不展开的点，先问用户点头，再把那一行加进 {IGNORED_TABLE}"])
        return EXIT_RED
    note = f"；比基准少 {len(fewer)} 个（基准里这几行可以删：{'、'.join(chr(32).join(node) for node in fewer[:SHOWN_AT_MOST])}）" if fewer else ""
    print(f"  ✓ crash-ignored-nodes：这一趟 ignore 开着的节点 {len(nodes)} 个（{counts_text}），都在基准 {len(baseline)} 行里{note}")
    return EXIT_GREEN


def zero_pairs(records):
    """{(流, 不变量): 评估过的次数之和}。"""
    totals = {}
    for record in records:
        if record[0] != "counts":
            continue
        _kind, source, label, tokens = record
        flow = f"{source}:{label}"
        for match in COUNT_TOKEN.finditer(tokens):
            pair = (flow, match.group(1))
            totals[pair] = totals.get(pair, 0) + int(match.group(2))
    return totals


def read_zero_baseline(root):
    rows, unseeded = table_rows(os.path.join(root, ZERO_TABLE))
    pairs, problems = {}, []
    for number, columns in rows:
        stripped = [column.strip() for column in columns]
        if len(stripped) != 3 or not stripped[0] or not re.match(r"^I-\d+(?:\.\d+)*$", stripped[1]):
            problems.append(f"{ZERO_TABLE}:{number} 不是「<流><TAB><不变量><TAB><理由>」（不变量写 I-x.y）")
            continue
        if (not stripped[2] or stripped[2] == REASON_PLACEHOLDER) and "empty-reason-accepted" not in BROKEN:
            problems.append(f"{ZERO_TABLE}:{number} 理由空着（{stripped[0]} 的 {stripped[1]}）：每一行写明这条流为什么评估不到它")
            continue
        key = (stripped[0], stripped[1])
        if key in pairs:
            problems.append(f"{ZERO_TABLE}:{number} 这一对登记了两回")
        pairs[key] = stripped[2]
    return pairs, problems, unseeded


def check_zero_evaluations(root, records):
    totals = zero_pairs(records)
    if not totals:
        print("  ! 本次未跑（这不是通过）：这一趟一条计数行（I-x.y=评估/违例/不适用）都没有，评估次数判不了")
        return EXIT_NOT_RUN
    try:
        baseline, problems, unseeded = read_zero_baseline(root)
    except OSError:
        report([], f"invariant-zero-evaluations：基准 {ZERO_TABLE} 不在",
               [f"建这张表（写法在 scripts/crash/crash-flow-coverage.py 文件头「判法」zero-evaluations 那一条），候选行用 crash-flow-coverage.py baseline zero-evaluations <这一趟的记录> 打"])
        return EXIT_RED
    zeros = sorted(pair for pair, total in totals.items() if total == 0)
    flows = sorted({flow for flow, _invariant in totals})
    if unseeded and "unseeded-marker-ignored" not in BROKEN:
        print(f"  ! 本次未跑（这不是通过）：基准 {ZERO_TABLE} 里还有「{UNSEEDED_MARKER}」：初值等一趟崩溃点重放之后填；"
              f"这一趟 {len(flows)} 条流、评估 0 次的对 {len(zeros)} 个，照下面的行填（头 {CANDIDATES_SHOWN} 行；全部用 crash-flow-coverage.py baseline zero-evaluations <记录文件> 打；理由逐行写）、删掉那一行")
        for flow, invariant in zeros[:CANDIDATES_SHOWN]:
            print(f"    {flow}\t{invariant}\t{REASON_PLACEHOLDER}")
        return EXIT_NOT_RUN
    if problems:
        report(problems, f"invariant-zero-evaluations：基准 {ZERO_TABLE} 有 {len(problems)} 行写错", ["照文件头那一句的写法改那几行，理由逐行写明"])
        return EXIT_RED
    new = [pair for pair in zeros if pair not in baseline]
    if new and "zero-growth-accepted" not in BROKEN:
        report([f"{flow}\t{invariant} 评估过 0 次，基准里没有这一行" for flow, invariant in new],
               f"invariant-zero-evaluations：{len(flows)} 条流，评估 0 次的对 {len(zeros)} 个，其中 {len(new)} 个不在基准里",
               ["一个（流，不变量）对评估 0 次不许涨（出口 4、六·3）：查那条流为什么评估不到它（判器漏判、流没走到那一类结构）；",
                f"确是这条流本来就评估不到的（新加的不变量在这条流上不适用这一类），把那一行连理由加进 {ZERO_TABLE}"])
        return EXIT_RED
    gone = sorted(key for key in baseline if totals.get(key, None) not in (0,))
    note = f"；基准里 {len(gone)} 行这一趟不是 0 或没出现（可以删或留着）" if gone else ""
    print(f"  ✓ invariant-zero-evaluations：{len(flows)} 条流、{len(totals)} 个（流，不变量）对，评估 0 次的 {len(zeros)} 个都在基准里{note}")
    return EXIT_GREEN


# ── 覆盖表与候选行 ──────────────────────────────────────

def table_text(root, records):
    entries, _problems, _present = read_entries(root)
    members, names, step_problem = read_step_kinds(root)
    name_to_member = {name: member for member, name in names.items()}
    out = ["# 崩溃流覆盖表", "",
           f"覆盖行 {sum(1 for record in records if record[0] == 'coverage')} 条、计数行 {sum(1 for record in records if record[0] == 'counts')} 条、"
           f"写法不对 {sum(1 for record in records if record[0] == 'malformed')} 条、行不全的测试二进制 {sum(1 for record in records if record[0] == 'unknown')} 个。"
           "罩住 = 有一条 expansion=expanded 的覆盖行；未展开 = 只有别的去向。"]
    if step_problem:
        out.append(f"StepKind 读不出：{step_problem}")
    matrix = coverage_matrix(records, members, name_to_member)
    out += ["", "| 入口 | " + " | ".join(members) + " |", "|---|" + "---|" * len(members)]
    for entry in list(entries) + sorted(set(matrix) - set(entries)):
        cells = matrix.get(entry, {})
        label = entry if entry in entries else f"{entry}（登记表外）"
        out.append(f"| {label} | " + " | ".join({"expanded": "罩住", "seen": "未展开"}.get(cells.get(member), "") for member in members) + " |")
    uncovered = [member for member in members if not any(cells.get(member) == "expanded" for cells in matrix.values())]
    out.append("")
    out.append(f"没被罩住的 StepKind 成员：{'、'.join(uncovered) or '无'}")
    out += ["", "## ignore 开着的节点", "", "| 去向 | 来源 | 入口 | 步骤 | 段 |", "|---|---|---|---|---|"]
    out += ["| " + " | ".join(node) + " |" for node in ignored_nodes(records)]
    totals = zero_pairs(records)
    by_flow = {}
    for (flow, invariant), total in sorted(totals.items()):
        if total == 0:
            by_flow.setdefault(flow, []).append(invariant)
    out += ["", "## 评估 0 次的（流，不变量）对", "", "| 流 | 0 次的不变量 |", "|---|---|"]
    out += [f"| {flow} | {' '.join(invariants)} |" for flow, invariants in by_flow.items()]
    unknown = [record for record in records if record[0] == "unknown"]
    if unknown:
        out += ["", "## 行不全的测试二进制", ""] + [f"- {record[1]}：{record[2]}" for record in unknown]
    return "\n".join(out) + "\n"


# ── 命令行 ─────────────────────────────────────────────

def usage(problem):
    print(f"  ✗ {problem}")
    print("  → 怎么办：子命令与参数见 scripts/crash/crash-flow-coverage.py 文件头「子命令」：extract、collect、run-directory、check、table、baseline、--selftest")
    return EXIT_USAGE


def option_value(arguments, name):
    if name in arguments:
        index = arguments.index(name)
        if index + 1 < len(arguments):
            value = arguments[index + 1]
            del arguments[index:index + 2]
            return value
        return None
    return ""


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if not arguments:
        return usage("缺子命令")
    command, rest = arguments[0], list(arguments[1:])
    try:
        if command == "extract":
            source = option_value(rest, "--source")
            if not source or not rest:
                return usage("extract 要 --source <来源> 与至少一份日志")
            for path in rest:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    for record in records_of_lines(handle, source):
                        print(record_line(record))
            return EXIT_GREEN
        if command == "collect":
            pass_name = option_value(rest, "--pass")
            if len(rest) != 4 or pass_name is None:
                return usage("collect 要 <根> <种类> <计划输出|-> <日志> [--pass <名>]")
            root, kind, plan_path, log_path = rest
            records, summary = collect(os.path.abspath(root), kind, plan_path, log_path, pass_name or "按条复用")
            for record in records:
                print(record_line(record))
            print(f"  · 覆盖行与计数行：{summary}", file=sys.stderr)
            return EXIT_GREEN
        if command == "run-directory":
            if len(rest) != 1:
                return usage("run-directory 要 <目录>")
            for record in run_directory_records(rest[0]):
                print(record_line(record))
            return EXIT_GREEN
        if command == "check":
            subset = "--subset" in rest
            rest = [argument for argument in rest if argument != "--subset"]
            if len(rest) < 3 or rest[0] not in ("coverage", "ignored", "zero-evaluations"):
                return usage("check 要 <coverage|ignored|zero-evaluations> <根> [--subset] <记录文件>…")
            cell, root, paths = rest[0], os.path.abspath(rest[1]), rest[2:]
            records = read_records(paths)
            if cell == "coverage":
                return check_coverage(root, records, subset)
            if cell == "ignored":
                return check_ignored(root, records)
            return check_zero_evaluations(root, records)
        if command == "table":
            directory = option_value(rest, "--run-directory")
            if not rest:
                return usage("table 要 <根> <记录文件>…，或 <根> --run-directory <目录>")
            root = os.path.abspath(rest[0])
            records = run_directory_records(directory) if directory else read_records(rest[1:])
            sys.stdout.write(table_text(root, records))
            return EXIT_GREEN
        if command == "baseline":
            if len(rest) < 2 or rest[0] not in ("ignored", "zero-evaluations"):
                return usage("baseline 要 <ignored|zero-evaluations> <记录文件>…")
            records = read_records(rest[1:])
            if rest[0] == "ignored":
                for node in ignored_nodes(records):
                    print("\t".join(node))
            else:
                for (flow, invariant), total in sorted(zero_pairs(records).items()):
                    if total == 0:
                        print(f"{flow}\t{invariant}\t{REASON_PLACEHOLDER}")
            return EXIT_GREEN
    except OSError as error:
        print(f"  ✗ 读不了 {error.filename}：{error.strerror}")
        print("  → 怎么办：看路径对不对、文件在不在；记录文件由 extract、collect 或 run-directory 打出来")
        return EXIT_USAGE
    return usage(f"认不出的子命令 {command}")


# ── 自证 ─────────────────────────────────────────────

SAMPLE_SEGMENTS = """/// 提交步骤的封闭枚举（样本）。
#[derive(Clone, Copy)]
pub enum StepKind {
    /// 整段清零
    ZeroFill,
    UnitWrite,
    // 注释里的 NotAMember,
    Barrier,
}

impl StepKind {
    pub const fn name(self) -> &'static str {
        match self {
            StepKind::ZeroFill => "zero_fill",
            StepKind::UnitWrite => "unit_write",
            StepKind::Barrier => "barrier",
        }
    }
}
"""


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def coverage_text(entry, step, segment, expansion):
    return f"CRASH_FLOW_COVERAGE entry={entry} step={step} segment={segment} expansion={expansion}"


def sample_world(work, name, segments=SAMPLE_SEGMENTS, entries=("mount_writable", "publish_first_file"), ignored_rows=None, zero_rows=None):
    root = os.path.join(work, name)
    write(os.path.join(root, SEGMENTS_SOURCE), segments)
    write(os.path.join(root, "crates", "singlefs-verification", "tests", "common_crash_points", "mod.rs"), 'println!("CRASH_FLOW_COVERAGE entry={} ...");\n')
    write(os.path.join(root, ENTRIES_TABLE), "# 样本\n" + "".join(f"{entry}\t样本入口\n" for entry in entries))
    if ignored_rows is not None:
        write(os.path.join(root, IGNORED_TABLE), "# 样本\n" + "".join(row + "\n" for row in ignored_rows))
    if zero_rows is not None:
        write(os.path.join(root, ZERO_TABLE), "# 样本\n" + "".join(row + "\n" for row in zero_rows))
    return root


GREEN_LOG_LINES = [
    coverage_text("mount_writable", "ZeroFill", 0, "expanded"),
    "test x ... " + coverage_text("mount_writable", "unit_write", 1, "expanded"),
    coverage_text("publish_first_file", "Barrier", 0, "expanded"),
    coverage_text("publish_first_file", "UnitWrite", 2, "not_expanded"),
    "CRASH_POINT_REPLAY_SAMPLE_FAST states=10 checker_by_invariant(evaluated/violated/not_applicable) I-1.1=10/0/0 I-2.1=0/0/10",
    "CHECKER record_root_without_record=0 I-1.1=4/0/0 I-2.1=0/0/0",
]


def run_quietly(function, *arguments):
    """调判法、吞掉它打的字，交退出码与输出。"""
    import contextlib
    import io
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        code = function(*arguments)
    return code, buffer.getvalue()


def collect_unreachable_cell(expect, repository, plan, log, work):
    """collect-unreachable-exits-75：计划整份复用（要取回存着的行）而服务连不上（socket 不在、TCP 端口刚放掉、重连总时长 2 秒），
    collect 照客户端退 75，不交 unknown、不当库里没有。在进程里换连哪一处、丢掉旧连接，判完换回来。"""
    import contextlib
    import io
    import socket
    probe = socket.socket()
    probe.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % probe.getsockname()[1]
    probe.close()
    client = sys.modules.get("singlefs_prune_server_client")
    changes = {"SINGLEFS_PRUNE_SERVER_SOCKET": os.path.join(work, "no-such.sock"), "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS": address,
               "SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS": "2"}
    saved = {name: os.environ.get(name) for name in changes}
    outcome, captured = "退 0", io.StringIO()
    try:
        os.environ.update(changes)
        if client is not None:
            client.reset_process_client()
        with contextlib.redirect_stderr(captured):
            records, _summary = collect(repository, "sample-kind", plan, log, "样本")
        outcome = f"退 0，交了 {records}"
    except SystemExit as failure:
        outcome = f"退 {failure.code}"
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        if client is not None:
            client.reset_process_client()
    expect("collect-unreachable-exits-75", outcome == "退 75",
           f"整份复用要取回存着的行而 prune-server 连不上，应当退 75（不当库里没有、不交 unknown）；实际{outcome}，标准错误 {captured.getvalue().strip()[-200:]}")


def selftest_cases(work):
    """[(格名, 判错时的一句 或 None)]。"""
    cases = []
    green_records = records_of_lines(GREEN_LOG_LINES, "sample_stream")

    def expect(name, condition, failure):
        cases.append((name, None if condition else failure))

    # 写法：四个字段、name() 的串、粘在 libtest 那一行后面、计数行的标签与三段写法
    kinds = [record[0] for record in green_records]
    expect("extract-lines", kinds == ["coverage"] * 4 + ["counts"] * 2 and green_records[4][2] == "CRASH_POINT_REPLAY_SAMPLE_FAST" and green_records[5][2] == "CHECKER",
           f"四条覆盖行、两条计数行（标签 CRASH_POINT_REPLAY_SAMPLE_FAST 与 CHECKER）应当都认出来，实际 {green_records}")
    named = records_of_lines(["CRASH_POINT_REPLAY_TREE_SPLIT stream=RootSplit states=3 I-1.1=3/0/0"], "s")
    expect("extract-stream-label", named == [("counts", "s", "CRASH_POINT_REPLAY_TREE_SPLIT/RootSplit", "I-1.1=3/0/0")],
           f"带 stream=<名> 的计数行，标签应当接成 <词>/<名>，实际 {named}")
    malformed = records_of_lines(["CRASH_FLOW_COVERAGE entry=a step=UnitWrite segment=x expansion=expanded",
                                  "CRASH_FLOW_COVERAGE entry=a step=UnitWrite expansion=expanded segment=0",
                                  "CRASH_FLOW_COVERAGE entry=a step=UnitWrite segment=0 expansion=expanded extra=1",
                                  "CRASH_FLOW_COVERAGE_SUMMARY lines=3"], "s")
    expect("extract-malformed", [record[0] for record in malformed] == ["malformed", "malformed", "malformed"],
           f"段不是数、字段换了次序、多出第五个字段的三行应当记成写法不对，CRASH_FLOW_COVERAGE_SUMMARY 不算覆盖行，实际 {malformed}")
    odd_counts = records_of_lines(["CHECKER record_root_without_record=0 I-1.1=4/0 I-2.1=0/0", "SAMPLE I-1.1=1/0/0/0", "SAMPLE I-1.1=1/0/0"], "s")
    expect("extract-count-shape", [record[0] for record in odd_counts] == ["malformed", "malformed", "counts"],
           f"计数行只认三段：两段的、四段的整行应当记成写法不对，三段的照认，实际 {odd_counts}")
    members, names, problem = read_step_kinds(sample_world(work, "steps"))
    expect("step-kinds-read", members == ["ZeroFill", "UnitWrite", "Barrier"] and names.get("UnitWrite") == "unit_write" and not problem,
           f"StepKind 应当读出 ZeroFill、UnitWrite、Barrier（注释里的不算）与 name() 的串，实际 {members} {names} {problem}")

    green = sample_world(work, "green", ignored_rows=["not_expanded\tsample_stream\tpublish_first_file\tUnitWrite\t2"],
                         zero_rows=["sample_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-2.1\t样本流不走二类结构",
                                    "sample_stream:CHECKER\tI-2.1\t样本流不走二类结构"])
    code, out = run_quietly(check_coverage, green, green_records, False)
    expect("coverage-green", code == EXIT_GREEN, f"登记的两个入口都有行、三个成员都罩住，coverage 应当判绿，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + [("coverage", "other", "unmount", "UnitWrite", "0", "expanded")], False)
    expect("coverage-unregistered-entry", code == EXIT_RED and "入口 unmount 不在登记表" in out, f"登记表外的入口 unmount 应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, sample_world(work, "entry-without-lines", entries=("mount_writable", "publish_first_file", "unmount")),
                            green_records, False)
    expect("coverage-entry-without-lines", code == EXIT_RED and "登记的入口 unmount" in out, f"登记了却一条行都没有的入口 unmount 应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, sample_world(work, "new-member", segments=SAMPLE_SEGMENTS.replace("    Barrier,\n", "    Barrier,\n    TrimHint,\n")),
                            green_records, False)
    expect("coverage-new-step-kind-member", code == EXIT_RED and "StepKind 成员 TrimHint" in out, f"StepKind 加了成员 TrimHint 而没有流罩住，应当判红，实际退 {code}：{out}")
    only_ignored = [record for record in green_records if not (record[0] == "coverage" and record[3] == "Barrier")] + \
                   [("coverage", "sample_stream", "publish_first_file", "Barrier", "3", "ignored_point")]
    code, out = run_quietly(check_coverage, green, only_ignored, False)
    expect("coverage-only-ignored-does-not-cover", code == EXIT_RED and "StepKind 成员 Barrier" in out,
           f"Barrier 只出现在 ignore 开着的点上，不算罩住，应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + [("coverage", "sample_stream", "mount_writable", "TrimHint", "0", "expanded")], False)
    expect("coverage-unknown-step", code == EXIT_RED and "步骤 TrimHint 不是 StepKind 的成员" in out, f"步骤不是成员应当判红，实际退 {code}：{out}")
    partial = [record for record in green_records if not (record[0] == "coverage" and record[2] == "publish_first_file")] + \
              [("unknown", "other_stream", "样本：有复用、库里没有存着的行")]
    code, out = run_quietly(check_coverage, green, partial, False)
    expect("coverage-incomplete-is-not-run", code == EXIT_NOT_RUN and "publish_first_file" in out,
           f"有测试二进制的行不全时，入口没有行只记本次未跑（退 77）、点名 publish_first_file，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, partial[:-1], True)
    expect("coverage-subset-is-not-run", code == EXIT_NOT_RUN, f"--subset 时入口没有行只记本次未跑，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + malformed, False)
    expect("coverage-malformed-red", code == EXIT_RED and "写法不对" in out, f"写法不对的覆盖行应当判红，实际退 {code}：{out}")
    # 登记表对全部行判：形如 <流>::<标签> 的入口名也要登记
    code, out = run_quietly(check_coverage, green, green_records + [("coverage", "other", "sample_flow::label", "UnitWrite", "0", "expanded")], False)
    expect("coverage-flow-entry-judged-against-the-table", code == EXIT_RED and "入口 sample_flow::label 不在登记表" in out,
           f"<流>::<标签> 形的入口名也按登记表判，没登记应当判红，实际退 {code}：{out}")

    code, out = run_quietly(check_ignored, green, green_records)
    expect("ignored-green", code == EXIT_GREEN, f"一个不展开的节点、基准里也是它，应当判绿，实际退 {code}：{out}")
    code, out = run_quietly(check_ignored, green, green_records + [("coverage", "sample_stream", "publish_first_file", "UnitWrite", "5", "registered_segment")])
    expect("ignored-one-more-node", code == EXIT_RED and "registered_segment" in out, f"比基准多一个节点应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_ignored, green, [record for record in green_records if record[5:6] != ("not_expanded",)])
    expect("ignored-one-fewer-node", code == EXIT_GREEN and "比基准少 1 个" in out, f"比基准少一个节点应当判绿并报出来，实际退 {code}：{out}")
    unseeded = sample_world(work, "unseeded", ignored_rows=[UNSEEDED_MARKER], zero_rows=[UNSEEDED_MARKER])
    code_ignored, out_ignored = run_quietly(check_ignored, unseeded, green_records)
    code_zero, out_zero = run_quietly(check_zero_evaluations, unseeded, green_records)
    expect("baselines-unseeded", code_ignored == EXIT_NOT_RUN and code_zero == EXIT_NOT_RUN and REASON_PLACEHOLDER in out_zero and "not_expanded" in out_ignored,
           f"两张基准都还写着「{UNSEEDED_MARKER}」，应当都退 77 并打候选行，实际退 {code_ignored}、{code_zero}：{out_ignored}{out_zero}")
    code, out = run_quietly(check_ignored, sample_world(work, "no-ignored-table"), green_records)
    expect("ignored-table-missing", code == EXIT_RED, f"基准不在应当判红，实际退 {code}：{out}")
    wide = sample_world(work, "wide-ignored-table", ignored_rows=["not_expanded\tsample_stream\tpublish_first_file\tUnitWrite\t2\textra"])
    code, out = run_quietly(check_ignored, wide, green_records)
    expect("ignored-table-wrong-width", code == EXIT_RED and "不是「<去向>" in out, f"基准的行多出一列（不是五列）应当判红，实际退 {code}：{out}")

    code, out = run_quietly(check_zero_evaluations, green, green_records)
    expect("zero-evaluations-green", code == EXIT_GREEN, f"两个 0 次的对都在基准里，应当判绿，实际退 {code}：{out}")
    new_zero = green_records + records_of_lines(["CRASH_POINT_REPLAY_SAMPLE_FAST I-1.1=0/0/3 I-3.1=0/0/3"], "second_stream")
    code, out = run_quietly(check_zero_evaluations, green, new_zero)
    expect("zero-evaluations-new-zero", code == EXIT_RED and "second_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-1.1" in out,
           f"一条流上 I-1.1 评估 0 次、基准里没有，应当判红，实际退 {code}：{out}")
    empty_reason = sample_world(work, "empty-reason", zero_rows=["sample_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-2.1\t",
                                                                 "sample_stream:CHECKER\tI-2.1\t" + REASON_PLACEHOLDER])
    code, out = run_quietly(check_zero_evaluations, empty_reason, green_records)
    expect("zero-evaluations-empty-reason", code == EXIT_RED and "理由空着" in out, f"基准里理由空着、写着占位的行应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_zero_evaluations, green, [])
    expect("zero-evaluations-no-counts", code == EXIT_NOT_RUN, f"一条计数行都没有应当退 77，实际退 {code}：{out}")

    # collect：git 仓当库根（库在临时 prune-server 里，键带临时目录的命名空间）；整份现判存、整份复用取回、一半复用而库里没有的报 unknown
    repository = os.path.join(work, "collect")
    os.makedirs(repository)
    subprocess.run(["git", "init", "-q", repository], check=True)
    plan = os.path.join(repository, "plan.out")
    log = os.path.join(repository, "pass.log")

    def plan_rows(decisions):
        write(plan, "".join(f"{name}\t{decision}\t{key}\ttarget\ttests/sample_stream.rs\t{name}\t样本\n" for name, decision, key in decisions) + "PLAN judge=0 reuse=0\n")

    log_text = "   Compiling x\n     Running tests/sample_stream.rs (target/release/deps/sample_stream-0)\nrunning 2 tests\n" + "\n".join(GREEN_LOG_LINES) + \
               "\ntest a ... ok\ntest b ... ok\n\ntest result: ok. 2 passed; 0 failed\n     Running unittests src/lib.rs (target/release/deps/lib-0)\nrunning 0 tests\n"
    plan_rows([("a", "judge", "k1"), ("b", "judge", "k2")])
    write(log, log_text)
    fresh, _summary = collect(repository, "sample-kind", plan, log, "样本")
    plan_rows([("a", "reuse", "k1"), ("b", "reuse", "k2")])
    write(log, "     Running tests/sample_stream.rs (target/release/deps/sample_stream-0)\nrunning 0 tests\n")
    recalled, summary = collect(repository, "sample-kind", plan, log, "样本")
    expect("collect-recalls-reused-binary", sorted(recalled) == sorted(fresh) and len(fresh) == 6,
           f"整份现判时存下的 6 条记录，下一趟整份复用时应当原样取回，实际头一趟 {len(fresh)} 条、复用那一趟 {recalled}（{summary}）")
    in_service = [relative for relative, _value in flow_lines_store(repository, "sample-kind").scan(FLOW_LINES_DIRECTORY + "/")]
    local_library = os.path.join(repository, ".git", "singlefs-unit-verdicts")
    expect("collect-lines-in-prune-server", len(in_service) == 1 and in_service[0].endswith(".lines") and not os.path.exists(local_library),
           f"整份现判那一趟的行应当只存进 prune-server（flow-lines/ 底下 1 条），本地库目录不建；实际服务里 {in_service}，"
           f"{local_library} {'建了' if os.path.exists(local_library) else '没建'}")
    collect_unreachable_cell(expect, repository, plan, log, work)
    plan_rows([("a", "reuse", "k1"), ("b", "judge", "k3")])
    partial_records, summary = collect(repository, "sample-kind", plan, log, "样本")
    expect("collect-partial-reuse-is-unknown", any(record[0] == "unknown" and record[1] == "sample_stream" for record in partial_records),
           f"一半复用、库里没有这一组键存下的行，应当交一条 unknown，实际 {partial_records}（{summary}）")
    unplanned, _summary = collect(repository, "sample-kind", "-", os.path.join(repository, "pass.log"), "样本")
    expect("collect-without-plan", not any(record[0] == "unknown" for record in unplanned), f"没有计划（全部现判）时日志里每一段都算整份，实际 {unplanned}")

    # run-directory：按条复用那一趟的日志、放量那一段的日志、enumeration 日志里点名的记录文件照读
    run = os.path.join(work, "run")
    write(os.path.join(run, "test-functions.sample_stream.log"), "\n".join(GREEN_LOG_LINES) + "\n")
    write(os.path.join(run, "pipeline.log"), "SAMPLE_PIPELINE I-1.1=0/0/2\n")
    records_file = os.path.join(work, "case-records.tsv")
    write(records_file, record_line(("coverage", "sample_stream", "mount_writable", "UnitWrite", "0", "expanded")) + "\n")
    write(os.path.join(run, "enumeration.sample.log"), f"  · 记在 x\n{RECORDS_FILE_LINE} {records_file}\n")
    gathered = run_directory_records(run)
    expect("run-directory", len(gathered) == 8 and ("counts", "crash-amplification", "SAMPLE_PIPELINE", "I-1.1=0/0/2") in gathered
           and ("coverage", "sample_stream", "mount_writable", "UnitWrite", "0", "expanded") in gathered,
           f"日志目录应当认出按条复用那一趟的 6 条、放量 1 条、enumeration 点名的记录文件 1 条，实际 {gathered}")
    table = table_text(green, gathered)
    expect("table", "| mount_writable | 罩住 | 罩住 |  |" in table and "| publish_first_file |  | 未展开 | 罩住 |" in table,
           f"覆盖表应当按入口 × 成员列罩住与未展开，实际：{table}")
    return cases


def selftest_service_refusal():
    """覆盖行存取那几格写 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）。
    是临时服务交空串，不是交一句为什么（客户端的 selftest_service_problem）。"""
    path = os.path.join(REPOSITORY_ROOT, "scripts", "prune-server", "prune_server_client.py")
    if not os.path.isfile(path):
        return f"这棵树里没有 {path}"
    name = "singlefs_prune_server_client"
    if name not in sys.modules:
        specification = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name].selftest_service_problem()


def selftest():
    service_problem = selftest_service_refusal()
    if service_problem:
        print(f"  ✗ crash-flow-coverage 自检没跑：{service_problem}")
        print("  → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）")
        return EXIT_USAGE
    work = tempfile.mkdtemp(prefix="crash-flow-coverage-selftest-")
    try:
        cases = selftest_cases(work)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    failures = [(name, failure) for name, failure in cases if failure]
    for name, failure in failures:
        print(f"  ✗ 自检 {name}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自检 {len(cases)} 格里有 {len(failures)} 格判错（{'、'.join(name for name, _failure in failures)}）")  # gate-lint:summary
        print("  → 看文件头「判法」与点名那一格的判法函数；CRASH_FLOW_COVERAGE_BREAK 设着的话这里本来就该红")
        return EXIT_RED
    print(f"  ✓ crash-flow-coverage 自检通过：覆盖行与计数行的认法、覆盖表（登记表外的入口、没有行的入口、新加的 StepKind 成员、只出现在 ignore 上的不算罩住、行不全时记本次未跑）、"
          f"ignore 节点比基准多一个判红、评估 0 次的新对与空理由判红、覆盖行存取（只在 prune-server 里、连不上退 75）、日志目录（查了 {len(cases)} 格）")
    return EXIT_GREEN


BREAK_VARIABLES = ("CRASH_FLOW_COVERAGE_BREAK",)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行不是单独一个 --selftest而 BREAK_VARIABLES 里有一个非空（不管值是什么），
    退 2 并点名它只在自证里认；在参数解析之前判。"""
    for name in BREAK_VARIABLES:
        if os.environ.get(name) and arguments != ["--selftest"]:
            print(f"  ✗ {name} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
            print(f"     → 怎么办：去掉 {name} 再跑；要证红就跑 {name}=<项> python3 scripts/crash/crash-flow-coverage.py --selftest", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
