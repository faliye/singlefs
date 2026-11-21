#!/usr/bin/env python3
# admission: always 读的是这一趟要验的树此刻的登记表与必红世界表，不存结论
# run-condition: none 只用 python 标准库读文件、打印文字，不起子进程（自证样本的 sample 只在调用方给的临时树里写文件）
"""合入后验证（scripts/gate/merge-and-verify.sh）对不落测试目标的登记表的认法，只有这一份；门禁 formal-verification 的必红世界表是其中一类。

必红世界表登记在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行（第二列是表的路径，一个 crate 一行；种类说明在那份表的文件头）。
它们是门禁 formal-verification 的输入，cargo 的测试目标不读它们：补丁改到这些表，不落测试目标、不退 3，报一句；
这一批补丁在表里新加或改过的行（打上补丁之前与之后两份表逐行比：变异名新出现的，或同名而别的列变了的；删掉的行不点；变异名是第一段，
项名 must-red:<变异名>，与 scripts/gate/verus-proof-check.py 的 must_red_rows / must_red_item 同一写法）
交门禁 formal-verification 点 --item，验证段第 2 步在快照上起一条整条命令（带范围声明，写法照 .claude/agents/implementation-writer.md 第 1d 步形式化那一条）；
一张表没有新加或改过的行（补丁只删了行）的，那张表不点、不交。
打上补丁之前的那一份表在环境变量 MERGE_AND_VERIFY_BASELINE_ROOT 点名的目录里（目录里按仓根相对路径放着补丁动到的文件打补丁之前的原样；没有这个文件当原来没有这张表，行都算新加）；
没给这个目录的（--skip-apply 与 --list-items 那两趟，它们不打第 2 步）同样行都算新加。
登记照被判的那棵树打上补丁之后的 verus-pins.tsv 现读，不写死表的路径；没登记的 .tsv 不在这里认，照包里别的文件判。
另一类登记表（崩溃枚举用例、放量分项与拒算测试的登记）的路径由调用方放在环境变量 MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS（一行一个），同样不落测试目标、不退 3。

调它的：merge-and-verify.sh 的 MERGE_AND_VERIFY_ITEMS_PROGRAM（targets 一段：Registered）、merge_in_commands（命令行 second-step）、
自证（命令行 sample、judge：造样本树与补丁目录、判一趟输出）。弄坏开关由调用方的 MERGE_AND_VERIFY_BREAK 取值传进来（参数 broken，这里不读环境里的开关），
各项的意思写在 merge-and-verify.sh 文件头开关表：registration-tables-refused、formal-tables-refused、formal-tables-by-suffix、formal-items-dropped、formal-items-whole-table。

命令行：
  lib_merge_verify_formal_tables.py second-step <弄坏开关> <快照> [<表路径><TAB><项>…]   打验证段第 2 步在快照上起的几条（改到必红世界表的另有 formal-verification 那一条）
  lib_merge_verify_formal_tables.py sample <树> <补丁目录> <registered|deleted|unregistered|plain>   在树里写登记与样本表、在补丁目录里写 crates.patch
  lib_merge_verify_formal_tables.py judge <registered|deleted|unregistered|plain> <退出码> <输出文件> <日志目录>   判那一趟输出，不对的打「✗ 自检：…」、退 1
"""
import difflib
import os
import re
import shlex
import sys

PINS_PATH = ".claude/gate.d/verus-pins.tsv"
PINS_KIND = "must-red-worlds"
ITEM_PREFIX = "must-red:"
REGISTRATION_ENV = "MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS"
BASELINE_ENV = "MERGE_AND_VERIFY_BASELINE_ROOT"
REGISTRATION_NOTE = "补丁改到登记表 {path}（崩溃枚举用例、放量分项与拒算测试的登记，不落测试目标；快照里照打上之后的登记排掉登记的用例、交验证包那一步）"
FORMAL_NOTE = "补丁改到必红世界表 {path}（verus-pins.tsv 登记，门禁 formal-verification 的输入，不落测试目标；{what}）"
FORMAL_POINTED = "它新加或改过的行交门禁 formal-verification 点 --item " + ITEM_PREFIX + "<变异名>（删掉的行不点），验证段第 2 步在快照上起"
FORMAL_NOT_POINTED = "它没有新加或改过的行（只删了行或没改行内容），不交门禁 formal-verification"
SCOPE_TEXT = ("种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 {count} 项; "
              "为什么=合入后验证：补丁改到 verus-pins.tsv 登记的必红世界表，点这几张表的每一行，键相同的照复用")


