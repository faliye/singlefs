# lib_project_code_runs.py 的部分模块：常量、登记表与阶段样本的读法、cargo 与提交客户端与绕开服务的认法（启动器与展开与 classify 在 lib_project_code_runs_launchers.py，自证在 lib_project_code_runs_selftest.py）。
# 入口 lib_project_code_runs.py 按 PROJECT_CODE_RUNS_PART_NAMES 的次序 import 这一份并把名字全部再导出；启动器与自证 import 这一份。自证与调用方换掉的模块变量（SELFTEST_REGISTRY_PATH、SELFTEST_REGISTRY_ROOT、SAMPLE_INDEX_REPOSITORY、TEMPORARY_SERVICE_TABLE_PATH、BASH_FOR_PATTERNS）在这一份里换，这一份与启动器里的函数读的是它们。
# 弄坏开关的名单与红的格登记在入口 lib_project_code_runs.py 的文件头里。
from lib_project_code_runs_bootstrap import (
    break_is_set,
)

import ast, contextlib, filecmp, fnmatch, functools, glob, hashlib, importlib.util, os, re, shlex, shutil, stat, subprocess, sys, tempfile
from typing import NamedTuple


def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(os.path.dirname(os.path.abspath(__file__)), module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


shell_words = load_sibling_module("lib_shell_words")


class ProgramUse(NamedTuple):
    kind: str      # 细分的用法（external-program、pattern-unresolved、unrecognized），project-code-guard.sh 按它写拒绝说明
    category: str  # 报给人看的类名，KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 要经 task-server/task 起、本仓的代码判法（project_code_use、starts_a_build、runs_compiled_code）认不出的三种写法：kind → 类名
KIND_CATEGORY = {
    "external-program": "本仓之外的外部程序", "project-code": "展开出来的本仓的代码", "pattern-unresolved": "判不出的通配", "unrecognized": "认不出起什么的写法",
}
# 这几个目录下的脚本带 --selftest 跑是跑它的自证；.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，带不带 --selftest 都算。认不认按登记表（registered_selftest）
# scripts/ 下按用途分的子目录（scripts/README.md）：脚本都住在这一层
SCRIPT_CATEGORY_DIRECTORIES = tuple(f"scripts/{category}" for category in ("agents", "crash", "devices", "experiment-apparatus", "gate", "hosts", "kb", "mutation", "prune-server"))
# 统一任务服务的根目录（records/2026-10-30-编译绕过内存总上限.md 批 3b 新加）：服务的脚本住在这一层，自证同样登记在登记表里（selftest_directories）
TASK_SERVER_DIRECTORY = "task-server"
# 判定库服务的根目录：control 脚本（装、起、停、查、监听，--selftest 只起假的 ip、ssh、systemctl）住在这一层，自证同样登记在登记表里
PRUNE_SERVER_DIRECTORY = "prune-server"
# GPU 批处理验证服务的根目录：control 脚本（装、起、停、查，--selftest 只起假的 ssh、systemctl）住在这一层，自证同样登记在登记表里
GPU_VERIFICATION_SERVER_DIRECTORY = "gpu-verification-server"
# scripts/mutation/tests：变异那几份脚本的回归单测（test_audit_regressions.py，单独一个 --selftest 跑 unittest），登记表里那一行落在这一层
MUTATION_REGRESSION_TESTS_DIRECTORY = "scripts/mutation/tests"
# 会话检查项目录（.claude/session-checks/，一项一个可执行文件，由上游钩子 session-checks.sh 起）：检查项的自证登记在登记表里，键落在这一层
SESSION_CHECKS_DIRECTORY = ".claude/session-checks"
SELFTEST_DIRECTORIES = SCRIPT_CATEGORY_DIRECTORIES + (MUTATION_REGRESSION_TESTS_DIRECTORY, TASK_SERVER_DIRECTORY, PRUNE_SERVER_DIRECTORY, GPU_VERIFICATION_SERVER_DIRECTORY, ".claude/hooks", SESSION_CHECKS_DIRECTORY, ".claude/gate.d/lib")
# 实验装置那一层（文件头「实验装置那一层」）：这个目录下正好一层的目录，登记了的单独 --selftest 才当自证
DEVICE_SELFTEST_PARENT = "scripts/experiment-apparatus"
GATE_LIBRARY_DIRECTORY = ".claude/gate.d/lib"
# 自证登记表：一行 `<相对仓根的路径><制表符><种类><制表符># <理由>`，写法在表头。放在这份文件旁边；自证换成样本表时改这个变量
SELFTEST_REGISTRY_NAME = "selftest-registry.tsv"
SELFTEST_REGISTRY_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), SELFTEST_REGISTRY_NAME)
SELFTEST_KIND = "selftest"
SELFTEST_REASON_MINIMUM = 4   # 理由（# 之后）至少几个字
# 门禁阶段的样本档（文件头「门禁样本」）：登记表里键是 .claude/gate.d/<阶段>.sh 的行，种类写这两种之一
GATE_STAGE_DIRECTORY = ".claude/gate.d"
# 门禁目录顶层以它起头的文件是共用库、不是阶段（lib-lag-baseline.py 这类）：gate_stage_name 不当阶段认，登记表里它那一行照脚本的自证判（gate_top_library_key）
GATE_TOP_LIBRARY_PREFIX = "lib-"
# 门禁目录顶层的每一道阶段，逐个写出来：同族拷贝比内容时本仓那一份从这张表里挑（own_script_digest）。与门禁目录顶层逐个相同由自证
# 「同族：门禁目录顶层列出来的阶段与 GATE_STAGE_FILES 逐个相同、每一道从表里挑出的 sha256 与盘上那一份相同」核（那一格照 glob 列、逐道读，算键库把列出来的每一道收进键，
# 加一道阶段键就变），加了阶段没加进来的，那一道在别处的拷贝照读正文（own_script_digest 交 None）
GATE_STAGE_FILES = (
    "broad-mutation.sh", "checker-independence-and-sync.sh", "code-source-discipline.sh", "code-tooling.sh", "crash-point-replay.sh",
    "crates-mutation-replay.sh", "doc-decisions.sh", "doc-experiments.sh", "doc-process-records.sh", "doc-registries.sh", "doc-text.sh",
    "formal-verification.sh", "lkmm.sh", "model-differential-and-scenarios.sh", "qemu-device-streams.sh", "research-build-and-replay.sh",
    "test-environment.sh",
)
# samples：样本只读文本、里面的工具都是假的；samples:<目录>：样本根里 <目录> 放着假工具（selftest-process-guard.py 不把那一层的工具当真的）
STAGE_SAMPLES, STAGE_SAMPLES_WITH_TOOLS = "samples", "samples:"
STAGE_SELFTEST_SCRIPT = "stage-selftest.sh"
# stage-selftest.sh 自己认的两个参数（它文件头「用法」）：--item <阶段文件名> 只喂点名那几道（给几次取并集），--list-items 只列可点名的阶段、不喂样本；别的词是阶段目录
STAGE_SELFTEST_ITEM_OPTION, STAGE_SELFTEST_LIST_OPTION = "--item", "--list-items"
# 阶段目录写成 $PWD/… 或 ${PWD}/…（拒绝说明里给的写法）：当前目录认得出、命令文本里没另设 PWD 时换成当前目录（stage_selftest_item_problems）
WORKING_DIRECTORY_VARIABLE = re.compile(r"^\$(?:PWD|\{PWD\})(?=/|$)")
# 命令词里带这几个通配字符的不当自证（registered_selftest）：bash 展开之后才知道起的是哪一份；case 的模式行被切成命令时也是这样的词
SELFTEST_PATTERN_CHARACTERS = set("*?[")
# 阶段参数里认不出是哪个目录的词：没展开的变量、命令替换、通配
UNRESOLVED_WORD_CHARACTERS = set("$`*?[]{}")
# 按名字判的仓内脚本：名字 → 它在仓里的位置。命令词是这个名字、又落在这个位置上，judged_by_name 为真
KNOWN_SCRIPT_LOCATIONS = {
    "gate.sh": ".claude/scripts/gate.sh", "check.sh": ".claude/scripts/check.sh", "lkmm.sh": ".claude/scripts/lkmm.sh",
    "gate-staged.sh": "scripts/gate/gate-staged.sh", "mutate.sh": "scripts/mutation/mutate.sh",
    "vm-bench.sh": "scripts/devices/vm-bench.sh", "e152-run.sh": "scripts/devices/e152-run.sh",
    "capped.sh": "scripts/hosts/capped.sh", "run-with-memory-cap.sh": "scripts/hosts/run-with-memory-cap.sh",
    "mutation-shard-run.sh": "scripts/mutation/mutation-shard-run.sh",
    "broad-mutation.py": "scripts/mutation/broad-mutation.py",
    "crash-amplification.sh": "scripts/crash/crash-amplification.sh", "crash-amplification-item.sh": "scripts/crash/crash-amplification-item.sh",
    "crash-amplification-two-hosts.sh": "scripts/crash/crash-amplification-two-hosts.sh",
    "build-patched-qemu.sh": "scripts/devices/build-patched-qemu.sh",
    "crash-point-replay.sh": "scripts/crash/crash-point-replay.sh", "replay.sh": "scripts/experiment-apparatus/replay.sh",
    STAGE_SELFTEST_SCRIPT: ".claude/singlefs-ai-sop/scripts/stage-selftest.sh",
}
# 广谱变异的逐条驱动（scripts/mutation/broad-mutation.py）：带这几个参数之一的只读（列项、汇总、帮助），不起 cargo、不改库目录
BROAD_MUTATION_SCRIPT_NAME = "broad-mutation.py"
# Verus 那一道（formal-verification.sh）的两份判法脚本：直接起时干活的用法算 verus-stage（verus_judge_use）
VERUS_PROOF_CHECK_SCRIPT = "verus-proof-check.py"
# 要经 task-server/task 起的外部程序（PATH 上直接起、本仓之外的）：本仓的脚本由 project_code_use 认，会编的 cargo 与编出来的二进制由 starts_a_build、
# runs_compiled_code 认，这几个程序它们认不出；命中就和直接起本仓的代码一样拒。名单不分档，名字取 basename：
# 前缀 qemu-system（qemu-system-x86_64、qemu-system-aarch64……）、herd7（只问版本的放行）、e152-file-system-benchmark（E152 装置的二进制，光名字从 PATH 起）
EXTERNAL_PROGRAM_PREFIXES = ("qemu-system",)
EXTERNAL_PROGRAM_NAMES = frozenset({"herd7", "e152-file-system-benchmark"})
# 只问版本、不起程序干活的参数写法（只带这一个参数）：程序名 → 参数表
EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS = {"herd7": (["-version"], ["--version"])}
E165_PIN_SCRIPT = "e165-pin-red-examples.py"
# 入口脚本之一 scripts/mutation/prove-red.sh（它从 python 里起 cargo，闸看不见）；用法只有 `--copy <仓副本> <crate> <变异名…>`
PROVE_RED_SCRIPT = "prove-red.sh"
# 命令词里没展开的变量名（`$b`、`${b}`、`${b:-x}`）；往它的定义里找 验证包测试目标名时最多往下点几层
UNEXPANDED_VARIABLE_NAME = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)")
UNEXPANDED_DEFINITION_DEPTH = 6


def program_use(kind, detail):
    return ProgramUse(kind, KIND_CATEGORY[kind], detail)



# libtest 的只列用例参数与带一个值的选项：lib_run_scope.py 的 cargo_run_shape 判「cargo test -- --list 只列不跑」用
LIST_ONLY_TEST_ARGUMENT = "--list"
LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}



def checked_cells(arguments):
    """阶段参数里 --check 点名的格（--check <格>[,<格>…] 与 --check=<…>，给几次取并集）；没给 --check 交 None（全跑）。lib_run_scope.py 用它数范围声明里的格数。
    弄坏开关 checked-cells-last-only 下给几次只认最后一次（不取并集）。"""
    cells, given, position = set(), False, 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--check" and position + 1 < len(arguments):
            if break_is_set("checked-cells-last-only"):
                cells.clear()
            cells.update(filter(None, arguments[position + 1].split(",")))
            given, position = True, position + 2
            continue
        if argument.startswith("--check="):
            if break_is_set("checked-cells-last-only"):
                cells.clear()
            cells.update(filter(None, argument[len("--check="):].split(",")))
            given = True
        position += 1
    return cells if given else None


