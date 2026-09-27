#!/usr/bin/env python3
# admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
"""崩溃枚举用例必须标 #[ignore] 并登记成 crash-case：直接调全量崩溃枚举函数的测试函数，漏一样就判红。

用法：
    crash-case-check.py [仓根]
    crash-case-check.py --selftest          # CRASH_CASE_CHECK_BREAK=<项> 时必须判红

判法：
  ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）里每个带 #[test] 的函数，函数体里直接调了
     enumerate_layer0 一族（名字以 enumerate_layer0 起头的函数），而不是快档（名字里带 quick_tier）的，算崩溃枚举用例；
  ② 它要带 #[ignore]（不带 --ignored 的 cargo test 不跑它）；
  ③ .claude/gate.d/stage-inputs.tsv 里要有一行键是 crash-case:<名>、第三列 test=<包>:<测试目标>:<这个函数名>，测试目标是文件名去掉 .rs。
  测试目标名字里带 layer0 的整个二进制归门禁 54 号的层 0 流（重型测试闸按名字认它、54 号整个二进制跑），②③ 都不判——标了 #[ignore] 反而让 54 号跑不到它。
  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：枚举的是几步的合成流、单跑几秒的，写明为什么。
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


def crash_enumeration_tests(root):
    """→ [(包, 测试目标, 函数名, 带没带 ignore, 文件:行)]。"""
    found = []
    for path in sorted(glob.glob(os.path.join(root, "crates", "*", "tests", "*.rs"))):
        package_directory = os.path.basename(os.path.dirname(os.path.dirname(path)))
        target = os.path.basename(path)[:-3]
        text = open(path, encoding="utf-8", errors="replace").read()
        for match in TEST_FUNCTION.finditer(text):
            attributes = match.group(1)
            if "#[test]" not in attributes:
                continue
            calls = [name for name in ENUMERATION_CALL.findall(function_body(text, match.end()))
                     if "quick_tier" not in name or BROKEN == "quick-tier-counts"]
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
        if "layer0" in target and BROKEN != "layer0-checked":
            continue
        if not ignored:
            problems.append(f"{where} {function}：直接调全量崩溃枚举，却没标 #[ignore]")
        if (target, function) not in registered_targets and BROKEN != "rows-ignored":
            problems.append(f"{where} {function}：没有登记成 crash-case（.claude/gate.d/stage-inputs.tsv 里没有 test=<包>:{target}:{function}）")
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
        print(f"  ✗ 崩溃枚举用例 {len(tests)} 条里有 {len(problems)} 处缺 #[ignore] 或缺登记（逐处列在上面）")  # gate-lint:summary
        print("  → 怎么办：给那个测试函数加 #[ignore]，把 crash-case:<名> 那一行（第三列 test=<包>:<测试目标>:<函数名> 与 count-line 等）登记进 "
              ".claude/gate.d/stage-inputs.tsv；层 0 流的快档用 quick_tier 那一族，不算崩溃枚举用例")
        return 1
    print(f"  ✓ 崩溃枚举用例都标了 #[ignore]、不在层 0 二进制里的都登记了（查了 {len(tests)} 条；按测试函数直接调 enumerate_layer0 一族认，经共用函数间接调的认不出）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="crash-case-check-selftest-")
    try:
        tests_directory = os.path.join(work, "crates", "demo", "tests")
        os.makedirs(tests_directory)
        os.makedirs(os.path.join(work, ".claude", "gate.d"))
        open(os.path.join(tests_directory, "plain.rs"), "w").write(
            "#[test]\n#[ignore]\nfn registered_case() {\n    let report = enumerate_layer0_versions(&stream);\n}\n\n"
            "#[test]\nfn quick_case() {\n    enumerate_layer0_quick_tier_versions(&stream);\n}\n\n"
            "#[test]\nfn unrelated() {\n    assert_eq!(1, 1);\n}\n")
        open(os.path.join(tests_directory, "other_layer0.rs"), "w").write("#[test]\nfn stream_full() {\n    enumerate_layer0(&s);\n}\n")
        open(os.path.join(tests_directory, "synthetic.rs"), "w").write(
            "// crash-case-check:not-a-crash-case 三步的合成流，单跑不到一秒\n#[test]\nfn tiny_stream() {\n    enumerate_layer0(&tiny);\n}\n")
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "w").write(
            "# 样本\ncrash-case:demo\tcrates/\ttest=demo:plain:registered_case count-line=x\t#样本\n")
        failures = []
        tests, problems = problems_in(work)
        if len(tests) != 2 or problems:
            failures.append(f"干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题（层 0 二进制里的不判），实际 {len(tests)} 条、{problems}")
        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\nfn forgot_everything() {\n    enumerate_layer0_selecting(&s, 1);\n}\n")
        _, problems = problems_in(work)
        if not any("forgot_everything" in problem and "#[ignore]" in problem for problem in problems) or \
                not any("forgot_everything" in problem and "登记" in problem for problem in problems):
            failures.append(f"没标 ignore、没登记的用例应当两处都报，实际 {problems}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 crash_enumeration_tests() / problems_in() 的判法；CRASH_CASE_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ crash-case-check 自检通过：登记了且标了 ignore 的放行，快档、无关测试、注明不是用例的不算，层 0 二进制不判，没标没登记的两处都报（查了 4 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else REPOSITORY_ROOT))
