#!/usr/bin/env python3
# admission: always 判的是此刻给的一组 cargo test 参数与此刻那一份登记表，上一次的结论不替这一次作保
# run-condition: none 只读给的参数与 <仓根>/.claude/gate.d/stage-inputs.tsv，不起子进程、不写文件
"""重测试目标只许 release：给一组 cargo test 参数（crates/mutations.tsv 第五段那种形态）与仓根，判这一组参数是不是必须带 release 档、依据是哪一样、已带没带。

依据：.claude/rules/test-script-discipline.md「编译与算法都要快」第一句（穷举、长时间随机、真设备这类用例用优化构建跑）。
「重」不另写名单，从仓里两样登记现算：
  crash-case 目标  登记表 .claude/gate.d/stage-inputs.tsv 里 crash-case: 行的 test=<包>:<测试目标>:<用例函数> 点到的（包、测试目标）。
                   读表与解析照 scripts/gate/admission.py 的 read_registration_rows、crash_cases_of（parse_crash_case），这里不另写一份。
                   参数选中这个目标就算：--test <目标>（--test=<目标>，cargo 的通配照认）点名它；或 --tests、--all-targets、一个目标选项都没写
                   （cargo test 默认连集成测试一起跑）。包：-p 点名了的要有登记的那个包；没写 -p、或写了 --workspace / --all 的不看包。
  gpu 特性         --features / -F（-F<名> 连写同样认）里有特性名（去掉 <包>/ 前缀）按 - 与 _ 切开带 gpu 这个词的（gpu、<包>/gpu、tests-on-a-real-gpu），或 --all-features。
已带 release：`--` 之前有 --release，或 --profile release / --profile=release（bench 承 release，同 scripts/mutation/lib_mutation_units.py 的 parse_cargo_arguments）。
参数的切法照 lib_mutation_units.py 的 parse_cargo_arguments（与 crates 变异判法的行键同一份），`--` 之后的 libtest 参数不看。

调法（按路径载入本文件，不往 sys.path 里加目录）：
  specification = importlib.util.spec_from_file_location("lib_release_only_tests", "<仓根>/scripts/mutation/lib_release_only_tests.py")
  module = importlib.util.module_from_spec(specification); specification.loader.exec_module(module)
  module.crash_case_targets(root) -> (CrashCaseTarget(package, target, keys)…)
      读 <root>/.claude/gate.d/stage-inputs.tsv 的 crash-case: 行，同一个（包、测试目标）并成一项、keys 是登记它的那几个键；
      表不在、读不了、crash-case: 行写法认不出（admission.py 的 RegistrationError）抛 RegistryUnreadable，不当「一个都没登记」
  module.release_requirement(arguments, root=None, registered=None) -> ReleaseRequirement(required, basis, already_release)
      arguments：cargo test 之后的参数（字符串按空白切，或词的列表）；root：读哪一份仓根的登记表；
      registered：crash_case_targets 交的那一份（一趟判几千行时先取一次再逐行交，不重读表）；root 与 registered 至少给一样，都没给抛 ValueError
      required：bool；basis：依据，元组，一样一句（「crash-case 目标 <包>:<测试目标>（<登记键>…）」「gpu 特性 <写法>」），不命中是空元组；
      already_release：bool；.release_missing：required 而没带 release
  module.with_release(arguments) -> str        补上 --release：插在第一个 `--` 之前，已带 release 的原样；--profile 写了别的档的连同它的值换成 --release
  module.release_forced(arguments, root=None, registered=None) -> (参数, ReleaseRequirement)
      release_missing 时交 with_release 补过的参数，否则原样交回调用方给的那一份
调它的：scripts/mutation/crates-mutation-rows.py（plan_rows 起 cargo 之前补 --release、static-check 的 RELEASE 行）、scripts/mutation/prove-red.sh（证红起 cargo 之前补 --release）。

命令行：
  lib_release_only_tests.py judge <仓根> -- <cargo test 参数…>
      打一行「RELEASE_REQUIREMENT<TAB>required=yes|no<TAB>already_release=yes|no<TAB>basis=<依据，；分隔，不命中写 ->」，退 0；
      登记表读不了打 ERROR 行与「→ 怎么办」，退 2；用法不对退 2
  lib_release_only_tests.py --selftest
      临时目录里造一份登记表，只调判法函数，不起子进程；退 0 全对、1 有格判错

弄坏开关 RELEASE_ONLY_TESTS_BREAK=<项>（只给证红用，逗号分隔可给几个；--selftest 认，命令行不是 --selftest 而设了它就在参数解析之前退 2、点名只在自证里认）。
项名、弄坏哪一处、打开后自证红在哪几格：
  RELEASE_ONLY_TESTS_BREAK=gpu-feature-ignored  不认 gpu 特性
      「gpu 特性：--features gpu、--features=<包>/gpu,<别的>、-F tests-on-a-real-gpu、-Fgpu、--all-features 都判必须 release，依据点名那一样」
  RELEASE_ONLY_TESTS_BREAK=crash-case-registry-unread  读了登记表却不收 crash-case: 行（登记的目标一个都没有）
      「crash-case 目标：--test 点名登记的目标、包对得上，判必须 release，依据点名那个目标与登记键」
      「crash-case 目标：没点名测试目标、--tests、--all-targets 而点名的包里有登记的目标，判必须 release」
      「命令行 judge：命中的打 required=yes 与依据、退 0；读不到登记表打 ERROR 与「→ 怎么办」、退 2」
  RELEASE_ONLY_TESTS_BREAK=missing-registry-as-empty  登记表不在时当空表（改错的样子：读不到就当一个都不命中）
      「读不到登记表：表不在、crash-case: 行写法认不出，都抛 RegistryUnreadable，不当不命中」
      「命令行 judge：命中的打 required=yes 与依据、退 0；读不到登记表打 ERROR 与「→ 怎么办」、退 2」
  RELEASE_ONLY_TESTS_BREAK=release-switch-unseen  已带 release 只认 --profile，不认 --release
      「已带 --release：认出已带、不算缺 release」
      「补 --release：插在第一个 -- 之前、已带的原样、--profile 的别的档换成 --release」
  RELEASE_ONLY_TESTS_BREAK=profile-release-unseen  已带 release 只认 --release，不认 --profile release
      「--profile release：--profile release 与 --profile=release 都认出已带」
  RELEASE_ONLY_TESTS_BREAK=every-row-required  一律判必须 release（改错的样子：两样都不中也补）
      「两样都不中：只跑 --lib、点名没登记的测试目标、包对不上的登记目标，都不判必须 release，依据是空的」
  RELEASE_ONLY_TESTS_BREAK=release-appended-at-end  补 --release 接在整串末尾（落进 `--` 之后成了 libtest 的参数）
      「补 --release：插在第一个 -- 之前、已带的原样、--profile 的别的档换成 --release」

退出码：0 判完（judge）或自证全对；1 自证有格判错；2 用法错、登记表读不了、弄坏开关设在自证之外。
管不到的：cargo 配置里 [alias] 起的别名、环境变量 CARGO_PROFILE 一类改档的写法不看；-p 的包名只按字面比，通配不认。
"""
import collections
import fnmatch
import importlib.util
import os
import re
import shutil
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

