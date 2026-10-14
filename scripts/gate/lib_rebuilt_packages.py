#!/usr/bin/env python3
# admission: always 判的是这一趟的编译输出与这一次给的补丁，上一次的结论不替这一次作保
# run-condition: none 只读补丁、源树里的 Cargo.toml 与编译日志，touch 只改给定源树里的修改时刻，不起子进程
"""拿副本编的脚本核「改过的每个包这一趟都重编了」：改过的包从补丁或变异行算出来，编译输出里那个包要有 `Compiling <包名> v…` 行，没有就判红。

为什么：几份仓副本共用一个编译目录、或拷副本时留着旧修改时刻，cargo 按 mtime 判本地包不用重编，测试静默跑在旧产物上
（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行；.claude/singlefs-ai-sop/rules/test-script-discipline.md「编译与算法都要快」）。

调它的：scripts/gate/merge-and-verify.sh（编完之后核补丁改过的包，命令行 check；编之前把补丁改过的文件刷到此刻，命令行 touch）、
scripts/mutation/prove-red.sh（每条变异施加后那一趟编译核被改的那个包，按路径载入本库调 package_of / compiled_packages）。

入口：
  paths_in_patch(text) -> [相对仓根的路径]          `+++ b/<路径>` 与 `--- a/<路径>` 两种行，去重、按出现次序；/dev/null 不算
  package_of(tree, relative) -> 包名 | None           从文件所在目录往上找到第一份带 [package] 的 Cargo.toml（不出树根），交它的 name
  changed_packages(tree, relatives) -> (包名排好序, 不核的路径)   只认落在包里的 .rs（rustc 的 dep-info 只记它们，表、文档、Cargo.toml 改了 cargo 不一定重编）
  files_known_to_cargo(target_directory, 绝对路径) -> (dep-info 里出现过的, 有没有 dep-info)
  packages_to_check(tree, relatives, target_directory) -> (要核的包名, 不核的说明)   包里改过的 .rs 一个都不在编译目录的 dep-info 里的不核
      （编完再看：编过的在新 dep-info 里，没重编的在上一次留下的 dep-info 里；都没有的是这几趟编译不读的，例如没开的 feature 下才编的模块）
  compiled_packages(text) -> {包名}                   编译输出里 `Compiling <包名> v<版本>` 的行（行首空白不算）
  missing_rebuilds(changed, texts) -> [包名]          改过而几份编译输出里都没有 Compiling 行的
命令行：
  lib_rebuilt_packages.py check --tree <树> [--target-dir <编译目录>] --log <编译日志>… (--patch-dir <补丁目录> | --path <相对树根的路径>)…
      退 0 改过的包都有 Compiling 行；1 有包没重编（逐个列名，出路：刷源码修改时刻或换干净编译目录重跑）；
      77 本次未判（补丁里没有要核的包）；2 用法错、读不了日志或补丁
  lib_rebuilt_packages.py touch --tree <树> (--patch-dir <补丁目录> | --path <路径>)…
      把落在包里、树里还在的那几个文件的修改时刻刷到此刻（cargo 下一次必定重编它们的包）；退 0，读不了补丁或 touch 不成退 2
  lib_rebuilt_packages.py --write-selftest-cargo <路径>   只给自证：写一份按包、按 mtime 判新不新鲜的假 cargo（见 FAKE_CARGO）
  lib_rebuilt_packages.py --selftest
补丁目录里读 crates.patch（.claude/agents/implementation-writer.md「产出」一节的格式）；没有这份文件的补丁目录不贡献路径。

弄坏开关 REBUILT_PACKAGES_BREAK=<项>（只给证红用）。项名、弄坏哪一处、打开后自证红在哪几格：
  any-compiling-line  编译输出里有任何一行 Compiling 就当改过的包全都重编了
      本库「改过两个包、只重编了一个：点名没重编的那个」；连带 merge-and-verify.sh「编译目录里的产物比源码新」那一格
  patch-new-side-only 只认 `+++ b/` 行，不认 `--- a/`（删掉的文件不算改过）
      本库「补丁里取出改过的路径（新旧两侧、去重、不认 /dev/null）」「补丁删了一个文件：那个包也算改过」
  ignore-dep-info     不看编译目录的 dep-info，包里改过 .rs 的都核
      本库「改过的 .rs 不在编译目录的任何 dep-info 里（没开的 feature）：那个包不核」
"""
import glob
import os
import re
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import tomllib  # noqa: E402

