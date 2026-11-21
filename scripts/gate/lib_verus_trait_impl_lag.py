#!/usr/bin/env python3
# admission: always 读的是调用方这一次给的根、此刻登记的行与基准里的表，不存结论
# run-condition: command git
"""verus! 外面的 trait impl 登记式放过表（.claude/gate.d/verus-trait-impls-outside-lag.tsv）与 gate 基准那一版比「只缩不涨」，连同这一比的自证格。

调它的：scripts/gate/verus-proof-check.py（门禁 formal-verification 的 verus-proof 格）。
  region_problems 调 trait_impl_lag_growth（放过表只缩不涨）与 in_test_only_module_file（cfg(test) 的 mod 声明引进来的整份文件里的 impl 不判）；自证调 selftest_cells，自证的开关轮读 BREAK_SWITCHES。
比法：按（trait 全路径、实现类型）分组，组内每一行算 1，所在文件照拆分族认（scripts/gate/lib_lag_split_family.py：认族与比之和只在那一份）。
  基准里点名一份入口的一行，族是它本身加按它的拆法命名的部分文件（.rs 入口加 <名>/ 下的 .rs，入口是 main.rs、mod.rs 的加同目录与下层的 .rs，.py、.sh 入口的三种拆法写在那一份的文件头）；
  族内此刻的行数之和超过基准那一行、或文件落到族外（基准里没有这一组、或这一组没有族含它的行）的判红。
  被判的根不在 git 仓里、基准里还没有这张表（初次登记）不比；git 取不到基准那一版判红。
弄坏开关：BREAK_SWITCHES 里十条，名字与弄坏哪一处写在 verus-proof-check.py 文件头开关表里（VERUS_PROOF_CHECK_BREAK）；
  这一份不读环境变量，调用方把已打开的开关集合交进来（break_switches）。
这一份没有单独的自证：判绿、判红与每条开关的红格都由 verus-proof-check.py 的 --selftest 跑 selftest_cells 证；
  与 scripts/gate/lib_lag_split_family.py 同一种做法。
"""
import os
import subprocess

import lib_function_units as function_units
import lib_lag_split_family as split_family

# 拆分族那几条开关在共用写法（scripts/gate/lib_lag_split_family.py）里的名字
FAMILY_DEFECTS = {"trait-impl-lag-family-unrecognized": split_family.UNRECOGNIZED, "trait-impl-lag-family-main-excluded": split_family.MAIN_EXCLUDED,
                  "trait-impl-lag-family-wide": split_family.WIDE, "trait-impl-lag-family-sum-unchecked": split_family.SUM_UNCHECKED,
                  "trait-impl-lag-family-python-split-unrecognized": split_family.PYTHON_SPLIT_UNRECOGNIZED,
                  "trait-impl-lag-family-hook-split-unrecognized": split_family.HOOK_SPLIT_UNRECOGNIZED,
                  "trait-impl-lag-family-shell-split-unrecognized": split_family.SHELL_SPLIT_UNRECOGNIZED,
                  "trait-impl-lag-family-directory-entry-unrecognized": split_family.DIRECTORY_ENTRY_UNRECOGNIZED}
# 自证开关轮要逐条打开的开关（verus-proof-check.py 的 items 展开它）
BREAK_SWITCHES = ("trait-impl-lag-growth-unchecked", "test-module-file-impl-judged", *FAMILY_DEFECTS)


def in_test_only_module_file(workspace, relative, unit, break_switches):
    """被验 crate 的 src 里 relative 这份文件里的 unit（一处 impl）只在按测试编译时才有：编这份文件的每个目标里（文档测试目标不算），
    单元自己与外层、把这份文件连进来的 mod 声明（目标里这份文件的链条件）带 cfg(test)；判的是 lib_function_units.py 的 rust_unit_is_test_code，
    文件开头的 #![cfg(test)] 已在单元自己的条件里。没有哪个目标把这份文件连进来的（没被 mod 声明引到）交 False。弄坏开关 test-module-file-impl-judged 下一律交 False。"""
    if "test-module-file-impl-judged" in break_switches:
        return False
    owners = [identifier for identifier, target in workspace.targets.items() if target.kind != "doctest" and relative in target.files]
    return bool(owners) and all(function_units.rust_unit_is_test_code(workspace, unit, identifier) for identifier in owners)


