"""形式化证明判法自证（入口 verus-proof-check.py 的 --selftest）的一块：假 Verus、假 cargo 与样本核；write-fake-tools 子命令也用它。
"""

import contextlib
import io
import json
import os
import lib_verdict_library  # noqa: E402  library_root（库根）、locked_dependency_lines（闭包的依赖版本行算不出时退回整个 crate 的）

from verus_proof_check_base import (
     REGISTRY, EXIT_GREEN, EXIT_USAGE, BREAK, report_red,
)
from verus_proof_check_listing import FAKE_TOOLCHAIN

def write_fake_tools(directory, root):
    """write-fake-tools <目录> <根>：往 <目录> 写一份假的 Verus 安装（verus、cargo、cargo-verus 三个可执行文件，加 version.json）；
    version.json 的 version 与 commit 取 <根> 登记表 release 行的第 2、3 段（样本照真仓登记，装的 Verus 与登记表同版本，工具链那一格才对得上）。
    三个假工具就是自证（--selftest）用的 FAKE_VERUS、FAKE_CARGO、FAKE_CARGO_VERUS，按源码里的标记出判定、不起真 Verus 与真 cargo；
    门禁 formal-verification 的样本档在样本根里写它、用 --verus-root 指到它、PATH 前面放它（文件头「门禁样本」）。
    弄坏开关 fake-tools-commit-unpinned 下 version.json 的 commit 写成全 0（工具链那一格与 Verus 报的 commit 对不上）。"""
    try:
        with open(os.path.join(root, REGISTRY), encoding="utf-8") as handle:
            rows = [line.split("\t") for line in handle.read().split("\n") if line.startswith("release\t")]
    except OSError as error:
        report_red(f"读不了登记表 {REGISTRY}：{error}", "第二个参数给带着登记表的仓根（样本根）")
        return EXIT_USAGE
    if not rows or len(rows[0]) < 3 or not rows[0][2]:
        report_red(f"{REGISTRY} 里没有 release 行（或它不到三段）：假安装的 version.json 没有版本与 commit 可抄", "登记表照真仓写 release 行")
        return EXIT_USAGE
    os.makedirs(directory, exist_ok=True)
    for tool, text in (("verus", FAKE_VERUS), ("cargo", FAKE_CARGO), ("cargo-verus", FAKE_CARGO_VERUS)):
        path = os.path.join(directory, tool)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.chmod(path, 0o755)
    commit = "0" * 40 if "fake-tools-commit-unpinned" in BREAK else rows[0][2]
    with open(os.path.join(directory, "version.json"), "w", encoding="utf-8") as handle:
        json.dump({"verus": {"version": rows[0][1], "commit": commit, "toolchain": FAKE_TOOLCHAIN}}, handle)
    print(f"  · 假 Verus 安装写在 {directory}（verus、cargo、cargo-verus、version.json；版本 {rows[0][1]}，commit {commit}）")  # gate-lint:detail
    return EXIT_GREEN


# ── 自证 ──

