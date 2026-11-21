"""测试目标的名字到它的源文件：全仓一份认法，不各写一份。

测试目标有两种写法，cargo 都认、目标名都是 <名>：tests/<名>.rs 一个文件；tests/<名>/main.rs 一个目录——main.rs 是入口，
拆出的子模块放同一个目录，由 main.rs 的 `mod <子模块>;` 声明（子模块再声明的住在 <子模块>/ 底下，或是 <子模块>/mod.rs）。
没有 main.rs、只有 mod.rs 的子目录（tests/common_*/mod.rs 这类）是共用模块，不是目标；带 #[path = …] 的 mod 指到共用模块目录，不算这个目标自己的文件。
二进制目标（装置）同样两种写法：src/bin/<名>.rs 一个文件，或 src/bin/<名>/main.rs 加同目录由 `mod <子模块>;` 声明到的子模块；认法与测试目标是同一份，
binary_target_source_files 只是把目录换成 <包目录>/src/bin。

调它的（各自按 sys.path 或文件路径导入；只被 import、没有命令行入口，判法由调它的几份脚本各自的 --selftest 里目录式目标的格覆盖）：
  scripts/crash/crash-case-check.py                 登记、模块声明、命名、一个场景、来历表五样读测试目标的源码（directory_targets、provenance_problems）；
  scripts/crash/lib_crash_point_replay_items.py     按项读库的设点文件（setting_files_of）与 target-source-file 子命令（两个放量入口写上次红的清单时取源文件，经 target_root_file）；
  .claude/gate.d/lib/code-tooling-source-scan-cells.sh  memory-bounded-parallelism 格：登记的 cargo:<包>:<目标> 入口罩着哪些文件（按文件路径载入，测试目标调 target_source_files、二进制目标调 binary_target_source_files）。
读 tests/<名>/ 的几处调同一个 target_source_files；只要目标根文件（tests/<名>.rs 或 tests/<名>/main.rs）时调 target_root_file；读 src/bin/<名>/ 的调 binary_target_source_files。

关键字参数是各调用方弄坏开关的落点（平常取默认值）：
  follow_submodules=False   目录式目标只收 main.rs，不沿 mod 声明收子模块
  every_file=True           目录式目标收目录里的全部 .rs，没被 mod 声明的也收
  entries_without_main=True 只有 mod.rs、没有 main.rs 的共用模块目录也当目标
"""
import glob
import os
import re

RUST_LINE_COMMENT = re.compile(r"//[^\n]*")
RUST_BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
# `mod <名>;`（前面可带属性行与 pub 可见性；带函数体的内联 mod 不算）；第一组是它前面的属性行
SUBMODULE_FILE_DECLARATION = re.compile(r"^[ \t]*((?:#\[[^\]\n]*\][ \t]*\n[ \t]*)*)(?:pub(?:\([^)]*\))?[ \t]+)?mod[ \t]+(\w+)[ \t]*;", re.M)
PATH_ATTRIBUTE = re.compile(r"#\[\s*path\b")


def target_root_file(tests_directory, target):
    """tests_directory（<包目录>/tests）下目标 target 的根文件：tests/<名>.rs 在就是它，不然 tests/<名>/main.rs 在就是它；两样都不在交 None。"""
    for candidate in (os.path.join(tests_directory, target + ".rs"), os.path.join(tests_directory, target, "main.rs")):
        if os.path.isfile(candidate):
            return candidate
    return None


def declared_submodules(text):
    """text 里去掉注释之后 `mod <名>;` 声明的子模块名，照出现的次序；带 #[path = …] 的不算。"""
    code = RUST_BLOCK_COMMENT.sub("", RUST_LINE_COMMENT.sub("", text))
    return [found.group(2) for found in SUBMODULE_FILE_DECLARATION.finditer(code) if not PATH_ATTRIBUTE.search(found.group(1))]


def directory_target_files(entry, follow_submodules=True, every_file=False):
    """目录式测试目标的入口 entry（tests/<名>/main.rs）连同它沿 `mod <名>;` 声明到的子模块文件，entry 排头、其余照发现的次序、去重：
    <名>.rs 或 <名>/mod.rs 都在声明它的那个文件所在的目录里找（main.rs 与 mod.rs 的子模块住在它们自己的目录，别的文件的子模块住在 <它的名字>/ 底下）。"""
    directory = os.path.dirname(entry)
    if not follow_submodules:
        return [entry]
    if every_file:
        return [entry] + [path for path in sorted(glob.glob(os.path.join(directory, "**", "*.rs"), recursive=True)) if path != entry]
    files, pending = [], [(entry, directory)]
    while pending:
        path, module_directory = pending.pop(0)
        if path in files or not os.path.isfile(path):
            continue
        files.append(path)
        for name in declared_submodules(open(path, encoding="utf-8", errors="replace").read()):
            flat, nested = os.path.join(module_directory, name + ".rs"), os.path.join(module_directory, name, "mod.rs")
            pending.append((flat if os.path.isfile(flat) else nested, os.path.join(module_directory, name)))
    return files


def directory_target_entries(tests_directory, entries_without_main=False):
    """tests_directory 下目录式测试目标的入口文件（有 main.rs 的子目录的 main.rs），按路径排。"""
    entries = glob.glob(os.path.join(tests_directory, "*", "main.rs"))
    if entries_without_main:
        entries += glob.glob(os.path.join(tests_directory, "*", "mod.rs"))
    return sorted(entries)


def target_source_files(tests_directory, target, follow_submodules=True, every_file=False):
    """目标 target 的源文件，根文件排头：tests/<名>.rs 一份；目录式目标是 main.rs 连同它 mod 声明到的子模块（directory_target_files）；目标不在交空表。"""
    flat = os.path.join(tests_directory, target + ".rs")
    if os.path.isfile(flat):
        return [flat]
    entry = os.path.join(tests_directory, target, "main.rs")
    return directory_target_files(entry, follow_submodules, every_file) if os.path.isfile(entry) else []


def binary_target_source_files(package_directory, target, follow_submodules=True, every_file=False):
    """包目录 package_directory 下二进制目标 target 的源文件，根文件排头：src/bin/<名>.rs 一份；src/bin/<名>/main.rs 连同它 mod 声明到的子模块（directory_target_files）；目标不在交空表。"""
    return target_source_files(os.path.join(package_directory, "src", "bin"), target, follow_submodules, every_file)