LIBRARY_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
ADMISSION_PATH = os.path.join(LIBRARY_DIRECTORY, "..", "gate", "admission.py")
MUTATION_UNITS_PATH = os.path.join(LIBRARY_DIRECTORY, "lib_mutation_units.py")
BREAK_VARIABLE = "RELEASE_ONLY_TESTS_BREAK"
EXIT_RED = 1
EXIT_USAGE = 2
RELEASE_SWITCH = "--release"
# bench 档承 release（cargo 的规矩），与 lib_mutation_units.py 的 parse_cargo_arguments 认的同一组
RELEASE_PROFILES = ("release", "bench")
GPU_FEATURE_WORD = "gpu"
# cargo test 选目标的选项：一个都没写时默认连集成测试一起跑
TARGET_OPTIONS = frozenset(("--lib", "--bin", "--bins", "--test", "--tests", "--example", "--examples", "--bench", "--benches", "--doc", "--all-targets"))
EVERY_TEST_TARGET_OPTIONS = ("--tests", "--all-targets")
EVERY_PACKAGE_OPTIONS = ("--workspace", "--all")
REGISTRY_HOWTO = ("照报错改 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行（写法在那张表的文件头与 scripts/gate/admission.py 的 parse_crash_case）；"
                  "表不在就是仓根给错了，<仓根> 写有这张表的那一份")

CrashCaseTarget = collections.namedtuple("CrashCaseTarget", "package target keys")


class ReleaseRequirement(collections.namedtuple("ReleaseRequirement", "required basis already_release")):
    __slots__ = ()

    @property
    def release_missing(self):
        return self.required and not self.already_release


class RegistryUnreadable(Exception):
    """登记表不在、读不了、crash-case: 行写法认不出：判不了哪几个是 crash-case 目标，调用方判红，不当不命中。"""


_LOADED_MODULES = {}