FAKE_VERUS = r'''#!/usr/bin/env python3
# 假的 Verus：按源码里的标记出判定。只给 verus-proof-check.py --selftest 用。
# 输出照真 Verus 0.2026.09.20 的形态（主 agent 2026-10-15 的探针）：整 crate 验时 verification-results 有 success、is-verifying-entire-crate true；
# --verify-function 下没有 success 键、is-verifying-entire-crate false；vir 错只有 encountered-error 与 encountered-vir-error 两样。
# 义务条数照真 Verus 量到的：proof fn 1 条，exec fn 1 条另加每个 while 1 条，spec fn 带 decreases 的 1 条、不带的 0 条，
# derive(Clone) 的类型另有一个 <类型>::clone（1 条）；函数里有 SELFTEST_ZERO_VERIFIED 的那个函数验出 0 条。
# impl 与 trait 里的方法照真 Verus 列可点名函数的写法记成 <类型>::<方法>（trait impl 也写类型）、<trait>::<方法>。
# --verify-function 照真 Verus 点：有恰好同名的取它，否则唯一包含它的那一个，几个都包含它报 more than one match，一个都没有报 could not find；
# 两头带 * 的照真 Verus 认通配（*<串> 以它收尾、<串>* 以它起头、*<串>* 包含它），点中几个一起验几个（2026-10-29 探针现量）。
# 模块也照真 Verus 0.2026.09.20（2026-10-18 的三层小 crate 探针）：--verify-function 只在 --verify-root（src/lib.rs）或 --verify-only-module <模块>
# （src/<甲>.rs、src/<甲>/mod.rs 是 <甲>，src/<甲>/<乙>.rs 是 <甲>::<乙>，不含子模块）点的那一个模块里找；两样都没给、两样都给、给的是 --verify-module，
# 退 255、stdout 空；点的模块没有报 could not find module。不同模块里同名的函数各是各的。环境里有 SELFTEST_VERUS_ARGUMENTS 的，每起一次往那份文件追加一行命令行参数（JSON；
# --compile 那一趟是 MIR 比对，不记）。
# MIR：带 --compile 与 -o <文件> 的那一趟验完整个 crate 之后往 -o 写一份假 MIR（verus 那一种）；--selftest-mir <product|verus> <crate 目录> <文件> 只写 MIR、不验
# （假 cargo 与假 cargo-verus 用）。假 MIR 每个 fn 一段，正文是那个 fn 的源码行；产品那一种去掉带 SELFTEST_MIR_VERUS_ONLY 的行，verus 那一种去掉带
# SELFTEST_MIR_PRODUCT_ONLY 的行、另多一段擦出来的空壳 selftest_ghost_shell；带 SELFTEST_MIR_RT_PATH 的行产品那一种把 core::panicking:: 印成 std::rt::。
import json, os, re, sys, time
here = os.path.dirname(os.path.realpath(__file__))
commit = json.load(open(os.path.join(here, "version.json")))["verus"]["commit"]
function_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:open\s+|closed\s+)?(?:broadcast\s+)?(?:uninterp\s+)?(?:axiom\s+)?(?P<mode>proof\s+|spec\s+|exec\s+)?(?:const\s+)?fn\s+(?P<name>\w+)")
# 带值的 const 与 assume_specification（照真 Verus 0.2026.09.20，2026-10-29 在 core 与探针 crate 上量的）：带值的 const 1 条（spec const 也是 1 条），
# 自由 const 列成它的名字、trait 里的列成 <trait>::<名>、impl 里的列成 impl&%<号>::<名>；assume_specification 指向常量（] 后面不是参数表）且写了 returns 的 1 条、别的 0 条，
# 点名的条目只有标识符与 :: 的照写，别的列成 core::impl&%0::<最后一个标识符>
const_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?P<mode>spec\s+)?const\s+(?P<name>\w+)\s*:[^=]*=")
assume_line = re.compile(r"^\s*(?:pub\s+)?assume_specification(?:<[^\[]*>)?\s*\[")
type_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(?P<name>\w+)")
impl_line = re.compile(r"^(?P<indent>\s*)impl\b(?:\s*<[^>]*>)?\s+(?:(?P<trait>[\w:]+(?:\s*<[^>]*>)?)\s+for\s+)?(?P<target>[\w:]+)")
trait_line = re.compile(r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?trait\s+(?P<name>\w+)")


def module_of(path, source="src"):
    parts = os.path.relpath(path, source)[:-len(".rs")].split(os.sep)
    parts = parts[:-1] if parts[-1] == "mod" else parts
    return "" if parts == ["lib"] else "::".join(parts)


def source_files(source="src"):
    files = []
    for directory, _s, names in os.walk(source):
        for name in sorted(names):
            if name.endswith(".rs"):
                files.append(os.path.join(directory, name))
    return sorted(files)


def blocks_of(path):
    """[(点名的名字, 模式, [源码行])]：一个 fn 从它那一行到下一个 fn 那一行之前；impl / trait 里的照 <类型或 trait>::<方法> 记。"""
    blocks, container, external = [], None, False
    for line in open(path).read().split("\n"):
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            name = (opened.groupdict().get("target") or opened.groupdict().get("name")).split("::")[-1]
            container = (name, len(opened.group("indent")))
        heading = function_line.match(line)
        if heading:
            name = heading.group("name") if container is None else f"{container[0]}::{heading.group('name')}"
            # 带 external_body 的 fn 函数体不验（真 Verus 照它的规格当真、不出义务）：模式记成 trusted，义务 0 条
            blocks.append([name, "trusted" if external else (heading.group("mode") or "exec ").strip(), [line]])
            external = False
        elif blocks:
            blocks[-1][2].append(line)
        if "external_body" in line.split("//")[0]:
            external = True
    return blocks


def mir_text(flavor, source="src"):
    skip = "SELFTEST_MIR_VERUS_ONLY" if flavor == "product" else "SELFTEST_MIR_PRODUCT_ONLY"
    pieces = []
    for path in source_files(source):
        module = module_of(path, source)
        for name, _mode, lines in blocks_of(path):
            body = [line.strip() for line in lines if line.strip() and skip not in line]
            if flavor == "product":
                # 带 SELFTEST_MIR_RT_PATH 的行照真 rustc 的产品构建把 core::panicking:: 印成 std 的再导出 std::rt::（Verus 构建照 core 的路径印）
                body = [line.replace("core::panicking::", "std::rt::") if "SELFTEST_MIR_RT_PATH" in line else line for line in body]
            pieces.append(f"fn {module + '::' if module else ''}{name}() -> () {{\n" + "".join(f"    {line}\n" for line in body) + "}\n")
    if flavor == "verus":
        pieces.append("fn selftest_ghost_shell() -> () {\n}\n")
    return "\n".join(pieces)


if sys.argv[1:2] == ["--selftest-mir"]:
    flavor, crate_directory, out = sys.argv[2:5]
    open(out, "w").write(mir_text(flavor, os.path.join(crate_directory, "src")))
    sys.exit(0)
compile_output = sys.argv[sys.argv.index("-o") + 1] if "--compile" in sys.argv and "-o" in sys.argv[:-1] else None
if os.environ.get("SELFTEST_VERUS_ARGUMENTS") and compile_output is None:
    open(os.environ["SELFTEST_VERUS_ARGUMENTS"], "a").write(json.dumps(sys.argv[1:]) + "\n")
no_cheating = "--no-cheating" in sys.argv
rlimit = float(sys.argv[sys.argv.index("--rlimit") + 1]) if "--rlimit" in sys.argv[:-1] else None
only = sys.argv[sys.argv.index("--verify-function") + 1] if "--verify-function" in sys.argv[:-1] else None
only_module = sys.argv[sys.argv.index("--verify-only-module") + 1] if "--verify-only-module" in sys.argv[:-1] else None
if only is not None:
    refusal = ("When using --verify-function, use --verify-only-module instead of --verify-module" if "--verify-module" in sys.argv
               else "--verify-function option requires --verify-only-module or --verify-root" if "--verify-root" not in sys.argv and only_module is None
               else "Must pass at most one --verify-only-module or --verify-root when using --verify-function" if "--verify-root" in sys.argv and only_module is not None
               else None)
    if refusal:
        sys.stderr.write(f"Error: {refusal}\n")
        sys.exit(255)
target_module = None if only is None else only_module or ""
if os.environ.get("SELFTEST_VERUS_TRACE"):
    open(os.environ["SELFTEST_VERUS_TRACE"], "a").write("起了一次\n")
files = source_files()
# 键是「模块\0点名的名字」：不同模块里同名的各是各的
obligations, order, home = {}, [], {}
for path in files:
    module, derive_clone = module_of(path), False
    for name, mode, lines in blocks_of(path):
        key = f"{module}\0{name}"
        obligations[key] = 0 if mode in ("spec", "trusted") else 1
        order.append(key)
        home[key] = module
        for line in lines[1:]:
            if mode == "spec" and re.search(r"\bdecreases\b", line) and obligations[key] == 0:
                obligations[key] = 1
            if mode == "exec" and re.match(r"^\s*while\b", line):
                obligations[key] += 1
        for line in lines:
            if "SELFTEST_ZERO_VERIFIED" in line:
                obligations[key] = -10 ** 6
    for line in open(path).read().split("\n"):
        declared = type_line.match(line)
        if declared and derive_clone:
            key = f"{module}\0{declared.group('name')}::clone"
            obligations[key] = 1
            order.append(key)
            home[key] = module
        derive_clone = line.lstrip().startswith("#[derive(") and "Clone" in line
    container, impls = None, 0
    for line in open(path).read().split("\n"):
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            impls += 1
            container = (impls, len(opened.group("indent")), opened.groupdict().get("name"))
        constant = const_line.match(line)
        named = None
        if constant:
            owner = None if container is None else container[2] or f"impl&%{container[0]}"
            named = (constant.group("name") if owner is None else f"{owner}::{constant.group('name')}", 1)
        specification = assume_line.match(line)
        if specification:
            start = specification.end() - 1
            depth, end = 0, None
            for position in range(start, len(line)):
                depth += {"[": 1, "]": -1}.get(line[position], 0)
                if depth == 0:
                    end = position
                    break
            point = line[start + 1:end].strip() if end is not None else ""
            rest = line[end + 1:].lstrip() if end is not None else ""
            plain = re.fullmatch(r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*", point)
            last = (re.findall(r"[A-Za-z_]\w*", point) or ["?"])[-1]
            named = (point if plain else f"core::impl&%0::{last}", 1 if rest.startswith("->") and "returns" in rest else 0)
        if named:
            key = f"{module}\0{named[0]}"
            obligations[key] = named[1]
            order.append(key)
            home[key] = module
selected = None
if target_module and target_module not in {module_of(path) for path in files}:
    sys.stderr.write(f"error: could not find module {target_module} specified by --verify-module or --verify-only-module\n"
                     "error: aborting due to 1 previous error\n\n")
    print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True}, "verus": {"commit": commit}}))
    sys.exit(1)
if only is not None:
    order = [key for key in order if home[key] == target_module]
    names = {key: key.split("\0", 1)[1] for key in order}
    exact = [key for key in order if names[key] == only]
    if len(only) > 1 and only.startswith("*") and only.endswith("*"):
        selected = [key for key in order if only[1:-1] in names[key]]
    elif only.startswith("*"):
        selected = [key for key in order if names[key].endswith(only[1:])]
    elif only.endswith("*"):
        selected = [key for key in order if names[key].startswith(only[:-1])]
    elif exact:
        selected = exact[:1]
    else:
        matched = [key for key in order if only in names[key]]
        if len(matched) > 1:
            sys.stderr.write(f"error: more than one match found for --verify-function {only}, consider using wildcard *{only}* to verify all matched results,\n"
                             "       or specify a unique substring for the desired function, matched results are:\n"
                             + "".join(f"         - {names[key]}\n" for key in matched) + "\nerror: aborting due to 1 previous error\n\n")
            print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True}, "verus": {"commit": commit}}))
            sys.exit(1)
        selected = matched[:1]
selected_names = {key.split("\0", 1)[1] for key in selected or ()}
functions, errors, vir, found = 0, [], [], bool(selected)
for key in order:
    if only is None or key in selected:
        functions += max(obligations[key], 0)
for path in files:
    lines = open(path).read().split("\n")
    owner_vacuous = owner_failed = closure_vacuous = False
    current = None
    in_scope = only is None or module_of(path) == target_module
    container = None
    for number, line in enumerate(lines, 1):
        if "SELFTEST_KILLED" in line or (os.environ.get("SELFTEST_KILL_MARK") and os.environ["SELFTEST_KILL_MARK"] in line):
            os.kill(os.getpid(), 9)
        if no_cheating and re.search(r"\bassume\(", line.split("//")[0]):
            vir.append(("assume/admit not allowed with --no-cheating", path, number, 1))
        if no_cheating and re.search(r"\b(assume_specification|external_body)\b", line.split("//")[0]):
            vir.append(("external_body/assume_specification not allowed with --no-cheating", path, number, 1))
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            container = ((opened.groupdict().get("target") or opened.groupdict().get("name")).split("::")[-1], len(opened.group("indent")))
        heading = function_line.match(line)
        if heading:
            current = heading.group("name") if container is None else f"{container[0]}::{heading.group('name')}"
        if only is not None and (current not in selected_names or not in_scope):
            continue
        sleeping = re.search(r"SELFTEST_SLEEP_(\d+)", line)
        if sleeping:
            time.sleep(int(sleeping.group(1)) / 10)   # SELFTEST_SLEEP_<n>：验到这一行睡 n/10 秒（逐项秒数各算各的那一格用）
        if heading:
            owner_vacuous = "SELFTEST_VACUOUS" in line
            owner_failed = False
        if "SELFTEST_CLOSURE_VACUOUS" in line:
            closure_vacuous = True
        if "assert(false);" in line:
            if closure_vacuous:
                closure_vacuous = False
            elif not owner_vacuous and not owner_failed:
                errors.append(("assertion failed", path, number, line.index("assert(false)") + 1))
                owner_failed = True
        if "SELFTEST_BROKEN" in line or (os.environ.get("SELFTEST_FAIL_MARK") and os.environ["SELFTEST_FAIL_MARK"] in line):
            errors.append(("postcondition not satisfied", path, number, 1))   # SELFTEST_FAIL_MARK：环境里点名的字（不进键）也当证明错，记红撤绿那一格用
        needs = re.search(r"SELFTEST_NEEDS_RLIMIT_(\d+)", line)
        if needs and rlimit is not None and rlimit < int(needs.group(1)):
            errors.append(("while loop: Resource limit (rlimit) exceeded; consider rerunning with --profile for more details", path, number, 1))
if only is not None and not found:
    vir.append((f"could not find function {only} specified by --verify-function", "src/lib.rs", 1, 1))
if rlimit is None:
    vir.append(("SELFTEST: --rlimit not on the command line", "src/lib.rs", 1, 1))
for message, path, number, column in errors + vir:
    sys.stderr.write(f"error: {message}\n   --> {path}:{number}:{column}\n    |\n\n")
if vir:
    vir_results = {"encountered-error": True, "encountered-vir-error": True}
    if only is None:
        vir_results["success"] = False
    print(json.dumps({"verification-results": vir_results, "verus": {"commit": commit}}))
    sys.exit(1)
failed = len({(p, n) for _m, p, n, _c in errors})
results = {"encountered-error": bool(errors), "encountered-vir-error": False}
if only is None:
    results["success"] = not errors
results.update({"verified": max(functions - failed, 0), "errors": failed, "is-verifying-entire-crate": only is None})
if any("SELFTEST_NO_COUNTS" in open(path).read() for path in files):
    del results["verified"], results["errors"]
print(json.dumps({"verification-results": results, "verus": {"commit": commit}}))
if compile_output is not None and not errors:
    open(compile_output, "w").write(mir_text("verus"))
sys.exit(1 if errors else 0)
'''

