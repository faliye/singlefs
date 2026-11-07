#!/usr/bin/env python3
# admission: always 判的是此刻树上的 Rust 源码，改一个名字、加一个模块就要现判；整仓十几秒跑完
# run-condition: none 只读仓里的 Cargo.toml 与 .rs（经 lib_function_units 切单元），不编译、不起别的进程
# gate-similar: duplicate-definitions.py 上游那一份判同一个文件、同一个作用域里同名的定义出现两次（后一个盖掉前一个）；这里判同一个编译目标里不同模块各自定义的同名条目，射程不重叠，上游副本这里不写
# gate-similar: lib_function_units.py 单元与按裸名找得到的索引（RustWorkspace.by_name、module_of）照用它那一份，不另切一遍；它判的是闭包与键，不判撞名
# gate-similar: code-source-discipline.sh 它的 vague-names、ordinal-names 判名字本身写得好不好，不比两个名字撞没撞
"""同一个 Rust 编译目标里，不同模块各自定义的同名条目判红（撞名必须修复，不得撞名）。

为什么：单元图的 R-裸名规则在同一个编译目标里只按名字连，不看 pub、不看模块（crates/singlefs-unit-graph/src/closure.rs 的
follow_bare_names；python 那一份 scripts/gate/lib_function_units.py 的 RustWorkspace.index 建 by_name）。两个模块各有一个同名条目，
点到其中一个的单元就连到两个，判定的键收进不相干的单元：判器键收进流水线的私有 fn step_kind_code 与 JudgingUnitsFingerprint，就是这样来的。

用法：
    rust-name-collisions.py [根]           # 不给就判这份脚本所在的仓
    rust-name-collisions.py --selftest     # 自证，自带开关轮；RUST_NAME_COLLISIONS_BREAK=<开关> 时必须红在点名的格

判法（单元与模块树经 lib_function_units.RustWorkspace 取：工作区成员与顺 path 依赖收进来的 crate，每个 crate 的 lib、bin、test、bench、
example、构建脚本各是一个编译目标，目标的文件经 mod 声明（含 #[path]）连进来）：
  ① 逐个编译目标，收按裸名找得到的条目里这八种：fn、struct、enum、union、const、static、type、trait，私有的也收；
     只在测试里编进来的（#[cfg(test)] 模块里的、带 #[test] 的、tests/ 下集成测试目标里的）照收。
  ② 条目所在的模块 = 文件在目标模块树里的路径接文件里的行内模块路径（RustWorkspace.module_of）。
  ③ 同一个名字落在两个以上的模块里，判一处撞名；逐处列出种类、模块路径与文件:行。
  同一个模块里同名的几份（#[cfg] 分开的两种写法）不算：那是同一个作用域，归上游 duplicate-definitions。
  几个目标里撞的是同一组定义（同一份文件经 mod 连进好几个目标）的只列一次，后面写撞在哪几个目标里。
不算撞名的（依据）：
  impl 块与 trait 里的成员（trait 规定的方法名 fmt、eq，固有方法，关联常量与关联类型）：它们不进按裸名的索引，挂在 members_by_name 下，
    由 R-方法 经闭包里已有的类型连（workspace.rs 建索引那一段 `if let Some(container) = unit.container` 那一支；lib_function_units.py 的
    RustWorkspace.index 里 `unit.container is None` 才进 by_name）；trait 规定的名字也改不了。
  不同编译目标里的同名条目（lib 里一个、src/bin 下一个）：R-裸名在同一个目标里按名字连，跨目标只连 pub。用户点名的是「同一个 Rust 包（一个编译目标）里」，
    跨目标的另算，这里不判。
  条目级宏调用生成的名字、macro_rules!、use 别名：不在用户点名的那几种里。
  不是 Cargo 包的 Rust 源（scripts/experiment-apparatus/ 下单独编的 .rs）：不在任何编译目标里。

输出：每处撞名一行「✗ 撞名 <名字>：…」，下面逐行列定义；产品代码的排在前面，只在测试里编进来的排在后面并标「只在测试里」。
退出码：0 没有撞名；1 有撞名（逐处列出）或读不出源码；2 用法错；77 根下没有 Rust 编译目标。

弄坏开关（只给自证用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认；自证自带开关轮，逐个设上起 --selftest-inner，
红的格逐条以「✗ 自证 <格名>：」起头，开关轮核每条的头一格在红格里；单跑一条：RUST_NAME_COLLISIONS_BREAK=<开关> python3 scripts/gate/rust-name-collisions.py --selftest-inner）：
  RUST_NAME_COLLISIONS_BREAK=private-skipped  私有的条目不收（只判 pub）
      「✗ 自证 private-items：」、「✗ 自证 test-module-helpers：」、「✗ 自证 inline-modules：」、「✗ 自证 red-only-the-three：」、「✗ 自证 every-definition-listed：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=test-code-skipped  只在测试里编进来的条目不收
      「✗ 自证 test-module-helpers：」、「✗ 自证 red-only-the-three：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=inline-modules-merged  模块只取文件那一层，不接文件里的行内模块路径
      「✗ 自证 inline-modules：」、「✗ 自证 test-module-helpers：」、「✗ 自证 red-only-the-three：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=impl-members-counted  impl 与 trait 里的成员也收
      「✗ 自证 trait-and-inherent-methods：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=same-module-counted  同一个模块里同名的几份也算撞名
      「✗ 自证 cfg-alternatives-same-module：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=targets-merged  一个 crate 的几个编译目标并成一个判
      「✗ 自证 different-targets：」、「✗ 自证 test-module-helpers：」、「✗ 自证 inline-modules：」、「✗ 自证 exit-codes：」
  RUST_NAME_COLLISIONS_BREAK=one-definition-listed  每处撞名只列第一份定义
      「✗ 自证 every-definition-listed：」
  RUST_NAME_COLLISIONS_BREAK=collisions-not-red  有撞名照退 0
      「✗ 自证 exit-codes：」
"""
import collections
import contextlib
import io
import os
import re
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