def trait_impl_lag_growth(root, rows, library, table, break_switches):
    """放过表只缩不涨：rows（这个 crate 的行）与 gate 基准那一版的表比（library 是 lib-lag-baseline.py 的模块，用它的 gate_base_of、baseline_rows；table 是表的仓内路径），
    交 (问题 [(卡在哪, 下一步)], 给成功句的一句)。按（trait 全路径、实现类型）分组，组内每一行算 1，所在文件照拆分族认（lib_lag_split_family.py）：
    基准里点名一份入口的一行，族是它本身加按它的拆法命名的部分文件（认法写在 lib_lag_split_family.py 文件头）；
    族内此刻的行数之和超过基准那一行、或文件落到族外（基准里没有这一组、或这一组没有族含它的行）的判红。
    被判的根不在 git 仓里、基准里还没有这张表（初次登记）不比、成功句照说；git 取不到基准那一版判红。
    弄坏开关 trait-impl-lag-growth-unchecked 下不比；trait-impl-lag-family-unrecognized、-main-excluded、-wide、-sum-unchecked、-python-split-unrecognized、
    -hook-split-unrecognized、-shell-split-unrecognized、-directory-entry-unrecognized 下拆分族那一半各弄坏一处（文件头逐项写）。"""
    if "trait-impl-lag-growth-unchecked" in break_switches:
        return [], "没与 gate 基准比"
    base, problem = library.gate_base_of(root)
    if problem is not None:
        return [(f"取不到 gate 基准，{table} 只缩不涨比不了：{problem}",
                 "照这一句修好 git（scripts/gate/changed-paths.sh 的 gate_diff_base gate 在被判的根上跑通）再跑这一格；不当成没有基准放过")], ""
    if base is None:
        return [], "被判的根不在 git 仓里，没与 gate 基准比"
    base_rows, problem = library.baseline_rows(base, table, cwd=root)
    if problem is not None:
        return [(f"{table} 只缩不涨比不了：{problem.sentence()}", "修好 git 对象库（git fsck）或照 GATE_BASE 给一个取得到的基准，再跑这一格")], ""
    if base_rows is None:
        return [], f"gate 基准 {base[:12]} 里还没有这张表（初次登记），没比"
    base_groups, current_groups = {}, {}
    for _number, fields in base_rows:
        if len(fields) >= 3:
            base_groups.setdefault((fields[0].strip(), fields[1].strip()), {})[fields[2].strip()] = 1
    for row in rows:
        counts = current_groups.setdefault((row["trait"], row["type"]), {})
        counts[row["file"]] = counts.get(row["file"], 0) + 1
    growth = {(group, item.path): item for group, item in split_family.grouped_family_growth(
        base_groups, current_groups, frozenset(defect for name, defect in FAMILY_DEFECTS.items() if name in break_switches))}
    problems = []
    for row in rows:
        item = growth.get(((row["trait"], row["type"]), row["file"]))
        if item is None:
            continue
        where = f"{table} 第 {row['number']} 行 {row['trait']} for {row['type']}（{row['file']}）"
        if item.owner is None:
            problems.append((f"{where}在 gate 基准 {base[:12]} 那一版的表里没有：这张表只缩不涨",
                             "删掉这一行，把那个 impl 挪进 verus! 里（方法照 trait 的 requires / ensures 证出来）；登记式放过只给 gate 基准里已有的行"))
        else:
            problems.append((f"{where}所在的拆分族 {item.owner} 在 gate 基准 {base[:12]} 那一版的表里是 {item.allowed} 行、此刻族内（它本身与同名目录下的 .rs）登记了 {item.total} 行：这张表只缩不涨",
                             "一个 impl 拆到几份文件也不许比基准多登记：把挪进 verus! 里的 impl 的行删掉，别加行"))
    return problems, f"与 gate 基准 {base[:12]} 那一版比过、没涨"