# 假的 cargo（产品构建那一趟，MIR 比对用）与假的 cargo-verus（cargo 那一种 crate 起的 cargo verus focus / build）：只给 --selftest 用。
# 两份都照 -p 点的包在副本里找成员（当前目录的 Cargo.toml 是那个包，或工作区 members 里名字对得上的那一个）；
# 假 cargo 只认 rustc：往 $CARGO_TARGET_DIR/debug/deps/<包名>-0.mir 写产品那一种假 MIR。
# 假 cargo-verus 认 verus focus 与 verus build：focus 在成员目录里起假 Verus（-- 之后的参数原样交给它），报错位置照真 cargo verus 改写成相对副本根的路径；
# build 照样验整个 crate，再往 $CARGO_TARGET_DIR/debug/deps/<包名>-1.mir 写 verus 那一种假 MIR。环境里有 SELFTEST_CARGO_VERUS_ARGUMENTS 的，每起一次追加一行参数（JSON）。
# focus 照真 cargo 的编译指纹：同一个编译目录里 <编译目录>/verus-partial/debug/.fingerprint/<包名>-fake 在就只打一行 Finished、不出判定行，不在就建它再验。
FAKE_CARGO_COMMON = r'''#!/usr/bin/env python3
import json, os, subprocess, sys, tomllib
here = os.path.dirname(os.path.realpath(__file__))
verus = os.path.join(os.environ.get("SELFTEST_FAKE_VERUS_DIRECTORY") or here, "verus")


def member_of(package):
    own = os.path.join(os.getcwd(), "Cargo.toml")
    manifest = tomllib.load(open(own, "rb"))
    if manifest.get("package", {}).get("name") == package:
        return os.getcwd()
    for member in manifest.get("workspace", {}).get("members", []):
        path = os.path.join(os.getcwd(), member)
        if tomllib.load(open(os.path.join(path, "Cargo.toml"), "rb")).get("package", {}).get("name") == package:
            return path
    sys.stderr.write(f"error: package `{package}` not found\n")
    sys.exit(101)


arguments = sys.argv[1:]
package = arguments[arguments.index("-p") + 1]
member = member_of(package)
deps = os.path.join(os.environ.get("CARGO_TARGET_DIR") or os.path.join(os.getcwd(), "target"), "debug", "deps")
'''