def libtest_lists_only(libtest_arguments):
    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算；
    见到 `--` 就停、交「不是只列」：libtest 在 `--` 处停止认选项，之后的 --list 是过滤词（`-- --ignored -- --list` 一条都不跑，照样算不是只列，接受的误拒）。
    弄坏开关 libtest-list-by-presence 下见到 --list 这个词就算只列（不跳过带值选项的值、不在 -- 处停）。"""
    if break_is_set("libtest-list-by-presence"):
        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
    position = 0
    while position < len(libtest_arguments):
        argument = libtest_arguments[position]
        if argument == "--":
            return False
        if argument == LIST_ONLY_TEST_ARGUMENT:
            return True
        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
    return False


HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# 自证登记表的键相对哪个仓根解成真实路径（registered_selftest 只认解到这一份的命令词）；自检的样本表那几格换成临时目录
SELFTEST_REGISTRY_ROOT = HOOK_REPOSITORY
# stage-selftest.sh 喂的样本拿哪个仓的 HEAD 加暂存区比（sample_directory_changes）；自检换成临时建的样本仓
SAMPLE_INDEX_REPOSITORY = HOOK_REPOSITORY


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path",
                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
GLOB_CHARACTERS = set("*?[")
# 命令词带通配（classify 先照 bash 展开）：每一份单独当命令判，一次最多判这么多份；bash 真起的那一条（头一份当命令、其余当参数）另外总判
PATTERN_EXPANSION_LIMIT = 256
# bash 的 [^…] 与 [!…] 同义，python 的 glob 只认 [!…]
CARET_NEGATION = re.compile(r"\[\^")
# 方括号里的 POSIX 字符类、等价类、整理符号（[:alpha:]、[=h=]、[.h.]）：python glob 不认，带它的词交给 bash 展开（bash_pattern_expansions）
BASH_BRACKET_PATTERN = re.compile(r"\[[:=.]")
# 交给 bash 展开时最多等几秒；起 bash 用的程序（自证换成不存在的路径，演 bash 起不来）
BASH_PATTERN_TIMEOUT = 5
BASH_FOR_PATTERNS = "bash"
# scripts/crash/gpu-quiesce-run.sh -- <命令…>：清场之后原样起那条命令（command_under_gpu_quiesce 剥掉它）
GPU_QUIESCE_SCRIPT = "gpu-quiesce-run.sh"
GPU_QUIESCE_DIRECTORY = "scripts/crash"
# scripts/hosts/two-logs.sh [--quiet] <名> -- <命令…>：日志出两份的包装（转打、写 <名>.log 与 <名>.problems.log，退出码是那条命令的）：command_under_two_logs 剥掉它
TWO_LOGS_SCRIPT = "two-logs.sh"
# scripts/hosts/run-build-directory.sh [--] <命令…>：包一层起、这一趟一个自己的编译目录（跑完删）：command_under_run_build_directory 剥掉它
RUN_BUILD_DIRECTORY_SCRIPT = "run-build-directory.sh"
RUN_BUILD_DIRECTORY_LOCATION = "scripts/hosts"
TWO_LOGS_DIRECTORY = "scripts/hosts"
# 派发起法：scripts/hosts/multi-host-run.sh --peer [选项…] -- <命令…>（command_under_dispatch_launcher 剥掉它）。host-dispatch.py 命令行只认 --selftest，不在这张表里。
# 名字 → (不带值的选项, 带值的选项 → 值的个数)，照脚本文件头「用法」与它切参数的那一段（multi-host-run.sh 的 multi_host_main）
DISPATCH_LAUNCHER_DIRECTORY = "scripts/hosts"
MULTI_HOST_RUN_SCRIPT = "multi-host-run.sh"
DISPATCH_LAUNCHER_OPTIONS = {
    MULTI_HOST_RUN_SCRIPT: (frozenset({"--peer"}), {"--memory-cap": 1, "--label": 1}),
}
# DISPATCH_LAUNCHER_OPTIONS 的键逐个写出来：派发起法的拷贝比内容时本仓那一份从这里挑（own_script_digest）；每个键都挑得到由自证「同族：本仓那一份的 sha256 按位置都挑得到…」核
DISPATCH_LAUNCHER_SCRIPTS = (MULTI_HOST_RUN_SCRIPT,)
# 上面三张表里的每份脚本（KNOWN_SCRIPT_LOCATIONS 的值、GATE_STAGE_FILES、DISPATCH_LAUNCHER_SCRIPTS）引的库（script_source_reading 认的与 OWN_SCRIPT_UNRECOGNIZED_SOURCES 登记了库的，
# 连同库再引的），相对仓根逐个写出来，按类分成下面三张库表：同族拷贝比库时本仓那一边逐张从这几张表里挑（own_library_digest），算键库折得出读的是哪几份库。表外的库、表里点名而本仓盘上没有的，
# 别处拷贝照不同认（读正文）。三张拼成的整张（OWN_SCRIPT_LIBRARY_FILES）与那几份脚本正文现算的逐个相同、每一份都在本仓盘上由自证「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上」核。
# 分成几张、每张不超过 OWN_SCRIPT_LIBRARY_TABLE_LIMIT 份：own_library_digest 照一张表拼出的读路径有几份就有几种取值，取值超过算键库的折叠上限
# （scripts/gate/lib_function_units_base.py 的 FOLD_LIMIT）时那一处整个折不出，表里的库一份都不进判法的键；每张的份数由自证「库表：」那一格核
# 门禁阶段的共用库（.claude/gate.d/lib/）与准入判法的 shell 库（preflight.sh 两份、规范副本的 lib.sh）
GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES = (
    ".claude/gate.d/lib/code-source-discipline-mutation-and-reading-cells.sh", ".claude/gate.d/lib/code-source-discipline-naming-and-constants-cells.sh",
    ".claude/gate.d/lib/code-source-discipline-unsafe-and-registry-cells.sh", ".claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh",
    ".claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh", ".claude/gate.d/lib/code-tooling-fixtures-and-layout-cells.sh",
    ".claude/gate.d/lib/code-tooling-heavy-thread-recognition.sh", ".claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh",
    ".claude/gate.d/lib/code-tooling-selftest-cells.sh", ".claude/gate.d/lib/code-tooling-shared-libraries.sh", ".claude/gate.d/lib/code-tooling-source-scan-cells.sh",
    ".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", ".claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh",
    ".claude/gate.d/lib/crash-point-replay-crash-case-cells.sh", ".claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh",
    ".claude/gate.d/lib/crash-point-replay-package-pass-verdicts.sh", ".claude/gate.d/lib/crash-point-replay-selection-and-plan.sh",
    ".claude/gate.d/lib/doc-decisions-evidence-and-owed-cells.sh", ".claude/gate.d/lib/doc-decisions-registry-and-history-cells.sh",
    ".claude/gate.d/lib/doc-decisions-status-cells.sh", ".claude/gate.d/lib/doc-experiments-index-and-evidence-cells.sh",
    ".claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh", ".claude/gate.d/lib/doc-experiments-verdict-and-citation-cells.sh",
    ".claude/gate.d/lib/doc-process-records-batch-and-review-cells.sh", ".claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh",
    ".claude/gate.d/lib/doc-registries-field-and-layout-cells.sh", ".claude/gate.d/lib/doc-registries-reference-and-invariant-cells.sh",
    ".claude/gate.d/lib/doc-registries-unit-header-and-debt-cells.sh", ".claude/gate.d/lib/doc-text-content-and-history-cells.sh",
    ".claude/gate.d/lib/doc-text-notation-and-translation-cells.sh", ".claude/gate.d/lib/stage-cells.sh", ".claude/scripts/preflight.sh",
    ".claude/singlefs-ai-sop/scripts/lib.sh", ".claude/singlefs-ai-sop/scripts/preflight.sh",
)
# 崩溃入口、实验复跑、改动路径与主机（派发、内存包装、重活线程数）的 shell 库
CRASH_REPLAY_AND_HOST_LIBRARY_FILES = (
    "scripts/crash/crash-amplification-lib.sh",
    "scripts/crash/crash-point-replay-selftest.sh", "scripts/experiment-apparatus/long-run-lib/binary-identity.sh", "scripts/experiment-apparatus/replay-admission.sh",
    "scripts/experiment-apparatus/replay-claims.sh", "scripts/experiment-apparatus/replay-dispatch.sh", "scripts/experiment-apparatus/replay-finish.sh",
    "scripts/experiment-apparatus/replay-inputs-and-build.sh", "scripts/experiment-apparatus/replay-row-library.sh", "scripts/experiment-apparatus/replay-selftest.sh",
    "scripts/experiment-apparatus/replay-table-and-dispatch-limits.sh", "scripts/gate/changed-paths.sh", "scripts/hosts/heavy-threads.sh",
    "scripts/hosts/multi-host-run-selftest.sh", "scripts/hosts/peer-host-lib.sh", "scripts/hosts/run-build-directory.sh", "scripts/hosts/run-with-memory-cap-selftest.sh",
)
# 变异那几份入口（broad-mutation.py、门禁 crates-mutation-replay 起的 crates-mutation-rows.py 这类）引的 python 模块
MUTATION_PYTHON_LIBRARY_FILES = (
    "scripts/mutation/broad_mutation_base.py", "scripts/mutation/broad_mutation_gpu.py",
    "scripts/mutation/broad_mutation_judging.py", "scripts/mutation/broad_mutation_library.py", "scripts/mutation/broad_mutation_run.py",
    "scripts/mutation/broad_mutation_runner.py", "scripts/mutation/broad_mutation_selftest_a.py", "scripts/mutation/broad_mutation_selftest_b.py",
    "scripts/mutation/crates-mutation-rows.py", "scripts/mutation/crates_mutation_rows_base.py", "scripts/mutation/crates_mutation_rows_baseline.py",
    "scripts/mutation/crates_mutation_rows_commands.py", "scripts/mutation/crates_mutation_rows_driver.py", "scripts/mutation/crates_mutation_rows_judging.py",
    "scripts/mutation/crates_mutation_rows_memory.py", "scripts/mutation/crates_mutation_rows_plan.py", "scripts/mutation/crates_mutation_rows_pool.py",
    "scripts/mutation/crates_mutation_rows_runs.py", "scripts/mutation/crates_mutation_rows_selftest_fixtures.py", "scripts/mutation/crates_mutation_rows_stage.py",
    "scripts/mutation/crates_mutation_rows_static.py", "scripts/mutation/crates_mutation_rows_support.py", "scripts/mutation/crates_mutation_rows_targets.py",
    "scripts/mutation/crates_mutation_rows_units.py", "scripts/mutation/lib_broad_mutation_selftest_fixtures.py", "scripts/mutation/lib_mutation_units.py",
    "scripts/mutation/lib_release_only_tests.py", "scripts/mutation/mutation-prune.py",
)
# 三张库表拼成的整张：自证比表、逐行数 source 行时整张一起用。拼的这一处不在读文件的路径上（own_library_digest 逐张挑，不读它），不受折叠上限管
OWN_SCRIPT_LIBRARY_FILES = GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES + CRASH_REPLAY_AND_HOST_LIBRARY_FILES + MUTATION_PYTHON_LIBRARY_FILES
# 每张库表最多几份：比算键库的折叠上限 FOLD_LIMIT（64）少 16 份，一张表涨到撞上那个上限、库悄悄不进键之前，先被自证「库表：」那一格判红、照类再拆一张
OWN_SCRIPT_LIBRARY_TABLE_LIMIT = 48
# 弄坏开关 own-library-table-short 下 own_library_digest 挑不到、表那一格比的表里也没有的一项（表少一项）
OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK = ".claude/scripts/preflight.sh"
# 弄坏开关 own-library-table-phantom 下表那一格比的表里多出来的一项：表里点名、本仓盘上没有（那一格的「表里每一份都在本仓盘上」那一项列出它）
OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK = ".claude/gate.d/lib/library-not-on-disk.sh"
# 三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的库正文里，script_source_reading 认不出写法的 source 行：
# (这一行所在的那一份（相对仓根）, 照写的参数, 它是那一份里第几条照写这个参数的 source 行（认出的、认不出的一起数，从 1 起）) → 它引的库（相对仓根）；
# 嵌在 shell 里的别的语言（python 的 if source is、for source in，awk 的 for … in）、echo 的说明文字里写的、自证里拿样本树或自己当库的、虚机里读的文件这类不是本仓的库的写空串。
# 登记了库的照那份库比，空串的不比；登记过的不让那一份照读正文判，没登记的认不出的那一行让它照读正文判（same_content_as_own）：同一份里多出一条照写同一个参数的，
# 它的第几条对不上登记，照没登记办。认不出的都登记了、登记的每一条都还在由自证「同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在」核
OWN_SCRIPT_UNRECOGNIZED_SOURCES = {
    (".claude/gate.d/lib/crash-point-replay-crash-case-cells.sh", '"$library"', 1): "scripts/hosts/heavy-threads.sh",
    (".claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh", "in", 1): "",
    (".claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh", "in", 2): "",
    (".claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh", "is", 1): "",
    (".claude/gate.d/lib/doc-text-content-and-history-cells.sh", "in", 1): "",
    # 门禁 doc-text 的 dated-status 那一格在子 bash 里 source 规范副本的 lib.sh：bash -c 'source "$1" && kb_directories "$2"' dated-status "$REPO_ROOT/.claude/singlefs-ai-sop/scripts/lib.sh" …
    (".claude/gate.d/lib/doc-text-content-and-history-cells.sh", '"$1"', 1): ".claude/singlefs-ai-sop/scripts/lib.sh",
    (".claude/scripts/lkmm.sh", '"${PREFLIGHT_PROGRAM%/*}/lib.sh"', 1): ".claude/singlefs-ai-sop/scripts/lib.sh",
    (".claude/scripts/preflight.sh", '"$project_preflight_library"', 1): ".claude/singlefs-ai-sop/scripts/preflight.sh",
    # 自证在子 bash 里 source 同目录的 crash-amplification-lib.sh（bash -c 'source "$1" …' … "$item_script_directory/crash-amplification-lib.sh"），六处
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 1): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 2): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 3): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 4): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 5): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-item.sh", '"$1"', 6): "scripts/crash/crash-amplification-lib.sh",
    ("scripts/crash/crash-amplification-lib.sh", '"$2"', 1): "",
    ("scripts/crash/crash-amplification-two-hosts.sh", '"$2"', 1): "",
    # 自证在子 bash 里 source ../hosts/heavy-threads.sh（bash -c 'source "$1"; …' _ "$script_directory/../hosts/heavy-threads.sh"）
    ("scripts/devices/build-patched-qemu.sh", '"$1"', 1): "scripts/hosts/heavy-threads.sh",
    ("scripts/devices/vm-bench.sh", "/args", 1): "",
    ("scripts/devices/vm-bench.sh", "./args", 1): "",
    # 怎么办那一句里写的单跑命令（echo 的说明文字），不是 source
    ("scripts/experiment-apparatus/replay-inputs-and-build.sh", "scripts/crash/crash-amplification-lib.sh", 2): "",
    # 自证在子 bash 里 source scripts/hosts/heavy-threads.sh（bash -c 'source scripts/hosts/heavy-threads.sh; …'），四处
    ("scripts/experiment-apparatus/replay-selftest.sh", "scripts/hosts/heavy-threads.sh", 1): "scripts/hosts/heavy-threads.sh",
    ("scripts/experiment-apparatus/replay-selftest.sh", "scripts/hosts/heavy-threads.sh", 2): "scripts/hosts/heavy-threads.sh",
    ("scripts/experiment-apparatus/replay-selftest.sh", "scripts/hosts/heavy-threads.sh", 3): "scripts/hosts/heavy-threads.sh",
    ("scripts/experiment-apparatus/replay-selftest.sh", "scripts/hosts/heavy-threads.sh", 4): "scripts/hosts/heavy-threads.sh",
    # 自证在子 bash 里 source 它自己（"$library_path" 是这一份）
    ("scripts/gate/changed-paths.sh", '"$1"', 1): "",
    # 自证在子 bash 里 source 同目录的 heavy-threads.sh（bash -c 'source "$1"; heavy_thread_default' capped-selftest "$here/heavy-threads.sh"）
    ("scripts/hosts/capped.sh", '"$1"', 1): "scripts/hosts/heavy-threads.sh",
    # 自证在子 bash 里 source 样本树里的拷贝与它自己
    ("scripts/hosts/heavy-threads.sh", '"$1"', 1): "",
    ("scripts/hosts/heavy-threads.sh", '"$1"', 2): "",
}
# 临时 prune-server 的包装写法（temporary_service_wrapped）：scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <原命令…>
TEMPORARY_SERVICE_WRAPPER_DIRECTORY = "scripts/prune-server"
TEMPORARY_SERVICE_WRAPPER_SCRIPT = "prune_server_client_selftest.py"
TEMPORARY_SERVICE_WRAPPER_MODE = "--with-temporary-service"
TEMPORARY_SERVICE_BINARY_OPTION = "--server-binary"
# 要在临时 prune-server 底下起的自证登记表（同目录，写法在表头；temporary_service_bare_selftest 读它）：一行 `<相对仓根的路径><空白><required 或 own-server><空白># <理由>`。自证换成样本表时改这个变量
TEMPORARY_SERVICE_TABLE_NAME = "selftest-temporary-service.tsv"
TEMPORARY_SERVICE_TABLE_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), TEMPORARY_SERVICE_TABLE_NAME)
TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_OWN_SERVER = "required", "own-server"
# 包装给里面那条原命令设的标记（scripts/prune-server/prune_server_client.py 的 TEMPORARY_SERVICE_VARIABLE）：自证开跑先核它，没有就退 2
TEMPORARY_SERVICE_VARIABLE = "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET"
# 自证源码里调客户端的临时服务核对时出现的那个函数名（门禁 code-tooling 认包装写法用的是同一个记号）。拆成两段写：这份文件自己也登记在自证登记表里，
# 整段写出来会被 temporary_service_table_problems 的第二条对一遍当成「调了核对、表里漏登」
TEMPORARY_SERVICE_SOURCE_MARK = "selftest_service_" + "problem"
# multi-host-run.sh 头一个参数是这几样时不是 --peer 那一种派发用法（自证），`--` 之后的命令不跑
MULTI_HOST_RUN_OTHER_MODES = frozenset({"--selftest"})
# 花括号展开与参数里的通配（classify 先照 bash 展开，文件头「花括号与参数里的通配」）：一个词最多展开成这么多份，超出的不判
WORD_EXPANSION_LIMIT = 1024
# 展开出来的命令再切一遍、再判时又带花括号或通配的（文件名里本身带这几个字符），最多再套这么多层
WORD_EXPANSION_DEPTH_LIMIT = 4
# 花括号展开只看这几种记号（brace_gobble）；一个词最多看这么多个，配不上的花括号多到超过的这个词不展开
BRACE_TOKENS = re.compile(r"\$\{|[{},]|\.\.")
BRACE_SCAN_LIMIT = 200000
# 花括号里的序列 {<起>..<止>[..<步长>]} 的展开交 shell_words.brace_sequence（起止同是整数或同是一个字母；步长是整数，0 当 1，正负只看起止的方向）
# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
# cargo mutants 只打帮助或版本的参数：只带这几个（一个或几个）时不起 cargo test
MUTANTS_QUERY_ARGUMENTS = frozenset({"--help", "-h", "--version", "-V"})
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


def test_invocation(subcommand, rest):
    """cargo 的子命令与它后面的参数 → (怎么起的：'test'、'nextest'、'run'、'bench' 或 None, 照 cargo test 的写法读的参数)。
    bench 与 test 同判（--no-run 照 test 的；开关 bench-unjudged 下不判）；nextest 只认 run（r）；miri、llvm-cov、hack 剥掉自己那一层再看里面那个子命令（llvm-cov 不带子命令时就是跑测试）；mutants 反复起 cargo test。"""
    if subcommand in ("test", "t"):
        return "test", rest
    if subcommand in ("run", "r"):
        return "run", rest
    if subcommand == "bench" and not break_is_set("bench-unjudged"):
        return "bench", rest
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
        if rest and set(rest) <= MUTANTS_QUERY_ARGUMENTS and not break_is_set("mutants-help-counted"):
            return None, []   # 只打帮助或版本，不起 cargo test
        return "test", rest
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


# 门禁阶段：.claude/gate.d/ 顶层的 *.sh（lib-*.sh 是共用库、lib/ 底下的不是阶段），按文件名认，不按编号认
GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/([^/]+\.sh)$")
# cargo 编出来的测试二进制：<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制的元数据哈希>，名字里的 - 已换成 _
TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")


def gate_stage_name(word, directory):
    """命令词是 .claude/gate.d/ 顶层的一道阶段时交回它的文件名，不是交 None。"""
    for candidate in (word, shell_words.resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match and not match.group(1).startswith(GATE_TOP_LIBRARY_PREFIX):
            return match.group(1)
    return None


def located_in(word, directory, relative_directory):
    """命令词（照写的样子，或按当前目录解开之后）是不是正好落在 relative_directory 这一层（相对仓根的写法，按路径尾部比）。"""
    for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""):
        parent = os.path.dirname(candidate)
        if parent == relative_directory or parent.endswith("/" + relative_directory):
            return True
    return False


def command_word_expanded(words, variables):
    """命令词带 `$名` / `${名}` 的，用这段命令文本里前面赋过的值与 $HOME 展开（lib_shell_words.py 的 expand_known_variables）；展开得出、不带空白的换上，
    展开不了的原样交回（文件头「命令词写在变量里」）。弄坏开关 command-word-variables-unexpanded 下原样交回。"""
    if not words or "$" not in words[0] or break_is_set("command-word-variables-unexpanded"):
        return list(words)
    value = shell_words.expand_known_variables(words[0], variables or {})
    if not value or any(character.isspace() for character in value):
        return list(words)
    return [value] + list(words[1:])


class SelftestRow(NamedTuple):
    kind: str             # selftest（脚本、钩子、门禁共用库的自证）或 samples（门禁阶段的样本档）
    tools_directory: str  # samples:<目录> 里的目录（样本根里放假工具的那一层，相对样本根）；没写交空串
    reason: str           # # 之后那段
    line: int             # 在表里第几行


def selftest_directories():
    """自证认的那几个目录（SELFTEST_DIRECTORIES）。弄坏开关 task-server-directory-unrecognized 下去掉 task-server、
    prune-server-directory-unrecognized 下去掉 prune-server、gpu-verification-server-directory-unrecognized 下去掉 gpu-verification-server
    （登记表里那一层的行算写坏的行，那一层的 --selftest 不当自证）。"""
    if break_is_set("task-server-directory-unrecognized"):
        return tuple(place for place in SELFTEST_DIRECTORIES if place != TASK_SERVER_DIRECTORY)
    if break_is_set("prune-server-directory-unrecognized"):
        return tuple(place for place in SELFTEST_DIRECTORIES if place != PRUNE_SERVER_DIRECTORY)
    if break_is_set("gpu-verification-server-directory-unrecognized"):
        return tuple(place for place in SELFTEST_DIRECTORIES if place != GPU_VERIFICATION_SERVER_DIRECTORY)
    return SELFTEST_DIRECTORIES


def device_selftest_layer(relative_directory):
    """相对仓根的目录是不是实验装置那一层（DEVICE_SELFTEST_PARENT 下正好一层）。弄坏开关 device-layer-unrecognized 下一律为假（照旧不认这一层）。"""
    if break_is_set("device-layer-unrecognized"):
        return False
    return os.path.dirname(relative_directory) == DEVICE_SELFTEST_PARENT and os.path.basename(relative_directory) not in ("", ".", "..")


def stage_samples_kind_is_valid(kind):
    """门禁阶段那一行的登记认不认得：samples，或 samples: 后面跟一个相对样本根、不带 .. 与空白的假工具目录。"""
    if kind == STAGE_SAMPLES:
        return True
    if not kind.startswith(STAGE_SAMPLES_WITH_TOOLS):
        return False
    tools = kind[len(STAGE_SAMPLES_WITH_TOOLS):]
    return bool(tools) and not os.path.isabs(tools) and ".." not in tools.split("/") and not any(character.isspace() for character in tools)


@functools.lru_cache(maxsize=8)
def selftest_registry_at(path, modified, size):
    """登记表 → ({键: SelftestRow}, [写坏的行，逐条一句])。modified、size 只当缓存的键（看门狗常驻，表改了要重读）。
    认不出的种类、没有 # 理由或理由太短、不在登记目录里的、重复的（留第一行）不收；弄坏开关 selftest-registry-lenient 下照收、不报。"""
    rows, problems = {}, []
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        return {}, [f"读不了登记表 {path}：{error}"]
    lenient = break_is_set("selftest-registry-lenient")
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split(None, 2)
        key, kind = fields[0], fields[1] if len(fields) > 1 else ""
        reason = fields[2][1:].strip() if len(fields) > 2 and fields[2].startswith("#") else ""
        stage_row = os.path.dirname(key) == GATE_STAGE_DIRECTORY and not gate_top_library_key(key)
        wrong = []
        if stage_row:
            if not stage_samples_kind_is_valid(kind):
                wrong.append(f"门禁阶段的登记「{kind}」认不出（只认 {STAGE_SAMPLES}、{STAGE_SAMPLES_WITH_TOOLS}<相对样本根、不带 .. 的假工具目录>）")
        elif kind != SELFTEST_KIND:
            wrong.append(f"登记「{kind}」认不出（只认 {SELFTEST_KIND}；门禁阶段的样本档写在 .claude/gate.d/<阶段>.sh 那一行）")
        if len(reason) < SELFTEST_REASON_MINIMUM:
            wrong.append(f"# 之后的理由不到 {SELFTEST_REASON_MINIMUM} 个字")
        if (not stage_row and os.path.dirname(key) not in selftest_directories() and not device_selftest_layer(os.path.dirname(key))
                and not gate_top_library_key(key)):
            wrong.append(f"不在 {'、'.join(selftest_directories())} 之一的这一层，也不在实验装置那一层 {DEVICE_SELFTEST_PARENT}/<装置>"
                         f"（门禁阶段的样本档写在 {GATE_STAGE_DIRECTORY} 这一层，那一层以 {GATE_TOP_LIBRARY_PREFIX} 起头的共用库照脚本登记）")
        if key in rows:
            wrong.append(f"与第 {rows[key].line} 行重复")
        if wrong and not lenient:
            problems.append(f"第 {number} 行 {key}：{'；'.join(wrong)}")
            continue
        if key not in rows:
            tools = kind[len(STAGE_SAMPLES_WITH_TOOLS):] if stage_row and kind.startswith(STAGE_SAMPLES_WITH_TOOLS) else ""
            rows[key] = SelftestRow(STAGE_SAMPLES if stage_row else SELFTEST_KIND, tools, reason, number)
    return rows, problems