def loaded_module(path, name):
    """按路径载入、本进程里只载一次。"""
    if name not in _LOADED_MODULES:
        specification = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _LOADED_MODULES[name] = module
    return _LOADED_MODULES[name]


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def crash_case_targets(root):
    """<root> 那一份登记表里 crash-case: 行点到的（包、测试目标），同一个并成一项，次序照登记表。"""
    admission = loaded_module(ADMISSION_PATH, "admission_for_lib_release_only_tests")
    table = os.path.join(root, admission.REGISTRATION_TABLE)
    if not os.path.isfile(table) and not broken("missing-registry-as-empty"):
        raise RegistryUnreadable(f"登记表 {table} 不在")
    try:
        cases = admission.crash_cases_of(admission.read_registration_rows(root))
    except admission.RegistrationError as error:
        raise RegistryUnreadable(f"登记表 {table} 的 crash-case: 行认不出：{error}") from error
    except (OSError, UnicodeDecodeError) as error:
        raise RegistryUnreadable(f"登记表 {table} 读不了：{error}") from error
    if broken("crash-case-registry-unread"):
        return ()
    keys = collections.OrderedDict()
    for case in cases:
        keys.setdefault((case.package, case.target), []).append(case.key)
    return tuple(CrashCaseTarget(package, target, tuple(found)) for (package, target), found in keys.items())


def argument_words(arguments):
    """参数切成词；`--` 之前连写的 -F<特性> 拆成 -F 与特性（cargo 认短选项紧接值，parse_cargo_arguments 只认分开写的）。"""
    words = arguments.split() if isinstance(arguments, str) else [str(word) for word in arguments]
    separator = words.index("--") if "--" in words else len(words)
    before = []
    for word in words[:separator]:
        if word.startswith("-F") and len(word) > 2:
            before += ["-F", word[2:].lstrip("=")]
        else:
            before.append(word)
    return before, words[separator:]


def release_already(before):
    """`--` 之前已带 release 档：--release，或 --profile 写 release / bench。"""
    profiles = [word.split("=", 1)[1] for word in before if word.startswith("--profile=")]
    profiles += [before[index + 1] for index, word in enumerate(before[:-1]) if word == "--profile"]
    by_profile = any(profile in RELEASE_PROFILES for profile in profiles)
    by_switch = RELEASE_SWITCH in before
    if broken("release-switch-unseen"):
        return by_profile
    if broken("profile-release-unseen"):
        return by_switch
    return by_switch or by_profile


def gpu_feature_basis(parsed):
    if broken("gpu-feature-ignored"):
        return []
    basis = ["gpu 特性 --all-features（开全部特性，gpu 在内）"] if parsed.all_features else []
    for feature in parsed.features:
        if GPU_FEATURE_WORD in re.split(r"[-_]", feature.split("/")[-1]):
            basis.append(f"gpu 特性 {feature}")
    return basis


def crash_case_basis(parsed, before, registered):
    named_tests = [value for kind, value in parsed.targets if kind == "test" and value != "*"]
    options = {word.split("=", 1)[0] for word in before}
    every_test_target = bool(options & set(EVERY_TEST_TARGET_OPTIONS)) or not options & TARGET_OPTIONS
    every_package = not parsed.packages or bool(options & set(EVERY_PACKAGE_OPTIONS))
    basis = []
    for target in registered:
        if not every_package and target.package not in parsed.packages:
            continue
        if every_test_target:
            how = "没点名测试目标（或 --tests、--all-targets），这个包的集成测试都跑"
        elif any(fnmatch.fnmatchcase(target.target, pattern) for pattern in named_tests):
            how = "--test 点名"
        else:
            continue
        basis.append(f"crash-case 目标 {target.package}:{target.target}（{how}；登记 {'、'.join(target.keys)}）")
    return basis


def release_requirement(arguments, root=None, registered=None):
    if registered is None:
        if root is None:
            raise ValueError("release_requirement 要给 root（读哪一份登记表）或 registered（crash_case_targets 交的那一份）")
        registered = crash_case_targets(root)
    before, _after = argument_words(arguments)
    parsed = loaded_module(MUTATION_UNITS_PATH, "lib_mutation_units_for_lib_release_only_tests").parse_cargo_arguments(before)
    basis = crash_case_basis(parsed, before, registered) + gpu_feature_basis(parsed)
    if broken("every-row-required") and not basis:
        basis = ["弄坏开关 every-row-required：两样都不中也判必须"]
    return ReleaseRequirement(bool(basis), tuple(basis), release_already(before))