JUDGED_KINDS = frozenset(("fn", "struct", "enum", "union", "const", "static", "type", "trait"))
UNNAMED_ITEM_NAME = "_"
BREAK_VARIABLE = "RUST_NAME_COLLISIONS_BREAK"
BREAKS = ("private-skipped", "test-code-skipped", "inline-modules-merged", "impl-members-counted", "same-module-counted",
          "targets-merged", "one-definition-listed", "collisions-not-red")
broken = set()


class Definition(collections.namedtuple("Definition", "kind module file line")):
    def module_text(self):
        return "::".join(self.module) or "<crate 根>"


def judged_units_of_target(workspace, target_id):
    """目标里按裸名找得到、种类在 JUDGED_KINDS 里的条目 → [(名字, 模块路径, 单元)]。"""
    target = workspace.targets[target_id]
    found = []
    for path in sorted(target.files):
        for unit in workspace.files[path].units:
            if unit.kind not in JUDGED_KINDS or unit.short == UNNAMED_ITEM_NAME:
                continue
            if unit.container is not None and "impl-members-counted" not in broken:
                continue
            if "private-skipped" in broken and not unit.public:
                continue
            if "test-code-skipped" in broken and function_units.rust_unit_is_test(workspace, unit, target_id):
                continue
            if "inline-modules-merged" in broken:
                module = tuple(target.files[unit.file][0])
            else:
                module = workspace.module_of(unit, target_id)
            found.append((unit.short, module, unit))
    return found


def judging_pools(workspace):
    """一个编译目标一池 → [(池名, [(目标 id, 名字, 模块路径, 单元)])]；弄坏开关 targets-merged 下一个 crate 一池（模块路径前面接目标 id，分得开）。"""
    pools = collections.OrderedDict()
    for target_id in sorted(workspace.targets):
        target = workspace.targets[target_id]
        if target.kind == "doctest":
            continue
        pool_name = target.crate.label if "targets-merged" in broken else target_id
        for name, module, unit in judged_units_of_target(workspace, target_id):
            if "targets-merged" in broken:
                module = (target_id,) + module
            pools.setdefault(pool_name, []).append((target_id, name, module, unit))
    return list(pools.items())


