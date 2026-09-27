"""一条命令是不是重型测试、属于哪一类。

heavy-test-guard.sh（执行前拒绝）与看门狗（research/scripts/agent-watch.py，在进程这一层）共用这一份判定，各自按文件路径用 importlib 导入；
同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑哪一类、要带什么前缀，是 heavy-test-guard.sh 的事，不在这里。

入口：
  classify(words, directory, environment=None) -> HeavyTest | None
      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None；
      environment 是这条命令看得到的、命令文本里设过的变量（lib_shell_words 的 CommandAtPosition.environment），认别名要用。
  command_under_launcher(words) -> (list[str], str | None) | None
      words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
      剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
      短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
      长选项查要值与不要值两张表（照各自的 --help 列全），值另起一个词的（`--expand-environment no`、`--output <文件>`）跳过那个值，
      只写前缀的（getopt_long 认唯一的前缀，`--outp`）按前缀认；
      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE`、`-p Environment=…` / `--property=Environment=…`（`EnvironmentFile=<文件>` 按内容读，
      读不了当它没设）、strace 的 `-E NAME=VALUE` / `--env=NAME=VALUE` 设的变量写在交回那条命令的最前面
      （再切一遍时当它的环境变量，认别名要用；strace 的 `-E NAME` 只清变量，不带）。
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
qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`。
**checker 档按包判，是唯一的判法**（D13（验证路线） 已定项 15，.claude/rules/verification.md「定义与名字」）：cargo 起测试、范围里有
checker 档包 singlefs-checker-tier（-p 点名它，-p 的值认光名字、name@version、pkgid URL 与通配；不挑包而范围含它；当前目录在它里面裸跑），
不管挑哪个目标（--lib、--doc、--bin、--test、--tests 都算：整个包都是 checker 档，库单测与装置二进制的内联单测也在内），算重型；
直接执行它的测试二进制（名字是它 tests/ 下的测试目标、src/bin/ 下的装置、或库本身 singlefs_checker_tier）也算。
harness 档（singlefs-harness）、core、format、池级 checker（singlefs-checker）的测试随时跑，不算重型；工作区全量（--workspace / --all、
工作区根裸跑、范围是全部成员）算「全量测试」。
别名：--config 里 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的、子命令就是它的，看得到值就展开再判；值读不出（TOML 写坏了）按 checker 档算。
libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
--test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列；
libtest 在 `--` 处停止认选项，`--` 之后的 --list 是过滤词，照样全跑，不算只列。
看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名；拷走改名的 checker 档测试二进制；systemd-run -p EnvironmentFile= 指到读不了的文件时里面定没定别名；
cargo mutants -d 指到别处的树。
弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 heavy-test-guard.sh --selftest 都必须判红：
  launcher-whole-word-options（短选项合写不拆，照旧按整词查表）、launcher-drops-setenv（systemd-run -E / --setenv 设的变量不带进里面那条命令）、
  list-by-presence（见到 --list 这个词就算只列）、strace-drops-env（strace -E / --env 设的变量不带进里面那条命令）、
  launcher-long-options-partial（要值的长选项照补全之前的表、不认前缀）、alias-not-expanded（--config / CARGO_ALIAS_ 定的别名看得到值也不展开）、
  package-specification-literal（-p 的值只按字面名比，不认通配、@版本、pkgid URL）、launcher-drops-property-environment（systemd-run -p Environment= /
  EnvironmentFile= 设的变量不带进里面那条命令）、list-past-separator（找 --list 时见到 `--` 不停）、configuration-before-subcommand-only（只收子命令之前的 --config）、mutants-configuration-ignored（cargo mutants 不看 --test-package、--test-workspace 与 .cargo/mutants.toml 的 test_workspace）、no-run-counted-as-running（`--no-run` 只编译也按跑了算）。
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
    kind: str      # 细分的用法（checker-tier-cargo、gate-sh……），heavy-test-guard.sh 按它定谁能跑
    category: str  # 报给人看的类名（层 0、全量测试……），KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 每一种重型用法：kind → 它属于哪一类
KIND_CATEGORY = {
    "checker-tier-cargo": "checker 档", "checker-tier-binary": "checker 档",
    "layer0-stage": "层 0",
    "qemu-stage": "QEMU", "qemu-system": "QEMU", "vm-bench": "QEMU",
    "herd7-stage": "herd7", "lkmm": "herd7", "herd7": "herd7",
    "crates-mutation-stage": "crates 变异整表", "crates-mutation-mutate": "crates 变异整表",
    "full-cargo": "全量测试", "check-sh": "全量测试",
    "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
    "e152": "E152 装置",
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
    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算；
    见到 `--` 就停、交「不是只列」：libtest 在 `--` 处停止认选项，之后的 --list 是过滤词（`-- --ignored -- --list` 一条都不跑，照样算不是只列，接受的误拒）。"""
    if break_is_set("list-by-presence"):
        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
    position = 0
    while position < len(libtest_arguments):
        argument = libtest_arguments[position]
        if argument == "--" and not break_is_set("list-past-separator"):
            return False
        if argument == LIST_ONLY_TEST_ARGUMENT:
            return True
        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
    return False


CHECKER_PACKAGE_NAME = "singlefs-checker-tier"   # checker 档住的包（D13 已定项 15）：它的集成测试、库单测与装置二进制都算重型
HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# libtest 只列用例、一条都不跑的参数
LIST_ONLY_TEST_ARGUMENT = "--list"
# libtest 带一个值的选项：`--skip --list` 里的 --list 是 --skip 的值（照样全跑），不是「只列」
LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}
# 弄坏开关的环境变量（写法见文件头）
BREAK_VARIABLE = "LIB_HEAVY_TESTS_BREAK"
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


