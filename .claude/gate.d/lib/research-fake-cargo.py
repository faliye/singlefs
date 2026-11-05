#!/usr/bin/env python3
# admission: always 只读被判的小工作区里的源码与 Cargo.toml，不连库，上一次的结论不替这一次作保
# run-condition: none 只用 python3 标准库
"""门禁 research-build-and-replay 样本档用的假 cargo：不编、不跑真测试，照样本小工作区的源码文本出 cargo 的输出。
门禁在样本根开跑时把这一份拷成 <样本根>/.research-fake-tools/cargo（登记表第二列 samples:.research-fake-tools 的假工具目录），PATH 前面放它。

认的子命令（别的退 101）：
  cargo -V | --version                 打一行版本
  cargo build [--release]              每个包的源码里有 SAMPLE_BUILD_ERROR 的退 101（打 error: 行），否则打 Finished 退 0
  cargo test [--release] [--all-features] [--bin <名>] [--no-fail-fast] [-p 包] [-- [--exact] [--skip <名>…] [--test-threads <n>] [<名>…]]
        每个包按 cargo 的次序：lib 的单测、Cargo.toml 的 [[bin]] 写了 path 的、src/bin 的单测、tests/ 下各集成测试、Doc-tests（带 lib 且 [lib] doctest 不是 false 的包才有）；打 libtest 形态的输出（Running 行的源文件路径相对包根、成员包也写 src/lib.rs，running N tests、
        test <全名> ... ok|FAILED、failures: 名单、test result 汇总行）。没有 --no-fail-fast 时一个二进制红了，后面的不跑（打 error: test failed）。
        --exact 按全名相等、不带按子串：--skip 点名的跳过，位置参数里给了名字的只留对得上的。--bin <名> 只跑那一个 [[bin]]（不跑 lib、集成测试与 Doc-tests）。
        [[bin]] 编不过的两种（样本里变异表的替换文造的）：先于一切测试，打 error 退 101：用了以 missing_ 起头的名字（error[E0425]: cannot find value `…` in this scope）；
        Cargo.toml 的 [lints.rust] 写了 warnings = "deny" 而收到的 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 里没有 --cap-lints，源码里有没人调用的函数（error: function `…` is never used；main 与紧跟在 #[test] 后面的函数不算）。

单测怎么判（evaluate_test）：测试函数体里
  assert_eq!(<路径>::<函数>(<整数参数>), <整数>…)  拿源码里 `fn <函数>(<形参>) -> u32 { <表达式> }` 的表达式（只认 + - * 与形参、整数）算，不等就是红；
  assert!(!std::path::Path::new("<路径>").exists()…)  路径相对包根，在就是红（样本的开关文件）；
  concat!(env!("CARGO_MANIFEST_DIR"), "/<路径>")      路径接在包根后面，文件不在就是红（单测读留存产物的写法，scripts/experiment-apparatus/lib.rs 那一段）；
  writeln!(std::io::stdout().lock(), "NOT_RUN_…")     那一行原样接在 `test <名> ... ` 后面打出来（装置读不到输入的记号），测试照常算绿。
别的断言当绿。"""
import ast
import os
import re
import sys
import tomllib

VERSION_LINE = "cargo 0.0.0 (research gate sample)"


def read_manifest(directory):
    try:
        with open(os.path.join(directory, "Cargo.toml"), "rb") as handle:
            return tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        return {}


def packages():
    """[(包名, 目录)]：当前目录的 [package]（有就排第一），再加 [workspace] members 逐个。"""
    top = read_manifest(".")
    found = []
    if isinstance(top.get("package"), dict) and top["package"].get("name"):
        found.append((top["package"]["name"], "."))
    for member in (top.get("workspace") or {}).get("members") or []:
        manifest = read_manifest(member)
        if isinstance(manifest.get("package"), dict) and manifest["package"].get("name"):
            found.append((manifest["package"]["name"], member))
    return found


def source_files(directory):
    found = []
    for base, _dirs, names in os.walk(directory):
        if os.sep + "target" in base + os.sep:
            continue
        for name in sorted(names):
            if name.endswith(".rs"):
                found.append(os.path.join(base, name))
    return sorted(found)


