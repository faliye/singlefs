"""run-scope-guard.sh 的判法：起验证脚本的命令要带范围声明 SINGLEFS_RUN_SCOPE，种类与范围要与命令的形状对得上。

认法、要声明与不要声明的形态、拒绝说明的样子、检出记录、罩不住的写法与弄坏开关都写在 run-scope-guard.sh 的文件头，这里不另抄。
切词、剥前缀与包装用同目录的 lib_shell_words.py；剥 flock、/usr/bin/time、detached-run.sh start、gpu-quiesce-run.sh 这一类，
认门禁阶段、认 cargo 的子命令用 lib_heavy_tests.py（按文件路径导入，不另写一份）。
用法：python3 lib_run_scope.py <钩子目录> [--selftest]，由 run-scope-guard.sh 起；hook 的 JSON 从标准输入读。
"""
import importlib.util
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unicodedata
from datetime import datetime, timezone
from typing import NamedTuple

sys.dont_write_bytecode = True

HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
GUARD_SCRIPT_NAME = "run-scope-guard.sh"
DECLARATION_VARIABLE = "SINGLEFS_RUN_SCOPE"
SCOPE_KINDS = ("whole", "items", "quick", "target")
SEGMENT_KEYS = ("种类", "跑什么", "范围", "为什么")
SEGMENT_START = re.compile(r"^(种类|跑什么|范围|为什么)\s*=\s*(.*)$", re.S)
REASON_MINIMUM_CHARACTERS = 12
# 不说理由的词：为什么一段里去掉它们再数字数
REASON_FILLER_WORDS = ("确认一下", "跑一下", "跑一遍", "要跑", "验证", "看看")
COMMIT_WHOLE_REASON = "提交前整轮"
WHOLE_RANGE_WORDS = ("整轮", "整包")
KIND_DESCRIPTIONS = {"whole": "整轮（whole）", "items": "点名的几项（items）", "quick": "轻型（quick）", "target": "点名的测试目标（target）"}
LAST_RUNS_VARIABLE = "SINGLEFS_LAST_RUNS_DIRECTORY"
DEFAULT_LAST_RUNS_DIRECTORY = "~/.local/share/singlefs/last-runs"
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
MAXIMUM_SCRIPT_DEPTH = 6
MAXIMUM_SCRIPT_BYTES = 1 << 20
SCRIPT_HEAD_BYTES = 4096
TEMPLATE_COMMAND_LIMIT = 2000
# 模板照抄整条命令时先摘掉命令开头已有的那份声明（不然照抄出来是两份，bash 取后一份，等于没改）
LEADING_DECLARATION = re.compile(r"\A\s*" + DECLARATION_VARIABLE + r"=(?:'[^']*'|\"(?:[^\"\\]|\\.)*\"|\S+)[ \t]+")
# 命令位置上的词里带这些字符的不当脚本路径去读（变量、命令替换、通配、引号残片、重定向）
UNREADABLE_PATH_CHARACTERS = set("$`*?[]{}()<>|;&'\"=,")
# 门禁阶段只打清单、自证、帮助、工具版本的参数：带了就不要声明
STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--selftest", "--help", "-h", "--tool-versions"})
CRASH_REPLAY_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
CRASH_AMPLIFICATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
SELFTEST_ONLY_QUERY_ARGUMENTS = frozenset({"--selftest", "--help", "-h"})
REPLAY_QUERY_ARGUMENTS = frozenset({"--selftest", "--admission-only", "--list-items", "--help", "-h"})
# 崩溃放量的分项驱动与双机驱动带一个值的选项（照两份的用法行）
CRASH_DRIVER_OPTIONS_WITH_VALUE = frozenset({"--test-filter", "--library", "--expand-up-to", "--threads", "--memory-cap", "--peer-memory-cap", "--label",
                                             "--host-weights", "--local-states-per-second"})
CARGO_NAMED_TARGET_OPTIONS = ("--test", "--bin", "--example", "--bench")
CARGO_SINGLE_TARGET_FLAGS = ("--lib", "--doc")
CARGO_EVERY_TARGET_FLAGS = frozenset({"--tests", "--bins", "--examples", "--benches", "--all-targets"})
CARGO_WHOLE_WORKSPACE_FLAGS = frozenset({"--workspace", "--all"})
GATE_WHOLE_LOCATIONS = (".claude/singlefs-ai-sop/scripts", ".claude/scripts")

