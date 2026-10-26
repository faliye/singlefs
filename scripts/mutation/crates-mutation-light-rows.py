#!/usr/bin/env python3
# admission: always 判的是此刻的 crates/mutations.tsv 与轻型变异登记表，几毫秒读完；两份有一份改了都要现判
# run-condition: none 只读仓里的两份文本，除了 python3 之外没有环境要求
"""crates 变异表的轻型登记表 .claude/gate.d/crates-mutation-light-rows.tsv 的读法与核法（一份判法：门禁 code-source-discipline 的
mutation-light-rows 格调 check，门禁 checker-tier-crates-mutation-replay 的 --quick 调 names）。

登记表写法在它自己的表头：一行「<变异名><TAB><测试目标>」，测试目标是那一行在 crates/mutations.tsv 第五列（cargo test 参数）里 ` -- ` 之前那一段，
轻型那一行取自重型（整表）的哪一行就由变异名点名、测试目标写明（用户 2026-10-10 定：轻型登记的每一行写明取自重型的哪一项）。
核心部分只取核心代码（用户 2026-10-10 定）：改坏对象（变异表第二列）在 crates/singlefs-core/src/ 下的行，里面每个测试目标固定一行。

用法：
    crates-mutation-light-rows.py check [仓根]    只核，核过打一句成功句（几行、几个测试目标）；核不过逐处列、退 1
    crates-mutation-light-rows.py names [仓根]    核过才打：一行一个变异名（门禁 checker-tier-crates-mutation-replay --quick 照它点名）；核不过同 check
    crates-mutation-light-rows.py --selftest      拿造出来的小仓核每一条核法；CRATES_MUTATION_LIGHT_ROWS_BREAK=<项> 时点名的格必须红

核法（problems）：
  ① 登记表每一行两段（再加 # 说明）、变异名不空、同一个变异名不登记两次；
  ② 点名的变异名在 crates/mutations.tsv 里有成形的一行（成形照 scripts/mutation/crates-mutation-rows.py 的 read_table，同一份判法）；
  ③ 写的测试目标与那一行第五列 ` -- ` 之前那一段相同；
  ④ crates/mutations.tsv 里改坏对象在 crates/singlefs-core/src/ 下的成形的行，它们出现过的每个测试目标，登记表里恰好一行：缺的逐个列出、
     并给出照挑法该补的那一行（那些核心行里行号最小的那一条）；一个测试目标登记了两行以上的也红。核心行之外的测试目标不要求登记；
  ⑤ 登记的每一行，它点名的那一行的改坏对象都在 crates/singlefs-core/src/ 下（不在的逐行列出）。
  登记表或变异表读不了算一处问题，不抛。
退出码：0 核过；1 有问题（逐处列）；2 用法错。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  CRATES_MUTATION_LIGHT_ROWS_BREAK=missing-targets-accepted  ④ 不核核心行的每个测试目标都有一行
      「✗ 自证 missing-target：」、「✗ 自证 missing-core-target：」
  CRATES_MUTATION_LIGHT_ROWS_BREAK=unknown-rows-accepted  ② 不核点名的变异名在变异表里
      「✗ 自证 unknown-row：」
  CRATES_MUTATION_LIGHT_ROWS_BREAK=duplicate-targets-accepted  ④ 一个测试目标登记两行不红
      「✗ 自证 two-rows-for-one-target：」
  CRATES_MUTATION_LIGHT_ROWS_BREAK=target-column-unchecked  ③ 不核写的测试目标与那一行的第五列对得上
      「✗ 自证 wrong-target-column：」
  CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-rows-accepted  ⑤ 不核登记的行改坏对象在核心代码下
      「✗ 自证 non-core-row：」
  CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-targets-required  ④ 把核心行之外的测试目标也算进「要登记」（旧判法：整表每个测试目标一行）
      「✗ 自证 clean：」、「✗ 自证 non-core-target-needs-no-row：」
"""
import importlib.util
import os
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("CRATES_MUTATION_LIGHT_ROWS_BREAK", "")
LIGHT_TABLE = os.path.join(".claude", "gate.d", "crates-mutation-light-rows.tsv")
MUTATION_TABLE = os.path.join("crates", "mutations.tsv")
CORE_PREFIX = "crates/singlefs-core/src/"
_ROWS_SPECIFICATION = importlib.util.spec_from_file_location("crates_mutation_rows_for_light_rows", os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))
mutation_rows = importlib.util.module_from_spec(_ROWS_SPECIFICATION)
_ROWS_SPECIFICATION.loader.exec_module(mutation_rows)