def parse_functions(text):
    """{函数名: (形参名列表, 表达式文本)}：只收 `fn 名(形参: 类型, …) -> 类型 { 单个表达式 }` 那种。"""
    found = {}
    for match in re.finditer(r"^\s*(?:pub\s+)?fn\s+(\w+)\(([^)]*)\)\s*->\s*\w+\s*\{\s*\n\s*([^\n;]+?)\s*\n\s*\}", text, re.M):
        found[match.group(1)] = ([part.split(":")[0].strip() for part in match.group(2).split(",") if part.strip()], match.group(3))
    return found


def evaluate(expression, names):
    def walk(node):
        if isinstance(node, ast.Expression):
            return walk(node.body)
        if isinstance(node, ast.Constant) and isinstance(node.value, int):
            return node.value
        if isinstance(node, ast.Name) and node.id in names:
            return names[node.id]
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Sub, ast.Mult)):
            left, right = walk(node.left), walk(node.right)
            return left + right if isinstance(node.op, ast.Add) else left - right if isinstance(node.op, ast.Sub) else left * right
        raise ValueError(expression)
    return walk(ast.parse(expression, mode="eval"))


def discover_tests(path):
    """[(全名, 函数体文本, 行号)]：#[test] 后面的 fn；全名带 mod 路径（mod 用缩进配对）。"""
    lines = open(path, encoding="utf-8").read().split("\n")
    tests, modules = [], []
    for number, line in enumerate(lines):
        indent = len(line) - len(line.lstrip())
        while modules and modules[-1][1] >= indent and line.strip() == "}":
            modules.pop()
            break
        opened = re.match(r"^(\s*)(?:pub\s+)?mod\s+(\w+)\s*\{\s*$", line)
        if opened:
            modules.append((opened.group(2), len(opened.group(1))))
            continue
        if line.strip() == "#[test]":
            heading = re.match(r"^(\s*)fn\s+(\w+)\(\)\s*\{\s*$", lines[number + 1]) if number + 1 < len(lines) else None
            if not heading:
                continue
            body, depth = [], 1
            for later in lines[number + 2:]:
                depth += later.count("{") - later.count("}")
                if depth <= 0:
                    break
                body.append(later)
            tests.append(("::".join([name for name, _indent in modules] + [heading.group(2)]), "\n".join(body), number + 2))
    return tests


def evaluate_test(body, functions, package_directory):
    """交 (是否绿, 失败说明, 要原样接在 test 行后面的行)。"""
    printed = ""
    for match in re.finditer(r'writeln!\(std::io::stdout\(\)\.lock\(\), "(NOT_RUN_[^"]*)"\)', body):
        printed = match.group(1)
    for match in re.finditer(r'concat!\(env!\("CARGO_MANIFEST_DIR"\),\s*"(/[^"]+)"\)', body):
        if not os.path.exists(os.path.normpath(package_directory + match.group(1))):
            return False, f"读不到 {match.group(1)}（No such file or directory）", printed
    for match in re.finditer(r"assert_eq!\(\s*[\w:]*?(\w+)\(([^)]*)\)\s*,\s*(-?\d+)", body):
        name, arguments, want = match.group(1), match.group(2), int(match.group(3))
        if name not in functions:
            continue
        parameters, expression = functions[name]
        try:
            values = [int(part) for part in arguments.split(",") if part.strip()]
            got = evaluate(expression, dict(zip(parameters, values)))
        except (ValueError, SyntaxError):
            continue
        if got != want:
            return False, f"assertion `left == right` failed\n  left: {got}\n right: {want}", printed
    for match in re.finditer(r'assert!\(\s*!\s*std::path::Path::new\("([^"]+)"\)\.exists\(\)', body):
        if os.path.exists(os.path.join(package_directory, match.group(1))):
            return False, f"assertion failed: 开关文件 {match.group(1)} 在", printed
    return True, "", printed