def registered_tables(tree):
    """tree 里 verus-pins.tsv 的 must-red-worlds 行点名的表（相对树根的路径，登记次序）。没有这份登记交空表；读不了照抛 OSError。"""
    path = os.path.join(tree, PINS_PATH)
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8") as handle:
        rows = [line.split("\t") for line in handle.read().split("\n") if line and not line.startswith("#")]
    return [row[1].strip() for row in rows if row[0] == PINS_KIND and len(row) > 1 and row[1].strip()]


def table_rows(root, table):
    """root 里 table 的每一行 [(变异名, 整行)…]（表里的次序）。不看空行与 # 行；表在 root 里不在（或 root 是空串）交空表。"""
    path = os.path.join(root, table) if root else ""
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8") as handle:
        rows = [(line.split("\t", 1)[0], line) for line in handle.read().split("\n") if line.strip() and not line.startswith("#")]
    return [row for row in rows if row[0]]


def table_items(tree, table):
    """tree 里 table 每一行的项名（must-red:<变异名>，表里的次序、去重）。"""
    return list(dict.fromkeys(ITEM_PREFIX + name for name, _line in table_rows(tree, table)))


def changed_items(tree, baseline, table):
    """打上补丁之后 tree 里的 table 相对打补丁之前 baseline 里的那一份新加或改过的行的项名（表里的次序、去重）：
    变异名在原来的表里没有，或有这个变异名而没有一模一样的整行。删掉的行不在里面。"""
    before = {}
    for name, line in table_rows(baseline, table):
        before.setdefault(name, set()).add(line)
    return list(dict.fromkeys(ITEM_PREFIX + name for name, line in table_rows(tree, table) if line not in before.get(name, ())))


class Registered:
    """补丁改到的路径是不是不落测试目标的登记表：note_for 交一句说明（空串是不是），report 打 NOTE 与 FORMAL 行。
    登记表有两类：环境里 MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS 列的（崩溃枚举用例、放量分项、拒算测试），与 tree 里 verus-pins.tsv 登记的必红世界表。
    必红世界表交门禁点的是新加或改过的行（changed_items，打补丁之前的表在环境变量 MERGE_AND_VERIFY_BASELINE_ROOT 点名的目录里）。
    弄坏开关 registration-tables-refused：前一类照不落在任何包里的路径拒；formal-tables-refused：后一类当没登记；
    formal-tables-by-suffix：包里任何 .tsv 都当必红世界表（不读登记）；formal-items-whole-table：点改到的必红世界表的全部行。"""

    def __init__(self, tree, broken):
        self.tree, self.broken = tree, broken
        self.plain = [line for line in os.environ.get(REGISTRATION_ENV, "").split("\n") if line]
        self.formal = [] if broken == "formal-tables-refused" else registered_tables(tree)
        self.baseline = os.environ.get(BASELINE_ENV, "")
        self.items = []

    def note_for(self, path):
        if path in self.plain and self.broken != "registration-tables-refused":
            return REGISTRATION_NOTE.format(path=path)
        if path in self.formal or (self.broken == "formal-tables-by-suffix" and path.endswith(".tsv")):
            items = table_items(self.tree, path) if self.broken == "formal-items-whole-table" else changed_items(self.tree, self.baseline, path)
            self.items += [(path, item) for item in items if all(item != known for _table, known in self.items)]
            return FORMAL_NOTE.format(path=path, what=FORMAL_POINTED if items else FORMAL_NOT_POINTED)
        return ""

    def report(self, notes):
        """NOTE 行（去重）与 FORMAL 行（改到的必红世界表里新加或改过的每个项一行：FORMAL<TAB>表路径<TAB>项）。"""
        for note in dict.fromkeys(notes):
            print(f"NOTE\t{note}")
        for table, item in self.items:
            print(f"FORMAL\t{table}\t{item}")


def formal_lines(snapshot, entries):
    """改到必红世界表的那一条 formal-verification：前言一行、整条命令一行（在快照上交，--cwd 与根都写它）。entries 是 [(表路径, 项)…]，空的不打。"""
    if not entries:
        return []
    tables = "".join(f"「{table}」" for table in dict.fromkeys(table for table, _item in entries))
    items = " ".join(f"--item {shlex.quote(item)}" for _table, item in entries)
    return [f"    这一批改到必红世界表 {tables}（verus-pins.tsv 登记，门禁 formal-verification 的输入），点它们的 {len(entries)} 项（点名单跑与整格里同一行的判定相同，键相同的照复用）：",
            f"    SINGLEFS_RUN_SCOPE='{SCOPE_TEXT.format(count=len(entries))}' task-server/task submit --cwd {shlex.quote(snapshot)} -- "
            f"bash .claude/gate.d/formal-verification.sh {items} {shlex.quote(snapshot)}"]