def collision_groups(workspace):
    """→ (判了几个目标, 判了几个条目, [(名字, (Definition, …), (目标 id, …), 只在测试里)])，撞同一组定义的几个目标并成一处。"""
    groups = collections.OrderedDict()
    test_flags = collections.defaultdict(list)
    targets_judged = sum(1 for target in workspace.targets.values() if target.kind != "doctest")
    units_judged = 0
    for _pool_name, entries in judging_pools(workspace):
        units_judged += len(entries)
        by_name = collections.defaultdict(lambda: collections.defaultdict(list))
        for target_id, name, module, unit in entries:
            by_name[name][module].append((target_id, unit))
        for name in sorted(by_name):
            modules = by_name[name]
            definitions_count = sum(len(placed) for placed in modules.values())
            if len(modules) < 2 and not ("same-module-counted" in broken and definitions_count > 1):
                continue
            definitions = tuple(sorted(Definition(unit.kind, module, unit.file, unit.first_line)
                                       for module, placed in modules.items() for _target_id, unit in placed))
            key = (name, definitions)
            groups.setdefault(key, set()).update(target_id for placed in modules.values() for target_id, _unit in placed)
            test_flags[key].extend(function_units.rust_unit_is_test(workspace, unit, target_id)
                                   for placed in modules.values() for target_id, unit in placed)
    collisions = [(name, definitions, tuple(sorted(groups[(name, definitions)])), all(test_flags[(name, definitions)]))
                  for name, definitions in groups]
    collisions.sort(key=lambda entry: (entry[3], entry[1][0].file, entry[1][0].line, entry[0]))
    return targets_judged, units_judged, collisions


def rendered_collision(name, definitions, target_ids, test_only):
    """一处撞名打成几行：头一行点名名字、模块数、撞在哪几个目标，下面每份定义一行。"""
    modules = len({definition.module for definition in definitions})
    shown_targets = "、".join(target_ids[:3]) + (f" 等 {len(target_ids)} 个" if len(target_ids) > 3 else "")
    lines = [f"  ✗ 撞名 {name}：{len(definitions)} 份定义在 {modules} 个模块里（编译目标 {shown_targets}）{'，只在测试里' if test_only else ''}"]
    listed = definitions[:1] if "one-definition-listed" in broken else definitions
    for definition in listed:
        lines.append(f"       {definition.kind} {definition.module_text()}  {definition.file}:{definition.line}")
    return lines


def judge(root):
    try:
        workspace = function_units.RustWorkspace(root)
    except (function_units.FunctionUnitsError, function_units.UsageError, OSError) as error:
        print(f"  ✗ 读不出 {root} 的 Rust 源码：{error}")
        print("     → 怎么办：照上面那一句修 Cargo.toml 或源文件（切单元的是 scripts/gate/lib_function_units.py 的 RustWorkspace）；修好再判，这一次一处都没判")
        return 1
    if not any(target.kind != "doctest" for target in workspace.targets.values()):
        print(f"  ! 本次无对象可判：{root} 下没有 Rust 编译目标（没有工作区成员 crate）")
        return 77
    targets_judged, units_judged, collisions = collision_groups(workspace)
    crates = len({target.crate.directory for target in workspace.targets.values() if target.kind != "doctest"})
    if not collisions:
        print(f"  ✓ 判了 {crates} 个 crate 的 {targets_judged} 个编译目标、{units_judged} 个按裸名找得到的条目，没有撞名")
        return 0
    for name, definitions, target_ids, test_only in collisions:
        for line in rendered_collision(name, definitions, target_ids, test_only):
            print(line)  # gate-lint:detail
    test_only_count = sum(1 for collision in collisions if collision[3])
    print(f"  ✗ 判了 {crates} 个 crate 的 {targets_judged} 个编译目标、{units_judged} 个按裸名找得到的条目：{len(collisions)} 处撞名"
          f"（产品代码 {len(collisions) - test_only_count} 处，只在测试里 {test_only_count} 处，逐处列在上面）")
    print("     → 怎么办：给撞名的几份各起一个说出它是什么、做什么的名字（只改一份也行，改到这个目标里只剩一份叫这个名字），调用处一起改；"
          "名字照 .claude/singlefs-ai-sop/rules/code-discipline.md「名字」；不登豁免表（用户 2026-11-06 定不得撞名）；判法与依据在 scripts/gate/rust-name-collisions.py 文件头")
    return 0 if "collisions-not-red" in broken else 1


# ── 自证 ──