def gate_top_library_key(key):
    """登记表的键（相对仓根）是不是门禁目录顶层以 lib- 起头的共用库：是就照脚本的自证判，不当门禁阶段的样本档行。
    弄坏开关 gate-top-library-as-stage 下一律为假（这一层的键都当样本档行）。"""
    if break_is_set("gate-top-library-as-stage"):
        return False
    return os.path.dirname(key) == GATE_STAGE_DIRECTORY and os.path.basename(key).startswith(GATE_TOP_LIBRARY_PREFIX)


def read_selftest_registry(path):
    """登记表此刻的内容（selftest_registry_at 按修改时刻与大小缓存）；文件不在交 ({}, [一句])。"""
    try:
        status = os.stat(path)
    except OSError as error:
        return {}, [f"读不了登记表 {path}：{error.strerror}"]
    return selftest_registry_at(path, status.st_mtime_ns, status.st_size)


def selftest_registry_problems(path, root):
    """登记表写坏的行，加上指到 root 下不存在的文件的行，逐条一句；空表是没毛病。弄坏开关 selftest-registry-lenient 下只报读不了。"""
    rows, problems = read_selftest_registry(path)
    problems = list(problems)
    for key, row in sorted(rows.items(), key=lambda item: item[1].line):
        if not os.path.isfile(os.path.join(root, key)) and not break_is_set("selftest-registry-lenient"):
            problems.append(f"第 {row.line} 行 {key}：{root} 下没有这个文件")
    return problems