def second_step_lines(broken, snapshot, entries):
    """验证段第 2 步在快照上起的几条（--cwd 与命令里的 <根> 都写快照）：crash-case-check.py、refused-test-registry.py list，改到必红世界表的另加 formal-verification。
    弄坏开关 formal-items-dropped：不打 formal-verification 那一条。"""
    quoted = shlex.quote(snapshot)
    return ([f"  · 验证段第 2 步在这一趟留下的快照 {snapshot} 上交（--cwd 与命令里的 <根> 都写它；.claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 2 步），都绿了才合：",
             f"    task-server/task submit --cwd {quoted} -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red {quoted}",
             f"    task-server/task submit --cwd {quoted} -- python3 scripts/gate/refused-test-registry.py list {quoted}"]
            + ([] if broken == "formal-items-dropped" else formal_lines(snapshot, entries)))


ALPHA_TABLE = "crates/alpha/verus-must-red-worlds.tsv"
BETA_TABLE = "crates/beta/verus-must-red-worlds.tsv"
UNREGISTERED_TABLE = "crates/alpha/other-table.tsv"
PLAIN_TABLE = ".claude/gate.d/refused-tests.tsv"
TEST_TARGET_FILE = "crates/alpha/src/merge_table.txt"
KEPT_ROW, EDITED_ROW, REMOVABLE_ROW, ADDED_ROW = "alpha-row-kept", "alpha-row-edited", "alpha-row-removable", "alpha 行新加"
BETA_ROW = "beta-row-one"
SAMPLE_SOURCE = "crates/alpha/src/lib.rs"
MOVED_SOURCE = "crates/alpha/src/lib/index.rs"
SAMPLE_KINDS = ("registered", "deleted", "unregistered", "plain")
TITLES = {"registered": "必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表", "deleted": "必红世界表：补丁只删一行的那张表不点、不交 formal-verification",
          "unregistered": "必红世界表：补丁改到包里没登记的 .tsv", "plain": "挑测试目标：补丁改到登记表时不落测试目标、不退 3"}


def sample_table(rows):
    """rows 是 [(变异名, 文件)…]。"""
    return "# 样本表：变异名、文件、原文、替换文、proof_errors=、landing=\n" + "".join(f"{name}\t{source}\tpub fn f() {{}}\tpub fn f() {{ }}\tproof_errors=1\tlanding=f\n" for name, source in rows)


def unified(path, before, after):
    return "".join(difflib.unified_diff(before.splitlines(True), after.splitlines(True), f"a/{path}", f"b/{path}"))


def write_sample(tree, patch_directory, kind):
    """样本：树里写登记（alpha、beta 两张必红世界表）、alpha 那张表三行（没改的、要改的、要删的）、beta 那张一行、一份没登记的 .tsv、一份拒算表
    （崩溃枚举用例等那一类登记表），补丁目录的 crates.patch 照 kind 改，每种都另加 alpha 的一份数据文件（让测试那一步有一项）。
    registered：alpha 那张表改一行（要改的那行的文件挪到 lib/index.rs）、末尾新加一行，没改的与要删的两行留着；deleted：alpha 那张表只删要删的那一行；
    unregistered：没登记的 .tsv；plain：拒算表。"""
    before = [(KEPT_ROW, SAMPLE_SOURCE), (EDITED_ROW, SAMPLE_SOURCE), (REMOVABLE_ROW, SAMPLE_SOURCE)]
    for path, text in ((PINS_PATH, "# 样本\n" + "".join(f"{PINS_KIND}\t{table}\n" for table in (ALPHA_TABLE, BETA_TABLE))),
                       (ALPHA_TABLE, sample_table(before)), (BETA_TABLE, sample_table([(BETA_ROW, SAMPLE_SOURCE)])),
                       (UNREGISTERED_TABLE, "1\n"), (PLAIN_TABLE, "x\n")):
        os.makedirs(os.path.dirname(os.path.join(tree, path)), exist_ok=True)
        with open(os.path.join(tree, path), "w", encoding="utf-8") as handle:
            handle.write(text)
    edited_and_added = [(name, MOVED_SOURCE if name == EDITED_ROW else source) for name, source in before] + [(ADDED_ROW, SAMPLE_SOURCE)]
    patch = {"registered": unified(ALPHA_TABLE, sample_table(before), sample_table(edited_and_added)),
             "deleted": unified(ALPHA_TABLE, sample_table(before), sample_table(before[:-1])),
             "unregistered": unified(UNREGISTERED_TABLE, "1\n", "2\n"), "plain": unified(PLAIN_TABLE, "x\n", "y\n")}[kind] + unified(TEST_TARGET_FILE, "1\n", "2\n")
    os.makedirs(patch_directory, exist_ok=True)
    with open(os.path.join(patch_directory, "crates.patch"), "w", encoding="utf-8") as handle:
        handle.write(patch)


