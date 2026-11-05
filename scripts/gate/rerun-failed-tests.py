#!/usr/bin/env python3
# admission: always 读的是这一次给的那份日志，上一次的结论不替这一次作保
# run-condition: none 只读一份文本日志与 --root 那个 cargo 工作区里各包的 Cargo.toml、打印命令，不起 cargo，不写任何文件
"""从一份 cargo test 日志里取出失败的用例，按测试目标各打印一条只跑它们的命令；自己不起 cargo。

用法：
  rerun-failed-tests.py <cargo test 的日志> -p <包名> [--root <cargo 工作区根>]   # 每个有失败、属于 <包名> 的测试目标一行命令，打到 stdout
  rerun-failed-tests.py <cargo test 的日志> --item <包名>:<目标根文件> [--item …] [--root <cargo 工作区根>]   # 只打点名的那几项里有失败的
  rerun-failed-tests.py --selftest
--root 不给取本脚本所在的仓（仓根的 Cargo.toml 是工作区）；实验工作区的日志给 --root scripts/experiment-apparatus。
-p 与 --item 只给一种；-p 只给一次，--item 给几次取并集。
项的写法（与 scripts/gate/merge-and-verify.sh 的 --item、--list-items 同一套，项的清单与「日志里一段属于哪一项」只在这里写一份，合入后验证经 importlib 调）：
  <包名>:<目标根文件>   目标根文件从包目录起，就是 cargo 段头 `Running …` 里的那个源文件：src/lib.rs、tests/<名>.rs、src/bin/<名>.rs 这一类；
  <包名>:doc            那个包的文档测试（段头 `Doc-tests <库名>`）。
  一个包有哪几项（test_targets_of_package）：照它的 Cargo.toml 与目录认 cargo test 不带特性时会跑的测试目标——库（[lib] test = false 的不算）、
    [[bin]] / [[test]] 写出的与 src/main.rs、src/bin/、tests/ 下自动认的（autobins / autotests = false 的不自动认；写了 required-features 的不带特性编不进，不算；
    test = false 的不算），[[example]] / [[bench]] 只算写了 test = true 的；库要跑文档测试（[lib] doctest 不是 false）再加一项 doc。
  挑目标的参数取清单写的名字：[[bin]] name = "unit-graph" path = "src/bin/unit_graph.rs" 那一项是 --bin unit-graph，不是照文件名猜的 unit_graph。
  日志里一段属于哪一项（section_item）：段头的源文件是那个包的一项、而且段头括号里编出来的文件名的 crate 名等于那一项的 crate 名
    （库是库名，别的是目标名把 - 换成 _）；文档测试那一段按库名认。一个包都对不上交 None（认不出），几个包都对得上交空串（拿不准），两种都不归到哪一项。
  --item 的项写错（包不在、那个包没有这一项）退 2，列出那个包可点的项（包不在列工作区里的包）。点名之外的段在 stderr 逐段点名「不在点名的项里，没打」。
按包筛目标：段头 `Running tests/<目标>.rs (…/deps/<目标>-…)` 不带包名，要读工作区：先照 <根>/Cargo.toml 的 [package] 与 [workspace] members
（展开通配、去掉 exclude）找到 name 等于 <包名> 的那个包（找不到退 2），再逐段判：
  --test <目标>  <包目录>/tests/<目标>.rs 在，或那个包 Cargo.toml 的 [[test]] name 等于 <目标>；
  --lib          段头括号里 deps 文件名的 crate 名（<名>-<哈希> 的 <名>）等于这个包的 lib 名（[lib] name，没写是包名把 - 换成 _）；
  --bin <名>     <包目录>/src/bin/<名>.rs 在，或 [[bin]] name 等于 <名>。
不属于这个包的段不打命令，在 stderr 逐段点名「属于别的包，没打」；属于的段照打。
打印出来的命令交给调用方原样经 Bash 起：每条都是 `SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test …`，经 task-server 跑
（不加内存包装、不写线程：上限、线程与机器都由服务定；records/2026-10-30-编译绕过内存总上限.md「收尾批」），钩子看得见它；脚本里起就绕过了闸。
子 agent 不自己起：经 submit 交的范围声明闸一律拒，照共用约束「子 agent 不交任务」写进报告「要主 agent 跑的命令」一节，由主 agent 起。
每条命令开头带范围声明（.claude/hooks/run-scope-guard.sh 文件头的写法），写在提交客户端前面，客户端照白名单带进任务：
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<包> <挑目标的参数> 里上一趟红的用例; 范围=<N> 个测试; 为什么=只重跑 <日志文件名> 里红的这 <N> 条用例，修完看它们转绿，不整份重跑'
  N 是那一条 `--exact` 后面点名的用例数；日志文件名里的 ; 与 = 换成 _（声明按 ; 与 = 切段）。
认的段头：`Running tests/<目标>.rs (…)` → `--test <目标>`；`Running unittests src/lib.rs (…)` → `--lib`；
`Running unittests src/bin/<名>.rs (…)` → `--bin <名>`；`Doc-tests` 段不认（文档测试按名字重跑不了），有失败就判红要人看。
完整性闸：每一段收到的 `test <名> ... FAILED` 条数要等于那一段 `test result:` 行报的 failed 数，对不上判红、stdout 不出命令。
没有段头的测试结果、认不出目标的新 Running 段都不并入前一个目标；无法定位的失败判红，不生成可能跑错目标的命令。
结果行与汇总行的解析在 scripts/gate/lib_libtest_result_lines.py（全仓一份）：#[should_panic] 的用例那一段「 - should panic」不进名字。
一个失败都没有：打印「没有失败的用例」到 stderr、退出 0、stdout 为空。
弄坏开关（只给自证证红用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认），一项一行，下一行起写打开后判错的自证格：
  RERUN_FAILED_TESTS_BREAK=merge-sections  段头之后的段并进第一段
      「按段取失败用例」「最后一段缺 test result 行应当判出问题」「FAILED 行与 test result 对不上应当判出问题」
      「按包筛目标：两个包各有失败的段」「按项筛目标：点名 rerun-target 的 tests/alpha.rs」（段都并进第一段，按包、按项都筛不开）
  RERUN_FAILED_TESTS_BREAK=own-failed-line  回到自己的正则 `^test (\\S+) \\.\\.\\. FAILED`，不经 lib_libtest_result_lines.py（认不了「 - should panic」）
      「should panic 的失败用例按全名重跑」
  lib_libtest_result_lines.py 的 LIBTEST_RESULT_LINES_BREAK=should-panic-in-name 同样让「should panic 的失败用例按全名重跑」判错。
  RERUN_FAILED_TESTS_BREAK=scope-dropped  打出的命令不带范围声明（改前的写法）
      「打出的每条命令开头带范围声明、范围声明闸照主 agent 判放行」
  RERUN_FAILED_TESTS_BREAK=submit-dropped  打出的命令不经 task-server/task submit、照改前直接是 cargo test
      「打出的每条命令经 task-server/task submit -- 起」
  RERUN_FAILED_TESTS_BREAK=package-filter-dropped  不按包筛目标，日志里每个有失败的段都打（改前的样子）
      「按包筛目标：两个包各有失败的段，-p 那个包只打它自己的两条、stderr 点名别的包那两段」
  RERUN_FAILED_TESTS_BREAK=test-arguments-added  打出的 cargo test 在 -- 之后，--exact 前面多带一个参数（--show-output）
      「按段取失败用例」「should panic 的失败用例按全名重跑」「按包筛目标：两个包各有失败的段，-p 那个包只打它自己的两条、stderr 点名别的包那两段」
      「按项筛目标：点名 rerun-target 的 tests/alpha.rs，只打它那一条、stderr 点名别的三段」「[[bin]] 写了 path 的项按清单里的名字挑」
  RERUN_FAILED_TESTS_BREAK=item-filter-dropped  --item 只认它的包，那个包里有失败的段全打（点名的目标不筛）
      「按项筛目标：点名 rerun-target 的 tests/alpha.rs，只打它那一条、stderr 点名别的三段」
  RERUN_FAILED_TESTS_BREAK=item-unknown-accepted  --item 点了包里没有的目标也照收（当没点那一项，不退 2）
      「项写错退 2：rerun-target 没有 tests/missing.rs，列出它可点的项」
  RERUN_FAILED_TESTS_BREAK=bin-selector-from-file-name  挑目标的参数照段头的文件名猜（src/bin/tool_x.rs → --bin tool_x），不读清单里写的名字
      「[[bin]] 写了 path 的项按清单里的名字挑」
"""
import glob
import os
import re
import shlex
import sys
import tempfile
import tomllib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
import importlib.util  # noqa: E402
from collections import namedtuple  # noqa: E402