def selftest_cells(expect, make_root, lag_root, selftest_kernel_with, table, lag_kernel, trait_declaration, impl_outside, second_impl_outside, lag_row,
                   expect_true, lag_library, break_switches):
    """只缩不涨的自证格：基准比过判绿、涨了判红（baseline-green、grown-red），拆分族判绿判红（family-moved-green、-main-green、-sum-red、-outside-red），
    入口是 .py、.sh 时的三种拆法判绿（family-python-split-green、-hook-split-green、-shell-split-green）与同目录名字对不上拆法的脚本判红（family-script-outside-red），
    入口是 main.rs、mod.rs 时同目录与下层的 .rs 判绿（family-directory-entry-green）。
    expect(标签, 检查, 根, 期望退出码, 期望文字) 是 verus-proof-check.py 自证里数格的那一个；make_root 造样本根；lag_root(名字, kernel, 行) 造登记了放过表的根；
    table 是放过表的仓内路径；lag_kernel 是带 trait 声明与 BadDevice 外面 impl 的核；trait_declaration、impl_outside、second_impl_outside、lag_row 是自证用的样本文本；
    expect_true(标签, 是否对, 说明) 数一格已经算出来的判定；lag_library 是 lib-lag-baseline.py 的模块；break_switches 是已打开的开关集合。
    verus! 外面的 impl 只在 .rs 里，造不出别的后缀的核：.py、.sh 入口的几格不起整格检查，直接调 trait_impl_lag_growth，行里的文件是哪个后缀照写；
    main.rs、mod.rs 入口那一格同样直接调（入口与部分文件的路径是它的判据，不需要整格去编译）。
    被判的根是 git 仓、gate 基准（没有上游取 HEAD）里的表只有 BadDevice 那一行，照旧判绿、成功句写比过；再登记一个 WorseDevice 判红（开关 trait-impl-lag-growth-unchecked 下两格都判错）。"""
    git_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}

    def commit_base(root):
        for arguments in (["init", "-q"], ["add", "-A"],
                          ["-c", "user.name=selftest", "-c", "user.email=selftest@invalid", "-c", "commit.gpgsign=false", "commit", "-q", "-m", "自证用的基准"]):
            subprocess.run(["git", *arguments], cwd=root, env=git_environment, check=True, capture_output=True)

    grown_root = lag_root("traitlaggrown", lag_kernel, [lag_row])
    commit_base(grown_root)
    saved_git = {name: os.environ.pop(name) for name in list(os.environ) if name.startswith("GIT_") or name == "GATE_BASE"}
    try:
        expect("trait-impl-lag-baseline-green", "proof", grown_root, 0, ("放过表：与 gate 基准 ", " 那一版比过、没涨"))
        with open(os.path.join(grown_root, "crates", "core", "src", "kernel.rs"), "a") as handle:
            handle.write(second_impl_outside)
        with open(os.path.join(grown_root, table), "a") as handle:
            handle.write("core_sample::kernel::Device\tWorseDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：基准里没有这一行\n")
        expect("trait-impl-lag-grown-red", "proof", grown_root, 1,
               ("第 3 行 core_sample::kernel::Device for WorseDevice（crates/core/src/kernel.rs）在 gate 基准 ", "这张表只缩不涨"))

        # 拆分族（lib_lag_split_family.py）：基准里点名 <名>.rs 的一行，族是它本身加 <名>/ 下的 .rs。基准里 base_file 里有 BadDevice 的 impl 与它的一行，
        # 此刻 base_file 删了、later_files 里各有一份 BadDevice 的 impl 与各自一行。族内同（trait、类型）的行数之和不超过基准那一行判绿
        # （开关 trait-impl-lag-family-unrecognized、-main-excluded 下判错）、超了判红（-sum-unchecked 下判错）、落到族外判红（-wide 下判错）
        def family_case(label, base_file, later_files, want_exit, want_text):
            root = make_root(label, kernel=selftest_kernel_with(trait_declaration))

            def put_files(files):
                for file in files:
                    os.makedirs(os.path.dirname(os.path.join(root, file)), exist_ok=True)
                    open(os.path.join(root, file), "w").write(impl_outside)
                open(os.path.join(root, table), "w").write(
                    "# 自证用\n" + "".join(lag_row.replace("crates/core/src/kernel.rs", file) + "\n" for file in files))

            put_files([base_file])
            commit_base(root)
            os.remove(os.path.join(root, base_file))
            put_files(later_files)
            expect(label, "proof", root, want_exit, want_text)

        family_case("trait-impl-lag-family-moved-green", "crates/core/src/device.rs", ["crates/core/src/device/bad_device.rs"], 0,
                    ("放过表：与 gate 基准 ", " 那一版比过、没涨"))
        family_case("trait-impl-lag-family-main-green", "crates/core/src/device_tool.rs", ["crates/core/src/device_tool/main.rs"], 0,
                    ("放过表：与 gate 基准 ", " 那一版比过、没涨"))
        family_case("trait-impl-lag-family-sum-red", "crates/core/src/device.rs", ["crates/core/src/device/bad_device.rs", "crates/core/src/device/bad_device_again.rs"], 1,
                    ("第 2 行 core_sample::kernel::Device for BadDevice（crates/core/src/device/bad_device.rs）所在的拆分族 crates/core/src/device.rs 在 gate 基准 ",
                     "那一版的表里是 1 行、此刻族内（它本身与同名目录下的 .rs）登记了 2 行：这张表只缩不涨",
                     "第 3 行 core_sample::kernel::Device for BadDevice（crates/core/src/device/bad_device_again.rs）所在的拆分族"))
        family_case("trait-impl-lag-family-outside-red", "crates/core/src/device.rs", ["crates/core/src/other.rs"], 1,
                    ("第 2 行 core_sample::kernel::Device for BadDevice（crates/core/src/other.rs）在 gate 基准 ", "那一版的表里没有：这张表只缩不涨"))

        # .py、.sh 入口的拆法（lib_lag_split_family.py 文件头逐种写）：基准里 base_file 那一行（BadDevice），此刻 later_files 各一行，直接调 trait_impl_lag_growth，交 (问题, 成功句)
        def script_family_judged(label, base_file, later_files):
            root = make_root(label, kernel=selftest_kernel_with(trait_declaration))
            with open(os.path.join(root, table), "w") as handle:
                handle.write("# 自证用\n" + lag_row.replace("crates/core/src/kernel.rs", base_file) + "\n")
            commit_base(root)
            rows = [{"number": number, "trait": "core_sample::kernel::Device", "type": "BadDevice", "file": file} for number, file in enumerate(later_files, start=2)]
            return trait_impl_lag_growth(root, rows, lag_library, table, break_switches)

        def script_family_green(label, base_file, later_file):
            problems, note = script_family_judged(label, base_file, [later_file])
            expect_true(label, problems == [] and note.startswith("与 gate 基准 ") and note.endswith(" 那一版比过、没涨"),
                        f"基准里 {base_file} 一行、此刻它的部分文件 {later_file} 一行，同一族、之和没超，期望没有问题、成功句写比过，实测问题 {problems}、成功句「{note}」")

        script_family_green("trait-impl-lag-family-python-split-green", "scripts/gate/data-tool.py", "scripts/gate/data_tool_base.py")
        script_family_green("trait-impl-lag-family-hook-split-green", ".claude/hooks/guard.sh", ".claude/hooks/guard-decisions.py")
        script_family_green("trait-impl-lag-family-shell-split-green", ".claude/gate.d/stage.sh", ".claude/gate.d/lib/stage-cells.sh")
        directory_entries = {"main.rs": ("crates/core/src/device_tool/main.rs", "crates/core/src/device_tool/bad_device.rs"),
                             "mod.rs": ("crates/core/src/device_tool/mod.rs", "crates/core/src/device_tool/nested/bad_device.rs")}
        directory_entry_results = {kind: script_family_judged(f"trait-impl-lag-family-directory-entry-{kind[:-3]}-root", base_file, [later_file])
                                   for kind, (base_file, later_file) in directory_entries.items()}
        expect_true("trait-impl-lag-family-directory-entry-green",
                    all(problems == [] and note.startswith("与 gate 基准 ") and note.endswith(" 那一版比过、没涨") for problems, note in directory_entry_results.values()),
                    f"基准里点名 main.rs、mod.rs 入口，此刻同目录与下层的 .rs 各一行（{'、'.join(later for _base, later in directory_entries.values())}），"
                    f"同一族、之和没超，期望没有问题、成功句写比过，实测 {directory_entry_results}")
        unrelated_files = {"python": ("scripts/gate/data-tool.py", "scripts/gate/other_tool.py"), "hook": (".claude/hooks/guard.sh", ".claude/hooks/other-decisions.py"),
                           "shell": (".claude/gate.d/stage.sh", ".claude/gate.d/lib/other-cells.sh")}
        unrelated_problems = {kind: script_family_judged(f"trait-impl-lag-family-{kind}-outside-root", base_file, [later_file])[0]
                              for kind, (base_file, later_file) in unrelated_files.items()}
        expect_true("trait-impl-lag-family-script-outside-red",
                    all(len(problems) == 1 and f"（{unrelated_files[kind][1]}）在 gate 基准 " in problems[0][0] and "那一版的表里没有：这张表只缩不涨" in problems[0][0]
                        for kind, problems in unrelated_problems.items()),
                    f"同目录名字对不上拆法的脚本（{'、'.join(later for _base, later in unrelated_files.values())}）期望各判红一处「那一版的表里没有」，实测 {unrelated_problems}")
    finally:
        os.environ.update(saved_git)

    # 整份算测试的文件（in_test_only_module_file）：kernel.rs 里 `mod device_tests;` 引进来的 kernel/device_tests.rs 里有 verus! 外面的 impl，
    # 声明带 #[cfg(test)] 的不判（开关 test-module-file-impl-judged 下判错），没带的照判
    def test_file_case(label, declaration, want_exit, want_text):
        root = make_root(label, kernel=selftest_kernel_with(trait_declaration, declaration))
        os.makedirs(os.path.join(root, "crates", "core", "src", "kernel"))
        with open(os.path.join(root, "crates", "core", "src", "kernel", "device_tests.rs"), "w") as handle:
            handle.write(impl_outside)
        expect(label, "proof", root, want_exit, want_text)

    test_file_case("trait-impl-test-module-file-not-judged", "\n#[cfg(test)]\nmod device_tests;\n", 0, "verus! 外面的 trait impl 0 处")
    test_file_case("trait-impl-plain-module-file-judged", "\nmod device_tests;\n", 1, "verus! 外面有 trait impl：crates/core/src/kernel/device_tests.rs:")