FAKE_CARGO = FAKE_CARGO_COMMON + r'''
if arguments[:1] != ["rustc"]:
    sys.stderr.write(f"假 cargo 只认 rustc：{arguments}\n")
    sys.exit(101)
os.makedirs(deps, exist_ok=True)
sys.exit(subprocess.run([verus, "--selftest-mir", "product", member, os.path.join(deps, package.replace("-", "_") + "-0.mir")]).returncode)
'''

FAKE_CARGO_VERUS = FAKE_CARGO_COMMON + r'''
if os.environ.get("SELFTEST_CARGO_VERUS_ARGUMENTS"):
    open(os.environ["SELFTEST_CARGO_VERUS_ARGUMENTS"], "a").write(json.dumps(arguments) + "\n")
subcommand = arguments[1]
rest = arguments[arguments.index("--") + 1:]
# cargo 的编译指纹：同一个编译目录里被验的包验过一回就当没变，focus 只打一行 Finished、不起 Verus、stdout 没有判定行（真 cargo verus 的样子）
fingerprint = os.path.join(os.environ.get("CARGO_TARGET_DIR") or os.path.join(os.getcwd(), "target"), "verus-partial", "debug", ".fingerprint", package + "-fake")
if subcommand == "focus":
    if os.path.isdir(fingerprint):
        sys.stderr.write("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s\n")
        sys.exit(0)
    os.makedirs(fingerprint)
mir = "--emit=mir" in rest
if mir:
    rest = rest[:rest.index("--")] if "--" in rest else [word for word in rest if word != "--emit=mir"]
relative = os.path.relpath(member, os.getcwd())
ran = subprocess.run([verus, *rest, "src/lib.rs"], cwd=member, capture_output=True, text=True)
sys.stdout.write(ran.stdout)
sys.stderr.write(ran.stderr.replace("--> src/", f"--> {relative}/src/"))
if ran.returncode:
    # 真 cargo 编不过时自己另打一句（没有 --> 位置），不是证明错（判法的 proof_error_messages 去掉它）
    previous = sum(1 for line in ran.stderr.split("\n") if line.startswith("error") and "aborting due to" not in line)
    sys.stderr.write(f"error: could not compile `{package}` (lib) due to {previous} previous error{'' if previous == 1 else 's'}\n")
if subcommand == "build" and mir and ran.returncode == 0:
    os.makedirs(deps, exist_ok=True)
    subprocess.run([verus, "--selftest-mir", "verus", member, os.path.join(deps, package.replace("-", "_") + "-1.mir")], check=True)
sys.exit(101 if ran.returncode else 0)
'''

