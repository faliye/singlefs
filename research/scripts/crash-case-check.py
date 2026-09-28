#!/usr/bin/env python3
# admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
"""测试源码的四样静态判据，判法只在这一份里；三道门禁各调各的那几样：

  placement     崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记成 crash-case:（54 号开跑前判）
  modules       checker 档每个测试文件第一行声明它测哪几个模块（54 号开跑前判）
  file-names    测试文件名不按里程碑、步号起（code-source-discipline 的一格）
  one-scenario  harness 档一条用例一个场景（harness-model-differential-and-scenarios 的一格）

用法：
    crash-case-check.py [--only <项>[,<项>…]] [仓根]     # 不给 --only 就四样都判
    crash-case-check.py --selftest                      # CRASH_CASE_CHECK_BREAK=<项> 时必须判红

判法：
  placement：
  ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）与 crates/*/src/**/*.rs（库与装置二进制里 #[cfg(test)] 的内联测试）
     里每个带 #[test] 的函数：函数体里调了 enumerate_layer0 一族（名字以 enumerate_layer0 起头的函数，快档 quick_tier 那几个除外），
     或自己逐个造崩溃状态（名字带 every_crash；或某个 for 循环体里对录制操作取到循环变量为止的前缀去 apply、或造 CrashImage），
     或调了同一文件里一个（传递地）这样做的函数，算崩溃枚举用例；只造一个固定崩溃态的不算；
  ② 它要住在 crates/singlefs-checker-tier/tests/（checker 档，D13 已定项 15，.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）：
     写在别的包（harness 这类日常测试包）里判红，那会让 cargo test 那个包的人跑到全量枚举；
  ③ checker 档包里标了 #[ignore] 的（全量那条），.claude/gate.d/stage-inputs.tsv 里要有一行键是 crash-case:<名>、第三列
     test=singlefs-checker-tier:<测试目标>:<这个函数名>（测试目标是文件名去掉 .rs）——标了 ignore 又没登记的 54 号 --full 不跑，谁都不跑；
     不标 #[ignore] 的是 checker 档的快档（小流、几步的合成流），随 cargo test -p singlefs-checker-tier 跑，不要求登记。
  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：写明为什么不算用例。
  modules：crates/singlefs-checker-tier/tests/*.rs 第一行「//! checker 档模块：<模块，按 TIER_MODULES 的次序用、隔开>」与它从
     singlefs_checker_tier:: 导入的模块逐个相同；一个都不导入的写「无（为什么）」。
  file-names：crates/singlefs-harness/tests/*.rs 与 crates/singlefs-checker-tier/tests/*.rs 的文件名不带里程碑序数、步号、增补号、
     并行线号、欠账号（MILESTONE_FILE_NAME）。
  one-scenario：crates/singlefs-harness/tests/*.rs 里带 #[test] 的函数，函数体里某个 for 循环体每一轮新建一个池
     （FRESH_POOL_CALL），判红；用例上面写了「harness-test-granularity:one-scenario <理由>」的不判。
判不了的：经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（成功行里写明是按直接调用认的）；
  一条用例里不建池的多场景（改的是同一个池）；名字里按里程碑起、却不是 MILESTONE_FILE_NAME 认得的那几种写法。
每一样各报一段：有问题的逐处列、再一句汇总带出路；没问题的一句成功行带查了多少；那一样一个对象都没有的报「本次无对象可判」。
退出码：0 选中的都齐（至少一样判过）；1 有缺的（逐个列出）；2 用法错或读不了登记表；77 选中的每一样都无对象可判。
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
HARNESS_PACKAGE = "singlefs-harness"
ITEMS = {
    "placement": "崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记成 crash-case:",
    "modules": "checker 档每个测试文件第一行声明它测哪几个模块",
    "file-names": "测试文件名不按里程碑、步号起",
    "one-scenario": "harness 档一条用例一个场景",
}
ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)\s*(?:::<[^>]*>)?\s*\(")
TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")
FOR_LOOP = re.compile(r"\bfor\s+\(?\s*(?:mut\s+)?([a-z_]\w*)[^{;]*?\bin\b")
SWEEP_BY_NAME = re.compile(r"every_crash")
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


def sweeps_crash_states_itself(name, body):
    """函数自己逐个造崩溃状态：名字带 every_crash；或某个 for 循环体里对录制操作取到循环变量为止的前缀去 apply
    （`.apply(&operations[..prefix])`）、或造崩溃镜像（`CrashImage {`）。只造一个固定崩溃态的（前缀不是循环变量）不算。"""
    if BROKEN == "self-sweep-unseen":
        return False
    if SWEEP_BY_NAME.search(name):
        return True
    for loop in FOR_LOOP.finditer(body):
        variable = loop.group(1)
        loop_body = function_body(body, loop.end())
        if re.search(r"\.apply\(\s*&\s*\w+\s*\[\s*\.\.=?\s*" + re.escape(variable) + r"\s*\]", loop_body):
            return True
        if re.search(r"\bCrashImage\s*\{", loop_body):
            return True
    return False


def enumerating_helpers(text):
    """同一文件里（传递地）调全量崩溃枚举的非测试函数名。"""
    if BROKEN == "helpers-unseen":
        return set()
    bodies = {}
    for match in TEST_FUNCTION.finditer(text):
        if "#[test]" not in match.group(1):
            bodies[match.group(2)] = function_body(text, match.end())
    enumerating = {name for name, body in bodies.items()
                   if any("quick_tier" not in call for call in ENUMERATION_CALL.findall(body))
                   or sweeps_crash_states_itself(name, body)}
    changed = True
    while changed:
        changed = False
        for name, body in bodies.items():
            if name not in enumerating and any(re.search(r"\b" + re.escape(helper) + r"\s*\(", body) for helper in enumerating):
                enumerating.add(name)
                changed = True
    return enumerating


def placement_source_paths(root):
    """placement 扫的源码：crates/*/tests/*.rs 与 crates/*/src/**/*.rs。"""
    paths = sorted(glob.glob(os.path.join(root, "crates", "*", "tests", "*.rs")))
    return paths + sorted(glob.glob(os.path.join(root, "crates", "*", "src", "**", "*.rs"), recursive=True))


def crash_enumeration_tests(root):
    """→ [(包, 测试目标, 函数名, 带没带 ignore, 文件:行)]。"""
    found = []
    for path in placement_source_paths(root):
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
            if sweeps_crash_states_itself(match.group(2), body):
                calls.append("自己逐个造崩溃状态")
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
    """placement：→ (认出的崩溃枚举用例, 问题)。读不了登记表抛 OSError。"""
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


FRESH_POOL_CALL = re.compile(r"\b(?:build_pool\w*|build_through_\w+|format_pool)\s*\(|\bMemoryPool::with_devices\s*\(")
TIER_MODULES = ["crash", "layer0_progress", "crash_injection", "bad_disk_input", "device_log", "on_device_modes", "crash_identity", "crash_amplification", "verdict_store", "gpu_unit_checks",
    "crash_facts",
    "crash_verify_gpu",
]
MODULE_DECLARATION = re.compile(r"^//! checker 档模块：(.+)$")
# 按里程碑、步号、增补号、并行线号、欠账号起的测试文件名：名字说的是它在哪一步写出来的，不是它测什么
MILESTONE_FILE_NAME = re.compile(r"(^|_)(first|second|third)_transaction(_|$)|_step_(zero|one|two|three|four|five|six|seven|eight|nine)(_|$)"
                                 r"|(^|_)supplement_(one|two|three|four)(_|$)|(^|_)parallel_line_(one|two|three)(_|$)|(^|_)c[0-9]{3}(_|$)")


def harness_test_paths(root):
    return sorted(glob.glob(os.path.join(root, "crates", HARNESS_PACKAGE, "tests", "*.rs")))


def tier_test_paths(root):
    return sorted(glob.glob(os.path.join(root, "crates", CHECKER_PACKAGE, "tests", "*.rs")))


def one_scenario_problems(root):
    """one-scenario：→ (查了几条 harness 用例, 问题)。"""
    problems, harness_tests = [], 0
    for path in harness_test_paths(root):
        text = open(path, encoding="utf-8", errors="replace").read()
        for match in TEST_FUNCTION.finditer(text):
            attributes = match.group(1)
            if "#[test]" not in attributes:
                continue
            harness_tests += 1
            if "harness-test-granularity:one-scenario" in attributes:
                continue
            body = function_body(text, match.end())
            for loop in FOR_LOOP.finditer(body):
                if FRESH_POOL_CALL.search(function_body(body, loop.end())) and BROKEN != "fresh-pool-loop-unseen":
                    line = text.count("\n", 0, match.start(2)) + 1
                    problems.append(f"{os.path.relpath(path, root)}:{line} {match.group(2)}：循环里每一轮新建一个池，是一条用例装了几个独立场景")
                    break
    return harness_tests, problems


def test_file_name_problems(root):
    """file-names：→ (查了几份测试文件, 问题)。"""
    problems = []
    paths = harness_test_paths(root) + tier_test_paths(root)
    for path in paths:
        stem = os.path.basename(path)[:-len(".rs")]
        if MILESTONE_FILE_NAME.search(stem) and BROKEN != "milestone-names-unseen":
            problems.append(f"{os.path.relpath(path, root)}：文件名按里程碑、步号、增补号或欠账号起（{stem}），改成它测什么：领域在前、场景在后")
    return len(paths), problems


def module_declaration_problems(root):
    """modules：→ (查了几份 checker 档测试文件, 问题)。"""
    problems, tier_files = [], 0
    for path in tier_test_paths(root):
        tier_files += 1
        text = open(path, encoding="utf-8", errors="replace").read()
        declared = MODULE_DECLARATION.match(text.split("\n", 1)[0])
        used = sorted({name for name in re.findall(r"singlefs_checker_tier::(\w+)", text) if name in TIER_MODULES}, key=TIER_MODULES.index)
        relative = os.path.relpath(path, root)
        if not declared:
            problems.append(f"{relative}：第一行没有「//! checker 档模块：…」（它导入的是 {'、'.join(used) or '无'}）")
            continue
        text_declared = declared.group(1).strip()
        declared_modules = [] if text_declared.startswith("无") else [name.strip() for name in text_declared.split("、")]
        if declared_modules != used and BROKEN != "module-declaration-unchecked":
            problems.append(f"{relative}：第一行声明 {text_declared}，导入的 checker 档模块是 {'、'.join(used) or '无'}")
    return tier_files, problems


def judge(root, items):
    """{项: (对象数, 问题, 成功行)}，按 items 的次序；读不了登记表时抛 OSError。"""
    if BROKEN == "only-ignored":
        items = list(ITEMS)
    results = {}
    for item in items:
        if item == "placement":
            scanned = len(placement_source_paths(root))
            tests, problems = problems_in(root)
            results[item] = (scanned, problems,
                             f"崩溃枚举用例都住在 {CHECKER_PACKAGE} 包里、标了 #[ignore] 的都登记了（扫了 {scanned} 份测试源码，认出 {len(tests)} 条；"
                             "按测试函数直接调 enumerate_layer0 一族、自己逐个造崩溃状态、或经同一文件里的函数传递地这样做认；"
                             "经 tests/<子目录>/ 里的共用函数间接调的、宏展开出来的认不出）")
        elif item == "modules":
            tier_files, problems = module_declaration_problems(root)
            results[item] = (tier_files, problems, f"checker 档 {tier_files} 份测试文件都声明了测哪几个模块、与导入对得上")
        elif item == "file-names":
            test_files, problems = test_file_name_problems(root)
            results[item] = (test_files, problems,
                             f"查了 {test_files} 份测试文件的名字（harness 档与 checker 档的 tests/*.rs），没有按里程碑、步号、增补号、并行线号或欠账号起的")
        elif item == "one-scenario":
            harness_tests, problems = one_scenario_problems(root)
            results[item] = (harness_tests, problems, f"harness 档 {harness_tests} 条用例都是一条一个场景")
    return results


REMEDIES = {
    "placement": [f"把那个测试文件挪进 crates/{CHECKER_PACKAGE}/tests/（共用模块经 #[path = \"../../singlefs-harness/tests/common/mod.rs\"] mod common; 指回去），",
                  "全量那条标 #[ignore] 并把 crash-case:<名> 那一行（第三列 test=singlefs-checker-tier:<测试目标>:<函数名> 与 count-line 等）登记进",
                  ".claude/gate.d/stage-inputs.tsv；小流的快档不标 ignore、不用登记（.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）。"],
    "modules": ["checker 档每个测试文件第一行写「//! checker 档模块：<它导入的模块，按 " + "、".join(TIER_MODULES) + " 的次序用、隔开>」，",
                "一个都不导入的写「无（为什么）」（.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）。"],
    "file-names": ["测试文件按它测什么起名：领域在前、场景在后，不带里程碑、步号、增补号、并行线号、欠账号，来历写进文件头的文档注释",
                   "（.claude/rules/verification.md「定义与名字」）；改名照 .claude/rules/path-moves.md「怎么做」改全仓引用。"],
    "one-scenario": ["harness 档一条用例只装一个场景：循环体挪进带参数的函数，每个取值一条 #[test]，红了只重跑那一条；",
                     "确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 // harness-test-granularity:one-scenario <理由>",
                     "（.claude/rules/verification.md「harness 档里再分轻用例与耗时用例」）。"],
}


def parse_only(text):
    """「a,b」→ [项…]（去重、保次序）；认不出的项 → None。"""
    items = []
    for item in (part.strip() for part in text.split(",")):
        if item not in ITEMS:
            return None
        if item not in items:
            items.append(item)
    return items or None


def run(root, items):
    try:
        results = judge(root, items)
    except OSError as error:
        print(f"  ✗ 读不了登记表或源码：{error}")
        print("  → 怎么办：在仓根跑，或给仓根参数；placement 那一样要读 .claude/gate.d/stage-inputs.tsv")
        return 2
    red, judged, empty = [], [], []
    for item, (objects, problems, _success) in results.items():
        if problems:
            red.append(item)
            for problem in problems:
                print(f"  ✗ {problem}")  # gate-lint:detail
            print(f"  ✗ {item}（{ITEMS[item]}）：查了 {objects} 项，{len(problems)} 处不对（逐处列在上面）")  # gate-lint:summary
            print("  → 怎么办：" + REMEDIES[item][0])
            for remedy_line in REMEDIES[item][1:]:
                print("            " + remedy_line)
        elif objects == 0:
            empty.append(item)
        else:
            judged.append(item)
    for item in empty:
        print(f"  ! {item}（{ITEMS[item]}）本次无对象可判：一个对象都没找到")
    if red:
        return 1
    if not judged:
        return 77
    for item in judged:
        print(f"  ✓ {results[item][2]}")
    print(f"  ✓ crash-case-check 判了 {len(judged)} 样（{'、'.join(judged)}），{len(empty)} 样无对象可判")
    return 0


SELFTEST_CASES = ["登记了的与快档放行、注明不是用例的不算", "标了 ignore 没登记", "住错了包（直接调、经 helper、自己逐个造崩溃状态）",
                  "固定崩溃态不算枚举", "每轮新建池与豁免", "模块声明缺与对不上", "按里程碑起名", "--only 只判点名的"]


def selftest():
    work =tempfile.mkdtemp(prefix="crash-case-check-selftest-")
    try:
        tests_directory = os.path.join(work, "crates", CHECKER_PACKAGE, "tests")
        harness_tests_directory = os.path.join(work, "crates", HARNESS_PACKAGE, "tests")
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
            "\nfn prepare_and_enumerate(s: &S) {\n    enumerate_layer0_selecting(s, 1);\n}\n\n#[test]\nfn full_through_a_helper() {\n    prepare_and_enumerate(&s);\n}\n"
            "\n#[test]\nfn sweeps_every_prefix() {\n    for prefix in before..=operations.len() {\n        let mut image = pool();\n        image.apply(&operations[..prefix]);\n    }\n}\n"
            "\n#[test]\nfn crash_image_on_each_boundary() {\n    for count in 0..=segments.len() {\n        let image = CrashImage { persisted: through(count) };\n    }\n}\n"
            "\nfn lowest_at_every_crash_point(o: &[Op]) -> u64 {\n    0\n}\n\n#[test]\nfn through_a_named_sweep() {\n    lowest_at_every_crash_point(&o);\n}\n"
            "\n#[test]\nfn one_fixed_crash_state_stays_in_the_harness() {\n    for request in requests {\n        check(request);\n    }\n    let image = CrashImage { persisted: all() };\n    base.apply(&operations[..last_root_slot_write]);\n}\n")
        _, problems = problems_in(work)
        if not any("forgot_to_register" in problem and "没有登记" in problem for problem in problems):
            failures.append(f"checker 档包里标了 ignore 没登记的用例应当报缺登记，实际 {problems}")
        if not any("full_in_harness" in problem and "singlefs-harness 包里" in problem for problem in problems):
            failures.append(f"写在 harness 包里的崩溃枚举用例应当报住错了包，实际 {problems}")
        if not any("full_through_a_helper" in problem for problem in problems):
            failures.append(f"harness 包里经同文件 helper 调全量枚举的用例应当报住错了包，实际 {problems}")
        for swept in ("sweeps_every_prefix", "crash_image_on_each_boundary", "through_a_named_sweep"):
            if not any(swept in problem for problem in problems):
                failures.append(f"harness 包里自己逐个造崩溃状态的 {swept} 应当报住错了包，实际 {problems}")
        if any("one_fixed_crash_state_stays_in_the_harness" in problem for problem in problems):
            failures.append(f"只造一个固定崩溃态的用例不该判成枚举，实际 {problems}")
        open(os.path.join(harness_tests_directory, "granular.rs"), "w").write(
            "#[test]\nfn several_scenarios() {\n    for size in [1, 2] {\n        let pool = build_pool(\"x\");\n    }\n}\n"
            "\n// harness-test-granularity:one-scenario 同一个池按次序覆盖写，几轮合起来是一个场景\n#[test]\nfn one_scenario_waived() {\n    for seed in [1, 2] {\n        let pool = build_pool(\"y\");\n    }\n}\n")
        open(os.path.join(harness_tests_directory, "second_transaction_step_one_sample.rs"), "w").write("#[test]\nfn sample() {}\n")
        open(os.path.join(tests_directory, "module_mismatch.rs"), "w").write("//! checker 档模块：crash\nuse singlefs_checker_tier::layer0_progress::X;\n")
        open(os.path.join(tests_directory, "no_declaration.rs"), "w").write("use singlefs_checker_tier::crash::Y;\n")
        _count, scenario = one_scenario_problems(work)
        if not any("several_scenarios" in problem for problem in scenario):
            failures.append(f"循环里每轮新建池的 harness 用例应当报出来，实际 {scenario}")
        if any("one_scenario_waived" in problem for problem in scenario):
            failures.append(f"写了 one-scenario 豁免的不该报，实际 {scenario}")
        _count, modules = module_declaration_problems(work)
        if not any("module_mismatch.rs" in problem and "layer0_progress" in problem for problem in modules):
            failures.append(f"声明与导入对不上的 checker 档测试文件应当报出来，实际 {modules}")
        if not any("no_declaration.rs" in problem for problem in modules):
            failures.append(f"第一行没声明模块的 checker 档测试文件应当报出来，实际 {modules}")
        _count, file_names = test_file_name_problems(work)
        if not any("second_transaction_step_one_sample" in problem for problem in file_names):
            failures.append(f"按里程碑起名的测试文件应当报出来，实际 {file_names}")
        only_file_names = judge(work, ["file-names"])
        if list(only_file_names) != ["file-names"]:
            failures.append(f"--only file-names 应当只判 file-names 一样（这棵树上 placement、modules、one-scenario 都有问题，不该报出来），实际判了 {list(only_file_names)}")
        if parse_only("placement,modules") != ["placement", "modules"] or parse_only("placement,no-such-item") is not None:
            failures.append("--only 应当认出 placement,modules、认不出 no-such-item")
        checked = len(SELFTEST_CASES)
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print(f"  ✗ 自检 {checked} 种里有 {len(failures)} 处判错")  # gate-lint:summary
            print("  → 看 crash_enumeration_tests() / problems_in() / judge() 的判法；CRASH_CASE_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ crash-case-check 自检通过：checker 档包里登记了的放行、不标 ignore 的快档放行，注明不是用例的不算，标了 ignore 没登记的报缺登记，"
              "写在 harness 包里的（直接调或经同文件 helper 调、自己逐个造崩溃状态）报住错了包，每轮新建池的、没声明模块的、按里程碑起名的各报出来，"
              f"--only 只判点名的那几样（查了 {checked} 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    items = list(ITEMS)
    root = REPOSITORY_ROOT
    rest = list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument == "--only" or argument.startswith("--only="):
            spec = argument[len("--only="):] if argument.startswith("--only=") else (rest.pop(0) if rest else "")
            parsed = parse_only(spec)
            if parsed is None:
                print(f"  ✗ --only 后面的「{spec}」里有认不出的项")
                print(f"  → 怎么办：项是 {'、'.join(ITEMS)} 之一，用逗号隔开：crash-case-check.py --only placement,modules [仓根]")
                return 2
            items = parsed
        elif argument.startswith("-"):
            print(f"  ✗ 认不出的参数 {argument}")
            print("  → 怎么办：只认 --only <项,…>、--selftest 与一个仓根：crash-case-check.py [--only <项,…>] [仓根]")
            return 2
        else:
            root = os.path.abspath(argument)
    return run(root, items)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