BREAK_VARIABLE = "REBUILT_PACKAGES_BREAK"
EXIT_RED = 1
EXIT_USAGE = 2
EXIT_NOT_RUN = 77
PATCH_FILE_NAME = "crates.patch"
COMPILING_LINE = re.compile(r"^\s*Compiling (?P<name>\S+) v\S+", re.M)


def broken(item):
    return os.environ.get(BREAK_VARIABLE, "") == item


def paths_in_patch(text):
    """补丁里改过的路径：`+++ b/<路径>`（新的一侧）与 `--- a/<路径>`（旧的一侧，删掉的文件只在这一侧）。"""
    found = []
    for line in text.splitlines():
        if line.startswith("+++ b/"):
            path = line[len("+++ b/"):]
        elif line.startswith("--- a/") and not broken("patch-new-side-only"):
            path = line[len("--- a/"):]
        else:
            continue
        path = path.split("\t", 1)[0].strip()
        if path and path not in found:
            found.append(path)
    return found


def package_of(tree, relative):
    """文件所在目录往上第一份带 [package] 的 Cargo.toml 的包名；到树根还没有、或读不了，交 None。"""
    tree = os.path.realpath(tree)
    directory = os.path.dirname(os.path.join(tree, relative))
    while True:
        manifest = os.path.join(directory, "Cargo.toml")
        if os.path.isfile(manifest):
            try:
                with open(manifest, "rb") as handle:
                    name = tomllib.load(handle).get("package", {}).get("name")
            except (OSError, tomllib.TOMLDecodeError):
                name = None
            if isinstance(name, str) and name:
                return name
        if os.path.realpath(directory) == tree or os.path.dirname(directory) == directory:
            return None
        directory = os.path.dirname(directory)


def files_by_package(tree, relatives):
    """({包名: [改过的 .rs 文件]}, 不核的路径)：只有落在包里的 .rs 文件进 rustc 的 dep-info、cargo 按它们判新不新鲜；
    别的（表、文档、Cargo.toml、不在哪个包里的）改了 cargo 不一定重编，不拿来核。"""
    grouped, skipped = {}, []
    for relative in relatives:
        name = package_of(tree, relative) if relative.endswith(".rs") else None
        if name is None:
            skipped.append(relative)
        else:
            grouped.setdefault(name, []).append(relative)
    return grouped, skipped


def changed_packages(tree, relatives):
    """(改过的包名排好序, 不核的路径)。"""
    grouped, skipped = files_by_package(tree, relatives)
    return sorted(grouped), skipped


def dep_info_files(target_directory):
    """编译目录里 rustc 写的 dep-info（<编译目录>/<档>/deps/*.d 与 <编译目录>/<目标平台>/<档>/deps/*.d）。"""
    return sorted(glob.glob(os.path.join(target_directory, "*", "deps", "*.d")) + glob.glob(os.path.join(target_directory, "*", "*", "deps", "*.d")))


def files_known_to_cargo(target_directory, absolute_paths):
    """→ (dep-info 里出现过的那几个绝对路径, 编译目录里有没有 dep-info)。编完再调：这一趟编过的文件在新的 dep-info 里，
    没重编的文件在上一次编它时留下的 dep-info 里；一份 dep-info 里都没有的，是这几趟编译根本不读的（没开的 feature 下才编的模块、没被引用的文件）。"""
    wanted, found = set(absolute_paths), set()
    files = dep_info_files(target_directory)
    for path in files:
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                tokens = handle.read().split()
        except OSError:
            continue
        found |= wanted & {token.rstrip(":") for token in tokens}
        if found == wanted:
            break
    return found, bool(files)


def compiled_packages(text):
    return {match.group("name") for match in COMPILING_LINE.finditer(text)}


