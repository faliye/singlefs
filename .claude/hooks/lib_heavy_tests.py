"""一条命令是不是重型测试、属于哪一类。

heavy-test-guard.sh（执行前拒绝）与看门狗（research/scripts/agent-watch.py，在进程这一层）共用这一份判定，各自按文件路径用 importlib 导入；
同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑哪一类、要带什么前缀，是 heavy-test-guard.sh 的事，不在这里。

入口：
  classify(words, directory, environment=None) -> HeavyTest | None
      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None；
      environment 是这条命令看得到的、命令文本里设过的变量（lib_shell_words 的 CommandAtPosition.environment），认 runner 与别名要用。
  command_under_launcher(words) -> (list[str], str | None) | None
      words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
      剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
      短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE`、strace 的 `-E NAME=VALUE` / `--env=NAME=VALUE` 设的变量写在交回那条命令的最前面
      （再切一遍时当它的环境变量，认 runner 要用；strace 的 `-E NAME` 只清变量，不带）。
      这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
  classify_process(argv, cwd) -> list[HeavyTest]
      一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
      先照 lib_shell_words 剥掉 bash / sh 起脚本、bash -c、nice / timeout / env / taskset、capped.sh N 这类包装，
      再照 command_under_launcher 剥掉 /usr/bin/time 这一类，逐条 classify；空列表就是不重型。
  runs_compiled_code(words, directory) -> str | None
      一条已经剥掉前缀与包装的命令会不会跑编译出来的代码：cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算），
      或直接执行 cargo 编出来的二进制（测试二进制，与名字里带 target 的目录底下 debug / release 里的）；会就交一句说明。
      heavy-test-guard.sh 拿它判子 agent 跑这一类经没经 research/scripts/run-with-memory-cap.sh（与重型不重型无关）。
  judged_by_name(word, directory) -> bool
      这个命令词是不是按名字判的仓内脚本（.claude/gate.d/ 下的阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在它们的仓内位置上）：
      算不算重型、带什么参数才算，都在名字那一格判完了，要读脚本正文的一方不再读进去。
  python3 lib_heavy_tests.py --selftest

认的输入：cargo 命令行（test / t、run / r，以及起测试的 nextest run、miri、llvm-cov、hack、mutants）、按名字认的脚本与门禁阶段、
qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，
名字含 layer0 的算层 0。libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
--test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列。
崩溃枚举用例（门禁 54 号逐条跑的那几条，登记在 .claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>）：
跑到它们的测试目标（cargo test 点名它、通配命中它、或不挑目标而包里有它；直接执行它的测试二进制）而下面任一条成立的算重型：
  libtest 参数带 --ignored 或 --include-ignored（nextest 是 --run-ignored 的值不是 default）；
  可能带上它而看不见：cargo 全局选项 --config 里定了别名（alias.<名>，且子命令就是那个别名）或 runner（按正则认 .runner =，
  另把值按 TOML 读，定了 target.<任何>.runner 也算：带引号的键 "runner" 这一类），
  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER（systemd-run -E / --setenv、strace -E / --env 设给里面那条命令的也算），或 CARGO_ALIAS_<名> 定的别名就是子命令；
  登记的用例函数有一处定义没标 #[ignore]，或判不出标没标（找不到那个目标、那个函数，宏生成的用例，读不了源码，导入不了 admission.py：都按没标算）。
  读那个包里测试目标的源码，判法与 research/scripts/admission.py 的 crash-cases 自查同一份：本模块按文件路径导入它，同名的每一处 fn <名>( 都判。
另有一条按参数认：任何命令（按文本处理参数的 grep、git、sed 这一类除外）参数里有 --ignored 或 --include-ignored、又有登记的用例函数名，
按跑崩溃枚举用例算（拷走改名的测试二进制、find -exec 起的这类，名字认不出，靠点名的用例函数认）。
登记表取两份的并：从命令的当前目录（有 --manifest-path 时取它所在的目录）往上找到的第一份，与这份文件所在仓的那一份。
接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒；
同一目标里别的模块有与登记的用例函数同名、合法不标 #[ignore] 的快用例时照拒；admission.py 导入不了时，点名登记目标的 cargo test 一律拒。
看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名用例函数的测试二进制；
cargo mutants -d 指到别处的树。别名与 runner 那两种由看门狗（research/scripts/agent-watch.py）在进程这一层兜：cargo 最后照样以原名
带 --ignored 起那个测试二进制（推的，没量）；拷走改名、又不点名用例函数的，看门狗同样认不出。
弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 heavy-test-guard.sh --selftest 都必须判红：
  launcher-whole-word-options（短选项合写不拆，照旧按整词查表）、launcher-drops-setenv（systemd-run -E / --setenv 设的变量不带进里面那条命令）、
  list-by-presence（见到 --list 这个词就算只列）、undecided-ignore-allowed（判不出标没标时放行）、
  runner-configuration-by-regex-only（--config 只按正则认 runner）、strace-drops-env（strace -E / --env 设的变量不带进里面那条命令）。
"""
import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
from typing import NamedTuple


