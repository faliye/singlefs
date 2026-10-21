#!/usr/bin/env python3
# admission: always 判的是这一趟崩溃点重放的日志与此刻的登记表、StepKind 源码；门禁与入口每一趟现判，几秒跑完
# run-condition: none 只读日志、登记表与 Rust 源码；判定库里存覆盖行的目录取不到（不在 git 仓里）时不存不取，照「不全」报
"""崩溃流覆盖：覆盖表、ignore 开着的节点、评估 0 次的（流，不变量）对、上一次全量的标记。判法只在这一份里，
门禁 checker-tier-crash-point-replay 的 crash-flow-coverage、crash-ignored-nodes、invariant-zero-evaluations、last-full-judgement 四格调它，
入口 scripts/crash/crash-point-replay.sh 每趟跑完拿它把覆盖表落进这一趟的日志目录。出口与条款：.claude/kb/milestone/03-third-txn.md「出口」的出口 4、
六·1、六·3、六·5；用户 2026-10-20 定的 K1–K7 原话在 records/2026-10-18-里程碑三收尾与近期欠账.md「用户 2026-10-19 弹窗定出口 4 七条」那一条。

读的两种行（crates 那一侧打，这里只读）：
  覆盖行  CRASH_FLOW_COVERAGE entry=<入口名> step=<StepKind 成员名> segment=<段序号> expansion=<expanded|ignored_point|registered_segment|not_expanded> domain=<quick|full>
          五个字段的名字与次序逐字如上，多一个少一个、值不认得都记成「写法不对」。step 认成员名（UnitWrite），也认它 name() 的串（unit_write），换成成员名再判。
          行首可以粘着 libtest 的「test <名> ... 」。
  计数行  一行里带 I-x.y=<评估过>/<判违例>/<不适用>（crates/singlefs-checker-tier/src/crash.rs 的 checker_counts_by_invariant；旧的两段写法 I-x.y=<评估过>/<判违例> 也认）。
          流 = <来源>:<这一行在第一个 I- 之前头一个全大写的词>（例 crash_enumeration_fixed_script_stream:CRASH_POINT_REPLAY_FIXED_SCRIPT_FAST；没有这样的词记 -；
          那之前另有 stream=<名> 或 flow=<名> 的接成 <词>/<名>，例 crash_enumeration_tree_split_streams:CRASH_POINT_REPLAY_TREE_SPLIT/AccountingLeafSplit），
          来源是测试目标名（tests/<名>.rs 去掉 .rs；库的单测记 lib，装置二进制记 bin/<名>），放量那一段记 crash-amplification。同一个流打了几行，次数相加。
          计数行不带域：门禁快档、崩溃点重放入口的快档那一段算 quick；门禁 --full 的用例日志算 full；入口放量那一段照档（--quick 是 quick，别的 full）。

记录（这份脚本自己的中间格式，一行一条，制表符隔开，行首可带空白）：
  CRASH_FLOW_RECORD coverage <来源> <入口> <步骤> <段> <展开方式> <域>
  CRASH_FLOW_RECORD counts <来源> <域> <标签> <I-x.y=a/b/c 用空格连起来>
  CRASH_FLOW_RECORD malformed <来源> <那一行>
  CRASH_FLOW_RECORD unknown <来源> <为什么这个测试二进制的行这一趟不全>
  日志里另有一行 CRASH_FLOW_RECORDS_FILE <路径> 的（门禁 --full 打的），run-directory 照路径把那份记录读进来。

子命令：
  extract --source <来源> --domain <quick|full> <日志>…        日志里的覆盖行与计数行转成记录；日志里本来就有的记录原样带上
  collect-quick <根> <种类> <计划输出|-> <日志> [--library-root <根>] [--pass <名>]
        门禁快档一趟（不带特性那一趟或带特性那一趟）的日志按测试二进制（「Running <目标根文件> (」那一行起的一段）转成记录。
        复用：快档按测试函数的单元键复用，复用的测试这一趟不跑、不打行。一个测试二进制里编得进的测试函数（计划输出里 judge、reuse、refused 的那几条）
        的（名字，键）排好序取 sha256 是它的组成键：这一趟整份现判（一条复用都没有）的，打出来的覆盖行与计数行照组成键存进判定库
        <git common-dir>/singlefs-unit-verdicts/<种类>/flow-lines/<组成键>.lines（判定库的布局在 scripts/gate/admission.py 文件头）；
        有复用的，库里有同一个组成键就取回存着的行（连同这一趟现判的那几条打的行）；没有就只交这一趟打的行，另交一条 unknown（这个测试二进制的行不全）。
        计划输出给 - 或没有一条计划行（按条复用算不出、全部现判）：日志里每一段都算整份现判、不存。
  run-directory <目录> --tier <quick|pruned|named|full>       崩溃点重放入口一趟的日志目录（fast-tier.<目标>.log、fast-tier.<目标>.two-hosts.log、
        pipeline.log、enumeration.<项>.log）转成记录
  check <coverage|ignored|zero-evaluations> <根> --domain <quick|full> [--subset] <记录文件>…    判一格（下面「判法」），退出码见下
  table <根> (<记录文件>… | --run-directory <目录> --tier <档>)  打覆盖表（两个域各一张）、ignore 开着的节点、评估 0 次的对，Markdown
  baseline <ignored|zero-evaluations> <记录文件>…              照这一趟的记录打基准表的候选行（填 crash-ignored-nodes.tsv、invariant-zero-evaluations.tsv 用；
        0 次那张的理由列打成 <填理由>，要人逐行写，门禁不认这个占位）
  last-full <读库输出文件> [--after-full]                      上一次全量的标记（C635），下面「上一次全量」
  --selftest

判法：
  coverage（K1 乙、K2 轻重都算）：行是入口登记表 .claude/gate.d/crash-entries.tsv（一行「<入口名><TAB><说明>」），列是 StepKind 的成员
    （crates/singlefs-harness/src/segments.rs 的 pub enum StepKind 静态读）。一格罩住 = 这个域里有一条覆盖行、步骤是它、expansion=expanded。
    判红：覆盖行写法不对；有覆盖行的入口不在登记表里；步骤不是 StepKind 的成员；登记的入口在这个域里一条覆盖行都没有；StepKind 有成员在这个域里没被任何一条罩住；
    登记表不在、行写错（不是两列、名字重了、说明空着）、StepKind 读不出。后两样（入口没有行、成员没被罩住）只在行齐的时候判：
    带了 --subset（只跑了点名的几条用例）或有 unknown 记录（有测试二进制的行不全）时这两样记本次未跑、逐个列名，退 77（别的照判，有红照红）。
    crates 侧还没打覆盖行（crates/ 下的 .rs 里一处 CRASH_FLOW_COVERAGE 都没有）：整格退 77。
  ignored（K3 丙）：节点 = 一条 expansion 是 ignored_point、registered_segment、not_expanded 的覆盖行，身份是（去向、来源、入口、步骤、段、域）。
    基准 .claude/gate.d/crash-ignored-nodes.tsv 一行一个节点，六列照这个次序；这一趟出现了基准里没有的节点就红（逐条列）；比基准少的报出来、不红（基准可以删那几行）。
    基准不在、行写错判红；基准里有一行「# 初值：未填」（还没跑过一趟轻型来填）整格退 77，同时打这一趟的候选行。crates 侧还没打覆盖行整格退 77。
  zero-evaluations（K4 甲）：这个域的计数行按（流，不变量）相加，评估过 0 次的对不在基准 .claude/gate.d/invariant-zero-evaluations.tsv
    （一行「<域><TAB><流><TAB><不变量><TAB><理由>」）里就红（逐条列）；基准每行理由不许空、不许是 <填理由>。基准里的对这一趟不是 0 了、
    或这一趟没出现，报出来、不红。这个域一条计数行都没有先退 77（不读基准）；基准有「# 初值：未填」一行整格退 77、打候选行。
  上一次全量（K6 甲）：读库程序（crates/singlefs-checker-tier/src/bin/crash_library_reader.rs，子命令名由调用方给）的输出，写法（制表符隔开、字段次序固定）：
    CRASH_LIBRARY_LAST_FULL	library=<库目录>	recorded_blocks=<录下的块>	full_under_the_current_key=<当前判法单元指纹下有全量判定的块>	not_full_under_the_current_key=<没有的块>	last_full_date=<YYYY-MM-DD|none>
    CRASH_LIBRARY_LAST_FULL_BLOCK	block=<块键十六进制>	judged=<YYYY-MM-DD|none>	run_kind=<full|pruned|item_by_item|none>
    （每个「没有全量判定」的块一行，行数要等于 not_full_under_the_current_key）；库不在只打一行 CRASH_LIBRARY_LAST_FULL	library=<库目录>	missing。
    库里认不出项名（块键只由内容定），这里按块列。recorded_blocks = 两个数之和、块行数对得上才认；认不出的写法退 1。
    结局：带 --after-full（这一趟跑了全量）且没有不是当前的块，退 0；别的一律退 77 并打一行信息（上一次全量的日期、哪些块不是当前的，头 20 块），不挡提交。
退出码：0 判绿；1 判红（逐处列、带出路）；2 用法错、读不了记录文件；77 本次未跑（上面各条写了哪几种）。

弄坏开关（只给 --selftest 证红用）CRASH_FLOW_COVERAGE_BREAK=<项>（几个用逗号连）。逐项写开关名、弄坏哪一处，下一行写打开后自证哪几格红
（格名是 selftest_cases() 里那一格的名字；门禁 checker-tier-crash-point-replay 的样本哪一份判错写在那道门禁文件头）：
  CRASH_FLOW_COVERAGE_BREAK=unregistered-entry-accepted  coverage 不判登记表外的入口
      coverage-unregistered-entry
  CRASH_FLOW_COVERAGE_BREAK=entry-absence-unchecked  coverage 不判登记的入口在这个域里一条行都没有
      coverage-entry-without-lines
  CRASH_FLOW_COVERAGE_BREAK=step-kind-coverage-unchecked  coverage 不判 StepKind 成员没被罩住
      coverage-new-step-kind-member、coverage-only-ignored-does-not-cover
  CRASH_FLOW_COVERAGE_BREAK=any-expansion-covers  罩住照「出现」算，不看 expansion=expanded
      coverage-only-ignored-does-not-cover、table
  CRASH_FLOW_COVERAGE_BREAK=incomplete-judged-as-complete  有 unknown 记录或 --subset 时照样判入口没有行与成员没被罩住
      coverage-incomplete-is-not-run、coverage-subset-is-not-run
  CRASH_FLOW_COVERAGE_BREAK=ignored-growth-accepted  ignored 不判比基准多出来的节点
      ignored-one-more-node
  CRASH_FLOW_COVERAGE_BREAK=unseeded-marker-ignored  两张基准不认「# 初值：未填」，照空基准判
      baselines-unseeded
  CRASH_FLOW_COVERAGE_BREAK=zero-growth-accepted  zero-evaluations 不判基准外的 0 次对
      zero-evaluations-new-zero
  CRASH_FLOW_COVERAGE_BREAK=empty-reason-accepted  zero-evaluations 不判基准里理由空着的行
      zero-evaluations-empty-reason
  CRASH_FLOW_COVERAGE_BREAK=reuse-without-recall  collect-quick 照改前：复用的测试二进制不取回存着的行、也不报 unknown
      collect-quick-recalls-reused-binary、collect-quick-partial-reuse-is-unknown
  CRASH_FLOW_COVERAGE_BREAK=block-count-unchecked  last-full 不核块行数与 not_full_under_the_current_key
      last-full-block-count
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
SEGMENTS_SOURCE = "crates/singlefs-harness/src/segments.rs"
ADMISSION_MODULE = os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py")
UNSEEDED_MARKER = "# 初值：未填"
REASON_PLACEHOLDER = "<填理由>"
EXPANSIONS = ("expanded", "ignored_point", "registered_segment", "not_expanded")
IGNORED_KINDS = EXPANSIONS[1:]
DOMAINS = ("quick", "full")
RECORD_PREFIX = "CRASH_FLOW_RECORD"
RECORDS_FILE_LINE = "CRASH_FLOW_RECORDS_FILE"
COVERAGE_MARK = re.compile(r"(?<![A-Za-z0-9_])CRASH_FLOW_COVERAGE(?=\s|$)")
COVERAGE_FIELDS = ("entry", "step", "segment", "expansion", "domain")
NAME_VALUE = re.compile(r"^[A-Za-z0-9_.:-]+$")
COUNT_TOKEN = re.compile(r"(?<![\w.-])(I-\d+(?:\.\d+)*)=(\d+)/(\d+)(?:/(\d+))?(?![\w/])")
LABEL_TOKEN = re.compile(r"^[A-Z][A-Z0-9_]{2,}$")
RUNNING_LINE = re.compile(r"^\s*Running (?:unittests )?(\S+) \(")
PLAN_DECISIONS = ("judge", "reuse", "refused")
FLOW_LINES_DIRECTORY = "flow-lines"
LAST_FULL_TAG = "CRASH_LIBRARY_LAST_FULL"
LAST_FULL_BLOCK_TAG = "CRASH_LIBRARY_LAST_FULL_BLOCK"
LAST_FULL_FIELDS = ("library", "recorded_blocks", "full_under_the_current_key", "not_full_under_the_current_key", "last_full_date")
BLOCK_FIELDS = ("block", "judged", "run_kind")
RUN_KINDS = ("full", "pruned", "item_by_item", "none")
DATE_VALUE = re.compile(r"^(\d{4}-\d{2}-\d{2}|none)$")
SHOWN_AT_MOST = 20
CANDIDATES_SHOWN = 40
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77


# ── 行与记录 ─────────────────────────────────────────

def clean(text):
    return text.replace("\t", " ").replace("\n", " ").strip()


def record_line(fields):
    return RECORD_PREFIX + "\t" + "\t".join(clean(str(field)) for field in fields)


def parse_coverage(text):
    """覆盖行 → (入口, 步骤, 段, 展开方式, 域)；写法不对 → None；不是覆盖行 → False。"""
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
    entry, step, segment, expansion, domain = values
    if not segment.isdigit() or expansion not in EXPANSIONS or domain not in DOMAINS:
        return None
    return entry, step, segment, expansion, domain


def count_tokens(text):
    return list(COUNT_TOKEN.finditer(text))


def counts_label(text, first_match):
    """第一个 I- 之前头一个全大写的词；那之前另有 stream=<名> 或 flow=<名> 的，接成 <词>/<名>（一个测试目标里几条流共用一个词时分得开）。"""
    words = text[:first_match.start()].split()
    label = next((word for word in words if LABEL_TOKEN.match(word)), "-")
    named = next((word.split("=", 1)[1] for word in words if re.match(r"^(stream|flow)=[A-Za-z0-9_.:-]+$", word)), "")
    return f"{label}/{named}" if named else label


def is_relevant(text):
    return parse_coverage(text) is not False or bool(count_tokens(text))


def records_of_lines(lines, source, domain):
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
        matches = count_tokens(text)
        if matches:
            tokens = " ".join(match.group(0) for match in matches)
            records.append(("counts", source, domain, counts_label(text, matches[0]), tokens))
    return records


RECORD_WIDTHS = {"coverage": 7, "counts": 5, "malformed": 3, "unknown": 3}


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


# ── 判定库里存的行（collect-quick） ─────────────────────────

def flow_lines_directory(library_root, kind):
    """<git common-dir>/singlefs-unit-verdicts/<种类>/flow-lines；取不到 git common-dir 交 None。布局照 admission.py 的 VerdictLibrary。"""
    specification = importlib.util.spec_from_file_location("admission_for_flow_lines", ADMISSION_MODULE)
    admission = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(admission)
    directory = admission.VerdictLibrary(library_root, kind).directory
    return os.path.join(directory, FLOW_LINES_DIRECTORY) if directory else None


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


def store_lines(directory, key, lines):
    os.makedirs(directory, exist_ok=True)
    handle_number, temporary = tempfile.mkstemp(dir=directory, prefix=f".{key}.partial.")
    with os.fdopen(handle_number, "w", encoding="utf-8") as handle:
        handle.write("".join(line + "\n" for line in lines))
    os.replace(temporary, os.path.join(directory, key + ".lines"))


def stored_lines(directory, key):
    if directory is None:
        return None
    try:
        with open(os.path.join(directory, key + ".lines"), encoding="utf-8", errors="replace") as handle:
            return [line.rstrip("\n") for line in handle]
    except OSError:
        return None


def collect_quick(root, kind, plan_path, log_path, library_root, pass_name):
    """交 (记录, 一句汇总)。"""
    with open(log_path, encoding="utf-8", errors="replace") as handle:
        sections = log_sections(handle)
    plan = read_plan(plan_path)
    records, fresh_complete, recalled, stored, incomplete = [], 0, 0, 0, []
    directory = flow_lines_directory(library_root, kind) if plan is not None else None
    roots = sorted(set(sections) | set(plan or {}))
    for target_root in roots:
        source = source_of_target_root(target_root)
        fresh = [line for line in sections.get(target_root, []) if is_relevant(line)]
        rows = (plan or {}).get(target_root, [])
        if plan is None or not rows:
            records += records_of_lines(fresh, source, "quick")
            fresh_complete += 1
            continue
        key = composition_key(target_root, rows)
        reused = [row for row in rows if row[1] == "reuse"]
        if not reused:
            if target_root not in sections:
                records.append(("unknown", source, f"{pass_name}：要现判 {len(rows)} 条，日志里却没有这个测试二进制那一段"))
                incomplete.append(source)
                continue
            records += records_of_lines(fresh, source, "quick")
            fresh_complete += 1
            if directory is not None:
                try:
                    store_lines(directory, key, fresh)
                    stored += 1
                except OSError as error:
                    print(f"  ! {source} 的覆盖行没存进判定库（{error}）：下一趟它整份复用时报不全", file=sys.stderr)
            continue
        if "reuse-without-recall" in BROKEN:
            records += records_of_lines(fresh, source, "quick")
            continue
        kept = stored_lines(directory, key)
        if kept is not None:
            merged = list(dict.fromkeys(kept + fresh))
            records += records_of_lines(merged, source, "quick")
            recalled += 1
            continue
        records += records_of_lines(fresh, source, "quick")
        records.append(("unknown", source, f"{pass_name}：{len(reused)} 条测试函数复用、{len(rows) - len(reused)} 条现判，判定库里没有这一组键整份现判时存下的行"))
        incomplete.append(source)
    summary = (f"{pass_name}：{len(roots)} 个测试二进制，整份现判 {fresh_complete} 个（存进判定库 {stored} 个）、取回存着的行 {recalled} 个、"
               f"行不全 {len(incomplete)} 个{('（' + '、'.join(incomplete[:SHOWN_AT_MOST]) + '）') if incomplete else ''}"
               + ("" if plan is not None else "；没有计划行（按条复用算不出、全部现判），日志里每一段都算整份、不存"))
    return records, summary


def run_directory_records(directory, tier):
    records = []
    for path in sorted(glob.glob(os.path.join(directory, "*.log"))):
        base = os.path.basename(path)
        if base == "fast-tier-build.log":
            continue
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.readlines()
        fast = re.match(r"^fast-tier\.(.+?)(?:\.two-hosts)?\.log$", base)
        if fast:
            records += records_of_lines(lines, fast.group(1), "quick")
        elif base == "pipeline.log":
            records += records_of_lines(lines, "crash-amplification", "quick" if tier == "quick" else "full")
        else:
            # enumeration.<项>.log 这一类（门禁 --full 的输出）只带它点名的记录文件与原样的记录行，别的行不当覆盖行读
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


def crates_connected(root):
    """crates/ 下的 .rs 里有没有 CRASH_FLOW_COVERAGE（crates 侧打不打覆盖行）。"""
    for path in glob.glob(os.path.join(root, "crates", "**", "*.rs"), recursive=True):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                if "CRASH_FLOW_COVERAGE" in handle.read():
                    return True
        except OSError:
            continue
    return False


# ── 判法 ─────────────────────────────────────────────

def report(problems, summary, remedy_lines):
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ {summary}")  # gate-lint:summary
    print("  → 怎么办：" + remedy_lines[0])
    for line in remedy_lines[1:]:
        print("            " + line)


def coverage_matrix(records, domain, members, name_to_member):
    """{入口: {成员: 'expanded' | 'seen'}}（这个域）。"""
    matrix = {}
    for record in records:
        if record[0] != "coverage" or record[6] != domain:
            continue
        _kind, _source, entry, step, _segment, expansion, _domain = record
        member = step if step in members else name_to_member.get(step, step)
        cells = matrix.setdefault(entry, {})
        if expansion == "expanded" or "any-expansion-covers" in BROKEN:
            cells[member] = "expanded"
        else:
            cells.setdefault(member, "seen")
    return matrix


def check_coverage(root, records, domain, subset):
    if not crates_connected(root):
        print(f"  ! 本次未跑（这不是通过）：crates/ 下的 .rs 里一处 CRASH_FLOW_COVERAGE 都没有，crates 侧还没打覆盖行；接上之后这一格照 {ENTRIES_TABLE} 与 StepKind 判")
        return EXIT_NOT_RUN
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
            problems.append(f"{record[1]}：覆盖行写法不对：{record[2]}")
    outside, bad_steps = {}, {}
    for record in coverage:
        _kind, source, entry, step, _segment, _expansion, _domain = record
        if entry not in entries:
            outside.setdefault(entry, set()).add(source)
        if members and step not in members and step not in name_to_member:
            bad_steps.setdefault(step, set()).add(source)
    if "unregistered-entry-accepted" not in BROKEN:
        for entry, sources in sorted(outside.items()):
            problems.append(f"入口 {entry} 不在登记表 {ENTRIES_TABLE} 里（打它的：{'、'.join(sorted(sources))}）")
    for step, sources in sorted(bad_steps.items()):
        problems.append(f"步骤 {step} 不是 StepKind 的成员（打它的：{'、'.join(sorted(sources))}；成员 {'、'.join(members)}）")
    matrix = coverage_matrix(records, domain, members, name_to_member)
    missing_entries = [entry for entry in entries if entry not in matrix]
    uncovered = [member for member in members if not any(cells.get(member) == "expanded" for cells in matrix.values())]
    complete = (not subset and not unknown) or "incomplete-judged-as-complete" in BROKEN
    not_judged = []
    if complete:
        if "entry-absence-unchecked" not in BROKEN:
            problems += [f"登记的入口 {entry} 在 {domain} 域里一条覆盖行都没有" for entry in missing_entries]
        if "step-kind-coverage-unchecked" not in BROKEN:
            problems += [f"StepKind 成员 {member} 在 {domain} 域里没被任何一条流罩住（没有一条 step={member} expansion=expanded 的覆盖行）" for member in uncovered]
    else:
        not_judged = [f"入口 {entry} 没有行" for entry in missing_entries] + [f"成员 {member} 没被罩住" for member in uncovered]
    domain_lines = sum(1 for record in coverage if record[6] == domain)
    if problems:
        report(problems, f"crash-flow-coverage（{domain} 域）：覆盖行 {domain_lines} 条，{len(problems)} 处不对（逐处列在上面）",
               [f"登记表外的入口：确是新入口就在 {ENTRIES_TABLE} 加一行「<入口名><TAB><说明>」，不是就改打行那一处的入口名；",
                "入口没有行、StepKind 成员没被罩住：给那个入口或那种提交步骤补一条崩溃流（入口一律经 CrashPointRecorder 设点、打覆盖行），或把那一段展开；",
                "写法不对的照 scripts/crash/crash-flow-coverage.py 文件头「覆盖行」改 crates 里打行那一处。"])
        return EXIT_RED
    if not complete:
        reasons = (["只跑了点名的几条（--subset）"] if subset else []) + [f"{record[1]} 的行不全（{record[2]}）" for record in unknown[:SHOWN_AT_MOST]]
        print(f"  ! 本次未跑（这不是通过）：crash-flow-coverage（{domain} 域）的「入口没有行、成员没被罩住」两样没判：{'；'.join(reasons)}；"
              f"这两样里眼下看得到的 {len(not_judged)} 处：{'、'.join(not_judged[:SHOWN_AT_MOST]) or '无'}")
        print("     → 补全：把行不全的测试二进制整份现判一趟（它们的测试函数键对不上的那几条跑过、判绿记库之后，下一趟整份复用时就取得回），或不带 --item 跑全部用例")
        return EXIT_NOT_RUN
    print(f"  ✓ crash-flow-coverage（{domain} 域）：登记的 {len(entries)} 个入口都有覆盖行，StepKind {len(members)} 个成员都被罩住（覆盖行 {domain_lines} 条）")
    return EXIT_GREEN


def node_of(record):
    _kind, source, entry, step, segment, expansion, domain = record
    return (expansion, source, entry, step, segment, domain)


def read_ignored_baseline(root):
    """→ (节点集合, 问题, 有没有初值未填的标记)；表不在抛 OSError。"""
    rows, unseeded = table_rows(os.path.join(root, IGNORED_TABLE))
    nodes, problems = set(), []
    for number, columns in rows:
        stripped = [column.strip() for column in columns]
        if len(stripped) != 6 or stripped[0] not in IGNORED_KINDS or stripped[5] not in DOMAINS or not stripped[4].isdigit() \
           or not all(stripped):
            problems.append(f"{IGNORED_TABLE}:{number} 不是「<去向><TAB><来源><TAB><入口><TAB><步骤><TAB><段><TAB><域>」"
                            f"（去向是 {'、'.join(IGNORED_KINDS)} 之一，段是数，域是 quick 或 full）")
            continue
        node = tuple(stripped)
        if node in nodes:
            problems.append(f"{IGNORED_TABLE}:{number} 这个节点登记了两回")
        nodes.add(node)
    return nodes, problems, unseeded


def ignored_nodes(records):
    return sorted({node_of(record) for record in records if record[0] == "coverage" and record[5] in IGNORED_KINDS})


def check_ignored(root, records, domain):
    if not crates_connected(root):
        print("  ! 本次未跑（这不是通过）：crates/ 下的 .rs 里一处 CRASH_FLOW_COVERAGE 都没有，crates 侧还没打覆盖行，ignore 开着的节点列不出")
        return EXIT_NOT_RUN
    try:
        baseline, problems, unseeded = read_ignored_baseline(root)
    except OSError:
        report([], f"crash-ignored-nodes：基准 {IGNORED_TABLE} 不在",
               [f"建这张表（表头照 scripts/crash/crash-flow-coverage.py 文件头「判法」ignored 那一条），初值用 crash-flow-coverage.py baseline ignored <这一趟的记录> 打"])
        return EXIT_RED
    nodes = ignored_nodes(records)
    if unseeded and "unseeded-marker-ignored" not in BROKEN:
        print(f"  ! 本次未跑（这不是通过）：基准 {IGNORED_TABLE} 里还有「{UNSEEDED_MARKER}」：初值等一趟轻型崩溃点重放之后填；这一趟的 {len(nodes)} 个节点照下面的行填进去（头 {CANDIDATES_SHOWN} 行；全部用 crash-flow-coverage.py baseline ignored <记录文件> 打）、删掉那一行")
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


def zero_pairs(records, domain):
    """{(流, 不变量): 评估过的次数之和}（这个域）。"""
    totals = {}
    for record in records:
        if record[0] != "counts" or record[2] != domain:
            continue
        _kind, source, _domain, label, tokens = record
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
        if len(stripped) != 4 or stripped[0] not in DOMAINS or not stripped[1] or not re.match(r"^I-\d+(?:\.\d+)*$", stripped[2]):
            problems.append(f"{ZERO_TABLE}:{number} 不是「<域><TAB><流><TAB><不变量><TAB><理由>」（域是 quick 或 full，不变量写 I-x.y）")
            continue
        if (not stripped[3] or stripped[3] == REASON_PLACEHOLDER) and "empty-reason-accepted" not in BROKEN:
            problems.append(f"{ZERO_TABLE}:{number} 理由空着（{stripped[1]} 的 {stripped[2]}）：每一行写明这条流为什么评估不到它")
            continue
        key = (stripped[0], stripped[1], stripped[2])
        if key in pairs:
            problems.append(f"{ZERO_TABLE}:{number} 这一对登记了两回")
        pairs[key] = stripped[3]
    return pairs, problems, unseeded


def check_zero_evaluations(root, records, domain):
    totals = zero_pairs(records, domain)
    if not totals:
        print(f"  ! 本次未跑（这不是通过）：这一趟 {domain} 域一条计数行（I-x.y=评估/违例/不适用）都没有，评估次数判不了")
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
        print(f"  ! 本次未跑（这不是通过）：基准 {ZERO_TABLE} 里还有「{UNSEEDED_MARKER}」：初值等一趟轻型崩溃点重放之后填；"
              f"这一趟 {domain} 域 {len(flows)} 条流、评估 0 次的对 {len(zeros)} 个，照下面的行填（头 {CANDIDATES_SHOWN} 行；全部用 crash-flow-coverage.py baseline zero-evaluations <记录文件> 打；理由逐行写）、删掉那一行")
        for flow, invariant in zeros[:CANDIDATES_SHOWN]:
            print(f"    {domain}\t{flow}\t{invariant}\t{REASON_PLACEHOLDER}")
        return EXIT_NOT_RUN
    if problems:
        report(problems, f"invariant-zero-evaluations：基准 {ZERO_TABLE} 有 {len(problems)} 行写错", ["照文件头那一句的写法改那几行，理由逐行写明"])
        return EXIT_RED
    new = [pair for pair in zeros if (domain, *pair) not in baseline]
    if new and "zero-growth-accepted" not in BROKEN:
        report([f"{domain}\t{flow}\t{invariant} 评估过 0 次，基准里没有这一行" for flow, invariant in new],
               f"invariant-zero-evaluations（{domain} 域）：{len(flows)} 条流，评估 0 次的对 {len(zeros)} 个，其中 {len(new)} 个不在基准里",
               ["一个（流，不变量）对评估 0 次不许涨（出口 4、六·3）：查那条流为什么评估不到它（判器漏判、流没走到那一类结构）；",
                f"确是这条流本来就评估不到的（新加的不变量在这条流上不适用这一类），把那一行连理由加进 {ZERO_TABLE}"])
        return EXIT_RED
    gone = sorted(key for key in baseline if key[0] == domain and totals.get(key[1:], None) not in (0,))
    note = f"；基准里 {len(gone)} 行这一趟不是 0 或没出现（可以删或留着）" if gone else ""
    print(f"  ✓ invariant-zero-evaluations（{domain} 域）：{len(flows)} 条流、{len(totals)} 个（流，不变量）对，评估 0 次的 {len(zeros)} 个都在基准里{note}")
    return EXIT_GREEN


def parse_fields(fields, names):
    values = {}
    if len(fields) != len(names):
        return None
    for name, field in zip(names, fields):
        key, separator, value = field.partition("=")
        if key != name or not separator or not value:
            return None
        values[name] = value
    return values


def check_last_full(text, after_full):
    summaries, blocks, problems = [], [], []
    for line in text.splitlines():
        fields = line.rstrip("\r").split("\t")
        if fields[0] == LAST_FULL_TAG:
            summaries.append(fields[1:])
        elif fields[0] == LAST_FULL_BLOCK_TAG:
            parsed = parse_fields(fields[1:], BLOCK_FIELDS)
            if parsed is None or not re.match(r"^[0-9a-f]+$", parsed["block"]) or not DATE_VALUE.match(parsed["judged"]) \
               or parsed["run_kind"] not in RUN_KINDS:
                problems.append(f"块那一行认不出：{clean(line)[:200]}")
            else:
                blocks.append(parsed)
    if len(summaries) != 1:
        problems.append(f"{LAST_FULL_TAG} 那一行要恰好一行，这一趟 {len(summaries)} 行")
    summary = None
    if len(summaries) == 1:
        fields = summaries[0]
        if len(fields) == 2 and fields[0].startswith("library=") and fields[1] == "missing":
            if not problems:
                print(f"  ! 本次未跑（这不是通过）：上一次全量的标记：崩溃判定库 {fields[0][len('library='):]} 不在，一块都没判过")
                return EXIT_NOT_RUN
        else:
            summary = parse_fields(fields, LAST_FULL_FIELDS)
            if summary is None or not all(summary[name].isdigit() for name in LAST_FULL_FIELDS[1:4]) or not DATE_VALUE.match(summary["last_full_date"]):
                problems.append(f"{LAST_FULL_TAG} 那一行认不出：{clean(chr(9).join(fields))[:300]}")
                summary = None
    if summary is not None:
        recorded, full, not_full = (int(summary[name]) for name in LAST_FULL_FIELDS[1:4])
        if recorded != full + not_full:
            problems.append(f"recorded_blocks={recorded} 不等于 full_under_the_current_key={full} 加 not_full_under_the_current_key={not_full}")
        if len(blocks) != not_full and "block-count-unchecked" not in BROKEN:
            problems.append(f"块行 {len(blocks)} 行，not_full_under_the_current_key 报 {not_full}：读库程序的输出不全")
    if problems:
        report(problems, f"上一次全量的标记：读库程序的输出有 {len(problems)} 处认不出",
               ["让读库程序那个子命令的输出与 scripts/crash/crash-flow-coverage.py 文件头「上一次全量」那一段的写法对上（读法与写法钉在同一段字上），再跑"])
        return EXIT_RED
    recorded, full, not_full = (int(summary[name]) for name in LAST_FULL_FIELDS[1:4])
    date = summary["last_full_date"]
    if recorded and not not_full:
        if after_full:
            print(f"  ✓ 上一次全量的标记：当前判法单元指纹下录下的 {recorded} 块都有全量判定（最晚 {date}）")
            return EXIT_GREEN
        print(f"  ! 本次未跑（这不是通过）：这一趟没跑全量；上一次全量 {date}，当前判法单元指纹下录下的 {recorded} 块都有全量判定（只报信息，不挡提交）")
        return EXIT_NOT_RUN
    if not recorded:
        print("  ! 本次未跑（这不是通过）：上一次全量的标记：库里一块都没录")
        return EXIT_NOT_RUN
    shown = "、".join(f"{block['block'][:16]}（{block['run_kind']}，{block['judged']}）" for block in blocks[:SHOWN_AT_MOST])
    print(f"  ! 本次未跑（这不是通过）：全量判定不是当前的块 {not_full} 块（录下的 {recorded} 块里；上一次全量 {date}"
          f"{'' if after_full else '；这一趟没跑全量'}）：{shown}{'…' if not_full > SHOWN_AT_MOST else ''}（只报信息，不挡提交；按项列不出：块键只由内容定）")
    return EXIT_NOT_RUN


# ── 覆盖表与候选行 ──────────────────────────────────────

def table_text(root, records):
    entries, _problems, _present = read_entries(root)
    members, names, step_problem = read_step_kinds(root)
    name_to_member = {name: member for member, name in names.items()}
    out = ["# 崩溃流覆盖表", "",
           f"覆盖行 {sum(1 for record in records if record[0] == 'coverage')} 条、计数行 {sum(1 for record in records if record[0] == 'counts')} 条、"
           f"写法不对 {sum(1 for record in records if record[0] == 'malformed')} 条、行不全的测试二进制 {sum(1 for record in records if record[0] == 'unknown')} 个。"
           "罩住 = 这个域里有一条 expansion=expanded 的覆盖行；未展开 = 只有别的去向。"]
    if step_problem:
        out.append(f"StepKind 读不出：{step_problem}")
    for domain in DOMAINS:
        matrix = coverage_matrix(records, domain, members, name_to_member)
        out += ["", f"## {domain} 域", "", "| 入口 | " + " | ".join(members) + " |", "|---|" + "---|" * len(members)]
        for entry in list(entries) + sorted(set(matrix) - set(entries)):
            cells = matrix.get(entry, {})
            label = entry if entry in entries else f"{entry}（登记表外）"
            out.append(f"| {label} | " + " | ".join({"expanded": "罩住", "seen": "未展开"}.get(cells.get(member), "") for member in members) + " |")
        uncovered = [member for member in members if not any(cells.get(member) == "expanded" for cells in matrix.values())]
        out.append("")
        out.append(f"没被罩住的 StepKind 成员：{'、'.join(uncovered) or '无'}")
    out += ["", "## ignore 开着的节点", "", "| 去向 | 来源 | 入口 | 步骤 | 段 | 域 |", "|---|---|---|---|---|---|"]
    out += ["| " + " | ".join(node) + " |" for node in ignored_nodes(records)]
    for domain in DOMAINS:
        totals = zero_pairs(records, domain)
        by_flow = {}
        for (flow, invariant), total in sorted(totals.items()):
            if total == 0:
                by_flow.setdefault(flow, []).append(invariant)
        out += ["", f"## 评估 0 次的（流，不变量）对：{domain} 域", "", "| 流 | 0 次的不变量 |", "|---|---|"]
        out += [f"| {flow} | {' '.join(invariants)} |" for flow, invariants in by_flow.items()]
    unknown = [record for record in records if record[0] == "unknown"]
    if unknown:
        out += ["", "## 行不全的测试二进制", ""] + [f"- {record[1]}：{record[2]}" for record in unknown]
    return "\n".join(out) + "\n"


# ── 命令行 ─────────────────────────────────────────────

def usage(problem):
    print(f"  ✗ {problem}")
    print("  → 怎么办：子命令与参数见 scripts/crash/crash-flow-coverage.py 文件头「子命令」：extract、collect-quick、run-directory、check、table、baseline、last-full、--selftest")
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
            source, domain = option_value(rest, "--source"), option_value(rest, "--domain")
            if not source or domain not in DOMAINS or not rest:
                return usage("extract 要 --source <来源> --domain <quick|full> 与至少一份日志")
            for path in rest:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    for record in records_of_lines(handle, source, domain):
                        print(record_line(record))
            return EXIT_GREEN
        if command == "collect-quick":
            library_root, pass_name = option_value(rest, "--library-root"), option_value(rest, "--pass")
            if len(rest) != 4 or library_root is None or pass_name is None:
                return usage("collect-quick 要 <根> <种类> <计划输出|-> <日志> [--library-root <根>] [--pass <名>]")
            root, kind, plan_path, log_path = rest
            records, summary = collect_quick(os.path.abspath(root), kind, plan_path, log_path, os.path.abspath(library_root or root), pass_name or "快档")
            for record in records:
                print(record_line(record))
            print(f"  · 覆盖行与计数行：{summary}", file=sys.stderr)
            return EXIT_GREEN
        if command == "run-directory":
            tier = option_value(rest, "--tier")
            if len(rest) != 1 or tier not in ("quick", "pruned", "named", "full"):
                return usage("run-directory 要 <目录> --tier <quick|pruned|named|full>")
            for record in run_directory_records(rest[0], tier):
                print(record_line(record))
            return EXIT_GREEN
        if command == "check":
            domain = option_value(rest, "--domain")
            subset = "--subset" in rest
            rest = [argument for argument in rest if argument != "--subset"]
            if len(rest) < 3 or rest[0] not in ("coverage", "ignored", "zero-evaluations") or domain not in DOMAINS:
                return usage("check 要 <coverage|ignored|zero-evaluations> <根> --domain <quick|full> [--subset] <记录文件>…")
            cell, root, paths = rest[0], os.path.abspath(rest[1]), rest[2:]
            records = read_records(paths)
            if cell == "coverage":
                return check_coverage(root, records, domain, subset)
            if cell == "ignored":
                return check_ignored(root, records, domain)
            return check_zero_evaluations(root, records, domain)
        if command == "table":
            directory, tier = option_value(rest, "--run-directory"), option_value(rest, "--tier")
            if not rest or (directory and tier not in ("quick", "pruned", "named", "full")):
                return usage("table 要 <根> <记录文件>…，或 <根> --run-directory <目录> --tier <档>")
            root = os.path.abspath(rest[0])
            records = run_directory_records(directory, tier) if directory else read_records(rest[1:])
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
                for domain in DOMAINS:
                    for (flow, invariant), total in sorted(zero_pairs(records, domain).items()):
                        if total == 0:
                            print(f"{domain}\t{flow}\t{invariant}\t{REASON_PLACEHOLDER}")
            return EXIT_GREEN
        if command == "last-full":
            after_full = "--after-full" in rest
            rest = [argument for argument in rest if argument != "--after-full"]
            if len(rest) != 1:
                return usage("last-full 要 <读库输出文件> [--after-full]")
            with open(rest[0], encoding="utf-8", errors="replace") as handle:
                return check_last_full(handle.read(), after_full)
    except OSError as error:
        print(f"  ✗ 读不了 {error.filename}：{error.strerror}")
        print("  → 怎么办：看路径对不对、文件在不在；记录文件由 extract、collect-quick 或 run-directory 打出来")
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


def coverage_text(entry, step, segment, expansion, domain="quick"):
    return f"CRASH_FLOW_COVERAGE entry={entry} step={step} segment={segment} expansion={expansion} domain={domain}"


def sample_world(work, name, segments=SAMPLE_SEGMENTS, entries=("mount_writable", "publish_first_file"), connected=True,
                 ignored_rows=None, zero_rows=None):
    root = os.path.join(work, name)
    write(os.path.join(root, SEGMENTS_SOURCE), segments)
    write(os.path.join(root, "crates", "singlefs-checker-tier", "tests", "common_crash_points", "mod.rs"),
          'println!("CRASH_FLOW_COVERAGE entry={} ...");\n' if connected else "// 还没打覆盖行\n")
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
    "CHECKER record_root_without_record=0 I-1.1=4/0 I-2.1=0/0",
]


def run_quietly(function, *arguments):
    """调判法、吞掉它打的字，交退出码与输出。"""
    import contextlib
    import io
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        code = function(*arguments)
    return code, buffer.getvalue()


def selftest_cases(work):
    """[(格名, 判错时的一句 或 None)]。"""
    cases = []
    green_records = records_of_lines(GREEN_LOG_LINES, "sample_stream", "quick")

    def expect(name, condition, failure):
        cases.append((name, None if condition else failure))

    # 写法：五个字段、name() 的串、粘在 libtest 那一行后面、计数行的标签与两段写法
    kinds = [record[0] for record in green_records]
    expect("extract-lines", kinds == ["coverage"] * 4 + ["counts"] * 2 and green_records[4][3] == "CRASH_POINT_REPLAY_SAMPLE_FAST" and green_records[5][3] == "CHECKER",
           f"四条覆盖行、两条计数行（标签 CRASH_POINT_REPLAY_SAMPLE_FAST 与 CHECKER）应当都认出来，实际 {green_records}")
    named = records_of_lines(["CRASH_POINT_REPLAY_TREE_SPLIT stream=RootSplit states=3 I-1.1=3/0/0"], "s", "quick")
    expect("extract-stream-label", named == [("counts", "s", "quick", "CRASH_POINT_REPLAY_TREE_SPLIT/RootSplit", "I-1.1=3/0/0")],
           f"带 stream=<名> 的计数行，标签应当接成 <词>/<名>，实际 {named}")
    malformed = records_of_lines(["CRASH_FLOW_COVERAGE entry=a step=UnitWrite segment=x expansion=expanded domain=quick",
                                  "CRASH_FLOW_COVERAGE entry=a step=UnitWrite expansion=expanded segment=0 domain=quick",
                                  "CRASH_FLOW_COVERAGE_SUMMARY lines=3"], "s", "quick")
    expect("extract-malformed", [record[0] for record in malformed] == ["malformed", "malformed"],
           f"段不是数、字段换了次序的两行应当记成写法不对，CRASH_FLOW_COVERAGE_SUMMARY 不算覆盖行，实际 {malformed}")
    members, names, problem = read_step_kinds(sample_world(work, "steps"))
    expect("step-kinds-read", members == ["ZeroFill", "UnitWrite", "Barrier"] and names.get("UnitWrite") == "unit_write" and not problem,
           f"StepKind 应当读出 ZeroFill、UnitWrite、Barrier（注释里的不算）与 name() 的串，实际 {members} {names} {problem}")

    green = sample_world(work, "green", ignored_rows=["not_expanded\tsample_stream\tpublish_first_file\tUnitWrite\t2\tquick"],
                         zero_rows=["quick\tsample_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-2.1\t样本流不走二类结构",
                                    "quick\tsample_stream:CHECKER\tI-2.1\t样本流不走二类结构"])
    code, out = run_quietly(check_coverage, green, green_records, "quick", False)
    expect("coverage-green", code == EXIT_GREEN, f"登记的两个入口都有行、三个成员都罩住，coverage 应当判绿，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + [("coverage", "other", "unmount", "UnitWrite", "0", "expanded", "quick")], "quick", False)
    expect("coverage-unregistered-entry", code == EXIT_RED and "入口 unmount 不在登记表" in out, f"登记表外的入口 unmount 应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, sample_world(work, "entry-without-lines", entries=("mount_writable", "publish_first_file", "unmount")),
                            green_records, "quick", False)
    expect("coverage-entry-without-lines", code == EXIT_RED and "登记的入口 unmount" in out, f"登记了却一条行都没有的入口 unmount 应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, sample_world(work, "new-member", segments=SAMPLE_SEGMENTS.replace("    Barrier,\n", "    Barrier,\n    TrimHint,\n")),
                            green_records, "quick", False)
    expect("coverage-new-step-kind-member", code == EXIT_RED and "StepKind 成员 TrimHint" in out, f"StepKind 加了成员 TrimHint 而没有流罩住，应当判红，实际退 {code}：{out}")
    only_ignored = [record for record in green_records if not (record[0] == "coverage" and record[3] == "Barrier")] + \
                   [("coverage", "sample_stream", "publish_first_file", "Barrier", "3", "ignored_point", "quick")]
    code, out = run_quietly(check_coverage, green, only_ignored, "quick", False)
    expect("coverage-only-ignored-does-not-cover", code == EXIT_RED and "StepKind 成员 Barrier" in out,
           f"Barrier 只出现在 ignore 开着的点上，不算罩住，应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + [("coverage", "sample_stream", "mount_writable", "TrimHint", "0", "expanded", "quick")], "quick", False)
    expect("coverage-unknown-step", code == EXIT_RED and "步骤 TrimHint 不是 StepKind 的成员" in out, f"步骤不是成员应当判红，实际退 {code}：{out}")
    partial = [record for record in green_records if not (record[0] == "coverage" and record[2] == "publish_first_file")] + \
              [("unknown", "other_stream", "样本：有复用、库里没有存着的行")]
    code, out = run_quietly(check_coverage, green, partial, "quick", False)
    expect("coverage-incomplete-is-not-run", code == EXIT_NOT_RUN and "publish_first_file" in out,
           f"有测试二进制的行不全时，入口没有行只记本次未跑（退 77）、点名 publish_first_file，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, partial[:-1], "quick", True)
    expect("coverage-subset-is-not-run", code == EXIT_NOT_RUN, f"--subset 时入口没有行只记本次未跑，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records + malformed, "quick", False)
    expect("coverage-malformed-red", code == EXIT_RED and "写法不对" in out, f"写法不对的覆盖行应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, sample_world(work, "not-connected", connected=False), [], "quick", False)
    expect("coverage-not-connected", code == EXIT_NOT_RUN, f"crates 侧还没打覆盖行，应当整格退 77，实际退 {code}：{out}")
    code, out = run_quietly(check_coverage, green, green_records, "full", False)
    expect("coverage-domain-separate", code == EXIT_RED and "full 域" in out, f"行都是 quick 域的，按 full 域判应当红，实际退 {code}：{out}")

    code, out = run_quietly(check_ignored, green, green_records, "quick")
    expect("ignored-green", code == EXIT_GREEN, f"一个不展开的节点、基准里也是它，应当判绿，实际退 {code}：{out}")
    code, out = run_quietly(check_ignored, green, green_records + [("coverage", "sample_stream", "publish_first_file", "UnitWrite", "5", "registered_segment", "quick")], "quick")
    expect("ignored-one-more-node", code == EXIT_RED and "registered_segment" in out, f"比基准多一个节点应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_ignored, green, [record for record in green_records if record[5:6] != ("not_expanded",)], "quick")
    expect("ignored-one-fewer-node", code == EXIT_GREEN and "比基准少 1 个" in out, f"比基准少一个节点应当判绿并报出来，实际退 {code}：{out}")
    unseeded = sample_world(work, "unseeded", ignored_rows=[UNSEEDED_MARKER], zero_rows=[UNSEEDED_MARKER])
    code_ignored, out_ignored = run_quietly(check_ignored, unseeded, green_records, "quick")
    code_zero, out_zero = run_quietly(check_zero_evaluations, unseeded, green_records, "quick")
    expect("baselines-unseeded", code_ignored == EXIT_NOT_RUN and code_zero == EXIT_NOT_RUN and REASON_PLACEHOLDER in out_zero and "not_expanded" in out_ignored,
           f"两张基准都还写着「{UNSEEDED_MARKER}」，应当都退 77 并打候选行，实际退 {code_ignored}、{code_zero}：{out_ignored}{out_zero}")
    code, out = run_quietly(check_ignored, sample_world(work, "no-ignored-table"), green_records, "quick")
    expect("ignored-table-missing", code == EXIT_RED, f"基准不在应当判红，实际退 {code}：{out}")

    code, out = run_quietly(check_zero_evaluations, green, green_records, "quick")
    expect("zero-evaluations-green", code == EXIT_GREEN, f"两个 0 次的对都在基准里，应当判绿，实际退 {code}：{out}")
    new_zero = green_records + records_of_lines(["CRASH_POINT_REPLAY_SAMPLE_FAST I-1.1=0/0/3 I-3.1=0/0/3"], "second_stream", "quick")
    code, out = run_quietly(check_zero_evaluations, green, new_zero, "quick")
    expect("zero-evaluations-new-zero", code == EXIT_RED and "second_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-1.1" in out,
           f"一条流上 I-1.1 评估 0 次、基准里没有，应当判红，实际退 {code}：{out}")
    empty_reason = sample_world(work, "empty-reason", zero_rows=["quick\tsample_stream:CRASH_POINT_REPLAY_SAMPLE_FAST\tI-2.1\t",
                                                                 "quick\tsample_stream:CHECKER\tI-2.1\t" + REASON_PLACEHOLDER])
    code, out = run_quietly(check_zero_evaluations, empty_reason, green_records, "quick")
    expect("zero-evaluations-empty-reason", code == EXIT_RED and "理由空着" in out, f"基准里理由空着、写着占位的行应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_zero_evaluations, green, green_records, "full")
    expect("zero-evaluations-no-counts", code == EXIT_NOT_RUN, f"full 域一条计数行都没有应当退 77，实际退 {code}：{out}")

    # collect-quick：在 git 仓里建判定库目录；整份现判存、整份复用取回、一半复用而库里没有的报 unknown
    repository = os.path.join(work, "collect")
    os.makedirs(repository)
    subprocess.run(["git", "init", "-q", repository], check=True)
    plan = os.path.join(repository, "plan.out")
    log = os.path.join(repository, "quick.log")

    def plan_rows(decisions):
        write(plan, "".join(f"{name}\t{decision}\t{key}\ttarget\ttests/sample_stream.rs\t{name}\t样本\n" for name, decision, key in decisions) + "PLAN judge=0 reuse=0\n")

    log_text = "   Compiling x\n     Running tests/sample_stream.rs (target/release/deps/sample_stream-0)\nrunning 2 tests\n" + "\n".join(GREEN_LOG_LINES) + \
               "\ntest a ... ok\ntest b ... ok\n\ntest result: ok. 2 passed; 0 failed\n     Running unittests src/lib.rs (target/release/deps/lib-0)\nrunning 0 tests\n"
    plan_rows([("a", "judge", "k1"), ("b", "judge", "k2")])
    write(log, log_text)
    fresh, _summary = collect_quick(repository, "sample-kind", plan, log, repository, "样本")
    plan_rows([("a", "reuse", "k1"), ("b", "reuse", "k2")])
    write(log, "     Running tests/sample_stream.rs (target/release/deps/sample_stream-0)\nrunning 0 tests\n")
    recalled, summary = collect_quick(repository, "sample-kind", plan, log, repository, "样本")
    expect("collect-quick-recalls-reused-binary", sorted(recalled) == sorted(fresh) and len(fresh) == 6,
           f"整份现判时存下的 6 条记录，下一趟整份复用时应当原样取回，实际头一趟 {len(fresh)} 条、复用那一趟 {recalled}（{summary}）")
    plan_rows([("a", "reuse", "k1"), ("b", "judge", "k3")])
    partial_records, summary = collect_quick(repository, "sample-kind", plan, log, repository, "样本")
    expect("collect-quick-partial-reuse-is-unknown", any(record[0] == "unknown" and record[1] == "sample_stream" for record in partial_records),
           f"一半复用、库里没有这一组键存下的行，应当交一条 unknown，实际 {partial_records}（{summary}）")
    unplanned, _summary = collect_quick(repository, "sample-kind", "-", os.path.join(repository, "quick.log"), repository, "样本")
    expect("collect-quick-without-plan", not any(record[0] == "unknown" for record in unplanned), f"没有计划（全部现判）时日志里每一段都算整份，实际 {unplanned}")

    # run-directory：快档日志算 quick；轻型的放量算 quick、重型的算 full；enumeration 日志里点名的记录文件照读
    run = os.path.join(work, "run")
    write(os.path.join(run, "fast-tier.sample_stream.log"), "\n".join(GREEN_LOG_LINES) + "\n")
    write(os.path.join(run, "pipeline.log"), "SAMPLE_PIPELINE I-1.1=0/0/2\n")
    records_file = os.path.join(work, "full-records.tsv")
    write(records_file, record_line(("coverage", "sample_stream", "mount_writable", "UnitWrite", "0", "expanded", "full")) + "\n")
    write(os.path.join(run, "enumeration.sample.log"), f"  · 记在 x\n{RECORDS_FILE_LINE} {records_file}\n")
    light = run_directory_records(run, "quick")
    heavy = run_directory_records(run, "pruned")
    expect("run-directory", len(light) == 8 and ("counts", "crash-amplification", "quick", "SAMPLE_PIPELINE", "I-1.1=0/0/2") in light
           and ("counts", "crash-amplification", "full", "SAMPLE_PIPELINE", "I-1.1=0/0/2") in heavy
           and ("coverage", "sample_stream", "mount_writable", "UnitWrite", "0", "expanded", "full") in light,
           f"日志目录应当认出快档 6 条、放量 1 条（轻型 quick、重型 full）、enumeration 点名的记录文件 1 条，实际 {light}")
    table = table_text(green, light)
    expect("table", "| mount_writable | 罩住 | 罩住 |  |" in table and "| publish_first_file |  | 未展开 | 罩住 |" in table,
           f"覆盖表应当按入口 × 成员列罩住与未展开，实际：{table}")

    reader = "\t".join([LAST_FULL_TAG, "library=/library", "recorded_blocks=3", "full_under_the_current_key=1", "not_full_under_the_current_key=2", "last_full_date=2026-10-01"])
    block_lines = [f"{LAST_FULL_BLOCK_TAG}\tblock=00aa\tjudged=2026-10-02\trun_kind=pruned", f"{LAST_FULL_BLOCK_TAG}\tblock=00bb\tjudged=none\trun_kind=none"]
    code, out = run_quietly(check_last_full, "\n".join([reader] + block_lines), False)
    expect("last-full-not-current", code == EXIT_NOT_RUN and "00aa" in out and "00bb" in out and "2026-10-01" in out,
           f"两块不是当前的全量判定，应当退 77、列出两块与上一次全量的日期，实际退 {code}：{out}")
    all_current = reader.replace("full_under_the_current_key=1", "full_under_the_current_key=3").replace("not_full_under_the_current_key=2", "not_full_under_the_current_key=0")
    code_after, _out = run_quietly(check_last_full, all_current, True)
    code_quick, _out = run_quietly(check_last_full, all_current, False)
    expect("last-full-after-full", code_after == EXIT_GREEN and code_quick == EXIT_NOT_RUN,
           f"全部块都有当前的全量判定：跑了全量的那一趟判绿、没跑的退 77，实际 {code_after}、{code_quick}")
    code, out = run_quietly(check_last_full, "\n".join([reader] + block_lines[:1]), False)
    expect("last-full-block-count", code == EXIT_RED and "输出不全" in out, f"块行比 not_full_under_the_current_key 少一行，应当判红，实际退 {code}：{out}")
    code, out = run_quietly(check_last_full, "CRASH_LIBRARY_LAST_FULL\tlibrary=/library\tmissing\n", False)
    expect("last-full-library-missing", code == EXIT_NOT_RUN, f"库不在应当退 77，实际退 {code}：{out}")
    code, out = run_quietly(check_last_full, "error: no such subcommand\n", False)
    expect("last-full-unrecognized", code == EXIT_RED, f"读库输出里没有那一行应当判红，实际退 {code}：{out}")
    return cases


def selftest():
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
          f"ignore 节点比基准多一个判红、评估 0 次的新对与空理由判红、覆盖行存取、日志目录、读库输出（查了 {len(cases)} 格）")
    return EXIT_GREEN


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