def selftest_registry_key(word, directory):
    """命令词正好落在登记目录之一（照写的样子，或按当前目录解开）时交登记表里的键「目录/文件名」；落在实验装置那一层的交
    「<DEVICE_SELFTEST_PARENT>/<装置>/文件名」（按路径尾部比，同 located_in）；都不是交 None。"""
    for place in selftest_directories():
        if located_in(word, directory, place):
            return f"{place}/{os.path.basename(os.path.normpath(word))}"
    if located_in(word, directory, GATE_STAGE_DIRECTORY):
        key = f"{GATE_STAGE_DIRECTORY}/{os.path.basename(os.path.normpath(word))}"
        if gate_top_library_key(key):
            return key   # 门禁目录顶层以 lib- 起头的共用库（文件头「自证」）
    for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""):
        parent = os.path.dirname(candidate)
        layer = f"{DEVICE_SELFTEST_PARENT}/{os.path.basename(parent)}"
        grandparent = os.path.dirname(parent)
        if (grandparent == DEVICE_SELFTEST_PARENT or grandparent.endswith("/" + DEVICE_SELFTEST_PARENT)) and device_selftest_layer(layer):
            return f"{layer}/{os.path.basename(candidate)}"
    return None


def file_sha256(path):
    """文件的 sha256（十六进制）；不是普通文件、读不了交 None。"""
    try:
        digest = hashlib.sha256()
        with open(path, "rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
        return digest.hexdigest()
    except OSError:
        return None


def own_library_digest(library):
    """本仓里 library（相对仓根）那一份库的 sha256，每次调用现读：只认三张库表（GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES、CRASH_REPLAY_AND_HOST_LIBRARY_FILES、MUTATION_PYTHON_LIBRARY_FILES）里的，
    别的与读不了的交 None。
    读的那一份从表里按相等挑、照表里的字面拼路径，不拿 library 拼（同 own_script_digest）：算键库折得出读的是哪几份库。
    逐张照同一写法各挑一圈、循环变量各起一个名字，不拿拼成的整张挑、不经一个按表取参数的函数挑：算键库（scripts/gate/lib_function_units_python_module.py 的 fold）
    一处读路径折出的取值超过 FOLD_LIMIT 就整个折不出，同一个局部名字的几处赋值、同一个参数的几处实参并在一起算；只认字面元组，拼出来的整张折不出。
    弄坏开关 own-library-digest-unlisted 下一律交 None；own-library-table-short 下表里的 OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK 那一项挑不到（表少一项）。"""
    if break_is_set("own-library-digest-unlisted"):
        return None
    for gate_stage_library in GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES:
        if gate_stage_library == OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK and break_is_set("own-library-table-short"):
            continue
        if gate_stage_library == library:
            return file_sha256(os.path.join(HOOK_REPOSITORY, gate_stage_library))
    for crash_replay_host_library in CRASH_REPLAY_AND_HOST_LIBRARY_FILES:
        if crash_replay_host_library == OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK and break_is_set("own-library-table-short"):
            continue
        if crash_replay_host_library == library:
            return file_sha256(os.path.join(HOOK_REPOSITORY, crash_replay_host_library))
    for mutation_library in MUTATION_PYTHON_LIBRARY_FILES:
        if mutation_library == OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK and break_is_set("own-library-table-short"):
            continue
        if mutation_library == library:
            return file_sha256(os.path.join(HOOK_REPOSITORY, mutation_library))
    return None


def own_script_digest(location):
    """本仓里 location（相对仓根）那一份的 sha256，每次调用现读：只认按名字判的几份（KNOWN_SCRIPT_LOCATIONS 的值）、门禁阶段（GATE_STAGE_FILES）
    与派发起法（DISPATCH_LAUNCHER_SCRIPTS），别的位置与读不了的交 None。读的那一份从这几张字面表里挑，不拿 location 拼路径：
    算键库（scripts/gate/lib_function_units.py 文件头「运行期读的数据文件」一）折得出读的是哪几份文件，判法的键只连这几份、不连它们所在的整个目录。
    弄坏开关 own-script-digest-stages-unlisted 下门禁阶段不挑（交 None）。"""
    for name in KNOWN_SCRIPT_LOCATIONS:
        if KNOWN_SCRIPT_LOCATIONS[name] == location:
            return file_sha256(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[name]))
    if not break_is_set("own-script-digest-stages-unlisted"):
        for stage in GATE_STAGE_FILES:
            if os.path.join(GATE_STAGE_DIRECTORY, stage) == location:
                return file_sha256(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage))
    for launcher in DISPATCH_LAUNCHER_SCRIPTS:
        if os.path.join(DISPATCH_LAUNCHER_DIRECTORY, launcher) == location:
            return file_sha256(os.path.join(HOOK_REPOSITORY, DISPATCH_LAUNCHER_DIRECTORY, launcher))
    return None


def selftest_registered_file(word, directory, key):
    """命令词解到的真实路径（符号链接按指到的算）是不是登记表所在仓（SELFTEST_REGISTRY_ROOT）里键指的那一份文件；相对路径而当前目录认不出的交 False。
    弄坏开关 selftest-registry-by-path-tail 下一律为真（照旧只按路径尾部认）。"""
    if break_is_set("selftest-registry-by-path-tail"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    if resolved is None:
        return False
    return os.path.realpath(resolved) == os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key))


def stage_samples_row(stage):
    """登记表里 .claude/gate.d/<阶段> 那一行（SelftestRow）；没登记交 None。"""
    return read_selftest_registry(SELFTEST_REGISTRY_PATH)[0].get(f"{GATE_STAGE_DIRECTORY}/{stage}")


def sample_index_entries(repository, relative_directory):
    """repository 的暂存区（HEAD 加暂存）里 relative_directory 底下的每个文件：({相对它的路径: (模式, 对象号)}, "")；读不了交 (None, 一句原因)。
    清掉 GIT_ 开头的环境变量再起 git（GIT_DIR、GIT_INDEX_FILE 漏进来会读到别的仓或别的暂存区）。"""
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    try:
        done = subprocess.run(["git", "--literal-pathspecs", "-C", repository, "ls-files", "-s", "-z", "--", relative_directory + "/"],
                              capture_output=True, env=environment, timeout=60)
    except (OSError, subprocess.SubprocessError) as error:
        return None, f"读不了 {repository} 的暂存区（{error}）"
    if done.returncode != 0:
        return None, f"读不了 {repository} 的暂存区（git ls-files 退 {done.returncode}：{done.stderr.decode(errors='replace').strip()[:200]}）"
    entries, prefix = {}, relative_directory + "/"
    for record in done.stdout.split(b"\0"):
        if not record:
            continue
        metadata, _, path = record.partition(b"\t")
        mode, object_name, stage_number = metadata.decode().split()
        name = os.fsdecode(path)
        if stage_number != "0":
            return None, f"{repository} 的暂存区里 {name} 在合并冲突里"
        entries[name[len(prefix):]] = (mode, object_name)
    return entries, ""


def git_blob_name(data, object_name_length):
    """一段内容当 git 的 blob 时的对象号（对象号 64 位的是 sha256 仓，40 位的是 sha1 仓）。"""
    algorithm = hashlib.sha256 if object_name_length == 64 else hashlib.sha1
    return algorithm(b"blob %d\0" % len(data) + data).hexdigest()


def sample_directory_changes(sample_directory, stage):
    """喂的样本目录（<阶段目录>/fixtures/<阶段>）与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区里那一道的样本比：多出的（没跟踪的新文件，连同 setup.sh）、
    少了的、内容或执行位（符号链接的指向）不同的、不是普通文件的，逐个交一句（相对样本目录的路径与哪里不同）；读不了暂存区交 [一句]。
    弄坏开关 stage-samples-fixtures-unchecked 下一律交 []。"""
    if break_is_set("stage-samples-fixtures-unchecked"):
        return []
    entries, problem = sample_index_entries(SAMPLE_INDEX_REPOSITORY, f"{GATE_STAGE_DIRECTORY}/fixtures/{stage}")
    if entries is None:
        return [problem]
    changed, seen = [], set()
    for current, directories, files in os.walk(sample_directory):
        links = [name for name in directories if os.path.islink(os.path.join(current, name))]
        directories[:] = sorted(name for name in directories if name not in links)
        for name in sorted(files + links):
            path = os.path.join(current, name)
            relative = os.path.relpath(path, sample_directory)
            seen.add(relative)
            entry = entries.get(relative)
            if entry is None:
                changed.append(f"{relative}（暂存区里没有）")
                continue
            try:
                information = os.lstat(path)
                if stat.S_ISLNK(information.st_mode):
                    data, actual_mode = os.fsencode(os.readlink(path)), "120000"
                elif stat.S_ISREG(information.st_mode):
                    with open(path, "rb") as handle:
                        data = handle.read()
                    actual_mode = "100755" if information.st_mode & stat.S_IXUSR else "100644"
                else:
                    changed.append(f"{relative}（不是普通文件）")
                    continue
            except OSError as error:
                changed.append(f"{relative}（读不了：{error.strerror}）")
                continue
            if actual_mode != entry[0]:
                changed.append(f"{relative}（模式 {actual_mode}，暂存区里是 {entry[0]}）")
            elif git_blob_name(data, len(entry[1])) != entry[1]:
                changed.append(f"{relative}（内容与暂存区不同）")
    changed += [f"{relative}（暂存区里有、喂的样本里没有）" for relative in sorted(set(entries) - seen)]
    return changed


class StageSelftestArguments(NamedTuple):
    directories: list   # 当阶段目录的词，照写的样子（stage-selftest.sh 取最后一个）
    items: list | None  # --item 的值，照写的样子（缺值记空串）；一个 --item 都没写是 None
    list_only: bool     # 带了 --list-items