def broken(switch):
    return switch in BROKEN.split(",")


def test_target_of(arguments):
    """第五列（cargo test 参数）里 ` -- ` 之前那一段：哪个包的哪个测试目标（筛选词不算）。"""
    return arguments.split(" -- ", 1)[0].strip()


def read_light_rows(root):
    """→ ([(行号, 变异名, 测试目标)], 问题)。"""
    rows, found = [], []
    try:
        lines = open(os.path.join(root, LIGHT_TABLE), encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了轻型变异登记表 {LIGHT_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t#", 1)[0].split("\t")
        if len(fields) != 2 or not fields[0].strip() or not fields[1].strip():
            found.append(f"{LIGHT_TABLE}:{number} 不是两段（变异名<TAB>测试目标）：{line[:120]}")
            continue
        rows.append((number, fields[0], fields[1].strip()))
    return rows, found


def problems(root):
    """登记表的问题清单（文件头「核法」）。"""
    light_rows, found = read_light_rows(root)
    try:
        heavy_rows, _malformed, _memory, _timeout = mutation_rows.read_table(os.path.join(root, MUTATION_TABLE))
    except OSError as error:
        return found + [f"读不了 {MUTATION_TABLE}：{error}"]
    target_of_name, object_of_name, first_row_of_target = {}, {}, {}
    for row in heavy_rows:
        target = test_target_of(row.arguments)
        target_of_name.setdefault(row.name, target)
        object_of_name.setdefault(row.name, row.path)
        if row.path.startswith(CORE_PREFIX) or broken("non-core-targets-required"):
            first_row_of_target.setdefault(target, row)
    seen_names, rows_of_target = set(), {}
    for number, name, target in light_rows:
        where = f"{LIGHT_TABLE}:{number}「{name}」"
        if name in seen_names:
            found.append(f"{where} 登记了两次")
        seen_names.add(name)
        if name not in target_of_name:
            if not broken("unknown-rows-accepted"):
                found.append(f"{where} 不是 {MUTATION_TABLE} 里成形的一行的变异名（那一行被删、改名或写坏了）")
            continue
        actual = target_of_name[name]
        if actual != target and not broken("target-column-unchecked"):
            found.append(f"{where} 写的测试目标是「{target}」，{MUTATION_TABLE} 里那一行第五列是「{actual}」")
        if not object_of_name[name].startswith(CORE_PREFIX):
            if not broken("non-core-rows-accepted"):
                found.append(f"{where} 的改坏对象是 {object_of_name[name]}，不在 {CORE_PREFIX} 下（核心部分只取核心代码；换成同一个测试目标在核心代码下行号最小的那一条，核心代码下没有这个测试目标的就删这一行）")
            continue
        rows_of_target.setdefault(actual, []).append(name)
    if not broken("missing-targets-accepted"):
        for target, row in first_row_of_target.items():
            if target not in rows_of_target:
                found.append(f"{MUTATION_TABLE} 的测试目标「{target}」在轻型登记表里没有一行（照挑法补这一行：{row.name}\t{target}，"
                             f"它在 {MUTATION_TABLE} 第 {row.line_number} 行）")
    if not broken("duplicate-targets-accepted"):
        for target, names in rows_of_target.items():
            if len(names) > 1:
                found.append(f"测试目标「{target}」在轻型登记表里登记了 {len(names)} 行（{'、'.join(names)}）：一个测试目标恰好一行")
    return found


def light_names(root):
    rows, _ = read_light_rows(root)
    return [name for _, name, _ in rows]


def core_target_count(root):
    """改坏对象在核心代码下的成形的行里，不同的测试目标有几个（核过之后才报数）。"""
    try:
        heavy_rows = mutation_rows.read_table(os.path.join(root, MUTATION_TABLE))[0]
    except OSError:
        return 0
    return len({test_target_of(row.arguments) for row in heavy_rows if row.path.startswith(CORE_PREFIX)})


def report_problems(found):
    for problem in found:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 轻型变异登记表 {LIGHT_TABLE} 有 {len(found)} 处不对（逐处列在上面）")  # gate-lint:summary
    print(f"  → 怎么办：照那几处改登记表（写法与挑法在 {LIGHT_TABLE} 表头）；变异表改了名或删了行的，轻型那一行跟着换成同一个测试目标在核心代码下剩下行号最小的那一条")


def command_check(root, print_names):
    found = problems(root)
    if found:
        report_problems(found)
        return 1
    if print_names:
        for name in light_names(root):
            print(name)
        return 0
    count = len(light_names(root))
    targets = core_target_count(root)
    print(f"  ✓ 轻型变异登记表核过：{count} 行，改坏对象在 {CORE_PREFIX} 下的行里 {targets} 个测试目标每个恰好一行、登记的行改坏对象都在核心代码下、"
          f"点名的行都在变异表里、测试目标与那一行第五列对得上")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="crates-mutation-light-rows-selftest-")
    failures, cells = [], []
    try:
        heavy = ["# 样本变异表",
                 "甲一\tcrates/singlefs-core/src/lib.rs\ta\tb\t-p demo --lib -- first\tfirst",
                 "甲二\tcrates/singlefs-core/src/lib.rs\tc\td\t-p demo --lib -- second\tsecond",
                 "乙一\tcrates/singlefs-core/src/walk.rs\te\tf\t-p demo --test walk -- walk\twalk",
                 "丙一\tcrates/outside/src/lib.rs\tg\th\t-p other --lib -- third\tthird"]

        def world(name, light_lines):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, "crates"))
            open(os.path.join(root, MUTATION_TABLE), "w", encoding="utf-8").write("\n".join(heavy) + "\n")
            open(os.path.join(root, LIGHT_TABLE), "w", encoding="utf-8").write("# 样本登记表\n" + "".join(line + "\n" for line in light_lines))
            return root

        def expect(cell, root, wanted):
            cells.append(cell)
            found = problems(root)
            if wanted is None and found:
                failures.append((cell, f"应当核过，实际 {found}"))
            elif wanted is not None and not any(wanted in problem for problem in found):
                failures.append((cell, f"应当报一处含「{wanted}」的问题，实际 {found}"))

        good = ["甲一\t-p demo --lib\t# 样本", "乙一\t-p demo --test walk"]
        expect("clean", world("clean", good), None)
        expect("non-core-target-needs-no-row", world("non-core-target-needs-no-row", good), None)
        expect("missing-target", world("missing-target", good[:1]), "测试目标「-p demo --test walk」在轻型登记表里没有一行（照挑法补这一行：乙一")
        expect("missing-core-target", world("missing-core-target", [good[0], "丙一\t-p other --lib"]),
               "测试目标「-p demo --test walk」在轻型登记表里没有一行（照挑法补这一行：乙一")
        expect("non-core-row", world("non-core-row", good + ["丙一\t-p other --lib"]), "「丙一」 的改坏对象是 crates/outside/src/lib.rs，不在 crates/singlefs-core/src/ 下")
        expect("unknown-row", world("unknown-row", good + ["戊一\t-p demo --lib"]), "「戊一」 不是 crates/mutations.tsv 里成形的一行的变异名")
        expect("two-rows-for-one-target", world("two-rows-for-one-target", good + ["甲二\t-p demo --lib"]), "测试目标「-p demo --lib」在轻型登记表里登记了 2 行")
        expect("wrong-target-column", world("wrong-target-column", ["甲一\t-p demo --test walk", "乙一\t-p demo --test walk"]), "写的测试目标是「-p demo --test walk」")
        expect("not-two-fields", world("not-two-fields", good + ["只有名字"]), "不是两段")
        names_root = world("names", good)
        cells.append("names")
        if light_names(names_root) != ["甲一", "乙一"]:
            failures.append(("names", f"names 应当按登记表次序打「甲一、乙一」，实际 {light_names(names_root)}"))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 problems() 那一条核法；CRATES_MUTATION_LIGHT_ROWS_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ crates-mutation-light-rows 自证通过：干净的表核过（核心行之外的测试目标不要求登记）、缺核心行的测试目标给出该补的那一行（非核心的行顶不了）、"
          f"登记了改坏对象不在核心代码的行、点名的行不在变异表、一个测试目标两行、测试目标写错、一行不是两段各报出来，names 照登记次序打（查了 {len(cells)} 格）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments and arguments[0] in ("check", "names") and len(arguments) <= 2:
        root = os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT
        return command_check(root, arguments[0] == "names")
    print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
    print("  → 怎么办：crates-mutation-light-rows.py check [仓根] | names [仓根] | --selftest")
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