def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(os.path.dirname(os.path.abspath(__file__)), module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


shell_words = load_sibling_module("lib_shell_words")


class HeavyTest(NamedTuple):
    kind: str      # 细分的用法（layer0-cargo、gate-sh……），heavy-test-guard.sh 按它定谁能跑
    category: str  # 报给人看的类名（层 0、全量测试……），KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 每一种重型用法：kind → 它属于哪一类
KIND_CATEGORY = {
    "layer0-stage": "层 0", "layer0-cargo": "层 0", "layer0-binary": "层 0",
    "qemu-stage": "QEMU", "qemu-system": "QEMU", "vm-bench": "QEMU",
    "herd7-stage": "herd7", "lkmm": "herd7", "herd7": "herd7",
    "crates-mutation-stage": "crates 变异整表", "crates-mutation-mutate": "crates 变异整表",
    "full-cargo": "全量测试", "check-sh": "全量测试",
    "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
    "e152": "E152 装置",
    "crash-case-cargo": "崩溃枚举用例", "crash-case-binary": "崩溃枚举用例",
}
# 只有这几道阶段是重型；.claude/gate.d/ 下其余阶段谁都能跑
STAGE_KIND = {"54": "layer0-stage", "55": "qemu-stage", "57": "herd7-stage", "59": "crates-mutation-stage", "87": "replay-all-stage"}
# 按名字判的仓内脚本：名字 → 它在仓里的位置。命令词是这个名字、又落在这个位置上，judged_by_name 为真
KNOWN_SCRIPT_LOCATIONS = {
    "gate.sh": ".claude/scripts/gate.sh", "check.sh": ".claude/scripts/check.sh", "lkmm.sh": ".claude/scripts/lkmm.sh",
    "gate-staged.sh": "research/scripts/gate-staged.sh", "mutate.sh": "research/scripts/mutate.sh",
    "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
    "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
    "layer0-shard-run.sh": "research/scripts/layer0-shard-run.sh",
}


def heavy_test(kind, detail):
    return HeavyTest(kind, KIND_CATEGORY[kind], detail)


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def libtest_lists_only(libtest_arguments):
    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算。"""
    if break_is_set("list-by-presence"):
        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
    position = 0
    while position < len(libtest_arguments):
        argument = libtest_arguments[position]
        if argument == LIST_ONLY_TEST_ARGUMENT:
            return True
        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
    return False


# 崩溃枚举用例的登记表（与 research/scripts/admission.py 读的是同一份）：键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>
CRASH_CASE_REGISTRY = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}
# libtest 只列用例、一条都不跑的参数
LIST_ONLY_TEST_ARGUMENT = "--list"
# libtest 带一个值的选项：`--skip --list` 里的 --list 是 --skip 的值（照样全跑），不是「只列」
LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}
# 弄坏开关的环境变量（写法见文件头）
BREAK_VARIABLE = "LIB_HEAVY_TESTS_BREAK"
# 判用例函数标没标 #[ignore] 的那一份（与门禁 54 号的 crash-cases 自查同一套判法）
ADMISSION_MODULE_PATH = os.path.join(HOOK_REPOSITORY, "research", "scripts", "admission.py")
# 按参数认崩溃枚举用例时不看的命令：它们把参数当文本搜、打印、比对，不执行测试二进制
COMMANDS_TREATING_ARGUMENTS_AS_TEXT = {"grep", "egrep", "fgrep", "rg", "ag", "git", "sed", "awk", "echo", "printf", "cat", "head", "tail",
                                       "less", "wc", "sort", "uniq", "diff", "tee", "jq", "ls"}


class RegisteredCrashCase(NamedTuple):
    registry_root: str  # 登记它的那份登记表所在的仓根
    package: str
    target: str
    function: str


def crash_case_registries(directory):
    """要读的登记表：从 directory 往上找到的第一份，加这份文件所在仓的那一份（两份是同一个文件时只算一次）。"""
    found = []
    while directory:
        candidate = os.path.join(directory, CRASH_CASE_REGISTRY)
        if os.path.isfile(candidate):
            found.append(candidate)
            break
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    own = os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)
    if os.path.isfile(own) and all(os.path.realpath(own) != os.path.realpath(path) for path in found):
        found.append(own)
    return found


def registered_crash_cases(directory):
    """登记的崩溃枚举用例（RegisteredCrashCase 的集合）。读不了的登记表当它没有。"""
    cases = set()
    for path in crash_case_registries(directory):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                lines = handle.read().split("\n")
        except OSError:
            continue
        registry_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(path))))
        for line in lines:
            columns = line.split("\t")
            if not columns[0].startswith("crash-case:") or len(columns) < 3 or columns[2].lstrip().startswith("#"):
                continue
            for token in columns[2].split():
                match = CRASH_CASE_TEST_CONDITION.match(token)
                if match:
                    cases.add(RegisteredCrashCase(registry_root, match.group("package"), match.group("target"), match.group("function")))
    return cases


def registered_crash_case_targets(directory):
    """登记的崩溃枚举用例：{(包名, 测试目标)}。"""
    return {(case.package, case.target) for case in registered_crash_cases(directory)}


@functools.cache
def admission_module():
    """research/scripts/admission.py（按文件路径导入一次）；导入不了交 None，调用方按「判不出」处理。"""
    try:
        spec = importlib.util.spec_from_file_location("admission_for_heavy_tests", ADMISSION_MODULE_PATH)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception:  # 文件不在、语法错、导入时抛的都算导入不了
        return None
    return module


def crash_case_package_directory(case, members):
    """这条用例的包目录（绝对路径）：命令所在工作区的成员里有它就取那一份（cargo 编的是它），没有就取登记它的那个仓的；都没有交 None。"""
    if case.package in members:
        return members[case.package]
    root_manifest = os.path.join(case.registry_root, "Cargo.toml")
    sections = manifest_sections(root_manifest)
    if sections and "workspace" in sections:
        return workspace_packages(root_manifest, sections).get(case.package)
    return None


def crash_case_function_runs_without_ignored(case, package_directory):
    """登记的用例函数在 package_directory 这个包里可能不带 --ignored 也被跑到：同名的每一处定义都标了 #[ignore] 才交 False；
    有一处没标，或判不出（找不到包、那个目标、那个函数，宏生成的用例，读不了源码，导入不了 admission.py）都交 True——
    判不出时放行，这一条在提交时 54 号的 crash-cases 自查判红之前就已经跑完了。"""
    module = admission_module()
    if module is None or package_directory is None:
        return not break_is_set("undecided-ignore-allowed")
    marked = module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function)
    if break_is_set("undecided-ignore-allowed"):
        return marked is False
    return marked is not True


def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def nearest_manifest(directory):
    while directory:
        candidate = os.path.join(directory, "Cargo.toml")
        if os.path.isfile(candidate):
            return candidate
        parent = os.path.dirname(directory)
        if parent == directory:
            return None
        directory = parent
    return None


def workspace_root_manifest(manifest_path):
    directory = os.path.dirname(manifest_path)
    while True:
        candidate = os.path.join(directory, "Cargo.toml")
        sections = manifest_sections(candidate) if os.path.isfile(candidate) else None
        if sections and "workspace" in sections:
            return candidate, sections
        parent = os.path.dirname(directory)
        if parent == directory:
            return None, None
        directory = parent


def workspace_packages(root_manifest, root_sections):
    """工作区成员：包名 → 包目录。"""
    root = os.path.dirname(root_manifest)
    packages = {}
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            sections = manifest_sections(os.path.join(directory, "Cargo.toml"))
            name = ((sections or {}).get("package") or {}).get("name")
            if name:
                packages[name] = os.path.normpath(directory)
    return packages


def layer0_test_targets(package_directory):
    """包里名字含 layer0 的集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
    names = set()
    for path in glob.glob(os.path.join(package_directory, "tests", "*.rs")):
        names.add(os.path.basename(path)[:-3])
    for path in glob.glob(os.path.join(package_directory, "tests", "*", "main.rs")):
        names.add(os.path.basename(os.path.dirname(path)))
    for target in (manifest_sections(os.path.join(package_directory, "Cargo.toml")) or {}).get("test") or []:
        if isinstance(target, dict) and target.get("name"):
            names.add(target["name"])
    return sorted(name for name in names if "layer0" in name)


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path",
                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
NARROWING_SELECTORS = {"--test", "--lib", "--bin", "--bins", "--example", "--examples", "--bench", "--benches", "--doc"}
GLOB_CHARACTERS = set("*?[")
# cargo nextest run --run-ignored 的值里会跑 ignored 用例的（default 不跑）
NEXTEST_RUN_IGNORED_VALUES = {"only", "all", "ignored-only"}
# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
# cargo mutants 的选项里 cargo test 也认、照搬过去的（选包的那几个）
MUTANTS_PACKAGE_OPTIONS_WITH_VALUE = {"-p", "--package", "--manifest-path"}
RUNNER_CONFIGURATION = re.compile(r"\.runner\s*=")
ALIAS_CONFIGURATION = re.compile(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=")
RUNNER_ENVIRONMENT_VARIABLE = re.compile(r"^CARGO_TARGET_.+_RUNNER$")
ALIAS_ENVIRONMENT_PREFIX = "CARGO_ALIAS_"


class CargoCommand(NamedTuple):
    subcommand: str
    rest: list                  # 子命令后面的参数
    directory: str | None       # 按 -C 换过之后的目录
    configuration_values: list  # 全局选项 --config 的值（--config=<值> 与 --config <值> 两种写法）


def cargo_subcommand(arguments, directory):
    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 CargoCommand；没有子命令交 None。"""
    index, configuration_values = 0, []
    while index < len(arguments):
        argument = arguments[index]
        if argument.startswith("+"):
            index += 1
        elif argument in CARGO_GLOBAL_OPTIONS_WITH_VALUE:
            if argument == "--config" and index + 1 < len(arguments):
                configuration_values.append(arguments[index + 1])
            index += 2
        elif argument == "-C":
            directory = shell_words.resolve_path(directory, arguments[index + 1]) if index + 1 < len(arguments) else directory
            index += 2
        elif argument.startswith("-"):
            if argument.startswith("--config="):
                configuration_values.append(argument[len("--config="):])
            index += 1
        else:
            break
    if index >= len(arguments):
        return None
    return CargoCommand(arguments[index], arguments[index + 1:], directory, configuration_values)


def mutants_test_arguments(rest):
    """cargo mutants 的参数 → 它反复起的 cargo test 的参数：选包的选项（-p / --package / --manifest-path / --workspace）照搬，`--` 之后的原样接上。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (rest, [])
    kept, index = [], 0
    while index < len(before):
        word = before[index]
        if word in MUTANTS_PACKAGE_OPTIONS_WITH_VALUE:
            kept += before[index:index + 2]
            index += 2
            continue
        if word == "--workspace" or word.startswith(("--package=", "--manifest-path=")) or (word.startswith("-p") and len(word) > 2):
            kept.append(word)
        index += 1
    return kept + after


def test_invocation(subcommand, rest):
    """cargo 的子命令与它后面的参数 → (怎么起的：'test'、'nextest'、'run' 或 None, 照 cargo test 的写法读的参数)。
    nextest 只认 run（r）；miri、llvm-cov、hack 剥掉自己那一层再看里面那个子命令（llvm-cov 不带子命令时就是跑测试）；mutants 反复起 cargo test。"""
    if subcommand in ("test", "t"):
        return "test", rest
    if subcommand in ("run", "r"):
        return "run", rest
    if subcommand == "nextest":
        return ("nextest", rest[1:]) if rest[:1] in (["run"], ["r"]) else (None, [])
    if subcommand == "miri":
        return test_invocation(rest[0], rest[1:]) if rest else (None, [])
    if subcommand == "llvm-cov":
        if rest[:1] == ["nextest"]:
            return "nextest", rest[1:]
        if rest and rest[0] in LLVM_COV_NON_RUNNING_SUBCOMMANDS:
            return None, []
        if rest and not rest[0].startswith("-"):
            return test_invocation(rest[0], rest[1:])
        return "test", rest
    if subcommand == "hack":
        options = rest[:rest.index("--")] if "--" in rest else rest
        for index, word in enumerate(options):
            if word in HACK_INNER_SUBCOMMANDS:
                return test_invocation(word, rest[:index] + rest[index + 1:])
        return None, []
    if subcommand == "mutants":
        return "test", mutants_test_arguments(rest)
    return None, []


def alias_of_subcommand(command, environment):
    """子命令是 --config 的 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的别名时交一句（展开之后是什么、带什么参数都看不见，当 cargo test 判）；不是交 None。"""
    aliases = {match.group(1) for value in command.configuration_values for match in [ALIAS_CONFIGURATION.match(value)] if match}
    if command.subcommand in aliases:
        return f"--config 里定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment:
        return f"环境变量 {alias_variable} 定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    return None


def configuration_value_names_runner(value):
    """--config 的一个值按 TOML 读：定了 target.<任何>.runner（带不带引号的键、点号两边带不带空白都算）交 True；
    读不成 TOML 的（多半是配置文件的路径，文件头「看不见的」）交 False，交给正则那一道。"""
    try:
        settings = tomllib.loads(value)
    except ValueError:  # TOMLDecodeError 是 ValueError
        return False
    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
    return any(isinstance(table, dict) and "runner" in table for table in targets.values())


def runner_of_test_binaries(command, environment):
    """--config 里定了 …runner、或环境变量里有 CARGO_TARGET_*_RUNNER 时交一句（runner 能替测试二进制加 --ignored，命令行上看不见）；没有交 None。"""
    by_table = not break_is_set("runner-configuration-by-regex-only")
    if any(RUNNER_CONFIGURATION.search(value) or (by_table and configuration_value_names_runner(value)) for value in command.configuration_values):
        return "--config 里定的 runner（它能替测试二进制加 --ignored）"
    runners = sorted(name for name in environment if RUNNER_ENVIRONMENT_VARIABLE.match(name))
    if runners:
        return f"环境变量 {runners[0]} 定的 runner（它能替测试二进制加 --ignored）"
    return None


def cargo_use(arguments, directory, environment=None):
    """cargo 这一条算不算重型：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
    found = cargo_subcommand(arguments, directory)
    if found is None:
        return None
    environment = environment or {}
    alias_reason = alias_of_subcommand(found, environment)
    invocation, rest = ("test", found.rest) if alias_reason else test_invocation(found.subcommand, found.rest)
    unseen_reason = alias_reason or runner_of_test_binaries(found, environment)
    directory = found.directory
    packages, test_names, selectors, binaries = [], [], set(), []
    whole_workspace, manifest_argument, run_ignored_value = False, None, None
    position = 0
    while position < len(rest):
        argument = rest[position]
        if argument == "--":
            break
        if argument.startswith("--") and "=" in argument:
            option, value = argument.split("=", 1)
            position += 1
        elif argument.startswith("-p") and len(argument) > 2 and not argument.startswith("--"):
            option, value = "-p", argument[2:].lstrip("=")
            position += 1
        elif argument in CARGO_TEST_OPTIONS_WITH_VALUE:
            option = argument
            value = rest[position + 1] if position + 1 < len(rest) else ""
            position += 2
        else:
            option, value = argument, None
            position += 1
        if option in ("-p", "--package"):
            packages.append(value)
        elif option == "--test":
            test_names.append(value)
            selectors.add(option)
        elif option in ("--bin", "--example", "--bench"):
            selectors.add(option)
            if option == "--bin":
                binaries.append(value)
        elif option == "--manifest-path":
            manifest_argument = value
        elif option in ("--workspace", "--all"):
            whole_workspace = True
        elif option in ("--lib", "--bins", "--examples", "--tests", "--benches", "--all-targets", "--doc"):
            selectors.add(option)
        elif option == "--run-ignored":
            run_ignored_value = value
    if invocation == "run":
        if "e152-file-system-benchmark" in binaries:
            return heavy_test("e152", "cargo run --bin e152-file-system-benchmark")
        return None
    if invocation is None:
        return None
    described = f"cargo {found.subcommand}"
    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
    if libtest_lists_only(libtest_arguments):
        return None
    if whole_workspace:
        return heavy_test("full-cargo", f"{described} 带 --workspace / --all")
    named_layer0 = [name for name in test_names if "layer0" in name]
    if named_layer0:
        return heavy_test("layer0-cargo", f"{described} --test {named_layer0[0]}")
    ignored_reasons = sorted(IGNORED_TEST_ARGUMENTS & set(libtest_arguments))
    if invocation == "nextest" and run_ignored_value in NEXTEST_RUN_IGNORED_VALUES:
        ignored_reasons.append(f"--run-ignored {run_ignored_value}")
    if unseen_reason:
        ignored_reasons.append(unseen_reason)
    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
    crash_cases = registered_crash_cases(os.path.dirname(manifest_path) if manifest_path else directory)
    sections = manifest_sections(manifest_path) if manifest_path else None
    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
    crash_case_targets = sorted({case.target for case in crash_cases})
    for pattern in test_names:
        matched = fnmatch.filter(crash_case_targets, pattern) if GLOB_CHARACTERS & set(pattern) else [name for name in crash_case_targets if name == pattern]
        found_heavy = crash_case_run("crash-case-cargo", f"{described} --test {pattern}", [case for case in crash_cases if case.target in matched],
                                     ignored_reasons, members) if matched else None
        if found_heavy:
            return found_heavy
    if sections is None:
        return None
    is_virtual_workspace_root = "workspace" in sections and "package" not in sections
    narrowed = bool(selectors & NARROWING_SELECTORS)
    if not packages and is_virtual_workspace_root and not narrowed:
        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
    if packages:
        scope = [members[name] for name in packages if name in members]
    elif is_virtual_workspace_root:
        scope = list(members.values())
    else:
        scope = [os.path.normpath(os.path.dirname(manifest_path))]
    if not narrowed and members and set(scope) == set(members.values()):
        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                         f"{len(members)} 个成员），等于全量"))
    layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
    unselected = not selectors or bool(selectors & {"--tests", "--all-targets"})
    crash_cases_in_scope = [case for case in crash_cases if members.get(case.package) in scope]
    if not layer0_targets and unselected and crash_cases_in_scope:
        found_heavy = crash_case_run("crash-case-cargo", f"{described} 不挑目标", crash_cases_in_scope, ignored_reasons, members)
        if found_heavy:
            return found_heavy
    if not layer0_targets:
        return None
    if not selectors or selectors & {"--tests", "--all-targets"}:
        return heavy_test("layer0-cargo", f"{described} 不挑目标，会跑到名字含 layer0 的测试二进制（{'、'.join(layer0_targets)}）")
    for pattern in test_names:
        if GLOB_CHARACTERS & set(pattern):
            matched = fnmatch.filter(layer0_targets, pattern)
            if matched:
                return heavy_test("layer0-cargo", f"{described} --test {pattern} 命中名字含 layer0 的测试二进制（{'、'.join(matched)}）")
    return None


def crash_case_run(kind, described, cases, ignored_reasons, members):
    """点到登记的崩溃枚举用例 cases 的那一条算不算重型：带了（或可能带了）跑 ignored 用例的参数，或有一条的用例函数没标 #[ignore]，
    交 kind 那一类的 HeavyTest；否则交 None。members 是命令所在工作区的成员（包名 → 包目录），找用例的源码用。"""
    targets = "、".join(sorted({case.target for case in cases}))
    if ignored_reasons:
        return heavy_test(kind, f"{described} 带 {ignored_reasons[0]}，跑到登记的崩溃枚举用例（{targets}）")
    unmarked = sorted({case.function for case in cases if crash_case_function_runs_without_ignored(case, crash_case_package_directory(case, members))})
    if unmarked:
        return heavy_test(kind, f"{described} 不带 --ignored，而登记的用例函数 {unmarked[0]} 没标 #[ignore]，照样跑到全量（{targets}）")
    return None


GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/(\d+)-[^/]+\.sh$")
# cargo 编出来的测试二进制：<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制的元数据哈希>，名字里的 - 已换成 _
TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")


def gate_stage_number(word, directory):
    for candidate in (word, shell_words.resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match:
            return match.group(1)
    return None


def test_binary_name(word):
    """直接执行的是 cargo 编出来的测试二进制时交回它的名字（去掉哈希），不是时交 None。"""
    match = TEST_BINARY_PATH.search(word)
    return match.group(1) if match else None


# 跑编译出来的代码的 cargo 子命令（t、r 是 test、run 的内置简写）；只编不跑的 build、check、clippy 不在内
COMPILED_CODE_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}
# cargo 编出来的二进制直接执行：<名字里带 target 的目录>/[<目标三元组>/]<debug|release>/[deps/]<名字>；名字里不许有点（排掉 .d、.rlib、.so）。
# CARGO_TARGET_DIR 设成名字里不带 target 的目录、再直接执行里面的二进制，认不出（测试二进制按 TEST_BINARY_PATH 照认）
BUILT_BINARY_PATH = re.compile(r"(?:^|/)[^/]*target[^/]*/(?:[^/]+/)?(?:debug|release)/(?:deps/)?[A-Za-z0-9_][A-Za-z0-9_-]*$")


def runs_compiled_code(words, directory):
    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（子 agent 要经 run-with-memory-cap.sh 跑的那一类）：会就交一句说明，不会交 None。
    cargo 的子命令按 test_invocation 认（nextest run、miri、llvm-cov、hack、mutants 同样算），另认 bench 与别名（展开之后看不见，当会跑）。"""
    if os.path.basename(words[0]) == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None:
            return None
        invocation, rest = test_invocation(found.subcommand, found.rest)
        if found.subcommand not in COMPILED_CODE_SUBCOMMANDS and invocation is None and not alias_of_subcommand(found, {}):
            return None
        options = rest[:rest.index("--")] if "--" in rest else rest
        if "--no-run" in options:
            return None
        return f"cargo {found.subcommand}"
    if TEST_BINARY_PATH.search(words[0]) or BUILT_BINARY_PATH.search(words[0]):
        return f"直接执行 cargo 编出来的二进制 {words[0]}"
    return None


# 包在命令外面、之后照样起那条命令的程序（lib_shell_words 那张前缀表之外的）：名字 → 它带一个值的选项。
# 这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
LAUNCHER_OPTIONS_WITH_VALUE = {
    "time": {"-f", "--format", "-o", "--output"},
    "flock": {"-w", "--wait", "--timeout", "-E", "--conflict-exit-code"},
    "chrt": {"-T", "--sched-runtime", "-P", "--sched-period", "-D", "--sched-deadline"},
    "prlimit": {"-o", "--output", "-p", "--pid"},
    "systemd-run": {"-u", "--unit", "-p", "--property", "--description", "--slice", "-E", "--setenv", "-M", "--machine", "-H", "--host",
                    "--uid", "--gid", "--nice", "--working-directory", "--on-active", "--on-boot", "--on-startup", "--on-unit-active",
                    "--on-unit-inactive", "--on-calendar", "--path-property", "--socket-property", "--timer-property", "--service-type",
                    "-C", "--capsule"},
    "strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X"},
    "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
             "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
             "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
    "rustup": set(),
}
# 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
# 给里面那条命令设环境变量的选项（值是 NAME=VALUE）：设的变量写在交回那条命令的最前面
LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}, "strace": {"-E", "--env"}}
# 交回的命令以 NAME=VALUE 打头时，那是设的变量、不是程序（值里的路径 basename 碰巧叫 strace 也不当包装剥）
ENVIRONMENT_ASSIGNMENT_WORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
PERF_SUBCOMMANDS_RUNNING_A_COMMAND = {"stat", "record", "trace"}
PERF_GROUPS_WITH_RECORD = {"mem", "c2c", "sched", "lock", "kmem", "kwork"}