def stage_selftest_arguments(arguments):
    """照 stage-selftest.sh 自己的参数循环切：先照它开头的 preflight 摘掉每一个整词 --force；--item 后面还有词、又不是空串的，取它当值（`--item --list-items` 的值
    是 --list-items），缺值或值是空串记一个空名（它自己退 2）；--list-items 只列；别的词都当阶段目录。"""
    words = [argument for argument in arguments if argument != "--force"]
    directories, items, list_only, position = [], [], False, 0
    while position < len(words):
        word = words[position]
        if word == STAGE_SELFTEST_ITEM_OPTION:
            if position + 1 < len(words) and words[position + 1]:
                items.append(words[position + 1])
                position += 2
            else:
                items.append("")
                position += 1
            continue
        if word == STAGE_SELFTEST_LIST_OPTION:
            list_only = True
        else:
            directories.append(word)
        position += 1
    return StageSelftestArguments(directories, items or None, list_only)


def stage_selftest_directory(word, written, directory):
    """stage-selftest.sh 喂的阶段目录：(实际路径, "")；判不出交 (None, 一句原因)。written 是参数里当阶段目录的那个词，没写交 None（它自己算的那一个：<脚本所在包的根>/.claude/gate.d）。"""
    if written is not None and UNRESOLVED_WORD_CHARACTERS & set(written):
        return None, f"阶段目录 {written} 写在变量或通配里，判不出它下面有哪些阶段"
    if written is not None:
        gate_directory = shell_words.resolve_path(directory, os.path.expanduser(written))
    else:
        script = shell_words.resolve_path(directory, word)
        gate_directory = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(script))), ".claude", "gate.d") if script else None
    if gate_directory is None or not os.path.isdir(gate_directory):
        return None, f"阶段目录 {written if written is not None else '（它默认的那一个）'} 此刻不在，判不出它下面有哪些阶段"
    return gate_directory, ""


def stage_feed_problem(gate_directory, stage):
    """stage-selftest.sh 喂的阶段目录里配了样本的一道放不放行（文件头「门禁样本」）：不放行交一句，放行交 ""。带不带 --item 都用这一份。
    要登记在表里，样本要与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区逐字节相同。"""
    if stage_samples_row(stage) is None:
        return f"{stage} 的样本档没在登记表里"
    changes = sample_directory_changes(os.path.join(gate_directory, "fixtures", stage), stage)
    if changes:
        return (f"{stage} 的样本与 {SAMPLE_INDEX_REPOSITORY} 的 HEAD 加暂存区不同（{len(changes)} 处：{'、'.join(changes[:3])}"
                f"{' 等' if len(changes) > 3 else ''}）")
    return ""


def stage_selftest_is_own_copy(word, directory):
    """命令词解到的真实路径正是本仓那一份 stage-selftest.sh（--item / --list-items 信的是它只喂点名的几道、只列不喂）。
    弄坏开关 stage-items-script-by-name 下一律为真（照旧只按名字与位置认）。"""
    if break_is_set("stage-items-script-by-name"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    return resolved is not None and os.path.realpath(resolved) == os.path.realpath(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT]))


def stage_selftest_item_problems(word, parsed, directory, environment=None):
    """stage-selftest.sh 带 --item 或 --list-items（文件头「门禁样本」）：不放行的原因一句，放行交 ""。只判点名后真喂的那几道；点名里没有一道是配了样本的阶段，
    它自己退 2、不喂样本，放行。弄坏开关 stage-items-unjudged 下点名的几道不判，stage-directory-pwd-unexpanded 下 $PWD 不换。"""
    if not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：带 --item / --list-items 时信的是"
                "它只喂点名的几道、只列不喂，别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    if parsed.list_only:
        return ""   # 只列可点名的阶段，不喂样本
    if len(parsed.directories) > 1:
        return f"参数里有 {len(parsed.directories)} 个当阶段目录的词（{'、'.join(parsed.directories)}），认不出喂的是哪一个"
    gate_directory, problem = stage_selftest_directory(word, stage_selftest_written_directory(parsed, directory, environment) or None, directory)
    if problem:
        return problem
    if break_is_set("stage-items-unjudged"):
        return ""
    unresolved = [item for item in parsed.items if UNRESOLVED_WORD_CHARACTERS & set(item)]
    if unresolved:
        return f"--item {unresolved[0]} 写在变量或通配里，判不出点的是哪几道"
    fed = stage_selftest_fed_stages(gate_directory, parsed.items)
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in fed) if problem)


def stage_selftest_written_directory(parsed, directory, environment=None):
    """参数里当阶段目录的那个词，没写交 ""；写成 $PWD/… 的，当前目录认得出、命令文本里没另设 PWD 时换成当前目录。弄坏开关 stage-directory-pwd-unexpanded 下不换。"""
    written = parsed.directories[0] if parsed.directories else ""
    if directory is not None and "PWD" not in (environment or {}) and not break_is_set("stage-directory-pwd-unexpanded"):
        written = WORKING_DIRECTORY_VARIABLE.sub(lambda _: directory, written, count=1)
    return written


def stage_selftest_fed_stages(gate_directory, items):
    """--item 点名的、阶段目录里配了样本的几道（阶段文件名，次序是目录里的）。照 stage-selftest.sh 的切法：一个值里用空白隔开的几个名字各算一道，
    每个名字摘掉结尾的 .sh 再比；只认带 .sh 的会把不带的点名当成「没这一道」放过去。"""
    requested_stage_names = {word[:-len(".sh")] if word.endswith(".sh") else word for item in items for word in item.split()}
    return [stage for stage in sorted(os.listdir(gate_directory))
            if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))
            and stage[:-len(".sh")] in requested_stage_names]


def stage_selftest_problems(word, arguments, directory, environment=None):
    """stage-selftest.sh 喂的样本不盯的原因（文件头「门禁样本」），一句；盯交 ""。带 --item / --list-items 的走 stage_selftest_item_problems
    （弄坏开关 stage-items-as-directory 下不认这两个参数，照不带的判）；不带的，命令词要解到本仓那一份 stage-selftest.sh（stage_selftest_is_own_copy，
    弄坏开关 stage-feed-script-by-name 下不核），阶段目录里配了样本的每一道都判。"""
    if break_is_set("stage-samples-ignored"):
        return "弄坏开关 stage-samples-ignored：门禁样本一律不认"
    parsed = stage_selftest_arguments(arguments)
    if (parsed.items is not None or parsed.list_only) and not break_is_set("stage-items-as-directory"):
        return stage_selftest_item_problems(word, parsed, directory, environment)
    if not break_is_set("stage-feed-script-by-name") and not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：盯的信的是它只喂样本、"
                "别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    positional = [argument for argument in arguments if not argument.startswith("-")]
    if len(positional) > 1:
        return f"参数里有 {len(positional)} 个像阶段目录的词，认不出喂的是哪一个"
    gate_directory, problem = stage_selftest_directory(word, positional[0] if positional else None, directory)
    if problem:
        return problem
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in sorted(os.listdir(gate_directory))
                                              if stage.endswith(".sh") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))) if problem)


# 跑编译出来的代码的 cargo 子命令（t、r 是 test、run 的内置简写）；只编不跑的 build、check、clippy 不在内
COMPILED_CODE_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}
# cargo 编出来的二进制直接执行：<名字里带 target 的目录>/[<目标三元组>/]<debug|release>/[deps/]<名字>；名字里不许有点（排掉 .d、.rlib、.so）。
# CARGO_TARGET_DIR 设成名字里不带 target 的目录、再直接执行里面的二进制，认不出（测试二进制按 TEST_BINARY_PATH 照认）
BUILT_BINARY_PATH = re.compile(r"(?:^|/)[^/]*target[^/]*/(?:[^/]+/)?(?:debug|release)/(?:deps/)?[A-Za-z0-9_][A-Za-z0-9_-]*$")


def runs_compiled_code(words, directory):
    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（只许经 task-server/task 起的一类）：会就交一句说明，不会交 None。
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


# 不编、不链接的 cargo 子命令（只查版本、清单、帮助，改源码格式，动依赖与锁文件，删编译目录）；别的子命令、别名与插件一律当会编（展开之后、插件里起什么看不见）
CARGO_NON_BUILDING_SUBCOMMANDS = {"fmt", "metadata", "tree", "clean", "version", "help", "locate-project", "pkgid", "read-manifest", "verify-project",
                                  "search", "update", "generate-lockfile", "fetch", "vendor", "new", "init", "add", "remove", "info", "config", "report",
                                  "owner", "login", "logout", "yank", "uninstall"}
# 自己在里面起会编的 cargo、里面不套内存包装的仓内脚本（按文件名认：别处树里的拷贝起的同样是 cargo）；单独一个 --selftest 用的是假 cargo，不算
SCRIPTS_STARTING_BUILDS = {"compile-then-swap.py"}


def starts_a_build(words, directory):
    """一条剥掉前缀与包装的命令会不会起编译：会编的 cargo（CARGO_NON_BUILDING_SUBCOMMANDS 之外的子命令，test 带 --no-run 的也算），
    或 SCRIPTS_STARTING_BUILDS 里的脚本（python 起的、直接执行的；单独一个 --selftest 不算）。会就交一句说明，不会交 None。
    project-code-guard.sh 拿它认「跑项目代码」的一种（不经 task-server/task 起的拒，经没经内存包装都一样），主 agent 与子 agent 一样判、不看目录。
    弄坏开关 build-subcommands-unjudged 下 cargo 一律交 None；build-scripts-unjudged 下那几份脚本交 None。"""
    name = os.path.basename(words[0])
    if name == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None or found.subcommand in CARGO_NON_BUILDING_SUBCOMMANDS or break_is_set("build-subcommands-unjudged"):
            return None
        return f"cargo {found.subcommand}"
    script_index = 0
    if shell_words.PYTHON_NAME.match(name):
        invocation = shell_words.python_invocation(words[1:])
        if invocation.code_string is not None or invocation.module is not None or invocation.script_index is None:
            return None
        script_index = invocation.script_index + 1
    script = os.path.basename(words[script_index])
    if script not in SCRIPTS_STARTING_BUILDS or words[script_index + 1:] == ["--selftest"] or break_is_set("build-scripts-unjudged"):
        return None
    return f"{script}（它在里面起会编的 cargo、里面不套内存包装）"


# ── 只认经 task-server/task 起（文件头「经 task-server 起」；records/2026-10-30-编译绕过内存总上限.md 批 3b ① ③） ──
TASK_SERVER_CLIENT = "task-server/task"
TASK_SERVER_CLIENT_SUBCOMMANDS = frozenset({"submit", "confirm", "promote", "wait", "status", "cancel", "log", "clean"})   # confirm：交任务的二次确认（task-server/task_server.py 文件头 ⑦），只给主 agent；promote：在排的任务提前到队头，run-scope-guard.sh 弹窗要用户确认
# 子 agent 起提交客户端只许这两个只读的子命令（文件头「经 task-server 起」subagent_task_server_refusal）
SUBAGENT_TASK_SERVER_SUBCOMMANDS = frozenset({"status", "log"})
# 子 agent 被拒时的出路：project-code-guard.sh 与 run-scope-guard.sh 的拒绝说明都照这一句写
SUBAGENT_HAND_BACK_WAY_OUT = ("子 agent 不往统一任务服务交任务，task-server/task 的 submit、wait、cancel、clean 都不起，run_in_background 也不起（只读的 status、log 照用）："
                              "把要跑的命令照共用约束 .claude/agent-common.md「不做」一节「子 agent 不交任务」那一条拼成 submit 的整条（范围声明写在 submit 前面），"
                              "写进报告「要主 agent 跑的命令」一节交回，写明在哪棵树里起、跑完看哪几行，由主 agent 交服务")
