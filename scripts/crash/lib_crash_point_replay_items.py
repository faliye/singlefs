#!/usr/bin/env python3
# admission: always 判的是此刻的合表、按段登记文件、stage-inputs.tsv 与 crash-amplification.sh 的 ITEMS，几毫秒读完；每次改其中一份都现判
# run-condition: none 只读仓里的三份文本，除了 python3 之外没有环境要求
"""崩溃点重放合表的读法与核法（一份判法，入口 scripts/crash-point-replay.sh 与 scripts/crash-case-check.py 的 placement 共用）。

合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头）：一项一条流，点名它在重型里的两部分——
  层 0 那一套（.claude/gate.d/stage-inputs.tsv 的 crash-case: 行）与放量那一套（scripts/crash-amplification.sh 的 ITEMS），
  再写轻型取自重型的哪几个测试目标。

用法：
    lib_crash_point_replay_items.py list [仓根]      逐行打「项名<TAB>crash-case 键（空格隔开，没有是 -）<TAB>放量分项（同上）<TAB>轻型的测试目标」；
                                                     合表核不过时不打，打问题、退 1
    lib_crash_point_replay_items.py check [仓根]     只核，核过打一句成功句（核了几项、几条 crash-case、几个放量分项、几个轻型测试目标）
    lib_crash_point_replay_items.py segments [仓根]  只核按段登记文件（见下「按段登记」），核过打一句成功句（几行、几项）；门禁 code-source-discipline 的
                                                     crash-point-replay-segments 格与入口 scripts/crash-point-replay.sh 开跑前都调它
  被 import 时用 read_items(root)、crash_case_rows(root)、pipeline_items(root)、problems(root)、crash_cases_named(root)、
  registered_functions(root)（合表点名的重型部分落到哪几个「测试目标、用例函数」，crash-case-check.py 的 placement 用）、segment_problems(root)；
  核法的判别力由 selftest_cells() 拿造出来的小仓核，它由 scripts/crash-case-check.py --selftest 起（placement 读的就是这一份核法）。

按段登记（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J10、M4 第 15 条；与引擎那一路实现 E2 的接口约定）：
  文件 .claude/gate.d/crash-point-replay-segments.tsv，一行 <项名>\t<段号>\t<段写数>\t<段写表摘要>（可再加一段 # 说明），# 开头的行是注释。
  入口开跑时往引擎送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<这份文件的绝对路径>，登记了的段由引擎在重型里按「甲二」展开。
  核法（segment_problems）：文件读得了；每行四段；项名是合表里的一项；段号与段写数是非负整数（十进制、不带前导 0）、段写数大于 0；
  摘要是 64 位小写十六进制（sha256）；同一项同一段只一行。合表读不了照合表的问题报。

核法（problems）：
  ① 每行四段（再加 # 说明）；项名是小写字母、数字、-，不重复，不含 layer0、amplification（判决第三节 J1：新名字不用这两个词）；
  ② 第二段的每个键是 stage-inputs.tsv 里一行 crash-case: 的键，第三段的每一项是 ITEMS 里的分项名；两段不能都是 -；
  ③ stage-inputs.tsv 的每一条 crash-case: 与 ITEMS 的每一项都恰好被一项点名（漏了的、两项都点名的都红）；
  ④ 第四段（轻型取自重型的哪几个测试目标）不空，每一个都是这一项第二、三段点名的部分的测试目标之一，
     crates/singlefs-checker-tier/tests/<目标>.rs 在（门禁核「轻型登记的每一行都指得到重型里现存的那一项」，用户 2026-10-10 定）。
  合表或 stage-inputs.tsv 读不了算一处问题，不抛；放量入口 crash-amplification.sh 不在当成 ITEMS 是空的
  （合表点名了放量分项的那几项照样报「不是 ITEMS 里的分项」；层 0 门禁的样本根里只有合表、没有放量入口）。
退出码：0 核过（list 打全表）；1 有问题（逐处列）；2 用法错。

弄坏开关（只给自证证红用）。逐项写开关名、弄坏哪一处、打开后 scripts/crash-case-check.py --selftest 哪几格红
（格名是 selftest_cells() 里的格名，那份自证打「✗ 自检：合表核法 <格名>：…」）：
  CRASH_POINT_REPLAY_ITEMS_BREAK=unknown-part-accepted  ② 不核点名的键与分项在不在
      unknown-crash-case、unknown-pipeline-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=unreferenced-accepted  ③ 不核每条 crash-case 与每个分项都被点名
      unreferenced-crash-case、unreferenced-pipeline-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=duplicate-reference-accepted  ③ 不核同一条被两项点名
      crash-case-named-twice
  CRASH_POINT_REPLAY_ITEMS_BREAK=light-outside-heavy-accepted  ④ 不核轻型的测试目标在不在这一项的重型部分里
      light-outside-heavy
  CRASH_POINT_REPLAY_ITEMS_BREAK=retired-words-accepted  ① 不核项名里的退役词
      retired-word-in-name
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-unknown-item-accepted  按段登记不核项名在不在合表里
      segments-unknown-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-numbers-unchecked  按段登记不核段号与段写数（非负整数、段写数大于 0）
      segments-bad-segment-number、segments-zero-writes
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-digest-unchecked  按段登记不核摘要是 64 位小写十六进制
      segments-bad-digest
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-duplicate-accepted  按段登记不核同一项同一段只一行
      segments-duplicate
"""
import os
import re
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(SCRIPT_DIRECTORY)
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("CRASH_POINT_REPLAY_ITEMS_BREAK", "")
ITEMS_TABLE = os.path.join(".claude", "gate.d", "crash-point-replay-items.tsv")
REGISTRATION_TABLE = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
PIPELINE_ENTRY = os.path.join("scripts", "crash-amplification.sh")
TIER_TESTS = os.path.join("crates", "singlefs-checker-tier", "tests")
ITEM_NAME = re.compile(r"^[a-z0-9][a-z0-9-]*$")
RETIRED_WORDS = ("layer0", "amplification")
CRASH_CASE_ROW = re.compile(r"^(crash-case:[a-z0-9][a-z0-9-]*)\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
PIPELINE_ITEM = re.compile(r"^\s*\$'([a-z0-9][a-z0-9-]*)\\t(\w+)\\t(\w+)\\t[^']*'\s*$")
SEGMENTS_TABLE = os.path.join(".claude", "gate.d", "crash-point-replay-segments.tsv")
NONNEGATIVE_INTEGER = re.compile(r"^(?:0|[1-9][0-9]*)$")
SEGMENT_DIGEST = re.compile(r"^[0-9a-f]{64}$")


class Item:
    def __init__(self, line_number, name, crash_cases, pipeline_items, light_targets):
        self.line_number = line_number
        self.name = name
        self.crash_cases = crash_cases
        self.pipeline_items = pipeline_items
        self.light_targets = light_targets


def words_of(field):
    return [] if field.strip() in ("", "-") else field.split()


def read_items(root):
    """→ (项的列表, 问题)。合表读不了是一处问题。"""
    path = os.path.join(root, ITEMS_TABLE)
    items, problems = [], []
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了合表 {ITEMS_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        body = line.split("\t#", 1)[0]
        fields = body.split("\t")
        if len(fields) != 4:
            problems.append(f"{ITEMS_TABLE}:{number} 是 {len(fields)} 段，要四段（项名、crash-case 键、放量分项、轻型的测试目标，制表符分隔）")
            continue
        items.append(Item(number, fields[0].strip(), words_of(fields[1]), words_of(fields[2]), words_of(fields[3])))
    return items, problems


def crash_case_rows(root):
    """stage-inputs.tsv 里 crash-case: 的行：{键: (测试目标, 用例函数)}；读不了抛 OSError。"""
    rows = {}
    for line in open(os.path.join(root, REGISTRATION_TABLE), encoding="utf-8"):
        match = CRASH_CASE_ROW.match(line)
        if match:
            rows[match.group(1)] = (match.group(3), match.group(4))
    return rows


def pipeline_items(root):
    """crash-amplification.sh 的 ITEMS：{分项名: (测试目标, 用例函数)}；文件不在交空表（文件头「核法」末句），在而读不了抛 OSError。"""
    found, inside = {}, False
    path = os.path.join(root, PIPELINE_ENTRY)
    if not os.path.exists(path):
        return found
    for line in open(path, encoding="utf-8"):
        if line.startswith("ITEMS=("):
            inside = True
            continue
        if inside and line.startswith(")"):
            break
        if inside:
            match = PIPELINE_ITEM.match(line)
            if match:
                found[match.group(1)] = (match.group(2), match.group(3))
    return found


def problems(root):
    """合表的问题清单（文件头「核法」）。"""
    items, found = read_items(root)
    try:
        cases = crash_case_rows(root)
    except OSError as error:
        return found + [f"读不了 {REGISTRATION_TABLE}：{error}"]
    try:
        pipeline_rows = pipeline_items(root)
    except OSError as error:
        return found + [f"读不了 {PIPELINE_ENTRY}：{error}"]
    seen_names = set()
    named_cases, named_pipeline = {}, {}
    for item in items:
        where = f"{ITEMS_TABLE}:{item.line_number} {item.name}"
        if not ITEM_NAME.match(item.name):
            found.append(f"{where}：项名只许小写字母、数字与 -")
        if item.name in seen_names:
            found.append(f"{where}：项名重复")
        seen_names.add(item.name)
        if BROKEN != "retired-words-accepted":
            for word in RETIRED_WORDS:
                if word in item.name:
                    found.append(f"{where}：项名里有退役的词 {word}（新名字用「崩溃点重放」那一族，判决第三节 J1）")
        if not item.crash_cases and not item.pipeline_items:
            found.append(f"{where}：第二、三段都是 -，这一项在重型里没有部分")
        heavy_targets = set()
        for key in item.crash_cases:
            named_cases.setdefault(key, []).append(item.name)
            if key in cases:
                heavy_targets.add(cases[key][0])
            elif BROKEN != "unknown-part-accepted":
                found.append(f"{where}：第二段的 {key} 不是 {REGISTRATION_TABLE} 里 crash-case: 的键")
        for name in item.pipeline_items:
            named_pipeline.setdefault(name, []).append(item.name)
            if name in pipeline_rows:
                heavy_targets.add(pipeline_rows[name][0])
            elif BROKEN != "unknown-part-accepted":
                found.append(f"{where}：第三段的 {name} 不是 {PIPELINE_ENTRY} 的 ITEMS 里的分项")
        if not item.light_targets:
            found.append(f"{where}：第四段（轻型取自重型的哪几个测试目标）是空的")
        for target in item.light_targets:
            if target not in heavy_targets and BROKEN != "light-outside-heavy-accepted":
                found.append(f"{where}：轻型的测试目标 {target} 不在这一项重型部分的测试目标里（{'、'.join(sorted(heavy_targets)) or '无'}）：轻型要取自重型")
            if not os.path.isfile(os.path.join(root, TIER_TESTS, target + ".rs")):
                found.append(f"{where}：轻型的测试目标 {target} 没有 {TIER_TESTS}/{target}.rs")
    if BROKEN != "unreferenced-accepted":
        for key in sorted(set(cases) - set(named_cases)):
            found.append(f"{REGISTRATION_TABLE} 的 {key} 没有被合表里哪一项点名（一项一条流：放进它那条流的那一项的第二段，或另起一项）")
        for name in sorted(set(pipeline_rows) - set(named_pipeline)):
            found.append(f"{PIPELINE_ENTRY} 的 ITEMS 里的 {name} 没有被合表里哪一项点名（放进它那条流的那一项的第三段，或另起一项）")
    if BROKEN != "duplicate-reference-accepted":
        for part, owners in sorted(list(named_cases.items()) + list(named_pipeline.items())):
            if len(owners) > 1:
                found.append(f"{part} 被合表里 {len(owners)} 项点名（{'、'.join(owners)}）：一条只许归一项")
    return found


def crash_cases_named(root):
    """合表点名了的 crash-case: 键的集合（placement 用）；合表读不了是空集。"""
    items, _ = read_items(root)
    return {key for item in items for key in item.crash_cases}


def registered_functions(root):
    """合表点名的重型部分落到的 {(测试目标, 用例函数)}：点名的 crash-case: 行的 test=，点名的放量分项在 ITEMS 里的测试目标与用例函数。
    读不了 stage-inputs.tsv 抛 OSError；合表读不了是空集（placement 另经 problems() 报那一处）。"""
    items, _ = read_items(root)
    cases = crash_case_rows(root)
    pipeline_rows = pipeline_items(root)
    found = set()
    for item in items:
        found.update(cases[key] for key in item.crash_cases if key in cases)
        found.update(pipeline_rows[name] for name in item.pipeline_items if name in pipeline_rows)
    return found


def read_segments(root):
    """按段登记文件的行：→ ([(行号, 项名, 段号, 段写数, 摘要)], 问题)。文件读不了、段数不对各是一处问题。"""
    rows, found = [], []
    try:
        lines = open(os.path.join(root, SEGMENTS_TABLE), encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了按段登记文件 {SEGMENTS_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t#", 1)[0].split("\t")
        if len(fields) != 4:
            found.append(f"{SEGMENTS_TABLE}:{number} 是 {len(fields)} 段，要四段（项名、段号、段写数、段写表摘要，制表符分隔）")
            continue
        rows.append((number, *(field.strip() for field in fields)))
    return rows, found


def segment_problems(root):
    """按段登记文件的问题清单（文件头「按段登记」）。"""
    rows, found = read_segments(root)
    items, item_problems = read_items(root)
    if item_problems:
        found += [f"核按段登记要读合表：{problem}" for problem in item_problems]
    names = {item.name for item in items}
    seen = {}
    for number, name, segment, writes, digest in rows:
        where = f"{SEGMENTS_TABLE}:{number}"
        if name not in names and BROKEN != "segment-unknown-item-accepted":
            found.append(f"{where}：项名 {name} 不是合表 {ITEMS_TABLE} 里的一项")
        if BROKEN != "segment-numbers-unchecked":
            if not NONNEGATIVE_INTEGER.match(segment):
                found.append(f"{where}：段号「{segment}」不是非负整数（十进制、不带前导 0）")
            if not NONNEGATIVE_INTEGER.match(writes) or writes == "0":
                found.append(f"{where}：段写数「{writes}」要是大于 0 的整数（十进制、不带前导 0）")
        if not SEGMENT_DIGEST.match(digest) and BROKEN != "segment-digest-unchecked":
            found.append(f"{where}：段写表摘要「{digest}」不是 64 位小写十六进制（sha256）")
        if (name, segment) in seen and BROKEN != "segment-duplicate-accepted":
            found.append(f"{where}：{name} 第 {segment} 段已在第 {seen[(name, segment)]} 行登记过，同一项同一段只一行")
        seen.setdefault((name, segment), number)
    return found


def command_segments(root):
    found = segment_problems(root)
    if found:
        for problem in found:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 按段登记文件 {SEGMENTS_TABLE} 有 {len(found)} 处不对（逐处列在上面）")  # gate-lint:summary
        print(f"  → 怎么办：照那几处改 {SEGMENTS_TABLE}（写法在它的表头与 scripts/lib_crash_point_replay_items.py 文件头「按段登记」）：项名照合表写，"
              "段号与段写数写十进制整数、段写数大于 0，摘要写段写表的 sha256（64 位小写十六进制），同一项同一段只留一行")
        return 1
    rows, _ = read_segments(root)
    print(f"  ✓ 崩溃点重放按段登记核过：{len(rows)} 行（{len({row[1] for row in rows})} 项），项名都在合表里、段号与段写数是非负整数且段写数大于 0、"
          "摘要是 64 位小写十六进制、同一项同一段只一行")
    return 0


def report_problems(found):
    for problem in found:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 崩溃点重放合表 {ITEMS_TABLE} 有 {len(found)} 处不对（逐处列在上面）")  # gate-lint:summary
    print(f"  → 怎么办：照那几处改合表（写法在 {ITEMS_TABLE} 表头）；新加的 crash-case: 行或放量分项要放进它那条流的那一项")


def command_list(root):
    found = problems(root)
    if found:
        report_problems(found)
        return 1
    items, _ = read_items(root)
    for item in items:
        print("\t".join([item.name, " ".join(item.crash_cases) or "-", " ".join(item.pipeline_items) or "-", " ".join(item.light_targets)]))
    return 0


def command_check(root):
    found = problems(root)
    if found:
        report_problems(found)
        return 1
    items, _ = read_items(root)
    case_count = sum(len(item.crash_cases) for item in items)
    pipeline_count = sum(len(item.pipeline_items) for item in items)
    light_count = sum(len(item.light_targets) for item in items)
    print(f"  ✓ 崩溃点重放合表核过：{len(items)} 项，点名 {case_count} 条 crash-case 与 {pipeline_count} 个放量分项（各恰好一次），"
          f"轻型 {light_count} 个测试目标都取自同一项的重型部分")
    return 0


SELFTEST_TABLE_HEADER = "# 样本合表\n"


def selftest_cells():
    """拿造出来的小仓核 problems() 的每一条核法：→ (格名的列表, [(格名, 判错的一句)])。CRASH_POINT_REPLAY_ITEMS_BREAK 设着时点名的格判错。"""
    work = tempfile.mkdtemp(prefix="crash-point-replay-items-selftest-")
    failures, cells = [], []

    def world(name, table_rows, crash_cases, pipeline_rows, targets):
        root = os.path.join(work, name)
        os.makedirs(os.path.join(root, ".claude", "gate.d"))
        os.makedirs(os.path.join(root, "scripts"))
        os.makedirs(os.path.join(root, TIER_TESTS))
        with open(os.path.join(root, ITEMS_TABLE), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_TABLE_HEADER + "".join("\t".join(row) + "\t# 样本\n" for row in table_rows))
        with open(os.path.join(root, REGISTRATION_TABLE), "w", encoding="utf-8") as handle:
            for key, target in crash_cases:
                handle.write(f"{key}\tcrates/\ttest=singlefs-checker-tier:{target}:the_case count-line=X\t# 样本\n")
        with open(os.path.join(root, PIPELINE_ENTRY), "w", encoding="utf-8") as handle:
            handle.write("ITEMS=(\n" + "".join(f"  $'{name}\\t{target}\\tthe_function\\t全量'\n" for name, target in pipeline_rows) + ")\n")
        for target in targets:
            open(os.path.join(root, TIER_TESTS, target + ".rs"), "w").close()
        return root

    def expect(cell, root, wanted):
        """wanted 是 None：要 0 处问题；是串：要有一处问题含它。"""
        cells.append(cell)
        found = problems(root)
        if wanted is None and found:
            failures.append((cell, f"应当核过，实际 {found}"))
        elif wanted is not None and not any(wanted in problem for problem in found):
            failures.append((cell, f"应当报一处含「{wanted}」的问题，实际 {found}"))

    cases = [("crash-case:stream-one", "stream_one"), ("crash-case:stream-two", "stream_two")]
    pipeline_rows = [("stream-two-piped", "stream_two_piped")]
    targets = ["stream_one", "stream_two", "stream_two_piped"]
    good = [["stream-one", "crash-case:stream-one", "-", "stream_one"],
            ["stream-two", "crash-case:stream-two", "stream-two-piped", "stream_two stream_two_piped"]]
    expect("clean", world("clean", good, cases, pipeline_rows, targets), None)
    expect("unknown-crash-case", world("unknown-crash-case", good + [["ghost", "crash-case:ghost", "-", "stream_one"]], cases, pipeline_rows, targets),
           "不是 .claude/gate.d/stage-inputs.tsv 里 crash-case: 的键")
    expect("unknown-pipeline-item", world("unknown-pipeline-item", good + [["ghost", "-", "ghost-piped", "stream_one"]], cases, pipeline_rows, targets),
           "不是 scripts/crash-amplification.sh 的 ITEMS 里的分项")
    expect("unreferenced-crash-case", world("unreferenced-crash-case", good[1:], cases, pipeline_rows, targets), "crash-case:stream-one 没有被合表里哪一项点名")
    expect("unreferenced-pipeline-item",
           world("unreferenced-pipeline-item", [good[0], ["stream-two", "crash-case:stream-two", "-", "stream_two"]], cases, pipeline_rows, targets),
           "stream-two-piped 没有被合表里哪一项点名")
    expect("crash-case-named-twice", world("crash-case-named-twice", good + [["stream-one-again", "crash-case:stream-one", "-", "stream_one"]], cases, pipeline_rows, targets),
           "crash-case:stream-one 被合表里 2 项点名")
    expect("light-outside-heavy", world("light-outside-heavy", [good[0], ["stream-two", "crash-case:stream-two", "stream-two-piped", "stream_two stream_one"]],
                                        cases, pipeline_rows, targets), "轻型的测试目标 stream_one 不在这一项重型部分的测试目标里")
    expect("retired-word-in-name", world("retired-word-in-name", [["layer0-stream-one", "crash-case:stream-one", "-", "stream_one"], good[1]], cases, pipeline_rows, targets),
           "项名里有退役的词 layer0")
    expect("light-test-file-missing", world("light-test-file-missing", good, cases, pipeline_rows, ["stream_two", "stream_two_piped"]),
           "没有 crates/singlefs-checker-tier/tests/stream_one.rs")
    expect("both-parts-empty", world("both-parts-empty", good + [["nothing", "-", "-", "stream_one"]], cases, pipeline_rows, targets), "第二、三段都是 -")
    entry_absent = world("pipeline-entry-absent", [good[0], ["stream-two", "crash-case:stream-two", "-", "stream_two"]], cases, [], targets)
    os.remove(os.path.join(entry_absent, PIPELINE_ENTRY))
    expect("pipeline-entry-absent", entry_absent, None)
    named_root = world("registered-functions", good, cases, pipeline_rows, targets)
    cells.append("registered-functions")
    wanted_functions = {("stream_one", "the_case"), ("stream_two", "the_case"), ("stream_two_piped", "the_function")}
    if registered_functions(named_root) != wanted_functions:
        failures.append(("registered-functions", f"合表点名的重型部分应当落到 {sorted(wanted_functions)}，实际 {sorted(registered_functions(named_root))}"))

    # 按段登记（文件头「按段登记」）：每格一份小仓（合表就是上面那两项），只换按段登记文件
    digest = "0123456789abcdef" * 4

    def segments_world(name, segment_rows):
        root = world(name, good, cases, pipeline_rows, targets)
        if segment_rows is not None:
            with open(os.path.join(root, SEGMENTS_TABLE), "w", encoding="utf-8") as handle:
                handle.write("# 样本按段登记\n" + "".join("\t".join(row) + "\n" for row in segment_rows))
        return root

    def expect_segments(cell, root, wanted):
        cells.append(cell)
        found = segment_problems(root)
        if wanted is None and found:
            failures.append((cell, f"按段登记应当核过，实际 {found}"))
        elif wanted is not None and not any(wanted in problem for problem in found):
            failures.append((cell, f"按段登记应当报一处含「{wanted}」的问题，实际 {found}"))

    clean_segments = [["stream-one", "0", "7", digest], ["stream-one", "2", "12", digest + "\t# 说明"], ["stream-two", "0", "3", digest]]
    expect_segments("segments-clean", segments_world("segments-clean", clean_segments), None)
    expect_segments("segments-header-only", segments_world("segments-header-only", []), None)
    expect_segments("segments-unknown-item", segments_world("segments-unknown-item", clean_segments + [["ghost", "0", "3", digest]]),
                    "项名 ghost 不是合表")
    expect_segments("segments-bad-segment-number", segments_world("segments-bad-segment-number", [["stream-one", "-1", "3", digest]]),
                    "段号「-1」不是非负整数")
    expect_segments("segments-zero-writes", segments_world("segments-zero-writes", [["stream-one", "0", "0", digest]]),
                    "段写数「0」要是大于 0 的整数")
    expect_segments("segments-bad-digest", segments_world("segments-bad-digest", [["stream-one", "0", "3", digest.upper()]]),
                    "不是 64 位小写十六进制")
    expect_segments("segments-duplicate", segments_world("segments-duplicate", clean_segments + [["stream-one", "2", "9", digest]]),
                    "stream-one 第 2 段已在第 3 行登记过")
    expect_segments("segments-field-count", segments_world("segments-field-count", [["stream-one", "0", "3"]]), "是 3 段，要四段")
    expect_segments("segments-table-missing", segments_world("segments-table-missing", None), "读不了按段登记文件")
    shutil.rmtree(work, ignore_errors=True)
    return cells, failures


def main(arguments):
    if arguments and arguments[0] in ("list", "check", "segments") and len(arguments) <= 2:
        root = os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT
        return {"list": command_list, "check": command_check, "segments": command_segments}[arguments[0]](root)
    print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
    print("  → 怎么办：lib_crash_point_replay_items.py list [仓根] | check [仓根] | segments [仓根]（核法的自证在 scripts/crash-case-check.py --selftest）")
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