# 被验区域那几格的自证用：插在 SELFTEST_KERNEL 的「} // verus!」之前（methods：impl 与 trait 里的方法成项；trust：要登记的三种可信声明；
# trait：被验的 trait，另在 verus! 外面接一个 trait impl 与一个 cfg(test) 下的 impl）
SELFTEST_METHODS_EXTRA = """
pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures
            r <= 10;
}

pub struct Region {
    pub start: u64,
}

impl Region {
    pub fn contains(&self, slot: u64) -> (r: bool)
        ensures
            r == (self.start <= slot),
    {
        self.start <= slot
    }
}

impl Device for Region {
    fn read_slot(&self, slot: u64) -> (r: u64)
    {
        3
    }
}
"""

SELFTEST_TRUST_EXTRA = """
pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len();

#[verifier::external_body]
pub fn read_device(x: u64) -> (r: u64)
    ensures
        r == x,
{
    x
}

pub uninterp spec fn crc(x: u64) -> u64;
"""

# 带值的 const 与 assume_specification 成项那一格用（proof-valued-const-and-assume-specification-items）：impl 里的关联 const、自由 const、
# 指向常量写 returns 的 assume_specification（假 Verus 验出 1 条）、指向 std 方法的 assume_specification（0 条）
SELFTEST_CONST_EXTRA = """
pub struct Width(pub u64);

impl Width {
    pub const NARROWEST: Self = Self(1);
}

pub const WIDEST_SAMPLE: u64 = 64;

pub assume_specification[ selftest_format::SLOT_WIDTH ] -> (width: u64) returns 8u64;

pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len();
"""

