#!/usr/bin/env python3
# admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
"""测试源码与崩溃点测试登记表的八样静态判据，判法只在这一份里；三道门禁各调各的那几样：

  placement     崩溃枚举用例住验证包、都登记进崩溃点重放合表、不标 #[ignore]（门禁 crash-point-replay 开跑前判）
  modules       验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块（门禁 crash-point-replay 开跑前判）
  file-names    测试文件名不按里程碑、步号起（code-source-discipline 的一格）
  test-names    #[test] 函数名不按里程碑、步号、欠账号起（code-source-discipline 的 test-function-names 格）
  one-scenario  验证包一条用例一个场景（model-differential-and-scenarios 的一格）
  must-red      每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标（crash-point-replay 的 crash-case-must-red 格）
  knobs         崩溃点测试的旋钮登记表每行取值与依据不空、出处路径在（crash-point-replay 的 crash-knobs 格）
  provenance    崩溃点重放合表的崩溃枚举项与放量分项在来历表 PROVENANCE_OF_THE_FLOWS 里都有一行（还没有门禁调它；主 agent 照实现员报告交 --only provenance）

用法：
    crash-case-check.py [--only <项>[,<项>…]] [仓根]     # 不给 --only 就八样都判
    crash-case-check.py --selftest                      # CRASH_CASE_CHECK_BREAK=<项> 时必须判红

判法：
  placement：
  ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）与 crates/*/src/**/*.rs（库与装置二进制里 #[cfg(test)] 的内联测试）
     里每个带 #[test] 的函数：函数体里调了 enumerate_crash_states 一族（名字以 enumerate_crash_states 起头的函数，名字里带什么都算），
     或自己逐个造崩溃状态（名字带 every_crash；或某个 for 循环体里对录制操作取到循环变量为止的前缀去 apply、或造 CrashImage），
     或调了同一文件里一个（传递地）这样做的函数，算崩溃枚举用例；只造一个固定崩溃态的不算；
  ② 它要住在 crates/singlefs-verification/tests/（验证包，.claude/rules/verification.md「门禁管哪一半」崩溃枚举用例住在验证包那一行）：
     写在别的包（singlefs-core 这类）里判红，那会让 cargo test 那个包的人跑到全量枚举；
  ③ 验证包里的崩溃枚举用例都要登记进崩溃点重放：崩溃点重放合表 .claude/gate.d/crash-point-replay-items.tsv（入口
     scripts/crash/crash-point-replay.sh 照它跑）有一项点名的部分落到 <测试目标>:<这个函数名>（测试目标是文件名去掉 .rs）——
     点名的 crash-case: 行（.claude/gate.d/stage-inputs.tsv 里键是 crash-case:<名>、第三列 test=singlefs-verification:<测试目标>:<函数名>），
     或点名的放量分项（scripts/crash/crash-amplification.sh 的 ITEMS 里那一行的测试目标与用例函数）。读法在 scripts/crash/lib_crash_point_replay_items.py
     的 registered_functions。没登记的入口不跑，谁都不跑；
     用例不标 #[ignore]：不带 --ignored 的 cargo test 就是它的跑法（门禁与入口按条复用那一趟用 --skip 把登记的用例排掉，交按块那一路），标了 ignore 的平常不跑，判红。
  ④ 合表本身核得过（同一份核法 lib_crash_point_replay_items.py 的 problems：每条 crash-case: 与每个放量分项恰好被一项点名，写法在合表表头）；
     有问题逐处报成 placement 的问题。
  ⑤ 验证包 crates/singlefs-verification/src/ 下的内联测试（库与装置二进制里的 #[cfg(test)]）不要求登记：崩溃枚举用例的跑法是
     cargo test --test <测试目标>，点不到内联测试，它们跟着门禁的测试函数那一段按条复用地跑；别的包 src/ 下的内联测试照 ② 报住错了包。
  豁免（确实不是崩溃用例，只录流不枚举、只建判表、两份造法对拍这类）只有一种写法：测试函数上面的注释
  「crash-case-check:not-a-crash-case <理由>」，理由至少 4 个字；没有豁免表。
  modules：crates/singlefs-verification/tests/*.rs 里代码（去掉字符串字面量与 // 注释）从 singlefs_verification:: 导入了 CRASH_ENGINE_MODULES 里模块的、
     与第一行写了声明的，第一行「//! 验证包模块：<模块，按 CRASH_ENGINE_MODULES 的次序用、隔开>」与它从
     singlefs_verification:: 导入的模块逐个相同；一个都不导入的写「无（为什么）」。
  file-names：crates/singlefs-verification/tests/*.rs 与 crates/singlefs-verification/tests/*.rs 的文件名不带里程碑序数、步号、增补号、
     并行线号、欠账号（MILESTONE_FILE_NAME）。
  test-names（C638）：crates/*/tests/ 下（含子目录）与 crates/*/src/ 下 .rs 里带 #[test] 的函数名，拿同一条 MILESTONE_FILE_NAME 判：
     带里程碑序数、步号、增补号、并行线号、欠账号（c<三位数>）的判红，来历进 commit message 或 .claude/kb/。
     立账时已在的存量登记在 .claude/gate.d/test-function-names-exempt.tsv（一行「<相对仓根的文件><TAB><函数名><TAB># 理由>」，理由至少 4 个字），
     登记了的不判；一行在那份文件里一条带欠账号的 #[test] 都对不上（函数改了名、删了、文件不在）判红：那一行删掉。表只缩不涨（不判涨，靠 review）。
     带实验号（e<数>）与 m<数>_ 的不在 MILESTONE_FILE_NAME 里，不判（算不算来历没定）。
  one-scenario：crates/singlefs-verification/tests/*.rs 里带 #[test] 的函数，
     函数体里某个 for 循环体每一轮新建一个池（FRESH_POOL_CALL），判红；用例上面写了「harness-test-granularity:one-scenario <理由>」的不判。
  must-red（K5，出口 4「每条流至少一个必红世界」，按崩溃枚举用例数）：.claude/gate.d/stage-inputs.tsv 每一行 crash-case:<名>（test=<包>:<测试目标>:<函数>）
     在 crates/mutations.tsv 里至少有一行变异，cargo test 的参数那一列带 -p <包>（或 --package <包>）与 --test <测试目标>；变异行归到用例只靠测试目标反推，
     同一个测试目标里的几条用例共用点名它的行。缺的逐条列、判红。这一样只判「在册」，变异真把那条用例改红由门禁 crates-mutation-replay 跑整表证。
     变异表不在、或一条 crash-case: 行都没有，记无对象可判。
  knobs（K7，出口 4「旋钮登记表带取值依据」）：.claude/gate.d/crash-knobs.tsv 一行一个旋钮「<旋钮><TAB><取值><TAB><依据><TAB><出处路径>」，
     四列都不许空（依据凭感觉取的写「推的」），出处路径（相对仓根）要在，旋钮名不许重；表不在记无对象可判。扫值不在这一样（六·4 的扫值产物另做）。
  provenance（拆分提案第一百三十二节）：崩溃点重放合表 .claude/gate.d/crash-point-replay-items.tsv（读法 lib_crash_point_replay_items.py 的 read_items）每一项：
     第二列点名的 crash-case: 键里有一条的测试目标文件（crates/singlefs-verification/tests/<测试目标>.rs，测试目标照 stage-inputs.tsv 那一行的 test=）
     写着 enumerate_crash_states 的，来历表 crates/singlefs-verification/src/crash_point_replay_provenance.rs（PROVENANCE_SOURCE，去掉 // 注释之后读）里
     要有一行 enumeration_of_the_item("<流名>", "<这一项的项名>", …)；第三列点名的每个放量分项要有一行 amplification_of_the_item("<流名>", "<分项名>", …)。
     缺的逐条列、判红；与验证包的 crash_points_observer_registration_is_complete 用例判同一件事（那条在提交时才跑，这里读源码、不起 cargo）。
     来历表或合表不在，记无对象可判。管不到：来历表那一行写的来历对不对、流名对不对（归那条用例与观察者）。
判不了的：变异行点名的测试目标对、可那一行改坏的不是这条用例那条流走到的代码（must-red 只认在册）；旋钮的取值与出处里写的是不是同一个数（knobs 只认出处在）；
  经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（成功行里写明是按直接调用认的）；
  一条用例里不建池的多场景（改的是同一个池）；名字里按里程碑起、却不是 MILESTONE_FILE_NAME 认得的那几种写法。
每一样各报一段：有问题的逐处列、再一句汇总带出路；没问题的一句成功行带查了多少；那一样一个对象都没有的报「本次无对象可判」。
退出码：0 选中的都齐（至少一样判过）；1 有缺的（逐个列出）；2 用法错或读不了登记表；77 选中的每一样都无对象可判。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头；
每一项红时都另有汇总行「✗ 自检 24 种里有 N 处判错」）：
  CRASH_CASE_CHECK_BREAK=fresh-pool-loop-unseen  one_scenario_problems() 不报循环里每轮新建池的用例
      循环里每轮新建池（「✗ 自检：循环里每轮新建池的用例应当报出来」）、
      按参数循环（「✗ 自检：按参数循环、每轮新建池的用例应当报出来」）
  CRASH_CASE_CHECK_BREAK=test-names-unseen  test_function_name_problems() 不报带欠账号的 #[test] 函数名
      用例名带欠账号（「✗ 自检：带欠账号的 #[test] 函数名应当报出来」）
  CRASH_CASE_CHECK_BREAK=test-names-exempt-stale-unseen  test_function_name_problems() 不报对不上的豁免行
      用例名豁免过期（「✗ 自检：一条用例都对不上的豁免行应当报出来」）
  CRASH_CASE_CHECK_BREAK=helpers-unseen  enumerating_helpers() 不认同一文件里调全量枚举的 helper
      经同文件 helper（「✗ 自检：别的包里经同文件 helper 调全量枚举的用例应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：别的包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=ignore-unseen  crash_enumeration_tests() 不认 #[ignore]（标了 ignore 的照放）
      登记的用例标了 ignore（「✗ 自检：登记了的崩溃枚举用例标了 ignore 应当报出来」）
  CRASH_CASE_CHECK_BREAK=inline-tests-require-rows  problems_in() 对 验证包 src/ 下的内联测试也要求登记
      内联测试（「✗ 自检：验证包 src/ 下的内联测试不要求登记（0 处问题）」）
  CRASH_CASE_CHECK_BREAK=milestone-names-unseen  test_file_name_problems() 不报按里程碑起名的测试文件
      按里程碑起名（「✗ 自检：按里程碑起名的测试文件应当报出来」）
  CRASH_CASE_CHECK_BREAK=module-declaration-unchecked  module_declaration_problems() 不比第一行声明的模块与导入的模块
      声明与导入对不上（「✗ 自检：声明与导入对不上的测试文件应当报出来」）、
      writable_mount_phases 的次序（「✗ 自检：writable_mount_phases 排在 crash_injection 之前的声明应当报出来」）、
      crash_given_states 的次序（「✗ 自检：crash_given_states 排在 writable_mount_phases 之前的声明应当报出来」）
  CRASH_CASE_CHECK_BREAK=module-declaration-scans-strings  module_declaration_problems() 把字符串字面量里的模块路径也算成导入
      字符串字面量提了没真导入的模块（「✗ 自检：字符串字面量里提过的模块不该算成真的导入」）
  CRASH_CASE_CHECK_BREAK=module-declaration-required-everywhere  module_declaration_problems() 对只用脚手架模块、没写声明的测试文件也要求声明
      只用脚手架模块（「✗ 自检：只用脚手架模块、注释里提到崩溃引擎模块、没写声明的测试文件不该报」）
  CRASH_CASE_CHECK_BREAK=writable-mount-phases-unlisted  CRASH_ENGINE_MODULES 里去掉 writable_mount_phases（模块表缺了这一项）
      声明了它、也导入了它的测试文件（「✗ 自检：声明并导入 writable_mount_phases、次序对的测试文件不该报」）
  CRASH_CASE_CHECK_BREAK=given-states-unlisted  CRASH_ENGINE_MODULES 里去掉 crash_given_states（模块表缺了这一项）
      声明了它、也导入了它的测试文件（「✗ 自检：声明并导入 crash_given_states、次序对的测试文件不该报」）
  CRASH_CASE_CHECK_BREAK=only-ignored  judge() 不认 --only，四样都判
      --only file-names（「✗ 自检：--only file-names 应当只判 file-names 一样」）
  CRASH_CASE_CHECK_BREAK=package-unchecked  problems_in() 不判崩溃枚举用例住在哪个包
      写在别的包里（「✗ 自检：写在别的包里的崩溃枚举用例应当报住错了包」）、
      经同文件 helper（「✗ 自检：别的包里经同文件 helper 调全量枚举的用例应当报住错了包」）、
      sweeps_every_prefix（「✗ 自检：别的包里自己逐个造崩溃状态的 sweeps_every_prefix 应当报住错了包」）、
      crash_image_on_each_boundary（「✗ 自检：别的包里自己逐个造崩溃状态的 crash_image_on_each_boundary 应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：别的包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）、
      别的包 src/ 下的内联测试（「✗ 自检：别的包 src/ 下的内联测试调枚举函数应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=rows-ignored  problems_in() 不判调枚举函数的用例有没有登记成 crash-case
      调枚举函数没登记（「✗ 自检：验证包里调枚举函数没登记的用例应当报缺登记」）
  CRASH_CASE_CHECK_BREAK=self-sweep-unseen  sweeps_crash_states_itself() 不认自己逐个造崩溃状态的用例
      sweeps_every_prefix（「✗ 自检：别的包里自己逐个造崩溃状态的 sweeps_every_prefix 应当报住错了包」）、
      crash_image_on_each_boundary（「✗ 自检：别的包里自己逐个造崩溃状态的 crash_image_on_each_boundary 应当报住错了包」）、
      through_a_named_sweep（「✗ 自检：别的包里自己逐个造崩溃状态的 through_a_named_sweep 应当报住错了包」）
  CRASH_CASE_CHECK_BREAK=waiver-ignored  crash_enumeration_tests() 不认「crash-case-check:not-a-crash-case」的注明
      干净的样本（「✗ 自检：干净的样本应当认出 1 条（注明不是用例的那条不算）、0 处问题」）
  CRASH_CASE_CHECK_BREAK=crash-case-rows-only  problems_in() 只认 stage-inputs.tsv 的 crash-case: 行，不认合表点名的放量分项
      合表点名的放量分项（「✗ 自检：只由合表点名的放量分项登记的用例不该报缺登记」）、
      登记了的用例标了 ignore（「✗ 自检：登记了的崩溃枚举用例标了 ignore 应当报出来」：那条用例只由放量分项登记，开关下判成没登记，报的是缺登记、不是标了 ignore）
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
  CRASH_CASE_CHECK_BREAK=provenance-unchecked  provenance_problems() 不报来历表里没有行的项与分项
      来历表核对（「✗ 自检：合表多一项而来历表没有它的 enumeration_of_the_item 行，应当报出来」、
      「✗ 自检：放量分项在来历表里没有 amplification_of_the_item 行，应当报出来」）
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

VERIFICATION_PACKAGE = "singlefs-verification"
ITEMS = {
    "placement": "崩溃枚举用例住验证包、都登记进崩溃点重放合表、不标 #[ignore]",
    "modules": "验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块",
    "file-names": "测试文件名不按里程碑、步号起",
    "test-names": "#[test] 函数名不按里程碑、步号、欠账号起",
    "one-scenario": "验证包一条用例一个场景",
    "must-red": "每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标",
    "knobs": "崩溃点测试的旋钮登记表每行取值与依据不空、出处路径在",
    "provenance": "崩溃点重放合表的崩溃枚举项与放量分项在来历表里都有一行",
}
ENUMERATION_CALL = re.compile(r"\b(enumerate_crash_states\w*)\s*(?:::<[^>]*>)?\s*\(")
TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")
FOR_LOOP = re.compile(r"\bfor\s+\(?\s*(?:mut\s+)?([a-z_]\w*)[^{;]*?\bin\b")
SWEEP_BY_NAME = re.compile(r"every_crash")
CRASH_CASE_ROW = re.compile(r"^crash-case:[^\t]+\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
CRASH_CASE_KEYED_ROW = re.compile(r"^(crash-case:[^\t]+)\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
MUTATIONS_TABLE = os.path.join("crates", "mutations.tsv")
KNOBS_TABLE = os.path.join(".claude", "gate.d", "crash-knobs.tsv")
TEST_NAMES_EXEMPT = os.path.join(".claude", "gate.d", "test-function-names-exempt.tsv")
PROVENANCE_SOURCE = os.path.join("crates", VERIFICATION_PACKAGE, "src", "crash_point_replay_provenance.rs")
PROVENANCE_ENUMERATION_ROW = re.compile(r'\benumeration_of_the_item\(\s*"[^"]*"\s*,\s*"([^"]*)"')
PROVENANCE_AMPLIFICATION_ROW = re.compile(r'\bamplification_of_the_item\(\s*"[^"]*"\s*,\s*"([^"]*)"')


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
                   if ENUMERATION_CALL.findall(body) or sweeps_crash_states_itself(name, body)}
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
            calls = ENUMERATION_CALL.findall(body)
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


def is_inline_test_of_the_verification_package(where):
    """where（相对仓根的路径:行号）是 验证包 src/ 下的内联测试：库与装置二进制里的 #[cfg(test)] 测试。
    崩溃枚举用例的跑法是 cargo test --test <测试目标>，点不到内联测试，所以既登记不了也不要求登记。"""
    return where.startswith(f"crates/{VERIFICATION_PACKAGE}/src/")


def problems_in(root):
    """placement：→ (认出的崩溃枚举用例, 问题)。读不了登记表抛 OSError。"""
    if BROKEN == "crash-case-rows-only":
        registered_targets = {(target, function) for _, target, function in registered_cases(root)}
    else:
        registered_targets = replay_items.registered_functions(root)
    tests = crash_enumeration_tests(root)
    problems = []
    for package_directory, target, function, ignored, where in tests:
        if package_directory == VERIFICATION_PACKAGE and is_inline_test_of_the_verification_package(where) and BROKEN != "inline-tests-require-rows":
            continue
        if package_directory != VERIFICATION_PACKAGE and BROKEN != "package-unchecked":
            problems.append(f"{where} {function}：直接调全量崩溃枚举，却写在 {package_directory} 包里（崩溃枚举用例要住在 crates/{VERIFICATION_PACKAGE}/tests/）")
            continue
        if (target, function) not in registered_targets and BROKEN != "rows-ignored":
            problems.append(f"{where} {function}：调了崩溃枚举一族的函数却没有登记成 crash-case、由崩溃点重放合表点名（.claude/gate.d/stage-inputs.tsv 里没有 "
                            f"test={VERIFICATION_PACKAGE}:{target}:{function} 的 crash-case: 行，或那一行与放量分项都没被 {replay_items.ITEMS_TABLE} 的一项点名），"
                            "入口 scripts/crash/crash-point-replay.sh 不跑它，谁都不跑它")
        elif ignored:
            problems.append(f"{where} {function}：登记成 crash-case 的用例标了 #[ignore]：崩溃枚举用例不分档，不带 --ignored 的 cargo test 就是它的跑法，"
                            "标了 ignore 平常谁都不跑它（去掉 #[ignore]；门禁与入口按条复用那一趟用 --skip 把它排掉，交按块那一路）")
    if BROKEN != "items-table-unchecked":
        problems += [f"崩溃点重放合表：{problem}" for problem in replay_items.problems(root)]
    return tests, problems


FRESH_POOL_CALL = re.compile(r"\b(?:build_pool\w*|build_through_\w+|format_pool)\s*\(|\bMemoryPool::with_devices\s*\(")
CRASH_ENGINE_MODULES = ["crash", "crash_state_enumeration", "crash_point_replay_progress", "crash_injection", "writable_mount_phases", "crash_given_states", "bad_disk_input", "device_log", "on_device_modes", "crash_identity", "crash_library", "crash_amplification", "verdict_store", "gpu_unit_checks",
    "crash_facts",
    "crash_verify_gpu",
    "crash_judge_tables",
    "crash_judge_gpu",
    "crash_judge_dispatch",
]
if BROKEN == "writable-mount-phases-unlisted":
    CRASH_ENGINE_MODULES.remove("writable_mount_phases")
if BROKEN == "given-states-unlisted":
    CRASH_ENGINE_MODULES.remove("crash_given_states")
MODULE_DECLARATION = re.compile(r"^//! 验证包模块：(.+)$")
# 字符串字面量：扫「从 singlefs_verification:: 导入了什么」之前先挖掉，不然字符串里提它（例如某个绕开
# 入口的调用整行抄进字符串常量里当样本）会被认成真的导入。只认转义写法的普通双引号串，原始字符串
# （r"…"、r#"…"#）今天 tests/ 下没有真用到；带 re.DOTALL 是因为字符串里的 `\` 续行会跨行，`.` 不开
# DOTALL 吃不到换行符，没开会让这一条字符串往后吞掉一大段真代码（现判过：crash_enumeration_multi_
# record_publish_stream.rs 的续行字符串就踩过这一下）。
STRING_LITERAL = re.compile(r'"(?:\\.|[^"\\])*"', re.DOTALL)
# 按里程碑、步号、增补号、并行线号、欠账号起的测试文件名：名字说的是它在哪一步写出来的，不是它测什么
MILESTONE_FILE_NAME = re.compile(r"(^|_)(first|second|third)_transaction(_|$)|_step_(zero|one|two|three|four|five|six|seven|eight|nine)(_|$)"
                                 r"|(^|_)supplement_(one|two|three|four)(_|$)|(^|_)parallel_line_(one|two|three)(_|$)|(^|_)c[0-9]{3}(_|$)")


def verification_test_paths(root):
    return sorted(glob.glob(os.path.join(root, "crates", VERIFICATION_PACKAGE, "tests", "*.rs")))


def one_scenario_problems(root):
    """one-scenario：→ (查了几条验证包用例, 问题)。"""
    problems, counted = [], 0
    for path in verification_test_paths(root):
        text = open(path, encoding="utf-8", errors="replace").read()
        for match in TEST_FUNCTION.finditer(text):
            attributes = match.group(1)
            if "#[test]" not in attributes:
                continue
            counted += 1
            if "harness-test-granularity:one-scenario" in attributes:
                continue
            body = function_body(text, match.end())
            for loop in FOR_LOOP.finditer(body):
                if FRESH_POOL_CALL.search(function_body(body, loop.end())) and BROKEN != "fresh-pool-loop-unseen":
                    line = text.count("\n", 0, match.start(2)) + 1
                    problems.append(f"{os.path.relpath(path, root)}:{line} {match.group(2)}：循环里每一轮新建一个池，是一条用例装了几个独立场景")
                    break
    return counted, problems


def test_function_paths(root):
    """test-names 扫的源码：crates/*/tests/ 下（含子目录）与 crates/*/src/ 下的 .rs，去重、按路径排。"""
    paths = glob.glob(os.path.join(root, "crates", "*", "tests", "**", "*.rs"), recursive=True)
    paths += glob.glob(os.path.join(root, "crates", "*", "src", "**", "*.rs"), recursive=True)
    return sorted(set(paths))


def exemption_rows(root, table):
    """豁免表（<相对仓根的文件><TAB><函数名><TAB># 理由，理由至少 4 个字）→ ({(文件, 函数名): 行号}, 写坏的行)；表不在交 ({}, [])。"""
    path = os.path.join(root, table)
    if not os.path.isfile(path):
        return {}, []
    rows, malformed = {}, []
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.rstrip("\n").split("\t")
        reason = columns[2].lstrip("#").strip() if len(columns) >= 3 else ""
        if len(columns) != 3 or not columns[2].startswith("#") or len(reason) < 4 or not columns[0] or not columns[1]:
            malformed.append(f"{table}:{number} 不是「<相对仓根的文件><TAB><函数名><TAB># 理由>」（理由至少 4 个字）")
            continue
        rows[(columns[0], columns[1])] = number
    return rows, malformed


