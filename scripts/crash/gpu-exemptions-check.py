#!/usr/bin/env python3
# admission: always 判的是此刻 crates/ 的 Rust 源码与豁免表，几秒读完；两边有一边改了都要现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/gpu-exemptions.tsv，除了 python3 之外没有环境要求
"""不走 GPU 的计算登记成豁免：豁免表 .claude/gate.d/gpu-exemptions.tsv 与代码里的标记 `// gpu-exempt: <截名>` 逐项对上
（欠账 C626（走 GPU 的计算类没有登记），用户定不得欠账；判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J5：
选路只看本机开关与仓内豁免表，豁免表每一行登记时由用户点头）。门禁 code-source-discipline 的 gpu-exemptions 格调它。

豁免表写法在它自己的表头：一行「截名<TAB>代码标记名<TAB>为什么上不了 GPU<TAB>用户点头日期」，代码标记名就是写在代码里的那一整句
`// gpu-exempt: <截名>`；代码里那一截不走 GPU 的计算，在它的函数上一行写这一句。

用法：
    gpu-exemptions-check.py check [仓根]    只核，核过打一句成功句（几行、几处标记、扫了几份 .rs）；核不过逐处列、退 1
    gpu-exemptions-check.py --selftest      拿造出来的小仓核每一条核法；GPU_EXEMPTIONS_CHECK_BREAK=<项> 时点名的格必须红

核法（problems）：
  ① 豁免表每一行四段（再加 # 说明）；截名是小写字母、数字、-，不重复；代码标记名恰好是「// gpu-exempt: <截名>」；
     为什么上不了 GPU 至少 8 个字；用户点头日期写成 YYYY-MM-DD；
  ② crates/ 下每一份 .rs 里，去掉行首空白之后以「// gpu-exempt:」起头的每一行是一处标记：冒号后面的截名在豁免表里有一行（没有的逐处列出，
     截名写坏的也列出）；
  ③ 豁免表的每一行在代码里至少有一处标记（一处都没有的逐行列出：那一截要么已经上了 GPU、删掉这一行，要么代码里还没写标记）。
  豁免表读不了算一处问题；豁免表不在照「读不了」报（这张表要随仓走）。
管不到的：一截计算在不在 GPU 上跑、标记有没有写在真不走 GPU 的那一截上、为什么写得对不对，靠 review 与用户点头。
退出码：0 核过；1 有问题（逐处列）；2 用法错。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  GPU_EXEMPTIONS_CHECK_BREAK=unregistered-markers-accepted  ② 不核代码里的标记在豁免表里
      「✗ 自证 marker-without-row：」
  GPU_EXEMPTIONS_CHECK_BREAK=unmarked-rows-accepted  ③ 不核豁免表每一行在代码里有标记
      「✗ 自证 row-without-marker：」
  GPU_EXEMPTIONS_CHECK_BREAK=marker-column-unchecked  ① 不核代码标记名一栏是不是「// gpu-exempt: <截名>」
      「✗ 自证 wrong-marker-column：」
  GPU_EXEMPTIONS_CHECK_BREAK=row-fields-unchecked  ① 不核为什么的字数与用户点头日期的写法
      「✗ 自证 reason-too-short：」、「✗ 自证 date-malformed：」
"""
import os
import re
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("GPU_EXEMPTIONS_CHECK_BREAK", "")
EXEMPTION_TABLE = os.path.join(".claude", "gate.d", "gpu-exemptions.tsv")
MARKER_PREFIX = "// gpu-exempt:"
SECTION_NAME = re.compile(r"^[a-z0-9][a-z0-9-]*$")
NOD_DATE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")
MINIMUM_REASON_CHARACTERS = 8


def broken(switch):
    return switch in BROKEN.split(",")


def read_rows(root):
    """→ ({截名: 行号}, 问题)。"""
    rows, found = {}, []
    try:
        lines = open(os.path.join(root, EXEMPTION_TABLE), encoding="utf-8").read().splitlines()
    except OSError as error:
        return {}, [f"读不了豁免表 {EXEMPTION_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t#", 1)[0].split("\t")
        where = f"{EXEMPTION_TABLE}:{number}"
        if len(fields) != 4:
            found.append(f"{where} 是 {len(fields)} 段，要四段（截名、代码标记名、为什么上不了 GPU、用户点头日期，制表符分隔）")
            continue
        name, marker, reason, nod = (field.strip() for field in fields)
        if not SECTION_NAME.match(name):
            found.append(f"{where} 截名「{name}」只许小写字母、数字与 -")
            continue
        if name in rows:
            found.append(f"{where} 截名 {name} 登记了两次（第 {rows[name]} 行已经有）")
            continue
        rows[name] = number
        if marker != f"{MARKER_PREFIX} {name}" and not broken("marker-column-unchecked"):
            found.append(f"{where} {name} 的代码标记名一栏是「{marker}」，要恰好是「{MARKER_PREFIX} {name}」")
        if not broken("row-fields-unchecked"):
            if len(reason) < MINIMUM_REASON_CHARACTERS:
                found.append(f"{where} {name} 的「为什么上不了 GPU」只有 {len(reason)} 个字，至少 {MINIMUM_REASON_CHARACTERS} 个字")
            if not NOD_DATE.match(nod):
                found.append(f"{where} {name} 的用户点头日期「{nod}」要写成 YYYY-MM-DD（每一行登记时由用户点头）")
    return rows, found