# 一个 trait impl 里的几个方法闭包相同那两格用（reuse-proof-entry-unit-own-key、reuse-vacuity-owner-unit-own-key；core 的 impl BlockDevice for FileBackedBlockDevice 的样子）：
# 方法连它的 impl 头、impl 头点名类型、类型照 R-隐式收它的 trait impl 全部成员，两个方法收到的单元一模一样；fill 带一个 while（假 Verus 验出 2 条）、width 1 条
SELFTEST_IMPL_SIBLINGS_EXTRA = """
pub trait SampleStore {
    fn fill(&self, count: u64) -> (filled: u64)
        ensures
            filled == count;

    fn width(&self) -> (width: u64)
        ensures
            width <= 64;
}

pub struct SampleDevice {
    pub width: u64,
}

impl SampleStore for SampleDevice {
    fn fill(&self, count: u64) -> (filled: u64) {
        let mut filled: u64 = 0;
        while filled < count
            invariant
                filled <= count,
            decreases count - filled,
        {
            filled = filled + 1;
        }
        filled
    }

    fn width(&self) -> (width: u64) {
        if self.width > 64 { 64 } else { self.width }
    }
}
"""

# 同一个 trait impl 里两个方法各一处 requires，头一个的前置条件空真（假 Verus 认 SELFTEST_VACUOUS：插的 assert(false) 照样验过）
SELFTEST_IMPL_SIBLINGS_VACUITY_EXTRA = """
pub trait SampleGate {
    fn open_gate(&self, level: u64) -> (opened: u64);

    fn close_gate(&self, level: u64) -> (closed: u64);
}

pub struct SampleLatch {
    pub level: u64,
}

impl SampleGate for SampleLatch {
    fn open_gate( /* SELFTEST_VACUOUS */ &self, level: u64) -> (opened: u64)
        requires
            level > 0,
    {
        level
    }

    fn close_gate(&self, level: u64) -> (closed: u64)
        requires
            level > 0,
    {
        level
    }
}
"""

# assume_specification 指到工作区里另一个包的常量那一格用（reuse-assume-specification-three-questions；core 指 singlefs_format 常量的样子）：
# 两条各指一个常量，第三个常量没人指
SELFTEST_ASSUME_TARGETS_EXTRA = """
pub assume_specification[ singlefs_format::SLOT_WIDTH ] -> (width: u64) returns 8u64;

pub assume_specification[ singlefs_format::RING_START ] -> (start: u64) returns 16u64;
"""
SELFTEST_ASSUME_TARGETS_FORMAT = "pub const SLOT_WIDTH: u64 = 8;\npub const RING_START: u64 = 16;\npub const UNUSED_SAMPLE: u64 = 3;\n"