def libtest_arguments(after):
    """-- 后面的 libtest 参数：(--exact 有没有, --skip 点名的名字, 位置参数里的筛选名)。--test-threads 连值吃掉，别的 -- 起头的忽略。"""
    exact, skipped, filters, index = False, [], [], 0
    while index < len(after):
        word = after[index]
        if word == "--exact":
            exact = True
        elif word == "--skip" and index + 1 < len(after):
            skipped.append(after[index + 1])
            index += 1
        elif word == "--test-threads":
            index += 1
        elif not word.startswith("--"):
            filters.append(word)
        index += 1
    return exact, skipped, filters


def run_test_binary(label, tests, functions, package_directory, exact, skipped, filters=()):
    """打一个测试二进制的输出，交 (失败条数, 输出文本)。filters 非空时只留名字对得上的（--exact 按全名相等，否则按子串）。"""
    matches = lambda test, name: (test[0] == name) if exact else (name in test[0])
    tests = [test for test in tests if not filters or any(matches(test, name) for name in filters)]
    chosen = [test for test in tests if not any(matches(test, name) for name in skipped)]
    filtered = len(tests) - len(chosen)
    out = [f"     Running {label}", f"running {len(chosen)} test{'' if len(chosen) == 1 else 's'}"]
    failures = []
    for full_name, body, line in sorted(chosen):
        green, why, printed = evaluate_test(body, functions, package_directory)
        out.append(f"test {full_name} ... " + (printed + "\n" if printed else "") + ("ok" if green else "FAILED"))
        if not green:
            failures.append((full_name, why, line))
    if failures:
        out.append("\nfailures:\n")
        for full_name, why, line in failures:
            out += [f"---- {full_name} stdout ----", f"thread '{full_name}' panicked at src:{line}:9:", why, ""]
        out += ["\nfailures:"] + [f"    {full_name}" for full_name, _why, _line in failures]
    outcome = "FAILED" if failures else "ok"
    out.append(f"\ntest result: {outcome}. {len(chosen) - len(failures)} passed; {len(failures)} failed; 0 ignored; 0 measured; {filtered} filtered out; finished in 0.00s\n")
    return len(failures), "\n".join(out)


def command_build():
    for name, directory in packages():
        for path in source_files(directory):
            if "SAMPLE_BUILD_ERROR" in open(path, encoding="utf-8").read():
                sys.stderr.write(f"error: could not compile `{name}` ({path}) due to 1 previous error\n")
                return 101
    sys.stderr.write("    Finished `release` profile [optimized] target(s) in 0.01s\n")
    return 0


def compile_problems(manifest, source):
    """源码文本里编不过的几处：用了以 missing_ 起头的名字、[lints.rust] warnings = "deny" 下收到的 RUSTFLAGS 里没有 --cap-lints 而有没人调的函数。"""
    text = open(source, encoding="utf-8").read()
    problems = []
    for number, line in enumerate(text.split("\n"), 1):
        for missing in re.findall(r"\b(missing_\w+)\b", line):
            problems.append(f"error[E0425]: cannot find value `{missing}` in this scope\n --> {source}:{number}:1")
    # 照真 cargo：CARGO_ENCODED_RUSTFLAGS 设了（空串也算）就只读它、不读 RUSTFLAGS
    rust_flags = os.environ["CARGO_ENCODED_RUSTFLAGS"] if "CARGO_ENCODED_RUSTFLAGS" in os.environ else os.environ.get("RUSTFLAGS", "")
    if (manifest.get("lints", {}).get("rust", {}).get("warnings") == "deny") and "--cap-lints" not in rust_flags:
        # #[test] 的函数没人调也不算死代码（测试框架调它）
        test_functions = set(re.findall(r"^\s*#\[test\]\s*\n\s*(?:pub\s+)?fn\s+(\w+)\(", text, re.M))
        for name in re.findall(r"^\s*(?:pub\s+)?fn\s+(\w+)\(", text, re.M):
            if name != "main" and name not in test_functions and len(re.findall(rf"\b{name}\(", text)) <= 1:
                problems.append(f"error: function `{name}` is never used\n --> {source}:1:1")
    return problems