def with_release(arguments):
    before, after = argument_words(arguments)
    if release_already(before):
        return " ".join(before + after)
    kept, skip_value = [], False
    for word in before:
        if skip_value:
            skip_value = False
        elif word == "--profile":
            skip_value = True
        elif not word.startswith("--profile="):
            kept.append(word)
    if broken("release-appended-at-end"):
        return " ".join(kept + after + [RELEASE_SWITCH])
    return " ".join(kept + [RELEASE_SWITCH] + after)


def release_forced(arguments, root=None, registered=None):
    requirement = release_requirement(arguments, root, registered)
    return (with_release(arguments) if requirement.release_missing else arguments), requirement


def judge_command(arguments):
    """judge <仓根> -- <cargo test 参数…>：→ (退出码, [输出行])。"""
    if len(arguments) < 2 or arguments[1] != "--":
        return EXIT_USAGE, ["ERROR\t用法不对", "     → 怎么办：lib_release_only_tests.py judge <仓根> -- <cargo test 参数…>"]
    try:
        requirement = release_requirement(arguments[2:], root=arguments[0])
    except RegistryUnreadable as error:
        return EXIT_USAGE, [f"ERROR\t{error}", f"     → 怎么办：{REGISTRY_HOWTO}"]
    yes_no = {True: "yes", False: "no"}
    return 0, [f"RELEASE_REQUIREMENT\trequired={yes_no[requirement.required]}\talready_release={yes_no[requirement.already_release]}\t"
               f"basis={'；'.join(requirement.basis) or '-'}"]


SELFTEST_REGISTRY = ("# 自证用的登记表\n"
                     "other-gate.sh\tscripts/other.py\t# 别的一道\n"
                     "crash-case:demo-stream\t-\ttest=demo-verification:demo_crash_stream:the_case count-line=DEMO\t# 样本\n"
                     "crash-case:demo-stream-again\t-\ttest=demo-verification:demo_crash_stream:another_case count-line=DEMO\t# 样本：同一个目标的另一条用例\n"
                     "crash-case:demo-faults\t-\ttest=demo-verification:demo_crash_faults:the_case count-line=DEMO\t# 样本\n")
SELFTEST_MALFORMED_REGISTRY = SELFTEST_REGISTRY + "crash-case:demo-broken\t-\ttest=demo-verification:demo_crash_stream:the_case test=demo-verification:demo_crash_stream:the_case\t# 两条 test=\n"