SELFTEST_ROOT_MANIFEST = '[workspace]\nmembers = ["crates/demo"]\n'
SELFTEST_CRATE_MANIFEST = '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\nfast = []\n'
# 红树：私有 fn 撞名、测试模块里的辅助撞名、同一份文件里两个行内模块撞名
SELFTEST_RED_FILES = {
    "crates/demo/src/lib.rs": "mod ledger;\nmod journal;\nmod inventory;\n"
                              "pub mod inline_pair {\n    pub mod reading {\n        pub fn shared_inline() {}\n    }\n"
                              "    pub mod writing {\n        fn shared_inline() {}\n    }\n}\n",
    "crates/demo/src/ledger.rs": "fn private_twin() {}\n",
    "crates/demo/src/journal.rs": "fn private_twin() {}\n\n#[cfg(test)]\nmod tests {\n    fn helper_twin() {}\n}\n",
    "crates/demo/src/inventory.rs": "#[cfg(test)]\nmod tests {\n    fn helper_twin() {}\n}\n",
}
# 绿树：两个 impl 里同名的固有方法与 trait 规定的方法、同一个模块里 cfg 分开的两份、lib 与 src/bin 下各一份同名私有 fn
SELFTEST_GREEN_FILES = {
    "crates/demo/src/lib.rs": "mod gauges;\nmod meters;\n\nfn worker_count() -> usize {\n    1\n}\n",
    "crates/demo/src/gauges.rs": "pub struct Gauge;\n\nimpl Gauge {\n    fn reading(&self) -> u32 {\n        1\n    }\n}\n\n"
                                "impl std::fmt::Display for Gauge {\n"
                                "    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        formatter.write_str(\"gauge\")\n    }\n}\n\n"
                                "#[cfg(feature = \"fast\")]\nfn chosen_path() -> u32 {\n    1\n}\n\n#[cfg(not(feature = \"fast\"))]\nfn chosen_path() -> u32 {\n    2\n}\n",
    "crates/demo/src/meters.rs": "pub struct Meter;\n\nimpl Meter {\n    fn reading(&self) -> u32 {\n        2\n    }\n}\n\n"
                               "impl std::fmt::Display for Meter {\n"
                               "    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        formatter.write_str(\"meter\")\n    }\n}\n",
    "crates/demo/src/bin/tool.rs": "fn worker_count() -> usize {\n    2\n}\n\nfn main() {\n    let _ = worker_count();\n}\n",
}


def write_selftest_tree(directory, files):
    for path, text in {"Cargo.toml": SELFTEST_ROOT_MANIFEST, "crates/demo/Cargo.toml": SELFTEST_CRATE_MANIFEST, **files}.items():
        full = os.path.join(directory, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)


def collisions_named(root):
    _targets, _units, collisions = collision_groups(function_units.RustWorkspace(root))
    return {name: (definitions, test_only) for name, definitions, _target_ids, test_only in collisions}


def judged_quietly(root):
    """judge(root) 的退出码与它打的行（不往自证的输出里漏）。"""
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        status = judge(root)
    return status, captured.getvalue()