def launcher_option(word, next_word, options_with_value):
    """包装程序的一个选项词：交回 (选项名, 它的值, 这个选项占几个词)。长选项 `--名=值` 与 `--名 值` 按整词查表；
    短选项合写（`-fo 文件`、`-xw 10`、`-o文件`）逐个字母查表，头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；一个要值的字母都没有的只占自己。"""
    if word.startswith("--") or break_is_set("launcher-whole-word-options"):
        option, has_attached_value, attached_value = word.partition("=")
        if has_attached_value:
            return option, attached_value, 1
        return (option, next_word, 2) if option in options_with_value else (option, "", 1)
    for index, letter in enumerate(word[1:], start=1):
        if "-" + letter in options_with_value:
            attached_value = word[index + 1:]
            return ("-" + letter, attached_value, 1) if attached_value else ("-" + letter, next_word, 2)
    return word, "", 1


def command_under_launcher(words):
    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE
    写在交回那条命令的最前面。
    flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。"""
    name = os.path.basename(words[0])
    if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    position = 1
    if name == "rustup":
        if words[1:2] != ["run"]:
            return None
        position = 2
    elif name == "perf":
        if words[1:2] and words[1] in PERF_SUBCOMMANDS_RUNNING_A_COMMAND:
            position = 2
        elif words[1:2] and words[1] in PERF_GROUPS_WITH_RECORD and words[2:3] == ["record"]:
            position = 3
        else:
            return None
    options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
    environment_options = set() if name == "strace" and break_is_set("strace-drops-env") else LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
    while position < len(words):
        word = words[position]
        if word == "--":
            position += 1
            break
        if not word.startswith("-") or word == "-":
            break
        option, value, word_count = launcher_option(word, words[position + 1] if position + 1 < len(words) else "", options_with_value)
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
        if option in environment_options and "=" in value and not break_is_set("launcher-drops-setenv"):
            assignments.append(value)
        position += word_count
    if name == "flock" and position + 1 < len(words) and words[position + 1] in ("-c", "--command"):
        return (["sh", "-c", words[position + 2]], working_directory) if position + 2 < len(words) else None
    position += LAUNCHER_POSITIONAL_COUNT.get(name, 0)
    return (assignments + words[position:], working_directory) if position < len(words) else None


def classify(words, directory, environment=None):
    """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
    name, arguments = os.path.basename(words[0]), words[1:]
    stage = gate_stage_number(words[0], directory)
    if stage is not None:
        return heavy_test(STAGE_KIND[stage], f"门禁 {stage} 号（{name}）") if stage in STAGE_KIND else None
    binary = test_binary_name(words[0])
    if binary is not None:
        if libtest_lists_only(arguments):
            return None
        if "layer0" in binary:
            return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）")
        cases = [case for case in registered_crash_cases(directory) if case.target == binary]
        runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(arguments))
        return crash_case_run("crash-case-binary", f"直接执行登记的崩溃枚举用例的测试二进制（{binary}）", cases, runs_ignored, {}) if cases else None
    if name.startswith("qemu-system"):
        return heavy_test("qemu-system", name)
    if name == "vm-bench.sh":
        return heavy_test("vm-bench", "research/scripts/vm-bench.sh（--selftest 也起虚机）")
    if name == "lkmm.sh":
        return heavy_test("lkmm", ".claude/scripts/lkmm.sh")
    if name == "herd7":
        return heavy_test("herd7", "herd7")
    if name == "mutate.sh":
        for argument in arguments:
            resolved = shell_words.resolve_path(directory, argument) or argument
            if argument.endswith("crates/mutations.tsv") or resolved.endswith("/crates/mutations.tsv"):
                return heavy_test("crates-mutation-mutate", "research/scripts/mutate.sh 跑 crates/mutations.tsv 整表")
        return None
    if name == "check.sh":
        return heavy_test("check-sh", "check.sh（里面是全量 cargo test）")
    if name == "gate.sh":
        return heavy_test("gate-sh", " ".join(["gate.sh", *arguments]))
    if name == "gate-staged.sh":
        return None if "--selftest" in arguments else heavy_test("gate-staged", "research/scripts/gate-staged.sh（跑 gate.sh --staged）")
    if name == "layer0-shard-run.sh":
        return None if "--selftest" in arguments else heavy_test(
            "crash-case-cargo", "research/scripts/layer0-shard-run.sh（两台各跑一片登记的崩溃枚举用例、带 --include-ignored，再本机 merge）")
    if name in ("e152-file-system-benchmark", "e152-run.sh"):
        return heavy_test("e152", name)
    if name == "cargo":
        return cargo_use(arguments, directory, environment)
    return crash_case_named_in_arguments(name, arguments, directory)