# 关联 const 点名那几格用（list-items-associated-const-prefix、proof-associated-const-prefix-green、proof-associated-const-point-names）：
# 关联 const FIRST 是自由 const FIRST_COUNTER 的前一段，自由 const LAST 是关联 const LAST_SLOT 的前一段
SELFTEST_CONST_PREFIX_EXTRA = """
pub struct Ordinal(pub u32);

impl Ordinal {
    pub const FIRST: Self = Self(1);

    pub const LAST_SLOT: u64 = 9;
}

pub const FIRST_COUNTER: u64 = 1;

pub const LAST: u64 = 2;
"""

# 同一个模块里另一个 impl 也有 FIRST（proof-point-name-two-impls-collision-red）
SELFTEST_CONST_SECOND_IMPL_EXTRA = """
pub struct OtherOrdinal(pub u32);

impl OtherOrdinal {
    pub const FIRST: Self = Self(2);
}
"""

# trait 里带默认值的 const 与 impl 里覆盖它的同名 const（proof-point-name-trait-default-collision-red）
SELFTEST_CONST_TRAIT_DEFAULT_EXTRA = """
pub struct Ordinal(pub u32);

pub trait Limited {
    const LIMIT: u64 = 3;
}

impl Limited for Ordinal {
    const LIMIT: u64 = 4;
}
"""

# 空真插入点认 const fn 那一格用（vacuity-const-fn-label）：带 requires 的 const fn 接在 keep_small 后面
SELFTEST_CONST_FN_EXTRA = """
pub const fn to_slot_offset(slot: u64) -> (offset: u64)
    requires
        slot < 1000,
    ensures
        offset == slot * 8,
{
    slot * 8
}
"""

# 空真插入点不认 assert(…) by (…) 的 requires 那几格用（list-items-assert-by-requires-skipped、vacuity-assert-by-requires-skipped、
# proof-assert-by-requires-skip-reported）：core 的 bytes.rs、unit.rs、journal.rs 里 by (nonlinear_arith) 接 requires … ; 的写法
SELFTEST_ASSERT_BY_EXTRA = """
pub fn halve_within(value: u64, bound: u64) -> (result: u64)
    requires
        value < 2 * bound,
    ensures
        result < bound,
{
    let result = value / 2;
    proof {
        assert(result < bound) by (nonlinear_arith)
            requires
                value < 2 * bound,
                result == value / 2,
        ;
    }
    result
}
"""

# requires 挂在普通 proof 块后面、往下一行只有 ;（list-items-requires-on-plain-block-red）
SELFTEST_PLAIN_BLOCK_REQUIRES_EXTRA = """
pub fn halve_plain(value: u64) -> (result: u64)
    ensures
        result <= value,
{
    let result = value / 2;
    proof {
        assert(result <= value);
    }
        requires
            value > 1,
    ;
    result
}
"""

# requires 挂在不带 by 的 assert 上、往下一行只有 ;（list-items-requires-on-assert-without-by-red）
SELFTEST_ASSERT_WITHOUT_BY_EXTRA = """
pub fn halve_unproved(value: u64) -> (result: u64)
    ensures
        result <= value,
{
    let result = value / 2;
    proof {
        assert(result <= value)
            requires
                value > 1,
        ;
    }
    result
}
"""

# spec const 成项那一格用（proof-spec-const-items）：impl 里的与自由的 spec const 各一个（真 Verus 各验出 1 条，点 *::<名> 与 <名>）
SELFTEST_SPEC_CONST_EXTRA = """
pub struct Window(pub u64);

impl Window {
    pub spec const SPEC_WIDTH: u64 = 4;
}

pub spec const SPEC_HEIGHT: u64 = 3;
"""

SELFTEST_TRAIT_EXTRA = """
pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures
            r <= 10;
}
"""

SELFTEST_TRAIT_OUTSIDE = """
pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> u64 { 99 }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock;
    impl Device for Mock {
        fn read_slot(&self, _slot: u64) -> u64 { 1 }
    }
}
"""

# 登记式放过表那几格用（trait-impl-lag-*）：放过表登记 BadDevice 那一行；同一个 trait 在 verus! 外面另有一个没登记的 impl；BadDevice 已挪进 verus! 里的样子
SELFTEST_TRAIT_LAG_ROW = "core_sample::kernel::Device\tBadDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：这一簇还没把它挪进 verus!"

SELFTEST_TRAIT_OUTSIDE_SECOND = """
pub struct WorseDevice;

impl Device for WorseDevice {
    fn read_slot(&self, _slot: u64) -> u64 { 98 }
}
"""

SELFTEST_TRAIT_INSIDE = """
pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> (r: u64)
    {
        3
    }
}
"""

# verus! 外面一个 trait 与类型都不在被验区域里的 impl（放过表里登记它不红、没放过什么，成功句点名）
SELFTEST_UNVERIFIED_OUTSIDE = """
pub trait Plain {
    fn plain(&self) -> u64;
}

pub struct Other;

impl Plain for Other {
    fn plain(&self) -> u64 { 1 }
}
"""