SECTION_HEADER = re.compile(r"^\s*Running (?:unittests )?(\S+?\.rs) \(([^)]*)")
BUILT_CRATE_NAME = re.compile(r"(?:^|/)([A-Za-z0-9_]+)-[0-9a-f]+(?:\.exe)?$")   # 段头括号里 deps 文件名：<crate 名>-<哈希>
DOCTEST_HEADER = re.compile(r"^\s*Doc-tests (\S+)")
OWN_FAILED_LINE = re.compile(r"^test (\S+) \.\.\. FAILED\s*$")   # 只给弄坏开关 own-failed-line 用：改前自己的那一份
BREAK_VARIABLE = "RERUN_FAILED_TESTS_BREAK"
SCOPE_VARIABLE = "SINGLEFS_RUN_SCOPE"
SUBMIT_WORDS = ["task-server/task", "submit", "--"]   # 打出的命令经提交客户端交 task-server（文件头）
RUN_SCOPE_GUARD = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "hooks", "run-scope-guard.sh")
_LIBTEST_SPECIFICATION = importlib.util.spec_from_file_location(
    "lib_libtest_result_lines_for_rerun_failed_tests", os.path.join(os.path.dirname(os.path.realpath(__file__)), "lib_libtest_result_lines.py"))
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_SPECIFICATION)
_LIBTEST_SPECIFICATION.loader.exec_module(libtest_result_lines)
DEFAULT_WORKSPACE_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
# 一段有失败的测试目标：挑目标的参数、失败的用例名、段头里的源文件、段头括号里的编出来的文件
FailedTarget = namedtuple("FailedTarget", ["selector", "names", "source", "artifact"])


def failed_name_of(line):
    """一行是 FAILED 的结果行就交用例全名（不带「 - should panic」），不是交 None。"""
    if os.environ.get(BREAK_VARIABLE) == "own-failed-line":
        found = OWN_FAILED_LINE.match(line)
        return found.group(1) if found else None
    parsed = libtest_result_lines.result_line(line)
    return parsed.full_name if parsed is not None and parsed.outcome == "FAILED" else None


