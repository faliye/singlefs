# lib_project_code_runs.py 的部分模块：包在命令外面的启动器（flock、/usr/bin/time、systemd-run、perf、xargs、find 这一类与它们设的环境变量）、临时服务包装、认不出起什么的写法、读进来的脚本正文里的起法、花括号与通配展开、classify 与按名字判的脚本。
# 入口 lib_project_code_runs.py 按 PROJECT_CODE_RUNS_PART_NAMES 的次序 import 这一份（它 import 引导那一份与 lib_project_code_runs_base.py，用到的名字写显式 import），不单独执行；自证与调用方换掉的 FAMILY_OWN_REPOSITORY 在这一份里换。
# 弄坏开关的名单与红的格登记在入口 lib_project_code_runs.py 里，开关表写在它的文件头。
import filecmp
import functools
import glob
import importlib.util
import os
import re
import shlex
import subprocess
import sys
from typing import NamedTuple
from lib_project_code_runs_base import (
    BASH_BRACKET_PATTERN, BASH_PATTERN_TIMEOUT, BRACE_SCAN_LIMIT, BRACE_TOKENS, BROAD_MUTATION_SCRIPT_NAME, CARET_NEGATION,
    DISPATCH_LAUNCHER_DIRECTORY, DISPATCH_LAUNCHER_OPTIONS, E165_PIN_SCRIPT, EXTERNAL_PROGRAM_NAMES, EXTERNAL_PROGRAM_PREFIXES,
    EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS, GATE_LIBRARY_DIRECTORY, GATE_STAGE_DIRECTORY, GLOB_CHARACTERS, GPU_QUIESCE_DIRECTORY,
    GPU_QUIESCE_SCRIPT, HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS, MULTI_HOST_RUN_OTHER_MODES, MULTI_HOST_RUN_SCRIPT, OWN_SCRIPT_UNRECOGNIZED_SOURCES, PATTERN_EXPANSION_LIMIT,
    PROVE_RED_SCRIPT, RUN_BUILD_DIRECTORY_LOCATION, RUN_BUILD_DIRECTORY_SCRIPT, SELFTEST_PATTERN_CHARACTERS, SELFTEST_REASON_MINIMUM,
    STAGE_SELFTEST_SCRIPT, TEMPORARY_SERVICE_BINARY_OPTION, TEMPORARY_SERVICE_OWN_SERVER, TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_SOURCE_MARK,
    TEMPORARY_SERVICE_TABLE_NAME, TEMPORARY_SERVICE_VARIABLE, TEMPORARY_SERVICE_WRAPPER_DIRECTORY, TEMPORARY_SERVICE_WRAPPER_MODE,
    TEMPORARY_SERVICE_WRAPPER_SCRIPT, TWO_LOGS_DIRECTORY, TWO_LOGS_SCRIPT, UNEXPANDED_DEFINITION_DEPTH, UNEXPANDED_VARIABLE_NAME,
    UNRESOLVED_WORD_CHARACTERS, VERUS_PROOF_CHECK_SCRIPT, WORD_EXPANSION_DEPTH_LIMIT, WORD_EXPANSION_LIMIT, command_word_expanded,
    device_selftest_layer, file_sha256, gate_stage_name, gate_top_library_key, located_in, own_library_digest, own_script_digest, program_use, project_code_use,
    read_selftest_registry,
    runs_compiled_code, selftest_registered_file, selftest_registry_key, shell_words, stage_selftest_arguments, stage_selftest_directory,
    stage_selftest_fed_stages, stage_selftest_is_own_copy, stage_selftest_written_directory, starts_a_build,
)
from lib_project_code_runs_bootstrap import (
    break_is_set,
)
import lib_project_code_runs_base


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


def command_under_gpu_quiesce(words, directory=None):
    """words[0] 是 scripts 这一层的 gpu-quiesce-run.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数照它开头的 preflight 摘掉每一个 --force 之后
    是 `-- <命令…>` 时交回 (<命令…> 的词, None)：当前目录与环境照外面那一条；--selftest、没写 `--` 的交 None。弄坏开关 gpu-quiesce-opaque 下一律交 None。"""
    if break_is_set("gpu-quiesce-opaque") or not words or os.path.basename(words[0]) != GPU_QUIESCE_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, GPU_QUIESCE_DIRECTORY):
        return None
    rest = [word for word in words[1:] if word != "--force"]
    if rest[:1] != ["--"] or len(rest) < 2:
        return None
    return rest[1:], None


def command_under_two_logs(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 two-logs.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `[--quiet] <名> -- <命令…>` 时交回 (<命令…> 的词, None)：
    当前目录与环境照外面那一条；它不摘参数里的 --force（只对 python3 做准入），所以 `--` 之后原样交回。--selftest、--from-log 与没写 `--` 的交 None。
    弄坏开关 two-logs-opaque 下一律交 None。"""
    if break_is_set("two-logs-opaque") or not words or os.path.basename(words[0]) != TWO_LOGS_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, TWO_LOGS_DIRECTORY):
        return None
    rest = words[1:]
    if rest[:1] == ["--quiet"]:
        rest = rest[1:]
    if len(rest) < 3 or rest[0].startswith("-") or rest[1] != "--":
        return None
    return rest[2:], None


def command_under_run_build_directory(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 run-build-directory.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `[--] <命令…>`（包一层起：
    这一趟一个自己的编译目录、跑完删，那份脚本文件末尾「包一层起」一段）时交回 (<命令…> 的词, None)：当前目录与环境照外面那一条。
    --selftest、没给命令、命令前写了别的选项的交 None。弄坏开关 run-build-directory-opaque 下一律交 None。"""
    if break_is_set("run-build-directory-opaque") or not words or os.path.basename(words[0]) != RUN_BUILD_DIRECTORY_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, RUN_BUILD_DIRECTORY_LOCATION):
        return None
    rest = words[1:]
    if rest[:1] == ["--"]:
        rest = rest[1:]
    elif rest[:1] and rest[0].startswith("-"):
        return None
    return (rest, None) if rest else None


def dispatched_command(words):
    """words[0] 是派发起法（DISPATCH_LAUNCHER_OPTIONS 的键）时，照它切参数的写法取 `--` 之后那条命令的词；没有命令交 None。
    `--` 之前有认不出的参数的，派发器按用法错退 2、什么都不跑，这里照严的一侧取它后面第一个 `--` 之后的那条照判。"""
    flags, arity = DISPATCH_LAUNCHER_OPTIONS[os.path.basename(words[0])]
    position = 1
    while position < len(words):
        word = words[position]
        if word == "--":
            return words[position + 1:] if position + 1 < len(words) else None
        if word in flags:
            position += 1
        elif word in arity:
            position += 1 + arity[word]
        else:
            break
    rest = words[position:]
    if "--" not in rest or rest.index("--") + 1 >= len(rest):
        return None
    return rest[rest.index("--") + 1:]