def missing_rebuilds(changed, texts):
    compiled = set()
    for text in texts:
        compiled |= compiled_packages(text)
    if broken("any-compiling-line") and compiled:
        return []
    return [name for name in changed if name not in compiled]


def patch_directory_paths(directory):
    path = os.path.join(directory, PATCH_FILE_NAME)
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8", errors="replace") as handle:
        return paths_in_patch(handle.read())


def collect_paths(patch_directories, paths):
    found = []
    for directory in patch_directories:
        if not os.path.isdir(directory):
            raise OSError(f"补丁目录 {directory} 不在")
        found += patch_directory_paths(directory)
    return list(dict.fromkeys(found + list(paths)))


def packages_to_check(tree, relatives, target_directory=None):
    """→ (要核的包名排好序, 不核的说明一句或 "")。target_directory 给了而里面有 dep-info：包里改过的 .rs 一个都不在 dep-info 里的包不核
    （这几趟编译根本不读它们，见 files_known_to_cargo）；编译目录里一份 dep-info 都没有（头一次编）就全核。"""
    grouped, skipped = files_by_package(tree, relatives)
    notes = []
    if skipped:
        notes.append(f"不是包里的 .rs 的 {len(skipped)} 个文件不核：{'、'.join(skipped[:5])}{' …' if len(skipped) > 5 else ''}")
    if target_directory and grouped and not broken("ignore-dep-info"):
        real_tree = os.path.realpath(tree)
        absolute = {relative: {os.path.join(real_tree, relative), os.path.abspath(os.path.join(tree, relative))} for files in grouped.values() for relative in files}
        found, has_dep_info = files_known_to_cargo(target_directory, set().union(*absolute.values()))
        if has_dep_info:
            unread = sorted(name for name, files in grouped.items() if not any(absolute[relative] & found for relative in files))
            if unread:
                notes.append(f"{'、'.join(unread)} 改过的 .rs 不在编译目录 {target_directory} 的任何 dep-info 里（这几趟编译不读它们：没开的 feature 下才编、或没被引用），不核")
            grouped = {name: files for name, files in grouped.items() if name not in unread}
    return sorted(grouped), ("（" + "；".join(notes) + "）") if notes else ""