def target_selector_of(source_path):
    """段头里的源文件 → cargo test 挑目标的参数；认不出交 None。"""
    if re.fullmatch(r"tests/[^/]+\.rs", source_path):
        return ["--test", os.path.basename(source_path)[:-len(".rs")]]
    if source_path == "src/lib.rs":
        return ["--lib"]
    if re.fullmatch(r"src/bin/[^/]+\.rs", source_path):
        return ["--bin", os.path.basename(source_path)[:-len(".rs")]]
    return None


def failed_tests_by_target(log_text):
    """(按段的 [(挑目标的参数, [失败的用例名])], [问题])。"""
    sections, problems = [], []
    current = None
    for line in log_text.split("\n"):
        header = SECTION_HEADER.match(line)
        doctest = DOCTEST_HEADER.match(line)
        running = re.match(r"^\s*Running\s+(.+)", line)
        if header or doctest or running:
            if os.environ.get(BREAK_VARIABLE) == "merge-sections" and current is not None:
                continue
            current = {"source": header.group(1) if header else f"doc-tests {doctest.group(1)}" if doctest else running.group(1),
                       "selector": target_selector_of(header.group(1)) if header else None,
                       "artifact": header.group(2).strip() if header else "",
                       "failed": [], "reported": None}
            sections.append(current)
            continue
        if current is None:
            if failed_name_of(line) is not None or libtest_result_lines.summary_line(line) is not None:
                problems.append("测试结果之前没有 Running / Doc-tests 段头，无法确认所属目标")
            continue
        failed = failed_name_of(line)
        if failed is not None:
            current["failed"].append(failed)
            continue
        result = libtest_result_lines.summary_line(line)
        if result:
            current["reported"] = (current["reported"] or 0) + result.failed
    commands = []
    for section in sections:
        if section["reported"] is None:
            problems.append(f"{section['source']}：没有 test result 行（这一段没跑完，或者日志被截断）")
            continue
        if len(section["failed"]) != section["reported"]:
            problems.append(f"{section['source']}：收到 {len(section['failed'])} 条 FAILED，test result 报 {section['reported']} 条")
            continue
        if not section["failed"]:
            continue
        if section["selector"] is None:
            problems.append(f"{section['source']}：有 {len(section['failed'])} 条失败，这一段按名字重跑不了")
            continue
        commands.append(FailedTarget(section["selector"], sorted(set(section["failed"])), section["source"], section["artifact"]))
    return commands, problems


class PackageNotFound(Exception):
    pass