TASK_SERVER_SUBMIT_DIRECTORY_OPTION = "--cwd"
# task-server/task submit 别的几个选项：写法照客户端 submit 的那一段（只认分开写的，`--选项=值` 客户端按用法错退 2）；task_server_submission 跳过它们才找得到 `--`
TASK_SERVER_SUBMIT_BARE_OPTIONS = ("--manual", "--full-gate")
TASK_SERVER_SUBMIT_VALUE_OPTION = "--whole-package-confirmed"
TASK_SERVER_LIBRARY_PATH = os.path.join(HOOK_REPOSITORY, "task-server", "task_server_lib.py")
# 直接跑、不经 task-server 的本仓脚本只有这两张表（钩子与看门狗都引这一处，别处不另抄）。
# 相对仓根的路径 → 放行的参数形态："any" 是除带 --selftest 之外都放；("first", {词…}) 是第一个参数是其中之一；("exact", (词…)) 是参数恰好这几个
# 谁都能直接跑的文本工具：「文本工具直接跑」（records/2026-10-30-编译绕过内存总上限.md 第 85 行），另两样见同一份记录问题清单 #7，
# 加提交客户端自己与 control check
TEXT_TOOLS_RUN_DIRECTLY = {
    "scripts/kb/replace-once.py": "any",
    "scripts/kb/replace-batch.py": "any",
    "scripts/kb/insert-row.py": "any",
    "scripts/hosts/work-directory-ownership.sh": ("first", frozenset({"claim"})),
    "scripts/kb/kb-spec-check.py": "any",
    "scripts/kb/stale-candidates.py": "any",
    "scripts/kb/kb-sections.py": "any",
    "scripts/agents/checklist-specs.py": "any",
    "scripts/agents/cite-check.py": "any",
    "scripts/kb/claim-experiment.sh": "any",
    "scripts/kb/relabel-item.py": "any",          # 主 agent 定的（records/2026-10-30-编译绕过内存总上限.md 问题清单 #7）：kb-scribe 定义点名的改标签工具
    "scripts/agents/copy-originals.py": "any",    # 同上：共用约束「写」一节教的备份原文件写法
    "scripts/kb/quote-kb.py": "any",              # 与 checklist-specs.py 同类的 kb 抽取工具
    "scripts/kb/quote-rust-items.py": "any",      # 与 quote-kb.py 同类的 Rust 源码抽取工具：只读 .rs 文本、按项名整段写到标准输出，不编译、不起项目代码
    "scripts/kb/append-record.py": "any",          # records/2026-11-17-subagent账本自上而下清欠.md 一百七十六节 ⑭：bash-command-detector.sh ⑲ 拒 Bash 往 records/ 追加、出路就是它，与 insert-row.py 同类的写前复核工具，主 agent 与子 agent 都直接跑
    "scripts/kb/ledger-append-section.py": "any",  # 主 agent 定的（records/2026-10-29-一百三十七节那一轮调度记录.md 批次 #19）：往现行拆分提案账本追加一节，取号与写入一步做完（records/2026-10-29-subagent拆分提案.md 第一百五十六节），主 agent 与子 agent 都直接跑
    "scripts/hosts/minimal-copy.py": "any",        # 副本拷贝不全的问题必须根治（records/2026-11-17-subagent账本自上而下清欠.md 第一百九十九节）：建仓副本只用它（范围按闭包现算），只 rsync、不编，主 agent 与子 agent 都直接跑
    "scripts/hosts/lib_repository_copy.py": ("first", frozenset({"entries"})),  # 同上：只打算出的副本闭包，不拷不编
    "scripts/gate/make-swap-spec.py": "any",      # 从基线与新版出 scripts/gate/hook-swap.py 的规格：只读两棵树、只写工作根下的 --out，不起子进程；工具员在草稿目录里出规格就起它（.claude/agents/tooling-writer.md「做什么」第 3 步），主 agent 与子 agent 都直接跑
    ".claude/singlefs-ai-sop/scripts/gate-overlap.py": ("exact", ("--list",)),
    TASK_SERVER_CLIENT: ("first", TASK_SERVER_CLIENT_SUBCOMMANDS),
    "task-server/control": ("first", frozenset({"check", "records", "bookings", "key-stats"})),   # records、key-stats 只读本机那份库（mode=ro），bookings 只读服务 /health 的记账明细，起在服务里会派到第二台、读不到
}
# 只给主 agent 直接跑的会话工具：起在服务的单元里就丢了会话身份（停进程按会话认、定时提醒与看门狗按会话号认）或停不了服务自己（control 的装与起停）
ACTIVE_SESSIONS_SCRIPT = "scripts/agents/active-sessions.py"
MAIN_AGENT_TOOLS_RUN_DIRECTLY = {
    ".claude/singlefs-ai-sop/scripts/proc.py": "any",
    "scripts/agents/remind.sh": "any",
    "scripts/agents/watch.sh": "any",
    "scripts/agents/agent-watch.py": "any",
    "scripts/agents/lib_session_registry.py": "any",  # 会话登记表现场查（by-session、by-name）：读本机 ~/.claude/sessions 与 /proc，起在服务里会派到第二台、查到的是那一台的登记表
    "scripts/agents/who-touched.py": "any",  # 查谁改过某个路径、谁提到过某个符号：读本机 ~/.claude/sessions 与会话记录，只读；起在服务里会派到第二台、查到的是那一台的登记表
    ACTIVE_SESSIONS_SCRIPT: "any",  # 查此刻活着的会话与各自最近一次自己动作：读本机 ~/.claude/sessions 与会话记录，只读；起在服务里会派到第二台、那一台的登记表是空的；带 --selftest 的那一形不在名单里
    "scripts/agents/dispatch-prompt.py": "any",
    "scripts/hosts/queue-health.py": "any",  # 按任务号现算内存：读此刻两台的 /proc 与 cgroup，排队就量不到此刻，起在服务里也会派到任意一台
    "task-server/control": ("first", frozenset({"install", "start", "stop", "watch", "clear-queue", "drop-history", "pin-cap", "unpin-cap", "pause", "resume", "recognized-card-lease"})),
    # 判定库服务的装与起停、监听：装的是本机用户级 systemd 单元，起在任务服务里就装到别处或丢了会话
    "prune-server/control": ("first", frozenset({"install", "swap-binary", "start", "stop", "check", "watch"})),
    # GPU 批处理验证服务的装与起停、查：装、起的是点名了卡的每一台的用户级 systemd 单元（第二台经 ssh 转），起在任务服务里就落到派到的那一台、丢了会话
    "gpu-verification-server/control": ("first", frozenset({"install", "start", "stop", "check"})),
    # 用户「模拟任务来跑」：服务自证起的是临时目录里的模拟服务与假主机，不进真队列等真任务
    "task-server/task_server_lib.py": ("exact", ("--selftest",)),
}
# 本仓脚本文件头的准入声明（.claude/singlefs-ai-sop/rules/preflight-discipline.md）：改名拷走的照认是本仓的代码
PROJECT_SCRIPT_HEADER = re.compile(rb"^#\s*(?:admission|run-condition):", re.MULTILINE)
PROJECT_HEAD_BYTES = 4096
# python3 -m 起单测的模块（lib_shell_words 交出来的是 <模块>.py）
TEST_RUNNER_MODULE_FILES = frozenset({"pytest.py", "unittest.py", "doctest.py", "nose2.py"})
# systemctl 只读的子命令：别的子命令（stop、kill、disable、mask、edit、restart、reset-failed、set-property 这类）点到 task-server 的单元就拒
SYSTEMCTL_READ_ONLY_VERBS = frozenset({"status", "show", "cat", "is-active", "is-enabled", "is-failed", "list-units", "list-unit-files",
                                       "list-dependencies", "list-timers", "list-sockets", "list-jobs", "help", "show-environment"})
# 不点名单元也停掉整个用户级服务管理器（连同 task-server）的子命令
SYSTEMCTL_MANAGER_VERBS = frozenset({"exit", "halt", "poweroff", "reboot", "kexec", "soft-reboot", "daemon-reexec", "isolate", "emergency", "rescue", "default", "switch-root"})
SYSTEMCTL_OPTIONS_WITH_VALUE = frozenset({"-t", "--type", "-p", "--property", "-H", "--host", "-M", "--machine", "-n", "--lines", "-o", "--output", "-s", "--signal",
                                          "--kill-whom", "--kill-value", "--state", "--root", "--image", "--job-mode", "--what", "--timestamp", "--message",
                                          "--drop-in", "--when", "--preset-mode", "--legend", "--check-inhibitors", "--boot-loader-menu", "--boot-loader-entry",
                                          "--reboot-argument"})
# 服务状态目录下放编译目录的那一层（派发记录点名它；task_server_lib.py 把编译目录放在 TASK_SERVER_BUILD_DIRECTORY；两处都只由服务删，谁都不直接删）
TASK_SERVER_STATE_BUILDS_NAME = "builds"
# 删文件与目录的命令词（命令位置上的）、find 里删东西的写法、python 代码里删的写法：点到服务的编译目录就拒（build_directory_removal）
REMOVAL_COMMAND_NAMES = frozenset({"rm", "rmdir", "unlink", "shred"})
FIND_EXECUTION_ACTIONS = frozenset({"-exec", "-execdir", "-ok", "-okdir"})
FIND_LEADING_OPTIONS = frozenset({"-H", "-L", "-P"})
PYTHON_REMOVAL_CALL = re.compile(r"\b(?:shutil\.rmtree|os\.(?:remove|unlink|rmdir|removedirs)|\.unlink|\.rmdir)\s*\(")
# 删服务编译目录被拒时说明里那一句：树的认法照 task-server/task_server_lib.py 文件头「编译目录」（build_tree_root）
BUILD_TREE_REMOVAL_NOTE = ("编译目录归树、不归任务，不用自己删：树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层（哪一层都没有就是解开后的 --cwd 本身），"
                           "树删了服务跟着删它那一整组")
# 载入 task-server 的共用库（拿它的密钥函数、直接开库）的认法（task_server_library_loaded，文件头 task_server_text_bypass 那一条写全了认法与认不到的）：
# 命令文本切成命令位置上的一条条命令（lib_shell_words 的 commands_at_command_position），只读命令（TASK_SERVER_READ_ONLY_COMMANDS、sed -n、git 的只读子命令）与它们读的 heredoc 不判，
# cat / tee 写进文件的 heredoc 正文只在写进 .py 文件时判；别的每一条命令（python3 -c 的代码、喂给它的 heredoc、here-string、管道前一段、< 的文件连同它的参数）各算一段，
# 同一段里库名（模块名 task_server_lib，文件名 task_server_lib.py 里也有它）与载入写法（TASK_SERVER_LIBRARY_LOAD_WORDS）同在就算载入，拆在几行、先存进变量再用的一样；
# python 从标准输入读程序、而标准输入是 < 或管道前一段只读命令读的这份库的，同样算
TASK_SERVER_LIBRARY_NAME = re.compile(r"\btask_server_lib\b")
TASK_SERVER_LIBRARY_FILE_NAME = "task_server_lib.py"
TASK_SERVER_LIBRARY_LOAD_WORDS = re.compile(r"\b(?:import|__import__|import_module|exec|execfile|compile|run_module|run_path|spec_from_file_location|SourceFileLoader|"
                                            r"SourcelessFileLoader|load_source|load_module|exec_module)\b")
# 弄坏开关 task-server-library-load-words-file-calls-only 下只认这几种按文件载入的调用
TASK_SERVER_LIBRARY_FILE_LOAD_CALLS = re.compile(r"\b(?:spec_from_file_location|SourceFileLoader|SourcelessFileLoader|run_path|load_source|execfile)\b")
# 只读命令：命令词是这几个的，它的参数与读的 heredoc 里写到库名与载入写法都不算载入
TASK_SERVER_READ_ONLY_COMMANDS = frozenset({"grep", "egrep", "fgrep", "rg", "wc", "cat", "head", "tail", "ls", "stat", "sha256sum", "md5sum", "diff", "cmp"})
# git 带这几个子命令（git 之后头一个不以 - 起头的词）也算只读命令
TASK_SERVER_READ_ONLY_GIT_SUBCOMMANDS = frozenset({"grep", "show", "diff", "log", "blame", "ls-files"})
# 把 heredoc 正文写进文件的命令：正文写进 .py 文件的照判（写出来再起就是载入），写进别的文件的不判
TASK_SERVER_HEREDOC_WRITERS = frozenset({"cat", "tee"})


class ProjectRun(NamedTuple):
    detail: str       # 起的是哪一份（相对仓根的路径，认不出相对路径的照写的样子）
    allowed: str      # "nobody"：要经 task-server 起；"main-agent"：主 agent 能直接跑（MAIN_AGENT_TOOLS_RUN_DIRECTLY）；"anyone"：谁都能直接跑（TEXT_TOOLS_RUN_DIRECTLY）
    real_path: str    # 起的那份文件的真实路径（调用方拿它认名单上的脚本正文里的那几行）


@functools.lru_cache(maxsize=1)
def project_top_directories():
    """本仓仓根下的目录名（target、.git 除外）：认仓副本里的文件用（project_relative_path）。"""
    try:
        return frozenset(name for name in os.listdir(HOOK_REPOSITORY)
                         if name not in ("target", ".git") and os.path.isdir(os.path.join(HOOK_REPOSITORY, name)))
    except OSError:
        return frozenset()


