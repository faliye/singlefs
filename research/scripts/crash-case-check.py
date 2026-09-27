#!/usr/bin/env python3
# admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
"""崩溃枚举用例必须住在 checker 档包、标 #[ignore] 的要登记成 crash-case：直接调全量崩溃枚举函数的测试函数，写在别的包里、或标了 ignore 却没登记，判红。

用法：
    crash-case-check.py [仓根]
    crash-case-check.py --selftest          # CRASH_CASE_CHECK_BREAK=<项> 时必须判红

判法：
  ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）与 crates/*/src/**/*.rs（库与装置二进制里 #[cfg(test)] 的内联测试）
     里每个带 #[test] 的函数：函数体里调了 enumerate_layer0 一族（名字以 enumerate_layer0 起头的函数，快档 quick_tier 那几个除外），
     或调了同一文件里一个（传递地）调它们的函数，算崩溃枚举用例；
  ② 它要住在 crates/singlefs-checker-tier/tests/（checker 档，D13 已定项 15，.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）：
     写在别的包（harness 这类日常测试包）里判红，那会让 cargo test 那个包的人跑到全量枚举；
  ③ checker 档包里标了 #[ignore] 的（全量那条），.claude/gate.d/stage-inputs.tsv 里要有一行键是 crash-case:<名>、第三列
     test=singlefs-checker-tier:<测试目标>:<这个函数名>（测试目标是文件名去掉 .rs）——标了 ignore 又没登记的谁都不跑；
     不标 #[ignore] 的是 checker 档的快档（小流、几步的合成流），随 cargo test -p singlefs-checker-tier 跑，不要求登记。
  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：写明为什么不算用例。
判不了的：经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（报告里写明是按直接调用认的）。
退出码：0 都齐；1 有缺的（逐个列出）；2 用法错或读不了登记表。
"""
import glob
import os
import re
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BROKEN = os.environ.get("CRASH_CASE_CHECK_BREAK", "")