def read_manifest(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def workspace_packages(workspace_root):
    """<根>/Cargo.toml 的 [package] 与 [workspace] members（展开通配、去掉 exclude）：{包名: (包目录, 它的 Cargo.toml)}，照清单的次序。读不了抛 PackageNotFound。"""
    root_manifest_path = os.path.join(workspace_root, "Cargo.toml")
    if not os.path.isfile(root_manifest_path):
        raise PackageNotFound(f"{workspace_root} 底下没有 Cargo.toml")
    try:
        root_manifest = read_manifest(root_manifest_path)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise PackageNotFound(f"读不了 {root_manifest_path}：{error}") from error
    packages = {}
    if root_manifest.get("package", {}).get("name"):
        packages[root_manifest["package"]["name"]] = (workspace_root, root_manifest)
    workspace = root_manifest.get("workspace", {})
    excluded = {os.path.normpath(os.path.join(workspace_root, pattern)) for pattern in workspace.get("exclude", [])}
    for pattern in workspace.get("members", []):
        for member in sorted(glob.glob(os.path.join(workspace_root, pattern))):
            member = os.path.normpath(member)
            manifest_path = os.path.join(member, "Cargo.toml")
            if member in excluded or not os.path.isfile(manifest_path):
                continue
            try:
                manifest = read_manifest(manifest_path)
            except (OSError, tomllib.TOMLDecodeError) as error:
                raise PackageNotFound(f"读不了 {manifest_path}：{error}") from error
            name = manifest.get("package", {}).get("name")
            if isinstance(name, str) and name not in packages:
                packages[name] = (member, manifest)
    return packages


def package_directory_of(workspace_root, package):
    """name 等于 package 的那个包（workspace_packages）：交 (包目录, 它的 Cargo.toml)。找不到抛 PackageNotFound。"""
    packages = workspace_packages(workspace_root)
    if package in packages:
        return packages[package]
    raise PackageNotFound(f"{os.path.join(workspace_root, 'Cargo.toml')} 的 [package] 与 [workspace] members 里没有叫 {package} 的包"
                          f"（成员里的包：{'、'.join(packages) or '一个都没有'}）")


DOC_TARGET = "doc"   # <包名>:doc 是那个包的文档测试（文件头「项的写法」）
# 一项：挑目标的参数、段头括号里编出来的文件名的 crate 名（文档测试那一项是库名）
TestTarget = namedtuple("TestTarget", ["selector", "crate_name"])


def item_name(package, target):
    return f"{package}:{target}"


def split_item(text):
    """「<包名>:<目标>」→ (包名, 目标)；没有冒号或有一边是空的交 None。"""
    package, separator, target = text.partition(":")
    return (package, target) if separator and package and target else None


def test_targets_of_package(package_directory, manifest, package):
    """cargo test 不带特性时这个包会跑的测试目标（文件头「项的写法」）：{目标根文件或 doc: TestTarget}，库在前、文档测试在最后。
    弄坏开关 bin-selector-from-file-name 下 [[bin]] / [[test]] 写了 path 的照文件名猜名字。"""
    found = {}
    package_section = manifest.get("package", {})

    def exists(relative):
        return os.path.isfile(os.path.join(package_directory, relative))

    def add(relative, selector, crate_name):
        if relative not in found and exists(relative):
            found[relative] = TestTarget(selector, crate_name)

    library = manifest.get("lib", {})
    library_root = library.get("path", "src/lib.rs")
    library_name = library.get("name") or package.replace("-", "_")
    if exists(library_root) and library.get("test", True):
        add(library_root, ["--lib"], library_name)
    guessed = os.environ.get(BREAK_VARIABLE) == "bin-selector-from-file-name"
    sections = (("bin", "--bin", "src/bin", package_section.get("autobins", True)), ("test", "--test", "tests", package_section.get("autotests", True)),
                ("example", "--example", "examples", False), ("bench", "--bench", "benches", False))
    for section, option, directory, automatic in sections:
        declared = set()
        for entry in manifest.get(section, []):
            name = entry.get("name")
            if not isinstance(name, str):
                continue
            default_root = "src/main.rs" if section == "bin" and name == package else f"{directory}/{name}.rs"
            relative = entry.get("path", default_root)
            declared.update((name, relative))
            wanted = entry.get("test", section in ("bin", "test")) and not entry.get("required-features")
            if wanted:
                chosen = os.path.splitext(os.path.basename(relative))[0] if guessed and relative != default_root else name
                add(relative, [option, chosen], chosen.replace("-", "_"))
        if not automatic:
            continue
        candidates = [("src/main.rs", package)] if section == "bin" else []
        absolute = os.path.join(package_directory, directory)
        for entry in sorted(os.listdir(absolute)) if os.path.isdir(absolute) else []:
            if entry.endswith(".rs") and os.path.isfile(os.path.join(absolute, entry)):
                candidates.append((f"{directory}/{entry}", entry[:-len(".rs")]))
            elif os.path.isfile(os.path.join(absolute, entry, "main.rs")):
                candidates.append((f"{directory}/{entry}/main.rs", entry))
        for relative, name in candidates:
            if relative not in declared and name not in declared:
                add(relative, [option, name], name.replace("-", "_"))
    if exists(library_root) and library.get("doctest", True):
        found[DOC_TARGET] = TestTarget(["--doc"], library_name)
    return found


def section_item(source, artifact, package_targets):
    """日志里一段属于哪一项（文件头「项的写法」）：source 是段头的源文件（文档测试那一段写 doc-tests <库名>），artifact 是段头括号里编出来的文件；
    package_targets 是 {包名: test_targets_of_package 的结果}。交项名；一个包都对不上交 None，几个包都对得上交空串。"""
    matched = []
    if source.startswith("doc-tests "):
        crate_name = source[len("doc-tests "):]
        matched = [package for package, targets in package_targets.items() if DOC_TARGET in targets and targets[DOC_TARGET].crate_name == crate_name]
        return item_name(matched[0], DOC_TARGET) if len(matched) == 1 else ("" if matched else None)
    built = BUILT_CRATE_NAME.search(artifact)
    for package, targets in package_targets.items():
        target = targets.get(source)
        if target is not None and source != DOC_TARGET and built is not None and built.group(1) == target.crate_name:
            matched.append(package)
    return item_name(matched[0], source) if len(matched) == 1 else ("" if matched else None)


def target_belongs_to_package(failed_target, package, package_directory, manifest):
    """这一段的测试目标属不属于这个包（文件头「按包筛目标」）。"""
    kind = failed_target.selector[0]
    if kind == "--test":
        name = failed_target.selector[1]
        return (os.path.isfile(os.path.join(package_directory, "tests", name + ".rs"))
                or any(entry.get("name") == name for entry in manifest.get("test", [])))
    if kind == "--bin":
        name = failed_target.selector[1]
        return (os.path.isfile(os.path.join(package_directory, "src", "bin", name + ".rs"))
                or any(entry.get("name") == name for entry in manifest.get("bin", [])))
    built = BUILT_CRATE_NAME.search(failed_target.artifact)
    library_name = manifest.get("lib", {}).get("name") or package.replace("-", "_")
    return built is not None and built.group(1) == library_name


def targets_of_package(commands, package, workspace_root):
    """(属于这个包的段, 属于别的包的段)。包找不到抛 PackageNotFound。弄坏开关 package-filter-dropped 下照改前全算属于。"""
    package_directory, manifest = package_directory_of(workspace_root, package)
    if os.environ.get(BREAK_VARIABLE) == "package-filter-dropped":
        return list(commands), []
    kept, elsewhere = [], []
    for failed_target in commands:
        (kept if target_belongs_to_package(failed_target, package, package_directory, manifest) else elsewhere).append(failed_target)
    return kept, elsewhere


def scope_declaration(selector, names, package, log_path):
    """一条重跑命令开头的范围声明（种类 target，范围是 --exact 点名的用例数）。"""
    log_name = re.sub(r"[;=]", "_", os.path.basename(log_path)) if log_path else "上一趟日志"
    value = (f"种类=target; 跑什么={package} {' '.join(selector)} 里上一趟红的用例; 范围={len(names)} 个测试; "
             f"为什么=只重跑 {log_name} 里红的这 {len(names)} 条用例，修完看它们转绿，不整份重跑")
    return f"{SCOPE_VARIABLE}={shlex.quote(value)}"


def cargo_command_line(selector, names, package):
    # `--` 之后只有 --exact 与点名的用例：标了 #[ignore] 的只剩要 NVIDIA 卡、量显存与量速度、要补丁表副本那几条，多带参数把它们起起来不对
    extra_tail = ["--show-output"] if os.environ.get(BREAK_VARIABLE) == "test-arguments-added" else []
    return shlex.join(["cargo", "test", "-p", package, *selector, "--", *extra_tail, "--exact", *names])


def submitted_command_line(selector, names, package):
    """经提交客户端交 task-server 的那一截；弄坏开关 submit-dropped 下照改前直接是 cargo test。"""
    if os.environ.get(BREAK_VARIABLE) == "submit-dropped":
        return cargo_command_line(selector, names, package)
    return f"{shlex.join(SUBMIT_WORDS)} {cargo_command_line(selector, names, package)}"


def command_lines(commands, package, log_path=None):
    if os.environ.get(BREAK_VARIABLE) == "scope-dropped":
        return [submitted_command_line(target.selector, target.names, package) for target in commands]
    return [f"{scope_declaration(target.selector, target.names, package, log_path)} {submitted_command_line(target.selector, target.names, package)}"
            for target in commands]


def without_declaration(line):
    """去掉开头的范围声明与提交客户端那一截，交 cargo 那一截（自证比命令用）。"""
    words = shlex.split(line)
    if words and words[0].startswith(SCOPE_VARIABLE + "="):
        words = words[1:]
    if words[:len(SUBMIT_WORDS)] == SUBMIT_WORDS:
        words = words[len(SUBMIT_WORDS):]
    return shlex.join(words)


def run_scope_guard_exit(command, detections):
    """把一条命令当主 agent 的 Bash（hook 输入不带 agent_type）喂给范围声明闸，交退出码。
    不当子 agent 喂：打出的命令经 task-server/task submit 交，子 agent 经 submit 交的闸一律拒（run-scope-guard.sh 文件头「经 task-server 提交」），由主 agent 起。"""
    import json
    import subprocess
    hook_input = {"tool_name": "Bash", "cwd": os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))),
                  "tool_input": {"command": command}}
    environment = {key: value for key, value in os.environ.items() if key not in (SCOPE_VARIABLE, "RUN_SCOPE_GUARD_BREAK")}
    environment["AGENT_HOOK_DETECTIONS"] = detections
    finished = subprocess.run(["bash", RUN_SCOPE_GUARD], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=60)
    return finished.returncode