def command_test(arguments):
    no_fail_fast = "--no-fail-fast" in arguments
    before = arguments[:arguments.index("--")] if "--" in arguments else arguments
    after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    only_bin = before[before.index("--bin") + 1] if "--bin" in before[:-1] else None
    exact, skipped, filters = libtest_arguments(after)
    exit_code = 0
    for name, directory in packages():
        functions = {}
        for path in source_files(directory):
            functions.update(parse_functions(open(path, encoding="utf-8").read()))
        targets = []
        # cargo 打的源文件路径相对这个包的根（成员包也写 src/lib.rs，不带成员目录）；门禁靠它对上计划里的目标根文件
        package_relative = lambda path: os.path.relpath(path, directory)
        lib = os.path.join(directory, "src", "lib.rs")
        if os.path.isfile(lib) and only_bin is None:
            targets.append((f"unittests {package_relative(lib)} (target/release/deps/{name.replace('-', '_')}-0000000000000000)", discover_tests(lib), "--lib"))
        # Cargo.toml 的 [[bin]] 写了 path 的（不在 src/bin 底下的）：先照编，编不过打 error 退 101，一条测试都不跑（照真 cargo 先编再跑）
        manifest = read_manifest(directory)
        for entry in manifest.get("bin", []):
            path = os.path.join(directory, entry.get("path", ""))
            if not entry.get("path") or not os.path.isfile(path) or (only_bin is not None and entry.get("name") != only_bin):
                continue
            problems = compile_problems(manifest, path)
            if problems:
                for problem in problems:
                    sys.stderr.write(problem + "\n")
                sys.stderr.write(f"error: could not compile `{name}` (bin \"{entry.get('name')}\" test) due to {len(problems)} previous error{'' if len(problems) == 1 else 's'}\n")
                return 101
            bin_name = entry.get("name", "bin")
            targets.append((f"unittests {package_relative(path)} (target/release/deps/{bin_name.replace('-', '_')}-0000000000000000)", discover_tests(path), f"--bin {bin_name}"))
        bin_directory = os.path.join(directory, "src", "bin")
        if os.path.isdir(bin_directory) and only_bin is None:
            for bin_name in sorted(os.listdir(bin_directory)):
                if bin_name.endswith(".rs"):
                    path = os.path.join(bin_directory, bin_name)
                    targets.append((f"unittests {package_relative(path)} (target/release/deps/{bin_name[:-3]}-0000000000000000)", discover_tests(path), f"--bin {bin_name[:-3]}"))
        tests_directory = os.path.join(directory, "tests")
        if os.path.isdir(tests_directory) and only_bin is None:
            for test_name in sorted(os.listdir(tests_directory)):
                if test_name.endswith(".rs"):
                    path = os.path.join(tests_directory, test_name)
                    targets.append((f"{package_relative(path)} (target/release/deps/{test_name[:-3]}-0000000000000000)", discover_tests(path), f"--test {test_name[:-3]}"))
        for label, tests, rerun in targets:
            if not tests:
                continue
            failed, text = run_test_binary(label, tests, functions, directory, exact, skipped, filters)
            sys.stdout.write(text + "\n")
            if failed:
                exit_code = 101
                if not no_fail_fast:
                    sys.stderr.write(f"error: test failed, to rerun pass `-p {name} {rerun}`\n")
                    return exit_code
        # Doc-tests 只有带 lib 目标、且 [lib] doctest 没写 false 的包才有（样本的小包都写了 doctest = false，批次数定成测试二进制的个数）
        has_doc_tests = os.path.isfile(lib) and (manifest.get("lib") or {}).get("doctest", True)
        if only_bin is None and has_doc_tests:
            sys.stdout.write(f"   Doc-tests {name.replace('-', '_')}\n\nrunning 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n")
    return exit_code


def main(arguments):
    if arguments[:1] in (["-V"], ["--version"]):
        print(VERSION_LINE)
        return 0
    commands = {"build": lambda: command_build(), "test": lambda: command_test(arguments[1:])}
    if not arguments or arguments[0] not in commands:
        sys.stderr.write(f"假 cargo 只认 -V、build、test：{arguments}\n")
        return 101
    return commands[arguments[0]]()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