def test_function_name_exemptions(root):
    return exemption_rows(root, TEST_NAMES_EXEMPT)


def test_function_name_problems(root):
    """test-names：→ (查了几条 #[test] 函数, 豁免了几条, 问题)。"""
    exempt, problems = test_function_name_exemptions(root)
    used, tests, exempted = set(), 0, 0
    for path in test_function_paths(root):
        relative = os.path.relpath(path, root)
        text = open(path, encoding="utf-8", errors="replace").read()
        for match in TEST_FUNCTION.finditer(text):
            if "#[test]" not in match.group(1):
                continue
            tests += 1
            name = match.group(2)
            if not MILESTONE_FILE_NAME.search(name):
                continue
            if (relative, name) in exempt:
                used.add((relative, name))
                exempted += 1
                continue
            if BROKEN != "test-names-unseen":
                line = text.count("\n", 0, match.start(2)) + 1
                problems.append(f"{relative}:{line} {name}：#[test] 函数名按里程碑、步号或欠账号起，改成它测什么，来历进 commit message 或 .claude/kb/")
    if BROKEN != "test-names-exempt-stale-unseen":
        for (relative, name), number in sorted(exempt.items(), key=lambda item: item[1]):
            if (relative, name) not in used:
                problems.append(f"{TEST_NAMES_EXEMPT}:{number} {relative} {name}：那份文件里没有这条带欠账号的 #[test]（改了名、删了或文件不在），这一行删掉")
    return tests, exempted, problems