def run(log_path, package, workspace_root):
    try:
        package_directory_of(workspace_root, package)
    except PackageNotFound as error:
        print(f"  ✗ 找不到包 {package}：{error}", file=sys.stderr)
        print("     → 怎么办：-p 写日志那一趟 cargo test 的包名；日志是别的 cargo 工作区跑出来的（实验工作区这类），--root 指到那个工作区的根", file=sys.stderr)
        return 2
    with open(log_path, encoding="utf-8", errors="replace") as handle:
        commands, problems = failed_tests_by_target(handle.read())
    if problems:
        print(f"  ✗ {log_path} 里有 {len(problems)} 段取不准失败的用例，一条命令都不打：", file=sys.stderr)  # gate-lint:summary
        for problem in problems:
            print(f"      {problem}", file=sys.stderr)  # gate-lint:detail
        print("     → 怎么办：日志不全就整份重跑一次再取；Doc-tests 有失败的照日志里的名字手跑；条数对不上多半是日志里混进了别的输出，换一份干净的日志。", file=sys.stderr)
        return 1
    commands, elsewhere = targets_of_package(commands, package, workspace_root)
    for target in elsewhere:
        print(f"  ! {target.source}（{target.artifact}）的 {len(target.names)} 条失败属于别的包，没打：照那个包另起一次 -p <它的包名>", file=sys.stderr)
    if not commands:
        print(f"  没有 {package} 的失败用例：不用重跑" + (f"（别的包的 {len(elsewhere)} 段见上）" if elsewhere else ""), file=sys.stderr)
        return 0
    for line in command_lines(commands, package, log_path):
        print(line)
    print(f"  ✓ {sum(len(target.names) for target in commands)} 条失败的用例，分在 {len(commands)} 个测试目标里；属于别的包、没打的 {len(elsewhere)} 段", file=sys.stderr)
    return 0


def resolve_items(items, workspace_root):
    """--item 点名的项认一遍（文件头「项的写法」）：交 ({包名: test_targets_of_package}（工作区里每个包）, [(项名, 包名, TestTarget)], [写错的那几句])。
    读不了工作区抛 PackageNotFound。弄坏开关 item-unknown-accepted 下包里没有的目标当没点。"""
    packages = workspace_packages(workspace_root)
    package_targets = {name: test_targets_of_package(directory, manifest, name) for name, (directory, manifest) in packages.items()}
    resolved, problems = [], []
    for text in items:
        parts = split_item(text)
        if parts is None:
            problems.append(f"「{text}」不是「<包名>:<目标根文件>」或「<包名>:doc」")
        elif parts[0] not in package_targets:
            problems.append(f"「{text}」的包 {parts[0]} 不在工作区里（工作区里的包：{'、'.join(package_targets) or '一个都没有'}）")
        elif parts[1] not in package_targets[parts[0]]:
            if os.environ.get(BREAK_VARIABLE) != "item-unknown-accepted":
                listed = "、".join(item_name(parts[0], target) for target in package_targets[parts[0]]) or "一项都没有"
                problems.append(f"「{text}」：包 {parts[0]} 没有这一项（它可点的项：{listed}）")
        elif text not in (name for name, _package, _target in resolved):
            resolved.append((text, parts[0], package_targets[parts[0]][parts[1]]))
    return package_targets, resolved, problems


