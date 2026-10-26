"""two-logs-guard.sh 的判法：门禁、cargo test、四个入口脚本与 gate-staged.sh 的输出不许用 > 、>>、&>、| tee 写到 .log；自证。

入口 two-logs-guard.sh 把 hook 的 JSON 从标准输入交给这一份（hook_main）；--selftest 走 selftest。
切词、剥包装、认命令位置都从同目录的 lib_shell_words.py 导入，认哪一道门禁阶段、哪个脚本在哪一层从 lib_heavy_tests.py 导入，不另写一份。

判法（log_writes_in_text）：
  把命令文本按管道切成一条条简单命令（lib_shell_words 的 shell_tokens / simple_commands），每条剥掉前缀与包装（nice、timeout、env、capped.sh、
  run-with-memory-cap.sh、bash 起脚本、bash -c）之后，里面有没有「被盯的命令」：
    门禁阶段（.claude/gate.d/ 顶层的 *.sh）、gate.sh、cargo test、replay.sh、crash-point-replay.sh、crash-amplification.sh、merge-and-verify.sh、gate-staged.sh；
    带 --selftest、--help、-h、--list、--list-items、--admission-only 的是只查清单或自证，不盯。
  盯到的命令：
    ① 简单命令自己的输出重定向（>、>>、>|、&>、&>>，2> 之类的 fd 号切词时已丢，也算）目标是日志 → 拒；
    ② 同一条管道里有一段是 tee（含 -a）、文件参数是日志 → 拒；
    ③ bash -c '…' 的代码里按同样两条再判一遍（递归，至多 MAXIMUM_NESTING 层）；
    ④ 整组的输出重定向（`{ …; } > x.log`、`( … ) > x.log`）与整组经管道接的 tee 挂到组里每一条被盯的命令上（grouped_commands、group_log_writes）。
  「目标是日志」：目标词里有 .log（后面不接字母数字下划线），或写成引用名字里带 log 的变量（$log、${LOG_FILE}、"$run_directory/$log_name"）；/dev/null 与 &1、&2 这类 fd 不算。
罩不住的：exec > x.log、if / while / for 这类复合命令整体的重定向、变量里拼出来的命令、eval、python 的 subprocess、驱动脚本里写的重定向（只判这一条命令文本）。
判不了的（hook 输入不是 JSON 对象、判的时候抛异常）照常放行，stderr 报一句、记一条检出（unjudged_and_noted）。

弄坏开关 TWO_LOGS_GUARD_BREAK=<项>（几个用逗号连；只给自证判红用；认不出的项抛 ValueError，钩子照常放行并报一句），见 BREAK_SWITCH_CELLS；
开关与红的格写在 two-logs-guard.sh 文件头，自证末了的开关轮核两边逐格相等。
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
from datetime import datetime, timezone
from typing import NamedTuple

sys.dont_write_bytecode = True

HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
GUARD_SCRIPT_NAME = "two-logs-guard.sh"
BREAK_VARIABLE_NAME = "TWO_LOGS_GUARD_BREAK"
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
TWO_LOGS_COMMAND = "scripts/hosts/two-logs.sh"
# 被盯的入口脚本：脚本名 → 它在仓里的目录（相对仓根）
WATCHED_ENTRY_SCRIPTS = {
    "replay.sh": "scripts/experiment-apparatus",
    "crash-point-replay.sh": "scripts/crash",
    "crash-amplification.sh": "scripts/crash",
    "merge-and-verify.sh": "scripts/gate",
    "gate-staged.sh": "scripts/gate",
}
# 带这些参数之一的只查清单、查用法或自证，输出短，不盯
QUERY_ARGUMENTS = frozenset({"--selftest", "--help", "-h", "--list", "--list-items", "--admission-only"})
LOG_NAME = re.compile(r"\.log(?![A-Za-z0-9_])")
LOG_VARIABLE = re.compile(r"\$\{?[A-Za-z_0-9]*[Ll][Oo][Gg][A-Za-z_0-9]*")
FILE_DESCRIPTOR_TARGET = re.compile(r"^&?(?:\d+|-)$")
TEE_COMMAND = "tee"
MAXIMUM_NESTING = 4
TEMPLATE_COMMAND_LIMIT = 600

# 弄坏开关 → 打开后红的自证格，写全（two-logs-guard.sh 文件头逐项照这张表写）
BREAK_SWITCH_CELLS = {
    "redirect-unchecked": ["bash -c 里的 cargo test > 写 .log 拒", "cargo test &> 写 .log 拒", "gate.sh >> 写 .log 拒", "merge-and-verify.sh > 写 .log 拒", "timeout 包着的 gate-staged.sh > 写 .log 拒", "入口：门禁阶段 > 写 .log 拒（退出码 2）", "套了 capped.sh 的 cargo test > 写 .log 拒", "套了 nice 与声明的 gate-staged.sh > 写 .log 拒", "崩溃放量入口 > 写 .log 拒", "崩溃点重放入口 > 写变量名带 log 的目标拒", "拒绝说明逐条写被盯的命令与目标", "门禁阶段 2>&1 重定向写 .log 拒", "门禁阶段 > 写 .log 拒"],
    "tee-unchecked": ["bash -c 里的管道 tee 写 .log 拒", "gate-staged.sh |& tee 写 .log 拒", "replay.sh | tee -a 写 .log 拒", "入口：拒绝说明写 two-logs.sh 的写法、被拒的记一行", "拒绝说明逐条写被盯的命令与目标", "门禁阶段 | tee 写 .log 拒"],
    "nested-code-unread": ["bash -c 里的 cargo test > 写 .log 拒", "bash -c 里的管道 tee 写 .log 拒"],
    "query-forms-guarded": ["崩溃放量 --list-items | tee 写 .log 放", "门禁阶段 --list > 写 .log 放", "门禁阶段 --selftest | tee 写 .log 放"],
    "variable-names-ignored": ["崩溃点重放入口 > 写变量名带 log 的目标拒"],
    "all-cargo-guarded": ["cargo build > 写 .log 放"],
    "non-log-targets-guarded": ["门禁阶段 > /dev/null 放", "门禁阶段 > 写 .txt 放", "花括号组 > 写 .txt 放"],
    "no-way-out": ["入口：拒绝说明写 two-logs.sh 的写法、被拒的记一行"],
    "group-redirect-unchecked": ["花括号组 > 写 .log 拒", "圆括号子 shell > 写 .log 拒", "花括号组 2>&1 | tee 写 .log 拒", "子 shell 里套数组赋值、组外 > 写 .log 拒", "入口：圆括号子 shell 整组 > 写 .log 拒（退出码 2）"],
    "failure-unrecorded": ["入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", "入口：判的时候抛异常放行、stderr 报一句、记一条检出",
                           "入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", "入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置"],
    "judge-crash-unrecorded": ["入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）"],
    "honor-outside-selftest": ["入口：平常跑设了弄坏开关退 2、点名它只在自证里认", "入口：标记文件内容对不上照拒"],
    "honor-outside-selftest-with-others": ["入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它"],
}
# 弄坏开关只在自证里认：入口只在环境里这个变量指到自证建的标记文件、内容逐字是 SELFTEST_MARK_CONTENT 时认开关（入口那一段守卫里写着同一句）
SELFTEST_MARK_VARIABLE = "TWO_LOGS_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "two-logs-guard.sh --selftest 起的子进程"

heavy_tests = None
shell_words = None


def guard_switch_on(name):
    """弄坏开关 TWO_LOGS_GUARD_BREAK 里有没有这一项；有不认识的项抛 ValueError（免得拼错的开关被当成没开）。"""
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


class LogWrite(NamedTuple):
    kind: str          # redirect / tee
    watched: str       # 被盯的命令是什么（给 agent 看的一句）
    target: str        # 写到哪个日志（照写的样子）
    prefix: list       # 简单命令命令词之前的前缀词（环境赋值、nice 这类），模板里放在 two-logs.sh 前面
    command: list      # 简单命令从命令词起的原词（连内存包装、capped.sh 都在），模板里放在 -- 后面


def two_logs_target_is_log(target):
    """重定向或 tee 的目标是不是日志：有 .log，或引用名字里带 log 的变量；/dev/null 与 fd（&1、2）不算。"""
    if FILE_DESCRIPTOR_TARGET.match(target):
        return False
    if guard_switch_on("non-log-targets-guarded"):
        return True
    if target == "/dev/null":
        return False
    return bool(LOG_NAME.search(target) or (LOG_VARIABLE.search(target) and not guard_switch_on("variable-names-ignored")))


def watched_description(words, directory):
    """命令（前缀与包装剥掉之后的词）是被盯的哪一样：交一句描述；不是、或是只查清单与自证的写法，交 None。"""
    if not words:
        return None
    if not guard_switch_on("query-forms-guarded") and any(argument in QUERY_ARGUMENTS for argument in words[1:]):
        return None
    name = os.path.basename(words[0])
    stage = heavy_tests.gate_stage_name(words[0], directory)
    if stage is not None:
        return f"门禁 {stage}"
    if name == "gate.sh":
        return "整轮门禁 gate.sh"
    if name in WATCHED_ENTRY_SCRIPTS and heavy_tests.located_in(words[0], directory, WATCHED_ENTRY_SCRIPTS[name]):
        return name
    if name == "cargo":
        command = heavy_tests.cargo_subcommand(words[1:], directory)
        if command is not None and (command.subcommand in ("test", "t") or guard_switch_on("all-cargo-guarded")):
            return "cargo test" if command.subcommand in ("test", "t") else f"cargo {command.subcommand}"
    return None


def split_pipelines(tokens):
    """词法单元 → 管道的列表；每条管道是若干段（| 与 |& 隔开），每段是一串词法单元。"""
    pipelines, segments, current = [], [], []
    for token in tokens:
        is_operator, text = token
        if is_operator is True and text in ("|", "|&"):
            segments.append(current)
            current = []
        elif is_operator is True and text in shell_words.COMMAND_SEPARATORS:
            segments.append(current)
            pipelines.append(segments)
            segments, current = [], []
        else:
            current.append(token)
    segments.append(current)
    pipelines.append(segments)
    return pipelines


def grouped_commands(tokens):
    """词法单元里的花括号组 `{ …; }` 与圆括号子 shell `( … )`（嵌套的各算一组）：交 [(组里的词法单元, 组外紧跟的输出重定向目标, 组后经管道接的 tee 的文件参数)]。
    开括号只在命令开头认（`{` 是词、`(` 是运算符）；`名字=(…)` 数组与 `f()` 的括号不算组；`}` 只在分隔符之后认。"""
    groups, stack, at_command_start = [], [], True
    for index, (is_operator, text) in enumerate(tokens):
        if is_operator is None:   # 命令替换的正文，另由 bash 当一段命令判，不影响这一层
            continue
        if (is_operator is True and text == "(") or (is_operator is False and text == "{" and at_command_start):
            stack.append((text if at_command_start else "array", index))
            at_command_start = True
            continue
        if stack and ((is_operator is True and text == ")" and stack[-1][0] in ("(", "array"))
                      or (is_operator is False and text == "}" and at_command_start and stack[-1][0] == "{")):
            kind, start = stack.pop()
            if kind != "array":
                groups.append((tokens[start + 1:index], *redirections_after_group(tokens, index + 1)))
            at_command_start = False
            continue
        if is_operator is True:
            at_command_start = text in shell_words.COMMAND_SEPARATORS
        else:
            at_command_start = at_command_start and text in shell_words.COMMAND_KEYWORDS
    return groups


def redirections_after_group(tokens, position):
    """组的右括号之后：紧跟的输出重定向目标，与经 | / |& 接的那一段若是 tee 时它的文件参数。交 (目标列表, tee 的文件参数列表)。"""
    targets, tee_files = [], []
    while (position + 1 < len(tokens) and tokens[position][0] is True and tokens[position][1] not in shell_words.COMMAND_SEPARATORS
           and tokens[position + 1][0] is False):
        if tokens[position][1] in shell_words.OUTPUT_REDIRECTS:
            targets.append(tokens[position + 1][1])
        position += 2
    if position < len(tokens) and tokens[position] in ((True, "|"), (True, "|&")):
        words = []
        for is_operator, text in tokens[position + 1:]:
            if is_operator is True and text in shell_words.COMMAND_SEPARATORS:
                break
            if is_operator is False:
                words.append(text)
        skipped = shell_words.skip_prefixes(words)
        if skipped.command_position is not None and os.path.basename(words[skipped.command_position]) == TEE_COMMAND:
            tee_files = [argument for argument in words[skipped.command_position + 1:] if not argument.startswith("-")]
    return targets, tee_files


def group_log_writes(tokens, directory, environment):
    """整组的输出重定向（与整组接的 tee）挂到组里每一条被盯的命令上：目标照「目标是日志」同一判据（文件头「判法」④）。"""
    found = []
    for body, targets, tee_files in grouped_commands(tokens):
        logs = [("redirect", target) for target in targets if two_logs_target_is_log(target)]
        logs += [("tee", target) for target in tee_files if two_logs_target_is_log(target)]
        if not logs:
            continue
        simple, _targets, _substitutions, _redirections = shell_words.simple_commands(body)
        for words in simple:
            skipped = shell_words.skip_prefixes(words)
            if not words or skipped.command_position is None:
                continue
            inner = shell_words.commands_at_command_position(shlex.join(words), directory, environment).commands
            what = next((described for described in (watched_description(command.words, command.directory) for command in inner) if described), None)
            if what:
                found += [LogWrite(kind, what, target, words[:skipped.command_position], words[skipped.command_position:]) for kind, target in logs]
    return found


def log_writes_in_text(text, directory=None, environment=None, depth=0):
    """一段命令文本里被盯的命令写日志的每一处（见文件头「判法」）。"""
    found = []
    tokens = shell_words.shell_tokens(shell_words.strip_data_heredocs(text))
    if not guard_switch_on("group-redirect-unchecked"):
        found += group_log_writes(tokens, directory, environment)
    for segments in split_pipelines(tokens):
        watched_here, tee_targets = [], []   # watched_here：(描述, 前缀词, 原词)
        for segment in segments:
            simple, _targets, _substitutions, redirections = shell_words.simple_commands(segment)
            for words, command_redirections in zip(simple, redirections):
                if not words:
                    continue
                skipped = shell_words.skip_prefixes(words)
                prefix = words[:skipped.command_position] if skipped.command_position is not None else []
                inner = shell_words.commands_at_command_position(shlex.join(words), directory, environment).commands
                described = [(watched_description(command.words, command.directory), command) for command in inner]
                described = [(what, command) for what, command in described if what]
                if described:
                    what, command = described[0]
                    raw = words[skipped.command_position:] if skipped.command_position is not None else list(words)
                    watched_here.append((what, prefix, raw))
                    if not guard_switch_on("redirect-unchecked"):
                        for operator, target in command_redirections:
                            if operator in shell_words.OUTPUT_REDIRECTS and two_logs_target_is_log(target):
                                found.append(LogWrite("redirect", what, target, prefix, raw))
                if inner and os.path.basename(inner[0].words[0]) == TEE_COMMAND and not guard_switch_on("tee-unchecked"):
                    tee_targets += [argument for argument in inner[0].words[1:] if not argument.startswith("-") and two_logs_target_is_log(argument)]
                if depth < MAXIMUM_NESTING and not guard_switch_on("nested-code-unread") and skipped.command_position is not None:
                    remaining = words[skipped.command_position:]
                    while remaining and os.path.basename(remaining[0]) in shell_words.WRAPPERS_TAKING_ONE_ARGUMENT and len(remaining) >= 3:
                        remaining = remaining[2:]
                    if remaining and os.path.basename(remaining[0]) in shell_words.SHELL_NAMES:
                        invocation = shell_words.shell_invocation(remaining[1:])
                        if invocation.code_string is not None:
                            found += log_writes_in_text(invocation.code_string, directory, environment, depth + 1)
        if watched_here and tee_targets:
            what, prefix, raw = watched_here[0]
            found.append(LogWrite("tee", what, tee_targets[0], prefix, raw))
    return found


def two_logs_name_of(target):
    """目标是字面写的 .log 路径时去掉 .log 当 two-logs.sh 的名；变量或别的写法交占位。"""
    if "$" not in target and target.endswith(".log"):
        return target[:-len(".log")]
    return "<名>"


def refusal_message(writes, command_text):
    lines = ["✗ 日志写法被拒，这条命令没跑（two-logs-guard.sh）：门禁、cargo test、replay.sh、crash-point-replay.sh、crash-amplification.sh、merge-and-verify.sh、gate-staged.sh 的输出",
             "  不用 >、>>、&>、| tee 写到 .log——一份全量日志动辄上万行，要看的只是里面的错误与警告。"]
    for index, write in enumerate(writes, 1):
        how = f"重定向到 {write.target}" if write.kind == "redirect" else f"经 tee 写到 {write.target}"
        lines.append(f"  {index}. {write.watched}：{how}")
    if guard_switch_on("no-way-out"):
        return "\n".join(lines)
    first = writes[0]
    name = two_logs_name_of(first.target)
    example = ""
    if first.command:
        prefix = (shlex.join(first.prefix) + " ") if first.prefix else ""
        example = f"{prefix}bash {TWO_LOGS_COMMAND} {name} -- {shlex.join(first.command)}"
        if len(example) > TEMPLATE_COMMAND_LIMIT:
            example = example[:TEMPLATE_COMMAND_LIMIT] + " …"
    lines += [
        f"→ 怎么办：改成 {TWO_LOGS_COMMAND} 包住它，日志自动出两份：<名>.log 是全量，<名>.problems.log 只收错误与警告（门禁的 ✗ 与它的 → 出路、⚠、本次未跑、逐格汇总，",
        "  cargo / rustc 的 error 与 warning:、libtest 的 FAILED 与 panicked、Python 的 Traceback、非 0 退出码），每行前面带它在全量日志里的行号；一般只看后一份。",
        f"  写法：bash {TWO_LOGS_COMMAND} <名，不带 .log 后缀> -- <原命令>；输出照常转打到屏幕，退出码是原命令的；只要落盘、不要屏幕上的输出加 --quiet。",
    ]
    if example:
        lines.append(f"  照抄这一条（名按你写的目标现取，变量写法的占位 <名> 自己填）：{example}")
    lines.append(f"→ 已经有一份全量 .log、要现筛出只含错误警告的那份：bash {TWO_LOGS_COMMAND} --from-log <名>")
    return "\n".join(lines)


class GuardDecision(NamedTuple):
    code: int
    message: str | None
    writes: list


def decide_logs(hook_input, project_root):
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    directory = hook_input.get("cwd") or project_root
    writes = log_writes_in_text(command_text, directory)
    if writes:
        return GuardDecision(2, refusal_message(writes, command_text), writes)
    return GuardDecision(0, None, [])


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


def detection_entry(hook_input, findings, outcome):
    """检出记录的一行；hook 输入读不出来（hook_input 是 None）时会话号、agent 与会话记录照实写「读不到」。"""
    known = hook_input if isinstance(hook_input, dict) else {"session_id": "读不到", "agent_id": "读不到", "agent_type": "读不到", "transcript_path": "读不到"}
    tool_input = known.get("tool_input") if isinstance(known.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    return {"time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME, "agent_id": known.get("agent_id"),
            "agent_type": known.get("agent_type") or "主 agent", "transcript_path": known.get("transcript_path"),
            "command": (command if isinstance(command, str) else repr(command))[:2000], "session_id": known.get("session_id"),
            "findings": findings, "outcome": outcome}


def append_entry(path, entry):
    if not path:
        return
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def record_refusal(hook_input, decision, path):
    """被拒的整条命令记一行（session_id、outcome 被拒；看门狗只记不叫醒）。放行的不记。"""
    if not path or not decision.code:
        return
    append_entry(path, detection_entry(hook_input, [f"日志写法被拒：{write.watched}：{write.kind} 写到 {write.target}" for write in decision.writes], "被拒"))


UNREADABLE_INPUT_SHOWN_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节


def failure_left_unrecorded():
    """弄坏开关 failure-unrecorded 设没设：直接读变量，不经 guard_switch_on（判的时候抛异常那一支也要读它，自证往开关里塞认不出的项来注入那个异常）。"""
    return "failure-unrecorded" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")


def unjudged_and_noted(message, hook_input, hook_directory):
    """判不了、照常执行：stderr 打这一句，检出记录记一行（outcome 只记）；弄坏开关 failure-unrecorded 下只打 stderr（改前的样子）。"""
    print(f"  ! {message}", file=sys.stderr)
    if not failure_left_unrecorded():
        append_entry(detections_path_now(hook_directory), detection_entry(hook_input, [message], "只记"))


def hook_main(hook_directory):
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw)
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        hook_input, why = None, f"解析出错：{error!r}"
    else:
        why = None if isinstance(hook_input, dict) else f"解出来是 {type(hook_input).__name__}，不是对象"
    if why:   # 照常执行，打一行可定位的诊断（钩子名、开头的字节）、记一条检出（会话号读不到）
        shown = raw[:UNREADABLE_INPUT_SHOWN_BYTES]
        unjudged_and_noted(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象（{why}），这条命令没判、照常执行；开头 {len(shown)} 字节（共 {len(raw)} 字节）：{shown!r}",
                           None, hook_directory)
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    # 粗筛：命令里连 log 这三个字母都没有的，写不出 .log 也写不出名字带 log 的变量目标
    if "log" not in command_text.lower():
        return 0
    try:
        load_judging_libraries(hook_directory)
        project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_directory))
        decision = decide_logs(hook_input, project_root)
        record_refusal(hook_input, decision, detections_path_now(hook_directory))
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出
        unjudged_and_noted(f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），这条命令照常执行", hook_input, hook_directory)
        return 0
    if decision.code:
        print(decision.message, file=sys.stderr)
    return decision.code


# ── 自证 ──────────────────────────────────────────────

def memory_cells():
    """[(格名, 命令, 该拒 2 / 该放 0)]：内存里判，不起进程。"""
    cells = []

    def cell(label, command, code):
        cells.append((label, command, code))

    cell("门禁阶段 > 写 .log 拒", "bash .claude/gate.d/doc-text.sh . > /tmp/doc-text.log", 2)
    cell("门禁阶段 2>&1 重定向写 .log 拒", "bash .claude/gate.d/code-tooling.sh --check agent-write-scope . > /tmp/out.log 2>&1", 2)
    cell("gate.sh >> 写 .log 拒", "bash .claude/scripts/gate.sh --staged >> gate.log", 2)
    cell("cargo test &> 写 .log 拒", "cargo test -p singlefs-harness --lib &> /tmp/t.log", 2)
    cell("崩溃点重放入口 > 写变量名带 log 的目标拒", 'bash scripts/crash/crash-point-replay.sh --quick > "$run_log"', 2)
    cell("崩溃放量入口 > 写 .log 拒", "bash scripts/crash/crash-amplification.sh --item fixed-script > amp.log", 2)
    cell("merge-and-verify.sh > 写 .log 拒", "bash scripts/gate/merge-and-verify.sh patches/a > merge.log 2>&1", 2)
    cell("套了 nice 与声明的 gate-staged.sh > 写 .log 拒",
         "SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=整轮; 范围=整轮; 为什么=提交前整轮' SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash scripts/gate/gate-staged.sh > /tmp/staged.log", 2)
    cell("bash -c 里的 cargo test > 写 .log 拒", "bash -c 'cargo test --workspace > /tmp/t.log 2>&1'", 2)
    cell("套了 capped.sh 的 cargo test > 写 .log 拒", "bash scripts/hosts/capped.sh 7 cargo test -p singlefs-core > /tmp/t.log", 2)
    cell("timeout 包着的 gate-staged.sh > 写 .log 拒", "timeout 200 bash scripts/gate/gate-staged.sh > $HOME/gate.log", 2)
    cell("门禁阶段 | tee 写 .log 拒", "bash .claude/gate.d/doc-text.sh . 2>&1 | tee /tmp/doc-text.log", 2)
    cell("replay.sh | tee -a 写 .log 拒", "bash scripts/experiment-apparatus/replay.sh E10 | tee -a replay.log", 2)
    cell("gate-staged.sh |& tee 写 .log 拒", "bash scripts/gate/gate-staged.sh |& tee g.log", 2)
    cell("bash -c 里的管道 tee 写 .log 拒", "bash -c 'bash .claude/gate.d/doc-text.sh . | tee t.log'", 2)
    cell("two-logs.sh 包着放行", "bash scripts/hosts/two-logs.sh /tmp/doc-text -- bash .claude/gate.d/doc-text.sh .", 0)
    cell("two-logs.sh 包着、外面再 tee 屏幕副本放行", "bash scripts/hosts/two-logs.sh /tmp/x -- cargo test 2>&1 | tee /tmp/screen.log", 0)
    cell("门禁阶段不重定向放行", "bash .claude/gate.d/doc-text.sh .", 0)
    cell("门禁阶段 > 写 .txt 放", "bash .claude/gate.d/doc-text.sh . > /tmp/doc-text.txt", 0)
    cell("门禁阶段 > /dev/null 放", "bash .claude/gate.d/doc-text.sh . > /dev/null 2>&1", 0)
    cell("门禁阶段 --list > 写 .log 放", "bash .claude/gate.d/doc-text.sh --list > /tmp/cells.log", 0)
    cell("门禁阶段 --selftest | tee 写 .log 放", "bash .claude/gate.d/doc-text.sh --selftest 2>&1 | tee /tmp/selftest.log", 0)
    cell("崩溃放量 --list-items | tee 写 .log 放", "bash scripts/crash/crash-amplification.sh --list-items | tee items.log", 0)
    cell("cargo build > 写 .log 放", "cargo build --release > /tmp/build.log 2>&1", 0)
    cell("不是被盯的命令写 .log 放", "echo hello > /tmp/a.log; cat /tmp/a.log | tee /tmp/b.log", 0)
    cell("别处的同名脚本放行", "bash /tmp/somewhere/replay.sh > /tmp/r.log", 0)
    # 整组重定向挂到组里每一条被盯的命令上（grouped_commands；开关 group-redirect-unchecked 下照拒的四格红，non-log-targets-guarded 下写 .txt 那一格红）
    cell("花括号组 > 写 .log 拒", "{ cargo test --workspace; } > /tmp/test.log", 2)
    cell("圆括号子 shell > 写 .log 拒", "( cargo test --workspace ) > /tmp/test.log", 2)
    cell("花括号组 2>&1 | tee 写 .log 拒", "{ echo start; bash .claude/gate.d/doc-text.sh .; } 2>&1 | tee /tmp/doc-text.log", 2)
    cell("子 shell 里套数组赋值、组外 > 写 .log 拒", "( items=(a b); cd /tmp && cargo test -p singlefs-harness --lib ) &> /tmp/t.log", 2)
    cell("组里没有被盯的命令 > 写 .log 放", "{ echo a; ls; } > /tmp/x.log", 0)
    cell("花括号组 > 写 .txt 放", "{ cargo test --workspace; } > /tmp/test.txt", 0)
    return cells


def entry_cells(work, guard_script):
    """另起入口 two-logs-guard.sh 喂 hook 的 JSON。交回 [(说明, 期望, 实际)]。"""
    results = []
    detections = os.path.join(work, "detections.jsonl")
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, CLAUDE_PROJECT_DIR=work)

    def feed(command, agent_type=None):
        hook_input = {"tool_name": "Bash", "cwd": work, "session_id": "s-two-logs", "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        return subprocess.run(["bash", guard_script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=120)

    results.append(("入口：门禁阶段 > 写 .log 拒（退出码 2）", 2, feed("bash .claude/gate.d/doc-text.sh . > /tmp/doc-text.log").returncode))
    if os.path.exists(detections):
        os.remove(detections)
    refused = feed("bash .claude/gate.d/doc-text.sh . 2>&1 | tee /tmp/doc-text.log", "tooling-writer")
    wanted = ("two-logs.sh", "--from-log", "--quiet", "problems.log", "bash scripts/hosts/two-logs.sh /tmp/doc-text -- bash .claude/gate.d/doc-text.sh .")
    recorded = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            recorded = [json.loads(line) for line in handle if line.strip()]
    results.append(("入口：拒绝说明写 two-logs.sh 的写法、被拒的记一行", (2, True, 1, "被拒", "tooling-writer"),
                    (refused.returncode, all(text in refused.stderr for text in wanted), len(recorded),
                     recorded[0].get("outcome") if recorded else None, recorded[0].get("agent_type") if recorded else None)))
    allowed = feed("bash scripts/hosts/two-logs.sh /tmp/doc-text -- bash .claude/gate.d/doc-text.sh .")
    results.append(("入口：包了 two-logs.sh 放行、不写 stderr", (0, ""), (allowed.returncode, allowed.stderr)))
    results.append(("入口：别的命令放行", 0, feed("ls -la").returncode))
    # 判法文件读不到：照常放行、报一句、记一条检出（钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉）。入口自己记：AGENT_HOOK_DETECTIONS 设了记那里，
    #   没设记进同目录 lib_work_root.py 的默认位置（这里把 SINGLEFS_WORK_ROOT 指进临时目录，不碰真的检出记录）。开关 failure-unrecorded 下记检出的两格红
    lonely = os.path.join(work, "lonely", ".claude", "hooks")
    os.makedirs(lonely, exist_ok=True)
    shutil.copy(guard_script, os.path.join(lonely, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(os.path.abspath(guard_script)), "lib_work_root.py"), lonely)
    claude_directory = os.path.dirname(os.path.dirname(os.path.abspath(guard_script)))
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "lonely", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)

    def recorded_at(path):
        if not os.path.exists(path):
            return []
        with open(path, encoding="utf-8") as handle:
            return [json.loads(line) for line in handle if line.strip()]
    orphan_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-orphan", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh . > x.log"}})
    before = len(recorded_at(detections))
    orphan = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_at(detections)[before:]
    results.append(("入口：判法文件读不到时放行并报一句", (0, True), (orphan.returncode, "照常执行" in orphan.stderr)))
    results.append(("入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", [("s-orphan", "只记", GUARD_SCRIPT_NAME, True)],
                    [(entry.get("session_id"), entry.get("outcome"), entry.get("hook"), "读不到判法" in " ".join(entry.get("findings") or [])) for entry in added]))
    default_root = os.path.join(work, "orphan-work-root")
    default_path = os.path.join(default_root, "agent-hook-detections.jsonl")
    unset_environment = {key: value for key, value in environment.items() if key not in (DETECTIONS_VARIABLE, "SINGLEFS_WORK_ROOT_BREAK")}
    unset_environment["SINGLEFS_WORK_ROOT"] = default_root
    before = len(recorded_at(default_path))
    orphan_default = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True,
                                    env=unset_environment, timeout=60)
    added = recorded_at(default_path)[before:]
    results.append(("入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置", (0, [("s-orphan", "只记")]),
                    (orphan_default.returncode, [(entry.get("session_id"), entry.get("outcome")) for entry in added])))
    # 判法文件在而没跑完（这里放一份导入就抛异常的 lib_two_logs_guard.py）：入口照常放行、stderr 报一句、自己记一条检出（会话号照输入）；
    #   judge-crash-unrecorded 下这一格红（只打 stderr、不记）
    crashing = os.path.join(work, "crashing", ".claude", "hooks")
    os.makedirs(crashing, exist_ok=True)
    shutil.copy(guard_script, os.path.join(crashing, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(os.path.abspath(guard_script)), "lib_work_root.py"), crashing)
    with open(os.path.join(crashing, "lib_two_logs_guard.py"), "w", encoding="utf-8") as handle:
        handle.write("raise RuntimeError('自证造的导入错')\n")
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "crashing", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    before = len(recorded_at(detections))
    crashed = subprocess.run(["bash", os.path.join(crashing, GUARD_SCRIPT_NAME)],
                             input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-crash", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh . > x.log"}}),
                             capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_at(detections)[before:]
    results.append(("入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）", (0, True, [("s-crash", "只记", True)]),
                    (crashed.returncode, "没跑完" in crashed.stderr,
                     [(entry.get("session_id"), entry.get("outcome"), "没跑完" in " ".join(entry.get("findings") or [])) for entry in added])))
    results.append(("入口：圆括号子 shell 整组 > 写 .log 拒（退出码 2）", 2, feed("( cargo test --workspace ) > /tmp/test.log").returncode))
    # 判不了的照常放行、stderr 报一句、记一条检出（结局「只记」）：hook 输入不是 JSON 对象（诊断带钩子名与开头的字节，会话号写读不到）、
    # 判的时候抛异常（TWO_LOGS_GUARD_BREAK 里加一个认不出的项，guard_switch_on 抛 ValueError）。开关 failure-unrecorded 下这两格红
    def entries():
        if not os.path.exists(detections):
            return []
        with open(detections, encoding="utf-8") as handle:
            return [json.loads(line) for line in handle if line.strip()]
    before = len(entries())
    unreadable = [subprocess.run(["bash", guard_script], input=raw, capture_output=True, env=environment, timeout=120)
                  for raw in (b'{"tool_name": "Bash", "tool_input": ', b'["Bash"]')]
    added = entries()[before:]
    results.append(("入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", ([0, 0], True, ["读不到", "读不到"], ["只记", "只记"]),
                    ([run.returncode for run in unreadable],
                     all(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象" in run.stderr.decode("utf-8", "replace") for run in unreadable)
                     and "b'[\"Bash\"]'" in unreadable[1].stderr.decode("utf-8", "replace"),
                     [entry.get("session_id") for entry in added], [entry.get("outcome") for entry in added])))
    before = len(entries())
    # 关守卫项 honor-outside-selftest 与别的项同写会被入口的守卫先拒（第一百二十二节 ②），注入认不出的项时摘掉它
    injected = ",".join([item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item and item != "honor-outside-selftest"] + ["not-a-switch"])
    raising = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise",
                             "tool_input": {"command": "bash .claude/gate.d/doc-text.sh . > /tmp/doc-text.log"}}), capture_output=True, text=True,
                             env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    added = entries()[before:]
    results.append(("入口：判的时候抛异常放行、stderr 报一句、记一条检出", (0, True, ["s-raise"], ["只记"]),
                    (raising.returncode, "没判成" in raising.stderr, [entry.get("session_id") for entry in added], [entry.get("outcome") for entry in added])))
    # 平常跑（不是 --selftest、环境里没有对得上的标记）设了开关：入口退 2，stderr 有一行点名变量、说只在自证里认（门禁 code-tooling 的
    #   break-switch-outside-selftest 格同样这么探）；标记文件写的不是那一句也照拒；带标记的自证子进程照认开关。
    #   开关 honor-outside-selftest 下探的值里带上它，前两格红；这三格的检出另写一份文件
    probe_item = "honor-outside-selftest" if "honor-outside-selftest" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") else "redirect-unchecked"
    without_mark = {key: value for key, value in environment.items() if key != SELFTEST_MARK_VARIABLE}
    outside_detections = os.path.join(work, "outside-selftest-detections.jsonl")
    probed = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                            env=dict(without_mark, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: outside_detections}))
    named = [line for line in probed.stderr.splitlines() if BREAK_VARIABLE_NAME in line and "自证" in line]
    results.append(("入口：平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True), (probed.returncode, bool(named))))
    wrong_mark = os.path.join(work, "wrong-selftest-mark")
    with open(wrong_mark, "w", encoding="utf-8") as handle:
        handle.write("不是自证写的\n")
    listing = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "ls"}})
    wrong = subprocess.run(["bash", guard_script], input=listing, capture_output=True, text=True, timeout=120,
                           env=dict(without_mark, **{BREAK_VARIABLE_NAME: probe_item, SELFTEST_MARK_VARIABLE: wrong_mark, DETECTIONS_VARIABLE: outside_detections}))
    results.append(("入口：标记文件内容对不上照拒", 2, wrong.returncode))
    logged = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh . > /tmp/doc-text.log"}})
    child = subprocess.run(["bash", guard_script], input=logged, capture_output=True, text=True, timeout=120,
                           env=dict(environment, **{BREAK_VARIABLE_NAME: "redirect-unchecked", DETECTIONS_VARIABLE: outside_detections}))
    results.append(("入口：自证起的子进程（带标记）照认弄坏开关", 0, child.returncode))
    # 关守卫项只在它是唯一一项时认：honor-outside-selftest 与别的项同写，不带标记的平常跑与带标记的子进程都退 2、stderr 点名它；
    #   开关 honor-outside-selftest-with-others 设着时，带标记那一次的开关里也写上它（改前的样子：同写照认），那一格红
    guard_and_other = "honor-outside-selftest,redirect-unchecked"
    outside_run = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                 env=dict(without_mark, **{BREAK_VARIABLE_NAME: guard_and_other, DETECTIONS_VARIABLE: outside_detections}))
    outside_named = "honor-outside-selftest" in outside_run.stderr and "唯一一项" in outside_run.stderr
    results.append(("入口：关守卫项与别的项同写：平常跑退 2、点名它只在单独一项时认", (2, True), (outside_run.returncode, outside_named)))
    with_others_on = "honor-outside-selftest-with-others" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    marked_switches = guard_and_other + (",honor-outside-selftest-with-others" if with_others_on else "")
    marked_run = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                env=dict(environment, **{BREAK_VARIABLE_NAME: marked_switches, DETECTIONS_VARIABLE: outside_detections}))
    marked_named = "honor-outside-selftest" in marked_run.stderr and "唯一一项" in marked_run.stderr
    results.append(("入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它", (2, True), (marked_run.returncode, marked_named)))
    return results


def judge_cells(work, hook_directory):
    """[(格名, 期望, 实际)]：内存里的格加另起入口的格。"""
    results = []
    for label, command, code in memory_cells():
        decision = decide_logs({"tool_input": {"command": command}, "cwd": work}, work)
        results.append((label, code, decision.code))
    listed = decide_logs({"tool_input": {"command": "bash .claude/gate.d/doc-text.sh . > /tmp/a.log; cargo test | tee b.log"}, "cwd": work}, work).message or ""
    results.append(("拒绝说明逐条写被盯的命令与目标", True, all(text in listed for text in ("1. 门禁 doc-text.sh：重定向到 /tmp/a.log", "2. cargo test：经 tee 写到 b.log"))))
    results += entry_cells(work, os.path.join(hook_directory, GUARD_SCRIPT_NAME))
    return results


def failed_labels(results):
    return [label for label, want, got in results if want != got]


def selftest(hook_directory):
    load_judging_libraries(hook_directory)
    outside_break = os.environ.get(BREAK_VARIABLE_NAME) or None
    work = tempfile.mkdtemp(prefix="two-logs-guard-")
    try:
        # 入口只在这个标记在、内容对得上时认弄坏开关：自证起的入口都继承它
        mark_path = os.path.join(work, "selftest-mark")
        with open(mark_path, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        os.environ[SELFTEST_MARK_VARIABLE] = mark_path
        results = judge_cells(work, hook_directory)
        failures = failed_labels(results)
        for label, want, got in results:
            if want != got:
                print(f"  ✗ 自检：{label}：应当是 {want!r}，实际 {got!r}")
                print(f"  → 怎么办：照 {label} 那一格的判法改 lib_two_logs_guard.py，再跑 bash .claude/hooks/{GUARD_SCRIPT_NAME} --selftest")
        if outside_break:
            print(f"  （{BREAK_VARIABLE_NAME}={outside_break} 下判了 {len(results)} 格，红 {len(failures)} 格：{'、'.join(failures)}；开关轮不跑）")
            return 1 if failures else 0
        labels = [label for label, _want, _got in results]
        try:
            with open(os.path.join(hook_directory, GUARD_SCRIPT_NAME), encoding="utf-8") as handle:
                header = handle.read()
        except OSError:
            header = ""
        switch_problems = []
        for switch, cells in BREAK_SWITCH_CELLS.items():
            missing = [cell for cell in cells if cell not in labels]
            if missing:
                switch_problems.append(f"开关 {switch} 点名的格不在自证里：{'、'.join(missing)}")
                continue
            written = re.search(r"^#\s+" + re.escape(f"{BREAK_VARIABLE_NAME}={switch}") + r"\s.*\n#\s+红：(.*)$", header, re.M)
            if written is None:
                switch_problems.append(f"开关 {switch} 没写进 {GUARD_SCRIPT_NAME} 的文件头（一行「{BREAK_VARIABLE_NAME}={switch}  <弄坏哪一处>」、下一行「红：…」）")
            elif sorted(re.findall(r"「([^」]*)」", written.group(1))) != sorted(cells):
                switch_problems.append(f"开关 {switch} 文件头写的红格与 BREAK_SWITCH_CELLS 不一致：{sorted(re.findall(r'「([^」]*)」', written.group(1)))!r} / {sorted(cells)!r}")
            os.environ[BREAK_VARIABLE_NAME] = switch
            try:
                broken = judge_cells(work, hook_directory)
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
            print(f"  → 怎么办：照上面点名的格改 {os.path.join(hook_directory, 'lib_two_logs_guard.py')} 的判法（或 {GUARD_SCRIPT_NAME} 的文件头），再单独跑 "
                  f"bash .claude/hooks/{GUARD_SCRIPT_NAME} --selftest 看它转绿")
            return 1
        print(f"  ✓ {GUARD_SCRIPT_NAME} 自检通过：{len(results)} 格（内存里判 {len(memory_cells()) + 1} 格、另起入口 {len(results) - len(memory_cells()) - 1} 格）；"
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