def test_file_name_problems(root):
    """file-names：→ (查了几份测试文件, 问题)。"""
    problems = []
    paths = verification_test_paths(root)
    for path in paths:
        stem = os.path.basename(path)[:-len(".rs")]
        if MILESTONE_FILE_NAME.search(stem) and BROKEN != "milestone-names-unseen":
            problems.append(f"{os.path.relpath(path, root)}：文件名按里程碑、步号、增补号或欠账号起（{stem}），改成它测什么：领域在前、场景在后")
    return len(paths), problems


def module_declaration_problems(root):
    """modules：→ (查了几份验证包测试文件, 要声明的几份, 问题)。导入（去掉字符串字面量与 // 注释之后的代码里 singlefs_verification::<模块>）了
    CRASH_ENGINE_MODULES 里模块的、或第一行写了声明的要声明、要对得上；两样都没有的（只用 model、memory_pool 这类脚手架模块的）不判。"""
    problems, scanned_files, declaring_files = [], 0, 0
    for path in verification_test_paths(root):
        scanned_files += 1
        text = open(path, encoding="utf-8", errors="replace").read()
        declared = MODULE_DECLARATION.match(text.split("\n", 1)[0])
        scan_text = text if BROKEN == "module-declaration-scans-strings" else STRING_LITERAL.sub("", text)
        used = sorted({name for name in re.findall(r"singlefs_verification::(\w+)", scan_text) if name in CRASH_ENGINE_MODULES}, key=CRASH_ENGINE_MODULES.index)
        code_text = re.sub(r"//[^\n]*", "", scan_text)
        imported_in_code = any(name in CRASH_ENGINE_MODULES for name in re.findall(r"singlefs_verification::(\w+)", code_text))
        if not declared and not imported_in_code and BROKEN != "module-declaration-required-everywhere":
            continue
        declaring_files += 1
        relative = os.path.relpath(path, root)
        if not declared:
            problems.append(f"{relative}：第一行没有「//! 验证包模块：…」（它导入的是 {'、'.join(used) or '无'}）")
            continue
        text_declared = declared.group(1).strip()
        declared_modules = [] if text_declared.startswith("无") else [name.strip() for name in text_declared.split("、")]
        if declared_modules != used and BROKEN != "module-declaration-unchecked":
            problems.append(f"{relative}：第一行声明 {text_declared}，导入的崩溃引擎模块是 {'、'.join(used) or '无'}")
    return scanned_files, declaring_files, problems


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