def run_items(log_path, items, workspace_root):
    """--item 那一种跑法：只打点名的那几项里有失败的测试目标（文件头「项的写法」）。弄坏开关 item-filter-dropped 下只认项的包、那个包里有失败的段全打。"""
    try:
        package_targets, resolved, item_problems = resolve_items(items, workspace_root)
    except PackageNotFound as error:
        print(f"  ✗ 读不了工作区：{error}", file=sys.stderr)
        print("     → 怎么办：--root 指到日志那一趟的 cargo 工作区根（实验工作区的日志给 --root scripts/experiment-apparatus）", file=sys.stderr)
        return 2
    if item_problems:
        for problem in item_problems:
            print(f"  ✗ --item {problem}", file=sys.stderr)  # gate-lint:detail
        print("     → 怎么办：项照「<包名>:<目标根文件>」写（例 singlefs-verification:tests/model_differential.rs、singlefs-core:src/lib.rs、singlefs-core:doc），"
              "可点的项照上面列的写；bash scripts/gate/merge-and-verify.sh --list-items 列这一趟的全部项", file=sys.stderr)
        return 2
    with open(log_path, encoding="utf-8", errors="replace") as handle:
        commands, problems = failed_tests_by_target(handle.read())
    if problems:
        print(f"  ✗ {log_path} 里有 {len(problems)} 段取不准失败的用例，一条命令都不打：", file=sys.stderr)  # gate-lint:summary
        for problem in problems:
            print(f"      {problem}", file=sys.stderr)  # gate-lint:detail
        print("     → 怎么办：日志不全就整份重跑一次再取；Doc-tests 有失败的照日志里的名字手跑；条数对不上多半是日志里混进了别的输出，换一份干净的日志。", file=sys.stderr)
        return 1
    named = {name: (package, target) for name, package, target in resolved}
    named_packages = {package for package, _target in named.values()}
    printed, elsewhere = [], 0
    for failed_target in commands:
        owner = section_item(failed_target.source, failed_target.artifact, package_targets)
        if os.environ.get(BREAK_VARIABLE) == "item-filter-dropped" and owner and split_item(owner)[0] in named_packages:
            package, target = split_item(owner)
            printed.append((package, failed_target._replace(selector=package_targets[package][target].selector)))
            continue
        if owner not in named:
            elsewhere += 1
            print(f"  ! {failed_target.source}（{failed_target.artifact}）的 {len(failed_target.names)} 条失败不在点名的项里，没打"
                  f"{'（认不出属于哪一项）' if owner is None else '（几个包都对得上，拿不准）' if owner == '' else '：' + owner}", file=sys.stderr)
            continue
        package, target = named[owner]
        printed.append((package, failed_target._replace(selector=target.selector)))
    if not printed:
        print(f"  点名的 {len(named)} 项里没有失败的用例：不用重跑" + (f"（不在点名的项里的 {elsewhere} 段见上）" if elsewhere else ""), file=sys.stderr)
        return 0
    for package, failed_target in printed:
        print(command_lines([failed_target], package, log_path)[0])
    print(f"  ✓ 点名的 {len(named)} 项里 {sum(len(target.names) for _package, target in printed)} 条失败的用例，分在 {len(printed)} 个测试目标里；不在点名的项里、没打的 {elsewhere} 段",
          file=sys.stderr)
    return 0


def selftest():
    sample = "\n".join([
        "     Running tests/alpha.rs (target/debug/deps/alpha-0123456789abcdef)",
        "test alpha_one ... ok",
        "test alpha_two ... FAILED",
        "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
        "     Running tests/beta.rs (target/debug/deps/beta-0123456789abcdef)",
        "test beta_one ... ok",
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        "     Running unittests src/lib.rs (target/debug/deps/sample-0123456789abcdef)",
        "test module::inner_case ... FAILED",
        "test module::other_case ... FAILED",
        "test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out",
    ])
    failures = []
    commands, problems = failed_tests_by_target(sample)
    lines = command_lines(commands, "sample", "/tmp/harness;run=1.log")
    expected = ["cargo test -p sample --test alpha -- --exact alpha_two",
                "cargo test -p sample --lib -- --exact module::inner_case module::other_case"]
    if problems or [without_declaration(line) for line in lines] != expected:
        failures.append(f"按段取失败用例：期望 {expected}，实际 {lines}，问题 {problems}")
    submitted = [(lambda words: "cargo" in words and words[words.index("cargo") - len(SUBMIT_WORDS):words.index("cargo")] == SUBMIT_WORDS)(shlex.split(line))
                 for line in lines]
    if not lines or not all(submitted):
        failures.append(f"打出的每条命令经 task-server/task submit -- 起（cargo 前面紧接提交客户端）：实际 {submitted}，命令 {lines}")
    with tempfile.TemporaryDirectory() as directory:
        detections = os.path.join(directory, "detections.jsonl")
        declared = [line.startswith(SCOPE_VARIABLE + "=") for line in lines]
        verdicts = [run_scope_guard_exit(line, detections) for line in lines]
        if not lines or not all(declared) or verdicts != [0] * len(lines):
            failures.append(f"打出的每条命令开头带范围声明、范围声明闸照主 agent 判放行：开头带声明 {declared}，闸的退出码 {verdicts}，命令 {lines}")
        bare = [run_scope_guard_exit(command, detections) for command in expected]
        if bare != [2] * len(expected):
            failures.append(f"不带声明的同一条命令范围声明闸拒（退 2）：实际退出码 {bare}；闸不判这种命令时上一格证不出什么")
    truncated = sample.rsplit("\n", 1)[0]
    _commands, problems = failed_tests_by_target(truncated)
    if not any("没有 test result" in problem for problem in problems):
        failures.append(f"最后一段缺 test result 行应当判出问题，实际 {problems}")
    miscounted = sample.replace("test alpha_two ... FAILED\n", "")
    _commands, problems = failed_tests_by_target(miscounted)
    if not any("收到 0 条 FAILED" in problem for problem in problems):
        failures.append(f"FAILED 行与 test result 对不上应当判出问题，实际 {problems}")
    should_panic = "\n".join([
        "     Running unittests src/lib.rs (target/debug/deps/sample-0123456789abcdef)",
        "test tests::answer_is_42 ... ok",
        "test tests::answer_panics - should panic ... FAILED",
        "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
    ])
    commands, problems = failed_tests_by_target(should_panic)
    lines = [without_declaration(line) for line in command_lines(commands, "sample")]
    expected = ["cargo test -p sample --lib -- --exact tests::answer_panics"]
    if problems or lines != expected:
        failures.append(f"should panic 的失败用例按全名重跑：期望 {expected}，实际 {lines}，问题 {problems}")
    with tempfile.TemporaryDirectory() as directory:
        workspace_root = build_two_package_workspace(directory)
        empty_log = os.path.join(directory, "clean.log")
        with open(empty_log, "w", encoding="utf-8") as handle:
            handle.write("     Running tests/beta.rs (x)\ntest beta_one ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n")
        if run(empty_log, "rerun-target", workspace_root) != 0:
            failures.append("没有失败的用例应当退 0")
        failures.extend(package_filter_failures(directory, workspace_root))
        failures.extend(item_filter_failures(directory, workspace_root))
    if failures:
        for failure in failures:
            print(f"  ✗ rerun-failed-tests 自检：{failure}")  # gate-lint:detail
        print(f"     → 看 failed_tests_by_target() 的分段与条数核对、targets_of_package() 的按包筛；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print("  ✓ rerun-failed-tests 自检通过（13 格：按段分目标、打出的命令经 task-server/task submit 起、打出的命令带范围声明且闸放行、不带声明闸拒、缺 test result、条数对不上、"
          "should panic 的按全名重跑、没有失败、按包筛目标、找不到包退 2、按项筛目标、项写错退 2、[[bin]] 写了 path 的项按清单里的名字挑）")
    return 0