def command_under_dispatch_launcher(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 multi-host-run.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `--peer [选项…] -- <命令…>` 时
    交回 (<命令…> 的词, None)（--peer 写没写不另分：不带 --peer 它退 2、什么都不跑，照样剥开判里面那条）：当前目录与环境照外面那一条。头一个参数是 MULTI_HOST_RUN_OTHER_MODES 之一的、--selftest 与没写 `--` 的交 None。
    不并进 command_under_launcher：task-server/task_server.py 借 command_under_launcher 剥启动器；project-code-guard.sh 在 command_under_launcher、
    command_under_gpu_quiesce 之后再试它，里面那条算经内存包装（dispatched_under_memory_cap）。
    弄坏开关 dispatch-launchers-unseen 下一律交 None。"""
    if break_is_set("dispatch-launchers-unseen") or not words or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    name = os.path.basename(words[0])
    if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):
        return None
    if name == MULTI_HOST_RUN_SCRIPT and words[1:2] and words[1] in MULTI_HOST_RUN_OTHER_MODES:
        return None
    dispatched = dispatched_command(words)
    return None if dispatched is None else (dispatched, None)


def temporary_service_wrapped(words, directory=None):
    """words[0] 是 scripts/prune-server 这一层的 prune_server_client_selftest.py（名字与位置，located_in；别处的同名拷贝不认）、头一个参数是
    --with-temporary-service 时交 (<-- 之后那条原命令的词>, 给没给 --server-binary <二进制>)；别的、-- 后面没有命令的交 None（文件头 temporary_service_wrapped）。
    -- 之前有认不出的参数的，包装按用法错退 2、什么都不跑，这里照严的一侧取第一个 -- 之后那条照判（同 dispatched_command）。
    弄坏开关 temporary-service-wrapper-unseen 下一律交 None。"""
    if break_is_set("temporary-service-wrapper-unseen") or len(words) < 2 or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    if os.path.basename(words[0]) != TEMPORARY_SERVICE_WRAPPER_SCRIPT or words[1] != TEMPORARY_SERVICE_WRAPPER_MODE \
            or not located_in(words[0], directory, TEMPORARY_SERVICE_WRAPPER_DIRECTORY):
        return None
    binary_given, position = False, 2
    while position < len(words):
        if words[position] == "--":
            return (words[position + 1:], binary_given) if position + 1 < len(words) else None
        if words[position] == TEMPORARY_SERVICE_BINARY_OPTION and position + 1 < len(words) and words[position + 1] != "--":
            binary_given, position = True, position + 2
            continue
        position += 1
    return None


def command_under_temporary_service(words, directory=None):
    """临时 prune-server 的包装给了 --server-binary 时交回 (<原命令…> 的词, None)：当前目录与环境照外面那一条，包装不编、只起那份二进制与原命令；
    没给 --server-binary（包装不现编退 2，或在服务给的编译目录里现编）与别的交 None（文件头 temporary_service_wrapped）。"""
    wrapped = temporary_service_wrapped(words, directory)
    return (wrapped[0], None) if wrapped is not None and wrapped[1] else None


@functools.lru_cache(maxsize=8)
def temporary_service_table_at(path, modified, size):
    """要临时服务的自证登记表 → ({键: 判定}, [写坏的行，逐条一句])。modified、size 只当缓存的键（看门狗常驻，表改了要重读）。
    判定认不出、没有 # 理由或理由太短、键重复（留第一行）的不收；弄坏开关 temporary-service-table-lenient 下照收、不报。"""
    rows, problems = {}, []
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        return {}, [f"读不了要临时服务的自证登记表 {path}：{error}"]
    lenient = break_is_set("temporary-service-table-lenient")
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split(None, 2)
        key, judgement = fields[0], fields[1] if len(fields) > 1 else ""
        reason = fields[2][1:].strip() if len(fields) > 2 and fields[2].startswith("#") else ""
        wrong = []
        if judgement not in (TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_OWN_SERVER):
            wrong.append(f"判定「{judgement}」认不出（只认 {TEMPORARY_SERVICE_REQUIRED}、{TEMPORARY_SERVICE_OWN_SERVER}）")
        if len(reason) < SELFTEST_REASON_MINIMUM:
            wrong.append(f"# 之后的理由不到 {SELFTEST_REASON_MINIMUM} 个字")
        if key in rows:
            wrong.append("键重复")
        if wrong and not lenient:
            problems.append(f"第 {number} 行 {key}：{'；'.join(wrong)}")
            continue
        rows.setdefault(key, judgement)
    return rows, problems


def read_temporary_service_table(path):
    """要临时服务的自证登记表此刻的内容（temporary_service_table_at 按修改时刻与大小缓存）；文件不在交 ({}, [一句])。"""
    try:
        status = os.stat(path)
    except OSError as error:
        return {}, [f"读不了要临时服务的自证登记表 {path}：{error.strerror}"]
    return temporary_service_table_at(path, status.st_mtime_ns, status.st_size)


SPLIT_FAMILY_LIBRARY_PATH = os.path.join("scripts", "gate", "lib_split_family_loaded.py")


def split_family_library():
    """scripts/gate/lib_split_family_loaded.py（认「入口加它拆出的部分文件」只在那一份里写）：按这份模块所在仓的路径载入，存进 sys.modules 的路径键下，同一个进程里载入几次用同一份；
    读不到、编不过、载入时抛错的抛 ImportError，点名是哪一份。"""
    path = os.path.join(HOOK_REPOSITORY, SPLIT_FAMILY_LIBRARY_PATH)
    key = f"lib_split_family_loaded@{path}"
    library = sys.modules.get(key)
    if library is None:
        try:
            specification = importlib.util.spec_from_file_location(key, path)
            library = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(library)
        except (OSError, SyntaxError, ValueError, ImportError) as error:
            raise ImportError(f"lib_project_code_runs.py 读不到或载不进拆分族的认法 {path}（{type(error).__name__}：{error}）") from error
        sys.modules[key] = library
    return library


def source_family(path, candidates=None):
    """一份脚本连同它拆出去的同目录文件（拆分族）：[path] 加同目录里名字落在命名族里（lib_lag_split_family 认的 <模块名>_<部分>.py、<名>-<部分>.sh 与 <名>-<部分>.py）、
    又被入口或族里别的文件载入（import、from … import、同目录的相对 import、字面量参数的 importlib.import_module、赋给 *PART_NAMES 的写死文件名、shell 的 source；注释与文档串里写的不算）的文件，
    交回完整路径、不重复、入口在最前；认法只在 split_family_library 载入的那一份里写。candidates 不给取入口同目录里的 .py 与 .sh；
    门禁阶段入口的部分文件在 .claude/gate.d/lib/ 下、不在入口的同目录里，由调用方给候选。
    名字像部分文件而入口没载入的、被载入而名字不像部分文件的（几份脚本共用的库、字符串里点名的别的脚本）、别的目录的库都不在族里。
    弄坏开关 temporary-service-mark-entry-only 下族里只有入口自己，temporary-service-family-by-directory 下候选里的每一份都算族里的（不看命名与载入），
    temporary-service-family-by-name 下命名族里的都算（不看入口载没载入），temporary-service-family-by-loading 下被载入的都算（不看名字），
    temporary-service-family-by-text 下载入按文本行认（注释、文档串里写的也算，import_module 认不出），temporary-service-family-absolute-paths 下命名那一半不把绝对路径换成仓内相对路径。"""
    if break_is_set("temporary-service-mark-entry-only"):
        return [path]
    library = split_family_library()
    if candidates is None:
        candidates = library.sibling_files(path)
    if break_is_set("temporary-service-family-by-directory"):
        return [path] + [member for member in candidates if os.path.normpath(member) != os.path.normpath(path)]
    defects = set()
    if break_is_set("temporary-service-family-by-name"):
        defects.add(library.NAME_ONLY)
    if break_is_set("temporary-service-family-by-loading"):
        defects.add(library.LOADED_ONLY)
    if break_is_set("temporary-service-family-by-text"):
        defects.add(library.TEXT_SCAN)
    if break_is_set("temporary-service-family-absolute-paths"):
        defects.add(library.ABSOLUTE_PATHS)
    return library.loaded_family(path, candidates, defects)


def family_has_temporary_service_mark(path):
    """入口 path 的拆分族（source_family）里有没有哪一份调了临时服务核对（TEMPORARY_SERVICE_SOURCE_MARK，source_has_temporary_service_mark 逐份读）。"""
    return any(source_has_temporary_service_mark(member) for member in source_family(path))


def temporary_service_table_problems(path, registry_path, root):
    """要临时服务的自证登记表与自证登记表、脚本源码「两边对一遍」（表头写了两条），逐条一句；空表（读得了）是没毛病：
    写坏的行；行里的文件在 root 下不在、没登记在 registry_path 里；required 行的源码里没有 TEMPORARY_SERVICE_SOURCE_MARK；
    registry_path 里非门禁阶段的行，源码里有记号而这张表里没有这一行。
    required 行的「源码里」是入口连同它的拆分族（source_family）：记号挪进拆出的同目录文件的，照样算有；漏登那一条只看入口原文（族里的同目录文件可能是另一份登记脚本的库，
    例如被别的脚本 source 或 import 的共用库，把它们算进族会把别的脚本判成漏登）。"""
    rows, problems = read_temporary_service_table(path)
    problems = list(problems)
    registry = read_selftest_registry(registry_path)[0]

    def source_has_mark(key):
        try:
            with open(os.path.join(root, key), encoding="utf-8", errors="replace") as handle:
                return TEMPORARY_SERVICE_SOURCE_MARK in handle.read()
        except OSError:
            return None

    for key, judgement in sorted(rows.items()):
        marked = source_has_mark(key)
        if marked is None:
            problems.append(f"{key}：{root} 下没有这个文件")
        elif key not in registry:
            problems.append(f"{key}：没登记在自证登记表 {registry_path} 里（先在那里登记，再来要临时服务的表加一行）")
        elif judgement == TEMPORARY_SERVICE_REQUIRED and not marked and not family_has_temporary_service_mark(os.path.join(root, key)):
            problems.append(f"{key}：登记了 {TEMPORARY_SERVICE_REQUIRED}，源码里却没有 {TEMPORARY_SERVICE_SOURCE_MARK}（裸跑不会因为没有临时服务退 2，登记写错了？）")
    for key in sorted(registry):
        if key in rows or (os.path.dirname(key) == GATE_STAGE_DIRECTORY and not gate_top_library_key(key)):
            continue
        if source_has_mark(key):
            problems.append(f"{key}：源码里调了 {TEMPORARY_SERVICE_SOURCE_MARK}，要临时服务的自证登记表里没有这一行（{TEMPORARY_SERVICE_REQUIRED} 或 {TEMPORARY_SERVICE_OWN_SERVER}）")
    return problems


def temporary_service_command(key):
    """要临时服务的自证的整条写法（不含 task-server/task submit -- 前缀）：包装写法套着 <解释器> <脚本> --selftest（.py 用 python3，别的用 bash）。
    包装不带 --server-binary：任务服务把这条认成会编的、给编译目录，包装现编一遍 prune-server（与源码同一版）；自己编了、两台都读得到的二进制才另加 --server-binary，不写进这一条。
    弄坏开关 temporary-service-command-bare 下是裸写的样子，temporary-service-command-binary-placeholder 下带 --server-binary 与尖括号占位
    （照抄进 shell 时尖括号被当成输入重定向、命令不执行）。"""
    return wrap_in_temporary_service(f"{'python3' if key.endswith('.py') else 'bash'} {key} --selftest")


def wrap_in_temporary_service(command_text):
    """原命令（文本）套上临时服务的包装写法，不含 task-server/task submit -- 前缀；包装不带 --server-binary（理由同 temporary_service_command）。
    弄坏开关 temporary-service-command-bare 下是原命令裸写的样子，temporary-service-command-binary-placeholder 下带 --server-binary 与尖括号占位。"""
    if break_is_set("temporary-service-command-bare"):
        return command_text
    binary = f"{TEMPORARY_SERVICE_BINARY_OPTION} <prune-server 二进制> " if break_is_set("temporary-service-command-binary-placeholder") else ""
    return f"python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {TEMPORARY_SERVICE_WRAPPER_MODE} {binary}-- {command_text}"


def environment_under_temporary_service(environment):
    """包装 `--with-temporary-service … -- <原命令>` 给里面那条原命令设的环境：外面的环境加上临时服务的标记（project-code-guard.sh 剥开包装重判里面那条时用）。
    弄坏开关 temporary-service-environment-dropped 下原样交回（包装里的原命令看起来没有标记）。"""
    if break_is_set("temporary-service-environment-dropped"):
        return dict(environment or {})
    return dict(environment or {}, **{TEMPORARY_SERVICE_VARIABLE: "temporary-service"})


def temporary_service_bare_selftest(words, directory, environment=None):
    """words 是一条已经剥掉前缀与包装的命令（命令词是脚本）：它起的是要临时服务的自证——命令词解到自证登记表所在仓里那一份、要临时服务的自证登记表里判定是 required——
    写成了裸的单独一个 --selftest，环境里又没有临时服务的标记（命令前自己设了 TEMPORARY_SERVICE_VARIABLE，或包装剥开重判时 environment_under_temporary_service 补的），
    交一句说明（为什么、整条包装写法），不是交 None。这种自证裸跑退 2、什么都不判，所以执行前拒，免得白交一趟。
    不并进 registered_selftest：它也被 scripts/gate/selftest-process-guard.py 与 command_under_launcher 剥开后的重判共用，
    改了它，包装里的原命令就盯不上；project-code-guard.sh 在命令位置上单独调这一个。
    弄坏开关 temporary-service-bare-unjudged 下一律交 None；temporary-service-own-server-as-required 下 own-server 的行也算 required；
    temporary-service-marker-ignored 下不看环境里的标记。"""
    if break_is_set("temporary-service-bare-unjudged") or not words or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or list(words[1:]) != ["--selftest"]:
        return None
    key = selftest_registry_key(words[0], directory)
    if key is None or not selftest_registered_file(words[0], directory, key):
        return None
    judgement = read_temporary_service_table(lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH)[0].get(key)
    if judgement != TEMPORARY_SERVICE_REQUIRED and not (judgement == TEMPORARY_SERVICE_OWN_SERVER and break_is_set("temporary-service-own-server-as-required")):
        return None
    if (environment or {}).get(TEMPORARY_SERVICE_VARIABLE) and not break_is_set("temporary-service-marker-ignored"):
        return None
    return (f"{key} --selftest 只在临时 prune-server 底下跑（开跑先核临时服务，环境里没有 {TEMPORARY_SERVICE_VARIABLE} 就退 2、什么都不判；"
            f"登记在 .claude/hooks/{TEMPORARY_SERVICE_TABLE_NAME}）：整条写成 `task-server/task submit -- {temporary_service_command(key)}`"
            f"（整条不带 {TEMPORARY_SERVICE_BINARY_OPTION}：交给任务服务的这一条由服务认成会编的、给编译目录，包装现编一遍 prune-server；不经服务起、环境里没有 CARGO_TARGET_DIR 时包装不现编、退 2；"
            f"自己编了、两台都读得到的二进制才在 {TEMPORARY_SERVICE_WRAPPER_MODE} 后面另加 {TEMPORARY_SERVICE_BINARY_OPTION} 与它的绝对路径，写在整条之外）")


def temporary_service_bare_stage_samples(words, directory, environment=None):
    """words 是一条已经剥掉前缀与包装的命令（命令词是脚本）：它起的是本仓那一份 stage-selftest.sh、用 --item 点名了阶段目录里配了样本的几道，
    其中有源码里调了临时服务核对（TEMPORARY_SERVICE_SOURCE_MARK，每次从阶段源码现读，不另写名单）的，环境里又没有临时服务的标记（认法同 temporary_service_bare_selftest），
    交一句说明（点名哪几道、整条包装写法），不是交 None。这几道的样本档进门就先核临时服务，裸跑退 2、一份样本都不判，所以执行前拒，免得白交一趟。
    不带 --item 的（喂阶段目录里全部的样本）、--list-items、阶段目录或 --item 写在变量里判不出的、命令词不是本仓那一份的，都交 None（别处另判）。
    弄坏开关 temporary-service-stage-samples-unjudged 下一律交 None；temporary-service-stage-samples-every-stage 下点名的每一道样本档都当要临时服务（不读源码）；
    temporary-service-marker-ignored 下不看环境里的标记。"""
    if break_is_set("temporary-service-stage-samples-unjudged") or not words or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or os.path.basename(words[0]) != STAGE_SELFTEST_SCRIPT:
        return None
    parsed = stage_selftest_arguments(words[1:])
    if parsed.items is None or parsed.list_only or len(parsed.directories) > 1 or not stage_selftest_is_own_copy(words[0], directory):
        return None
    if (environment or {}).get(TEMPORARY_SERVICE_VARIABLE) and not break_is_set("temporary-service-marker-ignored"):
        return None
    gate_directory, problem = stage_selftest_directory(words[0], stage_selftest_written_directory(parsed, directory, environment) or None, directory)
    if problem or any(UNRESOLVED_WORD_CHARACTERS & set(item) for item in parsed.items):
        return None
    needing = [stage for stage in stage_selftest_fed_stages(gate_directory, parsed.items)
               if break_is_set("temporary-service-stage-samples-every-stage") or source_has_temporary_service_mark(os.path.join(gate_directory, stage))]
    if not needing:
        return None
    return (f"门禁样本档 {'、'.join(needing)}（stage-selftest.sh --item 点名的）只在临时 prune-server 底下判（阶段源码里调了临时服务核对，进样本档先核，"
            f"环境里没有 {TEMPORARY_SERVICE_VARIABLE} 就退 2、一份样本都不判）：整条写成 "
            f"`task-server/task submit -- {wrap_in_temporary_service('bash ' + shlex.join(words))}`"
            f"（整条不带 {TEMPORARY_SERVICE_BINARY_OPTION}：交给任务服务的这一条由服务认成会编的、给编译目录，包装现编一遍 prune-server；不经服务起、环境里没有 CARGO_TARGET_DIR 时包装不现编、退 2；"
            f"自己编了、两台都读得到的二进制才在 {TEMPORARY_SERVICE_WRAPPER_MODE} 后面另加 {TEMPORARY_SERVICE_BINARY_OPTION} 与它的绝对路径，写在整条之外）")


def source_has_temporary_service_mark(path):
    """这一份源码（门禁阶段、或拆分族里的一份）里有没有调临时服务核对（TEMPORARY_SERVICE_SOURCE_MARK）；读不了的当没有。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return TEMPORARY_SERVICE_SOURCE_MARK in handle.read()
    except OSError:
        return False


def dispatched_under_memory_cap(words, directory=None):
    """派发起法剥出来的那条算不算经内存包装：multi-host-run.sh --peer 在第二台经仓里的 run-with-memory-cap.sh 起（不带 --peer 退 2、什么都不跑）。
    命令词解到盘上的文件而不是本仓那一份、内容（连同它 source / import 的库）又与本仓那一份不同的不算（同 existing_copy_elsewhere 的口径）；当前目录认不出的不算；盘上没有的算（起不来，什么都不跑）。"""
    location = os.path.join(DISPATCH_LAUNCHER_DIRECTORY, os.path.basename(words[0]))
    resolved = shell_words.resolve_path(directory, os.path.expanduser(words[0]))
    if resolved is None:
        return False
    return not os.path.isfile(resolved) or family_own_copy(words[0], directory, location) or same_content_as_own(resolved, location)


def command_under_launcher(words, directory=None):
    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE，
    与 systemd-run -p / --property 的 Environment= / EnvironmentFile= 设的（assignments_of_environment_property）写在交回那条命令的最前面。
    flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。
    scripts/hosts/two-logs.sh 的 `[--quiet] <名> -- <命令…>` 照 command_under_two_logs 剥。
    scripts/hosts/run-build-directory.sh 的 `[--] <命令…>` 照 command_under_run_build_directory 剥。
    scripts/prune-server/prune_server_client_selftest.py 的 `--with-temporary-service --server-binary <二进制> -- <命令…>` 照 command_under_temporary_service 剥。
    scripts/crash/gpu-quiesce-run.sh 不在这里剥（command_under_gpu_quiesce，调用方另试）。"""
    two_logs = command_under_two_logs(words, directory)
    if two_logs is not None:
        return two_logs
    own_build_directory = command_under_run_build_directory(words, directory)
    if own_build_directory is not None:
        return own_build_directory
    temporary_service = command_under_temporary_service(words, directory)
    if temporary_service is not None:
        return temporary_service
    name = os.path.basename(words[0])
    if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return command_under_extra_launcher(words)
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


# 名单外启动器（research/prompts/shell-words-r1-main-verification.md 报的 25 个、research/prompts/shell-words-r2-main-verification.md 第二节第 11 条、research/prompts/shell-words-r3-sonnet-s2-output.md 第八节 A3、A4）：
# 包在命令外面照样起那条命令的程序，剥掉之后里面那条照判（command_under_extra_launcher）。名字 → (带值的短选项字母, 带值的长选项, 选项之后、命令之前的位置参数个数)。
# 值只能贴着写的选项（`-r目录`、`--mount=文件`）不进带值的表。弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen 下这几样都不剥（命令词当普通程序，参数里的命令不判）。
EXTRA_LAUNCHER_OPTIONS = {
    "doas": ("aCu", set(), 0),
    "numactl": ("NmCipP", {"--cpunodebind", "--membind", "--physcpubind", "--interleave", "--preferred", "--preferred-many", "--cpubind"}, 0),
    "unshare": ("SGRw", {"--setuid", "--setgid", "--root", "--wd", "--map-user", "--map-group", "--map-users", "--map-groups", "--propagation", "--setgroups"}, 0),
    "setpriv": ("", {"--ruid", "--euid", "--rgid", "--egid", "--reuid", "--regid", "--groups", "--inh-caps", "--ambient-caps", "--bounding-set", "--securebits",
                     "--pdeathsig", "--selinux-label", "--apparmor-profile", "--landlock-access", "--landlock-rule"}, 0),
    "nsenter": ("tSG", {"--target", "--setuid", "--setgid"}, 0),
    "ltrace": ("aADeFlnopsuwx", {"--align", "--output", "--indent", "--library"}, 0),
    "valgrind": ("", set(), 0),
    "faketime": ("p", {"--date-prog"}, 1),
    "unbuffer": ("", set(), 0),
    "cpulimit": ("lpe", {"--limit", "--pid", "--exe"}, 0),
    "dbus-run-session": ("", {"--config-file", "--dbus-daemon"}, 0),
    "firejail": ("", set(), 0),
    "proot": ("rbmqwki", {"--rootfs", "--bind", "--mount", "--qemu", "--pwd", "--cwd", "--kernel-release", "--change-id"}, 0),
    "chroot": ("", {"--userspec", "--groups"}, 1),
    "cgexec": ("g", set(), 0),
}
# 起的命令不是逐词交的（ssh 把后面的词用空格连起来交给远端 shell、su -c 交一段代码）或要单独切的几样，各有自己的剥法（command_under_extra_launcher）
EXTRA_LAUNCHER_SPECIAL = frozenset({"xargs", "find", "ssh", "su", "runuser", "sg", "script", "watch", "gdb", "scl", "parallel", "bwrap"})
EXTRA_LAUNCHER_NAMES = frozenset(EXTRA_LAUNCHER_OPTIONS) | EXTRA_LAUNCHER_SPECIAL
# xargs 从标准输入读进来、接在命令后面（或换掉 -I 的替换串）的词，find -exec 换掉 {} 的找到的路径，parallel 的参数：执行前算不出，用一个没展开的变量词占位，
# 调用方照「认不出」判（命令词、脚本路径、cargo 参数写成它的，project-code-guard.sh 照认不出的写法拒）
XARGS_INPUT_WORD = "$XARGS_INPUT_FROM_STANDARD_INPUT"
FIND_PATH_WORD = "$FIND_PATH_FOUND"
PARALLEL_INPUT_WORD = "$PARALLEL_ARGUMENTS"
XARGS_SHORT_OPTIONS_WITH_VALUE = "adEILnPs"          # -e、-i、-l 的值只能贴着写
XARGS_LONG_OPTIONS_WITH_VALUE = {"--arg-file", "--delimiter", "--max-args", "--max-procs", "--max-chars", "--process-slot-var"}
SSH_SHORT_OPTIONS_WITH_VALUE = "BbcDEeFIiJLlmOopQRSWw"
SCRIPT_SHORT_OPTIONS_WITH_VALUE = "cEIOBTmo"
SCRIPT_LONG_OPTIONS_WITH_VALUE = {"--command", "--echo", "--log-in", "--log-out", "--log-io", "--log-timing", "--logging-format", "--output-limit"}
SU_SHORT_OPTIONS_WITH_VALUE = "cgGswu"
SU_LONG_OPTIONS_WITH_VALUE = {"--command", "--session-command", "--group", "--supp-group", "--shell", "--whitelist-environment", "--user"}
WATCH_SHORT_OPTIONS_WITH_VALUE = "nq"
WATCH_LONG_OPTIONS_WITH_VALUE = {"--interval", "--equexit"}
PARALLEL_SHORT_OPTIONS_WITH_VALUE = "jSaIdNnEXsC"
PARALLEL_LONG_OPTIONS_WITH_VALUE = {"--jobs", "--sshlogin", "--arg-file", "--colsep", "--delimiter", "--joblog", "--results", "--max-args", "--timeout",
                                    "--tmpdir", "--workdir", "--env", "--basefile", "--return", "--transfer-file", "--retries", "--load", "--memfree"}
BWRAP_OPTION_ARITY = {option: 2 for option in ("--bind", "--bind-try", "--dev-bind", "--dev-bind-try", "--ro-bind", "--ro-bind-try", "--symlink", "--file",
                                               "--bind-data", "--ro-bind-data", "--setenv", "--chmod")}
BWRAP_OPTION_ARITY.update({option: 1 for option in ("--uid", "--gid", "--hostname", "--chdir", "--unsetenv", "--lock-file", "--sync-fd", "--remount-ro", "--exec-label",
                                                    "--file-label", "--proc", "--dev", "--tmpfs", "--mqueue", "--dir", "--seccomp", "--add-seccomp-fd", "--block-fd",
                                                    "--userns-block-fd", "--info-fd", "--json-status-fd", "--cap-add", "--cap-drop", "--perms", "--size", "--argv0",
                                                    "--userns", "--userns2", "--pidns", "--args", "--overlay-src", "--overlay", "--tmp-overlay", "--ro-overlay")})


def skipped_launcher_options(words, position, short_with_value, long_with_value):
    """从 position 起跳过一串选项（`--` 收尾也跳过），交回第一个不是选项的词的下标：短选项簇里头一个带值的字母之后贴着的是值、没有就取下一个词；
    长选项不带 `=` 而在带值表里的取下一个词。"""
    while position < len(words):
        word = words[position]
        if word == "--":
            return position + 1
        if not word.startswith("-") or word == "-":
            return position
        if word.startswith("--"):
            position += 2 if "=" not in word and word in long_with_value else 1
            continue
        consumed = 1
        for index, letter in enumerate(word[1:]):
            if letter in short_with_value:
                if index == len(word) - 2:
                    consumed = 2
                break
        position += consumed
    return position


def launcher_option_value(words, short_letter, long_name, short_with_value, long_with_value):
    """选项里 `-<字母> 值` / `-<字母>值` / `<长名>=值` / `<长名> 值` 的值（最后一个）；没有交 None。只看命令之前的选项段。"""
    found, position = None, 1
    while position < len(words):
        word = words[position]
        if word == "--" or not word.startswith("-") or word == "-":
            break
        if word.startswith("--"):
            name, equals, value = word.partition("=")
            if name == long_name:
                found = value if equals else (words[position + 1] if position + 1 < len(words) else None)
            position += 2 if not equals and name in long_with_value else 1
            continue
        consumed = 1
        for index, letter in enumerate(word[1:]):
            if letter in short_with_value:
                rest = word[index + 2:]
                value = rest or (words[position + 1] if position + 1 < len(words) else None)
                if letter == short_letter:
                    found = value
                consumed = 1 if rest else 2
                break
        position += consumed
    return found


def command_under_extra_launcher(words):
    """words[0] 是名单外启动器（EXTRA_LAUNCHER_NAMES）时交回 (它起的那条命令的词, 那条命令的当前目录或 None)；不是、或后面没有要起的命令交 None。
    逐词起的（doas、numactl、unshare、setpriv、nsenter、ltrace、valgrind、faketime、unbuffer、cpulimit、dbus-run-session、firejail、proot、chroot、cgexec、bwrap）剥掉选项与位置参数；
    xargs 交它起的命令，后面接一个占位词 XARGS_INPUT_WORD（-I / -i / --replace 的把替换串换成它），不给命令时是 echo；
    find 的 -exec / -execdir / -ok / -okdir 交那几条命令（{} 换成 FIND_PATH_WORD，几条用 ` ; ` 连成 sh -c）；
    ssh 交 `sh -c <host 之后的词用空格连起来>`；su / runuser / sg / script 的 -c 代码与 watch 的命令交 `sh -c <代码>`；runuser -u 用户交后面那条命令；
    gdb 只认 --args；scl 认 `scl enable <集合…> -- <命令>` 与末尾一段代码；parallel 交它起的命令再接 PARALLEL_INPUT_WORD（不给命令时整条是那个占位词）。"""
    if break_is_set("extra-launchers-unseen") or not words:
        return None
    name = os.path.basename(words[0])
    if name not in EXTRA_LAUNCHER_NAMES or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    if name in EXTRA_LAUNCHER_OPTIONS:
        short_with_value, long_with_value, positional = EXTRA_LAUNCHER_OPTIONS[name]
        position = skipped_launcher_options(words, 1, short_with_value, long_with_value) + positional
        working_directory = None
        if name == "unshare":
            working_directory = launcher_option_value(words, "w", "--wd", short_with_value, long_with_value)
        elif name == "proot":
            working_directory = (launcher_option_value(words, "w", "--pwd", short_with_value, long_with_value)
                                 or launcher_option_value(words, "", "--cwd", short_with_value, long_with_value))
        return (words[position:], working_directory) if position < len(words) else None
    if name == "bwrap":
        position, working_directory = 1, None
        while position < len(words) and words[position].startswith("--"):
            option = words[position]
            if option == "--":
                position += 1
                break
            if option == "--chdir" and position + 1 < len(words):
                working_directory = words[position + 1]
            position += 1 + BWRAP_OPTION_ARITY.get(option, 0)
        return (words[position:], working_directory) if position < len(words) else None
    if name == "xargs":
        position = skipped_launcher_options(words, 1, XARGS_SHORT_OPTIONS_WITH_VALUE, XARGS_LONG_OPTIONS_WITH_VALUE)
        replacement = launcher_option_value(words, "I", "", XARGS_SHORT_OPTIONS_WITH_VALUE, XARGS_LONG_OPTIONS_WITH_VALUE)
        for word in words[1:position]:
            if word.startswith("--replace"):
                replacement = word.partition("=")[2] or "{}"
            elif word.startswith("-i") and not word.startswith("--"):
                replacement = word[2:] or "{}"
        inner = words[position:] or ["echo"]
        if replacement:
            return ([word.replace(replacement, XARGS_INPUT_WORD) for word in inner], None)
        return (inner + [XARGS_INPUT_WORD], None)
    if name == "find":
        commands, position = [], 1
        while position < len(words):
            if words[position] in ("-exec", "-execdir", "-ok", "-okdir"):
                end = position + 1
                while end < len(words) and words[end] not in (";", "+"):
                    end += 1
                if end > position + 1 and end < len(words):   # 没有收尾的 `;` / `+`：find 报 missing argument to -exec，什么都不起
                    commands.append([word.replace("{}", FIND_PATH_WORD) for word in words[position + 1:end]])
                position = end
            position += 1
        if not commands:
            return None
        return (commands[0], None) if len(commands) == 1 else (["sh", "-c", " ; ".join(shlex.join(command) for command in commands)], None)
    if name == "ssh":
        position = skipped_launcher_options(words, 1, SSH_SHORT_OPTIONS_WITH_VALUE, set())
        remote = words[position + 1:]
        return (["sh", "-c", " ".join(remote)], None) if remote else None
    if name in ("su", "runuser"):
        user = launcher_option_value(words, "u", "--user", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE) if name == "runuser" else None
        if user is not None:
            position = skipped_launcher_options(words, 1, SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE)
            return (words[position:], None) if position < len(words) else None
        code = (launcher_option_value(words, "c", "--command", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE)
                or launcher_option_value(words, "", "--session-command", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE))
        return (["sh", "-c", code], None) if code else None
    if name == "sg":
        rest = words[1:]
        if rest[:1] == ["-"]:
            rest = rest[1:]
        rest = rest[1:]
        if rest[:1] == ["-c"]:
            rest = rest[1:]
        return (["sh", "-c", " ".join(rest)], None) if rest else None
    if name == "script":
        code = launcher_option_value(words, "c", "--command", SCRIPT_SHORT_OPTIONS_WITH_VALUE, SCRIPT_LONG_OPTIONS_WITH_VALUE)
        return (["sh", "-c", code], None) if code else None
    if name == "watch":
        position = skipped_launcher_options(words, 1, WATCH_SHORT_OPTIONS_WITH_VALUE, WATCH_LONG_OPTIONS_WITH_VALUE)
        return (["sh", "-c", " ".join(words[position:])], None) if position < len(words) else None
    if name == "gdb":
        for marker in ("--args", "-args"):
            if marker in words:
                inner = words[words.index(marker) + 1:]
                return (inner, None) if inner else None
        return None
    if name == "scl":
        if words[1:2] != ["enable"]:
            return None
        if "--" in words:
            inner = words[words.index("--") + 1:]
            return (inner, None) if inner else None
        return (["sh", "-c", words[-1]], None) if len(words) > 3 else None
    if name == "parallel":
        position = skipped_launcher_options(words, 1, PARALLEL_SHORT_OPTIONS_WITH_VALUE, PARALLEL_LONG_OPTIONS_WITH_VALUE)
        end = position
        while end < len(words) and words[end] not in (":::", "::::", ":::+", "::::+"):
            end += 1
        inner = words[position:end]
        if not inner:
            return ([PARALLEL_INPUT_WORD], None)
        if any("{" in word for word in inner):
            return ([re.sub(r"\{[^{}]*\}", PARALLEL_INPUT_WORD, word) for word in inner], None)
        return (inner + [PARALLEL_INPUT_WORD], None)
    return None


def launcher_names():
    """command_under_launcher 剥的全部启动器名：LAUNCHER_OPTIONS_WITH_VALUE 的键与名单外的那几个（EXTRA_LAUNCHER_NAMES；bash-command-detector.sh 的自证钉住它那一份与这里相同）。"""
    return frozenset(LAUNCHER_OPTIONS_WITH_VALUE) | EXTRA_LAUNCHER_NAMES


# 认不出起的是什么的写法（认不出的一律拒（偏严），records/2026-10-29-一百三十七节那一轮调度记录.md）：project-code-guard.sh 在命令文本这一层（不是读进去的脚本正文）判，交 UNRECOGNIZED_KIND 的 ProgramUse，
# 主 agent 与子 agent 一样拒。弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下一律交 None。
UNRECOGNIZED_KIND = "unrecognized"
UNRESOLVED_LAUNCH_MARKS = ("$", "`", "<(", ">(")
# cargo 起编出来的代码、范围由参数定的子命令：参数里有展开不了的词就认不出跑的是哪个包、哪些目标
CARGO_RUNNING_SUBCOMMANDS = frozenset({"test", "t", "run", "r", "bench", "nextest", "miri", "llvm-cov", "hack", "mutants"})
# 这几个选项的值写成变量不改跑的范围（编译目录、并行度、输出样子）
CARGO_HARMLESS_VALUE_OPTIONS = frozenset({"--target-dir", "-j", "--jobs", "--color", "--message-format"})
CARGO_GLOBAL_FLAGS_WITH_VALUE = frozenset({"--color", "--config", "-Z", "-C"})


# 脚本路径是怎么起的（lib_shell_words.CommandAtPosition.launcher）→ 认不出时说明里怎么称呼它
LAUNCHED_FILE_NAMES = {"shell": "bash / sh 起的脚本路径", "source": "source 读进来的文件", "python": "python 起的脚本路径",
                       "startup": "shell 启动文件（BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）路径"}


def word_unresolved(word):
    """词里还有展开不了的东西：变量、命令替换、反引号、进程替换（引号已去掉；占位词 XARGS_INPUT_WORD 这类同样算）。"""
    return any(mark in word for mark in UNRESOLVED_LAUNCH_MARKS)


def cargo_arguments_expanded(words, variables):
    """cargo 这一条的参数里写成 `$名字`、`${名字}` 的，用这段命令文本里前面赋过的值换掉（lib_shell_words.expand_known_variables；整个词就是一个变量引用的，值按空白切成几个词）；
    命令词不是 cargo、展开不了的照原样留（交给 unrecognized_launch 判）。例如 `F=--workspace; cargo test $F` 要按展开后的参数判。
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded 下原样交回。"""
    if not words or os.path.basename(words[0]) != "cargo" or break_is_set("cargo-arguments-unexpanded") or not variables:
        return words
    expanded = [words[0]]
    for word in words[1:]:
        value = shell_words.expand_known_variables(word, variables) if "$" in word and "`" not in word else None
        if value is None:
            expanded.append(word)
        elif re.fullmatch(r"\$(?:[A-Za-z_][A-Za-z0-9_]*|\{[A-Za-z_][A-Za-z0-9_]*\})", word):
            expanded += value.split()
        else:
            expanded.append(value)
    return expanded


# 会话自己那份可执行文件（与检出钩子 bash_command_detector_scope_and_index.py 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组，自检核两边相同）：命令词是它的交检出钩子 ⑬ 判起 claude 会话
CLAUDE_EXECUTABLE_PATH_WORDS = frozenset({"$CLAUDE_CODE_EXECPATH", "${CLAUDE_CODE_EXECPATH}"})


def unrecognized_launch(words, launcher=None):
    """命令文本里这一条（已剥掉前缀与包装、命令词里赋过字面值的变量已展开）认不认得出起的是什么：认不出交一句说明，认得出交 None。认不出的三种：
    命令词（直接执行的命令、bash / source 起的脚本，或 shell 启动时先读的文件：BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）带展开不了的变量、命令替换、位置参数、进程替换，或展开出来夹着空白（`c='cargo test'; $c` 照 bash 切词另成几个词）；
    source / . 后面的文件写成这样（`source <(…)`）；cargo 起编出来的代码的子命令（CARGO_RUNNING_SUBCOMMANDS）或它的参数（`--` 之后给测试二进制的、
    CARGO_HARMLESS_VALUE_OPTIONS 的值除外）带这样的词（`cargo $(echo test --workspace)`、`bash -c 'cargo "$@"' _ test --workspace` 里层的 `$@`）。
    命令词是 CLAUDE_EXECUTABLE_PATH_WORDS 的（会话自己那份可执行文件，检出钩子 ⑬ 认它是起 claude 会话、交那一道判）不算认不出（复跑分类报告 d137-replay-classify「四」口径；
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized 下照认不出）。"""
    if break_is_set("unrecognized-launches-allowed") or not words:
        return None
    word = words[0]
    if launcher is None and word in CLAUDE_EXECUTABLE_PATH_WORDS and not break_is_set("claude-executable-path-unrecognized"):
        return None
    if word_unresolved(word) or any(blank in word for blank in " \t\n"):
        how = LAUNCHED_FILE_NAMES.get(launcher, "命令词")
        return f"{how} `{word}` 由变量、命令替换、位置参数或进程替换拼出，执行前展开不了，认不出起的是什么"
    name = os.path.basename(word)
    if launcher is None and name in ("source", ".") and len(words) > 1 and word_unresolved(words[1]):
        return f"source 读进来的 `{words[1]}` 是命令替换、进程替换或变量拼出的，执行前认不出读进来的是什么"
    if launcher is not None or name != "cargo":
        return None
    position = 1
    while position < len(words) and words[position].startswith(("-", "+")) and not word_unresolved(words[position]):
        position += 2 if words[position] in CARGO_GLOBAL_FLAGS_WITH_VALUE else 1
    if position >= len(words):
        return None
    subcommand = words[position]
    if word_unresolved(subcommand):
        return f"cargo 的子命令 `{subcommand}` 由变量或命令替换拼出，认不出跑的是什么"
    if subcommand not in CARGO_RUNNING_SUBCOMMANDS:
        return None
    rest = words[position + 1:]
    index = 0
    while index < len(rest):
        argument = rest[index]
        if argument == "--":
            break
        if argument in CARGO_HARMLESS_VALUE_OPTIONS:
            index += 2
            continue
        if any(argument.startswith(option + "=") for option in CARGO_HARMLESS_VALUE_OPTIONS):
            index += 1
            continue
        if word_unresolved(argument):
            return (f"cargo {subcommand} 的参数 `{argument}` 由变量、命令替换或位置参数拼出，执行前展开不了，认不出跑的是哪个包、哪些目标"
                    "（--workspace、-p singlefs-verification 都可能藏在里面）")
        index += 1
    return None


# 读进来的脚本正文里，命令词或脚本路径由变量拼出而解不出时，要看它起的是不是这几样（一百三十七节件 1b 第 3 条）：自证、门禁阶段、入口脚本（按名字判的那几份与
# Verus 两份判法、广谱变异驱动、prove-red.sh）、cargo。名字按词认（前后不接字母、数字、点、连字符、下划线）
SCRIPT_BODY_ENTRY_NAMES = tuple(sorted(set(KNOWN_SCRIPT_LOCATIONS) | {BROAD_MUTATION_SCRIPT_NAME, VERUS_PROOF_CHECK_SCRIPT, E165_PIN_SCRIPT, PROVE_RED_SCRIPT}))
SCRIPT_BODY_LAUNCH_SIGNATURES = (
    ("自证", re.compile(r"(?<![\w-])--selftest(?![\w-])")),
    ("门禁阶段", re.compile(r"\.claude/gate\.d/[\w.-]+\.sh(?![\w.-])")),
    ("入口脚本", re.compile(r"(?<![\w.-])(?:" + "|".join(re.escape(name) for name in SCRIPT_BODY_ENTRY_NAMES) + r")(?![\w.-])")),
    ("cargo", re.compile(r"(?<![\w.-])cargo(?![\w.-])")),
)
POSITIONAL_REFERENCE = re.compile(r"\$(?:[@*1-9]|\{[@*1-9][^}]*\})")
SCRIPT_SELF_REFERENCES = frozenset({"$0", "${0}", "$BASH_SOURCE", "${BASH_SOURCE}", "${BASH_SOURCE[0]}"})


def script_in_own_repository(path):
    """脚本的整条路径逐级都不是符号链接、落在本仓（HOOK_REPOSITORY，自证里另认 FAMILY_OWN_REPOSITORY）里：仓里的脚本受写范围闸与提交前的门禁管，
    script_body_unresolved_launch 不判它们（仓里 multi-host-run.sh、merge-and-verify.sh 这类本来就经变量起包装与命令）。
    别处的拷贝（仓副本里没改过的那一份）与本仓同一相对位置那一份逐字节相同的同样算：跑的就是本仓那一份的代码（同 same_content_as_own 的口径）。"""
    if not path:
        return False
    repositories = [os.path.normpath(repository) for repository in (HOOK_REPOSITORY, FAMILY_OWN_REPOSITORY) if repository]
    absolute = os.path.abspath(os.path.expanduser(path))
    walked = path_without_symbolic_links(absolute, "/")
    if walked is not None and any(walked.startswith(repository + os.sep) for repository in repositories):
        return True
    parts = absolute.strip("/").split("/")
    for start in range(1, len(parts) - 1):   # 至少两段（目录/文件），不拿仓根下的单个文件名去对
        suffix = os.path.join(*parts[start:])
        for repository in repositories:
            own = os.path.join(repository, suffix)
            if os.path.isfile(own):
                try:
                    return filecmp.cmp(own, absolute, shallow=False)
                except OSError:
                    return False
    return False


def script_body_unresolved_launch(words, launcher, definitions, positional=(), script_path=None):
    """读进来的脚本正文里这一条（命令词里赋过字面值的变量已展开）：命令词、bash / sh / source / python 起的脚本路径由变量、命令替换、位置参数拼出而解不出，
    而它连同它点到的变量在这份正文里的定义（variable_definitions，往下点到的接着收，最多 UNEXPANDED_DEFINITION_DEPTH 层）、点到位置参数时连同起这份脚本的那一条的参数
    （positional），里面写着自证（--selftest）、门禁阶段（.claude/gate.d/<名>.sh）、入口脚本（SCRIPT_BODY_ENTRY_NAMES）或 cargo 的：交一句说明（照认不出起什么的写法办）；
    cargo 是字面命令词而起编出来代码的子命令或参数解不出的，照 unrecognized_launch 的 cargo 那一半交说明。别的交 None。
    边界：定义里与位置参数里都点不到这几样的（`"$@"` 转发、`$cmd` 的定义里没有它们），照旧不判；正文所在的脚本（script_path）落在本仓里的不判
    （script_in_own_repository），本仓的脚本照常起（文件头「认不出的写法」）。
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged 下一律交 None（脚本正文里经变量拼出的起法看不见）。"""
    if break_is_set("script-body-launches-unjudged") or not words:
        return None
    if words[0] in SCRIPT_SELF_REFERENCES or script_in_own_repository(script_path):
        return None   # 脚本起它自己（`bash "$0" --selftest`）：起的就是正在读的这一份，它的正文已经在判
    if not word_unresolved(words[0]):
        if launcher is None and os.path.basename(words[0]) == "cargo":
            found = unrecognized_launch(words, None)
            return f"脚本正文里 {found}" if found else None
        return None
    pool, pending, seen = list(words), set(), set()
    for word in words:
        pending.update(UNEXPANDED_VARIABLE_NAME.findall(word))
        if POSITIONAL_REFERENCE.search(word):
            pool += list(positional)
    for _ in range(UNEXPANDED_DEFINITION_DEPTH):
        reached = set()
        for name in pending - seen:
            seen.add(name)
            for value in (definitions or {}).get(name, ()):
                pool.append(value)
                reached.update(UNEXPANDED_VARIABLE_NAME.findall(value))
                if POSITIONAL_REFERENCE.search(value):
                    pool += list(positional)
        pending = reached - seen
        if not pending:
            break
    text = "\n".join(pool)
    for what, pattern in SCRIPT_BODY_LAUNCH_SIGNATURES:
        match = pattern.search(text)
        if match:
            how = LAUNCHED_FILE_NAMES.get(launcher, "命令词")
            return (f"脚本正文里这一条的{how} `{words[0]}` 由变量、命令替换或位置参数拼出，执行前解不出，而它与它点到的变量的定义"
                    f"{'、起这份脚本时给的参数' if positional and POSITIONAL_REFERENCE.search(text) else ''}里写着{what}（`{match.group(0)}`）："
                    "认不出起的是哪一份、经没经改过（改过的钩子或脚本的自证不在副本里起），照认不出起什么的写法办")
    return None


# 读脚本正文时，入口按自己所在的目录 source 的库（`source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"`、`"${BASH_SOURCE[0]%/*}/…"`、`$(cd "$(dirname "$0")" && pwd)`）：
# 把这几种写法换成脚本所在的目录（照起它的那个路径，不解符号链接：bash 的 BASH_SOURCE 就是这个路径），source 那一行就成了字面路径，照读进来判（三方 m3-close-gov-b17 第一百一十八节跟进）。
SCRIPT_DIRECTORY_FORMS = re.compile(
    r'\$\(\s*cd\s+"?\$\(\s*dirname\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)"?\s*(?:&&|;)\s*pwd(?:\s+-P)?\s*\)'
    r'|\$\(\s*dirname\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)'
    r'|\$\{(?:BASH_SOURCE(?:\[0\])?|0)%/\*\}')


def script_directory_expanded(text, script_path):
    """脚本正文 text 里按自己所在目录取路径的写法（SCRIPT_DIRECTORY_FORMS）换成 script_path 所在的目录（照起它的路径，不解符号链接）；
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread 下原样交回（这种 source 的库读正文时看不见）。"""
    if break_is_set("sourced-libraries-unread") or not script_path:
        return text
    directory = os.path.dirname(os.path.abspath(script_path))
    return SCRIPT_DIRECTORY_FORMS.sub(lambda match: directory, text)


# 读一份 shell 文件里 source 的库（shell_source_lines）：命令位置（行首、; & | ( { ! 之后、case 分支的 ) 之后、then / do / else / elif / if / while / until 之后）上的 source 与 .，
# 前面可以垫几个命令前的赋值（QUIET=1）与 builtin / command；它的参数是一个 shell 词（shell_word_at），参数是进程替换 <(…) 的整段取下来、位置交 None；
# source 后面紧跟 = 的是嵌在 shell 里的别的语言的赋值（python 的 source = …），不算
SHELL_COMMAND_POSITION = r'(?:^|[;&|({!]|\bthen\b|\bdo\b|\belse\b|\bif\b|\bwhile\b|\buntil\b)\s*'
SHELL_SOURCE_POSITION_CHARACTERS = ";&|({!"
SHELL_SOURCE_CASE_ARM_CHARACTER = ")"
SHELL_SOURCE_POSITION_KEYWORDS = r"\bthen\b|\bdo\b|\belse\b|\bif\b|\bwhile\b|\buntil\b"
SHELL_SOURCE_ELIF_KEYWORD = r"\belif\b"
SHELL_SOURCE_PREFIXES = r'(?:(?:[A-Za-z_][A-Za-z0-9_]*=(?:"[^"]*"|\'[^\']*\'|[^\s;&|()<>"\'])*|builtin|command)\s+)*'
SHELL_SOURCE_COMMAND = re.compile(rf"(?:^|[{re.escape(SHELL_SOURCE_POSITION_CHARACTERS + SHELL_SOURCE_CASE_ARM_CHARACTER)}]|{SHELL_SOURCE_POSITION_KEYWORDS}|{SHELL_SOURCE_ELIF_KEYWORD})\s*"
                                  + SHELL_SOURCE_PREFIXES + r'(?:source|\.)\s+')
# 引号里交给 eval 或子 shell（bash / sh / dash / zsh / ksh 带 -c）跑的那段代码起头就是 source / . 的：它在另一个 shell 里跑，参数里的变量与位置参数是那边的，位置交 None（查登记表）
SHELL_QUOTED_SOURCE = re.compile(r'(?:\beval|\b(?:bash|sh|dash|zsh|ksh)(?:\s+-[A-Za-z]+)*\s+-[A-Za-z]*c[A-Za-z]*)\s+(?:--\s+)?["\']\s*(?:(?:builtin|command)\s+)?(?:source|\.)\s+')
# 变量赋值（命令位置上的 <名>=、local / readonly / export / declare / typeset 后面的）与 for 的循环变量：一个名字的每一处赋值都解到同一个位置才认
SHELL_ASSIGNMENT = re.compile(SHELL_COMMAND_POSITION + r'(?:(?:local|readonly|export|declare|typeset)\s+(?:-\w+\s+)*)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)=')
SHELL_LOOP_VARIABLE = re.compile(r'\bfor\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s+in\b')
# 不缩进的 cd（顶层）：之后的相对字面路径照它换到的目录认
SHELL_TOP_LEVEL_CD = re.compile(r'^cd\s+')
# 一个位置的起头：${BASH_SOURCE[0]%/*}（这份文件所在目录）或一个变量；$( … ) 起头的另认（shell_location）
SHELL_PATH_HEAD = re.compile(r'^(?:\$\{(?P<own_trimmed>BASH_SOURCE(?:\[0\])?|0)%/\*\}|\$\{(?P<braced>[A-Za-z_][A-Za-z0-9_]*)\}|\$(?P<bare>[A-Za-z_][A-Za-z0-9_]*))')
SHELL_PATH_REST = re.compile(r'^(?:/[^$`"\'\s]*)?$')
SHELL_CHANGED_DIRECTORY = re.compile(r'^\$\(\s*cd\s+(?P<inner>.+?)\s*(?:2>\s*/dev/null\s*)?(?:&&|;)\s*pwd(?:\s+-P)?\s*\)$', re.DOTALL)
SHELL_DIRECTORY_OF = re.compile(r'^\$\(\s*dirname\s+(?:--\s+)?(?P<inner>.+?)\s*\)$', re.DOTALL)
SHELL_OWN_NAME = re.compile(r'^\$\(\s*basename\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)$')
SHELL_PLAIN_RELATIVE = re.compile(r'^[\w.][\w./-]*$')
# 把相对位置换到另一份文件所在目录时垫在下面的假根（shell_places_inside）：够深，往上的 .. 不会走出它
SHELL_RELATIVE_ANCHOR = "/" + "/".join(f"level-{depth}" for depth in range(16))
PYTHON_SIBLING_REFERENCE = re.compile(r"^\s*(?:import\s+([A-Za-z_][A-Za-z0-9_]*)|from\s+([A-Za-z_][A-Za-z0-9_]*)\s+import)|[\"']([\w.-]+\.(?:py|sh))[\"']", re.MULTILINE)
# 库再引库往下读的层数上限（入口是第 0 层）：落在这一层的库记进 libraries 与 cut、正文不再读（引它的那一份照不同认，表那一格红）；
# 本仓三张表里最深的一条是 scripts/mutation/broad-mutation.py 那一串（第 5 层），上限留出余量
SOURCED_LIBRARY_DEPTH = 8
# 弄坏开关 library-depth-limit-lowered 下表那一格核「读完没撞上限」那一项时照这个上限读（本仓那几串都撞上，那一格红）
SOURCED_LIBRARY_LOWERED_DEPTH = 2


def shell_source_command():
    """认 source 行用的正则：没设弄坏开关就是 SHELL_SOURCE_COMMAND。弄坏开关 case-arm-position-unseen 下 case 分支的 ) 之后不算命令位置，
    elif-position-unseen 下 elif 之后不算，source-prefixes-unseen 下前面垫赋值、builtin、command 的不认。"""
    case_arm, elif_keyword, prefixes = (not break_is_set("case-arm-position-unseen"), not break_is_set("elif-position-unseen"),
                                        not break_is_set("source-prefixes-unseen"))
    if case_arm and elif_keyword and prefixes:
        return SHELL_SOURCE_COMMAND
    characters = SHELL_SOURCE_POSITION_CHARACTERS + (SHELL_SOURCE_CASE_ARM_CHARACTER if case_arm else "")
    keywords = SHELL_SOURCE_POSITION_KEYWORDS + (f"|{SHELL_SOURCE_ELIF_KEYWORD}" if elif_keyword else "")
    return re.compile(rf"(?:^|[{re.escape(characters)}]|{keywords})\s*" + (SHELL_SOURCE_PREFIXES if prefixes else "") + r'(?:source|\.)\s+')


def shell_word_at(text, position):
    """text 里从 position 起的一个 shell 词（照写的样子，带引号）：引号、$( … )、${ … }、反引号里的空白与 ; & | ) < > 不断词。"""
    depth, quote, index = 0, None, position
    while index < len(text):
        character = text[index]
        if quote == "'" and depth == 0:
            if character == "'":
                quote = None
        elif character == "\\":
            index += 1
        elif character in "\"'" and depth == 0 and quote in (None, character):
            quote = None if quote else character
        elif text.startswith("$(", index) or text.startswith("${", index):
            depth += 1
            index += 1
        elif character in ")}" and depth > 0:
            depth -= 1
        elif character == "`":
            closing = text.find("`", index + 1)
            index = len(text) if closing < 0 else closing
        elif quote is None and depth == 0 and (character.isspace() or character in ";&|)<>"):
            break
        index += 1
    return text[position:index]


def shell_word_unquoted(word):
    """去掉词最外一层的引号（$( … ) 与 ${ … } 里面的照留）：拿来比写法。"""
    kept, depth, quote, index = [], 0, None, 0
    while index < len(word):
        character = word[index]
        if depth == 0 and character in "\"'" and quote in (None, character):
            quote = None if quote else character
        else:
            if quote != "'" and (word.startswith("$(", index) or word.startswith("${", index)):
                depth += 1
            elif quote != "'" and character in ")}" and depth > 0:
                depth -= 1
            kept.append(character)
        index += 1
    return "".join(kept)


def shell_substitution_end(word):
    """word 以 $( 起头时，配对的那个 ) 之后的下标；配不上交 None。"""
    depth, index = 0, 0
    while index < len(word):
        if word.startswith("$(", index):
            depth, index = depth + 1, index + 2
            continue
        if word[index] == "(":
            depth += 1
        elif word[index] == ")":
            depth -= 1
            if depth == 0:
                return index + 1
        index += 1
    return None


class ShellPlaces(NamedTuple):
    """读一份 shell 文件时认得出的位置，都相对这份文件所在的目录：变量 → 位置；顶层 cd 之后的当前目录（没 cd 过、认不出是 None）；起这一趟的入口所在的目录（$0 指它）。"""
    variables: dict
    working_directory: object
    entry_directory: str


def shell_own_directory(word, places):
    """$(dirname "${BASH_SOURCE[0]}") 这一类（SCRIPT_DIRECTORY_FORMS）指的目录：BASH_SOURCE 指这份文件，$0 指起这一趟的入口。"""
    return "." if "BASH_SOURCE" in word else places.entry_directory


def shell_location(word, places):
    """一个词（去掉了最外一层引号）解到的位置（相对这份文件所在目录），解不出交 None：起头是认得出的位置（按自己所在目录的写法 SCRIPT_DIRECTORY_FORMS、
    `$(cd <认得出的位置> && pwd)`、`$(dirname <认得出的位置>)`、认得出的变量），后面只接字面路径。"""
    if word.startswith("$("):
        end = shell_substitution_end(word)
        if end is None:
            return None
        head, rest = word[:end], word[end:]
        changed, directory_of = SHELL_CHANGED_DIRECTORY.match(head), SHELL_DIRECTORY_OF.match(head)
        if SCRIPT_DIRECTORY_FORMS.fullmatch(head):
            base = shell_own_directory(head, places)
        elif changed:
            base = shell_location(shell_word_unquoted(changed.group("inner")), places)
        elif directory_of:
            inner = shell_location(shell_word_unquoted(directory_of.group("inner")), places)
            base = None if inner is None else (os.path.dirname(inner) or ".")
        else:
            base = None
    else:
        head = SHELL_PATH_HEAD.match(word)
        if head is None:
            return None
        rest = word[head.end():]
        base = shell_own_directory(head.group(0), places) if head.group("own_trimmed") else places.variables.get(head.group("braced") or head.group("bare"))
    if base is None or not SHELL_PATH_REST.match(rest):
        return None
    return os.path.normpath(os.path.join(base, rest.lstrip("/") or "."))


def shell_assigned_location(value, places, file_name):
    """赋值右边那个词（去掉了最外一层引号）是认得出的位置（shell_location）、或这份文件自己（<认得出的目录>/$(basename "${BASH_SOURCE[0]}")）时交那个位置，认不出交 None。"""
    directory, separator, name = value.rpartition("/")
    if separator and SHELL_OWN_NAME.match(name):
        base = shell_location(directory, places)
        return None if base is None else os.path.normpath(os.path.join(base, file_name))
    return shell_location(value, places)


def shell_places_inside(places, relative):
    """places（相对引它的那份文件所在目录）换成相对 relative 那份库所在的目录：库在引它的那份文件里被 source，看得到那份文件设的变量与当前目录。"""
    anchor = os.path.join(SHELL_RELATIVE_ANCHOR, os.path.dirname(relative))

    def moved(location):
        return None if location is None else os.path.relpath(os.path.normpath(os.path.join(SHELL_RELATIVE_ANCHOR, location)), anchor)
    return ShellPlaces({name: moved(location) for name, location in places.variables.items()}, moved(places.working_directory), moved(places.entry_directory))


def shell_source_lines(text, file_name, inherited=None):
    """一份 shell 文件正文里命令位置上（shell_source_command：case 分支的 ) 之后、elif 之后、垫着命令前的赋值或 builtin / command 的都算）的每一条 source / .，
    与引号里交给 eval 或子 shell 的代码起头的那一条（SHELL_QUOTED_SOURCE；注释行不算）：([(行号, 照写的参数, 相对这份文件所在目录的位置或 None)], 读完时认得出的位置 ShellPlaces)。
    引号里那几条与参数是进程替换 <(…) 的（照写的参数是那整段）位置一律交 None（由调用方查登记表，没登记的照认不出办）。
    认得出的位置：按自己所在目录的写法接字面路径（SCRIPT_DIRECTORY_FORMS：`$(dirname "${BASH_SOURCE[0]}")/…`、`$(cd "$(dirname "$0")" && pwd)/…`、`${BASH_SOURCE[0]%/*}/…`）；
    变量接字面路径或单独一个变量，变量在这份文件里每一处赋值都解到同一个认得出的位置（`X="$(cd "$(dirname "$0")" && pwd)"`、`R="$(cd "$X/../.." && pwd)"`、
    `F="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"` 再 `D="$(dirname "$F")"`、`L="$X/lib"`；for 的循环变量不认）；
    顶层 `cd <认得出的位置>` 之后的相对字面路径。inherited 是引它的那份文件设下的（shell_places_inside 换过），这份文件里没另赋的名字照它认。认不出的位置是 None。
    弄坏开关 script-directory-variables-skipped 下变量与 cd 都不认、认不出的那几条不交回（只剩按自己所在目录的写法直接接字面路径的）；
    quoted-shell-sources-skipped 下引号里那几条不找，process-substitution-sources-skipped 下参数是 <(…) 的那几条不交回。"""
    places = inherited or ShellPlaces({}, None, ".")
    skipped = break_is_set("script-directory-variables-skipped")
    code = [(number, line) for number, line in enumerate(text.split("\n"), 1) if not line.strip().startswith("#")]
    values = {}
    for _number, line in code:
        for assignment in SHELL_ASSIGNMENT.finditer(line):
            values.setdefault(assignment.group("name"), []).append(shell_word_unquoted(shell_word_at(line, assignment.end())))
        for loop in SHELL_LOOP_VARIABLE.finditer(line):
            values.setdefault(loop.group("name"), []).append(None)
    variables = {} if skipped else {name: location for name, location in places.variables.items() if name not in values}
    for _round in range(0 if skipped else len(values) + 1):
        resolved = dict(variables)
        for name, written in values.items():
            found = {None if value is None else shell_assigned_location(value, places._replace(variables=resolved), file_name) for value in written}
            if len(found) == 1 and None not in found:
                resolved[name] = found.pop()
        if resolved == variables:
            break
        variables = resolved
    places = places._replace(variables=variables, working_directory=None if skipped else places.working_directory)
    found = []
    source_command = shell_source_command()
    quoted_sources_read = not break_is_set("quoted-shell-sources-skipped")
    process_substitutions_read = not break_is_set("process-substitution-sources-skipped")
    for number, line in code:
        change = SHELL_TOP_LEVEL_CD.match(line)
        if change and not skipped:
            places = places._replace(working_directory=shell_location(shell_word_unquoted(shell_word_at(line, change.end())), places))
        # (source / . 之后的下标, 是不是引号里交给 eval 或子 shell 的代码)：同一个下标两种都认出的算命令位置上的
        starts = {command.end(): False for command in source_command.finditer(line)}
        if quoted_sources_read:
            for quoted in SHELL_QUOTED_SOURCE.finditer(line):
                starts.setdefault(quoted.end(), True)
        for start, inside_quotes in sorted(starts.items()):
            substituted = process_substitutions_read and line.startswith("<(", start)
            if substituted:
                closing = shell_substitution_end("$" + line[start + 1:])
                written = line[start:] if closing is None else line[start:start + closing]
            else:
                written = shell_word_at(line, start)
            if not written or written.startswith("="):
                continue
            if inside_quotes or substituted:
                location = None
            else:
                word = shell_word_unquoted(written)
                location = shell_location(word, places)
                if location is None and places.working_directory is not None and SHELL_PLAIN_RELATIVE.match(word):
                    location = os.path.normpath(os.path.join(places.working_directory, word))
            if location is None and skipped:
                continue
            found.append((number, written, location))
    return found, places


class SourceReading(NamedTuple):
    """script_source_reading 交回的：libraries 是引的库 [相对入口所在目录的路径]（照写的样子，没解符号链接）；
    unrecognized 是认不出写法、又没登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里的 source 行 [(相对入口所在目录的文件, 行号, 照写的参数)]；
    registered 是用到的登记 [(本仓那一份文件（相对仓根）, 照写的参数, 那一份里第几条照写这个参数的 source 行)]；
    lines 是读到的每一条 source 行 [(相对入口所在目录的文件, 行号, 照写的参数, 认出的位置或 None, 那一份里第几条照写这个参数的 source 行)]；
    cut 是落在深度上限那一层、正文没再往下读的库 [相对入口所在目录的路径]（同族拷贝照不同认）。"""
    libraries: list
    unrecognized: list
    registered: list
    lines: list
    cut: list


def script_source_reading(path, location=None, depth_limit=None):
    """一份脚本按自己所在目录 source（shell，shell_source_lines）或 import / 点名（python：同目录下有这个 .py 的 import，字符串里写的同目录 .py / .sh）的库，
    连同那些库自己再引的（库看得到引它的那份设的变量）：交 SourceReading。入口自己与读过的库只读一遍：库再引回入口、再引读过的库不再读、不往下多占一层。
    往下读到 depth_limit 层（不给取此刻的 SOURCED_LIBRARY_DEPTH），落在那一层的库记进 cut、不再读它的正文。location 是这份脚本在本仓里的位置（相对仓根）：
    给了的，认不出写法的 source 行照 OWN_SCRIPT_UNRECOGNIZED_SOURCES 按「本仓那一份文件、照写的参数、那一份里第几条照写这个参数的 source 行」查登记，
    登记了库的照那份库认、登记空串的不算库；没给的、没登记的进 unrecognized。
    弄坏开关 unrecognized-sources-skipped 下认不出的不进 unrecognized（静默跳过）；unrecognized-sources-registered-by-argument 下查登记不看第几条
    （同一份里照写同一个参数的几条都照头一条登记认）；library-entry-reread 下入口不先记成读过（库再引回入口时入口当库再读一遍）；
    library-depth-cut-unreported 下落在上限那一层的库不进 cut（静默截断）。"""
    reading = SourceReading([], [], [], [], [])
    limit = SOURCED_LIBRARY_DEPTH if depth_limit is None else depth_limit
    seen = set() if break_is_set("library-entry-reread") else {os.path.normpath(path)}
    script_sources_into(reading, path, os.path.basename(path), location, 0, seen, None, limit)
    return reading


def registered_source_key(own_file, written, ordinal):
    """OWN_SCRIPT_UNRECOGNIZED_SOURCES 里认这一条 source 行的那个键（本仓那一份文件、照写的参数、第几条），没登记交 None。
    弄坏开关 unrecognized-sources-registered-by-argument 下不看第几条：同一份文件、同一个参数的头一个登记就算。"""
    if break_is_set("unrecognized-sources-registered-by-argument"):
        return next((key for key in OWN_SCRIPT_UNRECOGNIZED_SOURCES if key[:2] == (own_file, written)), None)
    key = (own_file, written, ordinal)
    return key if key in OWN_SCRIPT_UNRECOGNIZED_SOURCES else None


def script_sources_into(reading, path, relative, location, depth, seen, inherited, limit):
    """把 path（入口所在目录下的 relative 那一份，第 depth 层）引的库与 source 行记进 reading，再往它引的库里读（script_source_reading），读到第 limit 层为止。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read(MAXIMUM_LIBRARY_SCAN_BYTES)
    except OSError:
        return
    own_file = None if location is None else os.path.normpath(os.path.join(os.path.dirname(location), relative))
    found, places = [], None
    if path.endswith(".py"):
        for match in PYTHON_SIBLING_REFERENCE.finditer(text):
            name = match.group(1) or match.group(2)
            sibling = f"{name}.py" if name else match.group(3)
            if sibling and os.path.isfile(os.path.join(os.path.dirname(path), sibling)):
                found.append(sibling)
    else:
        lines, places = shell_source_lines(text, os.path.basename(path), inherited)
        ordinals = {}
        for number, written, located in lines:
            ordinal = ordinals[written] = ordinals.get(written, 0) + 1
            reading.lines.append((relative, number, written, located, ordinal))
            if located is None:
                key = None if own_file is None else registered_source_key(own_file, written, ordinal)
                if key is None:
                    if not break_is_set("unrecognized-sources-skipped"):
                        reading.unrecognized.append((relative, number, written))
                    continue
                reading.registered.append(key)
                library = OWN_SCRIPT_UNRECOGNIZED_SOURCES[key]
                if not library:
                    continue
                located = os.path.relpath(library, os.path.dirname(own_file) or ".")
            found.append(located)
    for inner in found:
        full = os.path.normpath(os.path.join(os.path.dirname(path), inner))
        if full in seen:
            continue
        seen.add(full)
        inner_relative = os.path.normpath(os.path.join(os.path.dirname(relative), inner))
        reading.libraries.append(inner_relative)
        if depth + 1 < limit:
            script_sources_into(reading, full, inner_relative, location, depth + 1, seen, None if places is None else shell_places_inside(places, inner), limit)
        elif not break_is_set("library-depth-cut-unreported"):
            reading.cut.append(inner_relative)


def script_sourced_libraries(path, location=None):
    """script_source_reading 的 libraries 那一项：一份脚本引的库 [相对这份脚本所在目录的路径]。"""
    return script_source_reading(path, location).libraries


MAXIMUM_LIBRARY_SCAN_BYTES = 1 << 20


def variable_definitions(text, depth=0):
    """一段命令文本里每个变量可能取到的值（照写的样子，没展开）：{名字: [值…]}。收 `名=值` 的词（命令前的、单独一条的、export / local 这类的参数）
    与 `for 名 in 词…` 的每个词，命令替换里面的同样收；同一个名字几处定义都留。切词照 lib_shell_words（引号去掉，命令替换原样留在所在的词里）。"""
    definitions = {}
    try:
        tokens = shell_words.shell_tokens(shell_words.strip_data_heredocs(text))
        simple, _targets, substitutions, _redirections = shell_words.simple_commands(tokens)
    except Exception:   # 切不动的文本：一个定义都不收
        return definitions
    for words, inner_texts in zip(simple, substitutions):
        if depth < UNEXPANDED_DEFINITION_DEPTH:
            for inner in inner_texts:
                for name, values in variable_definitions(inner, depth + 1).items():
                    definitions.setdefault(name, []).extend(values)
        if len(words) >= 3 and words[0] == "for" and words[2] == "in":
            definitions.setdefault(words[1], []).extend(words[3:])
        for word in words:
            assignment = shell_words.ASSIGNMENT.match(word)
            if assignment:
                definitions.setdefault(assignment.group(1), []).append(assignment.group(2))
    return definitions


def command_word_expansions(word, directory):
    """命令词带通配时 bash 展开成的那几份路径（相对的按 directory 解、交回仍是相对的写法），排好序；零命中、相对路径而当前目录认不出的交 []
    （bash 照字面起）。只是 `[` / `[[` 的不展开（test 的内建命令）。bash 的 [^…] 换成 python glob 认的 [!…]（弄坏开关 pattern-caret-negation-literal 下不换）。
    带 POSIX 字符类、等价类、整理符号的（BASH_BRACKET_PATTERN）交给 bash 展开（bash_pattern_expansions，次序照 bash 的），判不出交 None
    （弄坏开关 posix-brackets-by-python-glob 下照旧交给 python glob）。"""
    if word in ("[", "[["):
        return []
    pattern = os.path.expanduser(word)
    if BASH_BRACKET_PATTERN.search(pattern) and not break_is_set("posix-brackets-by-python-glob"):
        return bash_pattern_expansions(pattern, directory)
    if not break_is_set("pattern-caret-negation-literal"):
        pattern = CARET_NEGATION.sub("[!", pattern)
    if os.path.isabs(pattern):
        return sorted(glob.glob(pattern))
    if directory is None:
        return []
    return sorted(glob.glob(pattern, root_dir=directory))


def bash_pattern_expansions(pattern, directory):
    """带 POSIX 字符类、等价类、整理符号的词交给 bash 展开成的那几份（次序照 bash 的，相对的仍是相对的写法）；零命中交 []。
    不起被判的命令：词经 $1 传进去、IFS 设空，只做路径展开；不读 rc 与 BASH_ENV。展不开或判不出的（相对路径而当前目录认不出、
    bash 起不来、退出码不是 0、超过 BASH_PATTERN_TIMEOUT 秒）交 None；弄坏开关 posix-brackets-unresolved-literal 下这几种也交 []（照字面判）。"""
    unresolved = [] if break_is_set("posix-brackets-unresolved-literal") else None
    absolute = os.path.isabs(pattern)
    if not absolute and directory is None:
        return unresolved
    environment = {key: value for key, value in os.environ.items() if key not in ("BASH_ENV", "ENV")}
    try:
        finished = subprocess.run([lib_project_code_runs_base.BASH_FOR_PATTERNS, "--norc", "--noprofile", "-c", 'shopt -s nullglob; IFS=; for f in $1; do printf "%s\\0" "$f"; done', "bash", pattern],
                                  cwd="/" if absolute else directory, env=environment, capture_output=True, timeout=BASH_PATTERN_TIMEOUT)
    except (OSError, subprocess.SubprocessError):
        return unresolved
    if finished.returncode != 0:
        return unresolved
    return [os.fsdecode(item) for item in finished.stdout.split(b"\0") if item]


def pattern_expansion_use(words, directory, environment=None):
    """命令词带通配（文件头「命令词带通配」）：按 directory 展开，bash 真起的那一条（头一份当命令、其余几份接在写的参数前面）与每一份单独当命令
    （带写的参数，最多 PATTERN_EXPANSION_LIMIT 份）各判一遍；有一条判成哪一类就交它（说明里写展开成了什么），都不是交 None（调用方再照字面判）。
    展开出来的那条当一段命令文本再切一遍、剥包装（写成通配的 capped.sh、nice、/usr/bin/time 这一类也认）；弄坏开关 pattern-expansions-bare 下直接 classify。"""
    matches = command_word_expansions(words[0], directory)
    if matches is None:
        where = "当前目录认不出" if directory is None and not os.path.isabs(os.path.expanduser(words[0])) else "交给 bash 展开没展成"
        return program_use("pattern-unresolved", f"命令词 {words[0]} 带方括号里的 POSIX 字符类、等价类或整理符号，{where}，判不出 bash 起的是哪一份"
                                                "（写成展开之后的字面路径再起）")
    if not matches:
        return None
    candidates = ([[*matches, *words[1:]]] if len(matches) > 1 else []) + [[match, *words[1:]] for match in matches[:PATTERN_EXPANSION_LIMIT]]
    global pattern_expansion_depth
    for candidate in candidates:
        pattern_expansion_depth += 1
        try:
            if GLOB_CHARACTERS & set(candidate[0]) or break_is_set("pattern-expansions-bare"):
                literal = classify(candidate, directory, environment, expand_patterns=False)
                tests = [literal] if literal else []
            else:
                tests = program_uses_in_command_text(shlex.join(candidate), directory, environment)
        finally:
            pattern_expansion_depth -= 1
        if tests:
            more = f" 等 {len(matches)} 份" if len(matches) > 1 else ""
            where = "" if os.path.isabs(os.path.expanduser(words[0])) else f"按 {directory} "
            return tests[0]._replace(detail=f"{tests[0].detail}（命令词 {words[0]} {where}展开成 {candidate[0]}{more}，照展开的判）")
    return None


class BraceScanTooLong(Exception):
    """一个词里配不上的花括号太多，扫描步数超过 BRACE_SCAN_LIMIT：这个词不展开。"""


def brace_gobble(word, position, wanted, budget):
    """照 bash braces.c 的 brace_gobbler：从 position 起找最外层的 wanted（`{`、`}` 或 `,`），交下标，找不到交 None。
    `${` 算进一层、它的 `{` 不算开头；里层的 `{`、`}` 按层数配，最外层多出来的 `}` 当普通字符；
    找 `}` 时要先见过最外层的逗号或 `..`（后头紧跟的不是 `}`），之前见到的最外层 `}` 当普通字符（{}a,b} 配的是后头那个）。
    只看这几种记号（BRACE_TOKENS），每看一个扣 budget[0] 一步，扣完抛 BraceScanTooLong。"""
    level, seen = 0, wanted != "}"
    for found in BRACE_TOKENS.finditer(word, position):
        budget[0] -= 1
        if budget[0] < 0:
            raise BraceScanTooLong()
        token, at = found.group(), found.start()
        if token == "${":
            level += 1
        elif token == wanted and level == 0 and seen:
            if token == "{" and at == 0 and word[1:2] in ("}", ""):
                continue   # 打头的 `{}`（find -exec 的那种）与单独一个 `{` 照 bash 不算开头
            return at
        elif token == "{":
            level += 1
        elif token == "}":
            level -= 1 if level else 0
        elif wanted == "}" and level == 0 and (token == "," or word[at + 2:at + 3] != "}"):
            seen = True
    return None


def brace_expansion(word):
    """一个词照 bash 的花括号展开成的几份（最多 WORD_EXPANSION_LIMIT 份），没有能展开的交 [word]；展开出来的空串照 bash 去掉。
    认的写法：{a,b}（最外层至少一个逗号，各份再各自展开）与序列 {1..3}、{a..e}、{01..10..3}；前后缀照接，几处花括号照乘。
    配不上的 `{`、里面最外层没有逗号又不是序列的（{a}、{}、{1...3}）照字面，接着往后找；`${…}` 是变量，不展开、里面也不找。
    引号里的花括号不展开：切词交来的 ShellWord 带着引号信息，unquoted_braces 为假的照字面交 [word]（函数体外引号里的花括号：放，
    `'{cargo,test,--all}'` 在命令词位置照字面、不当 cargo test）；不是 ShellWord 的普通 str（没有引号信息）照展。
    弄坏开关 brace-expansion-literal 下一律交 [word]，quoted-braces-expanded 下引号里的花括号也照展。"""
    if break_is_set("brace-expansion-literal") or "{" not in word or "}" not in word or ("," not in word and ".." not in word):
        return [word]
    if isinstance(word, shell_words.ShellWord) and not word.unquoted_braces and not break_is_set("quoted-braces-expanded"):
        return [word]
    try:
        results = [result for result in brace_expansion_with_empties(word, [BRACE_SCAN_LIMIT]) if result]
    except BraceScanTooLong:
        return [word]
    return results[:WORD_EXPANSION_LIMIT]


def brace_expansion_with_empties(word, budget):
    """照 bash braces.c 的 brace_expand（空串留着，brace_expansion 再去掉）；budget 见 brace_gobble。序列交 shell_words.brace_sequence（整数补零、最多 WORD_EXPANSION_LIMIT 份），
    弄坏开关 brace-sequence-literal 下序列一律不展开。"""
    opening = brace_gobble(word, 0, "{", budget)
    if opening is None:
        return [word]
    prefix, start = word[:opening], opening + 1
    close = brace_gobble(word, start, "}", budget)
    if close is None:   # 配不上：这个 `{` 照字面，接着往后找
        return [prefix + "{" + rest for rest in brace_expansion_with_empties(word[start:], budget)][:WORD_EXPANSION_LIMIT]
    amble = word[start:close]
    if "," in amble:   # 照 bash：里面哪一层有逗号都按逗号切（只切最外层的），各份再各自展开
        middles, position = [], 0
        while True:
            comma = brace_gobble(amble, position, ",", budget)
            middles += brace_expansion_with_empties(amble[position:len(amble) if comma is None else comma], budget)
            if comma is None:
                break
            position = comma + 1
    else:
        sequence = None if break_is_set("brace-sequence-literal") else shell_words.brace_sequence(amble, "u" * len(amble), WORD_EXPANSION_LIMIT)
        middles = None if sequence is None else [text for text, _ in sequence][:WORD_EXPANSION_LIMIT]
        if middles is None:
            if close + 1 >= len(word):
                return [word]
            middles = [word[opening:close + 1]]   # 不是序列（{1...3}）：照字面，后头接着展开
    suffixes = brace_expansion_with_empties(word[close + 1:], budget)
    return [prefix + middle + suffix for middle in middles[:WORD_EXPANSION_LIMIT] for suffix in suffixes][:WORD_EXPANSION_LIMIT]


def expanded_words(words):
    """bash 对这一条命令的词做的花括号展开（每个词各展开；命令词展开成几份时，头一份是命令、其余接在参数前面）。
    参数里的通配不在这里展开：判成哪一类只看命令词和只问版本的参数写法，参数里的通配展开与否判不出差别；命令词的通配交 pattern_expansion_use。"""
    return [expanded for word in words for expanded in brace_expansion(word)] or words


word_expansion_depth = 0   # word_expansion_use 套了几层（展开出来的文件名里本身带花括号时，再展开有个头）
pattern_expansion_depth = 0   # pattern_expansion_use 套了几层：花括号、通配展开出来的那条命令里本仓的代码、编译也要认（project_code_program_use）


def project_code_program_use(words, directory, launcher, wrappers=()):
    """展开（花括号、通配）出来的那条命令是本仓的代码、会编的 cargo 或编出来的二进制：交 ProgramUse（kind project-code），不是交 None。
    字面写的由 project_code_use、starts_a_build、runs_compiled_code 在命令位置上直接认；写成 bash 才展得开的写法（`bash .claude/gate.d/x.s?`、`{cargo,test}`）
    绕过那三样，所以展开之后在这里再认一遍。名单上谁都能直接跑的、只给主 agent 的会话工具不算。展开出来的是 capped.sh、run-with-memory-cap.sh 这类
    切词剥掉的包装（wrappers，交给服务的命令不带它们）的，包装本身就是本仓的代码，同样算。弄坏开关 expanded-project-code-unjudged 下一律交 None。"""
    if break_is_set("expanded-project-code-unjudged"):
        return None
    if wrappers:
        return program_use("project-code", f"经 {os.path.basename(wrappers[0][0])} 直接起（交给服务的命令不带包装）")
    project = project_code_use(words, directory, launcher)
    if project is not None and project.allowed == "nobody":
        return program_use("project-code", f"本仓的 {project.detail}")
    compiled = runs_compiled_code(words, directory) or starts_a_build(words, directory)
    if compiled:
        return program_use("project-code", compiled)
    return None


def word_expansion_use(words, directory, environment=None):
    """花括号（文件头「入口」classify 一段）：照 bash 展开（expanded_words），与写的不同就判 bash 真起的那一条；
    命令词自己花括号展开成几份的，再把每一份单独当命令、带展开过的参数各判一遍（最多 PATTERN_EXPANSION_LIMIT 份，与命令词带通配同一个口径）。
    命令词展开过的，展开出来的那条当一段命令文本再切一遍、剥包装（花括号写在 bash、nice 这类包装上的也认；弄坏开关 brace-command-words-bare 下直接 classify）。
    有一条判成哪一类就交它（说明里写展开成了什么），都不是、或没有能展开的交 None（调用方再照字面判）。"""
    global word_expansion_depth
    expanded = expanded_words(words)
    if expanded == words or word_expansion_depth >= WORD_EXPANSION_DEPTH_LIMIT:
        return None
    alternatives = brace_expansion(words[0])
    candidates = [expanded]
    if len(alternatives) > 1:
        arguments = [braced for word in words[1:] for braced in brace_expansion(word)]
        candidates += [[alternative, *arguments] for alternative in alternatives[:PATTERN_EXPANSION_LIMIT]]
    resplit = alternatives != [words[0]] and not break_is_set("brace-command-words-bare")
    found = []   # (判出的那一类, 那一条)
    word_expansion_depth += 1
    try:
        for candidate in candidates:
            if resplit:
                tests = program_uses_in_command_text(shlex.join(candidate), directory, environment)
            else:
                literal = classify(candidate, directory, environment)
                tests = [literal] if literal else []
            found += [(test, candidate) for test in tests]
            if tests:
                break
    finally:
        word_expansion_depth -= 1
    if not found:
        return None
    test, candidate = found[0]
    shown = shlex.join(candidate)
    shown = shown if len(shown) <= 200 else shown[:200] + " …"
    how = "bash 真起的那一条" if candidate is expanded else f"命令词 {words[0]} 展开出来的一份单独当命令"
    return test._replace(detail=f"{test.detail}（照 bash 的花括号展开与参数里的通配展开，{how}是 {shown}，照展开的判）")


def classify(words, directory, environment=None, expand_patterns=True):
    """一条已经剥掉前缀与包装的命令：返回 ProgramUse 或 None。environment 是这条命令看得到的变量。
    先照 bash 做花括号展开、展开出来的那一条判成哪一类就交它（word_expansion_use）；
    命令词带通配（*、?、[）的再照 bash 展开、逐个判（pattern_expansion_use），都判不出再照字面判；expand_patterns 为假时只照字面判
    （展开出来的文件名里本身带通配字符时用，免得再展开一遍）。弄坏开关 pattern-command-words-literal 下命令词的通配不展开。
    字面判的只有一样：外部程序名单（external_program_use）里的程序，名单外的一律交 None——本仓的代码由 project_code_use 认，会编的与编出来的由
    starts_a_build、runs_compiled_code 认，认不出起什么的写法由 unrecognized_launch 认。"""
    if expand_patterns:
        expanded = word_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    if expand_patterns and GLOB_CHARACTERS & set(words[0]) and not break_is_set("pattern-command-words-literal"):
        expanded = pattern_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    return external_program_use(os.path.basename(words[0]), words[1:])


def external_program_use(name, arguments):
    """命令词（取 basename）是 EXTERNAL_PROGRAM_NAMES 或 EXTERNAL_PROGRAM_PREFIXES 里的程序：交 ProgramUse，不是、或只问版本交 None。
    弄坏开关 external-programs-unlisted 下一律交 None；herd7-version-flag-counted 下 herd7 只问版本也算。"""
    if break_is_set("external-programs-unlisted"):
        return None
    if not (name in EXTERNAL_PROGRAM_NAMES or name.startswith(EXTERNAL_PROGRAM_PREFIXES)):
        return None
    if arguments in EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS.get(name, ()) and not break_is_set("herd7-version-flag-counted"):
        return None   # 只打程序的版本号
    return program_use("external-program", f"{name}（本仓之外的程序，PATH 上直接起；跑它和跑本仓的代码一样经 task-server/task submit --）")


def program_uses_in_command_text(text, directory, environment=None):
    """一段命令文本里命令位置上的每一处外部程序用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。"""
    found = []
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        command = command._replace(words=command_word_expanded(command.words, command.variables))
        launched = command_under_launcher(command.words, command.directory) or command_under_gpu_quiesce(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += program_uses_in_command_text(shlex.join(inner_words), inner_start, command.environment)
            continue
        test = classify(command.words, command.directory, command.environment)
        if not test and (word_expansion_depth or pattern_expansion_depth):
            test = project_code_program_use(command.words, command.directory, command.launcher, command.wrappers)
        if test:
            found.append(test)
    return found


def known_script_at_location(word, directory):
    """命令词（照写的样子，或按当前目录解开之后）是 KNOWN_SCRIPT_LOCATIONS 登记的名字、路径又以它的仓内位置收尾。只看路径尾部，不核是不是本仓那一份。"""
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    return any(candidate == location or candidate.endswith("/" + location)
               for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""))


def path_without_symbolic_links(path, directory):
    """路径（相对路径接在 directory 后面）从根一级一级走下去，每一级都不是符号链接时交走到的绝对路径，否则交 None；
    相对路径而 directory 认不出（None、不是绝对路径）的也交 None。.. 取父目录，走到它时前面各级已确认不是符号链接，所以字面上的父目录就是真实的父目录。"""
    if os.path.isabs(path):
        text = path
    elif directory is not None and os.path.isabs(directory):
        text = os.path.join(directory, path)
    else:
        return None
    walked = "/"
    for part in text.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            walked = os.path.dirname(walked)
            continue
        walked = os.path.join(walked, part)
        if os.path.islink(walked):
            return None
    return walked


def judged_by_name(word, directory):
    """按名字认的仓内入口脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：起它算跑本仓的代码（project_code_use 认），里面起什么由它们自己的自证与门禁管，
    读脚本正文的一方不再读进去。盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、经符号链接起的、内容连同它 source / import 的库与本仓那一份不逐字节相同的、
    有认不出又没登记的 source 行的、库读到深度上限没读完的，same_content_as_own）为假、照读正文；
    盘上没有的照旧按路径尾部为真（bash 起不来，没有正文可读）。弄坏开关 same-family-by-path-tail 下只看路径尾部。"""
    stage = gate_stage_name(word, directory)
    if stage is not None:
        # 门禁阶段同一口径：盘上有、又不是本仓 .claude/gate.d/ 里那一份的拷贝照读正文（第一百一十八节「门禁阶段与包装脚本同一口径」）；弄坏开关 same-family-by-path-tail 下照旧只看文件名
        if break_is_set("same-family-by-path-tail"):
            return True
        resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
        if resolved is None or not os.path.isfile(resolved):
            return True
        return (family_own_copy(word, directory, os.path.join(GATE_STAGE_DIRECTORY, stage))
                or same_content_as_own(resolved, os.path.join(GATE_STAGE_DIRECTORY, stage)))
    name = os.path.basename(word)
    if KNOWN_SCRIPT_LOCATIONS.get(name) is None or not known_script_at_location(word, directory):
        return False
    # 同族的别的几份（gate.sh、mutate.sh、capped.sh、run-with-memory-cap.sh、崩溃放量另两份……，第九十八节 ②）同一口径：盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、
    # 经符号链接起的）照读正文；盘上没有的照旧按名字判（bash 起不来，没有正文可读）。弄坏开关 same-family-by-path-tail 下退回只按路径尾部认
    if not break_is_set("same-family-by-path-tail") and existing_copy_elsewhere(word, directory):
        return False
    return True


# 自证里换成样本工作区：样本里的同族脚本照「本仓那一份」认（judged_by_name 那几格测的是名字与参数，不是在哪棵树里）；平常是 None，只认 HOOK_REPOSITORY
FAMILY_OWN_REPOSITORY = None


def family_own_copy(word, directory, location):
    """命令词的整条路径逐级都不是符号链接、走到的正是本仓（HOOK_REPOSITORY，自证里另认 FAMILY_OWN_REPOSITORY）里 location 那一份（同 own_repository_script）。"""
    walked = path_without_symbolic_links(os.path.expanduser(word), directory)
    return walked is not None and any(walked == os.path.normpath(os.path.join(repository, location))
                                      for repository in (HOOK_REPOSITORY, FAMILY_OWN_REPOSITORY) if repository)


def existing_copy_elsewhere(word, directory):
    """命令词解到的文件在盘上、却不是本仓那一份（路径里有符号链接，或落在别的树里）；名字不在 KNOWN_SCRIPT_LOCATIONS 里的交 False。"""
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    return (resolved is not None and os.path.isfile(resolved) and not family_own_copy(word, directory, location)
            and not same_content_as_own(resolved, location))


def same_content_as_own(path, location):
    """别处那一份与本仓 location 那一份逐字节相同、它按自己所在目录 source / import 的库（script_source_reading）在别处那边有的也都相同：
    跑的就是本仓那一份的代码，照本仓那一份认。本仓那一份的 sha256 照 own_script_digest 从字面表里挑（表外的位置交 None，照不同认）；
    本仓那一边的库（location 所在目录接上 relative）照 own_library_digest 从几张库表里逐张挑：表外的、表里点名而本仓盘上没有的交 None，照不同认
    （别处那边有没有这份都一样）；别处那边没有的那一份库 source 不进来、不跑别的代码，不比。正文里有认不出写法、又没登记的 source 行的（script_source_reading 的 unrecognized），
    引的是哪一份判不出，照不同认、读正文；库往下读撞上深度上限、有一份没读完的（script_source_reading 的 cut），它再引的库比不到，同样照不同认。
    弄坏开关 family-copy-content-unchecked 下一律交 False。"""
    if break_is_set("family-copy-content-unchecked"):
        return False
    digest = file_sha256(path)
    if digest is None or digest != own_script_digest(location):
        return False
    reading = script_source_reading(path, location)
    if reading.unrecognized or reading.cut:
        return False
    for relative in reading.libraries:
        copy_library = os.path.normpath(os.path.join(os.path.dirname(path), relative))
        own_digest = own_library_digest(os.path.normpath(os.path.join(os.path.dirname(location), relative)))
        if own_digest is None or (os.path.lexists(copy_library) and file_sha256(copy_library) != own_digest):
            return False
    return True


def registered_selftest(word, arguments, directory):
    """命令是不是登记表里登记过的自证：是交那一行（SelftestRow），不是交 None。
    脚本、钩子、门禁目录顶层 lib- 起头的共用库只认参数正好是 ["--selftest"]，.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，不带参数或单独一个 --selftest 都算；命令词要解到登记表所在仓里那一份文件
    （selftest_registered_file）；命令词带通配的不当自证（弄坏开关 selftest-pattern-words-judged 下照判）；派发起法 `… -- <命令…>`
    之后那条里的 --selftest 是里面那条的，不是外面这一条自己的自证（task-server/task submit -- <命令…> 同理：外面这一条的参数以 submit 起头，不是单独的 --selftest）；实验装置那一层只认登记了的单独 --selftest。
    弄坏开关 selftest-extra-arguments-allowed 下带了别的参数也算单独。"""
    if SELFTEST_PATTERN_CHARACTERS & set(word) and not break_is_set("selftest-pattern-words-judged"):
        return None   # 通配写出的命令词：展开之前判不出起的是哪一份（case 的模式行切成命令时也是这样的词），不当自证
    key = selftest_registry_key(word, directory)
    if key is None:
        return None
    if command_under_dispatch_launcher([word] + list(arguments), directory) is not None:
        return None   # 派发起法 `… -- <命令…>`：--selftest 落在里面那条命令里，不是派发器自己的自证（里面那条由调用方剥开另判）
    gate_library = os.path.dirname(key) == GATE_LIBRARY_DIRECTORY
    if not gate_library and "--selftest" not in arguments:
        return None
    alone = arguments in ([], ["--selftest"]) if gate_library else arguments == ["--selftest"]
    if break_is_set("selftest-extra-arguments-allowed"):
        alone = True
    row = read_selftest_registry(lib_project_code_runs_base.SELFTEST_REGISTRY_PATH)[0].get(key)
    if row is None or not alone or not selftest_registered_file(word, directory, key):
        return None
    if device_selftest_layer(os.path.dirname(key)) and arguments != ["--selftest"]:
        return None   # 实验装置那一层只认登记了的单独 --selftest（文件头「实验装置那一层」）
    return row