def selftest_root(work, name, registry):
    root = os.path.join(work, name)
    os.makedirs(os.path.join(root, ".claude", "gate.d"))
    if registry is not None:
        with open(os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv"), "w", encoding="utf-8") as handle:
            handle.write(registry)
    return root


def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="lib-release-only-tests-selftest-")

    def cell(name, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(f"{name}：{detail}")

    try:
        root = selftest_root(work, "registered", SELFTEST_REGISTRY)
        registered = crash_case_targets(root)

        named = release_requirement("-p demo-verification --test demo_crash_stream -- the_case --exact", root=root)
        cell("crash-case 目标：--test 点名登记的目标、包对得上，判必须 release，依据点名那个目标与登记键",
             named.required and not named.already_release and named.release_missing and len(named.basis) == 1
             and "crash-case 目标 demo-verification:demo_crash_stream" in named.basis[0]
             and "crash-case:demo-stream、crash-case:demo-stream-again" in named.basis[0], f"交 {named}")

        every = [release_requirement(arguments, registered=registered) for arguments in
                 ("-p demo-verification -- the_case", "-p demo-verification --tests -- the_case", "-p demo-verification --all-targets", "--test=demo_crash_*")]
        cell("crash-case 目标：没点名测试目标、--tests、--all-targets 而点名的包里有登记的目标，判必须 release",
             all(requirement.required and sum("crash-case 目标" in item for item in requirement.basis) == 2 for requirement in every), f"交 {every}")

        gpu_forms = ("-p demo --features gpu --lib -- doubles", "-p demo --features=demo/gpu,other --lib", "-p demo -F tests-on-a-real-gpu --test parity",
                     "-p demo -Fgpu --lib", "-p demo --all-features --lib")
        gpu = [release_requirement(arguments, registered=registered) for arguments in gpu_forms]
        cell("gpu 特性：--features gpu、--features=<包>/gpu,<别的>、-F tests-on-a-real-gpu、-Fgpu、--all-features 都判必须 release，依据点名那一样",
             all(requirement.required and len(requirement.basis) == 1 and requirement.basis[0].startswith("gpu 特性 ") for requirement in gpu)
             and gpu[1].basis[0] == "gpu 特性 demo/gpu" and gpu[2].basis[0] == "gpu 特性 tests-on-a-real-gpu", f"交 {gpu}")

        neither = [release_requirement(arguments, registered=registered) for arguments in
                   ("-p demo-verification --lib -- the_case", "-p demo-verification --test demo_unregistered -- the_case",
                    "-p other-package --test demo_crash_stream", "-p demo --features gpuish,debugger --lib", "-p demo --lib -- --features gpu")]
        cell("两样都不中：只跑 --lib、点名没登记的测试目标、包对不上的登记目标，都不判必须 release，依据是空的",
             all(not requirement.required and requirement.basis == () and not requirement.release_missing for requirement in neither), f"交 {neither}")

        released = release_requirement("-p demo-verification --release --test demo_crash_stream -- the_case", registered=registered)
        cell("已带 --release：认出已带、不算缺 release", released.already_release and not released.release_missing, f"交 {released}")

        profiles = [release_requirement(arguments, registered=registered) for arguments in
                    ("-p demo --profile release --features gpu --lib", "-p demo --profile=release --features gpu --lib")]
        cell("--profile release：--profile release 与 --profile=release 都认出已带",
             all(requirement.already_release and not requirement.release_missing for requirement in profiles), f"交 {profiles}")

        raised = []
        for name, registry in (("absent", None), ("malformed", SELFTEST_MALFORMED_REGISTRY)):
            try:
                outcome = release_requirement("-p demo --lib", root=selftest_root(work, name, registry))
            except RegistryUnreadable as error:
                outcome = f"RegistryUnreadable：{error}"
            raised.append(outcome)
        cell("读不到登记表：表不在、crash-case: 行写法认不出，都抛 RegistryUnreadable，不当不命中",
             all(isinstance(outcome, str) for outcome in raised) and "不在" in raised[0] and "恰好一条 test=" in raised[1], f"交 {raised}")

        added = [with_release(arguments) for arguments in
                 ("-p demo --features gpu --lib -- doubles --exact", "-p demo --features gpu --lib", "-p demo --release --lib -- x",
                  "-p demo --profile dev --lib -- x", "-p demo --profile=test --lib")]
        cell("补 --release：插在第一个 -- 之前、已带的原样、--profile 的别的档换成 --release",
             added == ["-p demo --features gpu --lib --release -- doubles --exact", "-p demo --features gpu --lib --release", "-p demo --release --lib -- x",
                       "-p demo --lib --release -- x", "-p demo --lib --release"], f"交 {added}")

        outputs = []
        for arguments in ([root, "--", "-p", "demo-verification", "--test", "demo_crash_faults"], [selftest_root(work, "judge-absent", None), "--", "-p", "demo", "--lib"]):
            outputs.append(judge_command(arguments))
        cell("命令行 judge：命中的打 required=yes 与依据、退 0；读不到登记表打 ERROR 与「→ 怎么办」、退 2",
             outputs[0][0] == 0 and outputs[0][1][0].startswith("RELEASE_REQUIREMENT\trequired=yes\talready_release=no\tbasis=crash-case 目标 demo-verification:demo_crash_faults")
             and outputs[1][0] == EXIT_USAGE and outputs[1][1][0].startswith("ERROR\t") and "→ 怎么办" in outputs[1][1][1], f"交 {outputs}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 crash_case_targets / release_requirement / release_already / with_release / judge_command；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return EXIT_RED
    print(f"  ✓ lib_release_only_tests 自检通过（查了 {checked} 格）：crash-case 目标点名与全选都判必须、gpu 特性五种写法都判必须、两样都不中的不判、"
          "--release 与 --profile release 都认出已带、登记表不在与写法认不出都抛 RegistryUnreadable、--release 插在第一个 -- 之前、命令行 judge 的两种结局")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["judge"]:
        code, lines = judge_command(arguments[1:])
        print("\n".join(lines))
        return code
    print("ERROR\t用法不对")
    print("     → 怎么办：lib_release_only_tests.py judge <仓根> -- <cargo test 参数…>，或 --selftest")
    return EXIT_USAGE


def refuse_break_outside_selftest(arguments):
    """直接执行时、参数解析之前调：不是 --selftest 而设了弄坏开关就打拒绝、交 2；没设或是 --selftest 交 None。"""
    setting = os.environ.get(BREAK_VARIABLE, "")
    if not setting or arguments == ["--selftest"]:
        return None
    print(f"  ✗ {BREAK_VARIABLE}={setting} 只在自证里认（只给 --selftest 证红用），这一趟不是 --selftest，什么都没做", file=sys.stderr)
    print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/mutation/lib_release_only_tests.py --selftest", file=sys.stderr)
    return EXIT_USAGE


if __name__ == '__main__':
    refused = refuse_break_outside_selftest(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