def source_files(root):
    found = []
    for directory, subdirectories, files in os.walk(os.path.join(root, "crates")):
        subdirectories[:] = sorted(name for name in subdirectories if name != "target")
        found += [os.path.join(directory, name) for name in sorted(files) if name.endswith(".rs")]
    return found


def markers(root):
    """→ ([(文件:行, 截名)], 扫了几份 .rs)。"""
    found, paths = [], source_files(root)
    for path in paths:
        for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
            stripped = line.strip()
            if stripped.startswith(MARKER_PREFIX):
                found.append((f"{os.path.relpath(path, root)}:{number}", stripped[len(MARKER_PREFIX):].strip()))
    return found, len(paths)


def problems(root):
    rows, found = read_rows(root)
    seen, _scanned = markers(root)
    marked = set()
    for where, name in seen:
        if not SECTION_NAME.match(name):
            found.append(f"{where} 的标记「{MARKER_PREFIX} {name}」截名写坏了（只许小写字母、数字与 -）")
            continue
        marked.add(name)
        if name not in rows and not broken("unregistered-markers-accepted"):
            found.append(f"{where} 标了「{MARKER_PREFIX} {name}」，豁免表里没有 {name} 这一行（不走 GPU 的计算要登记、由用户点头）")
    if not broken("unmarked-rows-accepted"):
        for name, number in rows.items():
            if name not in marked:
                found.append(f"{EXEMPTION_TABLE}:{number} {name} 在 crates/ 里一处标记都没有（那一截不走 GPU 的计算的函数上一行写「{MARKER_PREFIX} {name}」；已经上了 GPU 就删掉这一行）")
    return found


def command_check(root):
    found = problems(root)
    if found:
        for problem in found:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 不走 GPU 的计算与豁免表 {EXEMPTION_TABLE} 有 {len(found)} 处对不上（逐处列在上面）")  # gate-lint:summary
        print(f"  → 怎么办：代码里不走 GPU 的那一截在函数上一行写「{MARKER_PREFIX} <截名>」、豁免表登记同名一行（为什么上不了 GPU、用户点头日期；"
              "每加一行先问用户）；标记写坏的照截名改；表里一处标记都没有的行，那一截上了 GPU 就删掉、没上就补标记")
        return 1
    rows, _ = read_rows(root)
    seen, scanned = markers(root)
    print(f"  ✓ 不走 GPU 的计算与豁免表逐项对上：豁免表 {len(rows)} 行、代码里 {len(seen)} 处标记（扫了 crates/ 下 {scanned} 份 .rs），每一处标记都有表行、每一行都有标记")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="gpu-exemptions-check-selftest-")
    failures, cells = [], []
    try:
        def world(name, table_lines, source):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, "crates", "demo", "src"))
            open(os.path.join(root, EXEMPTION_TABLE), "w", encoding="utf-8").write("# 样本豁免表\n" + "".join(line + "\n" for line in table_lines))
            open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w", encoding="utf-8").write(source)
            return root

        def expect(cell, root, wanted):
            cells.append(cell)
            found = problems(root)
            if wanted is None and found:
                failures.append((cell, f"应当核过，实际 {found}"))
            elif wanted is not None and not any(wanted in problem for problem in found):
                failures.append((cell, f"应当报一处含「{wanted}」的问题，实际 {found}"))

        row = "writable-mount\t// gpu-exempt: writable-mount\t要真跑 Rust 实现的可写挂载路径，被测对象就是它\t2026-10-10"
        marked = "    // gpu-exempt: writable-mount\n    fn mount_writable() {}\n"
        expect("clean", world("clean", [row], marked), None)
        expect("marker-without-row", world("marker-without-row", [row], marked + "    // gpu-exempt: continuation\n    fn continue_after_mount() {}\n"),
               "标了「// gpu-exempt: continuation」，豁免表里没有 continuation 这一行")
        expect("row-without-marker", world("row-without-marker", [row], "fn mount_writable() {}\n"), "writable-mount 在 crates/ 里一处标记都没有")
        expect("wrong-marker-column", world("wrong-marker-column", [row.replace("// gpu-exempt: writable-mount", "gpu-exempt writable-mount")], marked),
               "代码标记名一栏是「gpu-exempt writable-mount」")
        expect("reason-too-short", world("reason-too-short", [row.replace("要真跑 Rust 实现的可写挂载路径，被测对象就是它", "跑实现")], marked),
               "「为什么上不了 GPU」只有 3 个字")
        expect("date-malformed", world("date-malformed", [row.replace("2026-10-10", "十月十日")], marked), "用户点头日期「十月十日」要写成 YYYY-MM-DD")
        expect("malformed-marker", world("malformed-marker", [row], marked + "// gpu-exempt: 可写挂载\n"), "截名写坏了")
        expect("three-fields", world("three-fields", ["writable-mount\t// gpu-exempt: writable-mount\t要真跑 Rust 实现"], marked), "是 3 段，要四段")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 problems() 那一条核法；GPU_EXEMPTIONS_CHECK_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ gpu-exemptions-check 自证通过：对上的核过，标记没有表行、表行没有标记、代码标记名一栏写错、为什么太短、日期写法不对、标记的截名写坏、"
          f"一行不是四段各报出来（查了 {len(cells)} 格）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments and arguments[0] == "check" and len(arguments) <= 2:
        return command_check(os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT)
    print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
    print("  → 怎么办：gpu-exemptions-check.py check [仓根] | --selftest")
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