def judge_run(kind, rc, output, logs):
    """合入后验证一趟（不带 --skip-apply、留着快照）的输出对不对，交不对的说明（空串是对）。"""
    lines = output.split("\n")
    refused = "✗ 测试那一步与验证包那一步没跑" in output
    summary = next((number for number, line in enumerate(lines) if line.startswith("  汇总：")), None)
    commands = [(number, line) for number, line in enumerate(lines) if line.startswith("    SINGLEFS_RUN_SCOPE=") and "formal-verification.sh" in line]
    seen = f"实际退出码 {rc}，汇总行在 {summary}，formal-verification 的命令 {len(commands)} 条：{matches(lines, '登记表|必红世界表|判不出|formal-verification|没跑|✗')}"
    if kind == "unregistered":
        if rc == 3 and refused and "补丁改到 alpha 包里的 other-table.tsv，判不出改到哪个测试目标" in output and not commands:
            return ""
        return f"没登记的 .tsv 应当照旧判不出、退 3、不交 formal-verification，{seen}"
    if kind == "plain":
        if rc == 0 and f"补丁改到登记表 {PLAIN_TABLE}" in output and f"不落在任何包里的 {PLAIN_TABLE}" not in output and not commands:
            return ""
        return f"拒算表这一类登记表应当不落测试目标、不退 3、报一句、不交 formal-verification，{seen}"
    if kind == "deleted":
        if (rc == 0 and f"补丁改到必红世界表 {ALPHA_TABLE}" in output and FORMAL_NOT_POINTED in output and not refused and summary is not None and not commands):
            return ""
        return f"补丁只删一行应当不落测试目标、不退 3、报一句没有新加或改过的行，汇总行之后没有 formal-verification（那张表不点），{seen}"
    quoted = shlex.quote(os.path.join(logs, "repo"))
    wanted = (f"task-server/task submit --cwd {quoted} -- bash .claude/gate.d/formal-verification.sh --item {ITEM_PREFIX}{EDITED_ROW} --item {shlex.quote(ITEM_PREFIX + ADDED_ROW)} {quoted}",
              "范围=点名的 2 项", "种类=items")
    if (rc == 0 and f"补丁改到必红世界表 {ALPHA_TABLE}" in output and not refused and summary is not None and len(commands) == 1
            and commands[0][0] > summary and all(text in commands[0][1] for text in wanted)
            and not any(name in commands[0][1] for name in (KEPT_ROW, REMOVABLE_ROW, BETA_ROW))):
        return ""
    return f"应当不落测试目标、不退 3、报一句，汇总行之后有一条只点 alpha 那张表改过的一行与新加的一行的 formal-verification（不点没改的两行，不点没改到的 beta 那张），{seen}"


def matches(lines, pattern):
    return " ".join(line.strip() for line in lines if re.search(pattern, line))[:400]


def main(arguments):
    mode = arguments[0] if arguments else ""
    if mode == "second-step" and len(arguments) >= 3:
        print("\n".join(second_step_lines(arguments[1], arguments[2], [tuple(word.split("\t", 1)) for word in arguments[3:]])))
        return 0
    if mode == "sample" and len(arguments) == 4 and arguments[3] in SAMPLE_KINDS:
        write_sample(arguments[1], arguments[2], arguments[3])
        return 0
    if mode == "judge" and len(arguments) == 5 and arguments[1] in SAMPLE_KINDS:
        with open(arguments[3], encoding="utf-8", errors="replace") as handle:
            problem = judge_run(arguments[1], int(arguments[2]), handle.read(), arguments[4])
        if problem:
            print(f"  ✗ 自检：{TITLES[arguments[1]]}：{problem}")
        return 1 if problem else 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