def crash_case_named_in_arguments(name, arguments, directory):
    """不认得的命令（按文本处理参数的 grep、git 这一类除外）参数里有 --ignored / --include-ignored、又有登记的用例函数名：
    按跑崩溃枚举用例算（拷走改名的测试二进制、find -exec 起的，名字认不出，靠点名的用例函数认）。"""
    runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(arguments))
    if name in COMMANDS_TREATING_ARGUMENTS_AS_TEXT or not runs_ignored:
        return None
    named = sorted({case.function for case in registered_crash_cases(directory)} & set(arguments))
    if not named:
        return None
    return heavy_test("crash-case-binary", f"{name} 带 {runs_ignored[0]} 与登记的用例函数名 {named[0]}：按跑崩溃枚举用例算")


def heavy_tests_in_command_text(text, directory, environment=None):
    """一段命令文本里命令位置上的每一处重型用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。"""
    found = []
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        launched = command_under_launcher(command.words)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += heavy_tests_in_command_text(shlex.join(inner_words), inner_start, command.environment)
            continue
        test = classify(command.words, command.directory, command.environment)
        if test:
            found.append(test)
    return found


def classify_process(argv, cwd):
    """一个在跑的进程（argv 与 cwd）里认出的重型用法；包装（bash 起脚本、bash -c、nice、timeout、env、capped.sh……）照共用切词剥掉，
    /usr/bin/time、flock、systemd-run 这一类照 command_under_launcher 剥掉。"""
    if not argv:
        return []
    # 进程的 argv[0] 是 time 时它就是那个程序（不是 bash 的关键字 time）：先按 argv 剥，再交给切词
    launched = command_under_launcher(argv)
    if launched is not None:
        inner_words, inner_directory = launched
        return classify_process(inner_words, shell_words.resolve_path(cwd, inner_directory) if inner_directory else cwd)
    return heavy_tests_in_command_text(shlex.join(argv), cwd)


