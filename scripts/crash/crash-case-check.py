#!/usr/bin/env python3
# admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
"""测试源码与崩溃点测试登记表的六样静态判据，判法只在这一份里；三道门禁各调各的那几样：

  placement     崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记进崩溃点重放合表（checker-tier-crash-point-replay 开跑前判）
  modules       checker 档每个测试文件第一行声明它测哪几个模块（checker-tier-crash-point-replay 开跑前判）
  file-names    测试文件名不按里程碑、步号起（code-source-discipline 的一格）
  one-scenario  harness 档一条用例一个场景（harness-model-differential-and-scenarios 的一格）
  must-red      每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标（checker-tier-crash-point-replay 的 crash-case-must-red 格）
  knobs         崩溃点测试的旋钮登记表每行取值与依据不空、出处路径在（checker-tier-crash-point-replay 的 crash-knobs 格）

用法：
    crash-case-check.py [--only <项>[,<项>…]] [仓根]     # 不给 --only 就六样都判
    crash-case-check.py --selftest                      # CRASH_CASE_CHECK_BREAK=<项> 时必须判红

判法：
  placement：
  ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）与 crates/*/src/**/*.rs（库与装置二进制里 #[cfg(test)] 的内联测试）
     里每个带 #[test] 的函数：函数体里调了 enumerate_crash_states 一族（名字以 enumerate_crash_states 起头的函数，快档 quick_tier 那几个除外），
     或自己逐个造崩溃状态（名字带 every_crash；或某个 for 循环体里对录制操作取到循环变量为止的前缀去 apply、或造 CrashImage），
     或调了同一文件里一个（传递地）这样做的函数，算崩溃枚举用例；只造一个固定崩溃态的不算；
  ② 它要住在 crates/singlefs-checker-tier/tests/（checker 档，D13 已定项 15，.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）：
     写在别的包（harness 这类日常测试包）里判红，那会让 cargo test 那个包的人跑到全量枚举；
  ③ checker 档包里标了 #[ignore] 的（全量那条）要登记进崩溃点重放：崩溃点重放合表 .claude/gate.d/crash-point-replay-items.tsv（入口
     scripts/crash/crash-point-replay.sh 照它跑）有一项点名的重型部分落到 <测试目标>:<这个函数名>（测试目标是文件名去掉 .rs）——
     点名的 crash-case: 行（.claude/gate.d/stage-inputs.tsv 里键是 crash-case:<名>、第三列 test=singlefs-checker-tier:<测试目标>:<函数名>），
     或点名的放量分项（scripts/crash/crash-amplification.sh 的 ITEMS 里那一行的测试目标与用例函数）。读法在 scripts/crash/lib_crash_point_replay_items.py
     的 registered_functions。标了 ignore 又没登记的入口不跑，谁都不跑；
     不标 #[ignore] 的是 checker 档的快档（小流、几步的合成流），随 cargo test -p singlefs-checker-tier 跑，不要求登记。
  ④ 合表本身核得过（同一份核法 lib_crash_point_replay_items.py 的 problems：每条 crash-case: 与每个放量分项恰好被一项点名、
     轻型的测试目标取自同一项的重型部分，写法在合表表头）；有问题逐处报成 placement 的问题。
  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：写明为什么不算用例。
  modules：crates/singlefs-checker-tier/tests/*.rs 第一行「//! checker 档模块：<模块，按 TIER_MODULES 的次序用、隔开>」与它从
     singlefs_checker_tier:: 导入的模块逐个相同；一个都不导入的写「无（为什么）」。
  file-names：crates/singlefs-harness/tests/*.rs 与 crates/singlefs-checker-tier/tests/*.rs 的文件名不带里程碑序数、步号、增补号、
     并行线号、欠账号（MILESTONE_FILE_NAME）。
  one-scenario：crates/singlefs-harness/tests/*.rs 里带 #[test] 的函数，函数体里某个 for 循环体每一轮新建一个池
     （FRESH_POOL_CALL），判红；用例上面写了「harness-test-granularity:one-scenario <理由>」的不判。
  must-red（K5，出口 4「每条流至少一个必红世界」，按崩溃枚举用例数）：.claude/gate.d/stage-inputs.tsv 每一行 crash-case:<名>（test=<包>:<测试目标>:<函数>）
     在 crates/mutations.tsv 里至少有一行变异，cargo test 的参数那一列带 -p <包>（或 --package <包>）与 --test <测试目标>；变异行归到用例只靠测试目标反推，
     同一个测试目标里的几条用例共用点名它的行。缺的逐条列、判红。这一样只判「在册」，变异真把那条用例改红由门禁 checker-tier-crates-mutation-replay 跑整表证。
     变异表不在、或一条 crash-case: 行都没有，记无对象可判。
  knobs（K7，出口 4「旋钮登记表带取值依据」）：.claude/gate.d/crash-knobs.tsv 一行一个旋钮「<旋钮><TAB><取值><TAB><依据><TAB><出处路径>」，
     四列都不许空（依据凭感觉取的写「推的」），出处路径（相对仓根）要在，旋钮名不许重；表不在记无对象可判。扫值不在这一样（六·4 的扫值产物另做）。
判不了的：变异行点名的测试目标对、可那一行改坏的不是这条用例那条流走到的代码（must-red 只认在册）；旋钮的取值与出处里写的是不是同一个数（knobs 只认出处在）；
  经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（成功行里写明是按直接调用认的）；
  一条用例里不建池的多场景（改的是同一个池）；名字里按里程碑起、却不是 MILESTONE_FILE_NAME 认得的那几种写法。
每一样各报一段：有问题的逐处列、再一句汇总带出路；没问题的一句成功行带查了多少；那一样一个对象都没有的报「本次无对象可判」。
退出码：0 选中的都齐（至少一样判过）；1 有缺的（逐个列出）；2 用法错或读不了登记表；77 选中的每一样都无对象可判。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头；
每一项红时都另有汇总行「✗ 自检 14 种里有 N 处判错」）：
  CRASH_CASE_CHECK_BREAK=fresh-pool-loop-unseen  one_scenario_problems() 不报循环里每轮新建池的用例
      循环里每轮新建池（「✗ 自检：循环里每轮新建池的 harness 用例应当报出来」）
  CRASH_CASE_CHECK_BREAK=helpers-unseen  enumerating_helpers() 不认同一文件里调全量枚举的 helper
      经同文件 helper（「✗ 自检：harness 包里经同文件 helper 调全量枚举的用例应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：harness 包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=ignore-unseen  crash_enumeration_tests() 不认 #[ignore]
      标了 ignore 没登记（「✗ 自检：checker 档包里标了 ignore 没登记的用例应当报缺登记」）
  CRASH_CASE_CHECK_BREAK=milestone-names-unseen  test_file_name_problems() 不报按里程碑起名的测试文件
      按里程碑起名（「✗ 自检：按里程碑起名的测试文件应当报出来」）
  CRASH_CASE_CHECK_BREAK=module-declaration-unchecked  module_declaration_problems() 不比第一行声明的模块与导入的模块
      声明与导入对不上（「✗ 自检：声明与导入对不上的 checker 档测试文件应当报出来」）、
      writable_mount_phases 的次序（「✗ 自检：writable_mount_phases 排在 crash_injection 之前的声明应当报出来」）、
      crash_given_states 的次序（「✗ 自检：crash_given_states 排在 writable_mount_phases 之前的声明应当报出来」）
  CRASH_CASE_CHECK_BREAK=module-declaration-scans-strings  module_declaration_problems() 把字符串字面量里的模块路径也算成导入
      字符串字面量提了没真导入的模块（「✗ 自检：字符串字面量里提过的模块不该算成真的导入」）
  CRASH_CASE_CHECK_BREAK=writable-mount-phases-unlisted  TIER_MODULES 里去掉 writable_mount_phases（模块表缺了这一项）
      声明了它、也导入了它的测试文件（「✗ 自检：声明并导入 writable_mount_phases、次序对的 checker 档测试文件不该报」）
  CRASH_CASE_CHECK_BREAK=given-states-unlisted  TIER_MODULES 里去掉 crash_given_states（模块表缺了这一项）
      声明了它、也导入了它的测试文件（「✗ 自检：声明并导入 crash_given_states、次序对的 checker 档测试文件不该报」）
  CRASH_CASE_CHECK_BREAK=only-ignored  judge() 不认 --only，四样都判
      --only file-names（「✗ 自检：--only file-names 应当只判 file-names 一样」）
  CRASH_CASE_CHECK_BREAK=package-unchecked  problems_in() 不判崩溃枚举用例住在哪个包
      写在 harness 包里（「✗ 自检：写在 harness 包里的崩溃枚举用例应当报住错了包」）、
      经同文件 helper（「✗ 自检：harness 包里经同文件 helper 调全量枚举的用例应当报住错了包」）、
      sweeps_every_prefix（「✗ 自检：harness 包里自己逐个造崩溃状态的 sweeps_every_prefix 应当报住错了包」）、
      crash_image_on_each_boundary（「✗ 自检：harness 包里自己逐个造崩溃状态的 crash_image_on_each_boundary 应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：harness 包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=quick-tier-counts  crash_enumeration_tests() 把快档 quick_tier 那几个也算成全量枚举
      干净的样本（「✗ 自检：干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题」）
  CRASH_CASE_CHECK_BREAK=rows-ignored  problems_in() 不判标了 #[ignore] 的有没有登记成 crash-case
      标了 ignore 没登记（「✗ 自检：checker 档包里标了 ignore 没登记的用例应当报缺登记」）
  CRASH_CASE_CHECK_BREAK=self-sweep-unseen  sweeps_crash_states_itself() 不认自己逐个造崩溃状态的用例
      sweeps_every_prefix（「✗ 自检：harness 包里自己逐个造崩溃状态的 sweeps_every_prefix 应当报住错了包」）、
      crash_image_on_each_boundary（「✗ 自检：harness 包里自己逐个造崩溃状态的 crash_image_on_each_boundary 应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：harness 包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=waiver-ignored  crash_enumeration_tests() 不认「crash-case-check:not-a-crash-case」的注明
      干净的样本（「✗ 自检：干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题」）
  CRASH_CASE_CHECK_BREAK=crash-case-rows-only  problems_in() 照改前只认 stage-inputs.tsv 的 crash-case: 行，不认合表点名的放量分项
      合表点名的放量分项（「✗ 自检：标了 ignore、只由合表点名的放量分项登记的用例不该报缺登记」）
  CRASH_CASE_CHECK_BREAK=items-table-unchecked  problems_in() 不报合表的问题（④）
      合表没点名的 crash-case（「✗ 自检：登记了 crash-case: 却没被合表点名，placement 应当报出来」）
  CRASH_CASE_CHECK_BREAK=must-red-unchecked  must_red_problems() 不报一行变异都没有的崩溃枚举用例
      必红世界缺的用例（「✗ 自检：一行变异都没有的崩溃枚举用例应当报出来」）、必红世界包对不上（「✗ 自检：变异行点名的是别的包里同名的测试目标，不该算」）
  CRASH_CASE_CHECK_BREAK=must-red-package-ignored  must_red_problems() 只认 --test <测试目标>、不看 -p <包>
      必红世界包对不上（「✗ 自检：变异行点名的是别的包里同名的测试目标，不该算」）
  CRASH_CASE_CHECK_BREAK=knob-source-unchecked  knob_problems() 不核出处路径在不在
      旋钮登记表出处不在（「✗ 自检：出处路径不在的旋钮应当报出来」）
  CRASH_CASE_CHECK_BREAK=knob-basis-unchecked  knob_problems() 不核依据空不空
      旋钮登记表依据空着（「✗ 自检：依据空着的旋钮应当报出来」）
  合表核法自己的开关 CRASH_POINT_REPLAY_ITEMS_BREAK（表在 scripts/crash/lib_crash_point_replay_items.py 文件头）也由这份自证证红：
  每一格判错打「✗ 自检：合表核法 <格名>：…」，汇总行同上。
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
# 崩溃点重放合表的读法与核法只在这一份库里（入口 scripts/crash/crash-point-replay.sh 用同一份）
sys.path.insert(0, SCRIPT_DIRECTORY)
import lib_crash_point_replay_items as replay_items  # noqa: E402

CHECKER_PACKAGE = "singlefs-checker-tier"
HARNESS_PACKAGE = "singlefs-harness"
ITEMS = {
    "placement": "崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记进崩溃点重放合表",
    "modules": "checker 档每个测试文件第一行声明它测哪几个模块",
    "file-names": "测试文件名不按里程碑、步号起",
    "one-scenario": "harness 档一条用例一个场景",
    "must-red": "每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标",
    "knobs": "崩溃点测试的旋钮登记表每行取值与依据不空、出处路径在",
}
ENUMERATION_CALL = re.compile(r"\b(enumerate_crash_states\w*)\s*(?:::<[^>]*>)?\s*\(")
TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")
FOR_LOOP = re.compile(r"\bfor\s+\(?\s*(?:mut\s+)?([a-z_]\w*)[^{;]*?\bin\b")
SWEEP_BY_NAME = re.compile(r"every_crash")
CRASH_CASE_ROW = re.compile(r"^crash-case:[^\t]+\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
CRASH_CASE_KEYED_ROW = re.compile(r"^(crash-case:[^\t]+)\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
MUTATIONS_TABLE = os.path.join("crates", "mutations.tsv")
KNOBS_TABLE = os.path.join(".claude", "gate.d", "crash-knobs.tsv")


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
    if BROKEN == "crash-case-rows-only":
        registered_targets = {(target, function) for _, target, function in registered_cases(root)}
    else:
        registered_targets = replay_items.registered_functions(root)
    tests = crash_enumeration_tests(root)
    problems = []
    for package_directory, target, function, ignored, where in tests:
        if package_directory != CHECKER_PACKAGE and BROKEN != "package-unchecked":
            problems.append(f"{where} {function}：直接调全量崩溃枚举，却写在 {package_directory} 包里（崩溃枚举用例要住在 crates/{CHECKER_PACKAGE}/tests/）")
            continue
        if ignored and (target, function) not in registered_targets and BROKEN != "rows-ignored":
            problems.append(f"{where} {function}：标了 #[ignore] 却没有登记成 crash-case、由崩溃点重放合表点名（.claude/gate.d/stage-inputs.tsv 里没有 "
                            f"test={CHECKER_PACKAGE}:{target}:{function} 的 crash-case: 行，或那一行与放量分项都没被 {replay_items.ITEMS_TABLE} 的一项点名），"
                            "入口 scripts/crash/crash-point-replay.sh 不跑它，谁都不跑它")
    if BROKEN != "items-table-unchecked":
        problems += [f"崩溃点重放合表：{problem}" for problem in replay_items.problems(root)]
    return tests, problems


FRESH_POOL_CALL = re.compile(r"\b(?:build_pool\w*|build_through_\w+|format_pool)\s*\(|\bMemoryPool::with_devices\s*\(")
TIER_MODULES = ["crash", "crash_state_enumeration", "crash_point_replay_progress", "crash_injection", "writable_mount_phases", "crash_given_states", "bad_disk_input", "device_log", "on_device_modes", "crash_identity", "crash_library", "crash_amplification", "verdict_store", "gpu_unit_checks",
    "crash_facts",
    "crash_verify_gpu",
    "crash_judge_tables",
    "crash_judge_gpu",
    "crash_judge_dispatch",
]
if BROKEN == "writable-mount-phases-unlisted":
    TIER_MODULES.remove("writable_mount_phases")
if BROKEN == "given-states-unlisted":
    TIER_MODULES.remove("crash_given_states")
MODULE_DECLARATION = re.compile(r"^//! checker 档模块：(.+)$")
# 字符串字面量：扫「从 singlefs_checker_tier:: 导入了什么」之前先挖掉，不然字符串里提它（例如某个绕开
# 入口的调用整行抄进字符串常量里当样本）会被认成真的导入。只认转义写法的普通双引号串，原始字符串
# （r"…"、r#"…"#）今天 tests/ 下没有真用到；带 re.DOTALL 是因为字符串里的 `\` 续行会跨行，`.` 不开
# DOTALL 吃不到换行符，没开会让这一条字符串往后吞掉一大段真代码（现判过：crash_enumeration_multi_
# record_publish_stream.rs 的续行字符串就踩过这一下）。
STRING_LITERAL = re.compile(r'"(?:\\.|[^"\\])*"', re.DOTALL)
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
        scan_text = text if BROKEN == "module-declaration-scans-strings" else STRING_LITERAL.sub("", text)
        used = sorted({name for name in re.findall(r"singlefs_checker_tier::(\w+)", scan_text) if name in TIER_MODULES}, key=TIER_MODULES.index)
        relative = os.path.relpath(path, root)
        if not declared:
            problems.append(f"{relative}：第一行没有「//! checker 档模块：…」（它导入的是 {'、'.join(used) or '无'}）")
            continue
        text_declared = declared.group(1).strip()
        declared_modules = [] if text_declared.startswith("无") else [name.strip() for name in text_declared.split("、")]
        if declared_modules != used and BROKEN != "module-declaration-unchecked":
            problems.append(f"{relative}：第一行声明 {text_declared}，导入的 checker 档模块是 {'、'.join(used) or '无'}")
    return tier_files, problems


def crash_case_rows(root):
    """stage-inputs.tsv 的 crash-case: 行 → [(键, 包, 测试目标, 函数)]，照登记的次序；读不了抛 OSError。"""
    found = []
    for line in open(os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv"), encoding="utf-8"):
        match = CRASH_CASE_KEYED_ROW.match(line)
        if match:
            found.append(match.groups())
    return found


def mutation_targets(root):
    """crates/mutations.tsv 每一行变异点名的（包, 测试目标）→ 点名它的变异名；表不在交 None。"""
    path = os.path.join(root, MUTATIONS_TABLE)
    if not os.path.isfile(path):
        return None
    named = {}
    for line in open(path, encoding="utf-8"):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.rstrip("\n").split("\t")
        if len(columns) < 5:
            continue
        words = columns[4].split()
        packages, targets = set(), set()
        for index, word in enumerate(words):
            following = words[index + 1] if index + 1 < len(words) else ""
            if word in ("-p", "--package"):
                packages.add(following)
            elif word.startswith("--package="):
                packages.add(word[len("--package="):])
            elif word == "--test":
                targets.add(following)
            elif word.startswith("--test="):
                targets.add(word[len("--test="):])
        if BROKEN == "must-red-package-ignored":
            packages = {"*"}
        for package in packages:
            for target in targets:
                named.setdefault((package, target), []).append(columns[0])
    return named


def must_red_problems(root):
    """must-red：→ (崩溃枚举用例数, 问题)。变异表不在交 (0, [])。"""
    named = mutation_targets(root)
    if named is None:
        return 0, []
    rows = crash_case_rows(root)
    problems = []
    for key, package, target, function in rows:
        if named.get((package, target)) or named.get(("*", target)):
            continue
        if BROKEN != "must-red-unchecked":
            problems.append(f"{key}（{package}:{target}:{function}）：{MUTATIONS_TABLE} 里没有一行变异的 cargo test 参数带 -p {package} --test {target}，这条流没有必红世界")
    return len(rows), problems


def knob_problems(root):
    """knobs：→ (旋钮数, 问题)。表不在交 (0, [])。"""
    path = os.path.join(root, KNOBS_TABLE)
    if not os.path.isfile(path):
        return 0, []
    problems, names, count = [], set(), 0
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        count += 1
        columns = [column.strip() for column in line.rstrip("\n").split("\t")]
        while columns and columns[-1].startswith("#"):
            columns.pop()
        where = f"{KNOBS_TABLE}:{number}"
        if len(columns) != 4:
            problems.append(f"{where} 不是四列「<旋钮><TAB><取值><TAB><依据><TAB><出处路径>」（这一行 {len(columns)} 列）")
            continue
        knob, value, basis, source = columns
        if not knob or not value:
            problems.append(f"{where} 旋钮名或取值空着")
        if not basis and BROKEN != "knob-basis-unchecked":
            problems.append(f"{where} {knob} 的依据空着：写出处（谁哪天定的、哪份产物量的），凭感觉取的写「推的」")
        if knob in names:
            problems.append(f"{where} 旋钮 {knob} 登记了两回")
        names.add(knob)
        if not source or (not os.path.exists(os.path.join(root, source)) and BROKEN != "knob-source-unchecked"):
            problems.append(f"{where} {knob} 的出处路径 {source or '（空）'} 不在仓里")
    return count, problems


def judge(root, items):
    """{项: (对象数, 问题, 成功行)}，按 items 的次序；读不了登记表时抛 OSError。"""
    if BROKEN == "only-ignored":
        items = list(ITEMS)
    results = {}
    for item in items:
        if item == "placement":
            scanned = len(placement_source_paths(root))
            tests, problems = problems_in(root)
            table_items, _ = replay_items.read_items(root)
            results[item] = (scanned, problems,
                             f"崩溃枚举用例都住在 {CHECKER_PACKAGE} 包里、标了 #[ignore] 的都登记进了崩溃点重放合表（扫了 {scanned} 份测试源码，认出 {len(tests)} 条；"
                             f"合表 {len(table_items)} 项核过；"
                             "按测试函数直接调 enumerate_crash_states 一族、自己逐个造崩溃状态、或经同一文件里的函数传递地这样做认；"
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
        elif item == "must-red":
            cases, problems = must_red_problems(root)
            results[item] = (cases, problems, f"{cases} 条崩溃枚举用例在 {MUTATIONS_TABLE} 里都有点名它的测试目标的变异行（只判在册，证红归 checker-tier-crates-mutation-replay）")
        elif item == "knobs":
            knobs, problems = knob_problems(root)
            results[item] = (knobs, problems, f"旋钮登记表 {KNOBS_TABLE} 的 {knobs} 个旋钮都写了取值与依据、出处路径都在")
    return results


REMEDIES = {
    "placement": [f"把那个测试文件挪进 crates/{CHECKER_PACKAGE}/tests/（共用模块经 #[path = \"../../singlefs-harness/tests/common/mod.rs\"] mod common; 指回去），",
                  "全量那条标 #[ignore] 并把 crash-case:<名> 那一行（第三列 test=singlefs-checker-tier:<测试目标>:<函数名> 与 count-line 等）登记进",
                  ".claude/gate.d/stage-inputs.tsv，再放进崩溃点重放合表 .claude/gate.d/crash-point-replay-items.tsv 它那条流的那一项（写法在合表表头）；",
                  "合表报的问题照那一句改；小流的快档不标 ignore、不用登记（.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）。"],
    "modules": ["checker 档每个测试文件第一行写「//! checker 档模块：<它导入的模块，按 " + "、".join(TIER_MODULES) + " 的次序用、隔开>」，",
                "一个都不导入的写「无（为什么）」（.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）。"],
    "file-names": ["测试文件按它测什么起名：领域在前、场景在后，不带里程碑、步号、增补号、并行线号、欠账号，来历进 commit message 或 .claude/kb/",
                   "（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）；改名照 .claude/rules/path-moves.md「怎么做」改全仓引用。"],
    "one-scenario": ["harness 档一条用例只装一个场景：循环体挪进带参数的函数，每个取值一条 #[test]，红了只重跑那一条；",
                     "确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 // harness-test-granularity:one-scenario <理由>",
                     "（.claude/rules/verification.md「harness 档里再分快档用例与慢档用例」）。"],
    "must-red": [f"给那条用例补一行变异进 {MUTATIONS_TABLE}（只追加）：改坏那条流走到的实现或判器，cargo test 的参数写 -p <包> --test <测试目标> -- <必红的测试> --exact，",
                 "末列写那条必红的测试名；补完由主 agent 照 .claude/skills/crash-test/commit-stages.md 证红（必红的测试在 checker 档，子 agent 跑不了）。"],
    "knobs": [f"照 {KNOBS_TABLE} 表头补齐那一行：取值写今天的值，依据写出处（谁哪天定的、哪份产物量的）或「推的」，出处路径写这个旋钮住的那份文件（相对仓根）。"],
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
                  "固定崩溃态不算枚举", "每轮新建池与豁免", "模块声明缺与对不上", "writable_mount_phases 的次序（排在 crash_injection 之后）", "crash_given_states 的次序（排在 writable_mount_phases 之后、bad_disk_input 之前）", "字符串字面量里的模块路径不算真导入", "按里程碑起名", "--only 只判点名的",
                  "合表点名的放量分项算登记", "合表没点名的 crash-case 报出来", "合表核法（lib_crash_point_replay_items.py 的 selftest_cells）",
                  "必红世界有与缺", "必红世界包对不上", "旋钮登记表齐与缺"]


def placement_world(work, name, stage_rows, table_rows, pipeline_rows):
    """placement 的一个小仓：checker 档 tests/streamy.rs 里一条标了 ignore、调全量枚举的用例 streamy_full；stage-inputs.tsv、合表、放量 ITEMS 照给的写。"""
    root = os.path.join(work, name)
    os.makedirs(os.path.join(root, "crates", CHECKER_PACKAGE, "tests"))
    os.makedirs(os.path.join(root, ".claude", "gate.d"))
    os.makedirs(os.path.join(root, "scripts"))
    os.makedirs(os.path.join(root, "scripts", "crash"), exist_ok=True)
    open(os.path.join(root, "crates", CHECKER_PACKAGE, "tests", "streamy.rs"), "w").write(
        "#[test]\n#[ignore]\nfn streamy_full() {\n    enumerate_crash_states(&stream);\n}\n")
    open(os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv"), "w").write("# 样本\n" + "".join(row + "\t# 样本\n" for row in stage_rows))
    open(os.path.join(root, ".claude", "gate.d", "crash-point-replay-items.tsv"), "w").write("# 样本\n" + "".join(row + "\t# 样本\n" for row in table_rows))
    open(os.path.join(root, "scripts", "crash", "crash-amplification.sh"), "w").write(
        "ITEMS=(\n" + "".join(f"  $'{row}\\t全量'\n" for row in pipeline_rows) + ")\n")
    return root


def selftest():
    work =tempfile.mkdtemp(prefix="crash-case-check-selftest-")
    try:
        tests_directory = os.path.join(work, "crates", CHECKER_PACKAGE, "tests")
        harness_tests_directory = os.path.join(work, "crates", HARNESS_PACKAGE, "tests")
        os.makedirs(tests_directory)
        os.makedirs(harness_tests_directory)
        os.makedirs(os.path.join(work, ".claude", "gate.d"))
        open(os.path.join(tests_directory, "plain.rs"), "w").write(
            "#[test]\n#[ignore]\nfn registered_case() {\n    let report = enumerate_crash_states_versions(&stream);\n}\n\n"
            "#[test]\nfn quick_case() {\n    enumerate_crash_states_quick_tier_versions(&stream);\n}\n\n"
            "#[test]\nfn small_stream_in_the_quick_tier() {\n    enumerate_crash_states_selecting(&small, 1);\n}\n\n"
            "#[test]\nfn unrelated() {\n    assert_eq!(1, 1);\n}\n")
        open(os.path.join(tests_directory, "synthetic.rs"), "w").write(
            "// crash-case-check:not-a-crash-case 三步的合成流，单跑不到一秒\n#[test]\nfn tiny_stream() {\n    enumerate_crash_states(&tiny);\n}\n")
        open(os.path.join(harness_tests_directory, "daily.rs"), "w").write("#[test]\nfn daily_quick() {\n    enumerate_crash_states_quick_tier_versions(&s);\n}\n")
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "w").write(
            f"# 样本\ncrash-case:demo\tcrates/\ttest={CHECKER_PACKAGE}:plain:registered_case count-line=x\t#样本\n")
        open(os.path.join(work, ".claude", "gate.d", "crash-point-replay-items.tsv"), "w").write("# 样本合表\ndemo\tcrash-case:demo\t-\tplain\t# 样本\n")
        failures = []
        # 合表点名的放量分项也算登记（ITEMS 里那一行的测试目标与用例函数）；合表没点名的 crash-case: 照合表核法报出来
        through_pipeline = placement_world(work, "through-pipeline", [], ["streamy\t-\tstreamy-piped\tstreamy"],
                                                ["streamy-piped\\tstreamy\\tstreamy_full"])
        _, problems = problems_in(through_pipeline)
        if problems:
            failures.append(f"标了 ignore、只由合表点名的放量分项登记的用例不该报缺登记，实际 {problems}")
        orphan = placement_world(work, "orphan-crash-case",
                                 [f"crash-case:streamy\tcrates/\ttest={CHECKER_PACKAGE}:streamy:streamy_full",
                                  f"crash-case:orphan\tcrates/\ttest={CHECKER_PACKAGE}:streamy:orphan_full"],
                                 ["streamy\tcrash-case:streamy\t-\tstreamy"], [])
        _, problems = problems_in(orphan)
        if not any("crash-case:orphan 没有被合表里哪一项点名" in problem for problem in problems):
            failures.append(f"登记了 crash-case: 却没被合表点名，placement 应当报出来，实际 {problems}")
        table_cells, table_failures = replay_items.selftest_cells()
        for cell_name, failure in table_failures:
            failures.append(f"合表核法 {cell_name}：{failure}")
        tests, problems = problems_in(work)
        if len(tests) != 2 or problems:
            failures.append(f"干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题（checker 档包里不标 ignore 的快档放行），实际 {len(tests)} 条、{problems}")
        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\n#[ignore]\nfn forgot_to_register() {\n    enumerate_crash_states_selecting(&s, 1);\n}\n")
        open(os.path.join(harness_tests_directory, "daily.rs"), "a").write("\n#[test]\nfn full_in_harness() {\n    enumerate_crash_states(&s);\n}\n"
            "\nfn prepare_and_enumerate(s: &S) {\n    enumerate_crash_states_selecting(s, 1);\n}\n\n#[test]\nfn full_through_a_helper() {\n    prepare_and_enumerate(&s);\n}\n"
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
        open(os.path.join(tests_directory, "module_mismatch.rs"), "w").write("//! checker 档模块：crash\nuse singlefs_checker_tier::crash_point_replay_progress::X;\n")
        open(os.path.join(tests_directory, "no_declaration.rs"), "w").write("use singlefs_checker_tier::crash::Y;\n")
        open(os.path.join(tests_directory, "string_literal_names_a_module.rs"), "w").write(
            "//! checker 档模块：无（只读这个包的源码文本，不调它的任何模块）\n"
            "const SAMPLE_LINE: &str = \"singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&every_card)\";\n")
        open(os.path.join(tests_directory, "writable_phases_in_order.rs"), "w").write(
            "//! checker 档模块：crash、crash_injection、writable_mount_phases\nuse singlefs_checker_tier::crash::A;\n"
            "use singlefs_checker_tier::crash_injection::B;\nuse singlefs_checker_tier::writable_mount_phases::C;\n")
        open(os.path.join(tests_directory, "writable_phases_out_of_order.rs"), "w").write(
            "//! checker 档模块：crash、writable_mount_phases、crash_injection\nuse singlefs_checker_tier::crash::A;\n"
            "use singlefs_checker_tier::crash_injection::B;\nuse singlefs_checker_tier::writable_mount_phases::C;\n")
        open(os.path.join(tests_directory, "given_states_in_order.rs"), "w").write(
            "//! checker 档模块：crash_injection、crash_given_states、bad_disk_input\nuse singlefs_checker_tier::crash_injection::B;\n"
            "use singlefs_checker_tier::crash_given_states::D;\nuse singlefs_checker_tier::bad_disk_input::E;\n")
        open(os.path.join(tests_directory, "given_states_out_of_order.rs"), "w").write(
            "//! checker 档模块：crash_given_states、writable_mount_phases\nuse singlefs_checker_tier::writable_mount_phases::C;\n"
            "use singlefs_checker_tier::crash_given_states::D;\n")
        _count, scenario = one_scenario_problems(work)
        if not any("several_scenarios" in problem for problem in scenario):
            failures.append(f"循环里每轮新建池的 harness 用例应当报出来，实际 {scenario}")
        if any("one_scenario_waived" in problem for problem in scenario):
            failures.append(f"写了 one-scenario 豁免的不该报，实际 {scenario}")
        _count, modules = module_declaration_problems(work)
        if not any("module_mismatch.rs" in problem and "crash_point_replay_progress" in problem for problem in modules):
            failures.append(f"声明与导入对不上的 checker 档测试文件应当报出来，实际 {modules}")
        if not any("no_declaration.rs" in problem for problem in modules):
            failures.append(f"第一行没声明模块的 checker 档测试文件应当报出来，实际 {modules}")
        if any("writable_phases_in_order.rs" in problem for problem in modules):
            failures.append(f"声明并导入 writable_mount_phases、次序对的 checker 档测试文件不该报，实际 {modules}")
        if not any("writable_phases_out_of_order.rs" in problem for problem in modules):
            failures.append(f"writable_mount_phases 排在 crash_injection 之前的声明应当报出来，实际 {modules}")
        if any("given_states_in_order.rs" in problem for problem in modules):
            failures.append(f"声明并导入 crash_given_states、次序对的 checker 档测试文件不该报，实际 {modules}")
        if not any("given_states_out_of_order.rs" in problem for problem in modules):
            failures.append(f"crash_given_states 排在 writable_mount_phases 之前的声明应当报出来，实际 {modules}")
        if any("string_literal_names_a_module.rs" in problem for problem in modules):
            failures.append(f"字符串字面量里提过的模块不该算成真的导入，实际 {modules}")
        _count, file_names = test_file_name_problems(work)
        if not any("second_transaction_step_one_sample" in problem for problem in file_names):
            failures.append(f"按里程碑起名的测试文件应当报出来，实际 {file_names}")
        only_file_names = judge(work, ["file-names"])
        if list(only_file_names) != ["file-names"]:
            failures.append(f"--only file-names 应当只判 file-names 一样（这棵树上 placement、modules、one-scenario 都有问题，不该报出来），实际判了 {list(only_file_names)}")
        if parse_only("placement,modules") != ["placement", "modules"] or parse_only("placement,no-such-item") is not None:
            failures.append("--only 应当认出 placement,modules、认不出 no-such-item")
        # must-red：两条用例，一条有点名它测试目标的变异行，一条只有别的包里同名测试目标的行，一条一行都没有
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "a").write(
            f"crash-case:covered\tcrates/\ttest={CHECKER_PACKAGE}:covered_stream:covered_full\t#样本\n"
            f"crash-case:other-package\tcrates/\ttest={CHECKER_PACKAGE}:shared_name:shared_full\t#样本\n")
        open(os.path.join(work, "crates", "mutations.tsv"), "w").write(
            "# 样本变异表\n"
            f"改坏一处\tcrates/singlefs-core/src/a.rs\tx\ty\t-p {CHECKER_PACKAGE} --test covered_stream -- covered_quick --exact\tcovered_quick\n"
            f"别的包\tcrates/singlefs-core/src/a.rs\tx\tz\t-p singlefs-harness --test shared_name -- t --exact\tt\n")
        cases, red_problems = must_red_problems(work)
        if not any("crash-case:demo" in problem for problem in red_problems) or any("crash-case:covered" in problem for problem in red_problems) or cases != 3:
            failures.append(f"一行变异都没有的崩溃枚举用例应当报出来（crash-case:demo 报、crash-case:covered 不报，查 3 条），实际 {cases} 条、{red_problems}")
        if not any("crash-case:other-package" in problem for problem in red_problems):
            failures.append(f"变异行点名的是别的包里同名的测试目标，不该算（crash-case:other-package 应当报出来），实际 {red_problems}")
        os.makedirs(os.path.join(work, "scripts", "crash"), exist_ok=True)
        open(os.path.join(work, "scripts", "crash", "knob-home.sh"), "w").write("LIGHT_EXPAND_UP_TO=6\n")
        open(os.path.join(work, KNOBS_TABLE), "w").write(
            "# 样本旋钮表\n展开上限（轻型）\t6\t推的\tscripts/crash/knob-home.sh\n"
            "线程数\t31\t\tscripts/crash/knob-home.sh\n批大小\t65536\t推的\tscripts/crash/no-such-file.sh\n")
        knobs, knob_found = knob_problems(work)
        if knobs != 3 or any("展开上限" in problem for problem in knob_found):
            failures.append(f"写全了的旋钮不该报（查 3 个），实际 {knobs} 个、{knob_found}")
        if not any("线程数 的依据空着" in problem for problem in knob_found):
            failures.append(f"依据空着的旋钮应当报出来，实际 {knob_found}")
        if not any("批大小 的出处路径" in problem for problem in knob_found):
            failures.append(f"出处路径不在的旋钮应当报出来，实际 {knob_found}")
        checked = len(SELFTEST_CASES)
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print(f"  ✗ 自检 {checked} 种里有 {len(failures)} 处判错")  # gate-lint:summary
            print("  → 看 crash_enumeration_tests() / problems_in() / judge() 的判法；CRASH_CASE_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ crash-case-check 自检通过：checker 档包里登记了的放行、不标 ignore 的快档放行，注明不是用例的不算，标了 ignore 没登记的报缺登记，"
              "写在 harness 包里的（直接调或经同文件 helper 调、自己逐个造崩溃状态）报住错了包，每轮新建池的、没声明模块的、按里程碑起名的各报出来，"
              "合表点名的放量分项算登记、合表没点名的 crash-case 报出来，没有必红变异的崩溃枚举用例与依据空着、出处不在的旋钮各报出来，"
              f"--only 只判点名的那几样（查了 {checked} 种；合表核法 {len(table_cells)} 格）")
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