def check(tree, logs, relatives, target_directory=None):
    """→ (退出码, [输出行])。"""
    texts = []
    for log in logs:
        try:
            with open(log, encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
        except OSError as error:
            return EXIT_USAGE, [f"✗ 读不了编译日志 {log}：{error}", "→ 怎么办：看那一趟 cargo 有没有写出这份日志；判不了重编没有，不算绿"]
    changed, note = packages_to_check(tree, relatives, target_directory)
    if not changed:
        return EXIT_NOT_RUN, [f"- 重编核对本次未判：补丁里没有要核的包{note}"]
    missing = missing_rebuilds(changed, texts)
    if missing:
        return EXIT_RED, [f"✗ 改过的包这一趟没重编：{'、'.join(missing)}（编译输出 {'、'.join(logs)} 里没有「Compiling <包名> v…」行；"
                          f"核了 {len(changed)} 个包：{'、'.join(changed)}）{note}",
                          "→ 怎么办：编译目录里的产物比源码新，cargo 认它们已编、测试跑的是旧产物。把那几个包的源文件修改时刻刷到此刻"
                          "（python3 scripts/hosts/source-stamp.py refresh <树>，或 touch 改过的文件），或设 CARGO_TARGET_DIR 换一个干净的编译目录，再整趟重跑"]
    return 0, [f"✓ 改过的包这一趟都重编了（核了 {len(changed)} 个包：{'、'.join(changed)}）{note}"]


def touch(tree, relatives):
    """把落在包里、树里还在的 .rs 刷到此刻：→ 刷了几个。"""
    grouped, _skipped = files_by_package(tree, relatives)
    count = 0
    for files in grouped.values():
        for relative in files:
            path = os.path.join(tree, relative)
            if os.path.isfile(path):
                os.utime(path)
                count += 1
    return count


# ── 自证用的假 cargo ──
FAKE_CARGO = r'''#!/usr/bin/env python3
# 自证用的假 cargo（scripts/gate/lib_rebuilt_packages.py --write-selftest-cargo 写的）：照 cargo 认本地包新不新鲜的办法按包编——
# crates/ 下每个带 [package] 的目录是一个包，包里任一文件的 mtime 比编译目录里这个包的 dep-info 新、或还没编过，才重编，
# 重编时照真 cargo 打「   Compiling <包名> v0.1.0 (<目录>)」；不重编什么都不打（真 cargo 不带 -v 时也不打 Fresh）。
# test 每个包打一条 ok（产物旧不旧测试都绿：这就是只靠测试结果看不出来的那种），fmt、clippy 退 0。
# FAKE_CARGO_TIMELINE 设了：test 开头与结尾各往那份文件追加一行「start|end <FAKE_CARGO_TAG> <纳秒>」；FAKE_CARGO_SLEEP 设了 test 中间睡这么多秒。
import os, sys, time
subcommand = sys.argv[1] if len(sys.argv) > 1 else ""
if subcommand in ("fmt", "clippy"):
    sys.exit(0)
def mark(word):
    if os.environ.get("FAKE_CARGO_TIMELINE"):
        with open(os.environ["FAKE_CARGO_TIMELINE"], "a") as handle:
            handle.write(f"{word} {os.environ.get('FAKE_CARGO_TAG', '-')} {time.time_ns()}\n")
if subcommand == "test":
    mark("start")
    time.sleep(float(os.environ.get("FAKE_CARGO_SLEEP", "0") or 0))
target = os.environ.get("CARGO_TARGET_DIR") or "target"
os.makedirs(os.path.join(target, "debug", ".fake"), exist_ok=True)
packages = []
for directory, directories, names in os.walk("crates"):
    directories[:] = [name for name in directories if name != "target"]
    if "Cargo.toml" in names:
        text = open(os.path.join(directory, "Cargo.toml")).read()
        for line in text.splitlines():
            if line.startswith("name = "):
                packages.append((line.split('"')[1], directory))
                break
for name, directory in sorted(packages):
    newest = max(os.stat(os.path.join(root, file_name)).st_mtime_ns for root, _, file_names in os.walk(directory) for file_name in file_names)
    depinfo = os.path.join(target, "debug", ".fake", name + ".d")
    if not os.path.exists(depinfo) or newest > os.stat(depinfo).st_mtime_ns:
        print(f"   Compiling {name} v0.1.0 ({os.path.abspath(directory)})")
        with open(depinfo, "a"):
            pass
        # 照 rustc 写 dep-info：<编译目录>/debug/deps/<包>.d，列出这个包的每个 .rs（绝对路径）
        os.makedirs(os.path.join(target, "debug", "deps"), exist_ok=True)
        rust_files = sorted(os.path.abspath(os.path.join(root, file_name)) for root, _, file_names in os.walk(directory) for file_name in file_names if file_name.endswith(".rs"))
        with open(os.path.join(target, "debug", "deps", name.replace("-", "_") + ".d"), "w") as handle:
            handle.write(f"{os.path.abspath(depinfo)}: {' '.join(rust_files)}\n")
        os.utime(depinfo)
if subcommand == "test":
    print(f"running {len(packages)} tests")
    for name, _ in sorted(packages):
        print(f"test {name.replace('-', '_')}::builds ... ok")
    print(f"test result: ok. {len(packages)} passed; 0 failed")
    mark("end")
sys.exit(0)
'''


def write_fake_cargo(path):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(FAKE_CARGO)
    os.chmod(path, 0o755)


def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="lib-rebuilt-packages-selftest-")
    try:
        tree = os.path.join(work, "tree")
        for directory, manifest in (("", '[workspace]\nmembers = ["crates/alpha", "crates/beta"]\n'),
                                    ("crates/alpha", '[package]\nname = "alpha"\n'), ("crates/beta", '[package]\nname = "beta-two"\n')):
            os.makedirs(os.path.join(tree, directory, "src") if directory else tree, exist_ok=True)
            with open(os.path.join(tree, directory, "Cargo.toml"), "w") as handle:
                handle.write(manifest)
        for relative in ("crates/alpha/src/lib.rs", "crates/beta/src/walk.rs"):
            with open(os.path.join(tree, relative), "w") as handle:
                handle.write("pub fn f() {}\n")
        os.makedirs(os.path.join(tree, "litmus"))
        patch = ("diff --git a/crates/alpha/src/lib.rs b/crates/alpha/src/lib.rs\n--- a/crates/alpha/src/lib.rs\n+++ b/crates/alpha/src/lib.rs\n@@ -1 +1 @@\n-a\n+b\n"
                 "diff --git a/crates/beta/src/gone.rs b/crates/beta/src/gone.rs\ndeleted file mode 100644\n--- a/crates/beta/src/gone.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-x\n"
                 "diff --git a/litmus/x.litmus b/litmus/x.litmus\n--- a/litmus/x.litmus\n+++ b/litmus/x.litmus\n@@ -1 +1 @@\n-a\n+b\n")

        def cell(name, condition, detail):
            nonlocal checked
            checked += 1
            if not condition:
                failures.append(f"{name}：{detail}")

        paths = paths_in_patch(patch)
        cell("补丁里取出改过的路径（新旧两侧、去重、不认 /dev/null）", paths == ["crates/alpha/src/lib.rs", "crates/beta/src/gone.rs", "litmus/x.litmus"], f"取出 {paths}")
        changed, unowned = changed_packages(tree, paths)
        cell("补丁删了一个文件：那个包也算改过", changed == ["alpha", "beta-two"] and unowned == ["litmus/x.litmus"], f"包 {changed}，不属于包的 {unowned}")
        log = "   Compiling alpha v0.1.0 (/x/crates/alpha)\n    Finished `test` profile\n"
        missing = missing_rebuilds(["alpha", "beta-two"], [log])
        cell("改过两个包、只重编了一个：点名没重编的那个", missing == ["beta-two"], f"交 {missing}")
        missing = missing_rebuilds(["alpha", "beta-two"], [log, "   Compiling beta-two v0.1.0 (/x/crates/beta)\n"])
        cell("两份编译输出合起来都重编了：判绿", missing == [], f"交 {missing}")
        logs_directory = os.path.join(work, "logs")
        os.makedirs(logs_directory)
        red_log, green_log = os.path.join(logs_directory, "red.log"), os.path.join(logs_directory, "green.log")
        with open(red_log, "w") as handle:
            handle.write("       Fresh alpha v0.1.0\ntest result: ok. 1 passed; 0 failed\n")
        with open(green_log, "w") as handle:
            handle.write("   Compiling alpha v0.1.0 (/x/crates/alpha)\ntest result: ok. 1 passed; 0 failed\n")
        code, lines = check(tree, [red_log], ["crates/alpha/src/lib.rs"])
        cell("红样本（只有 Fresh、没有 Compiling alpha）：退 1、点名 alpha、出路写刷修改时刻或换编译目录", code == EXIT_RED and "alpha" in lines[0] and "CARGO_TARGET_DIR" in lines[1], f"退 {code} {lines}")
        code, lines = check(tree, [green_log], ["crates/alpha/src/lib.rs"])
        cell("绿样本（有 Compiling alpha）：退 0", code == 0 and lines[0].startswith("✓"), f"退 {code} {lines}")
        code, lines = check(tree, [green_log], ["litmus/x.litmus"])
        cell("补丁里没有落在包里的文件：本次未判、退 77", code == EXIT_NOT_RUN, f"退 {code} {lines}")
        walk = os.path.join(tree, "crates", "beta", "src", "walk.rs")
        os.utime(walk, (1577836800, 1577836800))
        count = touch(tree, ["crates/beta/src/walk.rs", "crates/beta/src/gone.rs", "litmus/x.litmus"])
        cell("touch 只刷落在包里、还在的文件", count == 1 and os.stat(walk).st_mtime > 1577836800, f"刷了 {count} 个，walk.rs 的 mtime {os.stat(walk).st_mtime}")
        changed, skipped = changed_packages(tree, ["crates/alpha/Cargo.toml", "crates/beta/src/table.tsv"])
        cell("包里改的不是 .rs（Cargo.toml、表）：不核", changed == [] and len(skipped) == 2, f"包 {changed}，不核的 {skipped}")
        # 编译目录里的 dep-info：alpha 的 lib.rs 编过（在 dep-info 里），beta-two 的 walk.rs 一份 dep-info 里都没有（没开的 feature 下才编的那种）
        target = os.path.join(work, "target")
        os.makedirs(os.path.join(target, "debug", "deps"))
        with open(os.path.join(target, "debug", "deps", "alpha-0123.d"), "w") as handle:
            handle.write(f"{target}/debug/deps/alpha-0123.d: {os.path.realpath(tree)}/crates/alpha/src/lib.rs\n\n{os.path.realpath(tree)}/crates/alpha/src/lib.rs:\n")
        both = ["crates/alpha/src/lib.rs", "crates/beta/src/walk.rs"]
        code, lines = check(tree, [green_log], both, target)
        cell("改过的 .rs 不在编译目录的任何 dep-info 里（没开的 feature）：那个包不核", code == 0 and "beta-two" in lines[0] and "不核" in lines[0] and "核了 1 个包：alpha" in lines[0],
             f"退 {code} {lines}")
        code, lines = check(tree, [red_log], both, target)
        cell("没重编的文件在上一次留下的 dep-info 里：照核、判红", code == EXIT_RED and "没重编：alpha" in lines[0], f"退 {code} {lines}")
        code, lines = check(tree, [red_log], both, os.path.join(work, "empty-target"))
        cell("编译目录里一份 dep-info 都没有（头一次编）：全核", code == EXIT_RED and "alpha、beta-two" in lines[0], f"退 {code} {lines}")
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 paths_in_patch / changed_packages / missing_rebuilds / check / touch；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return EXIT_RED
    print(f"  ✓ lib_rebuilt_packages 自检通过（查了 {checked} 格）：补丁取路径认新旧两侧、删掉的文件也算、只重编一个包时点名另一个、红样本退 1 绿样本退 0、没有落在包里的文件退 77、touch 只刷包里还在的、"
          f"不是 .rs 的不核、不在 dep-info 里的包不核、在上一次 dep-info 里而没重编的照红、没有 dep-info 时全核")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if len(arguments) == 2 and arguments[0] == "--write-selftest-cargo":
        write_fake_cargo(arguments[1])
        return 0
    usage = ["✗ 用法不对", "→ 怎么办：lib_rebuilt_packages.py check --tree <树> [--target-dir <编译目录>] --log <日志>… (--patch-dir <目录> | --path <路径>)…；"
             "touch --tree <树> (--patch-dir <目录> | --path <路径>)…"]
    if not arguments or arguments[0] not in ("check", "touch"):
        print("\n".join(usage))
        return EXIT_USAGE
    command, tree, target_directory, logs, patch_directories, paths = arguments[0], None, None, [], [], []
    rest = arguments[1:]
    while rest:
        if len(rest) < 2 or rest[0] not in ("--tree", "--target-dir", "--log", "--patch-dir", "--path"):
            print("\n".join(usage))
            return EXIT_USAGE
        option, value, rest = rest[0], rest[1], rest[2:]
        if option == "--tree":
            tree = value
        elif option == "--target-dir":
            target_directory = value
        else:
            {"--log": logs, "--patch-dir": patch_directories, "--path": paths}[option].append(value)
    if tree is None or (command == "check" and not logs):
        print("\n".join(usage))
        return EXIT_USAGE
    try:
        relatives = collect_paths(patch_directories, paths)
        if command == "touch":
            print(f"  刷到此刻：补丁改过的文件里落在包里、还在的 {touch(tree, relatives)} 个")
            return 0
    except OSError as error:
        print(f"✗ {error}\n→ 怎么办：看补丁目录与树在不在、可不可写；判不了就不算绿")
        return EXIT_USAGE
    code, lines = check(tree, logs, relatives, target_directory)
    print("\n".join(lines))
    return code


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