def integration_test_targets(package_directory):
    """包里全部集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
    names = set()
    for path in glob.glob(os.path.join(package_directory, "tests", "*.rs")):
        names.add(os.path.basename(path)[:-3])
    for path in glob.glob(os.path.join(package_directory, "tests", "*", "main.rs")):
        names.add(os.path.basename(os.path.dirname(path)))
    for target in (manifest_sections(os.path.join(package_directory, "Cargo.toml")) or {}).get("test") or []:
        if isinstance(target, dict) and target.get("name"):
            names.add(target["name"])
    return sorted(names)


def checker_package_directories(directory):
    """directory 往上找到的第一个含 crates/<checker 档包> 的根里那个包目录；一个都找不到（仓外执行）才取本 hook 所在仓的那一个。"""
    while directory:
        candidate = os.path.join(directory, "crates", CHECKER_PACKAGE_NAME)
        if os.path.isdir(candidate):
            return [candidate]
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    own = os.path.join(HOOK_REPOSITORY, "crates", CHECKER_PACKAGE_NAME)
    return [own] if os.path.isdir(own) else []


def checker_test_targets(directory):
    """checker 档包的测试二进制名：tests/ 下的测试目标、src/bin/ 下的装置（它们的内联单测编成同名测试二进制）、库本身（包名把 - 换成 _）。"""
    names = set()
    for package_directory in checker_package_directories(directory):
        names.update(integration_test_targets(package_directory))
        names.update(os.path.splitext(os.path.basename(path))[0] for path in glob.glob(os.path.join(package_directory, "src", "bin", "*.rs")))
        names.add(CHECKER_PACKAGE_NAME.replace("-", "_"))
    return names


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path",
                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
NARROWING_SELECTORS = {"--test", "--lib", "--bin", "--bins", "--example", "--examples", "--bench", "--benches", "--doc"}
GLOB_CHARACTERS = set("*?[")
# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
# cargo mutants 的选项里 cargo test 也认、照搬过去的（选包的那几个）
MUTANTS_PACKAGE_OPTIONS_WITH_VALUE = {"-p", "--package", "--manifest-path"}
ALIAS_CONFIGURATION = re.compile(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=")
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
    rest = arguments[index + 1:]
    if not break_is_set("configuration-before-subcommand-only"):
        configuration_values += configuration_values_after_subcommand(rest)
    return CargoCommand(arguments[index], rest, directory, configuration_values)


def configuration_values_after_subcommand(rest):
    """子命令之后、`--` 之前的 --config 值（--config 是 cargo 的全局选项，写在 test 之后照认）：--config <值> 与 --config=<值> 两种写法；
    按 cargo test 的带值选项表跳过别的选项的值（`--features --config` 里的 --config 是 --features 的值）。"""
    values, position = [], 0
    while position < len(rest):
        word = rest[position]
        if word == "--":
            break
        if word == "--config":
            if position + 1 < len(rest):
                values.append(rest[position + 1])
            position += 2
        elif word.startswith("--config="):
            values.append(word[len("--config="):])
            position += 1
        else:
            position += 2 if word in CARGO_TEST_OPTIONS_WITH_VALUE else 1
    return values


MUTANTS_TEST_WORKSPACE_SETTING = re.compile(r"^\s*test_workspace\s*=\s*(true|false)\b", re.MULTILINE)


def mutants_configured_test_workspace(source_directory):
    """源码树里 .cargo/mutants.toml 的 test_workspace（True / False），没有这份配置或没写这一项交 None。
    cargo mutants 从源码树（-d 给的目录，不给就是当前目录）往上找工作区根，读那里的 .cargo/mutants.toml；这里从 source_directory 往上逐级找第一份。"""
    current = os.path.abspath(source_directory or ".")
    while True:
        path = os.path.join(current, ".cargo", "mutants.toml")
        if os.path.isfile(path):
            try:
                found = MUTANTS_TEST_WORKSPACE_SETTING.search(open(path, encoding="utf-8").read())
            except (OSError, UnicodeDecodeError):
                return None
            return None if found is None else found.group(1) == "true"
        parent = os.path.dirname(current)
        if parent == current:
            return None
        current = parent


def mutants_test_arguments(rest, directory=None):
    """cargo mutants 的参数 → 它反复起的 cargo test 的参数。选包的选项（-p / --package / --manifest-path / --workspace）照搬，`--` 之后的原样接上；
    测哪些包另看三样（cargo mutants 的 --test-workspace / --test-package 与 .cargo/mutants.toml 的 test_workspace）：
    --test-package X 测的是 X（换成 -p X）；--test-workspace=true 或配置里 test_workspace = true（没带 --no-config、没被 --test-workspace=false 盖掉）
    测整个工作区（加 --workspace）。-d / --dir 给的源码树换掉当前目录去找配置与工作区。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (rest, [])
    kept, test_packages, test_workspace_option, source_directory, reads_configuration = [], [], None, directory, True
    index = 0
    while index < len(before):
        word = before[index]
        if word in MUTANTS_PACKAGE_OPTIONS_WITH_VALUE:
            kept += before[index:index + 2]
            index += 2
            continue
        if word in ("-d", "--dir") and index + 1 < len(before):
            source_directory = shell_words.resolve_path(directory, before[index + 1]) if directory else before[index + 1]
            index += 2
            continue
        if word.startswith("--dir="):
            value = word.split("=", 1)[1]
            source_directory = shell_words.resolve_path(directory, value) if directory else value
        elif word == "--test-package" and index + 1 < len(before):
            test_packages += [name for name in before[index + 1].split(",") if name]
            index += 2
            continue
        elif word.startswith("--test-package="):
            test_packages += [name for name in word.split("=", 1)[1].split(",") if name]
        elif word == "--test-workspace" and index + 1 < len(before) and before[index + 1] in ("true", "false"):
            test_workspace_option = before[index + 1] == "true"
            index += 2
            continue
        elif word.startswith("--test-workspace="):
            test_workspace_option = word.split("=", 1)[1] == "true"
        elif word == "--no-config":
            reads_configuration = False
        elif word == "--workspace" or word.startswith(("--package=", "--manifest-path=")) or (word.startswith("-p") and len(word) > 2):
            kept.append(word)
        index += 1
    if break_is_set("mutants-configuration-ignored"):
        return kept + after
    if source_directory is not None and source_directory != directory and not any(
            word == "--manifest-path" or word.startswith("--manifest-path=") for word in kept):
        kept += ["--manifest-path", os.path.join(source_directory, "Cargo.toml")]
    if test_packages:
        kept = [word for position, word in enumerate(kept)
                if not (word in ("-p", "--package") or word.startswith(("--package=",)) or (word.startswith("-p") and len(word) > 2)
                        or (position > 0 and kept[position - 1] in ("-p", "--package")))]
        for name in test_packages:
            kept += ["-p", name]
        return kept + after
    test_workspace = test_workspace_option
    if test_workspace is None and reads_configuration:
        test_workspace = mutants_configured_test_workspace(source_directory)
    if test_workspace:
        kept.append("--workspace")
    return kept + after