def project_relative_path(real_path):
    """真实路径落在本仓里的交相对仓根的路径；落在别处、而从某一层上级往下的那一段（头一段是本仓仓根下的目录）在本仓里有这份文件的
    （仓副本、git worktree、拷走的整棵树）交那一段；都不是交 None。"""
    if real_path == HOOK_REPOSITORY or real_path.startswith(HOOK_REPOSITORY + os.sep):
        return os.path.relpath(real_path, HOOK_REPOSITORY)
    parent = os.path.dirname(real_path)
    while parent and parent != os.path.dirname(parent):
        relative = os.path.relpath(real_path, parent)
        if os.sep in relative and relative.split(os.sep, 1)[0] in project_top_directories() and os.path.isfile(os.path.join(HOOK_REPOSITORY, relative)):
            return relative
        parent = os.path.dirname(parent)
    return None


def has_project_header(real_path):
    try:
        with open(real_path, "rb") as handle:
            return bool(PROJECT_SCRIPT_HEADER.search(handle.read(PROJECT_HEAD_BYTES)))
    except OSError:
        return False


def argument_form_allows(form, arguments):
    if form == "any":
        return "--selftest" not in arguments
    kind, values = form
    if kind == "first":
        return bool(arguments) and arguments[0] in values
    return tuple(arguments) == tuple(values)


def same_as_own_copy(relative, real_path):
    """真实路径是本仓那一份（relative 相对仓根），或与它逐字节相同的拷贝。"""
    own = os.path.join(HOOK_REPOSITORY, relative)
    return real_path == os.path.realpath(own) or (os.path.isfile(own) and filecmp.cmp(real_path, own, shallow=False))


def directly_run_level(relative, real_path, arguments):
    """名单上直接跑的脚本：谁都能跑交 "anyone"，只给主 agent 交 "main-agent"，不在名单上（或参数不合、内容与本仓那一份不同）交 "nobody"。
    弄坏开关 text-tools-refused 下 TEXT_TOOLS_RUN_DIRECTLY 不认，main-agent-tools-refused 下 MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认，
    active-sessions-unlisted 下 MAIN_AGENT_TOOLS_RUN_DIRECTLY 里只有 ACTIVE_SESSIONS_SCRIPT 那一项不认。"""
    if relative is None or not same_as_own_copy(relative, real_path):
        return "nobody"
    form = TEXT_TOOLS_RUN_DIRECTLY.get(relative)
    if form is not None and argument_form_allows(form, arguments) and not break_is_set("text-tools-refused"):
        return "anyone"
    form = MAIN_AGENT_TOOLS_RUN_DIRECTLY.get(relative)
    if relative == ACTIVE_SESSIONS_SCRIPT and break_is_set("active-sessions-unlisted"):
        form = None
    if form is not None and argument_form_allows(form, arguments) and not break_is_set("main-agent-tools-refused"):
        return "main-agent"
    return "nobody"


def launched_file(words, directory, launcher):
    """一条剥掉前缀与包装的命令（lib_shell_words 的 CommandAtPosition 的词与 launcher）起的文件的真实路径：bash / sh / source / python 起的、
    直接执行的（命令词带 /）；盘上不在、读不了、认不出当前目录的，shell 启动文件与 awk -f 这类程序文件，PATH 上找的命令交 None。"""
    if not words or launcher in ("startup", "program"):
        return None
    word = words[0]
    if launcher is None and "/" not in word:
        return None
    if not os.path.isabs(os.path.expanduser(word)) and not directory:
        return None
    path = shell_words.resolve_path(directory, os.path.expanduser(word))
    if path is None or not os.path.isfile(path):
        return None
    return os.path.realpath(path)


def project_code_use(words, directory, launcher=None):
    """一条剥掉前缀与包装的命令是不是在跑本仓的代码（只许经 task-server/task 起的那一类，文件头「经 task-server 起」）：是交 ProjectRun，不是交 None。
    认的：起的文件（launched_file）真实路径落在本仓里、或是本仓某份文件的拷贝（project_relative_path）、或开头 4 KiB 里有行首 `# admission:` / `# run-condition:`；
    `python3 -m pytest`、`-m unittest` 这类起单测的。名单上直接跑的（TEXT_TOOLS_RUN_DIRECTLY、MAIN_AGENT_TOOLS_RUN_DIRECTLY）照交，allowed 写它能给谁直接跑。
    会编的 cargo 由 starts_a_build 认，编出来的二进制由 runs_compiled_code 认，这里不认。弄坏开关 project-scripts-unjudged 下一律交 None。"""
    if break_is_set("project-scripts-unjudged") or not words:
        return None
    if launcher == "python" and "/" not in words[0] and words[0] in TEST_RUNNER_MODULE_FILES:
        return ProjectRun(f"python3 -m {words[0][:-len('.py')]} 起单测", "nobody", "")
    real_path = launched_file(words, directory, launcher)
    if real_path is None:
        return None
    relative = project_relative_path(real_path)
    if relative is None and not has_project_header(real_path):
        return None
    return ProjectRun(relative or words[0], directly_run_level(relative, real_path, list(words[1:])), real_path)


def task_server_client_subcommand(words, directory, launcher=None):
    """命令词是本仓的 task-server/task（仓副本里与它逐字节相同的拷贝同算；python3 起的、直接执行的都认）：交它的子命令（没写的交空串）；不是交 None。"""
    if not words:
        return None
    real_path = launched_file(words, directory, launcher)
    if real_path is None or project_relative_path(real_path) != TASK_SERVER_CLIENT or not same_as_own_copy(TASK_SERVER_CLIENT, real_path):
        return None
    return words[1] if len(words) > 1 else ""


def subagent_task_server_refusal(words, directory, launcher=None):
    """子 agent 起这一条要拒就交一句说明（起的是哪个子命令），不拒交 None：命令是本仓的提交客户端（task_server_client_subcommand），
    子命令不是 SUBAGENT_TASK_SERVER_SUBCOMMANDS 里的（submit、wait、cancel、clean 与认不出的都拒）。
    project-code-guard.sh 判子 agent 的每一条命令（读进来的脚本正文里的照判）、run-scope-guard.sh 判子 agent 经 submit 交的验证命令，都只调这一处。"""
    subcommand = task_server_client_subcommand(words, directory, launcher)
    if subcommand is None or subcommand in SUBAGENT_TASK_SERVER_SUBCOMMANDS:
        return None
    return f"{TASK_SERVER_CLIENT} {subcommand}" if subcommand else f"{TASK_SERVER_CLIENT}（没写子命令）"


def task_server_submission(words, directory, launcher=None):
    """命令词是本仓的 task-server/task（仓副本里与它逐字节相同的拷贝同算；python3 起的、直接执行的都认）而子命令是 submit：
    交 (`--` 之后那条命令的词, 任务的当前目录：--cwd 给了的取它照这一条的目录解开，没给是 None)；不是交 None。
    `--` 之前认 --cwd <目录>、--manual、--full-gate、--whole-package-confirmed <文件>（次序不限，写法照客户端 submit 的那一段）。
    认不出的选项、没写 `--`、`--` 之后没有词的，客户端按用法错退 2、什么都不起，交 None（这一条照名单上的客户端放行）。
    弄坏开关 task-server-submission-unparsed 下一律交 None（submit 那一条只当名单上的客户端放行，后面那条一个字都不判）；
    task-server-submit-flags-unparsed 下只认 --cwd（带另外三个选项的提交一律交 None）。"""
    if break_is_set("task-server-submission-unparsed") or len(words) < 2 or words[1] != "submit":
        return None
    if task_server_client_subcommand(words, directory, launcher) != "submit":
        return None
    arguments, position, task_directory = list(words[2:]), 0, None
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            inner = arguments[position + 1:]
            if not inner:
                return None
            resolved = shell_words.resolve_path(directory, os.path.expanduser(task_directory)) if task_directory and directory else task_directory
            return inner, resolved
        if argument == TASK_SERVER_SUBMIT_DIRECTORY_OPTION and position + 1 < len(arguments):
            task_directory, position = arguments[position + 1], position + 2
            continue
        if argument.startswith(TASK_SERVER_SUBMIT_DIRECTORY_OPTION + "="):
            task_directory, position = argument.split("=", 1)[1], position + 1
            continue
        if not break_is_set("task-server-submit-flags-unparsed"):
            if argument in TASK_SERVER_SUBMIT_BARE_OPTIONS:
                position += 1
                continue
            if argument == TASK_SERVER_SUBMIT_VALUE_OPTION and position + 1 < len(arguments):
                position += 2
                continue
        return None
    return None


TASK_SERVER_LIBRARY_LOADED = []   # 空表是还没载；载过的放 (模块, None)，载不进放 (None, 原因)


def task_server_library():
    """按文件路径载入 task-server/task_server_lib.py（只用它的名字、缺省值与读配置的函数，不起服务）：交 (模块, None)，载不进交 (None, 原因)。
    自证换样本库时改 TASK_SERVER_LIBRARY_PATH 并清 TASK_SERVER_LIBRARY_LOADED。"""
    if not TASK_SERVER_LIBRARY_LOADED:
        try:
            specification = importlib.util.spec_from_file_location("task_server_lib_for_hooks", TASK_SERVER_LIBRARY_PATH)
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            TASK_SERVER_LIBRARY_LOADED.append((module, None))
        except Exception as error:   # 文件不在、语法错、它载入的库出错
            TASK_SERVER_LIBRARY_LOADED.append((None, f"载不进 {TASK_SERVER_LIBRARY_PATH}（{error!r}）"))
    return TASK_SERVER_LIBRARY_LOADED[0]


class TaskServerGuarded(NamedTuple):
    paths: tuple            # 库文件、签名密钥、socket 的路径（取 read_configuration；读不了是空的）
    names: tuple            # 同几样的文件名（库文件名、密钥文件名、「socket 所在目录名/socket 名」），路径写成变量拼的也认
    admitted_variable: str  # run-with-memory-cap.sh 的「已由 task-server 准入」开关变量（只由服务设）
    server_unit: str        # 服务自己的用户级单元
    task_unit_prefix: str   # 每一次尝试的单元名的头一段（单元名是 <它>-<库标签>-<号>-<第几次>.service）
    process_scripts: tuple  # 只由服务与它起的单元起的脚本（相对仓根）：服务本体与每次尝试的那一层
    build_directories: tuple  # 服务的编译目录（TASK_SERVER_BUILD_DIRECTORY 与状态目录下的 builds/，真实路径）：只由服务删（树删了跟着删整组；树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层，哪一层都没有就是解开后的 --cwd 本身），谁都不直接删


@functools.lru_cache(maxsize=1)
def task_server_guarded():
    """不经 task-server/task 不许碰的那几样，全取自 task_server_lib.py（不另抄一份）：交 (TaskServerGuarded, None)，载不进交 (None, 原因)。"""
    library, problem = task_server_library()
    if library is None:
        return None, problem
    try:
        configuration = library.read_configuration()
    except Exception:   # 配置不在、读不了：路径认不全，只认文件名
        configuration = None
    paths, names = [], [library.DATABASE_NAME]
    if configuration is not None:
        paths = [os.path.realpath(configuration.database), os.path.realpath(configuration.signing_key_file), os.path.realpath(configuration.socket)]
        key_file, runtime_directory = configuration.signing_key_file, configuration.runtime_directory
        build_roots = [configuration.build_directory, os.path.join(configuration.state_directory, TASK_SERVER_STATE_BUILDS_NAME)]
    else:   # 配置读不了：文件名取它的缺省值（default_values；没有这个函数或键的，只认库文件名）
        try:
            defaults = library.default_values(dict(os.environ))
        except Exception:
            defaults = {}
        key_file, runtime_directory = defaults.get("TASK_SERVER_SIGNING_KEY_FILE"), defaults.get("TASK_SERVER_RUNTIME_DIRECTORY")
        build_roots = [defaults.get("TASK_SERVER_BUILD_DIRECTORY")]
        if defaults.get("TASK_SERVER_STATE_DIRECTORY"):
            build_roots.append(os.path.join(defaults["TASK_SERVER_STATE_DIRECTORY"], TASK_SERVER_STATE_BUILDS_NAME))
    if key_file:
        names.append(os.path.basename(key_file))
    if runtime_directory:
        names.append(os.path.join(os.path.basename(runtime_directory), library.SOCKET_NAME))
    unit = library.UNIT_NAME
    prefix = unit[:unit.rindex("-") + 1] if "-" in unit else unit   # singlefs-task-server.service → singlefs-task-（服务起的单元名同一个头）
    scripts = tuple(os.path.relpath(path, HOOK_REPOSITORY) for path in (library.SERVER_SCRIPT, library.RUNNER_SCRIPT))
    builds = tuple(dict.fromkeys(os.path.realpath(root) for root in build_roots if root))
    return TaskServerGuarded(tuple(paths), tuple(names), library.ADMITTED_VARIABLE, unit, prefix, scripts, builds), None