def build_two_package_workspace(directory):
    """自证用的假 cargo 工作区：rerun-target 有 tests/alpha.rs、src/lib.rs 与 src/bin/tool.rs，neighbour-crate 有 tests/beta.rs 与 src/lib.rs，
    bin-crate 只有 [[bin]] name = "tool-x" path = "src/bin/tool_x.rs" 那一个目标。交工作区根。"""
    workspace_root = os.path.join(directory, "workspace")
    files = {
        "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\nexclude = ["crates/excluded"]\n',
        "crates/rerun-target/Cargo.toml": '[package]\nname = "rerun-target"\nversion = "0.1.0"\n',
        "crates/rerun-target/tests/alpha.rs": "",
        "crates/rerun-target/src/lib.rs": "",
        "crates/rerun-target/src/bin/tool.rs": "",
        "crates/neighbour/Cargo.toml": '[package]\nname = "neighbour-crate"\nversion = "0.1.0"\n',
        "crates/neighbour/tests/beta.rs": "",
        "crates/neighbour/src/lib.rs": "",
        "crates/excluded/Cargo.toml": '[package]\nname = "excluded-crate"\nversion = "0.1.0"\n',
        "crates/bin-crate/Cargo.toml": '[package]\nname = "bin-crate"\nversion = "0.1.0"\n\n[[bin]]\nname = "tool-x"\npath = "src/bin/tool_x.rs"\n',
        "crates/bin-crate/src/bin/tool_x.rs": "",
    }
    for relative, text in files.items():
        path = os.path.join(workspace_root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    return workspace_root


def package_filter_failures(directory, workspace_root):
    """按包筛目标那两格：两个包各有失败的段（tests 目标与 lib 各一段），-p rerun-target 只打它自己的 alpha 与 lib 两条、stderr 点名 neighbour-crate 那两段；
    包名写错退 2、一条不打。"""
    import subprocess
    failures = []
    log = os.path.join(directory, "two-packages.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("\n".join([
            "     Running tests/alpha.rs (target/debug/deps/alpha-0123456789abcdef)",
            "test alpha_case ... FAILED",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            "     Running tests/beta.rs (target/debug/deps/beta-0123456789abcdef)",
            "test beta_case ... FAILED",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            "     Running unittests src/lib.rs (target/debug/deps/rerun_target-0123456789abcdef)",
            "test inner::own_case ... FAILED",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            "     Running unittests src/lib.rs (target/debug/deps/neighbour_crate-0123456789abcdef)",
            "test inner::neighbour_case ... FAILED",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            "",
        ]))
    script = os.path.realpath(__file__)
    finished = subprocess.run([sys.executable, script, log, "-p", "rerun-target", "--root", workspace_root], capture_output=True, text=True, timeout=120)
    printed = [without_declaration(line) for line in finished.stdout.splitlines() if line.strip()]
    expected = ["cargo test -p rerun-target --test alpha -- --exact alpha_case",
                "cargo test -p rerun-target --lib -- --exact inner::own_case"]
    named_elsewhere = [line for line in finished.stderr.splitlines() if "属于别的包，没打" in line]
    if (finished.returncode != 0 or printed != expected or len(named_elsewhere) != 2
            or not any("tests/beta.rs" in line for line in named_elsewhere)
            or not any("neighbour_crate-" in line for line in named_elsewhere)):
        failures.append(f"按包筛目标：两个包各有失败的段，-p 那个包只打它自己的两条、stderr 点名别的包那两段：退 {finished.returncode}，"
                        f"期望 {expected}，实际 {printed}，点名别的包的行 {named_elsewhere}，stderr「{finished.stderr.strip()[-600:]}」")
    finished = subprocess.run([sys.executable, script, log, "-p", "excluded-crate", "--root", workspace_root], capture_output=True, text=True, timeout=120)
    if finished.returncode != 2 or finished.stdout.strip() or "找不到包 excluded-crate" not in finished.stderr:
        failures.append(f"找不到包退 2、一条不打（workspace exclude 掉的包也算找不到）：退 {finished.returncode}，stdout「{finished.stdout.strip()}」，"
                        f"stderr「{finished.stderr.strip()[-400:]}」")
    return failures