def judged_by_name(word, directory):
    """按名字判的仓内脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：判定在名字那一格做完，不再读脚本正文。"""
    if gate_stage_number(word, directory) is not None:
        return True
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    return any(candidate == location or candidate.endswith("/" + location)
               for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""))


def build_sample_workspace(work):
    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记七条崩溃枚举用例：
    harness 里的 sample_crash_enumeration，与只有它一个测试目标的包 singlefs-checker 里的 checker_crash_enumeration（用例函数都标了 #[ignore]），
    harness 里用例函数没标 #[ignore] 的 unmarked_crash_enumeration；harness 里另四种写法：同名函数 cfg 二选一、一份不标（cfg_split），
    子模块里同名标 ignore 的写在前面、顶层那一份不标（submodule_first），宏生成的用例（macro，没有字面的 fn <名>(），
    `# [ignore]`（# 与 [ 之间有空格，spaced_ignore，算标了）。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness", "crates/singlefs-checker"]\n'
                        'exclude = ["research"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
    write("crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs", "")
    write("crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs", "")
    marked_case = '#[test]\nfn quick_case() {}\n#[test]\n#[ignore = "崩溃枚举（样本）：平时不跑"]\nfn the_full_case() {}\n'
    write("crates/singlefs-harness/tests/sample_crash_enumeration.rs", marked_case)
    write("crates/singlefs-harness/tests/unmarked_crash_enumeration.rs", "#[test]\nfn the_unmarked_case() {}\n")
    write("crates/singlefs-harness/tests/cfg_split_crash_enumeration.rs",
          "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_split_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_split_case() {}\n")
    write("crates/singlefs-harness/tests/submodule_first_crash_enumeration.rs",
          "mod slow {\n    #[test]\n    #[ignore]\n    fn the_shadowed_case() {}\n}\n#[test]\nfn the_shadowed_case() {}\n")
    write("crates/singlefs-harness/tests/macro_crash_enumeration.rs",
          "macro_rules! crash_case {\n    ($name:ident) => {\n        #[test]\n        fn $name() {}\n    };\n}\ncrash_case!(the_generated_case);\n")
    write("crates/singlefs-harness/tests/spaced_ignore_crash_enumeration.rs", "#[test]\n# [ignore]\nfn the_spaced_case() {}\n")
    write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
    write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", marked_case)
    write(CRASH_CASE_REGISTRY, "# 样本登记表\n54-layer0-replay.sh\tcrates/\t# 样本\n"
                               "crash-case:sample\tcrates/ Cargo.toml\ttest=singlefs-harness:sample_crash_enumeration:the_full_case\t# 样本\n"
                               "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n"
                               "crash-case:unmarked\tcrates/ Cargo.toml\ttest=singlefs-harness:unmarked_crash_enumeration:the_unmarked_case\t# 样本\n"
                               "crash-case:cfg-split\tcrates/ Cargo.toml\ttest=singlefs-harness:cfg_split_crash_enumeration:the_split_case\t# 样本\n"
                               "crash-case:submodule-first\tcrates/ Cargo.toml\t"
                               "test=singlefs-harness:submodule_first_crash_enumeration:the_shadowed_case\t# 样本\n"
                               "crash-case:macro\tcrates/ Cargo.toml\ttest=singlefs-harness:macro_crash_enumeration:the_generated_case\t# 样本\n"
                               "crash-case:spaced-ignore\tcrates/ Cargo.toml\ttest=singlefs-harness:spaced_ignore_crash_enumeration:the_spaced_case\t# 样本\n")
    write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
    write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')


def selftest():
    """在临时工作区里判 cargo 命令行、直接执行的测试二进制与按名字判的脚本；返回退出码。"""
    work = tempfile.mkdtemp(prefix="lib-heavy-tests-")
    results = []
    try:
        build_sample_workspace(work)
        harness = os.path.join(work, "crates", "singlefs-harness")
        layer0_binary = "first_transaction_step_seven_layer0-0123456789abcdef"
        # (说明, 进程的 argv, 进程的 cwd, 该认出的 kind；None 是不重型)
        cargo_cases = [
            ("cargo test --all", ["cargo", "test", "--all"], work, "full-cargo"),
            ("argv[0] 是 cargo 的全路径", ["/home/user/.rustup/toolchains/stable/bin/cargo", "test", "--workspace"], work, "full-cargo"),
            ("在工作区根裸跑", ["cargo", "test"], work, "full-cargo"),
            ("-p harness 点名层 0 目标", ["cargo", "test", "--release", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0"], work, "layer0-cargo"),
            ("在 harness 里不挑目标", ["cargo", "test", "--release"], harness, "layer0-cargo"),
            ("经 nice 包一层", ["/usr/bin/nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("bash -c 里", ["bash", "-c", "cd crates/singlefs-harness && cargo test"], work, "layer0-cargo"),
            ("bash 起 54 号", ["bash", ".claude/gate.d/54-layer0-replay.sh", "--full", "/tmp/wt"], work, "layer0-stage"),
            ("bash 起双机分片的驱动脚本", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:sample", "/tmp/wt"], work, "crash-case-cargo"),
            ("双机分片的驱动脚本 --merged-log", ["bash", "research/scripts/layer0-shard-run.sh", "--merged-log", "crash-case:sample", "/tmp/wt", "0" * 64, "/tmp/log"],
             work, "crash-case-cargo"),
            ("双机分片的驱动脚本 --selftest 不重型", ["bash", "research/scripts/layer0-shard-run.sh", "--selftest"], work, None),
            ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
            ("一个不是层 0 的测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            ("clippy --all-targets", ["cargo", "clippy", "--all-targets"], work, None),
            ("bash 起轻阶段 12 号", ["bash", ".claude/gate.d/12-no-prime-marks.sh"], work, None),
            ("点名崩溃枚举用例、带 --include-ignored --exact",
             ["cargo", "test", "--release", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--exact", "the_full_case"],
             work, "crash-case-cargo"),
            ("点名崩溃枚举用例、带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work,
             "crash-case-cargo"),
            ("--test 通配命中崩溃枚举用例、带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_*", "--", "--ignored"], work,
             "crash-case-cargo"),
            ("只有崩溃枚举用例的包不挑目标、带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--", "--ignored"], work, "crash-case-cargo"),
            ("点名崩溃枚举用例、不带 --ignored（只跑它的快用例）", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"], work, None),
            ("点名崩溃枚举用例、libtest 参数只有 --nocapture",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--nocapture"], work, None),
            ("只有崩溃枚举用例的包 --lib 带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--lib", "--", "--ignored"], work, None),
            ("别的测试目标带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "second_transaction_step_one_overwrite", "--", "--ignored"],
             work, None),
            # 用例函数没标 #[ignore]：不带 --ignored 也跑到全量（读源码判，与 admission.py crash-cases 同一套）
            ("点名用例函数没标 #[ignore] 的登记目标、不带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "unmarked_crash_enumeration"],
             work, "crash-case-cargo"),
            # 只列用例、一条都不跑的不算
            ("点名崩溃枚举用例、--ignored --list", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--list"],
             work, None),
            ("点名层 0 目标、-- --list", ["cargo", "test", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0", "--", "--list"], work, None),
            # 起测试的外部子命令与包装子命令
            ("cargo nextest run --run-ignored all 点名崩溃枚举用例",
             ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "all"], work, "crash-case-cargo"),
            ("cargo nextest run --run-ignored=only -E 过滤式、只有崩溃枚举用例的包",
             ["cargo", "nextest", "run", "-p", "singlefs-checker", "--run-ignored=only", "-E", "test(=the_full_case)"], work, "crash-case-cargo"),
            ("cargo nextest run --run-ignored default 只跑快用例",
             ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "default"], work, None),
            ("cargo nextest run 在工作区根裸跑", ["cargo", "nextest", "run"], work, "full-cargo"),
            ("cargo nextest list 不起测试", ["cargo", "nextest", "list", "--workspace"], work, None),
            ("cargo mutants 整包（反复跑 cargo test，包里有层 0 目标）", ["cargo", "mutants", "-p", "singlefs-harness"], work, "layer0-cargo"),
            ("cargo mutants 尾参交给 cargo test、带 --include-ignored",
             ["cargo", "mutants", "-p", "singlefs-checker", "--", "--test", "checker_crash_enumeration", "--", "--include-ignored"], work, "crash-case-cargo"),
            ("cargo miri test 点名崩溃枚举用例、带 --ignored",
             ["cargo", "miri", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
            ("cargo llvm-cov 不带子命令、点名崩溃枚举用例带 --ignored",
             ["cargo", "llvm-cov", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
            ("cargo llvm-cov report 不起测试", ["cargo", "llvm-cov", "report"], work, None),
            ("cargo hack 在子命令前带自己的选项、test --workspace", ["cargo", "hack", "--each-feature", "test", "--workspace"], work, "full-cargo"),
            # 别名与 runner：展开之后带什么参数看不见，当可能带 --ignored
            ("--config 定别名再用别名、点名崩溃枚举用例", ["cargo", "--config", 'alias.xt="test"', "xt", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"],
             work, "crash-case-cargo"),
            ("--config= 写法定的 runner、点名崩溃枚举用例不带 --ignored",
             ["cargo", '--config=target.x86_64-unknown-linux-gnu.runner=["/tmp/add-ignored.sh"]', "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("环境变量里的 runner、点名崩溃枚举用例不带 --ignored",
             ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("环境变量 CARGO_ALIAS_XT 定的别名", ["env", "CARGO_ALIAS_XT=test", "cargo", "xt", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"],
             work, "crash-case-cargo"),
            ("--config 定了别名、子命令不是它（cargo build）", ["cargo", "--config", 'alias.xt="test"', "build"], work, None),
            ("环境变量里有 runner、cargo build 不跑测试", ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/x", "cargo", "build", "-p", "singlefs-harness"],
             work, None),
            ("环境变量里有 runner、点名不是崩溃枚举用例的目标", ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/x", "cargo", "test", "-p", "singlefs-core",
                                                  "--test", "core_contract"], work, None),
            # --config 的值按 TOML 读：带引号的键、点号两边带空白的 runner 正则认不出（开关 runner-configuration-by-regex-only 下这两格红）
            ("--config 里带引号的键 target.<三元组>.\"runner\"、点名崩溃枚举用例不带 --ignored",
             ["cargo", "--config", 'target.x86_64-unknown-linux-gnu."runner"="/tmp/add-ignored.sh"', "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("--config 里点号两边带空白的 target . <三元组> . runner",
             ["cargo", "--config", 'target . x86_64-unknown-linux-gnu . runner = "/tmp/add-ignored.sh"', "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("--config 定的是 build.jobs、不是 runner", ["cargo", "--config", "build.jobs=4", "test", "-p", "singlefs-harness", "--test",
                                                    "sample_crash_enumeration"], work, None),
            # --list 是 --skip、--logfile 这类带值选项的值时照样全跑（开关 list-by-presence 下前两格红）
            ("点名崩溃枚举用例、-- --include-ignored --skip --list（--list 是 --skip 的值）",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--skip", "--list"], work, "crash-case-cargo"),
            ("点名崩溃枚举用例、-- --include-ignored --exact the_full_case --logfile --list（--list 是日志文件名）",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--exact", "the_full_case",
              "--logfile", "--list"], work, "crash-case-cargo"),
            ("点名崩溃枚举用例、-- --ignored --test-threads 4 --list（只列）",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--test-threads", "4", "--list"], work, None),
            ("点名崩溃枚举用例、-- --ignored --skip=--list（一个词，没有 --list 选项）",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--skip=--list"], work, "crash-case-cargo"),
            # 用例函数的每一处定义都要标 #[ignore]；判不出标没标的按没标算（开关：admission.py 的 first-definition-only、ignore-attribute-without-space，
            # 这里的 undecided-ignore-allowed）
            ("点名同名函数 cfg 二选一、一份不标的登记目标、不带 --ignored", ["cargo", "test", "--release", "-p", "singlefs-harness", "--test",
                                                          "cfg_split_crash_enumeration"], work, "crash-case-cargo"),
            ("点名子模块里同名标 ignore 写在前面、顶层那一份不标的登记目标、不带 --ignored",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "submodule_first_crash_enumeration"], work, "crash-case-cargo"),
            ("点名用例是宏生成的（找不到字面的 fn <名>(）登记目标、不带 --ignored：判不出按没标算",
             ["cargo", "test", "-p", "singlefs-harness", "--test", "macro_crash_enumeration"], work, "crash-case-cargo"),
            ("点名用例函数标的是 # [ignore]（# 与 [ 之间有空格）的登记目标、不带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test",
                                                                      "spaced_ignore_crash_enumeration"], work, None),
        ]
        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）
        launcher_cases = [
            ("/usr/bin/time -v 包一层、点名崩溃枚举用例带 --ignored",
             ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
            ("time -f 格式 -o 文件包一层层 0 目标",
             ["time", "-f", "%M", "-o", "/tmp/t", "cargo", "test", "--release", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0"],
             work, "layer0-cargo"),
            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "cargo test --workspace"], work, "full-cargo"),
            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", "cargo", "test", "--all"], work, "full-cargo"),
            ("chrt -i 0 包一层", ["chrt", "-i", "0", "cargo", "test", "--all"], work, "full-cargo"),
            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", "cargo", "test", "--all"],
             work, "full-cargo"),
            ("systemd-run --working-directory= 进 harness 裸跑",
             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-harness", "cargo", "test"], work, "layer0-cargo"),
            ("strace -f -o 文件包一层", ["strace", "-f", "-o", "/tmp/trace", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf stat -e 事件包一层", ["perf", "stat", "-e", "cycles", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf record -g -- 之后", ["perf", "record", "-g", "--", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf mem record 包一层", ["perf", "mem", "record", "cargo", "test", "--all"], work, "full-cargo"),
            ("nice 里 /usr/bin/time 里 flock 一层套一层", ["nice", "-n", "19", "/usr/bin/time", "-v", "flock", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -p 进程号：没有要起的命令", ["strace", "-p", "1234"], work, None),
            ("chrt -p 查优先级：没有要起的命令", ["chrt", "-p", "1234"], work, None),
            ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
            ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
            ("/usr/bin/time 包一个不重型的测试目标", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            # 短选项合写：逐个字母查表，头一个要值的字母之后剩下的是值、没剩下取下一个词（开关 launcher-whole-word-options 下前四格红）
            ("strace -fo 文件（短选项合写）", ["strace", "-fo", "/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            ("flock -xw 10 锁文件（短选项合写）", ["flock", "-xw", "10", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope -qu 名字（短选项合写）", ["systemd-run", "--user", "--scope", "-qu", "k1", "cargo", "test", "--all"], work, "full-cargo"),
            ("/usr/bin/time -ao 文件（短选项合写）", ["/usr/bin/time", "-ao", "/tmp/t", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -o文件（值贴着写）", ["strace", "-o/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            # systemd-run -E / --setenv 设的变量带进里面那条命令（开关 launcher-drops-setenv 下前三格红）
            ("systemd-run -E 设 runner、点名崩溃枚举用例不带 --ignored",
             ["systemd-run", "--user", "--scope", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("systemd-run --setenv= 设 runner、点名崩溃枚举用例不带 --ignored",
             ["systemd-run", "--user", "--scope", "--setenv=CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("systemd-run -qE 合写设 runner、点名崩溃枚举用例不带 --ignored",
             ["systemd-run", "--user", "--scope", "-qE", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("systemd-run -E 设的不是 runner（值的末段叫 strace 也不当包装剥）、点名崩溃枚举用例不带 --ignored",
             ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", "cargo", "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, None),
            # strace -E / --env 设的变量同样带进里面那条命令（开关 strace-drops-env 下前四格红）
            ("strace -E 设 runner、点名崩溃枚举用例不带 --ignored",
             ["strace", "-f", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("strace --env= 设 runner、点名崩溃枚举用例不带 --ignored",
             ["strace", "--env=CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("strace --env 值另起一个词、设 runner、点名崩溃枚举用例不带 --ignored",
             ["strace", "--env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("strace -fE 合写设 runner、点名崩溃枚举用例不带 --ignored",
             ["strace", "-fE", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
            ("strace -E 只清变量（不带 =）、点名崩溃枚举用例不带 --ignored",
             ["strace", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER", "cargo", "test", "-p", "singlefs-harness", "--test",
              "sample_crash_enumeration"], work, None),
        ]
        binary_cases = [
            ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
            ("相对路径的层 0 测试二进制", [os.path.join("target", "debug", "deps", layer0_binary)], work, "layer0-binary"),
            ("带目标三元组、自定的 target 目录", ["/tmp/target-elsewhere/x86_64-unknown-linux-gnu/release/deps/some_layer0_stream-fedcba9876543210"], work, "layer0-binary"),
            ("经 timeout 包一层", ["timeout", "600", os.path.join(".", "target", "release", "deps", layer0_binary), "--exact", "case"], work, "layer0-binary"),
            ("名字不含 layer0 的测试二进制", [os.path.join(work, "target", "release", "deps", "second_transaction_step_one_overwrite-0123456789abcdef")], work, None),
            ("deps 下的 .d 依赖文件不是二进制", [os.path.join(work, "target", "release", "deps", layer0_binary + ".d")], work, None),
            ("哈希不是 16 位", [os.path.join(work, "target", "release", "deps", "first_transaction_step_seven_layer0-0123abcd")], work, None),
            ("名字只当参数", ["grep", "-c", "x", os.path.join("target", "release", "deps", layer0_binary)], work, None),
            ("崩溃枚举用例的测试二进制带 --include-ignored",
             [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"), "--include-ignored", "--exact", "the_full_case"],
             work, "crash-case-binary"),
            ("崩溃枚举用例的测试二进制不带 --ignored", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef")], work, None),
            ("用例函数没标 #[ignore] 的登记用例的测试二进制、不带 --ignored",
             [os.path.join(work, "target", "release", "deps", "unmarked_crash_enumeration-0123456789abcdef")], work, "crash-case-binary"),
            ("崩溃枚举用例的测试二进制 --ignored --list", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"),
                                                  "--ignored", "--list"], work, None),
            ("层 0 测试二进制 --list", [os.path.join(work, "target", "release", "deps", layer0_binary), "--list"], work, None),
            ("崩溃枚举用例的测试二进制 --include-ignored --skip --list（--list 是 --skip 的值）",
             [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"), "--include-ignored", "--skip", "--list"],
             work, "crash-case-binary"),
            ("层 0 测试二进制 --logfile --list（--list 是日志文件名）", [os.path.join(work, "target", "release", "deps", layer0_binary), "--logfile", "--list"],
             work, "layer0-binary"),
            # 名字认不出（拷走改名、find -exec）时按参数认：带 --ignored 又点名登记的用例函数
            ("拷走改名的测试二进制带 --ignored --exact 登记的用例函数", ["/tmp/elsewhere/rc", "--ignored", "--exact", "the_full_case"], work, "crash-case-binary"),
            ("find -exec 执行测试二进制、点名登记的用例函数",
             ["find", "target", "-name", "sample_crash_enumeration-*", "-exec", "{}", "--ignored", "the_full_case", ";"], work, "crash-case-binary"),
            ("拷走改名的测试二进制带 --ignored、不点名用例函数：认不出（文件头「看不见的」）", ["/tmp/elsewhere/rc", "--ignored"], work, None),
            ("grep 找 --include-ignored 与用例函数名：按文本处理参数的不算", ["grep", "-rn", "-e", "--include-ignored", "-e", "the_full_case", "crates"],
             work, None),
        ]
        # 名字不含 layer0 的那几条：含 layer0 的按层 0 那一类先认出来，判不出登记表取没取到
        own_repository_targets = sorted(target for _package, target in registered_crash_case_targets(HOOK_REPOSITORY) if "layer0" not in target)
        if own_repository_targets:
            binary_cases.append(("仓外执行、登记表取这份文件所在仓的那一份：那里登记的崩溃枚举用例带 --ignored",
                                 [f"/tmp/target-elsewhere/release/deps/{own_repository_targets[0]}-0123456789abcdef", "--ignored"], "/",
                                 "crash-case-binary"))
        else:
            results.append((f"这份文件所在仓的登记表（{os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)}）里有名字不含 layer0 的崩溃枚举用例",
                            True, False))
        for label, argv, cwd, want in cargo_cases + launcher_cases + binary_cases:
            found = classify_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
        # 导入不了 admission.py 时判不出标没标：点名登记目标、不带 --ignored 的 cargo test 按没标算（开关 undecided-ignore-allowed 下这一格红）
        module_globals, saved_module_path = globals(), ADMISSION_MODULE_PATH
        module_globals["ADMISSION_MODULE_PATH"] = os.path.join(work, "no-such-admission.py")
        admission_module.cache_clear()
        try:
            found = classify_process(["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"], work)
        finally:
            module_globals["ADMISSION_MODULE_PATH"] = saved_module_path
            admission_module.cache_clear()
        results.append(("导入不了 admission.py：点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 按没标算", "crash-case-cargo",
                        found[0].kind if found else None))
        name_cases = [
            ("门禁阶段按名字判", ".claude/gate.d/12-no-prime-marks.sh", work, True),
            ("仓内位置上的 gate-staged.sh 按名字判", "research/scripts/gate-staged.sh", work, True),
            ("仓内位置上的 layer0-shard-run.sh 按名字判", "research/scripts/layer0-shard-run.sh", work, True),
            ("从 research 里相对着写的 mutate.sh", "scripts/mutate.sh", os.path.join(work, "research"), True),
            ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
            ("没登记的脚本", "run-chain.sh", work, False),
        ]
        for label, word, directory, want in name_cases:
            results.append((label, want, judged_by_name(word, directory)))
        # 跑不跑编译出来的代码（与重型不重型无关）：真 / 假
        compiled_cases = [
            ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
            ("cargo nextest run", ["cargo", "nextest", "run", "-p", "singlefs-core"], True),
            ("cargo miri test", ["cargo", "miri", "test", "-p", "singlefs-core"], True),
            ("cargo llvm-cov 不带子命令", ["cargo", "llvm-cov", "-p", "singlefs-core"], True),
            ("cargo mutants", ["cargo", "mutants", "-p", "singlefs-core"], True),
            ("--config 定的别名", ["cargo", "--config", 'alias.xt="test"', "xt"], True),
            ("cargo nextest list 不跑", ["cargo", "nextest", "list"], False),
            ("cargo llvm-cov report 不跑", ["cargo", "llvm-cov", "report"], False),
            ("cargo t 简写", ["cargo", "t", "--lib"], True),
            ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
            ("cargo r 简写", ["cargo", "r"], True),
            ("cargo bench", ["cargo", "bench"], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "test"], True),
            ("cargo test --no-run 只编不跑", ["cargo", "test", "--no-run", "-p", "singlefs-core"], False),
            ("-- 之后的 --no-run 是测试二进制的参数", ["cargo", "test", "--", "--no-run"], True),
            ("cargo build", ["cargo", "build", "--release"], False),
            ("cargo b 简写是 build", ["cargo", "b"], False),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "second_transaction_step_one_overwrite-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["research/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/first_transaction_on_device"], True),
            ("deps 下的 .d 依赖文件", [os.path.join("target", "release", "deps", "tiny.d")], False),
            ("名字只当参数", ["ls", "target/release/tiny"], False),
            ("不在 target 底下的程序", ["./prog"], False),
        ]
        for label, words, want in compiled_cases:
            results.append((f"跑编译出来的代码：{label}", want, runs_compiled_code(words, work) is not None))
    finally:
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ lib_heavy_tests 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_invocation()、crash_case_run()、command_under_launcher()、launcher_option()、"
              "libtest_lists_only()、runner_of_test_binaries()、crash_case_function_runs_without_ignored()、test_binary_name()、"
              "runs_compiled_code()、classify_process()、judged_by_name() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
              "KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / LIBTEST_OPTIONS_WITH_VALUE 几张表；判用例函数标没标 #[ignore] 的那一段在 "
              f"research/scripts/admission.py；{BREAK_VARIABLE} 或 ADMISSION_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种、"
          f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种、"
          "导入不了 admission.py 1 种）")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_heavy_tests.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
