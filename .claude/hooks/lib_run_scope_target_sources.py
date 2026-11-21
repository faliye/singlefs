"""lib_run_scope.py（run-scope-guard.sh 的判法）判点到的验证包测试目标会不会走崩溃枚举、读哪个缩域变量时读的测试源码，与它自证里的目录式目标样本。

测试目标有两种写法，cargo 都认、目标名都是 <名>：tests/<名>.rs 一个文件；tests/<名>/main.rs 一个目录（拆出的子模块放 tests/<名>/<子模块>.rs，
由 main.rs 的 `mod <子模块>;` 声明）。缩域变量名常写在子模块里，所以目录式目标读 main.rs 连同目录底下（含子目录）的全部 .rs，main.rs 排头、别的按路径排，换行接成一份文本；
tests/<名>.rs 读得了就读它。两种都读不到的交 None（调用方记「判不了」）。

缩域变量 CRASH_SHRINK_VARIABLES 的名字只在这里定义（lib_run_scope.py 取用）。弄坏开关（RUN_SCOPE_GUARD_BREAK 的项，开关与红的格登记在 BREAK_SWITCH_CELLS，
由 lib_run_scope.py 并进它自己的表）：
   crash-domain-directory-sources-unread  目录式目标不认，只读 tests/<名>.rs
   crash-domain-directory-main-only  目录式目标只读 main.rs，不读目录里别的 .rs
   crash-domain-directory-targets-always-enumerate  目录式目标的文本末尾多一行放量缩域变量名（等于它读了它），一律当会走崩溃枚举
"""
import glob
import os
import re

CRASH_SHRINK_VARIABLES = ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")
SOURCES_UNREAD_SWITCH = "crash-domain-directory-sources-unread"
MAIN_ONLY_SWITCH = "crash-domain-directory-main-only"
ALWAYS_ENUMERATES_SWITCH = "crash-domain-directory-targets-always-enumerate"
READS_JUDGE_VARIABLE_CELL = "cargo test 点名源码在 tests/<目标>/ 下、子模块里读判器缩域变量的目标、没缩域：拒"
READS_NO_VARIABLE_CELL = "cargo test 点名源码在 tests/<目标>/ 下、哪个文件都没读缩域变量的目标：不走崩溃枚举，放"
BREAK_SWITCH_CELLS = {
    SOURCES_UNREAD_SWITCH: [READS_JUDGE_VARIABLE_CELL],
    MAIN_ONLY_SWITCH: [READS_JUDGE_VARIABLE_CELL],
    ALWAYS_ENUMERATES_SWITCH: [READS_NO_VARIABLE_CELL],
}


def directory_target_lines(directory, read_text_lines, switch_on):
    """目录式测试目标 directory（tests/<名>）的源码行：main.rs 在才算，main.rs 排头，目录底下别的 .rs 按路径接在后面；没有 main.rs 交 None。"""
    main = read_text_lines(os.path.join(directory, "main.rs"))
    if main is None:
        return None
    lines = list(main)
    if not switch_on(MAIN_ONLY_SWITCH):
        for path in sorted(glob.glob(os.path.join(directory, "**", "*.rs"), recursive=True)):
            if os.path.relpath(path, directory) != "main.rs":
                lines += read_text_lines(path) or []
    if switch_on(ALWAYS_ENUMERATES_SWITCH):
        lines.append(CRASH_SHRINK_VARIABLES[0])
    return lines


def target_source_text(tests_path, target, read_text_lines, switch_on):
    """验证包测试目标 target 的源码文本（tests_path 是验证包 tests/ 的路径，read_text_lines 读文件交行表、读不了交 None，switch_on 问弄坏开关开没开）；读不到交 None。
    目标名不是一个词（带路径符号这类）的交 None。"""
    if not re.fullmatch(r"\w+", target):
        return None
    lines = read_text_lines(os.path.join(tests_path, target + ".rs"))
    if lines is None and not switch_on(SOURCES_UNREAD_SWITCH):
        lines = directory_target_lines(os.path.join(tests_path, target), read_text_lines, switch_on)
    return None if lines is None else "\n".join(lines)


def write_directory_target_samples(write, tests_directory):
    """自证样本树里的两个目录式目标（write(相对路径, 内容) 是 lib_run_scope.py build_selftest_tree 的写文件函数，tests_directory 是验证包 tests/ 的相对路径）：
    没登记、子模块里代码读判器缩域变量的；没登记、哪个文件都不读缩域变量的。两个的 main.rs 都只声明子模块。"""
    write(f"{tests_directory}/crash_judge_sample_directory/main.rs", "// 样本：目录式目标，没登记；main.rs 只声明子模块\nmod part;\n")
    write(f"{tests_directory}/crash_judge_sample_directory/part.rs", f"// 样本：子模块里代码读判器缩域变量\nconst VARIABLE: &str = \"{CRASH_SHRINK_VARIABLES[1]}\";\n")
    write(f"{tests_directory}/plain_sample_directory/main.rs", "// 样本：目录式目标，没登记；main.rs 只声明子模块\nmod part;\n")
    write(f"{tests_directory}/plain_sample_directory/part.rs", "// 样本：不走崩溃枚举\n")


def directory_target_selftest_cells(scope_text, package):
    """自证格（说明, agent_type, 命令, 该退几），格式同 lib_run_scope.py 的 selftest_cells；scope_text 写范围声明，package 是验证包名。"""
    declaration = scope_text("target", "验证包一个测试目标", "1 个测试目标")
    return [
        (READS_JUDGE_VARIABLE_CELL, None, f"{declaration} cargo test -p {package} --test crash_judge_sample_directory", 2),
        (READS_NO_VARIABLE_CELL, None, f"{declaration} cargo test -p {package} --test plain_sample_directory", 0),
    ]