def provenance_problems(root):
    """provenance：→ (核了几处登记, 问题)。来历表或合表不在交 (0, [])；stage-inputs.tsv 读不了抛 OSError。"""
    source_path = os.path.join(root, PROVENANCE_SOURCE)
    if not os.path.isfile(source_path) or not os.path.isfile(os.path.join(root, replay_items.ITEMS_TABLE)):
        return 0, []
    source = re.sub(r"//[^\n]*", "", open(source_path, encoding="utf-8", errors="replace").read())
    enumerated = set(PROVENANCE_ENUMERATION_ROW.findall(source))
    amplified = set(PROVENANCE_AMPLIFICATION_ROW.findall(source))
    items, _table_problems = replay_items.read_items(root)
    cases = replay_items.crash_case_rows(root)
    checked, problems = 0, []
    for item in items:
        where = f"{replay_items.ITEMS_TABLE}:{item.line_number}"
        targets = [cases[key][0] for key in item.crash_cases if key in cases]
        runs_the_enumeration = False
        for target in targets:
            test_path = os.path.join(root, "crates", VERIFICATION_PACKAGE, "tests", f"{target}.rs")
            if os.path.isfile(test_path) and "enumerate_crash_states" in open(test_path, encoding="utf-8", errors="replace").read():
                runs_the_enumeration = True
        if runs_the_enumeration:
            checked += 1
            if item.name not in enumerated and BROKEN != "provenance-unchecked":
                problems.append(f"{where} 项 {item.name}：它的崩溃枚举用例调 enumerate_crash_states，来历表 {PROVENANCE_SOURCE} 的 PROVENANCE_OF_THE_FLOWS 里"
                                f"没有 enumeration_of_the_item(\"<流名>\", \"{item.name}\", …) 这一行（跑的时候在录任何东西之前就拒）")
        for pipeline_item in item.pipeline_items:
            checked += 1
            if pipeline_item not in amplified and BROKEN != "provenance-unchecked":
                problems.append(f"{where} 放量分项 {pipeline_item}：来历表 {PROVENANCE_SOURCE} 的 PROVENANCE_OF_THE_FLOWS 里"
                                f"没有 amplification_of_the_item(\"<流名>\", \"{pipeline_item}\", …) 这一行")
    return checked, problems


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
            inline_count = sum(1 for test in tests if test[0] == VERIFICATION_PACKAGE and is_inline_test_of_the_verification_package(test[4]))
            results[item] = (scanned, problems,
                             f"崩溃枚举用例都住在 {VERIFICATION_PACKAGE} 包里、都登记进了崩溃点重放合表、没有标 #[ignore] 的（扫了 {scanned} 份测试源码，认出 {len(tests)} 条，"
                             f"其中 src/ 下的内联测试 {inline_count} 条点不到、不要求登记，源码注释注明不是崩溃用例的不算；"
                             f"合表 {len(table_items)} 项核过；"
                             "按测试函数直接调 enumerate_crash_states 一族、自己逐个造崩溃状态、或经同一文件里的函数传递地这样做认；"
                             "经 tests/<子目录>/ 里的共用函数间接调的、宏展开出来的认不出）")
        elif item == "modules":
            scanned_files, declaring_files, problems = module_declaration_problems(root)
            results[item] = (scanned_files, problems, f"验证包 {scanned_files} 份测试文件里导入崩溃引擎模块或写了声明的 {declaring_files} 份都声明了测哪几个模块、与导入对得上"
                                                      f"（另 {scanned_files - declaring_files} 份只用脚手架模块、没写声明，不判）")
        elif item == "file-names":
            test_files, problems = test_file_name_problems(root)
            results[item] = (test_files, problems,
                             f"查了 {test_files} 份测试文件的名字（验证包的 tests/*.rs），没有按里程碑、步号、增补号、并行线号或欠账号起的")
        elif item == "test-names":
            tests, exempted, problems = test_function_name_problems(root)
            results[item] = (tests, problems,
                             f"查了 {tests} 条 #[test] 函数的名字（crates/*/tests/ 与 crates/*/src/），没有按里程碑、步号、增补号、并行线号或欠账号起的"
                             f"（豁免表 {TEST_NAMES_EXEMPT} 登记的存量 {exempted} 条不判）")
        elif item == "one-scenario":
            scenario_tests, problems = one_scenario_problems(root)
            results[item] = (scenario_tests, problems, f"验证包 {scenario_tests} 条用例都是一条一个场景")
        elif item == "must-red":
            cases, problems = must_red_problems(root)
            results[item] = (cases, problems, f"{cases} 条崩溃枚举用例在 {MUTATIONS_TABLE} 里都有点名它的测试目标的变异行（只判在册，证红归 crates-mutation-replay）")
        elif item == "knobs":
            knobs, problems = knob_problems(root)
            results[item] = (knobs, problems, f"旋钮登记表 {KNOBS_TABLE} 的 {knobs} 个旋钮都写了取值与依据、出处路径都在")
        elif item == "provenance":
            checked, problems = provenance_problems(root)
            results[item] = (checked, problems, f"崩溃点重放合表点名的 {checked} 处（调枚举的崩溃枚举项与放量分项）在来历表 {PROVENANCE_SOURCE} 里都有一行")
    return results