BREAK_VARIABLE_NAME = "RUN_SCOPE_GUARD_BREAK"
# 弄坏开关 → 打开后红的自证格，写全（文件头「弄坏开关」一段逐项照这张表写；开关轮核红的格与这里逐格相等，并核每个开关名都写在 run-scope-guard.sh 文件头里）
BREAK_SWITCH_CELLS = {
    "declaration-optional": ["门禁整道：没带声明拒", "崩溃点重放剪枝跑：没带声明拒", "崩溃放量全部分项：没带声明拒", "崩溃放量双机驱动：没带声明拒", "replay.sh 全部实验：没带声明拒", "gate-staged.sh 没带声明拒", "cargo test 整工作区：没带声明拒", "bash -c 里的门禁没带声明拒", "here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", "printf 经管道喂给 bash 的门禁：没带声明拒", "echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", "capped.sh 包着的 cargo test 没带声明拒", "env 带别的变量、没带声明拒", "nice 与 timeout 包着、没带声明拒", "flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "detached-run.sh start 包着的门禁没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒", "驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "detached-run.sh start 起驱动脚本、里面缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒", "入口：没带声明拒（退出码 2）", "入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", "入口：拒绝说明附上次红的清单", "入口：没有上次红的记录时照写", "入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）"],
    "segments-unchecked": ["声明缺段：没写为什么拒", "声明值空：范围一段是空的拒", "声明写两次：种类写两次拒", "声明种类认不出拒"],
    "kind-unchecked": ["门禁 --item 声明成 whole：种类对不上拒", "门禁 --quick 声明成 items：种类对不上拒", "merge-and-verify 声明成 items：种类对不上拒", "cargo test 整包声明成 target：种类对不上拒"],
    "count-unchecked": ["门禁 --check 两格写成 1 格：范围数对不上拒", "whole 范围没写整轮或整包拒", "cargo test 两个目标写成 3 个：范围数对不上拒", "cargo test --exact 两个测试写成 1 个：范围数对不上拒"],
    "reason-unchecked": ["gate.sh 为什么只写验证拒", "为什么太短拒", "为什么只写要跑验证拒"],
    "subagent-whole-allowed": ["子 agent 起崩溃点重放剪枝跑、带对的声明：whole 拒", "子 agent 起整道门禁、带对的声明：whole 拒", "入口：子 agent whole 拒、说明写交回主 agent", "入口：命令开头已有声明时模板只留一份"],
    "scripts-unread": ["驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "detached-run.sh start 起驱动脚本、里面缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒",
                       "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）"],
    "wrappers-opaque": ["flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "detached-run.sh start 包着的门禁没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒", "detached-run.sh start 起驱动脚本、里面缺声明拒"],
    "allowed-unrecorded": ["入口：放行的记进检出记录（四段与命令）"],
    "last-runs-unread": ["入口：拒绝说明附上次红的清单"],
    "query-forms-declared": ["门禁 --list 不要声明", "门禁 --selftest 不要声明", "崩溃点重放 --list-items 不要声明", "replay.sh --admission-only 不要声明", "replay.sh --list-items 不要声明", "cargo test --no-run 不要声明", "cargo test -- --list 不要声明"],
    "replay-list-items-declared": ["replay.sh --list-items 不要声明"],
    "tooling-scripts-read": ["仓里的工具脚本不读：放行"],
    "cargo-targets-as-whole": ["cargo test 两个目标、带对的声明：放", "cargo test --lib --exact 两个测试、带对的声明：放", "子 agent cargo test 点名目标、带对的声明：放", "capped.sh 外面带声明放", "驱动脚本里每一行都带声明：放"],
    "template-keeps-declaration": ["入口：命令开头已有声明时模板只留一份"],
    "failure-unrecorded": ["入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", "入口：判的时候抛异常放行、stderr 报一句、记一条检出", "入口：here-string 喂给 bash 的是变量放行、记一条检出",
                           "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）",
                           "入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", "入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置"],
    "judge-crash-unrecorded": ["入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）"],
    "driver-unread-unrecorded": ["入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）"],
    "honor-outside-selftest": ["入口：平常跑设了弄坏开关退 2、点名它只在自证里认", "入口：标记文件内容对不上照拒"],
    "honor-outside-selftest-with-others": ["入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它"],
}
# 弄坏开关只在自证里认：自证起的入口靠这个变量指到自证建的标记文件、内容逐字是 SELFTEST_MARK_CONTENT 才认开关（同一句写在 run-scope-guard.sh 那一段守卫里）
SELFTEST_MARK_VARIABLE = "RUN_SCOPE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "run-scope-guard.sh --selftest 起的子进程"

heavy_tests = None
shell_words = None


def guard_switch_on(name):
    """弄坏开关 RUN_SCOPE_GUARD_BREAK 里有没有这一项；有不认识的项抛 ValueError（免得拼错的开关被当成没开）。"""
    switches = [item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item]
    unknown = [item for item in switches if item not in BREAK_SWITCH_CELLS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE_NAME} 里有认不出的项 {'、'.join(unknown)}，只认 {'、'.join(BREAK_SWITCH_CELLS)}")
    return name in switches


def load_judging_libraries(hook_directory):
    """按文件路径导入 lib_heavy_tests.py（它再导入同目录的 lib_shell_words.py）。读不到或导入出错照抛，由调用方放行并报一句。"""
    global heavy_tests, shell_words
    if heavy_tests is None:
        spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(hook_directory, "lib_heavy_tests.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        heavy_tests, shell_words = module, module.shell_words
    return heavy_tests


class RunShape(NamedTuple):
    kind: str            # whole / items / quick / target
    what: str            # 这条命令跑的是什么（给 agent 看的一句）
    count: int | None    # items 与 target 的 N
    unit: str            # N 的单位：格、项、个实验、个测试目标、个测试
    range_hint: str      # 模板里范围一段现算好的写法
    script_key: str | None   # 上次红的清单按它找（脚本相对仓根的路径）；cargo test 没有
    family: str          # stage、crash、replay、merge、gate-whole、cargo


class FoundRun(NamedTuple):
    frames: tuple        # 读到它经过的驱动脚本：((脚本路径, 行号或 None), …)；命令文本里直接写的是 ()
    words: list          # 剥掉前缀与包装之后的那一条
    shape: RunShape
    declaration: str | None   # 这一条看得到的 SINGLEFS_RUN_SCOPE；没有是 None


class ScanState:
    """一次判定里各层共用：同一条命令里前面用 heredoc 写出的文件（实际路径 → 正文），各层判不了的、喂给 shell 的输入（每处一句）。"""
    def __init__(self):
        self.written = {}
        self.unjudged = []


def without_force(arguments):
    """各入口的 preflight 先摘掉每一个 --force，判的是摘完的参数。"""
    return [argument for argument in arguments if argument != "--force"]


def named_option_values(arguments, option):
    """`--item X` 与 `--item=X` 点名的值，去重、按出现次序。"""
    values, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == option and position + 1 < len(arguments):
            values.append(arguments[position + 1])
            position += 2
            continue
        if argument.startswith(option + "="):
            values.append(argument[len(option) + 1:])
        position += 1
    return list(dict.fromkeys(value for value in values if value))


def query_form(arguments, query_arguments):
    """只查清单、自证、帮助这类不要声明的跑法。弄坏开关 query-forms-declared 下一律当真跑。"""
    return not guard_switch_on("query-forms-declared") and any(argument in query_arguments for argument in arguments)


def counted_text(count, unit):
    return f"{count} {unit}"


def stage_run_shape(stage, arguments):
    rest = without_force(arguments)
    if query_form(rest, STAGE_QUERY_ARGUMENTS):
        return None
    key = f".claude/gate.d/{stage}"
    cells = sorted(heavy_tests.checked_cells(rest) or set())
    items = named_option_values(rest, "--item")
    if "--quick" in rest:
        named = f"，另点名 {len(items)} 项 {'、'.join(items)}" if items else ""
        return RunShape("quick", f"门禁 {stage} 的轻型（--quick）{named}", None, "", "轻型核心部分" + (f"加点名的 {len(items)} 项" if items else ""), key, "stage")
    if cells or items:
        parts = ([f"格 {'、'.join(cells)}"] if cells else []) + ([f"项 {'、'.join(items)}"] if items else [])
        unit = "格" if cells and not items else "项"
        count = len(cells) + len(items)
        return RunShape("items", f"门禁 {stage} 点名的{'，'.join(parts)}", count, unit, counted_text(count, unit), key, "stage")
    return RunShape("whole", f"门禁 {stage} 的全部格（不带 --check / --item）", None, "", "整轮", key, "stage")


def crash_run_shape(name, arguments):
    rest = without_force(arguments)
    key = f"scripts/crash/{name}"
    if name == "crash-point-replay.sh":
        if query_form(rest, CRASH_REPLAY_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if "--quick" in rest:
            return RunShape("quick", "崩溃点重放入口的轻型（--quick，每次跑全部项的核心部分）", None, "", "轻型核心部分", key, "crash")
        if items:
            how = "全量（--full）" if "--full" in rest else "逐条"
            return RunShape("items", f"崩溃点重放入口{how}点名的项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        if "--backlog" in rest:
            return RunShape("whole", "崩溃点重放入口的积压跑法（--backlog，照积压登记表取件）", None, "", "整轮", key, "crash")
        how = "全量（--full，只写不读）" if "--full" in rest else "剪枝跑全部项"
        return RunShape("whole", f"崩溃点重放入口的重型{how}", None, "", "整轮", key, "crash")
    if name == "crash-amplification.sh":
        if query_form(rest, CRASH_AMPLIFICATION_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if items:
            return RunShape("items", f"崩溃放量点名的分项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        return RunShape("whole", "崩溃放量的全部分项（不带 --item）", None, "", "整轮", key, "crash")
    if query_form(rest, SELFTEST_ONLY_QUERY_ARGUMENTS):
        return None
    target, filtered, position = None, False, 0
    while position < len(rest):
        argument = rest[position]
        if argument in CRASH_DRIVER_OPTIONS_WITH_VALUE:
            filtered = filtered or argument == "--test-filter"
            position += 2
            continue
        if not argument.startswith("-") and target is None:
            target = argument
        position += 1
    driver = "分项驱动" if name == "crash-amplification-item.sh" else "双机驱动"
    unit = "个测试" if filtered else "个测试目标"
    return RunShape("target", f"崩溃放量{driver}起测试目标 {target or '（没给）'}{'（带 --test-filter）' if filtered else ''}", 1, unit,
                    counted_text(1, unit), key, "crash")


def replay_run_shape(arguments):
    rest = without_force(arguments)
    query_arguments = REPLAY_QUERY_ARGUMENTS - {"--list-items"} if guard_switch_on("replay-list-items-declared") else REPLAY_QUERY_ARGUMENTS
    if query_form(rest, query_arguments):
        return None
    numbers = list(dict.fromkeys(argument for argument in rest if not argument.startswith("-")))
    key = "scripts/experiment-apparatus/replay.sh"
    if numbers:
        return RunShape("items", f"实验复跑点名的实验 {'、'.join(numbers)}", len(numbers), "个实验", counted_text(len(numbers), "个实验"), key, "replay")
    return RunShape("whole", "实验复跑的全部实验（不带实验号）", None, "", "整轮", key, "replay")


def cargo_run_shape(arguments, directory):
    command = heavy_tests.cargo_subcommand(arguments, directory)
    if command is None or command.subcommand not in ("test", "t"):
        return None
    rest = command.rest
    separator = rest.index("--") if "--" in rest else len(rest)
    before, after = rest[:separator], rest[separator + 1:]
    if "--no-run" in before and not guard_switch_on("query-forms-declared"):
        return None
    if heavy_tests.libtest_lists_only(after) and not guard_switch_on("query-forms-declared"):
        return None
    packages, targets, filters, every_target, whole_workspace = [], [], [], False, False
    position = 0
    while position < len(before):
        word = before[position]
        following = before[position + 1] if position + 1 < len(before) else ""
        if word in ("-p", "--package"):
            packages.append(following)
            position += 2
            continue
        if word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
        elif word.startswith("-p") and not word.startswith("--") and len(word) > 2:
            packages.append(word[2:])
        elif word in CARGO_NAMED_TARGET_OPTIONS:
            targets.append(f"{word} {following}")
            position += 2
            continue
        elif word.startswith(tuple(option + "=" for option in CARGO_NAMED_TARGET_OPTIONS)):
            targets.append(word.replace("=", " ", 1))
        elif word in CARGO_SINGLE_TARGET_FLAGS:
            targets.append(word)
        elif word in CARGO_EVERY_TARGET_FLAGS:
            every_target = True
        elif word in CARGO_WHOLE_WORKSPACE_FLAGS:
            whole_workspace = True
        elif word in heavy_tests.CARGO_TEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        elif not word.startswith("-"):
            filters.append(word)
        position += 1
    position = 0
    while position < len(after):
        word = after[position]
        if word in heavy_tests.LIBTEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if not word.startswith("-"):
            filters.append(word)
        position += 1
    packages = list(dict.fromkeys(package for package in packages if package))
    targets = list(dict.fromkeys(targets))
    filters = list(dict.fromkeys(filters))
    package_text = f"-p {'、'.join(packages)} " if packages else ""
    if whole_workspace or every_target or not targets or guard_switch_on("cargo-targets-as-whole"):
        if whole_workspace or not packages:
            return RunShape("whole", f"cargo test {package_text}不挑目标（整工作区，或当前目录那个包的全部测试目标）", None, "", "整轮", None, "cargo")
        return RunShape("whole", f"cargo test 整包 {'、'.join(packages)}（不挑目标，全部测试二进制）", None, "", "整包", None, "cargo")
    if "--exact" in after and filters:
        return RunShape("target", f"cargo test {package_text}{'、'.join(targets)} 里按全名点名的测试 {'、'.join(filters)}", len(filters), "个测试",
                        counted_text(len(filters), "个测试"), None, "cargo")
    return RunShape("target", f"cargo test {package_text}的测试目标 {'、'.join(targets)}", len(targets), "个测试目标",
                    counted_text(len(targets), "个测试目标"), None, "cargo")


def run_shape_of(words, directory):
    """剥完包装的一条命令是不是要声明的验证脚本：是就交 RunShape，不是（或是只查清单的写法）交 None。"""
    if not words or heavy_tests.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    word, arguments = words[0], words[1:]
    name = os.path.basename(word)
    stage = heavy_tests.gate_stage_name(word, directory)
    if stage is not None:
        return stage_run_shape(stage, arguments)
    if name in ("crash-point-replay.sh", "crash-amplification.sh", "crash-amplification-item.sh", "crash-amplification-two-hosts.sh") \
            and heavy_tests.located_in(word, directory, "scripts/crash"):
        return crash_run_shape(name, arguments)
    if name == "replay.sh" and heavy_tests.located_in(word, directory, "scripts/experiment-apparatus"):
        return replay_run_shape(arguments)
    if name == "merge-and-verify.sh" and heavy_tests.located_in(word, directory, "scripts/gate"):
        if query_form(without_force(arguments), SELFTEST_ONLY_QUERY_ARGUMENTS):
            return None
        return RunShape("whole", "合入后验证（打补丁、编、跑补丁改到的包与依赖它们的包的测试与 fmt、clippy、build）", None, "", "整轮",
                        "scripts/gate/merge-and-verify.sh", "merge")
    if name == "gate-staged.sh" and heavy_tests.located_in(word, directory, "scripts/gate"):
        if query_form(without_force(arguments), SELFTEST_ONLY_QUERY_ARGUMENTS):
            return None
        return RunShape("whole", "提交前的整轮门禁（gate-staged.sh 起整轮 gate.sh --staged）", None, "", "整轮", "scripts/gate/gate-staged.sh", "gate-whole")
    if name == "gate.sh" and any(heavy_tests.located_in(word, directory, location) for location in GATE_WHOLE_LOCATIONS):
        return RunShape("whole", "整轮门禁 gate.sh", None, "", "整轮", ".claude/singlefs-ai-sop/scripts/gate.sh", "gate-whole")
    if name == "cargo":
        return cargo_run_shape(arguments, directory)
    return None


class ScopeDeclaration(NamedTuple):
    segments: dict
    problems: list


def parse_scope_declaration(text):
    """SINGLEFS_RUN_SCOPE 的值按 ; 切成四段；值里自带的 ; 接到上一段后面。"""
    segments, problems, current = {}, [], None
    for raw in text.split(";"):
        part = raw.strip()
        found = SEGMENT_START.match(part)
        if found:
            key, value = found.group(1), found.group(2).strip()
            if key in segments:
                problems.append(f"「{key}」一段写了两次")
            segments[key] = value
            current = key
        elif current is not None:
            if part:
                segments[current] = f"{segments[current]}; {part}"
        elif part:
            problems.append(f"认不出「{part[:40]}」：每段写成 键=值，键只认 种类、跑什么、范围、为什么")
    for key in SEGMENT_KEYS:
        if key not in segments:
            problems.append(f"缺「{key}」一段")
        elif not segments[key]:
            problems.append(f"「{key}」一段是空的")
    kind = segments.get("种类")
    if kind and kind not in SCOPE_KINDS:
        problems.append(f"种类「{kind}」认不出，只认 whole、items、quick、target")
    return ScopeDeclaration(segments, problems)


def reason_character_count(reason):
    """为什么一段的字数：去掉不说理由的词，再只数字母与数字（汉字算字母），空白与标点不算。"""
    for filler in REASON_FILLER_WORDS:
        reason = reason.replace(filler, "")
    return sum(1 for character in reason if unicodedata.category(character)[0] in ("L", "N"))


def scope_problems(shape, declaration_text, agent_type):
    """这一条要声明的命令哪里不合：交回一串说明，空的是放行。"""
    problems = []
    if agent_type and shape.kind == "whole" and not guard_switch_on("subagent-whole-allowed"):
        problems.append(f"子 agent 不跑整轮：这一条是{shape.what}。交回主 agent——在交回报告「要主 agent 跑的命令」一节写明这条命令、四段声明与为什么要整轮；"
                        "自己只跑点名的格、项或测试目标")
    if declaration_text is None:
        if not guard_switch_on("declaration-optional"):
            problems.append("没带范围声明")
        return problems
    declaration = parse_scope_declaration(declaration_text)
    if not guard_switch_on("segments-unchecked"):
        problems += declaration.problems
    segments = declaration.segments
    declared_kind = segments.get("种类")
    if declared_kind in SCOPE_KINDS and declared_kind != shape.kind:
        if not guard_switch_on("kind-unchecked"):
            problems.append(f"声明的是{KIND_DESCRIPTIONS[declared_kind]}，命令跑的是{KIND_DESCRIPTIONS[shape.kind]}：{shape.what}")
    range_text = segments.get("范围", "")
    if range_text and declared_kind == shape.kind and not guard_switch_on("count-unchecked"):
        if shape.kind in ("items", "target"):
            numbers = re.findall(r"\d+", range_text)
            if not numbers or int(numbers[0]) != shape.count:
                problems.append(f"范围写的是「{range_text}」，命令点名的是 {shape.count} {shape.unit}：范围一段开头写这个数（阿拉伯数字）")
        elif shape.kind == "whole" and not any(word in range_text for word in WHOLE_RANGE_WORDS):
            problems.append(f"范围写的是「{range_text}」，命令跑的是整轮：whole 的范围写「整轮」或「整包」")
    reason = segments.get("为什么", "")
    if reason and not guard_switch_on("reason-unchecked") and not (shape.family == "gate-whole" and COMMIT_WHOLE_REASON in reason):
        counted = reason_character_count(reason)
        if counted < REASON_MINIMUM_CHARACTERS:
            problems.append(f"为什么一段只有 {counted} 个字（去掉空白、标点与「要跑」「验证」这类不说理由的词之后）：至少 {REASON_MINIMUM_CHARACTERS} 个字，"
                            "写为什么这么跑、这个范围为什么合适")
    return problems


def repository_tooling_script(real_path):
    """仓里的工具脚本：落在某一层带 .claude/hooks/ 的目录下的 scripts/ 或 .claude/ 里（主工作区与仓副本都算）。"""
    current = os.path.dirname(real_path)
    while True:
        parent = os.path.dirname(current)
        if parent == current:
            return False
        if os.path.basename(current) in ("scripts", ".claude") and os.path.isdir(os.path.join(parent, ".claude", "hooks")):
            return True
        current = parent


def remember_heredoc_writes(command, state):
    """`cat > 文件 <<EOF`、`cat <<EOF > 文件`、`tee [-a] 文件 <<EOF`：把 heredoc 正文记成那个文件此刻的内容（追加的接在后面）。"""
    if not command.standard_input_heredocs or not command.words:
        return
    body = command.standard_input_heredocs[-1].text
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    targets = []
    if name == "cat" and not arguments:
        targets = [(target, operator in (">>", "&>>")) for operator, target in command.output_redirections if operator in (">", ">|", ">>", "&>", "&>>")]
    elif name == "tee":
        appending = any(argument in ("-a", "--append") for argument in arguments)
        targets = [(argument, appending) for argument in arguments if not argument.startswith("-")]
    for target, appending in targets:
        if UNREADABLE_PATH_CHARACTERS & set(target):
            continue
        resolved = shell_words.resolve_path(command.directory, os.path.expanduser(target))
        if resolved is None:
            continue
        real = os.path.realpath(resolved)
        earlier = ""
        if appending:
            earlier = state.written.get(real)
            if earlier is None:
                try:
                    with open(real, encoding="utf-8") as handle:
                        earlier = handle.read()
                except (OSError, UnicodeDecodeError):
                    earlier = ""
        state.written[real] = earlier + body


def interpreter_of_head(head):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。"""
    if not head.startswith(b"#!"):
        return None
    first = head[2:].split(b"\n", 1)[0].decode("utf-8", "replace").split()
    if not first:
        return ""
    program = os.path.basename(first[0])
    if program == "env":
        rest = [word for word in first[1:] if not word.startswith("-") and "=" not in word]
        return os.path.basename(rest[0]) if rest else ""
    return program


def driver_script_text(command, state):
    """命令位置上执行的是不是要读进去的驱动脚本：是就交 (实际路径, 正文)，不是或读不到交 None。"""
    word = command.words[0]
    if command.launcher == "python" or word.startswith("-") or UNREADABLE_PATH_CHARACTERS & set(word):
        return None
    expanded = os.path.expanduser(word)
    if command.launcher is None and "/" not in expanded:
        return None
    resolved = shell_words.resolve_path(command.directory, expanded)
    if resolved is None:
        return None
    real = os.path.realpath(resolved)
    if real in state.written:
        data = state.written[real].encode("utf-8")
    else:
        if not os.path.isfile(real):
            return None
        if repository_tooling_script(real) and not guard_switch_on("tooling-scripts-read"):
            return None
        try:
            with open(real, "rb") as handle:
                data = handle.read(MAXIMUM_SCRIPT_BYTES + 1)
        except OSError as error:   # 在而读不了：里面要声明的命令判不了，记一处（弄坏开关 driver-unread-unrecorded 下静默跳过，改前的样子）
            if not guard_switch_on("driver-unread-unrecorded"):
                state.unjudged.append(f"命令位置上执行的脚本 {real} 读不了（{type(error).__name__}: {error.strerror or error}），里面要声明的命令没判")
            return None
    if b"\0" in data[:SCRIPT_HEAD_BYTES]:
        return None
    if len(data) > MAXIMUM_SCRIPT_BYTES:   # 太大：是 shell 脚本的照判不了记一处（同上）
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        shell_script = (command.launcher is not None or interpreter in shell_words.SHELL_NAMES
                        or (interpreter is None and (real.endswith(".sh") or resolved.endswith(".sh"))))
        if shell_script and not guard_switch_on("driver-unread-unrecorded"):
            state.unjudged.append(f"命令位置上执行的脚本 {real} 比 {MAXIMUM_SCRIPT_BYTES} 字节大，没读进去，里面要声明的命令没判")
        return None
    if command.launcher is None:
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        if interpreter is None and not real.endswith(".sh") and not resolved.endswith(".sh"):
            return None
        if interpreter is not None and interpreter not in shell_words.SHELL_NAMES:
            return None
    if b"\0" in data:
        return None
    try:
        return real, data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def script_command_line_numbers(script_text):
    """脚本里每条命令（剥完前缀与包装的词序列）出现在哪几行：逐个逻辑行（反斜杠续行接上）单独切，喂给非 shell 命令的 heredoc 正文换成空行、行号不变。"""
    numbered = {}
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    index = 0
    while index < len(lines):
        start_line, joined = index + 1, lines[index]
        while joined.endswith("\\") and index + 1 < len(lines):
            index += 1
            joined = joined + "\n" + lines[index]
        index += 1
        try:
            line_commands = shell_words.commands_at_command_position(joined).commands
        except Exception:
            continue
        for line_command in line_commands:
            numbered.setdefault(tuple(line_command.words), []).append(start_line)
    return numbered


def scope_runs_in_text(text, directory, environment=None, frames=(), script=None, state=None):
    """text 里命令位置上每一条要声明的命令；包装剥掉往里判，执行的驱动脚本读进去递归判。"""
    if state is None:
        state = ScanState()
    found, seen, numbered = [], {}, None
    scan = shell_words.commands_at_command_position(text, directory, environment)
    # 喂给从标准输入读命令的 shell、判不了的输入（lib_shell_words.py 的 shell_standard_input）：照常判字面的那一部分，这一处记进检出
    state.unjudged += [f"脚本 {script} 里{entry}" if script else entry for entry in scan.unjudged_shell_inputs]
    for command in scan.commands:
        key = tuple(command.words)
        occurrence = seen.get(key, 0)
        seen[key] = occurrence + 1
        if script is None:
            here = frames
        else:
            if numbered is None:
                numbered = script_command_line_numbers(text)
            line_numbers = numbered.get(key, [])
            here = frames + ((script, line_numbers[occurrence] if occurrence < len(line_numbers) else None),)
        remember_heredoc_writes(command, state)
        launched = None
        if not guard_switch_on("wrappers-opaque"):
            launched = (heavy_tests.command_under_launcher(command.words, command.directory)
                        or heavy_tests.command_under_gpu_quiesce(command.words, command.directory))
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += scope_runs_in_text(shlex.join(inner_words), inner_start, command.environment, here, None, state)
            continue
        shape = run_shape_of(command.words, command.directory)
        if shape is not None:
            found.append(FoundRun(here, list(command.words), shape, command.environment.get(DECLARATION_VARIABLE)))
            continue
        if guard_switch_on("scripts-unread") or len(here) >= MAXIMUM_SCRIPT_DEPTH:
            continue
        driver = driver_script_text(command, state)
        if driver is None:
            continue
        real, body = driver
        if real in {os.path.realpath(path) for path, _line in here if path}:
            continue
        inner_environment = {name: value for name, value in command.environment.items() if name != DECLARATION_VARIABLE}
        found += scope_runs_in_text(body, command.directory, inner_environment, here, real, state)
    return found


def last_runs_path(script_key):
    directory = os.environ.get(LAST_RUNS_VARIABLE) or os.path.expanduser(DEFAULT_LAST_RUNS_DIRECTORY)
    return os.path.join(directory, script_key.replace("/", "__") + ".tsv")


def last_red_lines(shape):
    """上次这一道红的清单（给拒绝说明用的几行）。"""
    if shape.script_key is None:
        return ["上次红的：cargo test 没有清单文件，从上次的日志取：python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包>"]
    path = last_runs_path(shape.script_key)
    if guard_switch_on("last-runs-unread"):
        return [f"上次红的：没有上次红的记录（{path}）"]
    try:
        with open(path, encoding="utf-8") as handle:
            names = [line.strip() for line in handle if line.strip() and not line.strip().startswith("#")]
    except (OSError, UnicodeDecodeError):
        return [f"上次红的：没有上次红的记录（{path} 不在或读不了；各道跑完写这份文件归下一批）"]
    if not names:
        return [f"上次红的：没有上次红的记录（{path} 是空的）"]
    return [f"上次红的（{path}，{len(names)} 项）：{'、'.join(names)}"]


def template_line(found, command_text):
    shape = found.shape
    declaration = f"{DECLARATION_VARIABLE}='种类={shape.kind}; 跑什么={shape.what}; 范围={shape.range_hint}; 为什么=<为什么这么跑、这个范围为什么合适>'"
    if found.frames:
        return f"  在 {frames_description(found.frames[-1:])} 那一行的命令前面加（已有的那份换掉）：{declaration}"
    bare = command_text if guard_switch_on("template-keeps-declaration") else LEADING_DECLARATION.sub("", command_text, count=1)
    if DECLARATION_VARIABLE in bare:
        return f"  把命令里已有的那份 {DECLARATION_VARIABLE} 换成：{declaration}"
    if len(bare) > TEMPLATE_COMMAND_LIMIT:
        return f"  这条命令 {len(bare)} 字，不整条贴；在起它的那一条前面加：{declaration}"
    return f"  {declaration} {bare}"


NARROWING_LINES = (
    "→ 先想范围合不合适：红了只跑红的，改了只点名改到的——",
    "  门禁：bash .claude/gate.d/<阶段>.sh --check <格>[,<格>…]（格名用 --list 列）；有逐项的门禁写 --item <项>（--list-items 列）",
    "  崩溃入口：bash scripts/crash/crash-point-replay.sh --item <项>（--list-items 列）；崩溃放量 scripts/crash/crash-amplification.sh --item <分项>",
    "  cargo test：cargo test -p <包> --test <目标> -- <测试名> --exact；上次红的用 python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包> 打出只重跑它们的命令",
    "  merge-and-verify：挑包在脚本里做（只跑补丁改到的包与依赖它们的包，--all-packages 才全跑），种类照旧写 whole；为什么写明这一批补丁碰了哪些包",
)
DECLARATION_RULE_LINES = (
    f"→ 声明写在命令开头：{DECLARATION_VARIABLE}='种类=…; 跑什么=…; 范围=…; 为什么=…'，四段用 ; 隔、各段 键=值；"
    "种类 whole（不带 --item / --check 的整轮、cargo test 整包或整工作区、merge-and-verify、整轮门禁）、items（带 --item / --check、replay.sh 带实验号）、"
    "quick（带 --quick）、target（cargo test 带 --test / --bin / --lib 目标、崩溃放量的分项与双机驱动）；"
    "范围：items 写「N 格 / N 项」、target 写「N 个测试目标」（带 --exact 筛选词的写「N 个测试」），N 是阿拉伯数字、等于点名的个数，whole 写「整轮」或「整包」；"
    f"为什么至少 {REASON_MINIMUM_CHARACTERS} 个字，只写「要跑」「验证」不算；整轮门禁的为什么写「提交前整轮」就放。驱动脚本里每一条验证命令那一行各自带声明。",
)


def frames_description(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)


def refusal_text(refused, command_text):
    lines = [f"✗ 范围声明不合，这条命令没跑（{GUARD_SCRIPT_NAME}）："]
    for number, (found, problems) in enumerate(refused, start=1):
        where = f"脚本 {frames_description(found.frames)} 里的 " if found.frames else ""
        words = shlex.join(found.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     跑的是：{found.shape.what}；范围：{found.shape.range_hint}")
        lines += [f"     ✗ {problem}" for problem in problems]
        lines += [f"     {line}" for line in last_red_lines(found.shape)]
    lines.append("→ 照抄声明模板，把为什么填上（写为什么这么跑、这个范围为什么合适）：")
    lines += [template_line(found, command_text) for found, _problems in refused]
    lines += list(NARROWING_LINES) + list(DECLARATION_RULE_LINES)
    return "\n".join(lines)


class ScopeDecision(NamedTuple):
    code: int            # 0 放行，2 拒绝
    message: str | None  # 拒绝时写给 stderr 的说明
    allowed: list        # 放行的每一条（FoundRun）
    refused: list        # 被拒的每一条：(FoundRun, [说明])
    unjudged: list = []  # 判不了的、喂给 shell 的输入（每处一句）：照常判别的，记进检出


def decide_scope(hook_input, project_root):
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = ScanState()
    runs = scope_runs_in_text(command_text, directory, state=state)
    refused, allowed = [], []
    for found in runs:
        problems = scope_problems(found.shape, found.declaration, agent_type)
        (refused if problems else allowed).append((found, problems) if problems else found)
    if refused:
        return ScopeDecision(2, refusal_text(refused, command_text), allowed, refused, state.unjudged)
    return ScopeDecision(0, None, allowed, [], state.unjudged)


def detections_path_now(hook_directory):
    configured = os.environ.get(DETECTIONS_VARIABLE)
    if configured:
        return configured
    try:
        sys.path.insert(0, hook_directory)
        import lib_work_root
        return lib_work_root.default_detections_path()
    except Exception:
        return None


def append_detection(path, entry):
    if not path:
        return
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def record_scope_decision(hook_input, decision, path):
    """放行的每一条记一行（会话号写在 scope_session_id，看门狗不当检出叫醒）；被拒的整条命令记一行（session_id、outcome 被拒，看门狗只记不叫醒）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = (tool_input.get("command") or "")[:2000]
    common = {"time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME, "agent_id": hook_input.get("agent_id"),
              "agent_type": hook_input.get("agent_type") or "主 agent", "transcript_path": hook_input.get("transcript_path"), "command": command_text}
    if not guard_switch_on("allowed-unrecorded"):
        for found in decision.allowed:
            declaration = parse_scope_declaration(found.declaration or "")
            entry = dict(common, scope_session_id=hook_input.get("session_id"), outcome="放行", run_scope={
                "where": frames_description(found.frames) if found.frames else "命令文本", "words": shlex.join(found.words)[:500],
                "kind_of_command": found.shape.kind, "what": found.shape.what, "declaration": found.declaration, "segments": declaration.segments})
            append_detection(path, entry)
    findings = [f"范围声明被拒：{found.shape.what}：{'；'.join(problems)}" for found, problems in decision.refused]
    if decision.unjudged and not failure_recording_dropped():   # 判不了的、喂给 shell 的输入：跟着这一条命令记（放行的写「只记」）
        findings += [f"范围声明闸判不了：{entry}" for entry in decision.unjudged]
    if findings:
        append_detection(path, dict(common, session_id=hook_input.get("session_id"), findings=findings, outcome="被拒" if decision.refused else "只记"))


HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节


def failure_recording_dropped():
    """弄坏开关 failure-unrecorded（判不了的只打 stderr、不记检出）设没设。不核别的项认不认得：判的时候抛异常那一支也读它，
    而认不出的项正是让 guard_switch_on 抛异常的那一种（自证拿它当注入）。"""
    return "failure-unrecorded" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")


def note_unjudged(warning, hook_input, hook_directory):
    """判不了、照常执行的一条：stderr 打一行，检出记录记一行（结局「只记」；hook 输入读不出来时会话号与 agent 写「读不到」）。"""
    print(f"  ! {warning}", file=sys.stderr)
    if failure_recording_dropped():
        return
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) and isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    known = hook_input if isinstance(hook_input, dict) else {}
    append_detection(detections_path_now(hook_directory), {
        "time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME,
        "session_id": known.get("session_id", "读不到"), "agent_id": known.get("agent_id", "读不到"),
        "agent_type": (known.get("agent_type") or "主 agent") if known else "读不到", "transcript_path": known.get("transcript_path", "读不到"),
        "command": (command if isinstance(command, str) else repr(command))[:2000], "findings": [warning], "outcome": "只记"})


def hook_main(hook_directory):
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw)
        unreadable = None if isinstance(hook_input, dict) else f"解出来是 {type(hook_input).__name__}，不是对象"
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        hook_input, unreadable = None, f"解析出错：{error!r}"
    if unreadable:   # 照常执行，打一行可定位的诊断（钩子名、开头的字节）、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象（{unreadable}），这条命令没判、照常执行；"
                      f"开头 {len(raw[:HOOK_INPUT_HEAD_BYTES])} 字节（共 {len(raw)} 字节）：{raw[:HOOK_INPUT_HEAD_BYTES]!r}", None, hook_directory)
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    # 粗筛：这几样一个都没有的命令起不了验证脚本（cargo、shell 与 .sh、路径、source / .）
    if not any(marker in command_text for marker in ("cargo", "sh", "/", "source", ". ")):
        return 0
    try:
        load_judging_libraries(hook_directory)
        project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_directory))
        decision = decide_scope(hook_input, project_root)
        record_scope_decision(hook_input, decision, detections_path_now(hook_directory))
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），这条命令照常执行", hook_input, hook_directory)
        return 0
    if decision.code:
        print(decision.message, file=sys.stderr)
    return decision.code


# ── 自证 ──────────────────────────────────────────────

def scope_text(kind, what, range_text, reason="改了这一道的判法，只跑改到的这几格看它转绿"):
    return f"{DECLARATION_VARIABLE}='种类={kind}; 跑什么={what}; 范围={range_text}; 为什么={reason}'"


def build_selftest_tree(work):
    """临时目录里的驱动脚本与一份仓的样子（带 .claude/hooks/ 的目录下的 scripts/ 是仓里的工具脚本）。"""
    def write(relative, content, executable=False):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(content)
        if executable:
            os.chmod(path, 0o755)
        return path
    declared_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮", "提交之前补跑这一道，看全仓文字的格是不是都绿")
    write("drivers/missing.sh", "#!/usr/bin/env bash\necho 开跑\nbash .claude/gate.d/doc-text.sh .\n"
                                f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-harness --test model_contract\n")
    write("drivers/declared.sh", f"#!/usr/bin/env bash\n{declared_stage} bash .claude/gate.d/doc-text.sh .\n"
                                 f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-harness --test model_contract\n")
    write("drivers/direct.sh", "#!/usr/bin/env bash\ncargo test --workspace\n", executable=True)
    write("drivers/outer.sh", "#!/usr/bin/env bash\nbash drivers/missing.sh\n")
    write("repo/.claude/hooks/placeholder", "仓的样子：这一层带 .claude/hooks/\n")
    write("repo/scripts/tool.sh", "#!/usr/bin/env bash\ncargo test --workspace\n")


def selftest_cells(work):
    """(说明, agent_type（None 是主 agent）, 命令, 该退 0 / 2, 当前目录)。"""
    sub = "implementation-writer"
    whole_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮")
    two_cells = "bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    quick_mutation = "bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick --item row-a ."
    cells = [
        # 门禁阶段
        ("门禁整道：没带声明拒", None, "bash .claude/gate.d/doc-text.sh .", 2),
        ("门禁整道：带对的声明放", None, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("门禁 --check 两格写成 2 格：放", None, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("门禁 --check 两格写成 1 格：范围数对不上拒", None, f"{scope_text('items', '门禁 code-tooling 两格', '1 格')} {two_cells}", 2),
        ("门禁 --item 声明成 items：放", None, f"{scope_text('items', '形式化点名 proof', '1 项')} bash .claude/gate.d/checker-tier-formal-verification.sh --item proof .", 0),
        ("门禁 --item 声明成 whole：种类对不上拒", None, f"{scope_text('whole', '形式化', '整轮')} bash .claude/gate.d/checker-tier-formal-verification.sh --item proof .", 2),
        ("门禁 --quick 声明成 quick：放", sub, f"{scope_text('quick', 'crates 变异轻型', '轻型核心部分加 1 项')} {quick_mutation}", 0),
        ("门禁 --quick 声明成 items：种类对不上拒", sub, f"{scope_text('items', 'crates 变异轻型', '1 项')} {quick_mutation}", 2),
        ("门禁 --list 不要声明", None, "bash .claude/gate.d/code-tooling.sh --list", 0),
        ("门禁 --selftest 不要声明", sub, "bash .claude/gate.d/code-tooling.sh --selftest", 0),
        ("whole 范围没写整轮或整包拒", None, f"{scope_text('whole', '门禁 doc-text', '全部')} bash .claude/gate.d/doc-text.sh .", 2),
        # 崩溃入口
        ("崩溃点重放剪枝跑：没带声明拒", None, "bash scripts/crash/crash-point-replay.sh", 2),
        ("崩溃点重放两项：带对的声明放", None, f"{scope_text('items', '崩溃点重放两项', '2 项')} bash scripts/crash/crash-point-replay.sh --item a --item b", 0),
        ("崩溃点重放 --list-items 不要声明", sub, "bash scripts/crash/crash-point-replay.sh --list-items", 0),
        ("子 agent 起崩溃点重放剪枝跑、带对的声明：whole 拒", sub, f"{scope_text('whole', '崩溃点重放', '整轮')} bash scripts/crash/crash-point-replay.sh", 2),
        ("子 agent 起崩溃点重放轻型、带对的声明：放", sub,
         f"{scope_text('quick', '崩溃点重放轻型', '轻型核心部分')} bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/crash/crash-point-replay.sh --quick", 0),
        ("崩溃放量一个分项：带对的声明放", None, f"{scope_text('items', '崩溃放量一个分项', '1 项')} bash scripts/crash/crash-amplification.sh --item tree-split-streams", 0),
        ("崩溃放量全部分项：没带声明拒", None, "bash scripts/crash/crash-amplification.sh", 2),
        ("崩溃放量分项驱动：带对的声明放", None,
         f"{scope_text('target', '分项驱动一个目标', '1 个测试目标')} bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script --threads 8", 0),
        ("崩溃放量双机驱动：没带声明拒", None, "bash scripts/crash/crash-amplification-two-hosts.sh crash_amplification_fixed_script", 2),
        # 实验复跑、合入后验证、整轮门禁
        ("replay.sh 两个实验：带对的声明放", None, f"{scope_text('items', '复跑 E142 与 E165', '2 个实验')} bash scripts/experiment-apparatus/replay.sh E142 E165", 0),
        ("replay.sh 全部实验：没带声明拒", None, "bash scripts/experiment-apparatus/replay.sh", 2),
        ("replay.sh --admission-only 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --admission-only E142", 0),
        ("replay.sh --list-items 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --list-items", 0),
        ("merge-and-verify 声明成 whole：放", None,
         f"{scope_text('whole', '合入后验证', '整轮', '这一批三份补丁碰了 harness 与 core 两个包，脚本只挑这两个包与依赖它们的包')} bash scripts/gate/merge-and-verify.sh p1 p2", 0),
        ("merge-and-verify 声明成 items：种类对不上拒", None, f"{scope_text('items', '合入后验证', '2 项')} bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("gate-staged.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=提交前门禁; 范围=整轮; 为什么=提交前整轮' SINGLEFS_HEAVY_TESTS=commit bash scripts/gate/gate-staged.sh", 0),
        ("gate-staged.sh 没带声明拒", None, "SINGLEFS_HEAVY_TESTS=commit bash scripts/gate/gate-staged.sh", 2),
        ("gate.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=提交前整轮' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 0),
        ("gate.sh 为什么只写验证拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=验证' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 2),
        # cargo test
        ("cargo test 整工作区：没带声明拒", None, "cargo test --workspace", 2),
        ("cargo test 整工作区：带对的声明放", None, f"{scope_text('whole', 'cargo test 整工作区', '整轮', '提交之前整工作区跑一遍，确认全部测试二进制都绿')} cargo test --workspace", 0),
        ("cargo test 两个目标、带对的声明：放", None,
         f"{scope_text('target', 'harness 两个目标', '2 个测试目标')} cargo test -p singlefs-harness --test model_contract --test scenarios", 0),
        ("cargo test 两个目标写成 3 个：范围数对不上拒", None,
         f"{scope_text('target', 'harness 两个目标', '3 个测试目标')} cargo test -p singlefs-harness --test model_contract --test scenarios", 2),
        ("cargo test --lib --exact 两个测试、带对的声明：放", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '2 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 0),
        ("cargo test --exact 两个测试写成 1 个：范围数对不上拒", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '1 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 2),
        ("cargo test 整包声明成 target：种类对不上拒", None, f"{scope_text('target', 'harness', '1 个测试目标')} cargo test -p singlefs-harness", 2),
        ("子 agent cargo test 点名目标、带对的声明：放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test model_contract", 0),
        ("cargo test --no-run 不要声明", sub, "cargo test -p singlefs-harness --no-run", 0),
        ("cargo test -- --list 不要声明", sub, "cargo test -p singlefs-harness --test model_contract -- --list", 0),
        ("cargo build 不要声明", sub, "cargo build --offline --all-targets", 0),
        # 包装
        ("bash -c 里的门禁没带声明拒", None, "bash -c 'bash .claude/gate.d/doc-text.sh .'", 2),
        ("bash -c 外面带声明放", None, f"{whole_stage} bash -c 'bash .claude/gate.d/doc-text.sh .'", 0),
        # here-string 与管道喂给从标准输入读命令的 shell 的字面文本当命令判（lib_shell_words.py 的 shell_standard_input；LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下这三格红）
        ("here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", None, "bash <<< 'cargo test --workspace'", 2),
        ("printf 经管道喂给 bash 的门禁：没带声明拒", None, "printf 'bash .claude/gate.d/doc-text.sh .\\n' | bash", 2),
        ("echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", None, "echo 'cargo test --workspace' | sh", 2),
        ("here-string 外面带声明放", None, f"{whole_stage} bash <<< 'bash .claude/gate.d/doc-text.sh .'", 0),
        ("capped.sh 包着的 cargo test 没带声明拒", sub, "bash scripts/hosts/capped.sh 31 cargo test -p singlefs-harness --test model_contract", 2),
        ("capped.sh 外面带声明放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/capped.sh 31 cargo test -p singlefs-harness --test model_contract", 0),
        ("env 带别的变量、没带声明拒", None, "env FOO=1 bash .claude/gate.d/doc-text.sh .", 2),
        ("env 的参数里带声明放", None, f"env {whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("nice 与 timeout 包着、没带声明拒", None, "nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 2),
        ("nice 与 timeout 外面带声明放", None, f"{whole_stage} nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 0),
        ("同一段命令前面 export 声明放", None, f"export {whole_stage}; bash .claude/gate.d/doc-text.sh .", 0),
        ("flock 包着的门禁没带声明拒", None, "flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 2),
        ("flock 外面带声明放", None, f"{whole_stage} flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 0),
        ("gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", None, "bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --quick", 2),
        ("gpu-quiesce-run.sh 外面带声明放", None,
         f"{scope_text('quick', '崩溃点重放轻型', '轻型核心部分')} bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --quick", 0),
        ("detached-run.sh start 包着的门禁没带声明拒", None,
         "bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label doc-text -- bash .claude/gate.d/doc-text.sh .", 2),
        ("detached-run.sh start 外面带声明放", None,
         f"{whole_stage} bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label doc-text -- bash .claude/gate.d/doc-text.sh .", 0),
        ("/usr/bin/time 包着的 cargo test 没带声明拒", None, "/usr/bin/time -v cargo test --workspace", 2),
        # 驱动脚本
        ("驱动脚本里一行缺声明拒", None, "bash drivers/missing.sh", 2),
        ("驱动脚本里每一行都带声明：放", None, "bash drivers/declared.sh", 0),
        ("外面带声明、驱动脚本里一行缺声明拒", None, f"{whole_stage} bash drivers/missing.sh", 2),
        ("驱动脚本再起驱动脚本、里面缺声明拒", None, "bash drivers/outer.sh", 2),
        ("直接执行的驱动脚本缺声明拒", None, "./drivers/direct.sh", 2),
        ("detached-run.sh start 起驱动脚本、里面缺声明拒", None,
         "bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label verify -- bash drivers/missing.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本缺声明拒", None, "cat > drivers/written.sh <<'EOF'\ncargo test --workspace\nEOF\nbash drivers/written.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本带声明：放", None,
         f"cat > drivers/written-ok.sh <<'EOF'\n{whole_stage} bash .claude/gate.d/doc-text.sh .\nEOF\nbash drivers/written-ok.sh", 0),
        ("仓里的工具脚本不读：放行", None, "bash repo/scripts/tool.sh", 0),
        # 声明的写法
        ("声明缺段：没写为什么拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=整轮' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明值空：范围一段是空的拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明写两次：种类写两次拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 种类=whole; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明种类认不出拒", None,
         f"{DECLARATION_VARIABLE}='种类=all; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么太短拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '改了文档')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么只写要跑验证拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '要跑验证要跑验证要跑验证要跑验证')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么里自带分号接到上一段：放", None,
         f"{scope_text('whole', '门禁 doc-text', '整轮', '提交之前补跑; 看全仓文字的格是不是都绿')} bash .claude/gate.d/doc-text.sh .", 0),
        # 子 agent 与别的命令
        ("子 agent 起整道门禁、带对的声明：whole 拒", sub, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 2),
        ("子 agent 起门禁两格、带对的声明：放", sub, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("不是验证脚本的命令：放", sub, "ls -la && bash .claude/hooks/run-scope-guard.sh --selftest", 0),
    ]
    return cells


def selftest_judge_cells(work):
    """在内存里逐格判；交回 [(说明, 期望, 实际)]。"""
    results = []
    for label, agent_type, command, want in selftest_cells(work):
        hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        try:
            got = decide_scope(hook_input, work).code
        except Exception as error:
            got = f"抛了 {error!r}"
        results.append((label, want, got))
    return results


def selftest_entry_cells(work, guard_script):
    """另起入口（run-scope-guard.sh）喂 hook 的 JSON：退出码、stderr 的说明、检出记录与上次红的清单。交回 [(说明, 期望, 实际)]。"""
    results = []
    detections = os.path.join(work, "detections.jsonl")
    last_runs = os.path.join(work, "last-runs")
    os.makedirs(last_runs, exist_ok=True)
    with open(os.path.join(last_runs, ".claude__gate.d__doc-text.sh.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# 上次这一道红的\nclock-times\nprime-marks\n")
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, SINGLEFS_LAST_RUNS_DIRECTORY=last_runs, CLAUDE_PROJECT_DIR=work)

    def feed(command, agent_type=None, session="s-run-scope"):
        hook_input = {"tool_name": "Bash", "cwd": work, "session_id": session, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        return subprocess.run(["bash", guard_script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=120)

    refused = feed("bash .claude/gate.d/doc-text.sh .")
    results.append(("入口：没带声明拒（退出码 2）", 2, refused.returncode))
    template = f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text.sh 的全部格（不带 --check / --item）; 范围=整轮; 为什么=<"
    results.append(("入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", True,
                    all(text in refused.stderr for text in ("跑的是：门禁 doc-text.sh 的全部格", template, "--check <格>", "--exact", "rerun-failed-tests.py",
                                                             "--all-packages 才全跑"))))
    results.append(("入口：拒绝说明附上次红的清单", True, "clock-times、prime-marks" in refused.stderr))
    no_record = feed("bash .claude/gate.d/doc-experiments.sh .")
    results.append(("入口：没有上次红的记录时照写", True, "没有上次红的记录" in no_record.stderr))
    sub_whole = feed(f"{scope_text('whole', '门禁 doc-text 全部格', '整轮')} bash .claude/gate.d/doc-text.sh .", "implementation-writer")
    results.append(("入口：子 agent whole 拒、说明写交回主 agent", (2, True), (sub_whole.returncode, "交回主 agent" in sub_whole.stderr)))
    template_lines = [line for line in sub_whole.stderr.splitlines() if line.startswith(f"  {DECLARATION_VARIABLE}=")]
    results.append(("入口：命令开头已有声明时模板只留一份", [1], [line.count(f"{DECLARATION_VARIABLE}=") for line in template_lines]))
    if os.path.exists(detections):
        os.remove(detections)
    allowed_command = f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    allowed = feed(allowed_command)
    entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            entries = [json.loads(line) for line in handle if line.strip()]
    recorded = [entry for entry in entries if entry.get("outcome") == "放行"]
    good_record = (len(recorded) == 1 and "session_id" not in recorded[0] and recorded[0].get("scope_session_id") == "s-run-scope"
                   and recorded[0]["run_scope"]["segments"].get("范围") == "2 格" and recorded[0]["run_scope"]["kind_of_command"] == "items"
                   and recorded[0]["command"] == allowed_command)
    results.append(("入口：放行的记进检出记录（四段与命令）", (0, True), (allowed.returncode, good_record)))
    feed("bash .claude/gate.d/doc-text.sh .")
    refused_entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            refused_entries = [json.loads(line) for line in handle if line.strip() and json.loads(line).get("outcome") == "被拒"]
    results.append(("入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）", True,
                    len(refused_entries) == 1 and refused_entries[0].get("session_id") == "s-run-scope"
                    and refused_entries[0]["findings"][0].startswith("范围声明被拒")))
    plain = feed("ls -la")
    results.append(("入口：别的命令放行、不写 stderr", (0, ""), (plain.returncode, plain.stderr)))
    # 判不了的照常放行、记一条检出（结局「只记」）：hook 输入不是 JSON 对象（stderr 打带钩子名与开头字节的诊断，会话号写读不到）、
    # 判的时候抛异常（RUN_SCOPE_GUARD_BREAK 里加一个认不出的项，guard_switch_on 抛 ValueError）、here-string 喂给 bash 的是变量。
    # RUN_SCOPE_GUARD_BREAK=failure-unrecorded 下这三格红，LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下第三格红
    def recorded_entries():
        if not os.path.exists(detections):
            return []
        with open(detections, encoding="utf-8") as handle:
            return [json.loads(line) for line in handle if line.strip()]
    before = len(recorded_entries())
    unreadable = [subprocess.run(["bash", guard_script], input=raw, capture_output=True, env=environment, timeout=120)
                  for raw in (b'{"tool_name": "Bash", "tool_input": ', b'["Bash"]')]
    added = recorded_entries()[before:]
    results.append(("入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", ([0, 0], True, 2, ["读不到", "读不到"]),
                    ([run.returncode for run in unreadable],
                     all(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象" in run.stderr.decode("utf-8", "replace") for run in unreadable)
                     and "b'[\"Bash\"]'" in unreadable[1].stderr.decode("utf-8", "replace"),
                     len(added), [entry.get("session_id") for entry in added])))
    before = len(recorded_entries())
    # 关守卫项 honor-outside-selftest 与别的项同写会被入口的守卫先拒（第一百二十二节 ②），注入认不出的项时摘掉它
    injected = ",".join([item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item and item != "honor-outside-selftest"] + ["not-a-switch"])
    raising = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise",
                             "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}), capture_output=True, text=True,
                             env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    added = recorded_entries()[before:]
    results.append(("入口：判的时候抛异常放行、stderr 报一句、记一条检出", (0, True, 1, "s-raise", "只记"),
                    (raising.returncode, "没判成" in raising.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None)))
    before = len(recorded_entries())
    fed_variable = feed('bash <<< "$CMD"', session="s-fed")
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-fed"]
    results.append(("入口：here-string 喂给 bash 的是变量放行、记一条检出", (0, 1, True, "只记"),
                    (fed_variable.returncode, len(added), bool(added) and any("here-string `$CMD`" in finding for finding in added[0].get("findings", [])),
                     added[0].get("outcome") if added else None)))
    # 判法文件读不到：照常放行、报一句、记一条检出（钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉）
    # 只拷入口与 lib_work_root.py（入口记检出时取默认位置用）；它 source 的 .claude/scripts/preflight.sh 与那一份找的规范副本 .claude/singlefs-ai-sop/ 链接到钩子所在的仓。
    #   入口自己记：AGENT_HOOK_DETECTIONS 设了记那里，没设记进 lib_work_root.py 的默认位置（这里把 SINGLEFS_WORK_ROOT 指进临时目录）。failure-unrecorded 下记检出的两格红
    lonely = os.path.join(work, "lonely", ".claude", "hooks")
    os.makedirs(lonely, exist_ok=True)
    shutil.copy(guard_script, os.path.join(lonely, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(lonely, "lib_work_root.py"))   # 入口记检出时取默认位置
    claude_directory = os.path.dirname(os.path.dirname(os.path.abspath(guard_script)))
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "lonely", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    orphan_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-orphan", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    before = len(recorded_entries())
    orphan = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件读不到时放行并报一句", (0, True), (orphan.returncode, "照常执行" in orphan.stderr)))
    results.append(("入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", (1, "s-orphan", "只记", GUARD_SCRIPT_NAME, True),
                    (len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None,
                     added[0].get("hook") if added else None, bool(added) and "读不到判法" in " ".join(added[0].get("findings") or []))))
    default_detections = os.path.join(work, "orphan-work-root", "agent-hook-detections.jsonl")

    def default_entries():
        try:
            with open(default_detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []
    before = len(default_entries())
    without_variable = {key: value for key, value in environment.items() if key not in (DETECTIONS_VARIABLE, "SINGLEFS_WORK_ROOT_BREAK")}
    orphan_default = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, timeout=60,
                                    env=dict(without_variable, SINGLEFS_WORK_ROOT=os.path.dirname(default_detections)))
    added = default_entries()[before:]
    results.append(("入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置", (0, 1, "s-orphan", "只记"),
                    (orphan_default.returncode, len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None)))
    # 判法文件在而没跑完（这里放一份导入就抛异常的 lib_run_scope.py）：入口照常放行、stderr 报一句、自己记一条检出（会话号照输入）；
    #   judge-crash-unrecorded 下这一格红（只打 stderr、不记）
    crashing = os.path.join(work, "crashing", ".claude", "hooks")
    os.makedirs(crashing, exist_ok=True)
    shutil.copy(guard_script, os.path.join(crashing, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(crashing, "lib_work_root.py"))
    with open(os.path.join(crashing, "lib_run_scope.py"), "w", encoding="utf-8") as handle:
        handle.write("raise RuntimeError('自证造的导入错')\n")
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "crashing", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    before = len(recorded_entries())
    crashed = subprocess.run(["bash", os.path.join(crashing, GUARD_SCRIPT_NAME)],
                             input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-crash", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}),
                             capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）", (0, True, 1, "s-crash", "只记", True),
                    (crashed.returncode, "没跑完" in crashed.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None, bool(added) and "没跑完" in " ".join(added[0].get("findings") or []))))
    # 驱动脚本在而读不了（chmod 成 000）：照常放行、这一条命令记一行检出（「范围声明闸判不了」起头）；driver-unread-unrecorded 下这一格红
    locked_driver = os.path.join(work, "locked-driver.sh")
    with open(locked_driver, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\nbash .claude/gate.d/doc-text.sh .\n")
    os.chmod(locked_driver, 0)
    before = len(recorded_entries())
    try:
        locked_run = feed("bash ./locked-driver.sh", session="s-locked")
    finally:
        os.chmod(locked_driver, 0o600)
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-locked"]
    results.append(("入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）", (0, 1, "只记", True),
                    (locked_run.returncode, len(added), added[0].get("outcome") if added else None,
                     bool(added) and any(str(finding).startswith("范围声明闸判不了") and "读不了" in str(finding) for finding in added[0].get("findings") or []))))
    # 弄坏开关只在自证里认：不带标记、或标记文件内容对不上的平常跑设了开关，入口在读 hook 的 JSON 之前退 2、stderr 点名它只在自证里认（探法同门禁
    #   code-tooling 的 break-switch-outside-selftest 格）；带标记的自证子进程照认开关。honor-outside-selftest 设着时探的开关里也带上它，前两格红。检出写另一份，不搅前面几格的数
    honored = "honor-outside-selftest" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    probe_item = "honor-outside-selftest" if honored else "declaration-optional"
    unmarked = {key: value for key, value in environment.items() if key != SELFTEST_MARK_VARIABLE}
    unjudged = os.path.join(work, "outside-selftest-detections.jsonl")
    plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                           env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True),
                    (plain.returncode, any(BREAK_VARIABLE_NAME in line and "自证" in line for line in plain.stderr.splitlines()))))
    forged_mark = os.path.join(work, "forged-selftest-mark")
    with open(forged_mark, "w", encoding="utf-8") as handle:
        handle.write("别处写的标记\n")
    plain_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "ls -la"}})
    forged = subprocess.run(["bash", guard_script], input=plain_input, capture_output=True, text=True, timeout=120,
                            env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged, SELFTEST_MARK_VARIABLE: forged_mark}))
    results.append(("入口：标记文件内容对不上照拒", 2, forged.returncode))
    marked_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    marked = subprocess.run(["bash", guard_script], input=marked_input, capture_output=True, text=True, timeout=120,
                            env=dict(environment, **{BREAK_VARIABLE_NAME: "declaration-optional", DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：自证起的子进程（带标记）照认弄坏开关", 0, marked.returncode))
    # 关守卫项 honor-outside-selftest 只在单独一项时认（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：与别的项同写，平常跑与带标记的子进程都退 2、点名它；
    #   honor-outside-selftest-with-others 设着时带标记那一格的开关里也带上它（改前的样子：同写照认），那一格红
    combined_item = "honor-outside-selftest,declaration-optional"
    combined_plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                    env=dict(unmarked, **{BREAK_VARIABLE_NAME: combined_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：平常跑退 2、点名它只在单独一项时认", (2, True),
                    (combined_plain.returncode, "honor-outside-selftest" in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
    lenient = "honor-outside-selftest-with-others" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    combined_marked = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                     env=dict(environment, **{BREAK_VARIABLE_NAME: combined_item + (",honor-outside-selftest-with-others" if lenient else ""),
                                                              DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它", (2, True),
                    (combined_marked.returncode, "honor-outside-selftest" in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    return results


def failed_labels(results):
    return [label for label, want, got in results if want != got]


def selftest(hook_directory):
    load_judging_libraries(hook_directory)
    guard_script = os.path.join(hook_directory, GUARD_SCRIPT_NAME)
    work = tempfile.mkdtemp(prefix="run-scope-guard-")
    # 外面设了弄坏开关：照它判一遍、点名的格应当红，不跑开关轮（开关轮自己逐个设）
    outside_break = os.environ.get(BREAK_VARIABLE_NAME) or None
    try:
        build_selftest_tree(work)
        # 自证起的入口带这个标记才认弄坏开关（入口的守卫；不带标记的平常跑设了开关退 2）
        selftest_mark = os.path.join(work, "selftest-mark")
        with open(selftest_mark, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        os.environ[SELFTEST_MARK_VARIABLE] = selftest_mark
        results = selftest_judge_cells(work) + selftest_entry_cells(work, guard_script)
        failures = failed_labels(results)
        for label, want, got in results:
            if want != got:
                print(f"  ✗ 自检：{label}：应当是 {want!r}，实际 {got!r}")
                print(f"  → 怎么办：照 {label} 那一格的判法改 lib_run_scope.py，再跑 bash .claude/hooks/run-scope-guard.sh --selftest")
        if outside_break:
            print(f"  （{BREAK_VARIABLE_NAME}={outside_break} 下判了 {len(results)} 格，红 {len(failures)} 格；开关轮不跑）")
            return 1 if failures else 0
        labels = [label for label, _want, _got in results]
        try:
            with open(guard_script, encoding="utf-8") as handle:
                header = handle.read()
        except OSError:
            header = ""
        switch_problems = []
        for switch, cells in BREAK_SWITCH_CELLS.items():
            missing = [cell for cell in cells if cell not in labels]
            if missing:
                switch_problems.append(f"开关 {switch} 点名的格不在自证里：{'、'.join(missing)}")
                continue
            if not re.search(r"^#\s+" + re.escape(f"{BREAK_VARIABLE_NAME}={switch}") + r"\s", header, re.M):
                switch_problems.append(f"开关 {switch} 没写进 {GUARD_SCRIPT_NAME} 的文件头（一行「{BREAK_VARIABLE_NAME}={switch}  <弄坏哪一处>」，下一行起写红的格）")
            os.environ[BREAK_VARIABLE_NAME] = switch
            try:
                broken = selftest_judge_cells(work)
                if any(cell.startswith("入口：") for cell in cells):   # 另起入口的几格慢，只在开关点名了它们时再跑
                    broken += selftest_entry_cells(work, guard_script)
            finally:
                os.environ.pop(BREAK_VARIABLE_NAME, None)
            reds = failed_labels(broken)
            still_green = [cell for cell in cells if cell not in reds]
            if still_green:
                switch_problems.append(f"开关 {switch} 打开后这几格没红：{'、'.join(still_green)}")
            unexpected = [cell for cell in reds if cell not in cells]
            if unexpected:
                switch_problems.append(f"开关 {switch} 打开后另有这几格红、表里没写：{'、'.join(unexpected)}（写进 BREAK_SWITCH_CELLS 与文件头）")
        for problem in switch_problems:
            print(f"  ✗ 自检：{problem}")
        if failures or switch_problems:
            print(f"  → 怎么办：照上面点名的格改 {os.path.join(hook_directory, 'lib_run_scope.py')} 的判法（或 {GUARD_SCRIPT_NAME} 的文件头），再单独跑 "
                  f"bash .claude/hooks/{GUARD_SCRIPT_NAME} --selftest 看它转绿")
            return 1
        memory_count = len(selftest_cells(work))
        print(f"  ✓ {GUARD_SCRIPT_NAME} 自检通过：{len(results)} 格（内存里判 {memory_count} 格、另起入口 {len(results) - memory_count} 格）；"
              f"开关轮 {len(BREAK_SWITCH_CELLS)} 个开关逐个打开，红的格与表里点名的逐格相等（共 {sum(len(cells) for cells in BREAK_SWITCH_CELLS.values())} 格次）")
        return 0
    finally:
        if outside_break:
            os.environ[BREAK_VARIABLE_NAME] = outside_break
        os.environ.pop(SELFTEST_MARK_VARIABLE, None)
        shutil.rmtree(work, ignore_errors=True)


def main():
    hook_directory = sys.argv[1] if len(sys.argv) > 1 else HOOK_DIRECTORY
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_directory)
    return hook_main(hook_directory)


if __name__ == "__main__":
    sys.exit(main())