def test_invocation(subcommand, rest, directory=None):
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
        return "test", mutants_test_arguments(rest, directory)
    return None, []


def package_specification_members(specification, members):
    """-p 的一个值对上的工作区成员名（集合）：认 cargo 的几种包规格——光名字、name@version、pkgid URL（path+file://…#name@version、…#name）、
    通配（*、?、[…]，cargo 1.64 起 -p 认 glob）。一个都对不上交空集。"""
    text = specification.strip().strip("'\"")
    if "#" in text:
        text = text.rsplit("#", 1)[1]
    text = text.split("@", 1)[0]
    if GLOB_CHARACTERS & set(text):
        return set(fnmatch.filter(list(members), text))
    return {text} if text in members else set()


def alias_expansion(command, environment):
    """别名展开成的那几个词：--config 里 alias.<名> = "…" 或环境变量 CARGO_ALIAS_<名> 的值，按空白切；看不到值交 None。"""
    for value in command.configuration_values:
        match = re.match(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=\s*(.+?)\s*$", value)
        if match and match.group(1) == command.subcommand:
            raw = match.group(2).strip()
            try:
                parsed = tomllib.loads(f"value = {raw}")["value"]
            except (tomllib.TOMLDecodeError, KeyError):
                return None
            return parsed if isinstance(parsed, list) else str(parsed).split()
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment and environment[alias_variable]:
        return environment[alias_variable].split()
    return None


def alias_of_subcommand(command, environment):
    """子命令是 --config 的 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的别名时交一句（展开之后是什么、带什么参数都看不见，当 cargo test 判）；不是交 None。"""
    aliases = {match.group(1) for value in command.configuration_values for match in [ALIAS_CONFIGURATION.match(value)] if match}
    if command.subcommand in aliases:
        return f"--config 里定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment:
        return f"环境变量 {alias_variable} 定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    return None


def cargo_use(arguments, directory, environment=None):
    """cargo 这一条算不算重型：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
    found = cargo_subcommand(arguments, directory)
    if found is None:
        return None
    environment = environment or {}
    expansion = alias_expansion(found, environment) if not break_is_set("alias-not-expanded") else None
    if expansion:
        expanded = cargo_use(list(expansion) + list(found.rest), found.directory,
                             {name: value for name, value in environment.items() if not name.startswith(ALIAS_ENVIRONMENT_PREFIX)})
        # 展开看得见：展开之后是什么就判什么（跑 harness 档的别名不重型），不再走「看不见」那一支
        return expanded._replace(detail=f"别名 {found.subcommand} 展开成 cargo {' '.join(expansion)}：{expanded.detail}") if expanded else None
    alias_reason = alias_of_subcommand(found, environment)
    if alias_reason:
        return heavy_test("checker-tier-cargo", f"cargo {found.subcommand}：{alias_reason}，跑到哪个包看不见，按 checker 档算")
    invocation, rest = test_invocation(found.subcommand, found.rest, found.directory)
    # `--no-run`（在 `--` 之前）只编译测试、一条都不跑：编译不是重型，谁都能做（子 agent 交回前也要编）。
    cargo_options = rest[:rest.index("--")] if "--" in rest else rest
    if invocation in ("test", "nextest") and "--no-run" in cargo_options and not break_is_set("no-run-counted-as-running"):
        return None
    directory = found.directory
    packages, test_names, selectors, binaries = [], [], set(), []
    whole_workspace, manifest_argument = False, None
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
    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
    sections = manifest_sections(manifest_path) if manifest_path else None
    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
    narrowed = bool(selectors & NARROWING_SELECTORS)
    scope, is_virtual_workspace_root = [], False
    if sections is not None:
        is_virtual_workspace_root = "workspace" in sections and "package" not in sections
        if packages:
            named = set()
            for specification in packages:
                named |= (package_specification_members(specification, members) if not break_is_set("package-specification-literal")
                          else ({specification} if specification in members else set()))
            scope = [members[name] for name in sorted(named)]
        elif is_virtual_workspace_root:
            scope = list(members.values())
        else:
            scope = [os.path.normpath(os.path.dirname(manifest_path))]
    if sections is not None and not packages and is_virtual_workspace_root and not narrowed:
        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
    if sections is not None and not narrowed and members and set(scope) == set(members.values()):
        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                         f"{len(members)} 个成员），等于全量"))
    # checker 档按包判（D13 已定项 15）：范围里有 checker 档包，不管挑哪个目标（库单测、集成测试、装置二进制的内联单测、doctest）都是重型；
    # 别的包（harness 档、core、format、池级 checker）的测试随时跑
    checker_directory = members.get(CHECKER_PACKAGE_NAME)
    if checker_directory and os.path.normpath(checker_directory) in {os.path.normpath(path) for path in scope}:
        return heavy_test("checker-tier-cargo", f"{described} 跑到 {CHECKER_PACKAGE_NAME} 包的测试（checker 档，默认只在提交时跑；"
                                                f"{'--test ' + '、'.join(test_names) if test_names else '按包'}）")
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
                    "-C", "--capsule", "--expand-environment"},
    "strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X",
               "--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
    "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
             "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
             "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
    "rustup": set(),
}
# 各包装程序不带值（或值只能用 = 贴着写）的长选项：长选项可以只写一个前缀（getopt_long 认），前缀对不上这里的、却是某个要值的长选项的前缀时，
# 按要值算（下一个词是它的值）。两张表照各自的 --help（systemd-run 照 systemd 255 的 man 页）列；strace 那几个「要不要值」照 strace -o /dev/null --<选项> true 量过
LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE = {
    "time": {"--append", "--portability", "--quiet", "--verbose", "--help", "--version"},
    "flock": {"--shared", "--exclusive", "--unlock", "--nonblock", "--nb", "--close", "--no-fork", "--verbose", "--help", "--version"},
    "chrt": {"--batch", "--deadline", "--fifo", "--idle", "--other", "--rr", "--reset-on-fork", "--all-tasks", "--max", "--pid", "--verbose",
             "--help", "--version"},
    "prlimit": {"--noheadings", "--raw", "--verbose", "--help", "--version", "--core", "--data", "--nice", "--fsize", "--sigpending", "--memlock",
                "--rss", "--nofile", "--msgqueue", "--rtprio", "--stack", "--cpu", "--nproc", "--as", "--locks", "--rttime"},
    "systemd-run": {"--collect", "--help", "--no-ask-password", "--no-block", "--on-clock-change", "--on-timezone-change", "--pipe", "--pty",
                    "--quiet", "--remain-after-exit", "--same-dir", "--scope", "--send-sighup", "--shell", "--slice-inherit", "--system", "--user",
                    "--version", "--wait"},
    "strace": {"--output-append-mode", "--summary-only", "--summary", "--debug", "--daemonize", "--follow-forks", "--output-separately", "--help",
               "--instruction-pointer", "--kill-on-exit", "--stack-trace", "--syscall-number", "--relative-timestamps", "--absolute-timestamps",
               "--timestamps", "--syscall-times", "--no-abbrev", "--version", "--summary-wall-clock", "--strings-in-hex", "--pidns-translation",
               "--successful-only", "--failed-only", "--failing-only", "--seccomp-bpf", "--tips", "--quiet", "--silent", "--silence", "--decode-fds"},
    "perf": set(),
    "rustup": set(),
}
# 弄坏开关 launcher-long-options-partial 下从要值的那张表里拿掉的（补全之前的表），同时不认长选项的前缀
LAUNCHER_LONG_OPTIONS_COMPLETED = {
    "systemd-run": {"--expand-environment"},
    "strace": {"--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
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


def resolved_long_option(option, options_with_value, options_without_value):
    """一个长选项词（`=` 之前那一段）：交回 (认成的选项名, 要不要值)。整词在两张表里的照表；都不在时按前缀认（getopt_long 认唯一的前缀）：
    恰好是一个已知长选项的前缀就当那一个；是几个的前缀时程序自己报「有歧义」、一条都不起，其中有要值的就按要值算（怎么算都一样）。
    弄坏开关 launcher-long-options-partial 下只按整词查表。"""
    if option in options_with_value:
        return option, True
    if option in options_without_value or break_is_set("launcher-long-options-partial") or not option.startswith("--"):
        return option, False
    matches = sorted(candidate for candidate in options_with_value | options_without_value if candidate.startswith("--") and candidate.startswith(option))
    if len(matches) == 1:
        return matches[0], matches[0] in options_with_value
    return option, any(candidate in options_with_value for candidate in matches)


def launcher_option(word, next_word, options_with_value, options_without_value=frozenset()):
    """包装程序的一个选项词：交回 (选项名, 它的值, 这个选项占几个词)。长选项 `--名=值` 与 `--名 值` 查两张表（resolved_long_option，认前缀，
    交回的选项名是认成的那一个）；短选项合写（`-fo 文件`、`-xw 10`、`-o文件`）逐个字母查表，头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
    一个要值的字母都没有的只占自己。"""
    if word.startswith("--") or break_is_set("launcher-whole-word-options"):
        option, has_attached_value, attached_value = word.partition("=")
        option, takes_value = resolved_long_option(option, options_with_value, options_without_value)
        if has_attached_value:
            return option, attached_value, 1
        return (option, next_word, 2) if takes_value else (option, "", 1)
    for index, letter in enumerate(word[1:], start=1):
        if "-" + letter in options_with_value:
            attached_value = word[index + 1:]
            return ("-" + letter, attached_value, 1) if attached_value else ("-" + letter, next_word, 2)
    return word, "", 1


# systemd-run -p EnvironmentFile= 读不了时替它交的那一个变量：名字照 CARGO_TARGET_*_RUNNER 的形状，判法按「环境变量里定了 runner」算（判不出，按宽）
UNREADABLE_ENVIRONMENT_FILE_RUNNER = "CARGO_TARGET_UNREADABLE_ENVIRONMENT_FILE_RUNNER"


def assignments_of_environment_property(value):
    """systemd-run -p / --property 的一个值设给里面那条命令的 NAME=VALUE：`Environment=A=1 "B=2 3"` 按 systemd 的引号规矩切；
    `EnvironmentFile=[-]<文件>` 按内容读（一行一个 NAME=VALUE，# 与 ; 起头的行、空行不算，行首的 export 去掉）；
    文件读不了（不在、没权限）交一个 UNREADABLE_ENVIRONMENT_FILE_RUNNER=<文件>：里面设了什么判不出，按设了 runner 算。别的属性交空表。"""
    name, _equals, rest = value.partition("=")
    if name == "Environment":
        try:
            words = shlex.split(rest)
        except ValueError:
            words = rest.split()
        return [word for word in words if ENVIRONMENT_ASSIGNMENT_WORD.match(word)]
    if name == "EnvironmentFile":
        path = rest[1:] if rest.startswith("-") else rest
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                lines = handle.read().split("\n")
        except OSError:
            return [f"{UNREADABLE_ENVIRONMENT_FILE_RUNNER}={path}"]
        assignments = []
        for line in lines:
            line = line.strip()
            if not line or line.startswith(("#", ";")):
                continue
            line = line[len("export "):].lstrip() if line.startswith("export ") else line
            if ENVIRONMENT_ASSIGNMENT_WORD.match(line):
                key, _separator, assigned = line.partition("=")
                assignments.append(f"{key}={assigned.strip().strip(chr(34)).strip(chr(39))}")
        return assignments
    return []


def command_under_launcher(words):
    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE，
    与 systemd-run -p / --property 的 Environment= / EnvironmentFile= 设的（assignments_of_environment_property）写在交回那条命令的最前面。
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
    if break_is_set("launcher-long-options-partial"):
        options_with_value = options_with_value - LAUNCHER_LONG_OPTIONS_COMPLETED.get(name, set())
    options_without_value = LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE.get(name, set())
    environment_options = set() if name == "strace" and break_is_set("strace-drops-env") else LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
    while position < len(words):
        word = words[position]
        if word == "--":
            position += 1
            break
        if not word.startswith("-") or word == "-":
            break
        option, value, word_count = launcher_option(word, words[position + 1] if position + 1 < len(words) else "", options_with_value,
                                                    options_without_value)
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
        if option in environment_options and "=" in value and not break_is_set("launcher-drops-setenv"):
            assignments.append(value)
        if name == "systemd-run" and option in ("-p", "--property") and not break_is_set("launcher-drops-property-environment"):
            assignments += assignments_of_environment_property(value)
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
        if binary in checker_test_targets(directory or HOOK_REPOSITORY):
            return heavy_test("checker-tier-binary", f"直接执行 {CHECKER_PACKAGE_NAME} 包的测试二进制（{binary}）：checker 档，默认只在提交时跑")
        return None
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
            "checker-tier-cargo", "research/scripts/layer0-shard-run.sh（两台各跑一片 checker 档登记的崩溃枚举用例、带 --include-ignored，再本机 merge）")
    if name in ("e152-file-system-benchmark", "e152-run.sh"):
        return heavy_test("e152", name)
    if name == "cargo":
        return cargo_use(arguments, directory, environment)
    return None


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
    """仓根虚工作区四个成员：core、harness 档（singlefs-harness）、池级 checker（singlefs-checker，只有库）、checker 档（singlefs-checker-tier：
    两个集成测试目标、一个装置二进制、库）；research/ 是另一个虚工作区；mutants-sample/ 是两个成员的虚工作区，带一份 test_workspace = true 的 .cargo/mutants.toml。另有两份给 systemd-run -p EnvironmentFile= 读的环境文件。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness", "crates/singlefs-checker", "crates/singlefs-checker-tier"]\n'
                        'exclude = ["research"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
    write("crates/singlefs-harness/tests/overwrite_in_one_instance.rs", "#[test]\nfn a_daily_case() {}\n")
    write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
    write("crates/singlefs-checker/src/lib.rs", "pub fn check_pool_image() {}\n")
    write("crates/singlefs-checker-tier/Cargo.toml", '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\n')
    write("crates/singlefs-checker-tier/src/lib.rs", "pub fn judge() {}\n")
    write("crates/singlefs-checker-tier/src/bin/e161_sample_device.rs", "fn main() {}\n")
    write("crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs",
          '#[test]\nfn quick_case() {}\n#[test]\n#[ignore = "崩溃枚举（样本）：平时不跑"]\nfn the_full_case() {}\n')
    write("crates/singlefs-checker-tier/tests/checker_quick_stream.rs", "#[test]\nfn a_quick_case() {}\n")
    write("alias.env", '# 样本：systemd-run -p EnvironmentFile= 读的\nCARGO_ALIAS_XT="test -p singlefs-checker-tier"\n')
    write("plain.env", "# 样本\nRUST_BACKTRACE=1\n")
    write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
    write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')
    write("mutants-sample/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["mutated", "other"]\n')
    write("mutants-sample/mutated/Cargo.toml", '[package]\nname = "mutated"\nversion = "0.0.0"\n')
    write("mutants-sample/other/Cargo.toml", '[package]\nname = "other"\nversion = "0.0.0"\n')
    write("mutants-sample/.cargo/mutants.toml", "# 样本：cargo mutants 每次测整个工作区\ntest_workspace = true\n")


def selftest():
    """在临时工作区里判 cargo 命令行、直接执行的测试二进制与按名字判的脚本；返回退出码。"""
    work = tempfile.mkdtemp(prefix="lib-heavy-tests-")
    results = []
    try:
        build_sample_workspace(work)
        harness = os.path.join(work, "crates", "singlefs-harness")
        tier = os.path.join(work, "crates", "singlefs-checker-tier")
        tier_binary = "crash_enumeration_new_pool_file_creation_stream-0123456789abcdef"
        alias_to_tier = "CARGO_ALIAS_XT=test -p singlefs-checker-tier"
        # (说明, 进程的 argv, 进程的 cwd, 该认出的 kind；None 是不重型)
        cargo_cases = [
            # 工作区全量
            ("cargo test --all", ["cargo", "test", "--all"], work, "full-cargo"),
            ("argv[0] 是 cargo 的全路径", ["/home/user/.rustup/toolchains/stable/bin/cargo", "test", "--workspace"], work, "full-cargo"),
            ("在工作区根裸跑", ["cargo", "test"], work, "full-cargo"),
            ("经 nice 包一层", ["/usr/bin/nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("cargo nextest run 在工作区根裸跑", ["cargo", "nextest", "run"], work, "full-cargo"),
            ("cargo hack 在子命令前带自己的选项、test --workspace", ["cargo", "hack", "--each-feature", "test", "--workspace"], work, "full-cargo"),
            # harness 档、core、池级 checker：随时跑
            ("harness 档整包", ["cargo", "test", "-p", "singlefs-harness"], work, None),
            ("在 harness 档目录里裸跑", ["cargo", "test", "--release"], harness, None),
            ("bash -c 里进 harness 档裸跑", ["bash", "-c", "cd crates/singlefs-harness && cargo test"], work, None),
            ("harness 档带 --ignored（它的重用例随时跑）", ["cargo", "test", "-p", "singlefs-harness", "--", "--ignored"], work, None),
            ("cargo mutants 整个 harness 档", ["cargo", "mutants", "-p", "singlefs-harness"], work, None),
            ("checker 档只编不跑（--no-run）", ["cargo", "test", "-p", "singlefs-checker-tier", "--no-run"], work, None),
            ("checker 档 -- 之后的 --no-run 是测试二进制的参数，照跑", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--no-run"], work, "checker-tier-cargo"),
            ("cargo mutants 变异 core、--test-package 点名 harness 档", ["cargo", "mutants", "-p", "singlefs-core", "--test-package", "singlefs-harness"], work, None),
            ("cargo mutants 变异 core、--test-package 点名 checker 档", ["cargo", "mutants", "-p", "singlefs-core", "--test-package=singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo mutants --test-workspace=true 测整个工作区", ["cargo", "mutants", "-p", "singlefs-harness", "--test-workspace=true"], work, "full-cargo"),
            ("cargo mutants 配置里 test_workspace = true", ["cargo", "mutants", "-p", "mutated"], os.path.join(work, "mutants-sample"), "full-cargo"),
            ("cargo mutants -d 指到带 test_workspace = true 配置的树", ["cargo", "mutants", "-d", "mutants-sample", "-p", "mutated"], work, "full-cargo"),
            ("cargo mutants 配置里 test_workspace = true、命令行 --test-workspace false 盖掉", ["cargo", "mutants", "-p", "mutated", "--test-workspace", "false"], os.path.join(work, "mutants-sample"), None),
            ("cargo mutants --no-config 不读 test_workspace", ["cargo", "mutants", "--no-config", "-p", "mutated"], os.path.join(work, "mutants-sample"), None),
            ("core 的一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            ("池级 checker 不是 checker 档", ["cargo", "test", "-p", "singlefs-checker"], work, None),
            ("池级 checker 的库单测", ["cargo", "test", "-p", "singlefs-checker", "--lib"], work, None),
            ("clippy --all-targets 不起测试", ["cargo", "clippy", "--all-targets"], work, None),
            ("cargo nextest list 不起测试", ["cargo", "nextest", "list", "--workspace"], work, None),
            ("cargo llvm-cov report 不起测试", ["cargo", "llvm-cov", "report"], work, None),
            # checker 档：按包判，挑哪个目标都算
            ("checker 档整包", ["cargo", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("checker 档点名一个集成测试", ["cargo", "test", "--release", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream"], work,
             "checker-tier-cargo"),
            ("checker 档 --tests", ["cargo", "test", "-p", "singlefs-checker-tier", "--tests"], work, "checker-tier-cargo"),
            ("checker 档 --lib", ["cargo", "test", "-p", "singlefs-checker-tier", "--lib"], work, "checker-tier-cargo"),
            ("checker 档 --doc", ["cargo", "test", "-p", "singlefs-checker-tier", "--doc"], work, "checker-tier-cargo"),
            ("checker 档 --bin 装置的内联单测", ["cargo", "test", "-p", "singlefs-checker-tier", "--bin", "e161_sample_device"], work, "checker-tier-cargo"),
            ("checker 档 --test 通配", ["cargo", "test", "-p", "singlefs-checker-tier", "--test", "*"], work, "checker-tier-cargo"),
            ("checker 档带 --include-ignored --exact", ["cargo", "test", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream", "--",
                                                     "--include-ignored", "--exact", "the_full_case"], work, "checker-tier-cargo"),
            ("在 checker 档目录里裸跑", ["cargo", "test", "--release"], tier, "checker-tier-cargo"),
            ("-p core 加 -p checker 档", ["cargo", "test", "-p", "singlefs-core", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("--manifest-path 指 checker 档", ["cargo", "test", "--manifest-path", "crates/singlefs-checker-tier/Cargo.toml"], work, "checker-tier-cargo"),
            ("cargo nextest run -p checker 档", ["cargo", "nextest", "run", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo miri test -p checker 档", ["cargo", "miri", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo llvm-cov 不带子命令 -p checker 档", ["cargo", "llvm-cov", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo mutants -p checker 档", ["cargo", "mutants", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            # -p 的几种包规格（开关 package-specification-literal 下这三格红）
            ("-p 通配命中 checker 档", ["cargo", "test", "-p", "singlefs-checker-t*"], work, "checker-tier-cargo"),
            ("-p name@version", ["cargo", "test", "-p", "singlefs-checker-tier@0.1.0"], work, "checker-tier-cargo"),
            ("-p pkgid URL", ["cargo", "test", "-p", "path+file:///x/crates/singlefs-checker-tier#singlefs-checker-tier@0.1.0"], work, "checker-tier-cargo"),
            # 别名：看得到值就展开再判（开关 alias-not-expanded 下前三格红；configuration-before-subcommand-only 下第四格红）
            ("--config 别名展开成 test -p checker 档", ["cargo", "--config", 'alias.xt="test -p singlefs-checker-tier"', "xt"], harness, "checker-tier-cargo"),
            ("--config 别名写成数组", ["cargo", "--config", 'alias.xt=["test", "-p", "singlefs-checker-tier"]', "xt"], work, "checker-tier-cargo"),
            ("环境变量 CARGO_ALIAS_XT 定的别名", ["env", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("--config 写在子命令之后定的别名", ["cargo", "xt", "--config", 'alias.xt="test -p singlefs-checker-tier"'], work, "checker-tier-cargo"),
            ("别名展开成 harness 档", ["env", "CARGO_ALIAS_XT=test -p singlefs-harness", "cargo", "xt"], work, None),
            ("--config 定了别名、子命令不是它（cargo build）", ["cargo", "--config", 'alias.xt="test -p singlefs-checker-tier"', "build"], work, None),
            ("别名的值读不出（TOML 写坏了）：按 checker 档算", ["cargo", "--config", "alias.xt=[unclosed", "xt"], work, "checker-tier-cargo"),
            # --list：只列不跑（开关 list-by-presence 下前三格红；list-past-separator 下后两格红）
            ("checker 档 -- --include-ignored --skip --list（--list 是 --skip 的值）",
             ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored", "--skip", "--list"], work, "checker-tier-cargo"),
            ("checker 档 -- --logfile --list（--list 是日志文件名）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--logfile", "--list"], work,
             "checker-tier-cargo"),
            ("checker 档 -- --ignored --skip=--list（一个词，没有 --list 选项）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--ignored", "--skip=--list"],
             work, "checker-tier-cargo"),
            ("checker 档 -- --list（只列）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--list"], work, None),
            ("checker 档 -- --ignored --test-threads 4 --list（只列）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--ignored", "--test-threads", "4",
                                                                  "--list"], work, None),
            ("checker 档 -- --include-ignored the_full_case -- --list（第二个 -- 之后 --list 是过滤词）",
             ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored", "the_full_case", "--", "--list"], work, "checker-tier-cargo"),
            ("checker 档 -- --exact the_full_case -- --list", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--exact", "the_full_case", "--", "--list"],
             work, "checker-tier-cargo"),
            # 按名字认的脚本与门禁阶段
            ("bash 起 54 号", ["bash", ".claude/gate.d/54-layer0-replay.sh", "--full", "/tmp/wt"], work, "layer0-stage"),
            ("bash 起双机分片的驱动脚本", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:sample", "/tmp/wt"], work, "checker-tier-cargo"),
            ("双机分片的驱动脚本 --selftest 不重型", ["bash", "research/scripts/layer0-shard-run.sh", "--selftest"], work, None),
            ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
            ("bash 起轻阶段 12 号", ["bash", ".claude/gate.d/12-no-prime-marks.sh"], work, None),
        ]
        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）
        launcher_cases = [
            ("/usr/bin/time -v 包 checker 档", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("time -f 格式 -o 文件包 checker 档的集成测试",
             ["time", "-f", "%M", "-o", "/tmp/t", "cargo", "test", "--release", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream"],
             work, "checker-tier-cargo"),
            ("systemd-run --working-directory= 进 checker 档裸跑",
             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-checker-tier", "cargo", "test"], work, "checker-tier-cargo"),
            ("systemd-run --working-directory= 进 harness 档裸跑",
             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-harness", "cargo", "test"], work, None),
            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "cargo test --workspace"], work, "full-cargo"),
            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", "cargo", "test", "--all"], work, "full-cargo"),
            ("chrt -i 0 包一层", ["chrt", "-i", "0", "cargo", "test", "--all"], work, "full-cargo"),
            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", "cargo", "test", "--all"],
             work, "full-cargo"),
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
            ("systemd-run -E 设别名", ["systemd-run", "--user", "--scope", "-E", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run --setenv= 设别名", ["systemd-run", "--user", "--scope", "--setenv=" + alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -qE 合写设别名", ["systemd-run", "--user", "--scope", "-qE", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -E 设的不是别名（值的末段叫 strace 也不当包装剥）", ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", "cargo", "xt"],
             work, None),
            # strace -E / --env 设的变量同样带进里面那条命令（开关 strace-drops-env 下前四格红）
            ("strace -E 设别名", ["strace", "-f", "-E", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace --env= 设别名", ["strace", "--env=" + alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace --env 值另起一个词设别名", ["strace", "--env", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace -fE 合写设别名", ["strace", "-fE", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace -E 只清变量（不带 =）", ["strace", "-E", "CARGO_ALIAS_XT", "cargo", "xt"], work, None),
            # 要值的长选项、值另起一个词：跳过那个值（开关 launcher-long-options-partial 下前四格红）
            ("systemd-run --user --scope --expand-environment no（值另起一个词）包一层",
             ["systemd-run", "--user", "--scope", "--expand-environment", "no", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --output 文件（值另起一个词）包一层", ["strace", "--output", "/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --string-limit 200 包一层", ["strace", "--string-limit", "200", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -c --summary-col calls（长选项只写前缀）包一层", ["strace", "-c", "--summary-col", "calls", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --quiet（不带值的长选项）包一层", ["strace", "--quiet", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope --collect --same-dir（不带值的长选项）包一层",
             ["systemd-run", "--user", "--scope", "--collect", "--same-dir", "cargo", "test", "--all"], work, "full-cargo"),
            # systemd-run -p Environment= / EnvironmentFile= 设给里面那条命令的变量（开关 launcher-drops-property-environment 下前四格红）
            ("systemd-run -p Environment= 设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", 'Environment="' + alias_to_tier + '"', "cargo", "xt"],
             work, "checker-tier-cargo"),
            ("systemd-run --property=Environment= 设两个变量、第二个是别名",
             ["systemd-run", "--user", "--wait", "--pipe", '--property=Environment=A=1 "' + alias_to_tier + '"', "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run --prop Environment=（长选项只写前缀）设别名", ["systemd-run", "--user", "--wait", "--pipe", "--prop", 'Environment="' + alias_to_tier + '"',
                                                                  "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -p EnvironmentFile= 文件里设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "alias.env"),
                                                         "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -p Environment= 设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "Environment=RUST_BACKTRACE=1", "cargo", "xt"], work, None),
            ("systemd-run -p EnvironmentFile= 文件里设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "plain.env"),
                                                               "cargo", "xt"], work, None),
        ]
        binary_cases = [
            ("绝对路径的 checker 档集成测试二进制", [os.path.join(work, "target", "release", "deps", tier_binary), "--test-threads", "4"], work, "checker-tier-binary"),
            ("相对路径的 checker 档集成测试二进制", [os.path.join("target", "debug", "deps", tier_binary)], work, "checker-tier-binary"),
            ("经 timeout 包一层", ["timeout", "600", os.path.join(".", "target", "release", "deps", tier_binary), "--exact", "case"], work, "checker-tier-binary"),
            ("checker 档装置二进制的内联单测", [os.path.join(work, "target", "debug", "deps", "e161_sample_device-0123456789abcdef")], work, "checker-tier-binary"),
            ("checker 档库的单测", [os.path.join(work, "target", "debug", "deps", "singlefs_checker_tier-0123456789abcdef")], work, "checker-tier-binary"),
            ("checker 档二进制 --logfile --list（--list 是日志文件名）", [os.path.join(work, "target", "release", "deps", tier_binary), "--logfile", "--list"], work,
             "checker-tier-binary"),
            ("checker 档二进制 --list（只列）", [os.path.join(work, "target", "release", "deps", tier_binary), "--list"], work, None),
            ("harness 档的测试二进制", [os.path.join(work, "target", "release", "deps", "overwrite_in_one_instance-0123456789abcdef")], work, None),
            ("名字带 layer0 而不是 checker 档的二进制（按包判，不按名字）", ["/tmp/target-elsewhere/release/deps/some_layer0_stream-fedcba9876543210"], work, None),
            ("deps 下的 .d 依赖文件不是二进制", [os.path.join(work, "target", "release", "deps", tier_binary + ".d")], work, None),
            ("哈希不是 16 位", [os.path.join(work, "target", "release", "deps", "crash_enumeration_new_pool_file_creation_stream-0123abcd")], work, None),
            ("名字只当参数", ["grep", "-c", "x", os.path.join("target", "release", "deps", tier_binary)], work, None),
        ]
        own_repository_targets = sorted(checker_test_targets(HOOK_REPOSITORY))
        if own_repository_targets:
            binary_cases.append(("仓外执行、取这份文件所在仓的 checker 档包：直接执行它的测试二进制按包判",
                                 [f"/tmp/target-elsewhere/release/deps/{own_repository_targets[0]}-0123456789abcdef"], "/", "checker-tier-binary"))
        else:
            results.append((f"这份文件所在仓（{HOOK_REPOSITORY}）里找得到 checker 档包的测试目标", True, False))
        for label, argv, cwd, want in cargo_cases + launcher_cases + binary_cases:
            found = classify_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
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
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "overwrite_in_one_instance-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["research/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/new_pool_file_creation_on_device"], True),
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
        print("    → 看 classify()、cargo_subcommand()、cargo_use()、package_specification_members()、alias_expansion()、test_invocation()、"
              "command_under_launcher()、launcher_option()、libtest_lists_only()、checker_test_targets()、test_binary_name()、"
              "runs_compiled_code()、classify_process()、judged_by_name() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
              f"KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / LIBTEST_OPTIONS_WITH_VALUE 几张表；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种、"
          f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种）")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_heavy_tests.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