def path_mentioned(text, path):
    """命令文本里点到这条路径：照写的、~/ 起头的、$HOME / ${HOME} 起头的、库文件的 -wal / -shm / -journal 跟着算（前缀相同）。"""
    home = os.path.expanduser("~")
    forms = [path]
    if path.startswith(home + os.sep):
        tail = path[len(home):]
        forms += ["~" + tail, "$HOME" + tail, "${HOME}" + tail]
    return any(form in text for form in forms)


def expanded_target(word, directory):
    """rm / find 的一个路径参数 → 真实路径：~ 与 $HOME、${HOME} 起头的换成家目录，相对路径从命令的当前目录算；别的变量与命令替换照字面算（判不到，不拒）。"""
    home = os.path.expanduser("~")
    for prefix in ("${HOME}", "$HOME", "~"):
        if word == prefix or word.startswith(prefix + "/"):
            word = home + word[len(prefix):]
            break
    return os.path.realpath(os.path.join(directory or os.getcwd(), word))


def build_directory_removal(words, directory, guarded):
    """一条剥掉前缀与包装的命令直接删服务的编译目录：rm / rmdir / unlink / shred 的目标、find 带 -delete 或 -exec rm 的起点，
    落在 guarded.build_directories 里、等于它们或是它们的上级（连同编译目录一起删）。交说明，不是交 None。"""
    name = os.path.basename(words[0])
    if name in REMOVAL_COMMAND_NAMES:
        targets, options_done = [], False
        for word in words[1:]:
            if word == "--" and not options_done:
                options_done = True
            elif options_done or not word.startswith("-"):
                targets.append(word)
    elif name == "find":
        deleting = "-delete" in words[1:] or any(word in FIND_EXECUTION_ACTIONS and position + 1 < len(words)
                                                 and os.path.basename(words[position + 1]) in REMOVAL_COMMAND_NAMES
                                                 for position, word in enumerate(words) if position)
        if not deleting:
            return None
        targets = []
        for word in words[1:]:
            if word in FIND_LEADING_OPTIONS and not targets:
                continue
            if word.startswith(("-", "(", ")", "!")):
                break
            targets.append(word)
    else:
        return None
    for target in targets:
        real = expanded_target(target, directory)
        for root in guarded.build_directories:
            if real == root or real.startswith(root + os.sep) or root.startswith(real.rstrip(os.sep) + os.sep):
                return f"直接删 task-server 的编译目录（{target}，落在 {root} 里或是它的上级；{BUILD_TREE_REMOVAL_NOTE}）"
    return None


def task_server_read_only_command(words):
    """一条命令（照写的词，words[0] 是命令词）是不是只读命令：命令词在 TASK_SERVER_READ_ONLY_COMMANDS 里，或是带 -n（--quiet、--silent）又不带 -i（--in-place）的 sed，
    或是 git 之后头一个不以 - 起头的词在 TASK_SERVER_READ_ONLY_GIT_SUBCOMMANDS 里。"""
    name = os.path.basename(words[0])
    options = [word for word in words[1:] if word.startswith("-") and word != "-"]
    if name == "sed":
        quiet = any(word in ("--quiet", "--silent") or (not word.startswith("--") and "n" in word[1:]) for word in options)
        in_place = any(word.startswith("--in-place") or (not word.startswith("--") and "i" in word[1:]) for word in options)
        return quiet and not in_place
    if name == "git":
        return next((word for word in words[1:] if not word.startswith("-")), None) in TASK_SERVER_READ_ONLY_GIT_SUBCOMMANDS
    return name in TASK_SERVER_READ_ONLY_COMMANDS


def task_server_library_code_texts(text):
    """一段命令文本里要判载不载入共用库的几段文本，与一定是载入的几条命令：([段…], [python 从标准输入读这份库的那几条…])。
    切法与哪几段算、哪几段不算写在 TASK_SERVER_LIBRARY_NAME 上面那段注释里；切不动的整段当一段（偏严）。
    弄坏开关 task-server-library-loads-whole-text 下不切、整段当一段（只读命令里写的也算，标准输入那一样不认）；
    task-server-library-loads-written-python-unjudged 下写进 .py 文件的 heredoc 正文也不判；task-server-library-stdin-program-unjudged 下标准输入那一样不认。"""
    if break_is_set("task-server-library-loads-whole-text"):
        return [text], []
    try:
        commands = shell_words.commands_at_command_position(text).commands
    except Exception:   # 切不动的文本：整段当一段判（偏严）
        return [text], []
    texts, loads = [], []
    for command in commands:
        words = [str(word) for word in command.words]
        if not words:
            continue
        name = os.path.basename(words[0])
        bodies = [body.text for body in command.standard_input_heredocs]
        if name in TASK_SERVER_HEREDOC_WRITERS and bodies:
            destinations = [str(target) for _operator, target in command.output_redirections] + (words[1:] if name == "tee" else [])
            if any(destination.endswith(".py") for destination in destinations) and not break_is_set("task-server-library-loads-written-python-unjudged"):
                texts.append("\n".join(bodies))
            bodies = []
        if task_server_read_only_command(words):
            continue
        piped = [str(word) for word in command.piped_from[0]] if command.piped_from else []
        redirected = [str(target) for _operator, target in command.input_redirections]
        texts.append("\n".join([" ".join(words)] + bodies + [" ".join(piped)] + redirected))
        if shell_words.PYTHON_NAME.match(name) and not break_is_set("task-server-library-stdin-program-unjudged"):
            invocation = shell_words.python_invocation(words[1:])
            if invocation.code_string is None and invocation.module is None and invocation.script_index is None:
                program_files = [str(target) for operator, target in command.input_redirections if operator == "<"]
                if piped and task_server_read_only_command(piped):
                    program_files.append(" ".join(piped))
                if any(TASK_SERVER_LIBRARY_NAME.search(program_file) for program_file in program_files):
                    loads.append(" ".join(words))
    return texts, loads


def task_server_library_named_with_load(code_text):
    """一段文本里库名与载入写法同在（task_server_library_code_texts 切出来的一段）。弄坏开关 task-server-library-loads-module-name-unseen 下库名只认文件名 task_server_lib.py，
    task-server-library-load-words-file-calls-only 下载入写法只认 TASK_SERVER_LIBRARY_FILE_LOAD_CALLS，task-server-library-loads-same-statement 下两样要在同一句里（按分号与换行切）。"""
    load_words = TASK_SERVER_LIBRARY_FILE_LOAD_CALLS if break_is_set("task-server-library-load-words-file-calls-only") else TASK_SERVER_LIBRARY_LOAD_WORDS
    file_name_only = break_is_set("task-server-library-loads-module-name-unseen")
    statements = re.split(r"[;\n]", code_text) if break_is_set("task-server-library-loads-same-statement") else [code_text]
    return any((TASK_SERVER_LIBRARY_FILE_NAME in statement if file_name_only else TASK_SERVER_LIBRARY_NAME.search(statement)) and load_words.search(statement)
               for statement in statements)


def task_server_library_loaded(text):
    """一段命令文本里有没有载入 task-server 的共用库（认法在 TASK_SERVER_LIBRARY_NAME 上面那段注释与文件头 task_server_text_bypass 那一条）。"""
    if not TASK_SERVER_LIBRARY_NAME.search(text):   # 文本里连库名都没有：不切
        return False
    code_texts, loads = task_server_library_code_texts(text)
    return bool(loads) or any(task_server_library_named_with_load(code_text) for code_text in code_texts)


def task_server_text_bypass(text):
    """③ 整段命令文本里绕开 task-server 的写法：点到库文件、签名密钥、socket（路径或文件名），载入 task_server_lib（task_server_library_loaded：只读命令与写进非 .py 文件的
    heredoc 里写到的不算，别的每一条命令里库名与载入写法同在的算），在命令文本里设「已由 task-server 准入」开关（赋值、export、environ 写、setenv / putenv），
    python 代码里删服务的编译目录。交 ([说明…], 判不了的原因或 None)。
    弄坏开关 task-server-paths-unjudged 下不认点到库、密钥、socket，task-server-library-loads-unjudged 下不认载入共用库，
    admitted-variable-unjudged 下不认准入开关，task-server-builds-unjudged 下不认删编译目录。"""
    if not text:
        return [], None
    guarded, problem = task_server_guarded()
    if guarded is None:
        return [], problem
    found = []
    if not break_is_set("task-server-paths-unjudged"):
        for path in guarded.paths:
            if path_mentioned(text, path):
                found.append(f"点到 task-server 的 {os.path.basename(path)}（{path}）")
        for name in guarded.names:
            if name in text and not any(name in entry for entry in found):
                found.append(f"点到 task-server 的 {name}")
    if not break_is_set("task-server-library-loads-unjudged") and task_server_library_loaded(text):
        found.append("在代码里载入 task-server/task_server_lib.py（它管库与签名密钥）")
    variable = re.escape(guarded.admitted_variable)
    setting = re.compile(variable + r"""['"]?\s*\]?\s*=(?!=)|\b(?:export|declare\s+-[A-Za-z]*x[A-Za-z]*|typeset\s+-[A-Za-z]*x[A-Za-z]*)\b[^;&|\n]*\b""" + variable
                         + r"""|(?:putenv|setenv|setdefault|update)\s*\([^)]*""" + variable)
    if not break_is_set("admitted-variable-unjudged") and setting.search(text):
        found.append(f"在 task-server 之外设 {guarded.admitted_variable}（它让内存包装跳过排队与记账，只由服务设）")
    if not break_is_set("task-server-builds-unjudged") and PYTHON_REMOVAL_CALL.search(text) and (
            any(path_mentioned(text, root) for root in guarded.build_directories) or any(os.path.basename(root) in text for root in guarded.build_directories[:1])):
        found.append(f"python 代码里删 task-server 的编译目录（{BUILD_TREE_REMOVAL_NOTE}）")
    return found, None


def systemctl_unit_touch(words, guarded):
    """交说明、不交 None 的情形：systemctl 停、禁、改 task-server 的单元（服务自己的与每一次尝试的），或停整个用户级服务管理器。"""
    position, positional = 1, []
    while position < len(words):
        word = words[position]
        if word.startswith("-"):
            position += 2 if word in SYSTEMCTL_OPTIONS_WITH_VALUE else 1
            continue
        positional.append(word)
        position += 1
    if not positional:
        return None
    verb, units = positional[0], positional[1:]
    if verb in SYSTEMCTL_READ_ONLY_VERBS:
        return None
    if verb in SYSTEMCTL_MANAGER_VERBS and "--user" in words:
        return f"systemctl --user {verb} 停掉整个用户级服务管理器（连同 {guarded.server_unit}）"
    sample_task_unit = guarded.task_unit_prefix + "x-1-1.service"
    for unit in units:
        if unit.startswith(guarded.task_unit_prefix) or fnmatch.fnmatchcase(guarded.server_unit, unit) or fnmatch.fnmatchcase(sample_task_unit, unit):
            return f"systemctl {verb} {unit}（task-server 的单元只经 task-server/control 起停、经 task-server/task cancel 取消）"
    return None


def task_server_command_bypass(words, environment, directory, launcher=None):
    """③ 一条剥掉前缀与包装的命令绕开 task-server：环境里设了「已由 task-server 准入」开关、直接起服务本体或每次尝试的那一层、
    systemctl 停、禁、改它的单元（systemctl_unit_touch）、直接删服务的编译目录（build_directory_removal）。交 ([说明…], 判不了的原因或 None)。
    弄坏开关 admitted-variable-unjudged 下不认第一样，task-server-scripts-unjudged 下不认第二样，task-server-units-unjudged 下不认第三样，task-server-builds-unjudged 下不认第四样。"""
    if not words:
        return [], None
    guarded, problem = task_server_guarded()
    if guarded is None:
        return [], problem
    found = []
    if guarded.admitted_variable in (environment or {}) and not break_is_set("admitted-variable-unjudged"):
        found.append(f"在 task-server 之外设 {guarded.admitted_variable}（它让内存包装跳过排队与记账，只由服务设）")
    real_path = launched_file(words, directory, launcher)
    if real_path is not None and not break_is_set("task-server-scripts-unjudged") and project_relative_path(real_path) in guarded.process_scripts:
        found.append(f"直接起 {project_relative_path(real_path)}（服务由 task-server/control 起停，每一次尝试的那一层只由服务起）")
    if os.path.basename(words[0]) == "systemctl" and not break_is_set("task-server-units-unjudged"):
        touched = systemctl_unit_touch(words, guarded)
        if touched:
            found.append(touched)
    if not break_is_set("task-server-builds-unjudged"):
        removal = build_directory_removal(words, directory, guarded)
        if removal:
            found.append(removal)
    return found, None