CHECKER_PACKAGE = "singlefs-checker-tier"
ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)\s*(?:::<[^>]*>)?\s*\(")
TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")
CRASH_CASE_ROW = re.compile(r"^crash-case:[^\t]+\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")


def function_body(text, start):
    """从 fn 签名之后第一个 { 起按括号配对取函数体；字符串与注释里的括号不另判（测试函数里少见）。"""
    opening = text.find("{", start)
    if opening < 0:
        return ""
    depth = 0
    for index in range(opening, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[opening:index + 1]
    return text[opening:]


def enumerating_helpers(text):
    """同一文件里（传递地）调全量崩溃枚举的非测试函数名。"""
    if BROKEN == "helpers-unseen":
        return set()
    bodies = {}
    for match in TEST_FUNCTION.finditer(text):
        if "#[test]" not in match.group(1):
            bodies[match.group(2)] = function_body(text, match.end())
    enumerating = {name for name, body in bodies.items()
                   if any("quick_tier" not in call for call in ENUMERATION_CALL.findall(body))}
    changed = True
    while changed:
        changed = False
        for name, body in bodies.items():
            if name not in enumerating and any(re.search(r"\b" + re.escape(helper) + r"\s*\(", body) for helper in enumerating):
                enumerating.add(name)
                changed = True
    return enumerating


def crash_enumeration_tests(root):
    """→ [(包, 测试目标, 函数名, 带没带 ignore, 文件:行)]。"""
    found = []
    paths = sorted(glob.glob(os.path.join(root, "crates", "*", "tests", "*.rs")))
    paths += sorted(glob.glob(os.path.join(root, "crates", "*", "src", "**", "*.rs"), recursive=True))
    for path in paths:
        relative = os.path.relpath(path, os.path.join(root, "crates"))
        package_directory = relative.split(os.sep)[0]
        target = os.path.basename(path)[:-3]
        text = open(path, encoding="utf-8", errors="replace").read()
        helpers = enumerating_helpers(text)
        for match in TEST_FUNCTION.finditer(text):
            attributes = match.group(1)
            if "#[test]" not in attributes:
                continue
            body = function_body(text, match.end())
            calls = [name for name in ENUMERATION_CALL.findall(body)
                     if "quick_tier" not in name or BROKEN == "quick-tier-counts"]
            calls += [helper for helper in helpers if re.search(r"\b" + re.escape(helper) + r"\s*\(", body)]
            if not calls:
                continue
            waiver = re.search(r"crash-case-check:not-a-crash-case\s+(\S.{3,})", attributes)
            if waiver and BROKEN != "waiver-ignored":
                continue
            ignored = bool(re.search(r"#\[ignore\b", attributes)) and BROKEN != "ignore-unseen"
            line = text.count("\n", 0, match.start(2)) + 1
            found.append((package_directory, target, match.group(2), ignored, f"{os.path.relpath(path, root)}:{line}"))
    return found


def registered_cases(root):
    path = os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv")
    registered = set()
    for line in open(path, encoding="utf-8"):
        match = CRASH_CASE_ROW.match(line)
        if match:
            registered.add((match.group(1), match.group(2), match.group(3)))
    return registered


def problems_in(root):
    registered = registered_cases(root)
    registered_targets = {(target, function) for _, target, function in registered}
    tests = crash_enumeration_tests(root)
    problems = []
    for package_directory, target, function, ignored, where in tests:
        if package_directory != CHECKER_PACKAGE and BROKEN != "package-unchecked":
            problems.append(f"{where} {function}：直接调全量崩溃枚举，却写在 {package_directory} 包里（崩溃枚举用例要住在 crates/{CHECKER_PACKAGE}/tests/）")
            continue
        if ignored and (target, function) not in registered_targets and BROKEN != "rows-ignored":
            problems.append(f"{where} {function}：标了 #[ignore] 却没有登记成 crash-case（.claude/gate.d/stage-inputs.tsv 里没有 test={CHECKER_PACKAGE}:{target}:{function}），谁都不跑它")
    return tests, problems


def run(root):
    try:
        tests, problems = problems_in(root)
    except OSError as error:
        print(f"✗ 读不了登记表或源码：{error}\n→ 怎么办：在仓根跑，或给仓根参数")
        return 2
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems:
        print(f"  ✗ 崩溃枚举用例 {len(tests)} 条里有 {len(problems)} 处写在 checker 档包之外、或标了 ignore 没登记（逐处列在上面）")  # gate-lint:summary
        print(f"  → 怎么办：把那个测试文件挪进 crates/{CHECKER_PACKAGE}/tests/（共用模块经 #[path = \"../../singlefs-harness/tests/common/mod.rs\"] mod common; 指回去），"
              "全量那条标 #[ignore] 并把 crash-case:<名> 那一行（第三列 test=singlefs-checker-tier:<测试目标>:<函数名> 与 count-line 等）登记进 "
              ".claude/gate.d/stage-inputs.tsv；小流的快档不标 ignore、不用登记（.claude/rules/verification.md）")
        return 1
    print(f"  ✓ 崩溃枚举用例都住在 {CHECKER_PACKAGE} 包里、标了 #[ignore] 的都登记了（查了 {len(tests)} 条；按测试函数直接调 enumerate_layer0 一族、或经同一文件里的函数传递地调它们认；经 tests/<子目录>/ 里的共用函数间接调的、宏展开出来的认不出）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="crash-case-check-selftest-")
    try:
        tests_directory = os.path.join(work, "crates", CHECKER_PACKAGE, "tests")
        harness_tests_directory = os.path.join(work, "crates", "singlefs-harness", "tests")
        os.makedirs(tests_directory)
        os.makedirs(harness_tests_directory)
        os.makedirs(os.path.join(work, ".claude", "gate.d"))
        open(os.path.join(tests_directory, "plain.rs"), "w").write(
            "#[test]\n#[ignore]\nfn registered_case() {\n    let report = enumerate_layer0_versions(&stream);\n}\n\n"
            "#[test]\nfn quick_case() {\n    enumerate_layer0_quick_tier_versions(&stream);\n}\n\n"
            "#[test]\nfn small_stream_in_the_quick_tier() {\n    enumerate_layer0_selecting(&small, 1);\n}\n\n"
            "#[test]\nfn unrelated() {\n    assert_eq!(1, 1);\n}\n")
        open(os.path.join(tests_directory, "synthetic.rs"), "w").write(
            "// crash-case-check:not-a-crash-case 三步的合成流，单跑不到一秒\n#[test]\nfn tiny_stream() {\n    enumerate_layer0(&tiny);\n}\n")
        open(os.path.join(harness_tests_directory, "daily.rs"), "w").write("#[test]\nfn daily_quick() {\n    enumerate_layer0_quick_tier_versions(&s);\n}\n")
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "w").write(
            f"# 样本\ncrash-case:demo\tcrates/\ttest={CHECKER_PACKAGE}:plain:registered_case count-line=x\t#样本\n")
        failures = []
        tests, problems = problems_in(work)
        if len(tests) != 2 or problems:
            failures.append(f"干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题（checker 档包里不标 ignore 的快档放行），实际 {len(tests)} 条、{problems}")
        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\n#[ignore]\nfn forgot_to_register() {\n    enumerate_layer0_selecting(&s, 1);\n}\n")
        open(os.path.join(harness_tests_directory, "daily.rs"), "a").write("\n#[test]\nfn full_in_harness() {\n    enumerate_layer0(&s);\n}\n"
            "\nfn prepare_and_enumerate(s: &S) {\n    enumerate_layer0_selecting(s, 1);\n}\n\n#[test]\nfn full_through_a_helper() {\n    prepare_and_enumerate(&s);\n}\n")
        _, problems = problems_in(work)
        if not any("forgot_to_register" in problem and "没有登记" in problem for problem in problems):
            failures.append(f"checker 档包里标了 ignore 没登记的用例应当报缺登记，实际 {problems}")
        if not any("full_in_harness" in problem and "singlefs-harness 包里" in problem for problem in problems):
            failures.append(f"写在 harness 包里的崩溃枚举用例应当报住错了包，实际 {problems}")
        if not any("full_through_a_helper" in problem for problem in problems):
            failures.append(f"harness 包里经同文件 helper 调全量枚举的用例应当报住错了包，实际 {problems}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 crash_enumeration_tests() / problems_in() 的判法；CRASH_CASE_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ crash-case-check 自检通过：checker 档包里登记了的放行、不标 ignore 的快档放行，注明不是用例的不算，标了 ignore 没登记的报缺登记，写在 harness 包里的（直接调或经同文件 helper 调）报住错了包（查了 6 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else REPOSITORY_ROOT))