REMEDIES = {
    "placement": [f"把那个测试文件挪进 crates/{VERIFICATION_PACKAGE}/tests/（共用模块照同一个目录里的 mod common; 引），",
                  "用例不标 #[ignore]，并把 crash-case:<名> 那一行（第三列 test=singlefs-verification:<测试目标>:<函数名> 与 count-line 等）登记进",
                  ".claude/gate.d/stage-inputs.tsv，再放进崩溃点重放合表 .claude/gate.d/crash-point-replay-items.tsv 它那条流的那一项（写法在合表表头）；",
                  "合表报的问题照那一句改（.claude/rules/verification.md「门禁管哪一半」崩溃枚举用例住在验证包那一行）；不调枚举函数的小流测试不用登记；",
                  "调了枚举一族函数又确实不是崩溃用例的（只录流不枚举、只建判表、两份造法对拍），在函数上面写 // crash-case-check:not-a-crash-case <理由>（理由至少 4 个字）。"],
    "modules": ["验证包里导入崩溃引擎模块的测试文件第一行写「//! 验证包模块：<它导入的模块，按 " + "、".join(CRASH_ENGINE_MODULES) + " 的次序用、隔开>」，",
                "写了声明而一个都不导入的写「无（为什么）」（.claude/rules/verification.md「门禁管哪一半」崩溃枚举用例住在验证包那一行）。"],
    "file-names": ["测试文件按它测什么起名：领域在前、场景在后，不带里程碑、步号、增补号、并行线号、欠账号，来历进 commit message 或 .claude/kb/",
                   "（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）；改名照 .claude/rules/path-moves.md「怎么做」改全仓引用。"],
    "test-names": ["#[test] 函数按它测什么起名，不带里程碑、步号、增补号、并行线号、欠账号，来历进 commit message 或 .claude/kb/",
                   "（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）；带欠账号的是被测对象的名字（例：一条判据叫 c670）的，改成说那个对象是什么；",
                   f"立账时已在、还没逐条读的存量登记在 {TEST_NAMES_EXEMPT}，表只缩不涨，改了名就删那一行。"],
    "one-scenario": ["验证包一条用例只装一个场景：循环体挪进带参数的函数，每个取值一条 #[test]，红了只重跑那一条；",
                     "确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 // harness-test-granularity:one-scenario <理由>",
                     "（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）。"],
    "must-red": [f"给那条用例补一行变异进 {MUTATIONS_TABLE}（只追加）：改坏那条流走到的实现或判器，cargo test 的参数写 -p <包> --test <测试目标> -- <必红的测试> --exact，",
                 "末列写那条必红的测试名；补完由主 agent 照 .claude/skills/crash-test/commit-stages.md 证红（必红的测试在验证包，子 agent 跑不了）。"],
    "knobs": [f"照 {KNOBS_TABLE} 表头补齐那一行：取值写今天的值，依据写出处（谁哪天定的、哪份产物量的）或「推的」，出处路径写这个旋钮住的那份文件（相对仓根）。"],
    "provenance": [f"在 {PROVENANCE_SOURCE} 的 PROVENANCE_OF_THE_FLOWS 里给那一项补一行（崩溃枚举项写 enumeration_of_the_item，放量分项写 amplification_of_the_item，",
                   "来历照那条流的基镜像怎么来的写；改 crates/ 归实现员）；合表里那一项删了或改了名的，来历表同一次改。"],
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


SELFTEST_CASES = ["登记了的放行、注明不是用例的不算", "调枚举函数没登记", "登记了的用例标了 ignore", "住错了包（直接调、经 helper、自己逐个造崩溃状态）",
                  "固定崩溃态不算枚举", "每轮新建池与豁免", "模块声明缺与对不上", "writable_mount_phases 的次序（排在 crash_injection 之后）", "crash_given_states 的次序（排在 writable_mount_phases 之后、bad_disk_input 之前）", "字符串字面量里的模块路径不算真导入", "只用脚手架模块的测试文件不要求声明", "按里程碑起名", "用例名带欠账号", "用例名豁免过期", "按参数循环", "--only 只判点名的",
                  "合表点名的放量分项算登记", "合表没点名的 crash-case 报出来", "验证包 src 内联测试不要求登记", "合表核法（lib_crash_point_replay_items.py 的 selftest_cells）",
                  "必红世界有与缺", "必红世界包对不上", "旋钮登记表齐与缺", "来历表核对（合表多一项、放量分项来历表没行、不调枚举的项不要求）"]


def placement_world(work, name, stage_rows, table_rows, pipeline_rows, streamy_attributes=""):
    """placement 的一个小仓：验证包 tests/streamy.rs 里一条调枚举一族函数的用例 streamy_full（streamy_attributes 是挂在 #[test] 后面的属性，例 #[ignore]）；
    stage-inputs.tsv、合表、放量 ITEMS 照给的写。"""
    root = os.path.join(work, name)
    os.makedirs(os.path.join(root, "crates", VERIFICATION_PACKAGE, "tests"))
    os.makedirs(os.path.join(root, ".claude", "gate.d"))
    os.makedirs(os.path.join(root, "scripts"))
    os.makedirs(os.path.join(root, "scripts", "crash"), exist_ok=True)
    open(os.path.join(root, "crates", VERIFICATION_PACKAGE, "tests", "streamy.rs"), "w").write(
        f"#[test]\n{streamy_attributes}fn streamy_full() {{\n    enumerate_crash_states(&stream);\n}}\n")
    open(os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv"), "w").write("# 样本\n" + "".join(row + "\t# 样本\n" for row in stage_rows))
    open(os.path.join(root, ".claude", "gate.d", "crash-point-replay-items.tsv"), "w").write("# 样本\n" + "".join(row + "\t# 样本\n" for row in table_rows))
    open(os.path.join(root, "scripts", "crash", "crash-amplification.sh"), "w").write(
        "ITEMS=(\n" + "".join(f"  $'{row}'\n" for row in pipeline_rows) + ")\n")
    return root


def selftest():
    work =tempfile.mkdtemp(prefix="crash-case-check-selftest-")
    try:
        tests_directory = os.path.join(work, "crates", VERIFICATION_PACKAGE, "tests")
        other_package_tests_directory = os.path.join(work, "crates", "singlefs-core", "tests")
        os.makedirs(tests_directory)
        os.makedirs(other_package_tests_directory)
        os.makedirs(os.path.join(work, ".claude", "gate.d"))
        open(os.path.join(tests_directory, "plain.rs"), "w").write(
            "#[test]\nfn registered_case() {\n    let report = enumerate_crash_states_versions(&stream);\n}\n\n"
            "#[test]\nfn unrelated() {\n    assert_eq!(1, 1);\n}\n")
        open(os.path.join(tests_directory, "synthetic.rs"), "w").write(
            "// crash-case-check:not-a-crash-case 三步的合成流，单跑不到一秒\n#[test]\nfn tiny_stream() {\n    enumerate_crash_states(&tiny);\n}\n")
        open(os.path.join(other_package_tests_directory, "daily.rs"), "w").write("#[test]\nfn daily_check() {\n    assert_eq!(1, 1);\n}\n")
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "w").write(
            f"# 样本\ncrash-case:demo\tcrates/\ttest={VERIFICATION_PACKAGE}:plain:registered_case count-line=x\t#样本\n")
        open(os.path.join(work, ".claude", "gate.d", "crash-point-replay-items.tsv"), "w").write("# 样本合表\ndemo\tcrash-case:demo\t-\t# 样本\n")
        failures = []
        # 合表点名的放量分项也算登记（ITEMS 里那一行的测试目标与用例函数）；合表没点名的 crash-case: 照合表核法报出来
        through_pipeline = placement_world(work, "through-pipeline", [], ["streamy\t-\tstreamy-piped"],
                                                ["streamy-piped\\tstreamy\\tstreamy_full"])
        _, problems = problems_in(through_pipeline)
        if problems:
            failures.append(f"只由合表点名的放量分项登记的用例不该报缺登记，实际 {problems}")
        # 登记了的用例标了 ignore：平常不跑，报出来（开关 ignore-unseen 下照放）
        ignored_registered = placement_world(work, "ignored-registered", [], ["streamy\t-\tstreamy-piped"],
                                             ["streamy-piped\\tstreamy\\tstreamy_full"], streamy_attributes="#[ignore]\n")
        _, problems = problems_in(ignored_registered)
        if not any("streamy_full" in problem and "标了 #[ignore]" in problem for problem in problems):
            failures.append(f"登记了的崩溃枚举用例标了 ignore 应当报出来，实际 {problems}")
        orphan = placement_world(work, "orphan-crash-case",
                                 [f"crash-case:streamy\tcrates/\ttest={VERIFICATION_PACKAGE}:streamy:streamy_full",
                                  f"crash-case:orphan\tcrates/\ttest={VERIFICATION_PACKAGE}:streamy:orphan_full"],
                                 ["streamy\tcrash-case:streamy\t-"], [])
        _, problems = problems_in(orphan)
        if not any("crash-case:orphan 没有被合表里哪一项点名" in problem for problem in problems):
            failures.append(f"登记了 crash-case: 却没被合表点名，placement 应当报出来，实际 {problems}")
        # 验证包 src/ 下的内联测试点不到 cargo test --test，不要求登记；别的包 src/ 下的照报住错了包
        inline_tests = placement_world(work, "inline-tests", [], [], [])
        os.remove(os.path.join(inline_tests, "crates", VERIFICATION_PACKAGE, "tests", "streamy.rs"))
        os.makedirs(os.path.join(inline_tests, "crates", VERIFICATION_PACKAGE, "src"))
        os.makedirs(os.path.join(inline_tests, "crates", "demo-core", "src"))
        open(os.path.join(inline_tests, "crates", VERIFICATION_PACKAGE, "src", "inline.rs"), "w").write(
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn inline_sweep() {\n        enumerate_crash_states(&s);\n    }\n}\n")
        _, problems = problems_in(inline_tests)
        if problems:
            failures.append(f"验证包 src/ 下的内联测试不要求登记（0 处问题），实际 {problems}")
        open(os.path.join(inline_tests, "crates", "demo-core", "src", "lib.rs"), "w").write(
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn other_package_inline() {\n        enumerate_crash_states(&s);\n    }\n}\n")
        _, problems = problems_in(inline_tests)
        if not any("other_package_inline" in problem and "demo-core 包里" in problem for problem in problems):
            failures.append(f"别的包 src/ 下的内联测试调枚举函数应当报住错了包，实际 {problems}")
        # 来历表核对：合表三项——streamy 有来历行；added-stream 是合表多出来的一项（调枚举、来历表没它的行，注释里写的那一行不算），
        # 它的放量分项 added-piped 也没行；given-states 的测试目标不调枚举，不要求行
        provenance_world = placement_world(
            work, "provenance",
            [f"crash-case:streamy\tcrates/\ttest={VERIFICATION_PACKAGE}:streamy:streamy_full",
             f"crash-case:added\tcrates/\ttest={VERIFICATION_PACKAGE}:streamy:added_full",
             f"crash-case:given\tcrates/\ttest={VERIFICATION_PACKAGE}:given_states:given_full"],
            ["streamy\tcrash-case:streamy\tstreamy-piped", "added-stream\tcrash-case:added\tadded-piped",
             "given-states\tcrash-case:given\t-"],
            ["streamy-piped\\tstreamy\\tstreamy_full", "added-piped\\tstreamy\\tadded_full"])
        open(os.path.join(provenance_world, "crates", VERIFICATION_PACKAGE, "tests", "given_states.rs"), "w").write(
            "#[test]\nfn given_full() {\n    judge_given_states(&table);\n}\n")
        os.makedirs(os.path.dirname(os.path.join(provenance_world, PROVENANCE_SOURCE)))
        open(os.path.join(provenance_world, PROVENANCE_SOURCE), "w").write(
            "pub const PROVENANCE_OF_THE_FLOWS: &[ProvenanceOfAFlow] = &[\n"
            "    enumeration_of_the_item(\"streamy\", \"streamy\", FlowProvenance::RecordedAfterTheMakeFilesystemReturned),\n"
            "    // enumeration_of_the_item(\"added\", \"added-stream\", FlowProvenance::RecordedAfterTheMakeFilesystemReturned),\n"
            "    amplification_of_the_item(\"streamy-flow\", \"streamy-piped\", FlowProvenance::RecordedAfterTheMakeFilesystemReturned),\n"
            "];\n")
        provenance_checked, provenance_found = provenance_problems(provenance_world)
        if not any("项 added-stream" in problem for problem in provenance_found):
            failures.append(f"合表多一项而来历表没有它的 enumeration_of_the_item 行，应当报出来，实际 {provenance_found}")
        if not any("放量分项 added-piped" in problem for problem in provenance_found):
            failures.append(f"放量分项在来历表里没有 amplification_of_the_item 行，应当报出来，实际 {provenance_found}")
        if provenance_checked != 4 or any("streamy：" in problem or "streamy-piped" in problem or "given-states" in problem for problem in provenance_found):
            failures.append(f"有来历行的项与分项、不调枚举的项不该报（核 4 处：streamy、streamy-piped、added-stream、added-piped），实际 {provenance_checked} 处、{provenance_found}")
        table_cells, table_failures = replay_items.selftest_cells()
        for cell_name, failure in table_failures:
            failures.append(f"合表核法 {cell_name}：{failure}")
        tests, problems = problems_in(work)
        if len(tests) != 1 or problems:
            failures.append(f"干净的样本应当认出 1 条（注明不是用例的那条不算）、0 处问题，实际 {len(tests)} 条、{problems}")
        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\nfn forgot_to_register() {\n    enumerate_crash_states_selecting(&s, 1);\n}\n")
        open(os.path.join(other_package_tests_directory, "daily.rs"), "a").write("\n#[test]\nfn full_in_the_core_package() {\n    enumerate_crash_states(&s);\n}\n"
            "\nfn prepare_and_enumerate(s: &S) {\n    enumerate_crash_states_selecting(s, 1);\n}\n\n#[test]\nfn full_through_a_helper() {\n    prepare_and_enumerate(&s);\n}\n"
            "\n#[test]\nfn sweeps_every_prefix() {\n    for prefix in before..=operations.len() {\n        let mut image = pool();\n        image.apply(&operations[..prefix]);\n    }\n}\n"
            "\n#[test]\nfn crash_image_on_each_boundary() {\n    for count in 0..=segments.len() {\n        let image = CrashImage { persisted: through(count) };\n    }\n}\n"
            "\nfn lowest_at_every_crash_point(o: &[Op]) -> u64 {\n    0\n}\n\n#[test]\nfn through_a_named_sweep() {\n    lowest_at_every_crash_point(&o);\n}\n"
            "\n#[test]\nfn one_fixed_crash_state_stays_in_the_core_package() {\n    for request in requests {\n        check(request);\n    }\n    let image = CrashImage { persisted: all() };\n    base.apply(&operations[..last_root_slot_write]);\n}\n")
        _, problems = problems_in(work)
        if not any("forgot_to_register" in problem and "没有登记" in problem for problem in problems):
            failures.append(f"验证包里调枚举函数没登记的用例应当报缺登记，实际 {problems}")
        if not any("full_in_the_core_package" in problem and "singlefs-core 包里" in problem for problem in problems):
            failures.append(f"写在别的包里的崩溃枚举用例应当报住错了包，实际 {problems}")
        if not any("full_through_a_helper" in problem and "singlefs-core 包里" in problem for problem in problems):
            failures.append(f"别的包里经同文件 helper 调全量枚举的用例应当报住错了包，实际 {problems}")
        for swept in ("sweeps_every_prefix", "crash_image_on_each_boundary", "through_a_named_sweep"):
            if not any(swept in problem and "singlefs-core 包里" in problem for problem in problems):
                failures.append(f"别的包里自己逐个造崩溃状态的 {swept} 应当报住错了包，实际 {problems}")
        if any("one_fixed_crash_state_stays_in_the_core_package" in problem for problem in problems):
            failures.append(f"只造一个固定崩溃态的用例不该判成枚举，实际 {problems}")
        open(os.path.join(tests_directory, "granular.rs"), "w").write(
            "#[test]\nfn several_scenarios() {\n    for size in [1, 2] {\n        let pool = build_pool(\"x\");\n    }\n}\n"
            "\n// harness-test-granularity:one-scenario 同一个池按次序覆盖写，几轮合起来是一个场景\n#[test]\nfn one_scenario_waived() {\n    for seed in [1, 2] {\n        let pool = build_pool(\"y\");\n    }\n}\n")
        open(os.path.join(tests_directory, "second_transaction_step_one_sample.rs"), "w").write("#[test]\nfn sample() {}\n")
        open(os.path.join(tests_directory, "module_mismatch.rs"), "w").write("//! 验证包模块：crash\nuse singlefs_verification::crash_point_replay_progress::X;\n")
        open(os.path.join(tests_directory, "no_declaration.rs"), "w").write("use singlefs_verification::crash::Y;\n")
        open(os.path.join(tests_directory, "string_literal_names_a_module.rs"), "w").write(
            "//! 验证包模块：无（只读这个包的源码文本，不调它的任何模块）\n"
            "const SAMPLE_LINE: &str = \"singlefs_verification::gpu_unit_checks::gpu_cards_by_priority(&every_card)\";\n")
        open(os.path.join(tests_directory, "writable_phases_in_order.rs"), "w").write(
            "//! 验证包模块：crash、crash_injection、writable_mount_phases\nuse singlefs_verification::crash::A;\n"
            "use singlefs_verification::crash_injection::B;\nuse singlefs_verification::writable_mount_phases::C;\n")
        open(os.path.join(tests_directory, "writable_phases_out_of_order.rs"), "w").write(
            "//! 验证包模块：crash、writable_mount_phases、crash_injection\nuse singlefs_verification::crash::A;\n"
            "use singlefs_verification::crash_injection::B;\nuse singlefs_verification::writable_mount_phases::C;\n")
        open(os.path.join(tests_directory, "given_states_in_order.rs"), "w").write(
            "//! 验证包模块：crash_injection、crash_given_states、bad_disk_input\nuse singlefs_verification::crash_injection::B;\n"
            "use singlefs_verification::crash_given_states::D;\nuse singlefs_verification::bad_disk_input::E;\n")
        open(os.path.join(tests_directory, "given_states_out_of_order.rs"), "w").write(
            "//! 验证包模块：crash_given_states、writable_mount_phases\nuse singlefs_verification::writable_mount_phases::C;\n"
            "use singlefs_verification::crash_given_states::D;\n")
        # 只用脚手架模块的测试文件：没写声明、注释里提到崩溃引擎模块，不要求声明
        open(os.path.join(tests_directory, "scaffold_only.rs"), "w").write(
            "//! 只用理想模型；`singlefs_verification::crash_amplification` 的外层循环在别处测\nuse singlefs_verification::model::Model;\n")
        # 按参数循环（C637）：一条每轮新建池的报出来，一条写了 one-scenario 豁免的不报
        open(os.path.join(tests_directory, "rollback_parameters.rs"), "w").write(
            "//! 验证包模块：无（样本：只造池）\n#[test]\nfn every_rollback_parameter() {\n    for floor in [1, 2] {\n        let pool = build_pool(\"z\");\n    }\n}\n"
            "\n// harness-test-granularity:one-scenario 同一个池按次序回退两次，合起来是一个场景\n#[test]\nfn rollback_scenario_waived() {\n"
            "    for floor in [1, 2] {\n        let pool = build_pool(\"w\");\n    }\n}\n")
        scenario_count, scenario = one_scenario_problems(work)
        if not any("several_scenarios" in problem for problem in scenario):
            failures.append(f"循环里每轮新建池的用例应当报出来，实际 {scenario}")
        if any("one_scenario_waived" in problem or "rollback_scenario_waived" in problem for problem in scenario):
            failures.append(f"写了 one-scenario 豁免的不该报，实际 {scenario}")
        if not any("rollback_parameters.rs" in problem and "every_rollback_parameter" in problem for problem in scenario):
            failures.append(f"按参数循环、每轮新建池的用例应当报出来（查了 {scenario_count} 条），实际 {scenario}")
        _scanned, _declaring, modules = module_declaration_problems(work)
        if not any("module_mismatch.rs" in problem and "crash_point_replay_progress" in problem for problem in modules):
            failures.append(f"声明与导入对不上的测试文件应当报出来，实际 {modules}")
        if not any("no_declaration.rs" in problem for problem in modules):
            failures.append(f"第一行没声明模块的测试文件应当报出来，实际 {modules}")
        if any("writable_phases_in_order.rs" in problem for problem in modules):
            failures.append(f"声明并导入 writable_mount_phases、次序对的测试文件不该报，实际 {modules}")
        if not any("writable_phases_out_of_order.rs" in problem for problem in modules):
            failures.append(f"writable_mount_phases 排在 crash_injection 之前的声明应当报出来，实际 {modules}")
        if any("given_states_in_order.rs" in problem for problem in modules):
            failures.append(f"声明并导入 crash_given_states、次序对的测试文件不该报，实际 {modules}")
        if not any("given_states_out_of_order.rs" in problem for problem in modules):
            failures.append(f"crash_given_states 排在 writable_mount_phases 之前的声明应当报出来，实际 {modules}")
        if any("scaffold_only.rs" in problem for problem in modules):
            failures.append(f"只用脚手架模块、注释里提到崩溃引擎模块、没写声明的测试文件不该报，实际 {modules}")
        if any("string_literal_names_a_module.rs" in problem for problem in modules):
            failures.append(f"字符串字面量里提过的模块不该算成真的导入，实际 {modules}")
        _count, file_names = test_file_name_problems(work)
        if not any("second_transaction_step_one_sample" in problem for problem in file_names):
            failures.append(f"按里程碑起名的测试文件应当报出来，实际 {file_names}")
        # #[test] 函数名带欠账号（C638）：src/ 里的单测与 tests/ 子目录里的都扫；登记了豁免的不报，豁免行对不上的报
        os.makedirs(os.path.join(work, "crates", "demo-core", "src"))
        os.makedirs(os.path.join(tests_directory, "common"))
        open(os.path.join(work, "crates", "demo-core", "src", "admission.rs"), "w").write(
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn c123_two_devices_refuse_one_unit() {}\n\n    #[test]\n    fn c456_listed_while_it_is_read() {}\n\n"
            "    #[test]\n    fn refuses_one_unit_when_one_slot_short() {}\n}\n")
        open(os.path.join(tests_directory, "common", "mod.rs"), "w").write("#[test]\nfn helper_checked_by_c789() {}\n")
        open(os.path.join(work, TEST_NAMES_EXEMPT), "w").write(
            "# 样本豁免表\ncrates/demo-core/src/admission.rs\tc456_listed_while_it_is_read\t# 样本：存量，待逐条读\n"
            "crates/demo-core/src/admission.rs\tc999_renamed_long_ago\t# 样本：早改了名，这一行该删\n")
        named_tests, named_exempted, names = test_function_name_problems(work)
        if not any("c123_two_devices_refuse_one_unit" in problem for problem in names) or not any("helper_checked_by_c789" in problem for problem in names):
            failures.append(f"带欠账号的 #[test] 函数名应当报出来（src/ 里的单测与 tests/ 子目录里的都扫；查了 {named_tests} 条），实际 {names}")
        if any("c456_listed_while_it_is_read：" in problem for problem in names) or named_exempted != 1:
            failures.append(f"登记了豁免的用例名不该报、豁免数应当是 1，实际豁免 {named_exempted}、{names}")
        if not any("c999_renamed_long_ago" in problem and "这一行删掉" in problem for problem in names):
            failures.append(f"一条用例都对不上的豁免行应当报出来，实际 {names}")
        only_file_names = judge(work, ["file-names"])
        if list(only_file_names) != ["file-names"]:
            failures.append(f"--only file-names 应当只判 file-names 一样（这棵树上 placement、modules、one-scenario 都有问题，不该报出来），实际判了 {list(only_file_names)}")
        if parse_only("placement,modules") != ["placement", "modules"] or parse_only("placement,no-such-item") is not None:
            failures.append("--only 应当认出 placement,modules、认不出 no-such-item")
        # must-red：两条用例，一条有点名它测试目标的变异行，一条只有别的包里同名测试目标的行，一条一行都没有
        open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "a").write(
            f"crash-case:covered\tcrates/\ttest={VERIFICATION_PACKAGE}:covered_stream:covered_full\t#样本\n"
            f"crash-case:other-package\tcrates/\ttest={VERIFICATION_PACKAGE}:shared_name:shared_full\t#样本\n")
        open(os.path.join(work, "crates", "mutations.tsv"), "w").write(
            "# 样本变异表\n"
            f"改坏一处\tcrates/singlefs-core/src/a.rs\tx\ty\t-p {VERIFICATION_PACKAGE} --test covered_stream -- covered_quick --exact\tcovered_quick\n"
            f"别的包\tcrates/singlefs-core/src/a.rs\tx\tz\t-p singlefs-core --test shared_name -- t --exact\tt\n")
        cases, red_problems = must_red_problems(work)
        if not any("crash-case:demo" in problem for problem in red_problems) or any("crash-case:covered" in problem for problem in red_problems) or cases != 3:
            failures.append(f"一行变异都没有的崩溃枚举用例应当报出来（crash-case:demo 报、crash-case:covered 不报，查 3 条），实际 {cases} 条、{red_problems}")
        if not any("crash-case:other-package" in problem for problem in red_problems):
            failures.append(f"变异行点名的是别的包里同名的测试目标，不该算（crash-case:other-package 应当报出来），实际 {red_problems}")
        os.makedirs(os.path.join(work, "scripts", "crash"), exist_ok=True)
        open(os.path.join(work, "scripts", "crash", "knob-home.sh"), "w").write("SAMPLE_EXPAND_UP_TO=6\n")
        open(os.path.join(work, KNOBS_TABLE), "w").write(
            "# 样本旋钮表\n展开上限（样本）\t6\t推的\tscripts/crash/knob-home.sh\n"
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
        print("  ✓ crash-case-check 自检通过：验证包里登记了的放行、注明不是用例的不算，调枚举函数没登记的报缺登记，登记了的标了 ignore 的报出来，"
              "写在别的包里的（直接调或经同文件 helper 调、自己逐个造崩溃状态）报住错了包，每轮新建池的、导入崩溃引擎模块而没声明的、按里程碑起名的各报出来，只用脚手架模块的不要求声明，"
              "合表点名的放量分项算登记、合表没点名的 crash-case 报出来，验证包 src 内联测试不要求登记，"
              "没有必红变异的崩溃枚举用例与依据空着、出处不在的旋钮各报出来，"
              "来历表里没有行的崩溃枚举项与放量分项报出来，"
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


BREAK_VARIABLES = ("CRASH_CASE_CHECK_BREAK", "CRASH_POINT_REPLAY_ITEMS_BREAK",)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行不是单独一个 --selftest而 BREAK_VARIABLES 里有一个非空（不管值是什么），
    退 2 并点名它只在自证里认；在参数解析之前判。"""
    for name in BREAK_VARIABLES:
        if os.environ.get(name) and arguments != ["--selftest"]:
            print(f"  ✗ {name} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
            print(f"     → 怎么办：去掉 {name} 再跑；要证红就跑 {name}=<项> python3 scripts/crash/crash-case-check.py --selftest", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