# external_body 的规格原文那一格用（trust-digest-signature-only）：impl 里唯一（也是最后）的 fn 带 external_body
SELFTEST_IMPL_EXTERNAL_BODY = """
pub struct Shown;

impl Shown {
    #[verifier::external_body]
    pub fn show(&self, x: u64) -> (shown: u64)
        ensures
            shown == x,
    {
        x
    }
}
"""

# trait 里没有函数体的声明、ensures 里有结构体字面量那一格用（proof-trait-declaration-struct-literal）：Store::put 不成项，Shelf::put 成项
SELFTEST_TRAIT_LITERAL_EXTRA = """
pub struct Request {
    pub slot: u64,
}

pub open spec fn issued(request: Request) -> bool {
    request.slot < 10
}

pub trait Store {
    fn put(&mut self, slot: u64) -> (r: bool)
        ensures
            r == issued(Request { slot: slot }),
    ;
}

pub struct Shelf;

impl Store for Shelf {
    fn put(&mut self, slot: u64) -> (r: bool)
    {
        slot < 10
    }
}
"""


def captured_stdout(function, *arguments):
    """自证用：起 function，交 (它的返回值, 收下的 stdout)（StringIO，读 getvalue()）。"""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        returned = function(*arguments)
    return returned, buffer


def counting_workspace_loads(function):
    """自证用：不带参数起 function，交 (它的返回值, 这期间 Rust 工作区（lib_function_units.py 的 RustWorkspace）被载入了几次)。"""
    module = lib_verdict_library.function_units()
    original = module.RustWorkspace
    loads = []

    class CountingWorkspace(original):
        def __init__(self, *arguments, **keywords):
            loads.append(1)
            super().__init__(*arguments, **keywords)
    module.RustWorkspace = CountingWorkspace
    try:
        returned = function()
    finally:
        module.RustWorkspace = original
    return returned, len(loads)


def selftest_kernel_with(extra, after=""):
    """SELFTEST_KERNEL 的 verus! 里另加 extra，verus! 之后接 after。"""
    return SELFTEST_KERNEL.replace("} // verus!\n", extra + "} // verus!\n") + after


SELFTEST_KERNEL = """use vstd::prelude::*;
verus! {
// 这里的 assume 只在注释里，不算作弊标记
pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= 10,
{
    let pick = |candidate: u64| -> (picked: u64)
        requires
            candidate <= 10,
        { candidate };
    if value > 10 { pick(10) } else { pick(value) }
}
} // verus!
"""


# 必红世界按落点判的自证用：tiny 的函数体带一个原来就有的标记（假 Verus 在这一行报一条证明错），在 clamp_to_limit 与 keep_small 之间；
# 改坏 clamp_to_limit 的替换文比原文多几行时，tiny 里那一行在改坏之后的文件里挪到 keep_small 在改坏之前的文件里的行数上
SELFTEST_LANDING_KERNEL = """use vstd::prelude::*;
verus! {
pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn tiny(value: u64) -> (result: u64)
    ensures
        result <= 3,
{
    if value > 3 { 3 } else { value } // SELFTEST_BROKEN
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= 10,
{
    if value > 10 { 10 } else { value }
}
} // verus!
"""


SELFTEST_SHARED_SPEC_KERNEL = """use vstd::prelude::*;
verus! {
pub open spec fn small_bound() -> u64 {
    10
}

pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= small_bound(),
{
    if value > 10 { 10 } else { value }
}

pub fn keep_tiny(value: u64) -> (result: u64)
    ensures
        result <= small_bound(),
{
    if value > 3 { 3 } else { value }
}
} // verus!
"""


# 证明成项的四类各一个、加一个不成项的 spec fn：义务条数（假 Verus 照真 Verus 量到的数）Pair::clone 1、sum_to 1、lemma_within_itself 1、count_up 2（一个 while），
# within 0（不成项）；之和 5
SELFTEST_ITEMS_KERNEL = """use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub low: u64,
    pub high: u64,
}

pub open spec fn within(value: u64, limit: u64) -> bool {
    value <= limit
}

pub open spec fn sum_to(n: nat) -> nat
    decreases n,
{
    if n == 0 { 0 } else { n + sum_to((n - 1) as nat) }
}

pub proof fn lemma_within_itself(value: u64)
    ensures
        within(value, value),
{
}

pub fn count_up(limit: u64) -> (result: u64)
    ensures
        result == limit,
{
    let mut index: u64 = 0;
    while index < limit
        invariant
            index <= limit,
        decreases limit - index,
    {
        index = index + 1;
    }
    index
}
} // verus!
"""