def run_selftest():
    os.environ[function_units.RUST_LEX_CACHE_DIRECTORY_VARIABLE] = "off"   # 自证不写跨轮的词法缓存
    cells = []
    with tempfile.TemporaryDirectory() as work:
        red_root, green_root, empty_root = (os.path.join(work, name) for name in ("red", "green", "empty"))
        write_selftest_tree(red_root, SELFTEST_RED_FILES)
        write_selftest_tree(green_root, SELFTEST_GREEN_FILES)
        os.makedirs(empty_root)
        red = collisions_named(red_root)
        green = collisions_named(green_root)
        twin = red.get("private_twin")
        cells.append(("private-items", twin is not None and not twin[1] and {definition.file for definition in twin[0]}
                      == {"crates/demo/src/ledger.rs", "crates/demo/src/journal.rs"},
                      f"ledger 与 journal 里各一个私有 fn private_twin 应当报一处产品代码撞名、两份定义各在一份文件，实报 {twin}"))
        helper = red.get("helper_twin")
        cells.append(("test-module-helpers", helper is not None and helper[1] and {definition.module for definition in helper[0]}
                      == {("journal", "tests"), ("inventory", "tests")},
                      f"journal::tests 与 inventory::tests 里各一个 helper_twin 应当报一处只在测试里的撞名，实报 {helper}"))
        inline = red.get("shared_inline")
        cells.append(("inline-modules", inline is not None and {definition.module for definition in inline[0]}
                      == {("inline_pair", "reading"), ("inline_pair", "writing")},
                      f"lib.rs 里 inline_pair::reading 与 inline_pair::writing 各一个 shared_inline 应当报一处撞名，实报 {inline}"))
        cells.append(("red-only-the-three", set(red) == {"private_twin", "helper_twin", "shared_inline"},
                      f"红树只该报 private_twin、helper_twin、shared_inline 三处，实报 {sorted(red)}"))
        cells.append(("trait-and-inherent-methods", "reading" not in green and "fmt" not in green,
                      f"两个 impl 里同名的固有方法 reading 与 trait 规定的 fmt 不算撞名，实报 {sorted(green)}"))
        cells.append(("cfg-alternatives-same-module", "chosen_path" not in green,
                      f"同一个模块里 cfg 分开的两份 chosen_path 不算撞名，实报 {sorted(green)}"))
        cells.append(("different-targets", "worker_count" not in green,
                      f"lib 里与 src/bin/tool.rs 里各一个 worker_count 在不同的编译目标里，不算撞名，实报 {sorted(green)}"))
        rendered = "\n".join(rendered_collision("private_twin", twin[0], ("demo:lib:demo",), False)) if twin else ""
        cells.append(("every-definition-listed", "crates/demo/src/ledger.rs:1" in rendered and "crates/demo/src/journal.rs:1" in rendered,
                      f"一处撞名要逐份列出文件:行（ledger.rs:1 与 journal.rs:1 都在），实打 {rendered!r}"))
        statuses = {name: judged_quietly(root) for name, root in (("red", red_root), ("green", green_root), ("empty", empty_root))}
        cells.append(("exit-codes", statuses["red"][0] == 1 and "3 处撞名" in statuses["red"][1] and statuses["green"][0] == 0
                      and "没有撞名" in statuses["green"][1] and statuses["empty"][0] == 77,
                      f"红树退 1 且报「3 处撞名」、绿树退 0 且报「没有撞名」、空目录退 77，实退 红 {statuses['red'][0]}、绿 {statuses['green'][0]}、"
                      f"空 {statuses['empty'][0]}"))
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（私有撞名、测试模块里的辅助、行内模块、只报这三处、impl 成员不算、同一模块 cfg 两份不算、"
          "不同编译目标不算、逐份列文件:行、退出码）")
    return 0


def switch_wheel():
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格。"""
    expected = {"private-skipped": "private-items", "test-code-skipped": "test-module-helpers", "inline-modules-merged": "inline-modules",
                "impl-members-counted": "trait-and-inherent-methods", "same-module-counted": "cfg-alternatives-same-module",
                "targets-merged": "different-targets", "one-definition-listed": "every-definition-listed", "collisions-not-red": "exit-codes"}
    failures = 0
    for switch, cell in expected.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True,
                                   env=dict(os.environ, **{BREAK_VARIABLE: switch}))
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        if completed.returncode == 1 and cell in red_cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，含点名的 {cell}")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的是 {cell}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(expected)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(expected)} 个开关逐个打开，各红在点名的格")
    return 0


def main(arguments):
    if arguments[:1] in (["--selftest"], ["--selftest-inner"]):
        if arguments[0] == "--selftest-inner":
            broken.update(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item in BREAKS)
            return run_selftest()
        status = run_selftest()
        return status or switch_wheel()
    if len(arguments) > 1 or any(argument.startswith("-") for argument in arguments):
        print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
        print("     → 怎么办：用法 rust-name-collisions.py [根] 或 --selftest")
        return 2
    root = os.path.realpath(arguments[0]) if arguments else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 根不在：{root}")
        print("     → 怎么办：给仓根，或不给参数判这份脚本所在的仓")
        return 2
    return judge(root)


BREAK_VARIABLES = (BREAK_VARIABLE,)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头（--selftest-inner 同）；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] in (["--selftest"], ["--selftest-inner"]) or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 python3 scripts/gate/rust-name-collisions.py --selftest（开关轮逐个打开）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