def item_filter_failures(directory, workspace_root):
    """按项那三格（文件头「项的写法」）：两个包各有失败的段，--item rerun-target:tests/alpha.rs 只打 alpha 那一条、stderr 点名别的三段；
    点了包里没有的目标退 2、一条不打、列出那个包可点的项；bin-crate 的 [[bin]] 写了 path，点 bin-crate:src/bin/tool_x.rs 打的是 --bin tool-x。"""
    import subprocess
    failures = []
    script = os.path.realpath(__file__)
    log = os.path.join(directory, "two-packages.log")
    finished = subprocess.run([sys.executable, script, log, "--item", "rerun-target:tests/alpha.rs", "--root", workspace_root], capture_output=True, text=True, timeout=120)
    printed = [without_declaration(line) for line in finished.stdout.splitlines() if line.strip()]
    expected = ["cargo test -p rerun-target --test alpha -- --exact alpha_case"]
    named_elsewhere = [line for line in finished.stderr.splitlines() if "不在点名的项里，没打" in line]
    if finished.returncode != 0 or printed != expected or len(named_elsewhere) != 3:
        failures.append(f"按项筛目标：点名 rerun-target 的 tests/alpha.rs，只打它那一条、stderr 点名别的三段：退 {finished.returncode}，期望 {expected}，实际 {printed}，"
                        f"点名别的段的行 {named_elsewhere}，stderr「{finished.stderr.strip()[-600:]}」")
    finished = subprocess.run([sys.executable, script, log, "--item", "rerun-target:tests/missing.rs", "--root", workspace_root], capture_output=True, text=True, timeout=120)
    if (finished.returncode != 2 or finished.stdout.strip() or "rerun-target:tests/alpha.rs" not in finished.stderr
            or "rerun-target:src/bin/tool.rs" not in finished.stderr):
        failures.append(f"项写错退 2：rerun-target 没有 tests/missing.rs，列出它可点的项：退 {finished.returncode}，stdout「{finished.stdout.strip()}」，"
                        f"stderr「{finished.stderr.strip()[-600:]}」")
    bin_log = os.path.join(directory, "bin-crate.log")
    with open(bin_log, "w", encoding="utf-8") as handle:
        handle.write("\n".join([
            "     Running unittests src/bin/tool_x.rs (target/debug/deps/tool_x-0123456789abcdef)",
            "test checks::tool_case ... FAILED",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            "",
        ]))
    finished = subprocess.run([sys.executable, script, bin_log, "--item", "bin-crate:src/bin/tool_x.rs", "--root", workspace_root], capture_output=True, text=True, timeout=120)
    printed = [without_declaration(line) for line in finished.stdout.splitlines() if line.strip()]
    expected = ["cargo test -p bin-crate --bin tool-x -- --exact checks::tool_case"]
    if finished.returncode != 0 or printed != expected:
        failures.append(f"[[bin]] 写了 path 的项按清单里的名字挑：退 {finished.returncode}，期望 {expected}，实际 {printed}，stderr「{finished.stderr.strip()[-600:]}」")
    return failures


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    workspace_root, packages, items, positional, broken_usage = DEFAULT_WORKSPACE_ROOT, [], [], [], False
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("-p", "--item", "--root"):
            if position + 1 >= len(arguments) or not arguments[position + 1]:
                broken_usage = True
                break
            value = arguments[position + 1]
            if argument == "-p":
                packages.append(value)
            elif argument == "--item":
                items.append(value)
            else:
                workspace_root = os.path.abspath(value)
            position += 2
            continue
        if argument.startswith("--item="):
            items.append(argument[len("--item="):])
        elif argument.startswith("-"):
            broken_usage = True
        else:
            positional.append(argument)
        position += 1
    if broken_usage or len(positional) != 1 or (len(packages) != 1) == (not items) or (packages and items):
        print("  ✗ 用法：rerun-failed-tests.py <cargo test 的日志> -p <包名> [--root <cargo 工作区根>]，"
              "或 rerun-failed-tests.py <cargo test 的日志> --item <包名>:<目标根文件> [--item …] [--root <cargo 工作区根>]（-p 与 --item 只给一种，-p 只给一次）", file=sys.stderr)
        print("     → 例：rerun-failed-tests.py /tmp/harness.log -p singlefs-verification；rerun-failed-tests.py /tmp/test.log --item singlefs-core:tests/alloc.rs；"
              "实验工作区的日志另给 --root scripts/experiment-apparatus；自检：rerun-failed-tests.py --selftest", file=sys.stderr)
        sys.exit(2)
    if items:
        sys.exit(run_items(positional[0], items, workspace_root))
    sys.exit(run(positional[0], packages[0], workspace_root))


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
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/rerun-failed-tests.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    main()
